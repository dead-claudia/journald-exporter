use crate::prelude::*;

use super::request::RequestContext;
use super::request::ResponseHead;
use super::request::ResponseHeaderTemplate;
use super::request::Route;
use std::net::SocketAddr;
use tiny_http::Method;

pub struct TinyHttpRequestContextInner {
    received: Instant,
    request: tiny_http::Request,
}

pub struct TinyHttpRequestContext(Option<TinyHttpRequestContextInner>);

impl TinyHttpRequestContext {
    pub fn new(request: tiny_http::Request) -> Self {
        Self(Some(TinyHttpRequestContextInner {
            request,
            received: Instant::now(),
        }))
    }

    // This should never panic, as the inner request instance is only settable to `None` on
    // response.
    fn inner(&self) -> &TinyHttpRequestContextInner {
        self.0.as_ref().unwrap()
    }
}

impl RequestContext for TinyHttpRequestContext {
    fn authorization(&self) -> Option<&[u8]> {
        for header in self.inner().request.headers() {
            if header.field.equiv("authorization") {
                return Some(header.value.as_bytes());
            }
        }
        None
    }

    fn route(&self) -> Route {
        match (self.inner().request.method(), self.inner().request.url()) {
            (Method::Get, "/metrics") => Route::MetricsGet,
            (Method::Get | Method::Head, _) => Route::InvalidPath,
            (_, _) => Route::InvalidMethod,
        }
    }

    fn received(&self) -> Instant {
        self.inner().received
    }

    fn peer_addr(&self) -> &SocketAddr {
        self.inner().request.remote_addr().unwrap()
    }

    fn respond(mut self, head: &ResponseHead, body: &[u8]) {
        // SAFETY: `self` is forgotten before this block ends, and there's no other fields in the
        // struct in need of being dropped.
        let inner = self.0.take().unwrap();

        let status = tiny_http::StatusCode(head.status);

        fn single_header(header: &[u8], value: &[u8]) -> Vec<tiny_http::Header> {
            Vec::from_iter([tiny_http::Header::from_bytes(header, value).unwrap()])
        }

        let headers = match head.header_template {
            ResponseHeaderTemplate::Empty => Vec::new(),
            ResponseHeaderTemplate::Metrics => {
                single_header(b"content-type", b"application/openmetrics-text")
            }
            ResponseHeaderTemplate::BadAuthSyntax => {
                single_header(b"www-authenticate", b"Basic realm=\"metrics\"")
            }
            ResponseHeaderTemplate::MethodNotAllowed => single_header(b"allow", b"GET,HEAD"),
            ResponseHeaderTemplate::Disconnect => single_header(b"connection", b"close"),
        };

        let response = tiny_http::Response::new(status, headers, body, Some(body.len()), None);

        // `tiny_http` ignores client closing errors internally. No need to do it here. :-)
        if let Err(e) = inner.request.respond(response) {
            log::warn!(
                "Error while returning response: {}",
                normalize_errno(e, None)
            );
        }
    }
}

impl Drop for TinyHttpRequestContext {
    fn drop(&mut self) {
        // Just warn. The `Drop` impl for `tiny_http::Request` will close the connection anyways.
        if let Some(inner) = self.0.take() {
            log::warn!(
                "Request to {} {} did not receive a response.",
                inner.request.method(),
                inner.request.url(),
            )
        }
    }
}
