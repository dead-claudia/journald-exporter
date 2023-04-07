use crate::prelude::*;

use super::common::*;

pub const REQUEST_METRICS: u8 = 0x00;
pub const REQUEST_KEY: u8 = 0x01;
pub const TRACK_REQUEST: u8 = 0x02;

const STATE_METRICS_REQUESTED: u8 = 1 << 0;
const STATE_KEYS_REQUESTED: u8 = 1 << 1;
const STATE_VERSION_ADDED: u8 = 1 << 2;

pub struct DecoderRequest {
    flags: u8,
    tracked_metrics_requests: usize,
}

impl PartialEq for DecoderRequest {
    fn eq(&self, other: &Self) -> bool {
        self.metrics_requested() == other.metrics_requested()
            && self.keys_requested() == other.keys_requested()
            && self.tracked_metrics_requests == other.tracked_metrics_requests
    }
}

impl DecoderRequest {
    // For readability
    #[cfg(test)]
    pub const NO_FLAGS: u8 = 0;

    pub const METRICS_REQUESTED: u8 = STATE_METRICS_REQUESTED;
    pub const KEYS_REQUESTED: u8 = STATE_KEYS_REQUESTED;

    pub const fn new(flags: u8, tracked_metrics_requests: usize) -> Self {
        Self {
            flags,
            tracked_metrics_requests,
        }
    }

    pub const fn metrics_requested(&self) -> bool {
        (self.flags & DecoderRequest::METRICS_REQUESTED) != 0
    }

    pub const fn keys_requested(&self) -> bool {
        (self.flags & DecoderRequest::KEYS_REQUESTED) != 0
    }

    pub const fn tracked_metrics_requests(&self) -> usize {
        self.tracked_metrics_requests
    }
}

impl fmt::Debug for DecoderRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecoderRequest")
            .field("metrics_requested", &self.metrics_requested())
            .field("keys_requested", &self.keys_requested())
            .field("tracked_metrics_requests", &self.tracked_metrics_requests())
            .finish()
    }
}

pub struct Decoder {
    state: u8,
    tracked_metrics_requests: Wrapping<usize>,
    version_phase: ReadPhase,
}

impl Decoder {
    pub const fn new() -> Self {
        Self {
            state: 0,
            tracked_metrics_requests: Wrapping(0),
            version_phase: ReadPhase::new(),
        }
    }

    pub fn take_request(&mut self) -> DecoderRequest {
        let state = self.state;
        let tracked_metrics_requests = self.tracked_metrics_requests.0;
        self.state &= !(STATE_METRICS_REQUESTED | STATE_KEYS_REQUESTED);
        self.tracked_metrics_requests.0 = 0;
        DecoderRequest::new(state, tracked_metrics_requests)
    }

    pub fn read_bytes(&mut self, buf: &[u8]) {
        let mut iter = ReadIter::new(buf);

        if (self.state & STATE_VERSION_ADDED) == 0 {
            match iter.phase_next_32(&mut self.version_phase) {
                None => return,
                Some(super::VERSION) => self.state |= STATE_VERSION_ADDED,
                Some(version) => unknown_version(version),
            }
        }

        for byte in iter.remaining() {
            match byte {
                0x00 => self.state |= STATE_METRICS_REQUESTED,
                0x01 => self.state |= STATE_KEYS_REQUESTED,
                0x02 => self.tracked_metrics_requests += 1,
                _ => unknown_byte(*byte),
            }
        }
    }
}
