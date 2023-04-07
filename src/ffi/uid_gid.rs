use super::syscall_utils::syscall_check_int;
use crate::prelude::*;

pub const ROOT_UID: u32 = 0;
pub const ROOT_GID: u32 = 0;

pub fn current_uid() -> u32 {
    if cfg!(miri) {
        99999
    } else {
        // SAFETY: `getuid` can never fail.
        unsafe { libc::getuid() }
    }
}

pub fn current_gid() -> u32 {
    if cfg!(miri) {
        77777
    } else {
        // SAFETY: `getgid` can never fail.
        unsafe { libc::getgid() }
    }
}

pub fn set_euid(id: u32) -> io::Result<()> {
    assert_not_miri();

    // SAFETY: Result is checked, and it doesn't touch Rust-accessible memory
    unsafe {
        syscall_check_int("seteuid", libc::seteuid(id))?;
    }

    Ok(())
}

pub fn set_egid(id: u32) -> io::Result<()> {
    assert_not_miri();

    // SAFETY: Result is checked, and it doesn't touch Rust-accessible memory
    unsafe {
        syscall_check_int("setegid", libc::setegid(id))?;
    }

    Ok(())
}

// These are stubbed out in Miri, so there's no real point in "testing" them.
#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;

    #[test]
    fn current_uid_works() {
        let _ = current_uid();
    }

    #[test]
    fn current_gid_works() {
        let _ = current_gid();
    }
}
