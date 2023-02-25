use crate::prelude::*;

use super::syscall_utils::syscall_check_int;
use std::os::fd::AsRawFd;

pub use super::pollable_flags::*;

// This serves two purposes: reduce polymorphism and allow better flexibility around polling.
fn do_poll_fd(fd: i32, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
    let timeout = duration.map_or(-1, |d| truncate_u128_i32(d.as_millis()));
    let mut pollfd = libc::pollfd {
        fd,
        events: flags.bits(),
        revents: 0,
    };

    // SAFETY: FFI call, doesn't leave anything uninitalized when returning.
    let result = syscall_check_int("poll", unsafe { libc::poll(&mut pollfd, 1, timeout) })?;
    if result == 0 {
        Err(ErrorKind::TimedOut.into())
    } else {
        Ok(PollResult::from_bits_truncate(pollfd.revents))
    }
}

pub trait Pollable {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult>;
}

impl<T: Pollable> Pollable for &T {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        (**self).poll(flags, duration)
    }
}

impl<T: Pollable> Pollable for &mut T {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        (**self).poll(flags, duration)
    }
}

macro_rules! poll_by_fd {
    ($ty:ty) => {
        impl Pollable for $ty {
            fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
                do_poll_fd(self.as_raw_fd(), flags, duration)
            }
        }
    };
}

poll_by_fd!(std::os::fd::OwnedFd);
poll_by_fd!(std::process::ChildStdin);
poll_by_fd!(std::process::ChildStdout);
poll_by_fd!(io::Stdin);
poll_by_fd!(io::Stdout);
poll_by_fd!(io::Stderr);
poll_by_fd!(std::fs::File);

impl Pollable for &[u8] {
    fn poll(&self, _: PollFlags, _: Option<Duration>) -> io::Result<PollResult> {
        Ok(PollResult::IN)
    }
}

pub trait ImmutableWrite {
    type Inner<'a>: Write + Pollable
    where
        Self: 'a;
    fn inner(&self) -> Self::Inner<'_>;
}

macro_rules! immutable_write {
    ($ty:ty) => {
        impl ImmutableWrite for $ty {
            type Inner<'a> = &'a $ty;
            fn inner(&self) -> Self::Inner<'_> {
                self
            }
        }
    };
}

immutable_write!(io::Stdout);
immutable_write!(io::Stderr);
