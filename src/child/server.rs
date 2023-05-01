use crate::prelude::*;

use super::request::ResponseContext;
use super::request::ResponseHead;
use super::request::ResponseHeaderTemplate;
use super::request::Route;
use super::request::StaticRequestContext;
use std::net::SocketAddr;
use tiny_http::Method;

pub fn build_request_context(
    received: Instant,
    request: &tiny_http::Request,
) -> StaticRequestContext {
    let mut authorization = None;
    let peer_addr = match request.remote_addr() {
        None => unreachable!(),
        Some(SocketAddr::V4(v4)) => v4.ip().to_ipv6_mapped(),
        Some(SocketAddr::V6(v6)) => *v6.ip(),
    };

    for header in request.headers() {
        if header.field.equiv("authorization") {
            authorization = Some(header.value.as_bytes().into());
        }
    }

    let route = match (request.method(), request.url()) {
        (Method::Get, "/metrics") => Route::MetricsGet,
        (Method::Get | Method::Head, _) => Route::InvalidPath,
        (_, _) => Route::InvalidMethod,
    };

    StaticRequestContext {
        authorization,
        received,
        peer_addr,
        route,
    }
}

pub fn respond(request: tiny_http::Request, head: &ResponseHead, body: &[u8]) {
    let status = tiny_http::StatusCode(head.status);

    fn single_header(header: &[u8], value: &[u8]) -> Option<Vec<tiny_http::Header>> {
        let mut result = try_new_dynamic_vec(1)?;
        result.push(tiny_http::Header::from_bytes(header, value).unwrap());
        Some(result)
    }

    let headers = match head.header_template {
        ResponseHeaderTemplate::Empty => Some(Vec::new()),
        ResponseHeaderTemplate::Metrics => {
            single_header(b"content-type", b"application/openmetrics-text")
        }
        ResponseHeaderTemplate::BadAuthSyntax => {
            single_header(b"www-authenticate", b"Basic realm=\"metrics\"")
        }
        ResponseHeaderTemplate::MethodNotAllowed => single_header(b"allow", b"GET,HEAD"),
        ResponseHeaderTemplate::Disconnect => single_header(b"connection", b"close"),
    };

    let headers = match headers {
        Some(headers) => headers,
        None => std::panic::panic_any("Unable to allocate memory for headers!"),
    };

    let response = tiny_http::Response::new(status, headers, body, Some(body.len()), None);

    // `tiny_http` ignores client closing errors internally. No need to do it here. :-)
    if let Err(e) = request.respond(response) {
        log::warn!(
            "Error while returning response: {}",
            normalize_errno(e, None)
        );
    }
}

pub struct TinyHttpResponseContext(Option<tiny_http::Request>);

impl TinyHttpResponseContext {
    pub fn new(request: tiny_http::Request) -> Self {
        Self(Some(request))
    }

    // This should never panic, as the inner request instance is only settable to `None` on
    // response.
    pub fn inner(&self) -> &tiny_http::Request {
        self.0.as_ref().unwrap()
    }
}

impl ResponseContext for TinyHttpResponseContext {
    fn respond(mut self, head: &ResponseHead, body: &[u8]) {
        respond(self.0.take().unwrap(), head, body)
    }
}

impl Drop for TinyHttpResponseContext {
    fn drop(&mut self) {
        // Just warn. The `Drop` impl for `tiny_http::Request` will close the connection anyways.
        if let Some(inner) = self.0.take() {
            log::warn!(
                "Request to {} {} did not receive a response.",
                inner.method(),
                inner.url(),
            )
        }
    }
}
