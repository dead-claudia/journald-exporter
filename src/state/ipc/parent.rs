/*
Warning: this module interacts very closely with `crate::state` and its children.
*/

use crate::prelude::*;

use super::common::*;

pub const METRICS_RESPONSE_HEADER: &[u8] = &[0x00, 0, 0, 0, 0];

pub fn finish_response_metrics(buf: &mut [u8]) {
    let len = buf.len().checked_sub(5).expect("buffer not initialized");
    let [a, b, c, d] = truncate_usize_u32(len).to_le_bytes();
    buf[1] = a;
    buf[2] = b;
    buf[3] = c;
    buf[4] = d;
}

pub fn receive_key_set_bytes(key_set: KeySet) -> Box<[u8]> {
    let key_set = key_set.insecure_view_keys();
    debug_assert!(key_set.len() <= zero_extend_u8_usize(u8::MAX));
    let mut buf = Vec::new();
    buf.extend_from_slice(&[0x01, truncate_usize_u8(key_set.len())]);

    for key in key_set.iter() {
        let key_value = key.insecure_get_value();
        debug_assert!(key_value.len() <= zero_extend_u8_usize(u8::MAX));
        buf.push(truncate_usize_u8(key_value.len()));
        buf.extend_from_slice(key.insecure_get_value());
    }

    buf.into()
}

#[derive(Debug)]
enum DecoderState {
    Locked,
    Version,
    Start,
    ResponseMetrics,
    ResponseMetricsExpectBody,
    ReceiveKeySet,
    ReceiveKeySetExpectEntry,
    ReceiveKeySetExpectKey,
}

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseItem<T> {
    None,
    AllocationFailed,
    Some(T),
}

#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct DecoderResponse {
    pub key_set: ResponseItem<KeySet>,
    pub metrics: ResponseItem<Box<[u8]>>,
}

impl fmt::Debug for DecoderResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecoderResponse")
            .field("key_set", &self.key_set)
            .field(
                "metrics",
                &match &self.metrics {
                    ResponseItem::None => ResponseItem::None,
                    ResponseItem::AllocationFailed => ResponseItem::AllocationFailed,
                    ResponseItem::Some(vec) => ResponseItem::Some(BinaryToDebug(vec)),
                },
            )
            .finish()
    }
}

pub struct Decoder {
    state: DecoderState,
    read_phase: ReadPhase,
    response: DecoderResponse,
    byte_acc: Option<ByteAccumulator>,
    key_acc: Option<KeyAccumulator>,
}

impl Decoder {
    pub const fn new() -> Self {
        Self {
            state: DecoderState::Version,
            read_phase: ReadPhase::new(),
            response: DecoderResponse {
                key_set: ResponseItem::None,
                metrics: ResponseItem::None,
            },
            byte_acc: None,
            key_acc: None,
        }
    }

    pub fn take_response(&mut self) -> DecoderResponse {
        replace(
            &mut self.response,
            DecoderResponse {
                key_set: ResponseItem::None,
                metrics: ResponseItem::None,
            },
        )
    }

    pub fn read_bytes(&mut self, buf: &[u8]) {
        let mut iter = ReadIter::new(buf);
        let mut state = replace(&mut self.state, DecoderState::Locked);

        self.state = loop {
            match state {
                DecoderState::Locked => unreachable!("Decoder is locked."),

                DecoderState::Version => match iter.phase_next_32(&mut self.read_phase) {
                    None => break DecoderState::Version,
                    Some(super::VERSION) => state = DecoderState::Start,
                    Some(version) => unknown_version(version),
                },

                DecoderState::Start => match iter.next() {
                    None => break DecoderState::Start,
                    Some(0) => state = DecoderState::ResponseMetrics,
                    Some(1) => state = DecoderState::ReceiveKeySet,
                    Some(byte) => unknown_byte(byte),
                },

                DecoderState::ResponseMetrics => match iter.phase_next_32(&mut self.read_phase) {
                    None => break DecoderState::ResponseMetrics,
                    Some(len) => {
                        self.byte_acc = Some(ByteAccumulator::new(len));
                        state = DecoderState::ResponseMetricsExpectBody;
                    }
                },

                DecoderState::ResponseMetricsExpectBody => {
                    let byte_acc = self.byte_acc.as_mut().unwrap();
                    if byte_acc.has_remaining() {
                        state = DecoderState::ResponseMetricsExpectBody;
                        if !byte_acc.push_from_iter(&mut iter) {
                            break state;
                        }
                    } else {
                        self.response.metrics = match self.byte_acc.take() {
                            None => ResponseItem::None,
                            Some(response) => match response.finish() {
                                None => ResponseItem::AllocationFailed,
                                Some(metrics_data) => ResponseItem::Some(metrics_data),
                            },
                        };
                        state = DecoderState::Start;
                    }
                }

                DecoderState::ReceiveKeySet => match iter.next() {
                    None => break DecoderState::ReceiveKeySet,
                    Some(len) => {
                        self.key_acc = Some(KeyAccumulator::new(zero_extend_u8_u32(len)));
                        state = DecoderState::ReceiveKeySetExpectEntry;
                    }
                },

                DecoderState::ReceiveKeySetExpectEntry => {
                    let key_acc = self.key_acc.as_mut().unwrap();
                    if key_acc.has_remaining() {
                        match iter.next() {
                            None => break DecoderState::ReceiveKeySetExpectEntry,
                            Some(len) => {
                                if len > truncate_usize_u8(MAX_KEY_LEN) {
                                    std::panic::panic_any("Key entry too long.");
                                }
                                self.byte_acc = Some(ByteAccumulator::new(zero_extend_u8_u32(len)));
                                state = DecoderState::ReceiveKeySetExpectKey;
                            }
                        }
                    } else {
                        self.response.key_set = match self.key_acc.take() {
                            None => ResponseItem::None,
                            Some(response) => match response.finish() {
                                None => ResponseItem::AllocationFailed,
                                Some(key_set) => ResponseItem::Some(key_set),
                            },
                        };
                        state = DecoderState::Start;
                    }
                }

                DecoderState::ReceiveKeySetExpectKey => {
                    let byte_acc = self.byte_acc.as_mut().unwrap();
                    if byte_acc.has_remaining() {
                        state = DecoderState::ReceiveKeySetExpectKey;
                        if !byte_acc.push_from_iter(&mut iter) {
                            break state;
                        }
                    } else {
                        self.key_acc
                            .as_mut()
                            .unwrap()
                            .push_raw(byte_acc.initialized());
                        state = DecoderState::ReceiveKeySetExpectEntry;
                    }
                }
            }
        }
    }
}
