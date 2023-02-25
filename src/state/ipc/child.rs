use super::common::*;

pub use super::child_flags::DecoderRequestFlags;
use super::child_flags::DecoderState;

pub const REQUEST_METRICS: u8 = 0x00;
pub const REQUEST_KEY: u8 = 0x01;
pub const TRACK_REQUEST: u8 = 0x02;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoderRequest {
    flags: DecoderRequestFlags,
    tracked_requests: usize,
}

impl DecoderRequest {
    pub const fn new(flags: DecoderRequestFlags, tracked_requests: usize) -> Self {
        Self {
            flags,
            tracked_requests,
        }
    }

    pub const fn metrics_requested(&self) -> bool {
        self.flags.contains(DecoderRequestFlags::METRICS)
    }

    pub const fn keys_requested(&self) -> bool {
        self.flags.contains(DecoderRequestFlags::KEY)
    }

    pub const fn tracked_requests(&self) -> usize {
        self.tracked_requests
    }
}

#[derive(Debug)]
pub struct Decoder {
    state: DecoderState,
    tracked_requests: usize,
    version_phase: ReadPhase,
}

impl Decoder {
    pub const fn new() -> Self {
        Self {
            state: DecoderState::empty(),
            tracked_requests: 0,
            version_phase: ReadPhase::new(),
        }
    }

    pub fn take_request(&mut self) -> DecoderRequest {
        let state = self.state;
        let tracked_requests = self.tracked_requests;
        self.state.remove(DecoderState::from_bits_truncate(
            DecoderRequestFlags::all().bits(),
        ));
        self.tracked_requests = 0;
        DecoderRequest::new(
            DecoderRequestFlags::from_bits_truncate(state.bits()),
            tracked_requests,
        )
    }

    pub fn read_bytes(&mut self, buf: &[u8]) {
        let mut iter = ReadIter::new(buf);

        if !self.state.contains(DecoderState::VERSION_ADDED) {
            match iter.phase_next_32(&mut self.version_phase) {
                None => return,
                Some(super::VERSION) => self.state.insert(DecoderState::VERSION_ADDED),
                Some(version) => unknown_version(version),
            }
        }

        for byte in iter.remaining() {
            match byte {
                0x00 => self.state.insert(DecoderState::REQUEST_METRICS),
                0x01 => self.state.insert(DecoderState::REQUEST_KEY),
                0x02 => self.tracked_requests = self.tracked_requests.wrapping_add(1),
                _ => unknown_byte(*byte),
            }
        }
    }
}
