// FIXME: re-evaluate once https://github.com/rust-lang/rust-clippy/pull/10309 is released.
#![allow(clippy::arithmetic_side_effects)]

use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PollFlags(libc::c_short);

impl PollFlags {
    #[allow(unused)]
    pub const EMPTY: PollFlags = PollFlags(0);
    #[allow(unused)]
    pub const IN: PollFlags = PollFlags(libc::POLLIN);
    #[allow(unused)]
    pub const OUT: PollFlags = PollFlags(libc::POLLOUT);

    pub const fn raw_bits(&self) -> libc::c_short {
        self.0
    }

    pub const fn has_in(&self) -> bool {
        (self.0 & libc::POLLIN) != 0
    }

    pub const fn has_out(&self) -> bool {
        (self.0 & libc::POLLOUT) != 0
    }
}

impl fmt::Debug for PollFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        let mut has_flags = false;

        result.push_str("PollFlags(");

        if self.has_in() {
            result.push_str("IN");
            has_flags = true;
        }

        if self.has_out() {
            result.push_str(if has_flags { " | OUT" } else { "OUT" });
            has_flags = true;
        }

        if !has_flags {
            result.push('0');
        }

        result.push(')');

        f.write_str(&result)
    }
}

pub struct PollResult(libc::c_short);

impl PollResult {
    #[allow(unused)]
    pub const EMPTY: PollResult = PollResult(0);
    #[allow(unused)]
    pub const IN: PollResult = PollResult(libc::POLLIN);
    #[allow(unused)]
    pub const OUT: PollResult = PollResult(libc::POLLOUT);
    #[allow(unused)]
    pub const ERR: PollResult = PollResult(libc::POLLERR);
    #[allow(unused)]
    pub const HUP: PollResult = PollResult(libc::POLLHUP);

    pub const fn from_raw_bits(bits: libc::c_short) -> PollResult {
        Self(bits)
    }

    pub const fn has_in(&self) -> bool {
        (self.0 & libc::POLLIN) != 0
    }

    pub const fn has_out(&self) -> bool {
        (self.0 & libc::POLLOUT) != 0
    }

    pub const fn has_err(&self) -> bool {
        (self.0 & libc::POLLERR) != 0
    }

    pub const fn has_hup(&self) -> bool {
        (self.0 & libc::POLLHUP) != 0
    }
}

impl std::ops::BitOr for PollResult {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl fmt::Debug for PollResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        let mut has_flags = false;

        result.push_str("PollResult(");

        if self.has_in() {
            result.push_str("IN");
            has_flags = true;
        }

        if self.has_out() {
            result.push_str(if has_flags { " | OUT" } else { "OUT" });
            has_flags = true;
        }

        if self.has_err() {
            result.push_str(if has_flags { " | ERR" } else { "ERR" });
            has_flags = true;
        }

        if self.has_hup() {
            result.push_str(if has_flags { " | HUP" } else { "HUP" });
            has_flags = true;
        }

        if !has_flags {
            result.push('0');
        }

        result.push(')');

        f.write_str(&result)
    }
}
