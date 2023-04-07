use crate::prelude::*;

use super::ipc::*;
use super::request::*;
use super::server::TinyHttpRequestContext;
use crate::cli::args::ChildArgs;
use crate::ffi::set_non_blocking;
use crate::ffi::ExitCode;
use crate::ffi::ExitResult;
use std::net::Ipv4Addr;

static SERVER_STATE: ServerState<TinyHttpRequestContext> = ServerState::new();
static NOTIFY_EXIT: Notify = Notify::new();

pub fn start_child(args: ChildArgs) -> io::Result<ExitResult> {
    // Set the standard input and output to non-blocking mode so reads will correctly not block.
    set_non_blocking(libc::STDIN_FILENO)?;
    set_non_blocking(libc::STDOUT_FILENO)?;

    // Set up all the state.
    let notify_guard = NOTIFY_EXIT.create_guard();

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

    let (ctx_send, ctx_recv) = make_channel();

    // Spawn all the threads
    let handle_request_handle = ThreadHandle::spawn(handle_request_task(ctx_recv, shared));
    let server_recv_handle = ThreadHandle::spawn(server_recv_task(server, ctx_send));

    let child_ipc_result = child_ipc(&SERVER_STATE, io::stdin(), &NOTIFY_EXIT);
    drop(notify_guard);

    // Wait for everything else to settle
    let handle_request_result = handle_request_handle.join();
    let server_recv_result = server_recv_handle.join();
    // And now wire up errors and return.
    handle_request_result?;
    server_recv_result?;
    child_ipc_result?;

    Ok(ExitResult::Code(ExitCode(1)))
}

fn server_recv_task(
    server: tiny_http::Server,
    ctx_send: ChannelSender<TinyHttpRequestContext>,
) -> ThreadTask {
    Box::new(move || {
        let _guard = NOTIFY_EXIT.create_guard();

        while !NOTIFY_EXIT.has_notified() {
            if let Some(request) = server.recv_timeout(Duration::from_secs(1))? {
                let ctx = TinyHttpRequestContext::new(request);
                if matches!(ctx_send.send(ctx), SendResult::Disconnected(_)) {
                    break;
                }
            }
        }

        Ok(())
    })
}

fn handle_request_task(
    ctx_recv: ChannelReceiver<TinyHttpRequestContext>,
    shared: RequestShared<TinyHttpRequestContext, std::io::Stdout>,
) -> ThreadTask {
    Box::new(move || {
        let _guard = NOTIFY_EXIT.create_guard();
        let mut target = 1;

        while !NOTIFY_EXIT.has_notified() {
            let mut duration = Instant::now().saturating_duration_since(shared.initialized);
            if duration.as_secs() >= target {
                target = duration.as_secs().wrapping_add(1);
                duration = Duration::from_secs(1);
                SERVER_STATE.limiter.lock().reap(target);
            }

            match ctx_recv.read_timeout(duration) {
                ReadTimeoutResult::Received(requests) => {
                    for ctx in requests {
                        handle_request(ctx, &shared);
                    }
                }
                ReadTimeoutResult::TimedOut => {}
                ReadTimeoutResult::Disconnected => break,
            }
        }

        Ok(())
    })
}
