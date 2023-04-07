use crate::prelude::*;

use super::syscall_utils::syscall_check_int;
use std::os::fd::RawFd;

#[cold]
pub fn set_non_blocking(fd: RawFd) -> io::Result<()> {
    assert_not_miri();

    // SAFETY: FFI call, called with correct parameters and doesn't modify program-internal state.
    unsafe {
        let result = syscall_check_int("fcntl", libc::fcntl(fd, libc::F_GETFL))?;
        // SAFETY: FFI call, called with correct parameters and doesn't modify program-internal state.
        syscall_check_int(
            "fcntl",
            libc::fcntl(fd, libc::F_SETFL, result | libc::O_NONBLOCK),
        )?;
    }
    Ok(())
}
