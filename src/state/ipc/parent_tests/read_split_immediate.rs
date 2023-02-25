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

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }
}

#[test]
#[should_panic = "Unknown IPC byte 'FE'"]
fn panics_on_invalid_start_byte_then_response_metrics_start() {
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
fn panics_on_invalid_start_byte_then_response_metrics() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0xFE,
        // Operation ID
        0x00,
        // Data length (16)
        0x10, 0x00, 0x00, 0x00,
        // Data
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
        b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }
}

#[test]
#[should_panic = "Unknown IPC byte 'FE'"]
fn panics_on_invalid_start_byte_then_receive_key_start() {
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
fn panics_on_invalid_start_byte_then_receive_key() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0xFE,
        // Operation ID
        0x01,
        // Key set length
        0x01,
        // Key 1: all hex digits (length: 16)
        0x0F,
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
        b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
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
fn processes_partial_response_metrics() {
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
        D.lock().take_response(),
        DecoderResponse {
            key_set: None,
            metrics: None,
        }
    );
}

#[test]
fn processes_receive_metrics_zero_data_bytes_ingested() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x00,
        // Data length (0)
        0x00, 0x00, 0x00, 0x00,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }

    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: None,
            metrics: Some(Box::new([])),
        }
    );
}

#[test]
fn processes_receive_metrics_1_byte_data_len_ingested() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x00,
        // Data length (16)
        0x10, 0x00, 0x00, 0x00,
        // Data
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
        b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }

    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: None,
            metrics: Some(Box::new(*b"0123456789ABCDEF")),
        }
    );
}

#[test]
fn processes_receive_metrics_2_byte_data_len_ingested() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    const REQUEST_LENGTH: usize = 1000;
    let mut req = Vec::with_capacity(declarative_request![].len() + REQUEST_LENGTH + 9);
    req.extend_from_slice(declarative_request![]);
    // Operation ID
    req.push(0x00);
    // Data length
    req.extend_from_slice(&truncate_usize_u32(REQUEST_LENGTH).to_le_bytes());
    // Data
    req.extend((0..REQUEST_LENGTH).map(index_hex));

    D.lock().read_bytes(&req);
    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: None,
            metrics: Some(expected_data_hex(REQUEST_LENGTH)),
        }
    );
}

#[test]
// This takes a long time with Miri, and the code path is already tested elsewhere
#[cfg_attr(miri, ignore)]
fn processes_receive_metrics_3_byte_data_len_ingested() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    const REQUEST_LENGTH: usize = 200_000;
    let mut req = Vec::with_capacity(declarative_request![].len() + REQUEST_LENGTH + 9);
    req.extend_from_slice(declarative_request![]);
    // Operation ID
    req.push(0x00);
    // Data length
    req.extend_from_slice(&truncate_usize_u32(REQUEST_LENGTH).to_le_bytes());
    // Data
    req.extend((0..REQUEST_LENGTH).map(index_hex));

    D.lock().read_bytes(&req);
    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: None,
            metrics: Some(expected_data_hex(REQUEST_LENGTH)),
        }
    );
}

#[test]
// This takes a long time with Miri, and the code path is already tested elsewhere
#[cfg_attr(miri, ignore)]
fn processes_receive_metrics_4_byte_data_len_ingested() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    const REQUEST_LENGTH: usize = 50_000_000;
    let mut req = Vec::with_capacity(declarative_request![].len() + REQUEST_LENGTH + 9);
    req.extend_from_slice(declarative_request![]);
    // Operation ID
    req.push(0x00);
    // Data length
    req.extend_from_slice(&truncate_usize_u32(REQUEST_LENGTH).to_le_bytes());
    // Data
    req.extend((0..REQUEST_LENGTH).map(index_hex));

    D.lock().read_bytes(&req);
    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: None,
            metrics: Some(expected_data_hex(REQUEST_LENGTH)),
        }
    );
}

#[test]
fn processes_partial_receive_key() {
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
        D.lock().take_response(),
        DecoderResponse {
            key_set: None,
            metrics: None,
        }
    );
}

#[test]
fn processes_empty_key_set() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x01,
        // Key set length
        0x00,
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }

    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: Some(Box::new([])),
            metrics: None,
        }
    );
}

#[test]
fn processes_single_item_key_set() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x01,
        // Key set length
        0x01,
        // Key 1: all hex digits (length: 16)
        0x0F,
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
        b'8', b'9', b'A', b'B', b'C', b'D', b'E', b'F',
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }

    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: Some(Box::new([Key::from_raw(b"0123456789ABCDEF")])),
            metrics: None,
        }
    );
}

#[test]
fn processes_max_length_key_set() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());

    #[rustfmt::skip]
    static REQUEST: &[u8] = declarative_request![
        // Operation ID
        0x01,
        // Key set length
        0xFF,
        // Keys, of the following pattern (so it's easier to type):
        // - 2 `A`s
        // - 4 `B`s
        // - 2 `C`s
        // - 4 `D`s
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C', 0x03, b'D', b'D', b'D', b'D',
        0x01, b'A', b'A', 0x03, b'B', b'B', b'B', b'B', 0x01, b'C', b'C',
    ];

    // Keys, of the following pattern (so it's easier to type):
    // - 2 `A`s
    // - 4 `B`s
    // - 2 `C`s
    // - 4 `D`s
    #[rustfmt::skip]
    let expected_keys = [
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"), Key::from_raw(b"DDDD"),
        Key::from_raw(b"AA"), Key::from_raw(b"BBBB"), Key::from_raw(b"CC"),
    ];

    for chunk in split_req(REQUEST) {
        D.lock().read_bytes(chunk);
    }

    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: Some(Box::new(expected_keys)),
            metrics: None,
        }
    );
}
