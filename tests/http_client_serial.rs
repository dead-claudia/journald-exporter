mod util;

use util::request_server::REQUEST_KEY;

macro_rules! ipc {
    ($($tt:tt)*) => {{
        &[0, 0, 0, 0, $($tt)*]
    }};
}

#[test]
fn handles_a_post_method_request() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/", &["-X", "POST"])
        .assert_response(
            br#"
HTTP/1.1 405 Method Not Allowed
content-length: 0
allow: GET,HEAD
connection: close

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_a_put_method_request() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/", &["-X", "PUT"])
        .assert_response(
            br#"
HTTP/1.1 405 Method Not Allowed
content-length: 0
allow: GET,HEAD
connection: close

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_a_patch_method_request() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/", &["-X", "PATCH"])
        .assert_response(
            br#"
HTTP/1.1 405 Method Not Allowed
content-length: 0
allow: GET,HEAD
connection: close

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_a_delete_method_request() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/", &["-X", "DELETE"])
        .assert_response(
            br#"
HTTP/1.1 405 Method Not Allowed
content-length: 0
allow: GET,HEAD
connection: close

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_a_trace_method_request() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/", &["-X", "TRACE"])
        .assert_response(
            br#"
HTTP/1.1 405 Method Not Allowed
content-length: 0
allow: GET,HEAD
connection: close

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_a_non_standard_method_request() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/", &["-X", "NOT-A-REAL-METHOD"])
        .assert_response(
            br#"
HTTP/1.1 405 Method Not Allowed
content-length: 0
allow: GET,HEAD
connection: close

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_a_wrong_path() {
    let http_server = util::request_server::Server::new(false);

    http_server.send_request("/what", &[]).assert_response(
        br#"
HTTP/1.1 404 Not Found
content-length: 0

"#,
    );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_no_auth_header() {
    let http_server = util::request_server::Server::new(false);

    http_server.send_request("/metrics", &[]).assert_response(
        br#"
HTTP/1.1 403 Not Authorized
www-authenticate: Basic realm="metrics"
content-length: 0

"#,
    );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_wrong_auth_type() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/metrics", &["-H", "Authorization: Bearer 123456789"])
        .assert_response(
            br#"
HTTP/1.1 403 Not Authorized
www-authenticate: Basic realm="metrics"
content-length: 0

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_missing_token() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/metrics", &["-H", "Authorization: Basic"])
        .assert_response(
            br#"
HTTP/1.1 403 Not Authorized
www-authenticate: Basic realm="metrics"
content-length: 0

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_incomplete_token() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/metrics", &["-H", "Authorization: Basic a"])
        .assert_response(
            br#"
HTTP/1.1 403 Not Authorized
www-authenticate: Basic realm="metrics"
content-length: 0

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}

#[test]
fn handles_not_base64() {
    let http_server = util::request_server::Server::new(false);

    http_server
        .send_request("/metrics", &["-H", "Authorization: Basic asr@v32@t!#a"])
        .assert_response(
            br#"
HTTP/1.1 403 Not Authorized
www-authenticate: Basic realm="metrics"
content-length: 0

"#,
        );

    http_server.assert_stdout(ipc![0, 0, 0, 0, REQUEST_KEY]);
}
