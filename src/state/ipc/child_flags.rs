// FIXME: re-evaluate once https://github.com/rust-lang/rust-clippy/pull/10309 is released.
#![allow(clippy::arithmetic_side_effects)]

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DecoderRequestFlags: u8 {
        const METRICS = 1 << 0;
        const KEY     = 1 << 1;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DecoderState: u8 {
        // Note: the values for `REQUEST_*` must match the similarly-named items in
        // `DecoderRequest`.
        const REQUEST_METRICS = 1 << 0;
        const REQUEST_KEY     = 1 << 1;
        const VERSION_ADDED   = 1 << 2;
    }
}
