use crate::prelude::*;

use super::common::*;

#[test]
fn empty_buffer_produces_no_messages() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());
    D.lock().read_bytes(&[]);
    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: ResponseItem::None,
            metrics: ResponseItem::None,
        }
    );
}

// 1 billion, encoded as little endian
static BAD_VERSION_BYTES: [u8; 4] = [0x00, 0xCA, 0x9A, 0x3B];

#[test]
#[should_panic = "Bad version ID: 1000000000"]
fn version_mismatch_panics() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());
    D.lock().read_bytes(&BAD_VERSION_BYTES);
}

#[test]
#[should_panic = "Bad version ID: 1000000000"]
fn split_version_mismatch_panics() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());
    D.lock().read_bytes(&BAD_VERSION_BYTES[0..2]);
    D.lock().read_bytes(&BAD_VERSION_BYTES[2..4]);
}

#[test]
fn empty_buffer_post_split_version_produces_no_messages() {
    static D: Uncontended<Decoder> = Uncontended::new(Decoder::new());
    D.lock().read_bytes(&VERSION_BYTES[0..2]);
    D.lock().read_bytes(&VERSION_BYTES[2..4]);
    D.lock().read_bytes(&[]);
    assert_eq!(
        D.lock().take_response(),
        DecoderResponse {
            key_set: ResponseItem::None,
            metrics: ResponseItem::None,
        }
    );
}
