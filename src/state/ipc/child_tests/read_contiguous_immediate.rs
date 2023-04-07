use crate::prelude::*;

use super::common::*;

macro_rules! declarative_request {
    ($($tt:tt)*) => {{
        &[
            // IPC Version
            VERSION_BYTES[0], VERSION_BYTES[1], VERSION_BYTES[2], VERSION_BYTES[3],
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

    D.lock().read_bytes(REQUEST);
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

    D.lock().read_bytes(REQUEST);
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

    D.lock().read_bytes(REQUEST);
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

    D.lock().read_bytes(REQUEST);
}

#[test]
fn processes_single_request_metrics() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x00,
    ];

    D.lock().read_bytes(REQUEST);
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

    D.lock().read_bytes(REQUEST);
    assert_eq!(
        D.lock().take_request(),
        DecoderRequest::new(DecoderRequest::KEYS_REQUESTED, 0)
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

    D.lock().read_bytes(REQUEST);
    assert_eq!(
        D.lock().take_request(),
        DecoderRequest::new(DecoderRequest::NO_FLAGS, 1)
    );
}
