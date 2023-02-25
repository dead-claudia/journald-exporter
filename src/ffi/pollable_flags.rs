// FIXME: re-evaluate once https://github.com/rust-lang/rust-clippy/pull/10309 is released.
#![allow(clippy::arithmetic_side_effects)]

bitflags! {
    pub struct PollFlags: i16 {
        const IN = libc::POLLIN;
        const OUT = libc::POLLOUT;
    }
}

bitflags! {
    pub struct PollResult: i16 {
        const IN = libc::POLLIN;
        const OUT = libc::POLLOUT;
        const ERR = libc::POLLERR;
        const HUP = libc::POLLHUP;
    }
}
