use crate::prelude::*;

use super::Id128;
use super::SystemdMonotonicUsec;
use super::SystemdProvider;
use crate::ffi::syscall_utils::sd_check;
use crate::ffi::syscall_utils::syscall_check_int;
use libsystemd_sys::daemon;
use std::ffi::CStr;

static WATCHDOG_MSG: &CStr = c_str(b"WATCHDOG=1\0");

pub struct NativeSystemdProvider {
    watchdog_enabled: bool,
    boot_id: Id128,
}

impl SystemdProvider for NativeSystemdProvider {
    fn watchdog_notify(&self) -> io::Result<()> {
        if self.watchdog_enabled {
            // SAFETY: It's just invoking the native systemd function, and invariants are upheld via the
            // function's type.
            let result = sd_check("sd_notify", unsafe {
                daemon::sd_notify(0, WATCHDOG_MSG.as_ptr())
            })?;

            if result == 0 {
                return Err(ErrorKind::NotConnected.into());
            }
        }

        Ok(())
    }

    fn boot_id(&self) -> Id128 {
        self.boot_id
    }

    // I have to use the underlying syscall, as systemd expects the raw value and not any sort of
    // time delta.
    fn get_monotonic_time_usec(&'static self) -> SystemdMonotonicUsec {
        let mut timespec = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        // SAFETY: it's only passed in valid addresses, and the result is asserted.
        if let Err(e) = syscall_check_int("clock_gettime", unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut timespec)
        }) {
            // This should never happen absent an OS bug.
            panic!(
                "Failed to get current time due to error: {}",
                crate::ffi::NormalizeErrno(&e, None),
            );
        }

        let seconds = Wrapping(reinterpret_i64_u64(timespec.tv_sec));
        let nanos = Wrapping(reinterpret_i64_u64(timespec.tv_nsec));

        SystemdMonotonicUsec((seconds * Wrapping(1_000_000) + nanos / Wrapping(1000)).0)
    }
}

impl NativeSystemdProvider {
    pub const fn new(watchdog_enabled: bool, boot_id: Id128) -> NativeSystemdProvider {
        NativeSystemdProvider {
            watchdog_enabled,
            boot_id,
        }
    }

    pub fn open_provider() -> io::Result<NativeSystemdProvider> {
        let watchdog_enabled = match sd_check(
            "sd_watchdog_enabled",
            // SAFETY: It's just invoking the native systemd function, and invariants are upheld via
            // the function's type.
            unsafe { daemon::sd_watchdog_enabled(0, None.map_or(std::ptr::null_mut(), |s| s)) },
        ) {
            Ok(0) => false,
            Ok(_) => true,
            Err(_) => panic!("`WATCHDOG_USEC` and/or `WATCHDOG_PID` are invalid"),
        };

        Ok(NativeSystemdProvider::new(
            watchdog_enabled,
            Id128::get_from_boot()?,
        ))
    }
}

// Skip in Miri due to FFI calls
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn native_systemd_provider_mirrors_boot_id() {
        let guard = setup_capture_logger();
        static PROVIDER: NativeSystemdProvider = NativeSystemdProvider::new(true, Id128(123));
        assert_eq!(PROVIDER.boot_id(), Id128(123));
        guard.expect_logs(&[]);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn native_systemd_provider_notify_works_with_watchdog_disabled() {
        let guard = setup_capture_logger();
        static PROVIDER: NativeSystemdProvider = NativeSystemdProvider::new(false, Id128(123));
        assert_result_eq(PROVIDER.watchdog_notify(), Ok(()));
        guard.expect_logs(&[]);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn native_systemd_provider_notify_works_with_watchdog_enabled() {
        let guard = setup_capture_logger();
        static PROVIDER: NativeSystemdProvider = NativeSystemdProvider::new(true, Id128(123));
        // Assert the unit tests are running outside a service.
        assert_result_eq(
            PROVIDER.watchdog_notify(),
            Err(ErrorKind::NotConnected.into()),
        );
        guard.expect_logs(&[]);
    }
}
