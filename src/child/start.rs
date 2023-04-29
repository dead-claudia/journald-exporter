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

static SERVER_STATE: ServerState<TinyHttpRequestContext> = ServerState::new();
static REQUEST_CHANNEL: Channel<TinyHttpRequestContext, PENDING_REQUEST_CAPACITY> = Channel::new();

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

    let server = match tiny_http::Server::http((Ipv4Addr::UNSPECIFIED, args.port.into())) {
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

fn server_recv_task(server: tiny_http::Server) -> ThreadTask {
    Box::new(move || {
        let _guard = REQUEST_CHANNEL.close_guard();

        while !REQUEST_CHANNEL.has_closed() {
            if let Some(request) = server.recv_timeout(Duration::from_secs(1))? {
                if let Err(ctx) = REQUEST_CHANNEL.send(TinyHttpRequestContext::new(request)) {
                    ctx.respond(&RESPONSE_UNAVAILABLE, &[]);
                }
            }
        }

        Ok(())
    })
}

fn handle_request_task(
    shared: RequestShared<TinyHttpRequestContext, std::io::Stdout>,
) -> ThreadTask {
    Box::new(move || {
        let _guard = REQUEST_CHANNEL.close_guard();
        let mut target = 1;

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
    })
}
