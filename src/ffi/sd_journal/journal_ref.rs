use crate::prelude::*;

use super::Cursor;
use super::Id128;
use super::JournalRef;
use super::NativeSystemdProvider;
use super::SystemdMonotonicUsec;
use crate::ffi::syscall_utils::sd_check;
use libsystemd_sys::journal::*;
use std::ffi::CStr;
use std::ptr::NonNull;

pub struct NativeJournalRef {
    raw: NonNull<sd_journal>,
}

impl JournalRef for NativeJournalRef {
    type Provider = NativeSystemdProvider;

    fn open(_: &'static Self::Provider) -> io::Result<Self> {
        let mut raw = std::ptr::null_mut();
        // SAFETY: It's just invoking the native systemd function, and invariants are upheld via
        // the function's type.
        sd_check("sd_journal_open", unsafe { sd_journal_open(&mut raw, 0) })?;
        Ok(NativeJournalRef {
            raw: NonNull::new(raw).expect("`sd_journal_open` returned null pointer for journal"),
        })
    }

    fn set_data_threshold(&mut self, threshold: usize) -> io::Result<()> {
        // SAFETY: FFI call doesn't modify anything directly observable by safe Rust code.
        sd_check("sd_journal_set_data_threshold", unsafe {
            sd_journal_set_data_threshold(self.raw.as_ptr(), threshold)
        })?;
        Ok(())
    }

    fn seek_monotonic_usec(
        &mut self,
        boot_id: &Id128,
        start_usec: SystemdMonotonicUsec,
    ) -> io::Result<()> {
        // SAFETY: FFI call doesn't modify anything directly observable by safe Rust code.
        sd_check("sd_journal_seek_monotonic_usec", unsafe {
            sd_journal_seek_monotonic_usec(self.raw.as_ptr(), boot_id.as_raw(), start_usec.0)
        })?;
        Ok(())
    }

    fn seek_cursor(&mut self, cursor: &Cursor) -> io::Result<()> {
        // SAFETY: `self.raw` is a valid journal pointer.
        sd_check("sd_journal_seek_cursor", unsafe {
            sd_journal_seek_cursor(self.raw.as_ptr(), cursor.as_ptr())
        })?;
        Ok(())
    }

    /// Returns `true` if things changed, `false` otherwise.
    fn wait(&mut self, duration: Duration) -> io::Result<bool> {
        let timeout_usec = truncate_u128_u64(duration.as_micros());
        // SAFETY: FFI call doesn't modify anything directly observable by safe Rust code.
        let result = sd_check("sd_journal_wait", unsafe {
            sd_journal_wait(self.raw.as_ptr(), timeout_usec)
        })?;
        Ok(result != SD_JOURNAL_NOP)
    }

    /// Returns `true` if things changed, `false` otherwise.
    fn next(&mut self) -> io::Result<bool> {
        // SAFETY: FFI call doesn't modify anything directly observable by safe Rust code.
        let result = sd_check("sd_journal_next", unsafe {
            sd_journal_next(self.raw.as_ptr())
        })?;
        Ok(result != 0)
    }

    /// Returns `true` if things changed, `false` otherwise.
    fn cursor(&mut self) -> io::Result<Cursor> {
        // SAFETY: FFI call only writes to the cursor pointer, and it doesn't modify anything
        // directly observable by safe Rust code. Additionally, the cursor is written out as a
        // non-null pointer and is only read from if the operation itself was successful.
        unsafe {
            let mut cursor = MaybeUninit::uninit();
            sd_check(
                "sd_journal_get_cursor",
                sd_journal_get_cursor(self.raw.as_ptr(), cursor.as_mut_ptr()),
            )?;

            match NonNull::new(cursor.assume_init().cast_mut()) {
                None => Err(error!(
                    ErrorKind::InvalidData,
                    "`sd_journal_get_cursor` returned an empty pointer"
                )),
                Some(raw) => Ok(Cursor::from_raw(FixedCString::from_ptr(raw))),
            }
        }
    }

    // The lifetime here is to assert it doesn't get re-called while the result is borrowed.
    fn get_data<'a>(&'a mut self, field: &CStr) -> io::Result<&'a [u8]> {
        // SAFETY: all pointers are initialized after the call.
        unsafe {
            let mut data = MaybeUninit::uninit();
            let mut len = MaybeUninit::uninit();

            sd_check(
                "sd_journal_get_data",
                sd_journal_get_data(
                    self.raw.as_ptr(),
                    field.as_ptr(),
                    data.as_mut_ptr(),
                    len.as_mut_ptr(),
                ),
            )?;

            // Key with trailing null byte (what's received) has the same length as the
            // field suffixed with an equals sign (what's given)
            let prefix_len = field.to_bytes_with_nul().len();

            // Saturate to zero in case systemd returns a length less than the field
            // prefix length (which would be against its API contract). Easier to
            // tolerate than to debug later.
            return Ok(
                &std::slice::from_raw_parts(data.assume_init(), len.assume_init())[prefix_len..],
            );
        }
    }
}

#[cfg(test)]
impl NativeJournalRef {
    pub fn get_data_threshold(&mut self) -> io::Result<usize> {
        let mut threshold = 0;
        // SAFETY: It's just invoking the native systemd function, and invariants are upheld via
        // the function's type.
        sd_check("sd_journal_get_data_threshold", unsafe {
            sd_journal_get_data_threshold(self.raw.as_ptr(), &mut threshold)
        })?;

        Ok(threshold)
    }
}

impl Drop for NativeJournalRef {
    fn drop(&mut self) {
        // SAFETY: FFI call doesn't expose anything to safe Rust code.
        unsafe {
            sd_journal_close(self.raw.as_ptr());
        }
    }
}

// Skip in Miri - it's all just testing interaction through FFI, and Miri's not likely to support
// this ever.
#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;

    use crate::ffi::syscall_utils::sd_check;
    use crate::ffi::NativeSystemdProvider;
    use crate::ffi::SystemdProvider;
    use std::collections::HashMap;
    use std::ffi::CString;

    struct LazyProvider {
        inner: OnceCell<io::Result<NativeSystemdProvider>>,
    }

    impl LazyProvider {
        const fn new() -> LazyProvider {
            LazyProvider {
                inner: OnceCell::new(),
            }
        }

        fn get(&self) -> &NativeSystemdProvider {
            let inner = self.inner.get_or_init(NativeSystemdProvider::open_provider);

            inner.as_ref().unwrap()
        }
    }

    const EXPECTED_TIME_TO_FLUSH_TO_JOURNAL: Duration = Duration::from_millis(10);
    const MAX_TIME_TO_FLUSH_TO_JOURNAL: Duration = Duration::from_secs(5);

    fn journal_send(entries: &[&'static [u8]]) {
        let mut iovecs = Vec::new();

        for entry in entries {
            iovecs.push(libsystemd_sys::const_iovec {
                iov_base: entry.as_ptr().cast(),
                iov_len: entry.len(),
            });
        }

        // SAFETY: FFI call initialized with all pointers correct.
        sd_check("sd_journal_sendv", unsafe {
            sd_journal_sendv(iovecs.as_ptr(), truncate_usize_c_int(iovecs.len()))
        })
        .unwrap();
    }

    #[test]
    fn open_works() {
        let guard = setup_capture_logger();
        static PROVIDER: LazyProvider = LazyProvider::new();

        assert!(
            NativeJournalRef::open(PROVIDER.get()).is_ok(),
            "Journal ref opened correctly."
        );
        guard.expect_logs(&[]);
    }

    #[test]
    fn finds_logs() {
        let guard = setup_capture_logger();
        static PROVIDER: LazyProvider = LazyProvider::new();

        assert!(
            NativeJournalRef::open(PROVIDER.get()).is_ok(),
            "Journal ref opened correctly."
        );
        guard.expect_logs(&[]);
    }

    #[test]
    fn set_data_threshold_works() {
        let guard = setup_capture_logger();
        static PROVIDER: LazyProvider = LazyProvider::new();

        let mut journal = NativeJournalRef::open(PROVIDER.get()).unwrap();
        assert_result_eq(journal.get_data_threshold(), Ok(64 * 1024));
        assert_result_eq(journal.set_data_threshold(1234), Ok(()));
        assert_result_eq(journal.get_data_threshold(), Ok(1234));
        guard.expect_logs(&[]);
    }

    #[test]
    fn finds_new_journal_entries() {
        static PROVIDERS: PerAttemptStatic<LazyProvider, 3> = PerAttemptStatic::new([
            LazyProvider::new(),
            LazyProvider::new(),
            LazyProvider::new(),
        ]);

        with_attempts(3, 0.5, &|| {
            let guard = setup_capture_logger();
            let provider = PROVIDERS.next().get();
            let start_usec = provider.get_monotonic_time_usec();

            let start = Instant::now();

            journal_send(&[
                b"TEST_LABEL_ONE=journald-exporter native_journal_ref_finds_new_journal_entries 1",
                b"TEST_LABEL_TWO=journald-exporter native_journal_ref_finds_new_journal_entries 2",
                b"TEST_LABEL_THREE=journald-exporter native_journal_ref_finds_new_journal_entries 3",
            ]);

            let mut journal = NativeJournalRef::open(provider).unwrap();

            journal
                .seek_monotonic_usec(provider.boot_id(), start_usec)
                .unwrap();

            let end = start + MAX_TIME_TO_FLUSH_TO_JOURNAL;

            let cursor = check_for_entry(&mut journal, end);
            journal.seek_cursor(cursor.as_ref().unwrap()).unwrap();
            check_for_entry(&mut journal, end);
            guard.expect_logs(&[]);
        });

        fn get_data(journal: &mut NativeJournalRef, field: &'static str) -> io::Result<Box<str>> {
            Ok(
                String::from_utf8_lossy(journal.get_data(&CString::new(field).unwrap())?)
                    .into_owned()
                    .into(),
            )
        }

        fn check_for_entry(journal: &mut NativeJournalRef, end: Instant) -> Option<Cursor> {
            let result = 'read: loop {
                if !journal.wait(EXPECTED_TIME_TO_FLUSH_TO_JOURNAL).unwrap() {
                    if Instant::now() >= end {
                        break 'read None;
                    }
                    continue 'read;
                }

                while journal.next().unwrap() {
                    let test_label_1 = match get_data(journal, "TEST_LABEL_ONE") {
                        Ok(entry) => entry,
                        Err(e) => match e.raw_os_error() {
                            Some(libc::ENOENT | libc::E2BIG | libc::ENOBUFS | libc::EBADMSG) => {
                                if Instant::now() >= end {
                                    break 'read None;
                                }
                                continue;
                            }
                            _ => panic!("Error while reading entry TEST_LABEL_ONE: {e:?}"),
                        },
                    };

                    let test_label_2 = get_data(journal, "TEST_LABEL_TWO")
                        .expect("Error while reading entry TEST_LABEL_TWO");

                    let test_label_3 = get_data(journal, "TEST_LABEL_THREE")
                        .expect("Error while reading entry TEST_LABEL_THREE");

                    break 'read Some((
                        journal.cursor().unwrap(),
                        [test_label_1, test_label_2, test_label_3],
                    ));
                }
            };

            assert_eq!(
                result.as_ref().map(|r| &r.1),
                Some(&[
                    "journald-exporter native_journal_ref_finds_new_journal_entries 1".into(),
                    "journald-exporter native_journal_ref_finds_new_journal_entries 2".into(),
                    "journald-exporter native_journal_ref_finds_new_journal_entries 3".into(),
                ]),
            );

            result.map(|r| r.0)
        }
    }
}
