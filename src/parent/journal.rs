use crate::prelude::*;

use super::ipc_state::ParentIpcState;
use super::types::ParentIpcMethods;
use crate::common::*;
use crate::ffi::Cursor;
use crate::ffi::JournalRef;
use crate::ffi::NormalizeErrno;
use crate::ffi::SystemdMonotonicUsec;
use crate::ffi::SystemdProvider;
use crate::parent::watchdog_counter::WatchdogCounter;
use std::ffi::CStr;

static MESSAGE: &CStr = c_str(b"MESSAGE\0");
static PRIORITY: &CStr = c_str(b"PRIORITY\0");
static UID: &CStr = c_str(b"_UID\0");
static GID: &CStr = c_str(b"_GID\0");
static SYSTEMD_UNIT: &CStr = c_str(b"_SYSTEMD_UNIT\0");

enum ServiceErrorType {
    Invalid,
    TooLong,
}

struct MessageReaderState<M: ParentIpcMethods + 'static> {
    state: &'static ParentIpcState<M>,
    reported_unreadable: bool,
    reported_error: bool,
    service_error_type: ServiceErrorType,
}

impl<M: ParentIpcMethods> MessageReaderState<M> {
    fn new(state: &'static ParentIpcState<M>) -> Self {
        Self {
            state,
            reported_unreadable: false,
            reported_error: false,
            service_error_type: ServiceErrorType::Invalid,
        }
    }

    #[cold]
    fn report_unreadable(&mut self, name: &mut Option<Box<[u8]>>, value: &[u8]) {
        self.reported_unreadable = true;
        self.reported_error = true;
        *name = Some(value.into());
        self.state.state().add_unreadable_field();
    }

    fn get_data<'a>(
        &mut self,
        j: &'a mut impl JournalRef,
        field: &'static CStr,
    ) -> io::Result<Option<&'a [u8]>> {
        match j.get_data(field) {
            Ok(name) => {
                self.state.state().add_field_ingested(name.len());
                Ok(Some(name))
            }
            Err(e) => match e.raw_os_error() {
                // Data field missing
                Some(libc::ENOENT) => Ok(None),
                // Data field too large for architecture.
                Some(libc::E2BIG) => {
                    self.state.state().add_unreadable_field();
                    Ok(None)
                }
                // Compressed entry too large.
                Some(libc::ENOBUFS) => {
                    self.state.state().add_unreadable_field();
                    Ok(None)
                }
                // Entry is corrupted.
                Some(libc::EBADMSG) => {
                    self.state.state().add_corrupted_field();
                    Ok(None)
                }
                // Other errors I'm not really able to tolerate.
                _ => Err(e),
            },
        }
    }
}

struct MessageReader<M: ParentIpcMethods + 'static> {
    inner: MessageReaderState<M>,
    malformed_service: Option<Box<[u8]>>,
    malformed_priority: Option<Box<[u8]>>,
    malformed_uid: Option<Box<[u8]>>,
    malformed_gid: Option<Box<[u8]>>,
    key: MessageKey,
}

impl<M: ParentIpcMethods> MessageReader<M> {
    fn new(state: &'static ParentIpcState<M>) -> Self {
        Self {
            key: MessageKey::new(),
            malformed_service: None,
            malformed_priority: None,
            malformed_uid: None,
            malformed_gid: None,
            inner: MessageReaderState::new(state),
        }
    }

    // `true` means continue, `false` or error means abort.
    fn try_read_service(&mut self, j: &mut impl JournalRef) -> io::Result<bool> {
        let result = self.inner.get_data(j, SYSTEMD_UNIT)?;

        if self.inner.state.terminate_notify().has_notified() {
            return Ok(false);
        }

        // It's okay if the service is missing - it's common for things like cron jobs and other
        // things that write logs to syslog directly rather than through systemd's mechanisms. It
        // also includes cases like corrupted service names, which are easier to just tolerate.
        if let Some(name) = result {
            match Service::from_slice(name) {
                Ok(service) => {
                    self.key.set_service(service);
                }
                Err(ServiceParseError::Empty) => {
                    // Treat as missing
                }
                Err(ServiceParseError::Invalid) => {
                    self.inner.service_error_type = ServiceErrorType::Invalid;
                    self.inner
                        .report_unreadable(&mut self.malformed_service, name);
                }
                Err(ServiceParseError::TooLong) => {
                    self.inner.service_error_type = ServiceErrorType::TooLong;
                    self.inner
                        .report_unreadable(&mut self.malformed_service, name);
                }
            }
        }

        Ok(true)
    }

    fn try_read_priority(&mut self, j: &mut impl JournalRef) -> io::Result<bool> {
        let result = self.inner.get_data(j, PRIORITY)?;

        if self.inner.state.terminate_notify().has_notified() {
            return Ok(false);
        }

        if let Some(value) = result {
            match Priority::from_severity_value(value) {
                Ok(priority) => self.key.set_priority(priority),
                // If it somehow has an invalid label, treat it as missing. (See below for why it's
                // set to `Debug`.)
                Err(PriorityParseError::Empty) => self.key.set_priority(Priority::Debug),
                Err(PriorityParseError::Invalid) => {
                    self.inner
                        .report_unreadable(&mut self.malformed_priority, value);
                }
            }
        } else {
            // If there's no priority label, fall back to the lowest priority, as it's probably
            // just a control message or something.
            self.key.set_priority(Priority::Debug);
        }

        Ok(true)
    }

    fn try_read_uid(&mut self, j: &mut impl JournalRef) -> io::Result<bool> {
        let result = self.inner.get_data(j, UID)?;

        if self.inner.state.terminate_notify().has_notified() {
            return Ok(false);
        }

        // Omission is okay.
        if let Some(id_bytes) = result {
            match parse_u32(id_bytes) {
                Some(id) => self.key.set_uid(id.into()),
                None => {
                    self.inner
                        .report_unreadable(&mut self.malformed_uid, id_bytes);
                }
            }
        }

        Ok(true)
    }

    fn try_read_gid(&mut self, j: &mut impl JournalRef) -> io::Result<bool> {
        let result = self.inner.get_data(j, GID)?;

        if self.inner.state.terminate_notify().has_notified() {
            return Ok(false);
        }

        // Omission is okay.
        if let Some(id_bytes) = result {
            match parse_u32(id_bytes) {
                Some(id) => self.key.set_gid(id.into()),
                None => {
                    self.inner
                        .report_unreadable(&mut self.malformed_gid, id_bytes);
                }
            }
        }

        Ok(true)
    }

    fn try_read_msg(&mut self, j: &mut impl JournalRef) -> io::Result<()> {
        if self.try_read_service(j)?
            && self.try_read_priority(j)?
            && self.try_read_uid(j)?
            && self.try_read_gid(j)?
        {
            // Fall back to a "message length" of 0 if missing.
            let msg_len = self.inner.get_data(j, MESSAGE)?.map_or(0, |msg| msg.len());

            // No need to check. It'll get checked after this function returns anyways, and the
            // below step is fairly trivial.

            self.inner
                .state
                .state()
                .add_message_line_ingested(&self.key, msg_len);
        }

        Ok(())
    }

    // This is in the error path. Keep it out of the main path, as it usually indicates either very
    // exceptional conditions or much deeper issues.
    // Returns `true` if it should continue reading more, `false` or an error if not.
    #[inline(never)]
    #[cold]
    fn report_read_errors(&self) {
        let service_ref = self.key.service();
        let unit = match &service_ref {
            Some(service) => service.as_str(),
            None => "(unknown)",
        };

        if let Some(field_value) = &self.malformed_service {
            match self.inner.service_error_type {
                ServiceErrorType::TooLong => {
                    log::warn!(
                        "Received too-long field '_SYSTEMD_UNIT' in message: '{}...' (truncated)",
                        // Don't show the whole string - it's a waste of memory and storage and,
                        // since this utility also sees the messages it generates, it could result
                        // in breaching the data threshold and resulting in a much less informative
                        // error instead. The `.min` is so it doesn't break in test with the much
                        // smaller strings.
                        BinaryToDisplay(&field_value[..MAX_SERVICE_LEN.min(field_value.len())]),
                    );
                }
                ServiceErrorType::Invalid => {
                    log::warn!(
                        "Received malformed field '_SYSTEMD_UNIT' in message: '{}'",
                        BinaryToDisplay(field_value),
                    );
                }
            }
        }

        if let Some(field_value) = &self.malformed_priority {
            log::warn!(
                "Received malformed field 'PRIORITY' in message from unit '{unit}': '{}'",
                BinaryToDisplay(field_value),
            );
        }

        if let Some(field_value) = &self.malformed_uid {
            log::warn!(
                "Received malformed field '_UID' in message from unit '{unit}': '{}'",
                BinaryToDisplay(field_value),
            );
        }

        if let Some(field_value) = &self.malformed_gid {
            log::warn!(
                "Received malformed field '_GID' in message from unit '{unit}': '{}'",
                BinaryToDisplay(field_value),
            );
        }
    }
}

const FORCE_REPORT_INTERVAL_ENTRIES: usize = 1000;

// Don't inline, as I want to be able to track its existence better in assembly and profiles, and
// in practice, only the inner loop *here* is actually hot.
#[inline(never)]
fn run_loop_inner<J: JournalRef>(
    s: &'static ParentIpcState<impl ParentIpcMethods>,
    provider: &'static J::Provider,
    resume_cursor: &mut Option<Cursor>,
) -> io::Result<()> {
    if s.terminate_notify().has_notified() {
        return Ok(());
    }

    let mut journal = J::open(provider)?;

    // Explicitly specify the default in case it changes.
    const MAX_MESSAGE_DATA_LEN: usize = 64 * 1024;

    if s.terminate_notify().has_notified() {
        return Ok(());
    }

    journal.set_data_threshold(MAX_MESSAGE_DATA_LEN)?;

    if s.terminate_notify().has_notified() {
        return Ok(());
    }

    match resume_cursor {
        None => {
            // Look back up to at most 1 minute. This only is used when first running the journal.
            const LOOKBACK_INTERVAL_USEC: u64 = 60_000_000;

            let current_usec = provider.get_monotonic_time_usec();
            let start_usec =
                SystemdMonotonicUsec(current_usec.0.saturating_sub(LOOKBACK_INTERVAL_USEC));

            if s.terminate_notify().has_notified() {
                return Ok(());
            }

            let boot_id = provider.boot_id();

            if s.terminate_notify().has_notified() {
                return Ok(());
            }

            journal.seek_monotonic_usec(boot_id, start_usec)?;
        }
        Some(cursor) => {
            journal.seek_cursor(cursor)?;
        }
    }

    loop {
        let mut watchdog_counter = WatchdogCounter::<FORCE_REPORT_INTERVAL_ENTRIES>::new();

        if s.terminate_notify().has_notified() {
            return Ok(());
        }

        if journal.wait(Duration::from_secs(1))? {
            if s.terminate_notify().has_notified() {
                return Ok(());
            }

            while journal.next()? {
                if s.terminate_notify().has_notified() {
                    return Ok(());
                }

                s.state().add_entry_ingested();

                // Always save the current cursor, in case it can be retried.
                *resume_cursor = Some(journal.cursor()?);

                if s.terminate_notify().has_notified() {
                    return Ok(());
                }

                let mut reader = MessageReader::new(s);
                let read_msg_result = reader.try_read_msg(&mut journal);

                if reader.inner.reported_error {
                    reader.report_read_errors();
                }

                read_msg_result?;

                if s.terminate_notify().has_notified() {
                    return Ok(());
                }

                if watchdog_counter.hit() {
                    provider.watchdog_notify()?;
                }
            }
        };

        provider.watchdog_notify()?;
    }
}

pub fn run_journal_loop<J: JournalRef>(
    s: &'static ParentIpcState<impl ParentIpcMethods>,
    provider: &'static J::Provider,
) -> io::Result<()> {
    if s.terminate_notify().has_notified() {
        return Ok(());
    }

    provider.watchdog_notify()?;

    // Has to be here so it's thread-local.
    let mut resume_cursor = None;

    loop {
        let prev_cursor = resume_cursor.clone();

        if s.terminate_notify().has_notified() {
            return Ok(());
        }

        match run_loop_inner::<J>(s, provider, &mut resume_cursor) {
            Ok(()) => return Ok(()),
            Err(e) => match e.raw_os_error() {
                Some(
                    libc::EPIPE
                    | libc::EBADF
                    | libc::ECONNRESET
                    | libc::ECONNABORTED
                    | libc::ETIMEDOUT
                    | libc::EMFILE
                    | libc::ENFILE,
                ) => {
                    s.state().add_fault();
                    if resume_cursor.is_some() && resume_cursor == prev_cursor {
                        s.state().add_cursor_double_retry();
                        return Err(Error::new(
                            ErrorKind::Other,
                            "Cursor read failed after 2 attempts.",
                        ));
                    }

                    if s.terminate_notify().has_notified() {
                        return Ok(());
                    }

                    log::warn!(
                        "Fatal journal processing error: {}",
                        NormalizeErrno(&e, None)
                    );

                    if s.terminate_notify().has_notified() {
                        return Ok(());
                    }

                    log::warn!("Restarting journal loop...");

                    if s.terminate_notify().has_notified() {
                        return Ok(());
                    }

                    provider.watchdog_notify()?;
                }
                _ => return Err(e),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ffi::FakeJournalRef;
    use crate::ffi::FakeSystemdProvider;
    use crate::ffi::Id128;
    use crate::parent::ipc_mocks::FakeIpcChildHandle;

    struct TestState {
        state: ParentIpcState<FakeIpcChildHandle>,
        provider: FakeSystemdProvider,
    }

    struct Entry {
        unit: Result<&'static [u8], i32>,
        priority: Result<&'static [u8], i32>,
        uid: Result<&'static [u8], i32>,
        gid: Result<&'static [u8], i32>,
        message: Result<&'static [u8], i32>,
    }

    impl TestState {
        const fn init() -> Self {
            Self {
                state: ParentIpcState::new("/bin/cat", FakeIpcChildHandle::new()),
                provider: FakeSystemdProvider::new(Id128(123)),
            }
        }

        fn start(&'static self) -> io::Result<()> {
            run_journal_loop::<&FakeJournalRef>(&self.state, &self.provider)
        }

        fn snapshot(&'static self) -> PromSnapshot {
            self.state.state().snapshot()
        }

        fn push_field(&'static self, key: &'static CStr, value: Result<&'static [u8], i32>) {
            let key = key.to_owned();
            match value {
                Ok(result) => self.provider.journal.get_data.enqueue_ok(key, result),
                Err(code) => self.provider.journal.get_data.enqueue_err(key, code),
            }
        }

        fn push_entry(&'static self, entry: Entry) {
            static SYSTEMD_UNIT: &CStr = c_str(b"_SYSTEMD_UNIT\0");
            static PRIORITY: &CStr = c_str(b"PRIORITY\0");
            static UID: &CStr = c_str(b"_UID\0");
            static GID: &CStr = c_str(b"_GID\0");
            static MESSAGE: &CStr = c_str(b"MESSAGE\0");

            self.push_field(SYSTEMD_UNIT, entry.unit);
            self.push_field(PRIORITY, entry.priority);
            self.push_field(UID, entry.uid);
            self.push_field(GID, entry.gid);
            self.push_field(MESSAGE, entry.message);
        }
    }

    #[test]
    fn aborts_on_initial_watchdog_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[]);

        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            }
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn aborts_on_initial_termination() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.state.terminate_notify().notify();

        assert_result_eq(T.start(), Ok(()));
        logger_guard.expect_logs(&[]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn aborts_on_early_fatal_open_failure() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn aborts_on_early_fatal_set_data_threshold() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider
            .journal
            .set_data_threshold
            .enqueue_err(libc::ENOMEM);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::ENOMEM)));
        logger_guard.expect_logs(&[]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn aborts_on_early_fatal_seek() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider
            .journal
            .seek_monotonic_usec
            .enqueue_err(libc::ECHILD);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::ECHILD)));
        logger_guard.expect_logs(&[]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn retries_on_out_of_descriptors() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_err(libc::EMFILE);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_err(libc::ENFILE);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[
            "Fatal journal processing error: EMFILE: Too many open files",
            "Restarting journal loop...",
            "Fatal journal processing error: ENFILE: Too many open files in system",
            "Restarting journal loop...",
        ]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 2,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn retries_on_connection_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::ECONNRESET);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::ECONNABORTED);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000), (Id128(123), 122_940_000_000)]);
        logger_guard.expect_logs(&[
            "Fatal journal processing error: ECONNRESET: Connection reset by peer",
            "Restarting journal loop...",
            "Fatal journal processing error: ECONNABORTED: Software caused connection abort",
            "Restarting journal loop...",
        ]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 2,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn aborts_on_io_error_during_wait() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[]);
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn aborts_on_memory_error_during_next() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_err(libc::ENOMEM);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::ENOMEM)));
        logger_guard.expect_logs(&[]);
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn aborts_on_missing_cursor_after_next() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider.journal.cursor.enqueue_err(libc::ENOENT);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::ENOENT)));
        logger_guard.expect_logs(&[]);
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 1,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot::empty()
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn pushes_entry_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"some text"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[]);
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 1,
                fields_ingested: 5,
                data_bytes_ingested: 34,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"my-service.service"),
                            Priority::Warning
                        ),
                        lines: 1,
                        bytes: 9,
                    }])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn pushes_entry_with_empty_message_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b""),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[]);
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 1,
                fields_ingested: 5,
                data_bytes_ingested: 25,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"my-service.service"),
                            Priority::Warning
                        ),
                        lines: 1,
                        bytes: 0,
                    }])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn processes_unreadable_service_names_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 1"));
        T.push_entry(Entry {
            unit: Err(libc::ENOENT),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 2"));
        T.push_entry(Entry {
            unit: Err(libc::E2BIG),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 3"));
        T.push_entry(Entry {
            unit: Err(libc::ENOBUFS),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 4"));
        T.push_entry(Entry {
            unit: Err(libc::EBADMSG),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        logger_guard.expect_logs(&[]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 4,
                fields_ingested: 16,
                data_bytes_ingested: 56,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 2,
                corrupted_fields: 1,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(Some(123), Some(123), None, Priority::Warning),
                        lines: 4,
                        bytes: 28,
                    }])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn processes_unreadable_priorities_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 1"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Err(libc::ENOENT),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 2"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Err(libc::E2BIG),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 3"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Err(libc::ENOBUFS),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 4"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            uid: Ok(b"123"),
            priority: Err(libc::EBADMSG),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        logger_guard.expect_logs(&[]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 4,
                fields_ingested: 16,
                data_bytes_ingested: 124,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 2,
                corrupted_fields: 1,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"my-service.service"),
                            Priority::Debug
                        ),
                        lines: 4,
                        bytes: 28,
                    }])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn processes_invalid_priority_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 1"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"wut"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[
            "Received malformed field 'PRIORITY' in message from unit 'my-service.service': 'wut'",
        ]);
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 1,
                fields_ingested: 5,
                data_bytes_ingested: 34,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 1,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"my-service.service"),
                            Priority::Emergency
                        ),
                        lines: 1,
                        bytes: 7,
                    }],)
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn processes_unreadable_uids_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 1"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Err(libc::ENOENT),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 2"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Err(libc::E2BIG),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 3"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Err(libc::ENOBUFS),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 4"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Err(libc::EBADMSG),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        logger_guard.expect_logs(&[]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 4,
                fields_ingested: 16,
                data_bytes_ingested: 116,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 2,
                corrupted_fields: 1,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            None,
                            Some(123),
                            Some(b"my-service.service"),
                            Priority::Warning
                        ),
                        lines: 4,
                        bytes: 28,
                    }])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn processes_invalid_uids_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 1"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"wut"),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 2"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Err(libc::E2BIG),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 3"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Err(libc::ENOBUFS),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 4"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Err(libc::EBADMSG),
            gid: Ok(b"123"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        logger_guard.expect_logs(&[
            "Received malformed field '_UID' in message from unit 'my-service.service': 'wut'",
        ]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 4,
                fields_ingested: 17,
                data_bytes_ingested: 119,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 3,
                corrupted_fields: 1,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            None,
                            Some(123),
                            Some(b"my-service.service"),
                            Priority::Warning
                        ),
                        lines: 4,
                        bytes: 28,
                    }])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn processes_unreadable_gids_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 1"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Err(libc::ENOENT),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 2"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Err(libc::E2BIG),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 3"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Err(libc::ENOBUFS),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 4"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Err(libc::EBADMSG),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        logger_guard.expect_logs(&[]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 4,
                fields_ingested: 16,
                data_bytes_ingested: 116,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 2,
                corrupted_fields: 1,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            None,
                            Some(b"my-service.service"),
                            Priority::Warning
                        ),
                        lines: 4,
                        bytes: 28,
                    }])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn processes_invalid_gids_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 1"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"wut"),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 2"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Err(libc::E2BIG),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 3"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Err(libc::ENOBUFS),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 4"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Err(libc::EBADMSG),
            message: Ok(b"message"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        logger_guard.expect_logs(&[
            "Received malformed field '_GID' in message from unit 'my-service.service': 'wut'",
        ]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 4,
                fields_ingested: 17,
                data_bytes_ingested: 119,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 3,
                corrupted_fields: 1,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            None,
                            Some(b"my-service.service"),
                            Priority::Warning
                        ),
                        lines: 4,
                        bytes: 28,
                    }])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn processes_unreadable_messages_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 1"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Err(libc::ENOENT),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 2"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Err(libc::E2BIG),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 3"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Err(libc::ENOBUFS),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor 4"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"123"),
            message: Err(libc::EBADMSG),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[]);
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 4,
                fields_ingested: 16,
                data_bytes_ingested: 100,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 2,
                corrupted_fields: 1,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"my-service.service"),
                            Priority::Warning
                        ),
                        lines: 4,
                        bytes: 0,
                    }],)
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }

    #[test]
    fn pushes_multiple_entries_then_aborts_on_wait_error() {
        let logger_guard = setup_capture_logger();
        static T: TestState = TestState::init();

        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.open.enqueue_ok(());
        T.provider.journal.set_data_threshold.enqueue_ok(());
        T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
        T.provider.journal.seek_monotonic_usec.enqueue_ok(());
        T.provider.journal.wait.enqueue_ok(true);
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"456"),
            message: Ok(b"some text 1"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"4"),
            uid: Ok(b"123"),
            gid: Ok(b"456"),
            message: Ok(b"some text 2"),
        });
        T.provider.journal.next.enqueue_ok(true);
        T.provider
            .journal
            .cursor
            .enqueue_ok(Cursor::new(b"test cursor"));
        T.push_entry(Entry {
            unit: Ok(b"my-service.service"),
            priority: Ok(b"6"),
            uid: Ok(b"456"),
            gid: Ok(b"123"),
            message: Ok(b"some text 3"),
        });
        T.provider.journal.next.enqueue_ok(false);
        T.provider.watchdog_notify.enqueue_ok(());
        T.provider.journal.wait.enqueue_err(libc::EIO);

        assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
        logger_guard.expect_logs(&[]);
        T.provider
            .journal
            .seek_monotonic_usec
            .assert_calls(&[(Id128(123), 122_940_000_000)]);
        assert_eq!(
            T.snapshot(),
            PromSnapshot {
                entries_ingested: 3,
                fields_ingested: 15,
                data_bytes_ingested: 108,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot {
                    data: Box::new([
                        ByteCountSnapshotEntry {
                            key: MessageKey::build(
                                Some(123),
                                Some(456),
                                Some(b"my-service.service"),
                                Priority::Warning
                            ),
                            lines: 2,
                            bytes: 22,
                        },
                        ByteCountSnapshotEntry {
                            key: MessageKey::build(
                                Some(456),
                                Some(123),
                                Some(b"my-service.service"),
                                Priority::Informational
                            ),
                            lines: 1,
                            bytes: 11,
                        },
                    ])
                }
            },
        );
        T.provider.assert_no_calls_remaining();
    }
}
