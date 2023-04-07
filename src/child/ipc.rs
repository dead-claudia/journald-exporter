use crate::prelude::*;

use super::request::RequestContext;
use super::request::RequestShared;
use super::request::ResponseHead;
use super::request::ServerState;
use super::request::RESPONSE_OK_METRICS;
use super::request::RESPONSE_UNAVAILABLE;
use crate::ffi::ImmutableWrite;
use crate::ffi::Pollable;

fn read_request(
    state: &ServerState<impl RequestContext>,
    buf: &[u8],
) -> ipc::parent::DecoderResponse {
    let mut decoder = state.decoder.lock();
    decoder.read_bytes(buf);
    decoder.take_response()
}

fn resume_queued_requests(
    state: &ServerState<impl RequestContext>,
    head: &'static ResponseHead,
    body: &[u8],
) {
    // Avoid contention and deadlock by minimizing the critical section.
    let pending = {
        let mut guard = state
            .ipc_requester
            .pending_requests
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        take(&mut *guard)
    };

    for ctx in pending {
        ctx.respond(head, body);
    }
}

fn handle_key_response(state: &ServerState<impl RequestContext>, keys: Box<[Key]>) {
    let mut guard = state.key_set.write().unwrap_or_else(|e| e.into_inner());
    *guard = KeySet::new(keys);
}

fn handle_metrics_snapshot(state: &ServerState<impl RequestContext>, body: Box<[u8]>) {
    resume_queued_requests(state, &RESPONSE_OK_METRICS, &body);
}

pub fn child_ipc(
    state: &ServerState<impl RequestContext>,
    mut input: impl Read + Pollable,
    terminate_notify: &Notify,
) -> io::Result<()> {
    // Use a decently read large buffer. Stack space is cheap.
    let mut read_buf = [0_u8; 65536];

    loop {
        let response = match try_read(&mut input, terminate_notify, &mut read_buf) {
            ReadWriteResult::Success(buf) => read_request(state, buf),
            ReadWriteResult::Terminated => break,
            ReadWriteResult::Err(e) => return Err(e),
        };

        if let Some(keys) = response.key_set {
            handle_key_response(state, keys)
        }

        if let Some(snapshot) = response.metrics {
            handle_metrics_snapshot(state, snapshot);
        }
    }

    resume_queued_requests(state, &RESPONSE_UNAVAILABLE, &[]);
    Ok(())
}

pub struct IPCRequester<C> {
    pending_requests: Mutex<Vec<C>>,
}

impl<C: RequestContext> IPCRequester<C> {
    pub const fn new() -> Self {
        Self {
            pending_requests: Mutex::new(Vec::new()),
        }
    }

    #[cfg(test)]
    pub fn has_requests_pending(&self) -> bool {
        !self
            .pending_requests
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    }
}

// Returns `true` if successfully sent.
pub fn send_msg<C: RequestContext + 'static>(
    shared: &RequestShared<C, impl ImmutableWrite>,
    buf: &[u8],
) -> bool {
    try_send_msg(&shared.state.terminate_notify, shared.output.inner(), buf)
}

pub fn request_metrics<C: RequestContext + 'static>(
    ctx: C,
    shared: &RequestShared<C, impl ImmutableWrite>,
) {
    let pending_requests = &shared.state.ipc_requester.pending_requests;
    let mut guard = pending_requests.lock().unwrap_or_else(|e| e.into_inner());
    let is_first = guard.is_empty();
    guard.push(ctx);

    // Don't retain the lock longer than necessary.
    drop(guard);

    if is_first && !send_msg(shared, &[ipc::child::REQUEST_METRICS]) {
        resume_queued_requests(shared.state, &RESPONSE_UNAVAILABLE, &[]);
    }
}
