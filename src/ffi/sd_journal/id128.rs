use crate::prelude::*;

use crate::ffi::syscall_utils::sd_check;
use libsystemd_sys::id128;

// Make this type-safe and a little easier to work with.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Id128(pub u128);

impl Id128 {
    pub fn get_from_boot() -> io::Result<Id128> {
        // SAFETY: the pointer is initialized after the call.
        unsafe {
            let mut raw = MaybeUninit::<id128::sd_id128_t>::uninit();
            sd_check(
                "sd_id128_get_boot",
                id128::sd_id128_get_boot(raw.as_mut_ptr()),
            )?;
            Ok(Id128(u128::from_le_bytes(raw.assume_init().bytes)))
        }
    }
}
