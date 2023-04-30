use crate::prelude::*;

use super::Id128;
use super::SystemdMonotonicUsec;
use super::SystemdProvider;
use crate::ffi::syscall_utils::sd_check;
use crate::ffi::syscall_utils::syscall_assert_int;
use const_str::cstr;
use libsystemd_sys::daemon;
use std::ffi::CStr;

static WATCHDOG_MSG: &CStr = cstr!("WATCHDOG=1");

pub struct NativeSystemdProvider {
    watchdog_usec: SystemdMonotonicUsec,
    last_watchdog: Mutex<SystemdMonotonicUsec>,
    boot_id: Id128,
}

impl SystemdProvider for NativeSystemdProvider {
    fn watchdog_notify(&'static self) -> io::Result<()> {
        self.sd_notify(WATCHDOG_MSG)
    }

    fn boot_id(&self) -> &Id128 {
        &self.boot_id
    }

    // I have to use the underlying syscall, as systemd expects the raw value and not any sort of
    // time delta.
    fn get_monotonic_time_usec(&'static self) -> SystemdMonotonicUsec {
        // Note: this *is* safe for Miri: https://github.com/rust-lang/miri/issues/641

        let mut timespec = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        // SAFETY: it's only passed in valid addresses, and the result is asserted.
        syscall_assert_int("clock_gettime", unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut timespec)
        });

        let seconds = Wrapping(reinterpret_i64_u64(timespec.tv_sec));
        let nanos = Wrapping(reinterpret_i64_u64(timespec.tv_nsec));

        SystemdMonotonicUsec((seconds * Wrapping(1_000_000) + nanos / Wrapping(1000)).0)
    }
}

impl NativeSystemdProvider {
    pub const fn new(watchdog_usec: u64, boot_id: Id128) -> NativeSystemdProvider {
        NativeSystemdProvider {
            // Give some cushion for missed watchdogs.
            watchdog_usec: SystemdMonotonicUsec(watchdog_usec / 3),
            last_watchdog: Mutex::new(SystemdMonotonicUsec(0)),
            boot_id,
        }
    }

    pub fn open_provider() -> io::Result<NativeSystemdProvider> {
        let boot_id = Id128::get_from_boot()?;

        let mut watchdog_usec = 0;

        if !cfg!(miri) {
            // SAFETY: It's just invoking the native systemd function, and nothing here accesses
            // any Rust-visible memory.
            match sd_check("sd_watchdog_enabled", unsafe {
                daemon::sd_watchdog_enabled(0, &mut watchdog_usec)
            }) {
                Ok(0) => watchdog_usec = 0,
                Ok(_) => {}
                Err(_) => {
                    std::panic::panic_any("`WATCHDOG_USEC` and/or `WATCHDOG_PID` are invalid")
                }
            }
        }

        Ok(NativeSystemdProvider::new(watchdog_usec, boot_id))
    }

    pub fn sd_notify(&'static self, msg: &CStr) -> io::Result<()> {
        if self.watchdog_usec != SystemdMonotonicUsec(0) {
            if cfg!(miri) {
                return Err(Error::from_raw_os_error(libc::ENOTCONN));
            }

            let next: SystemdMonotonicUsec = self.get_monotonic_time_usec();

            // Easier to use a mutex. It's almost never contended, so shouldn't be an issue in
            // practice.
            {
                let mut prev = self.last_watchdog.lock().unwrap_or_else(|e| e.into_inner());
                if prev.0 >= next.0 {
                    return Ok(());
                }
                prev.0 = next.0.wrapping_add(self.watchdog_usec.0);
            }

            // SAFETY: It's just invoking the native systemd function, and invariants are upheld via the
            // function's type.
            let result = sd_check("sd_notify", unsafe { daemon::sd_notify(0, msg.as_ptr()) })?;

            if result == 0 {
                return Err(Error::from_raw_os_error(libc::ENOTCONN));
            }
        }

        Ok(())
    }
}

// Skip in Miri due to FFI calls
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_systemd_provider_mirrors_boot_id() {
        static PROVIDER: NativeSystemdProvider = NativeSystemdProvider::new(111, Id128(123));
        assert_eq!(PROVIDER.boot_id(), &Id128(123));
    }

    #[test]
    fn native_systemd_provider_get_monotonic_usec_works() {
        static PROVIDER: NativeSystemdProvider = NativeSystemdProvider::new(111, Id128(123));
        // Just needs to run.
        let _time = PROVIDER.get_monotonic_time_usec();
    }

    #[test]
    fn native_systemd_provider_notify_works_with_watchdog_disabled() {
        static PROVIDER: NativeSystemdProvider = NativeSystemdProvider::new(0, Id128(123));
        assert_result_eq(PROVIDER.watchdog_notify(), Ok(()));
    }

    #[test]
    fn native_systemd_provider_notify_works_with_watchdog_enabled() {
        static PROVIDER: NativeSystemdProvider = NativeSystemdProvider::new(111, Id128(123));
        // Assert the unit tests are running outside a service.
        assert_result_eq(
            PROVIDER.watchdog_notify(),
            Err(Error::from_raw_os_error(libc::ENOTCONN)),
        );
    }
}
