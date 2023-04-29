use crate::prelude::*;

use super::ipc::request_metrics;
use super::ipc::IPCRequester;
use super::limiter::Limiter;
use crate::ffi::ImmutableWrite;
use base64::engine::general_purpose::STANDARD_NO_PAD as ENGINE;
use base64::engine::Engine as _;
use std::net::SocketAddr;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Clone, Copy))]
pub enum Route {
    InvalidMethod,
    InvalidPath,
    MetricsGet,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResponseHeaderTemplate {
    Empty,
    Metrics,
    BadAuthSyntax,
    MethodNotAllowed,
    Disconnect,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResponseHead {
    pub status: u16,
    pub header_template: ResponseHeaderTemplate,
}

pub struct RequestShared<C: RequestContext + 'static, W: ImmutableWrite> {
    pub state: &'static ServerState<C>,
    pub output: W,
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

pub static RESPONSE_SERVER_ERROR: ResponseHead = ResponseHead {
    status: 500,
    header_template: ResponseHeaderTemplate::Empty,
};

pub static RESPONSE_UNAVAILABLE: ResponseHead = ResponseHead {
    status: 503,
    header_template: ResponseHeaderTemplate::Disconnect,
};

// Very simplistic parsing. The username's hard-coded as it's just easier that way.
fn handle_metrics_get<C: RequestContext + 'static>(
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
    if !super::ipc::send_msg(shared, &[ipc::child::TRACK_REQUEST]) {
        ctx.respond(&RESPONSE_UNAVAILABLE, &[]);
        return;
    }

    match ctx.route() {
        Route::InvalidMethod => ctx.respond(&RESPONSE_METHOD_NOT_ALLOWED, &[]),
        Route::InvalidPath => ctx.respond(&RESPONSE_NOT_FOUND, &[]),
        Route::MetricsGet => {
            if let Some(ctx) = handle_metrics_get(ctx, shared) {
                request_metrics(ctx, shared);
            }
        }
    }
}
