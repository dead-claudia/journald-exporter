use crate::prelude::*;

use super::syscall_utils::syscall_check_int;
use std::os::fd::AsRawFd;

pub use super::pollable_flags::*;
use std::os::fd::RawFd;

pub trait Pollable {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult>;
}

//  ######
//  #     # #####   ####  #    # #   #    # #    # #####  #       ####
//  #     # #    # #    #  #  #   # #     # ##  ## #    # #      #
//  ######  #    # #    #   ##     #      # # ## # #    # #       ####
//  #       #####  #    #   ##     #      # #    # #####  #           #
//  #       #   #  #    #  #  #    #      # #    # #      #      #    #
//  #       #    #  ####  #    #   #      # #    # #      ######  ####

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

//  ######
//  #     # #    # # #      #####       # #    #    # #    # #####  #       ####
//  #     # #    # # #        #         # ##   #    # ##  ## #    # #      #
//  ######  #    # # #        #   ##### # # #  #    # # ## # #    # #       ####
//  #     # #    # # #        #         # #  # #    # #    # #####  #           #
//  #     # #    # # #        #         # #   ##    # #    # #      #      #    #
//  ######   ####  # ######   #         # #    #    # #    # #      ######  ####

// This serves two purposes: reduce polymorphism and allow better flexibility around polling.
fn do_poll_fd(fd: RawFd, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
    assert_not_miri();

    let timeout = duration.map_or(-1, |d| truncate_u128_i32(d.as_millis()));
    let mut pollfd = libc::pollfd {
        fd,
        events: flags.raw_bits(),
        revents: 0,
    };

    // SAFETY: FFI call, doesn't leave anything uninitalized when returning.
    let result = syscall_check_int("poll", unsafe { libc::poll(&mut pollfd, 1, timeout) })?;
    if result == 0 {
        Err(ErrorKind::TimedOut.into())
    } else {
        Ok(PollResult::from_raw_bits(pollfd.revents))
    }
}

impl Pollable for std::os::fd::OwnedFd {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        do_poll_fd(self.as_raw_fd(), flags, duration)
    }
}

impl Pollable for std::process::ChildStdin {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        do_poll_fd(self.as_raw_fd(), flags, duration)
    }
}

impl Pollable for std::process::ChildStdout {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        do_poll_fd(self.as_raw_fd(), flags, duration)
    }
}

impl Pollable for io::Stdin {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        do_poll_fd(self.as_raw_fd(), flags, duration)
    }
}

impl Pollable for io::Stdout {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        do_poll_fd(self.as_raw_fd(), flags, duration)
    }
}

impl Pollable for io::Stderr {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        do_poll_fd(self.as_raw_fd(), flags, duration)
    }
}

impl Pollable for std::fs::File {
    fn poll(&self, flags: PollFlags, duration: Option<Duration>) -> io::Result<PollResult> {
        do_poll_fd(self.as_raw_fd(), flags, duration)
    }
}

//  #     #
//  ##   ##  ####   ####  #    #    # #    # #####  #       ####
//  # # # # #    # #    # #   #     # ##  ## #    # #      #
//  #  #  # #    # #      ####      # # ## # #    # #       ####
//  #     # #    # #      #  #      # #    # #####  #           #
//  #     # #    # #    # #   #     # #    # #      #      #    #
//  #     #  ####   ####  #    #    # #    # #      ######  ####

impl Pollable for &[u8] {
    fn poll(&self, _: PollFlags, _: Option<Duration>) -> io::Result<PollResult> {
        Ok(PollResult::IN)
    }
}

//  ###                                                        #     #
//   #  #    # #    # #    # #####   ##   #####  #      ###### #  #  # #####  # ##### ######
//   #  ##  ## ##  ## #    #   #    #  #  #    # #      #      #  #  # #    # #   #   #
//   #  # ## # # ## # #    #   #   #    # #####  #      #####  #  #  # #    # #   #   #####
//   #  #    # #    # #    #   #   ###### #    # #      #      #  #  # #####  #   #   #
//   #  #    # #    # #    #   #   #    # #    # #      #      #  #  # #   #  #   #   #
//  ### #    # #    #  ####    #   #    # #####  ###### ######  ## ##  #    # #   #   ######

pub trait ImmutableWrite {
    type Inner<'a>: Write + Pollable
    where
        Self: 'a;
    fn inner(&self) -> Self::Inner<'_>;
}

impl ImmutableWrite for io::Stdout {
    type Inner<'a> = &'a io::Stdout;
    fn inner(&self) -> Self::Inner<'_> {
        self
    }
}

impl ImmutableWrite for io::Stderr {
    type Inner<'a> = &'a io::Stderr;
    fn inner(&self) -> Self::Inner<'_> {
        self
    }
}
