use crate::prelude::*;

use super::syscall_utils::syscall_assert_int;
use std::os::fd::RawFd;

// It's okay for this to panic. It's only used in the child setup code.
pub fn set_non_blocking(fd: RawFd) {
    assert_not_miri();

    // SAFETY: FFI call, called with correct parameters and doesn't modify program-internal state.
    unsafe {
        let result = syscall_assert_int("fcntl", libc::fcntl(fd, libc::F_GETFL));
        // SAFETY: FFI call, called with correct parameters and doesn't modify program-internal state.
        syscall_assert_int(
            "fcntl",
            libc::fcntl(fd, libc::F_SETFL, result | libc::O_NONBLOCK),
        );
    }
}
