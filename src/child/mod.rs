mod ipc;
mod limiter;
mod request;
#[cfg(test)]
mod request_tests;
mod server;
mod start;

pub use start::start_child;

// This shouldn't be seeing very many requests. If this many concurrent requests are occurring,
// it's clearly a sign that *way* too many requests are being sent.
pub const PENDING_REQUEST_CAPACITY: usize = 256;
