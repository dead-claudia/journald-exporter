use crate::prelude::*;

use super::ipc::request_metrics;
use super::ipc::IPCRequester;
use super::limiter::Limiter;
use crate::ffi::ImmutableWrite;
use std::net::Ipv6Addr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub struct RequestShared<C: 'static, W> {
    pub state: &'static ServerState<C>,
    pub output: W,
    pub initialized: Instant,
}

pub struct StaticRequestContext {
    pub authorization: Option<Box<[u8]>>,
    pub received: Instant,
    pub peer_addr: Ipv6Addr,
    pub route: Route,
}

impl RequestContext for StaticRequestContext {
    fn authorization(&self) -> Option<&[u8]> {
        self.authorization.as_deref()
    }
    fn route(&self) -> Route {
        self.route
    }
    fn received(&self) -> Instant {
        self.received
    }
    fn peer_addr(&self) -> Ipv6Addr {
        self.peer_addr
    }
}

pub trait RequestContext {
    fn authorization(&self) -> Option<&[u8]>;
    fn route(&self) -> Route;
    fn received(&self) -> Instant;
    fn peer_addr(&self) -> Ipv6Addr;
}

pub trait ResponseContext {
    fn respond(self, head: &'static ResponseHead, body: &[u8]);
}

pub struct ServerState<C> {
    pub key_set: RwLock<Option<KeySet>>,
    pub ipc_requester: IPCRequester<C>,
    pub limiter: Uncontended<Limiter>,
    pub decoder: Uncontended<ipc::parent::Decoder>,
    pub terminate_notify: Notify,
}

impl<C: ResponseContext> ServerState<C> {
    pub const fn new() -> Self {
        Self {
            ipc_requester: IPCRequester::new(),
            key_set: RwLock::new(None),
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
fn handle_metrics_get<C: ResponseContext + 'static>(
    req: impl RequestContext,
    res: C,
    shared: &RequestShared<C, impl ImmutableWrite>,
) -> Option<C> {
    let Some(auth_header) = req.authorization() else {
        res.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
        return None;
    };

    let Some(rest) = auth_header.strip_prefix(b"Basic ") else {
        res.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
        return None;
    };

    let Ok(rest) = std::str::from_utf8(rest) else {
        res.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
        return None;
    };

    let Ok(mut decoded) = openssl::base64::decode_block(rest) else {
        res.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
        return None;
    };

    let password = match decoded.as_mut_slice() {
        [b'm', b'e', b't', b'r', b'i', b'c', b's', b':', password @ ..] if !password.is_empty() => {
            password
        }
        _ => {
            if decoded.contains(&b':') {
                res.respond(&RESPONSE_FORBIDDEN, &[]);
                return None;
            } else {
                res.respond(&RESPONSE_BAD_AUTH_SYNTAX, &[]);
                return None;
            }
        }
    };

    let guard = shared
        .state
        .key_set
        .read()
        .unwrap_or_else(|e| e.into_inner());

    let Some(key_set) = &*guard else {
        // No need to retain the lock while responding.
        drop(guard);
        res.respond(&RESPONSE_FORBIDDEN, &[]);
        return None;
    };

    if !key_set.check_key(password) {
        // No need to retain the lock while responding.
        drop(guard);
        res.respond(&RESPONSE_FORBIDDEN, &[]);
        return None;
    }

    let diff = req.received().saturating_duration_since(shared.initialized);
    let mut limiter = shared.state.limiter.lock();

    if limiter.check_throttled(diff.as_secs(), req.peer_addr()) {
        drop(limiter);
        res.respond(&RESPONSE_THROTTLED, &[]);
        None
    } else {
        Some(res)
    }
}

pub fn handle_request<C: ResponseContext + 'static>(
    req: impl RequestContext,
    res: C,
    shared: &RequestShared<C, impl ImmutableWrite>,
) {
    if !super::ipc::send_msg(shared, &[ipc::child::TRACK_REQUEST]) {
        res.respond(&RESPONSE_UNAVAILABLE, &[]);
        return;
    }

    match req.route() {
        Route::InvalidMethod => res.respond(&RESPONSE_METHOD_NOT_ALLOWED, &[]),
        Route::InvalidPath => res.respond(&RESPONSE_NOT_FOUND, &[]),
        Route::MetricsGet => {
            if let Some(res) = handle_metrics_get(req, res, shared) {
                request_metrics(res, shared);
            }
        }
    }
}
