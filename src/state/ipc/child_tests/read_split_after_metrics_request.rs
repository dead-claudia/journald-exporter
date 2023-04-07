use crate::prelude::*;

use super::common::*;

macro_rules! declarative_request {
    ($($tt:tt)*) => {{
        &[
            // IPC Version
            VERSION_BYTES[0], VERSION_BYTES[1], VERSION_BYTES[2], VERSION_BYTES[3],
            // Operation ID
            0x00,
            $($tt)*
        ]
    }};
}

#[test]
#[should_panic = "Unknown IPC byte 'FE'"]
fn panics_on_invalid_start_byte() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0xFE,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }
}

#[test]
#[should_panic = "Unknown IPC byte 'FE'"]
fn panics_on_invalid_start_byte_then_request_metrics() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0xFE,
        // Operation ID
        0x00,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }
}

#[test]
#[should_panic = "Unknown IPC byte 'FE'"]
fn panics_on_invalid_start_byte_then_request_key() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0xFE,
        // Operation ID
        0x01,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }
}

#[test]
#[should_panic = "Unknown IPC byte 'FE'"]
fn panics_on_invalid_start_byte_then_invalid_start_byte() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0xFE,
        // Operation ID
        0xFE,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }
}

#[test]
fn processes_single_request_metrics() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x00,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }

    assert_eq!(
        D.lock().take_request(),
        DecoderRequest::new(DecoderRequest::METRICS_REQUESTED, 0)
    );
}

#[test]
fn processes_single_request_key() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x01,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }

    assert_eq!(
        D.lock().take_request(),
        DecoderRequest::new(
            DecoderRequest::METRICS_REQUESTED | DecoderRequest::KEYS_REQUESTED,
            0
        )
    );
}

#[test]
fn processes_single_track_request() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x02,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }

    assert_eq!(
        D.lock().take_request(),
        DecoderRequest::new(DecoderRequest::METRICS_REQUESTED, 1)
    );
}
