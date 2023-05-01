use crate::prelude::*;

use super::request::RequestShared;
use super::request::ResponseContext;
use super::request::ResponseHead;
use super::request::ServerState;
use super::request::RESPONSE_OK_METRICS;
use super::request::RESPONSE_SERVER_ERROR;
use super::request::RESPONSE_UNAVAILABLE;
use super::PENDING_REQUEST_CAPACITY;
use crate::ffi::ImmutableWrite;
use crate::ffi::Pollable;
use crate::state::ipc::parent::ResponseItem;

fn read_request(
    state: &ServerState<impl ResponseContext>,
    buf: &[u8],
) -> ipc::parent::DecoderResponse {
    let mut decoder = state.decoder.lock();
    decoder.read_bytes(buf);
    decoder.take_response()
}

fn resume_queued_requests(
    state: &ServerState<impl ResponseContext>,
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

    for res in pending {
        res.respond(head, body);
    }
}

fn handle_key_set_response(
    state: &ServerState<impl ResponseContext>,
    response: ResponseItem<KeySet>,
) {
    match response {
        ResponseItem::None => {}
        ResponseItem::AllocationFailed => {
            log::error!("Child key set response allocation failed. Retaining current keys.");
        }
        ResponseItem::Some(keys) => {
            *state.key_set.write().unwrap_or_else(|e| e.into_inner()) = Some(keys);
        }
    }
}

fn handle_metrics_response(
    state: &ServerState<impl ResponseContext>,
    response: ResponseItem<Box<[u8]>>,
) {
    match response {
        ResponseItem::None => {}
        ResponseItem::AllocationFailed => {
            log::error!("Child metrics response allocation failed.");
            resume_queued_requests(state, &RESPONSE_SERVER_ERROR, &[]);
        }
        ResponseItem::Some(snapshot) => {
            resume_queued_requests(state, &RESPONSE_OK_METRICS, &snapshot);
        }
    }
}

pub fn child_ipc(
    state: &ServerState<impl ResponseContext>,
    mut input: impl Read + Pollable,
    terminate_notify: &Notify,
) -> io::Result<()> {
    // Use a decently read large buffer. Stack space is cheap.
    let mut read_buf = [0_u8; 65536];

    while let Some(buf) = try_read(&mut input, terminate_notify, &mut read_buf)? {
        let response = read_request(state, buf);

        handle_key_set_response(state, response.key_set);
        handle_metrics_response(state, response.metrics);
    }

    resume_queued_requests(state, &RESPONSE_UNAVAILABLE, &[]);
    Ok(())
}

pub struct IPCRequester<C> {
    pending_requests: Mutex<heapless::Vec<C, PENDING_REQUEST_CAPACITY>>,
}

impl<C: ResponseContext> IPCRequester<C> {
    pub const fn new() -> Self {
        Self {
            pending_requests: Mutex::new(heapless::Vec::new()),
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
pub fn send_msg<C: ResponseContext + 'static>(
    shared: &RequestShared<C, impl ImmutableWrite>,
    buf: &[u8],
) -> bool {
    try_send_msg(&shared.state.terminate_notify, shared.output.inner(), buf)
}

pub fn request_metrics<C: ResponseContext + 'static>(
    res: C,
    shared: &RequestShared<C, impl ImmutableWrite>,
) {
    let pending_requests = &shared.state.ipc_requester.pending_requests;
    let mut guard = pending_requests.lock().unwrap_or_else(|e| e.into_inner());
    let is_first = guard.is_empty();
    let result = guard.push(res);

    // Don't retain the lock longer than necessary.
    drop(guard);

    match result {
        Ok(()) => {
            if is_first && !send_msg(shared, &[ipc::child::REQUEST_METRICS]) {
                resume_queued_requests(shared.state, &RESPONSE_UNAVAILABLE, &[]);
            }
        }
        Err(res) => {
            res.respond(&RESPONSE_UNAVAILABLE, &[]);
        }
    }
}
