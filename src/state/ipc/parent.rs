/*
Warning: this module interacts very closely with `crate::state` and its children.
*/

use crate::prelude::*;

use super::common::*;

pub fn init_response_metrics_header(buf: &mut Vec<u8>) {
    buf.extend_from_slice(&[0x00, 0, 0, 0, 0]);
}

pub fn finish_response_metrics(mut buf: Vec<u8>) -> Box<[u8]> {
    let len = buf.len().checked_sub(5).expect("buffer not initialized");
    let [a, b, c, d] = truncate_usize_u32(len).to_le_bytes();
    buf[1] = a;
    buf[2] = b;
    buf[3] = c;
    buf[4] = d;
    buf.into()
}

pub fn receive_key_set_bytes(key_set: KeySet) -> Box<[u8]> {
    let key_set = key_set.into_insecure_view_keys();
    assert!(key_set.len() <= zero_extend_u8_usize(u8::MAX));
    let mut buf = Vec::new();
    buf.push(0x01);
    buf.push(truncate_usize_u8(key_set.len()));

    for key in key_set.iter() {
        let key_value = key.insecure_get_value();
        assert!(key_value.len() <= zero_extend_u8_usize(u8::MAX));
        // It's okay to wrap - it can't be zero.
        buf.push(truncate_usize_u8(key_value.len().wrapping_sub(1)));
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

#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct DecoderResponse {
    pub key_set: Option<Box<[Key]>>,
    pub metrics: Option<Box<[u8]>>,
}

impl fmt::Debug for DecoderResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecoderResponse")
            .field("key_set", &self.key_set)
            .field("metrics", &self.metrics.as_deref().map(BinaryToDebug))
            .finish()
    }
}

#[derive(Debug)]
pub struct Decoder {
    state: DecoderState,
    read_phase: ReadPhase,
    key_set_response: Option<SliceAccumulator<Key>>,
    metrics_response: Option<SliceAccumulator<u8>>,
    byte_acc: Option<SliceAccumulator<u8>>,
    key_acc: Option<SliceAccumulator<Key>>,
}

impl Decoder {
    pub const fn new() -> Self {
        Self {
            state: DecoderState::Version,
            read_phase: ReadPhase::new(),
            key_set_response: None,
            metrics_response: None,
            byte_acc: None,
            key_acc: None,
        }
    }

    pub fn take_response(&mut self) -> DecoderResponse {
        DecoderResponse {
            key_set: self.key_set_response.take().map(|r| r.finish()),
            metrics: self.metrics_response.take().map(|r| r.finish()),
        }
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
                        self.byte_acc = Some(SliceAccumulator::new(len));
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
                        self.metrics_response = self.byte_acc.take();
                        state = DecoderState::Start;
                    }
                }

                DecoderState::ReceiveKeySet => match iter.next() {
                    None => break DecoderState::ReceiveKeySet,
                    Some(len) => {
                        self.key_acc = Some(SliceAccumulator::new(zero_extend_u8_u32(len)));
                        state = DecoderState::ReceiveKeySetExpectEntry;
                    }
                },

                DecoderState::ReceiveKeySetExpectEntry => {
                    let key_acc = self.key_acc.as_mut().unwrap();
                    if key_acc.has_remaining() {
                        match iter.next() {
                            None => break DecoderState::ReceiveKeySetExpectEntry,
                            Some(len) => {
                                self.byte_acc = Some(SliceAccumulator::new(
                                    zero_extend_u8_u32(len).wrapping_add(1),
                                ));
                                state = DecoderState::ReceiveKeySetExpectKey;
                            }
                        }
                    } else {
                        self.key_set_response = self.key_acc.take();
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
                            .push(Key::from_raw(byte_acc.initialized()));
                        state = DecoderState::ReceiveKeySetExpectEntry;
                    }
                }
            }
        }
    }
}
