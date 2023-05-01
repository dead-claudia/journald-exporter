use crate::prelude::*;

use super::request::RequestContext;
use super::request::RequestShared;
use super::request::ResponseContext;
use crate::child::ipc::child_ipc;
use crate::child::request::handle_request;
use crate::child::request::ResponseHead;
use crate::child::request::ResponseHeaderTemplate;
use crate::child::request::Route;
use crate::child::request::ServerState;
use crate::ffi::Pollable;
use crate::state::ipc::VERSION_BYTES;

//  #     #
//  ##    #  ####  #####    ######  ####  #    # #    # #####     ##### ######  ####  #####  ####
//  # #   # #    #   #      #      #    # #    # ##   # #    #      #   #      #        #   #
//  #  #  # #    #   #      #####  #    # #    # # #  # #    #      #   #####   ####    #    ####
//  #   # # #    #   #      #      #    # #    # #  # # #    #      #   #           #   #        #
//  #    ## #    #   #      #      #    # #    # #   ## #    #      #   #      #    #   #   #    #
//  #     #  ####    #      #       ####   ####  #    # #####       #   ######  ####    #    ####

fn test_not_found(
    target: &'static WriteSpy,
    state: &'static ServerState<SyntheticRequestContext>,
    head: &'static ResponseHead,
    route: Route,
) {
    let shared = make_shared(state, target, &[]);
    let state = Arc::new(SyntheticRequestState::new(route, None));

    target.enqueue_write(Ok(1));

    let context = SyntheticRequestContext(state.clone());
    handle_request(context.clone(), context, &shared);

    let response = state.response.lock().take().expect("No response received");

    assert_eq!(response.head, head);

    target.assert_data_written(&[ipc::child::TRACK_REQUEST]);
}

#[test]
fn handles_an_unknown_method_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    static RESPONSE_HEAD: ResponseHead = ResponseHead {
        status: 405,
        header_template: ResponseHeaderTemplate::MethodNotAllowed,
    };
    test_not_found(&TARGET, &STATE, &RESPONSE_HEAD, Route::InvalidMethod);
}

#[test]
fn handles_an_unknown_path_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    static RESPONSE_HEAD: ResponseHead = ResponseHead {
        status: 404,
        header_template: ResponseHeaderTemplate::Empty,
    };
    test_not_found(&TARGET, &STATE, &RESPONSE_HEAD, Route::InvalidPath);
}

//  #######
//  #        ####  #    # #    # #####     ##### ######  ####  #####  ####
//  #       #    # #    # ##   # #    #      #   #      #        #   #
//  #####   #    # #    # # #  # #    #      #   #####   ####    #    ####
//  #       #    # #    # #  # # #    #      #   #           #   #        #
//  #       #    # #    # #   ## #    #      #   #      #    #   #   #    #
//  #        ####   ####  #    # #####       #   ######  ####    #    ####

#[test]
fn handles_an_unauthorized_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();

    let logger_guard = setup_capture_logger();
    let shared = make_shared(&STATE, &TARGET, &[b"0123456789abcdef"]);
    let terminate_notify = Arc::new(Notify::new());
    let _terminate_guard = terminate_notify.create_guard();

    let state = Arc::new(SyntheticRequestState::new(Route::MetricsGet, None));

    TARGET.enqueue_write(Ok(1));

    let context = SyntheticRequestContext(state.clone());
    handle_request(context.clone(), context, &shared);

    assert!(
        !STATE.ipc_requester.has_requests_pending(),
        "Expected request not to be queued.",
    );

    let response = state.response.lock().take().expect("No response received");

    assert_eq!(
        response,
        SyntheticResponse {
            head: &ResponseHead {
                status: 401,
                header_template: ResponseHeaderTemplate::BadAuthSyntax,
            },
            body: Vec::new(),
        }
    );

    TARGET.assert_data_written(&[ipc::child::TRACK_REQUEST]);
    TARGET.assert_no_calls_remaining();
    logger_guard.expect_logs(&[]);
}

fn test_bad_auth_syntax(
    target: &'static WriteSpy,
    state: &'static ServerState<SyntheticRequestContext>,
    authorization: &'static [u8],
) {
    let logger_guard = setup_capture_logger();
    let shared = make_shared(state, target, &[b"0123456789abcdef"]);
    let terminate_notify = Arc::new(Notify::new());
    let _terminate_guard = terminate_notify.create_guard();

    let request_state = Arc::new(SyntheticRequestState::new(
        Route::MetricsGet,
        Some(authorization),
    ));

    target.enqueue_write(Ok(1));

    let context = SyntheticRequestContext(request_state.clone());
    handle_request(context.clone(), context, &shared);

    assert!(
        !state.ipc_requester.has_requests_pending(),
        "Expected request not to be queued.",
    );

    let response = request_state
        .response
        .lock()
        .take()
        .expect("No response received");

    assert_eq!(
        response,
        SyntheticResponse {
            head: &ResponseHead {
                status: 401,
                header_template: ResponseHeaderTemplate::BadAuthSyntax,
            },
            body: Vec::new()
        }
    );

    target.assert_data_written(&[ipc::child::TRACK_REQUEST]);
    target.assert_no_calls_remaining();
    logger_guard.expect_logs(&[]);
}

#[test]
fn handles_empty_authorization_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    test_bad_auth_syntax(&TARGET, &STATE, b"");
}

#[test]
fn handles_non_basic_authorization_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    test_bad_auth_syntax(&TARGET, &STATE, b"NotBasic abc123");
}

#[test]
fn handles_basic_no_space_authorization_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    test_bad_auth_syntax(&TARGET, &STATE, b"Basicabc123");
}

#[test]
fn handles_empty_base64_authorization_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    test_bad_auth_syntax(&TARGET, &STATE, b"Basic ");
}

#[test]
fn handles_whitespace_base64_authorization_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    test_bad_auth_syntax(&TARGET, &STATE, b"Basic           ");
}

#[test]
fn handles_invalid_base64_authorization_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    test_bad_auth_syntax(&TARGET, &STATE, b"Basic ???");
}

#[test]
fn handles_wrong_username_missing_password_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    // Decoded: `bad`
    test_bad_auth_syntax(&TARGET, &STATE, b"Basic YmFk");
}

#[test]
fn handles_right_username_missing_password_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    // Decoded: `metrics`
    test_bad_auth_syntax(&TARGET, &STATE, b"Basic bWV0cmljcw==");
}

fn test_bad_auth_credentials(
    target: &'static WriteSpy,
    state: &'static ServerState<SyntheticRequestContext>,
    authorization: &'static [u8],
) {
    let logger_guard = setup_capture_logger();
    let shared = make_shared(state, target, &[b"0123456789abcdef"]);
    let terminate_notify = Arc::new(Notify::new());
    let _terminate_guard = terminate_notify.create_guard();

    let request_state = Arc::new(SyntheticRequestState::new(
        Route::MetricsGet,
        Some(authorization),
    ));

    target.enqueue_write(Ok(1));

    let context = SyntheticRequestContext(request_state.clone());
    handle_request(context.clone(), context, &shared);

    assert!(
        !state.ipc_requester.has_requests_pending(),
        "Expected request not to be queued.",
    );

    let response = request_state
        .response
        .lock()
        .take()
        .expect("No response received");

    assert_eq!(
        response,
        SyntheticResponse {
            head: &ResponseHead {
                status: 403,
                header_template: ResponseHeaderTemplate::Empty,
            },
            body: Vec::new()
        }
    );

    target.assert_data_written(&[ipc::child::TRACK_REQUEST]);
    target.assert_no_calls_remaining();
    logger_guard.expect_logs(&[]);
}

#[test]
fn handles_bad_username_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    // Decoded: `bad:0123456789abcdef`
    test_bad_auth_credentials(&TARGET, &STATE, b"Basic YmFkOjAxMjM0NTY3ODlhYmNkZWY=");
}

#[test]
fn handles_right_username_empty_password_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    // Decoded: `metrics:`
    test_bad_auth_credentials(&TARGET, &STATE, b"Basic bWV0cmljczo=");
}

#[test]
fn handles_bad_password_in_metrics_get_request() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
    // Decoded: `metrics:000044448888cccc`
    test_bad_auth_credentials(&TARGET, &STATE, b"Basic bWV0cmljczowMDAwNDQ0NDg4ODhjY2Nj");
}

#[test]
fn handles_an_authorized_metrics_get_request() {
    #[rustfmt::skip]
    static IPC_RECV: &[u8] = &[
        VERSION_BYTES[0], VERSION_BYTES[1], VERSION_BYTES[2], VERSION_BYTES[3],
        0x00, // Operation ID
        0x10, 0x00, 0x00, 0x00, // Data length (16)
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', // Data
        b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
    ];

    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();

    let logger_guard = setup_capture_logger();
    let shared = make_shared(&STATE, &TARGET, &[b"0123456789abcdef"]);
    let terminate_notify = Arc::new(Notify::new());
    let _terminate_guard = terminate_notify.create_guard();

    TARGET.enqueue_write(Ok(1));
    TARGET.enqueue_write(Ok(1));

    // Decoded: `metrics:0123456789abcdef`
    let state = Arc::new(SyntheticRequestState::new(
        Route::MetricsGet,
        Some(b"Basic bWV0cmljczowMTIzNDU2Nzg5YWJjZGVm"),
    ));

    let context = SyntheticRequestContext(state.clone());
    handle_request(context.clone(), context, &shared);

    assert!(
        STATE.ipc_requester.has_requests_pending(),
        "Expected request to be queued.",
    );

    assert_result_eq(resume_request(&STATE, &terminate_notify, IPC_RECV), Ok(()));

    let response = state.response.lock().take().expect("No response received");

    assert_eq!(
        response,
        SyntheticResponse {
            head: &ResponseHead {
                status: 200,
                header_template: ResponseHeaderTemplate::Metrics,
            },
            body: b"0123456789abcdef".to_vec()
        }
    );

    TARGET.assert_data_written(&[ipc::child::TRACK_REQUEST, ipc::child::REQUEST_METRICS]);
    TARGET.assert_no_calls_remaining();
    logger_guard.expect_logs(&[]);
}

#[test]
fn trims_whitespace_in_metrics_get_request() {
    #[rustfmt::skip]
    static IPC_RECV: &[u8] = &[
        VERSION_BYTES[0], VERSION_BYTES[1], VERSION_BYTES[2], VERSION_BYTES[3],
        0x00, // Operation ID
        0x10, 0x00, 0x00, 0x00, // Data length (16)
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', // Data
        b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
    ];

    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();

    let logger_guard = setup_capture_logger();
    let shared = make_shared(&STATE, &TARGET, &[b"0123456789abcdef"]);
    let terminate_notify = Arc::new(Notify::new());
    let _terminate_guard = terminate_notify.create_guard();

    TARGET.enqueue_write(Ok(1));
    TARGET.enqueue_write(Ok(1));

    // Decoded: `metrics:0123456789abcdef`
    let state = Arc::new(SyntheticRequestState::new(
        Route::MetricsGet,
        Some(b"Basic    bWV0cmljczowMTIzNDU2Nzg5YWJjZGVm    "),
    ));

    let context = SyntheticRequestContext(state.clone());
    handle_request(context.clone(), context, &shared);

    assert!(
        STATE.ipc_requester.has_requests_pending(),
        "Expected request to be queued.",
    );

    assert_result_eq(resume_request(&STATE, &terminate_notify, IPC_RECV), Ok(()));

    let response = state.response.lock().take().expect("No response received");

    assert_eq!(
        response,
        SyntheticResponse {
            head: &ResponseHead {
                status: 200,
                header_template: ResponseHeaderTemplate::Metrics,
            },
            body: b"0123456789abcdef".to_vec()
        }
    );

    TARGET.assert_data_written(&[ipc::child::TRACK_REQUEST, ipc::child::REQUEST_METRICS]);
    TARGET.assert_no_calls_remaining();
    logger_guard.expect_logs(&[]);
}

#[test]
fn handles_metrics_get_request_disconnects_early() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();

    let logger_guard = setup_capture_logger();
    let shared = make_shared(&STATE, &TARGET, &[b"0123456789abcdef"]);
    let terminate_notify = Arc::new(Notify::new());
    terminate_notify.notify();

    TARGET.enqueue_write(Err(libc::EPIPE));

    // Decoded: `metrics:0123456789abcdef`
    let state = Arc::new(SyntheticRequestState::new(
        Route::MetricsGet,
        Some(b"Basic bWV0cmljczowMTIzNDU2Nzg5YWJjZGVm"),
    ));

    let context = SyntheticRequestContext(state.clone());
    handle_request(context.clone(), context, &shared);

    assert!(
        !STATE.ipc_requester.has_requests_pending(),
        "Expected request not to be queued.",
    );

    let response = state.response.lock().take().expect("No response received");

    assert_eq!(
        response,
        SyntheticResponse {
            head: &ResponseHead {
                status: 503,
                header_template: ResponseHeaderTemplate::Disconnect,
            },
            body: Vec::new(),
        }
    );

    TARGET.assert_data_written(&[]);
    TARGET.assert_no_calls_remaining();
    logger_guard.expect_logs(&["EPIPE: Broken pipe"]);
}

#[test]
fn handles_metrics_get_request_disconnects_late() {
    static TARGET: WriteSpy = WriteSpy::new("TARGET");
    static STATE: ServerState<SyntheticRequestContext> = ServerState::new();

    let logger_guard = setup_capture_logger();
    let shared = make_shared(&STATE, &TARGET, &[b"0123456789abcdef"]);
    let terminate_notify = Arc::new(Notify::new());
    let terminate_guard = terminate_notify.create_guard();

    // Decoded: `metrics:0123456789abcdef`
    let state = Arc::new(SyntheticRequestState::new(
        Route::MetricsGet,
        Some(b"Basic bWV0cmljczowMTIzNDU2Nzg5YWJjZGVm"),
    ));

    TARGET.enqueue_write(Ok(1));
    TARGET.enqueue_write(Ok(1));

    let context = SyntheticRequestContext(state.clone());
    handle_request(context.clone(), context, &shared);

    assert!(
        STATE.ipc_requester.has_requests_pending(),
        "Expected request to be queued.",
    );

    drop(terminate_guard);
    assert_result_eq(
        resume_request(&STATE, &terminate_notify, Disconnected),
        Ok(()),
    );

    let response = state.response.lock().take().expect("No response received");

    assert_eq!(
        response,
        SyntheticResponse {
            head: &ResponseHead {
                status: 503,
                header_template: ResponseHeaderTemplate::Disconnect,
            },
            body: Vec::new()
        }
    );

    TARGET.assert_data_written(&[ipc::child::TRACK_REQUEST, ipc::child::REQUEST_METRICS]);
    TARGET.assert_no_calls_remaining();
    logger_guard.expect_logs(&[]);
}

//  #     #
//  #     # ###### #      #####  ###### #####   ####
//  #     # #      #      #    # #      #    # #
//  ####### #####  #      #    # #####  #    #  ####
//  #     # #      #      #####  #      #####       #
//  #     # #      #      #      #      #   #  #    #
//  #     # ###### ###### #      ###### #    #  ####

fn make_shared(
    state: &'static ServerState<SyntheticRequestContext>,
    output: &'static WriteSpy,
    keys: &[&[u8]],
) -> RequestShared<SyntheticRequestContext, &'static WriteSpy> {
    let mut guard = state.key_set.write().unwrap_or_else(|e| e.into_inner());
    *guard = if keys.is_empty() {
        None
    } else {
        Some(KeySet::build(keys))
    };

    RequestShared {
        state,
        output,
        initialized: Instant::now(),
    }
}

#[derive(Debug, PartialEq)]
struct SyntheticResponse {
    head: &'static ResponseHead,
    body: Vec<u8>,
}

struct SyntheticRequestState {
    route: Route,
    authorization: Option<&'static [u8]>,
    received: Instant,
    response: Uncontended<Option<SyntheticResponse>>,
}

impl SyntheticRequestState {
    fn new(route: Route, authorization: Option<&'static [u8]>) -> Self {
        Self {
            route,
            authorization,
            response: Uncontended::new(None),
            received: Instant::now(),
        }
    }
}

#[derive(Clone)]
struct SyntheticRequestContext(Arc<SyntheticRequestState>);

impl RequestContext for SyntheticRequestContext {
    fn authorization(&self) -> Option<&[u8]> {
        self.0.authorization
    }

    fn route(&self) -> Route {
        self.0.route
    }

    fn received(&self) -> Instant {
        self.0.received
    }

    fn peer_addr(&self) -> std::net::Ipv6Addr {
        std::net::Ipv6Addr::LOCALHOST
    }
}

impl ResponseContext for SyntheticRequestContext {
    fn respond(self, head: &'static ResponseHead, body: &[u8]) {
        let mut guard = self.0.response.lock();

        if guard.is_some() {
            panic!("Response already returned!");
        }

        *guard = Some(SyntheticResponse {
            head,
            body: body.to_vec(),
        })
    }
}

struct Disconnected;

impl Read for Disconnected {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(Error::from_raw_os_error(libc::EPIPE))
    }
}

impl Pollable for Disconnected {
    fn poll(
        &self,
        _: crate::ffi::PollFlags,
        _: Option<Duration>,
    ) -> io::Result<crate::ffi::PollResult> {
        Err(Error::from_raw_os_error(libc::EPIPE))
    }
}

fn resume_request(
    state: &ServerState<SyntheticRequestContext>,
    terminate_notify: &Arc<Notify>,
    ipc_recv: impl Read + Pollable,
) -> io::Result<()> {
    // This is okay since there's nothing else that could theoretically resume the
    // request (and thus cause a race condition).

    let child_ipc_terminate_handle = {
        let terminate_notify = terminate_notify.clone();
        ThreadHandle::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            terminate_notify.notify();
            Ok(())
        })
    };

    child_ipc(state, ipc_recv, terminate_notify)?;
    child_ipc_terminate_handle.join()?;
    Ok(())
}
