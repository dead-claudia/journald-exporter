use crate::prelude::*;

use super::ipc::*;
use super::request::*;
use super::server::TinyHttpRequestContext;
use super::PENDING_REQUEST_CAPACITY;
use crate::cli::args::ChildArgs;
use crate::ffi::set_non_blocking;
use crate::ffi::ExitCode;
use crate::ffi::ExitResult;
use std::net::Ipv4Addr;
use std::net::TcpListener;
use std::os::unix::prelude::OsStringExt;

static SERVER_STATE: ServerState<TinyHttpRequestContext> = ServerState::new();
static REQUEST_CHANNEL: Channel<TinyHttpRequestContext, PENDING_REQUEST_CAPACITY> = Channel::new();

fn cli_invariant_failed(s: &str) -> ! {
    let mut stderr = std::io::stderr().lock();
    let mut buf = s.as_bytes();
    while !buf.is_empty() {
        match stderr.write(buf) {
            Ok(size) => buf = &buf[size..],
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => {}
                ErrorKind::Interrupted => {}
                _ => std::process::abort(),
            },
        }
    }
    drop(stderr.flush());
    std::process::abort();
}

pub fn start_child(args: ChildArgs) -> io::Result<ExitResult> {
    // Set the standard input and output to non-blocking mode so reads will correctly not block.
    set_non_blocking(libc::STDIN_FILENO);
    set_non_blocking(libc::STDOUT_FILENO);

    // Set up all the state.
    let channel_guard = REQUEST_CHANNEL.close_guard();

    let shared = RequestShared {
        state: &SERVER_STATE,
        output: io::stdout(),
        initialized: Instant::now(),
    };

    let tls = match (
        std::env::var_os("TLS_CERTIFICATE"),
        std::env::var_os("TLS_PRIVATE_KEY"),
    ) {
        (Some(certificate), Some(private_key)) => Some(tiny_http::SslConfig {
            certificate: certificate.into_vec(),
            private_key: private_key.into_vec(),
        }),
        (None, None) => None,
        (None, Some(_)) => cli_invariant_failed("Received private key but not certificate.\n"),
        (Some(_), None) => cli_invariant_failed("Received certificate but not private key.\n"),
    };

    let listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, args.port.into())) {
        Ok(listener) => listener,
        Err(e) if e.kind() == ErrorKind::AddrInUse => {
            return Err(error!(
                ErrorKind::AddrInUse,
                "TCP port {} is already in use.", args.port
            ))
        }
        Err(e) => return Err(e),
    };

    log::info!("Server listener bound at port {}.", args.port);

    let server = match tiny_http::Server::from_listener(listener, tls) {
        Ok(server) => server,
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
    };

    static INITIAL_BYTES: &[u8] = &[
        ipc::VERSION_BYTES[0],
        ipc::VERSION_BYTES[1],
        ipc::VERSION_BYTES[2],
        ipc::VERSION_BYTES[3],
        ipc::child::REQUEST_KEY,
    ];

    match try_write(
        &SERVER_STATE.terminate_notify,
        &shared.output,
        INITIAL_BYTES,
    ) {
        WriteOutputRequestResult::Written => {}
        WriteOutputRequestResult::Terminated => return Ok(ExitResult::Code(ExitCode(1))),
        WriteOutputRequestResult::Err(e) => return Err(e),
    }

    // Spawn all the threads
    let handle_request_handle = ThreadHandle::spawn(handle_request_task(shared));
    let server_recv_handle = ThreadHandle::spawn(server_recv_task(server));

    log::info!("Child IPC ready.");

    let child_ipc_result = child_ipc(&SERVER_STATE, io::stdin(), REQUEST_CHANNEL.close_notify());
    drop(channel_guard);

    // Wait for everything else to settle
    let handle_request_result = handle_request_handle.join();
    let server_recv_result = server_recv_handle.join();
    // And now wire up errors and return.
    handle_request_result?;
    server_recv_result?;
    child_ipc_result?;

    Ok(ExitResult::Code(ExitCode(1)))
}

fn server_recv_task(server: tiny_http::Server) -> impl FnOnce() -> io::Result<()> + Send {
    move || {
        let _guard = REQUEST_CHANNEL.close_guard();

        log::info!("Server accepting connections.");

        while !REQUEST_CHANNEL.has_closed() {
            if let Some(request) = server.recv_timeout(Duration::from_secs(1))? {
                if let Err(ctx) = REQUEST_CHANNEL.send(TinyHttpRequestContext::new(request)) {
                    ctx.respond(&RESPONSE_UNAVAILABLE, &[]);
                }
            }
        }

        Ok(())
    }
}

fn handle_request_task(
    shared: RequestShared<TinyHttpRequestContext, std::io::Stdout>,
) -> impl FnOnce() -> io::Result<()> + Send {
    move || {
        let _guard = REQUEST_CHANNEL.close_guard();
        let mut target = 1;

        log::info!("Server ready to process requests.");

        while !REQUEST_CHANNEL.has_closed() {
            let mut duration = Instant::now().saturating_duration_since(shared.initialized);
            if duration.as_secs() >= target {
                target = duration.as_secs().wrapping_add(1);
                duration = Duration::from_secs(1);
                SERVER_STATE.limiter.lock().reap(target);
            }

            if let Some(requests) = REQUEST_CHANNEL.read_timeout(duration) {
                for ctx in requests.into_iter() {
                    handle_request(ctx, &shared);
                }
            }
        }

        Ok(())
    }
}
