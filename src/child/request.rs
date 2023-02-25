use crate::prelude::*;

use super::ipc::request_metrics;
use super::ipc::IPCRequester;
use super::limiter::Limiter;
use crate::ffi::ImmutableWrite;
use base64::engine::general_purpose::STANDARD_NO_PAD as ENGINE;
use base64::engine::Engine as _;
use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    InvalidMethod,
    InvalidPath,
    MetricsGet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseHeaderTemplate {
    Empty,
    Metrics,
    BadAuthSyntax,
    MethodNotAllowed,
    Disconnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponseHead {
    pub status: u16,
    pub header_template: ResponseHeaderTemplate,
}

pub struct RequestShared<C: RequestContext + 'static, W: ImmutableWrite> {
    pub state: &'static ServerState<C>,
    pub output: W,
    pub environment: PromEnvironment,
    pub initialized: Instant,
}

pub trait RequestContext: Sized {
    fn authorization(&self) -> Option<&[u8]>;
    fn route(&self) -> Route;
    fn received(&self) -> Instant;
    fn peer_addr(&self) -> &SocketAddr;
    fn respond(self, head: &'static ResponseHead, body: &[u8]);
}

pub struct ServerState<C: RequestContext> {
    pub key_set: RwLock<KeySet>,
    pub ipc_requester: IPCRequester<C>,
    pub limiter: Uncontended<Limiter>,
    pub decoder: Uncontended<ipc::parent::Decoder>,
    pub terminate_notify: Notify,
}

impl<C: RequestContext> ServerState<C> {
    pub const fn new() -> Self {
        Self {
            ipc_requester: IPCRequester::new(),
            key_set: RwLock::new(KeySet::empty()),
            limiter: Uncontended::new(Limiter::new()),
            decoder: Uncontended::new(ipc::parent::Decoder::new()),
            terminate_notify: Notify::new(),
        }
    }
}

pub static RESPONSE_OK_METRICS: ResponseHead = ResponseHead {
    status: 200,
    header_template: ResponseHeaderTemplate::Metrics,
};

pub static RESPONSE_BAD_AUTH_SYNTAX: ResponseHead = ResponseHead {
    status: 401,
    header_template: ResponseHeaderTemplate::BadAuthSyntax,
};

pub static RESPONSE_FORBIDDEN: ResponseHead = ResponseHead {
    status: 403,
    header_template: ResponseHeaderTemplate::Empty,
};

pub static RESPONSE_THROTTLED: ResponseHead = ResponseHead {
    status: 429,
    header_template: ResponseHeaderTemplate::Empty,
};

pub static RESPONSE_METHOD_NOT_ALLOWED: ResponseHead = ResponseHead {
    status: 405,
    header_template: ResponseHeaderTemplate::MethodNotAllowed,
};

pub static RESPONSE_NOT_FOUND: ResponseHead = ResponseHead {
    status: 404,
    header_template: ResponseHeaderTemplate::Empty,
};

pub static RESPONSE_UNAVAILABLE: ResponseHead = ResponseHead {
    status: 503,
    header_template: ResponseHeaderTemplate::Disconnect,
};

// Very simplistic parsing. The username's hard-coded as it's just easier that way.
fn check_authorization<C: RequestContext + 'static>(
    ctx: C,
    shared: &RequestShared<C, impl ImmutableWrite>,
) -> Option<C> {
    let Some(auth_header) = ctx.authorization() else {
        ctx.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
        return None;
    };

    let Some(rest) = auth_header.strip_prefix(b"Basic ") else {
        ctx.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
        return None;
    };

    let rest = trim_auth_token(rest);

    // 0 = empty
    // 1 = invalid Base64
    if rest.len() <= 1 {
        ctx.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
        return None;
    }

    let Ok(decoded) = ENGINE.decode(rest) else {
        ctx.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
        return None;
    };

    let Some(password) = decoded.strip_prefix(b"metrics:") else {
        // Require a username, but treat passwords as optional for the purpose of authentication.
        if matches!(decoded.first(), None | Some(&b':')) {
            ctx.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
            return None;
        } else {
            ctx.respond(&RESPONSE_FORBIDDEN, &[]);
            return None;
        }
    };

    if password.is_empty() {
        ctx.respond(&RESPONSE_FORBIDDEN, &[]);
        return None;
    }

    let guard = shared
        .state
        .key_set
        .read()
        .unwrap_or_else(|e| e.into_inner());

    if !guard.check_key(password) {
        // No need to retain the lock while responding.
        drop(guard);
        ctx.respond(&RESPONSE_FORBIDDEN, &[]);
        return None;
    }

    let diff = ctx.received().saturating_duration_since(shared.initialized);
    let mut limiter = shared.state.limiter.lock();

    if limiter.check_throttled(diff.as_secs(), ctx.peer_addr().ip()) {
        drop(limiter);
        ctx.respond(&RESPONSE_THROTTLED, &[]);
        None
    } else {
        Some(ctx)
    }
}

pub fn handle_request<C: RequestContext + 'static>(
    ctx: C,
    shared: &RequestShared<C, impl ImmutableWrite>,
) {
    match ctx.route() {
        Route::InvalidMethod => ctx.respond(&RESPONSE_METHOD_NOT_ALLOWED, &[]),
        Route::InvalidPath => ctx.respond(&RESPONSE_NOT_FOUND, &[]),
        Route::MetricsGet => {
            if let Some(ctx) = check_authorization(ctx, shared) {
                request_metrics(ctx, shared);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::child::ipc::child_ipc;
    use crate::ffi::Pollable;
    use crate::state::ipc::VERSION_BYTES;
    use std::time::SystemTime;

    //  #     #
    //  ##    #  ####  #####    ######  ####  #    # #    # #####     ##### ######  ####  #####  ####
    //  # #   # #    #   #      #      #    # #    # ##   # #    #      #   #      #        #   #
    //  #  #  # #    #   #      #####  #    # #    # # #  # #    #      #   #####   ####    #    ####
    //  #   # # #    #   #      #      #    # #    # #  # # #    #      #   #           #   #        #
    //  #    ## #    #   #      #      #    # #    # #   ## #    #      #   #      #    #   #   #    #
    //  #     #  ####    #      #       ####   ####  #    # #####       #   ######  ####    #    ####

    #[test]
    fn handles_an_unknown_method_request() {
        static T: NotFound = NotFound::new(
            Route::InvalidMethod,
            405,
            ResponseHeaderTemplate::MethodNotAllowed,
        );
        T.run_test();
    }

    #[test]
    fn handles_an_unknown_path_request() {
        static T: NotFound = NotFound::new(Route::InvalidPath, 404, ResponseHeaderTemplate::Empty);
        T.run_test();
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

        handle_request(SyntheticRequestContext(state.clone()), &shared);

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

        TARGET.assert_data_written(&[]);
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

        handle_request(SyntheticRequestContext(request_state.clone()), &shared);

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

        target.assert_data_written(&[]);
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

        handle_request(SyntheticRequestContext(request_state.clone()), &shared);

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

        target.assert_data_written(&[]);
        target.assert_no_calls_remaining();
        logger_guard.expect_logs(&[]);
    }

    #[test]
    fn handles_wrong_username_missing_password_in_metrics_get_request() {
        static TARGET: WriteSpy = WriteSpy::new("TARGET");
        static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
        // Decoded: `bad`
        test_bad_auth_credentials(&TARGET, &STATE, b"Basic YmFk");
    }

    #[test]
    fn handles_bad_username_in_metrics_get_request() {
        static TARGET: WriteSpy = WriteSpy::new("TARGET");
        static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
        // Decoded: `bad:0123456789abcdef`
        test_bad_auth_credentials(&TARGET, &STATE, b"Basic YmFkOjAxMjM0NTY3ODlhYmNkZWY=");
    }

    #[test]
    fn handles_right_username_missing_password_in_metrics_get_request() {
        static TARGET: WriteSpy = WriteSpy::new("TARGET");
        static STATE: ServerState<SyntheticRequestContext> = ServerState::new();
        // Decoded: `metrics`
        test_bad_auth_credentials(&TARGET, &STATE, b"Basic bWV0cmljcw==");
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

        TARGET.enqueue_write_ok(1);
        TARGET.enqueue_write_ok(1);

        // Decoded: `metrics:0123456789abcdef`
        let state = Arc::new(SyntheticRequestState::new(
            Route::MetricsGet,
            Some(b"Basic bWV0cmljczowMTIzNDU2Nzg5YWJjZGVm"),
        ));

        handle_request(SyntheticRequestContext(state.clone()), &shared);

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

        TARGET.enqueue_write_ok(1);
        TARGET.enqueue_write_ok(1);

        // Decoded: `metrics:0123456789abcdef`
        let state = Arc::new(SyntheticRequestState::new(
            Route::MetricsGet,
            Some(b"Basic    bWV0cmljczowMTIzNDU2Nzg5YWJjZGVm    "),
        ));

        handle_request(SyntheticRequestContext(state.clone()), &shared);

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
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn handles_metrics_get_request_disconnects_early() {
        static TARGET: WriteSpy = WriteSpy::new("TARGET");
        static STATE: ServerState<SyntheticRequestContext> = ServerState::new();

        let logger_guard = setup_capture_logger();
        let shared = make_shared(&STATE, &TARGET, &[b"0123456789abcdef"]);
        let terminate_notify = Arc::new(Notify::new());
        terminate_notify.notify();

        TARGET.enqueue_write_err(libc::EPIPE);

        // Decoded: `metrics:0123456789abcdef`
        let state = Arc::new(SyntheticRequestState::new(
            Route::MetricsGet,
            Some(b"Basic bWV0cmljczowMTIzNDU2Nzg5YWJjZGVm"),
        ));

        handle_request(SyntheticRequestContext(state.clone()), &shared);

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

        TARGET.enqueue_write_ok(1);
        TARGET.enqueue_write_ok(1);

        handle_request(SyntheticRequestContext(state.clone()), &shared);

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
        *guard = KeySet::new(
            keys.iter()
                .map(|k| Key::from_hex(k).unwrap())
                .collect::<Vec<_>>()
                .into(),
        );

        RequestShared {
            state,
            output,
            environment: PromEnvironment {
                created: SystemTime::UNIX_EPOCH + Duration::from_millis(123),
            },
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

        fn peer_addr(&self) -> &SocketAddr {
            static TEST_ADDR: OnceCell<SocketAddr> = OnceCell::new();
            TEST_ADDR.get_or_init(|| SocketAddr::new(std::net::Ipv4Addr::LOCALHOST.into(), 12345))
        }

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

    struct NotFound {
        state: ServerState<SyntheticRequestContext>,
        target: WriteSpy,
        route: Route,
        status: u16,
        header_template: ResponseHeaderTemplate,
    }

    impl NotFound {
        const fn new(
            route: Route,
            status: u16,
            header_template: ResponseHeaderTemplate,
        ) -> NotFound {
            NotFound {
                state: ServerState::new(),
                target: WriteSpy::new("TARGET"),
                route,
                status,
                header_template,
            }
        }

        fn run_test(&'static self) {
            let shared = make_shared(&self.state, &self.target, &[]);
            let state = Arc::new(SyntheticRequestState::new(self.route, None));

            handle_request(SyntheticRequestContext(state.clone()), &shared);

            let response = state.response.lock().take().expect("No response received");

            assert_eq!(response.head.status, self.status);
            assert_eq!(response.head.header_template, self.header_template);

            self.target.assert_data_written(b"");
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
            ThreadHandle::spawn(Box::new(move || {
                std::thread::sleep(Duration::from_millis(10));
                terminate_notify.notify();
                Ok(())
            }))
        };

        child_ipc(state, ipc_recv, terminate_notify)?;
        child_ipc_terminate_handle.join()?;
        Ok(())
    }
}
