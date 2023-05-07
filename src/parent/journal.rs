use crate::prelude::*;

use super::ipc::ParentIpcMethods;
use super::ipc::ParentIpcState;
use crate::ffi::Cursor;
use crate::ffi::JournalRef;
use crate::ffi::SystemdMonotonicUsec;
use crate::ffi::SystemdProvider;
use crate::parent::utils::WatchdogCounter;
use const_str::cstr;
use std::ffi::CStr;

static MESSAGE: &CStr = cstr!("MESSAGE");
static PRIORITY: &CStr = cstr!("PRIORITY");
static UID: &CStr = cstr!("_UID");
static GID: &CStr = cstr!("_GID");
static SYSTEMD_UNIT: &CStr = cstr!("_SYSTEMD_UNIT");

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

    fn try_read_id(
        &mut self,
        j: &mut impl JournalRef,
        field: &'static CStr,
        unreadable_name: &mut Option<Box<[u8]>>,
        target: &mut Option<u32>,
    ) -> io::Result<bool> {
        let result = self.get_data(j, field)?;

        if self.state.terminate_notify().has_notified() {
            return Ok(false);
        }

        // Omission is okay.
        if let Some(id_bytes) = result {
            match parse_u32(id_bytes) {
                Some(id) => *target = Some(id),
                None => self.report_unreadable(unreadable_name, id_bytes),
            }
        }

        Ok(true)
    }
}

struct Malformed {
    service: Option<Box<[u8]>>,
    priority: Option<Box<[u8]>>,
    uid: Option<Box<[u8]>>,
    gid: Option<Box<[u8]>>,
}

struct MessageReader<M: ParentIpcMethods + 'static> {
    inner: MessageReaderState<M>,
    malformed: Malformed,
    key: MessageKey,
}

impl<M: ParentIpcMethods> MessageReader<M> {
    fn new(state: &'static ParentIpcState<M>) -> Self {
        Self {
            inner: MessageReaderState::new(state),
            malformed: Malformed {
                service: None,
                priority: None,
                uid: None,
                gid: None,
            },
            key: MessageKey::new(),
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
            match Service::from_full_service(name) {
                Ok(service) => {
                    self.key.set_service(service);
                }
                Err(ServiceParseError::Empty) => {
                    // Treat as missing
                }
                Err(ServiceParseError::Invalid) => {
                    self.inner.service_error_type = ServiceErrorType::Invalid;
                    self.inner
                        .report_unreadable(&mut self.malformed.service, name);
                }
                Err(ServiceParseError::TooLong) => {
                    self.inner.service_error_type = ServiceErrorType::TooLong;
                    self.inner
                        .report_unreadable(&mut self.malformed.service, name);
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
                Ok(priority) => self.key.priority = priority,
                // If it somehow has an invalid label, treat it as missing. (See below for why it's
                // set to `Debug`.)
                Err(PriorityParseError::Empty) => self.key.priority = Priority::Debug,
                Err(PriorityParseError::Invalid) => {
                    self.inner
                        .report_unreadable(&mut self.malformed.priority, value);
                }
            }
        } else {
            // If there's no priority label, fall back to the lowest priority, as it's probably
            // just a control message or something.
            self.key.priority = Priority::Debug;
        }

        Ok(true)
    }

    fn try_read_uid(&mut self, j: &mut impl JournalRef) -> io::Result<bool> {
        self.inner
            .try_read_id(j, UID, &mut self.malformed.uid, &mut self.key.uid)
    }

    fn try_read_gid(&mut self, j: &mut impl JournalRef) -> io::Result<bool> {
        self.inner
            .try_read_id(j, GID, &mut self.malformed.gid, &mut self.key.gid)
    }

    fn try_read_msg(&mut self, j: &mut impl JournalRef) -> io::Result<()> {
        if self.try_read_service(j)?
            && self.try_read_priority(j)?
            && self.try_read_uid(j)?
            && self.try_read_gid(j)?
        {
            // Fall back to an empty "message" if missing.
            let msg = self.inner.get_data(j, MESSAGE)?.unwrap_or(b"");

            self.inner
                .state
                .state()
                .add_message_line_ingested(&self.key, msg);
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

        fn emit_small_malformed_unit_value(unit: &str, field_name: &str, field_value: &[u8]) {
            let mut result = String::new();
            result.push_str("Received malformed field '");
            result.push_str(field_name);
            result.push_str("' in message from unit '");
            result.push_str(unit);
            result.push_str("': '");
            emit_truncatable_value_and_tail(result, field_value);
        }

        fn emit_truncatable_value_and_tail(mut result: String, field_value: &[u8]) {
            binary_to_display(
                &mut result,
                &field_value[..MAX_SERVICE_LEN.min(field_value.len())],
            );
            if field_value.len() >= MAX_SERVICE_LEN {
                result.push_str("...' (truncated)");
            } else {
                result.push('\'');
            }
            log::warn!("{}", result);
        }

        if let Some(field_value) = &self.malformed.service {
            let prefix = match self.inner.service_error_type {
                ServiceErrorType::TooLong => {
                    "Received too-long field '_SYSTEMD_UNIT' in message: '"
                }
                ServiceErrorType::Invalid => {
                    "Received malformed field '_SYSTEMD_UNIT' in message: '"
                }
            };

            let mut result = String::new();
            result.push_str(prefix);
            // Don't show the whole string - it's a waste of memory and storage and,
            // since this utility also sees the messages it generates, it could result
            // in breaching the data threshold and resulting in a much less informative
            // error instead. The `.min` is so it doesn't break in test with the much
            // smaller strings.
            emit_truncatable_value_and_tail(result, field_value);
        }

        if let Some(field_value) = &self.malformed.priority {
            // Anything longer than 1 character is invalid, so it's okay to truncate to 256
            emit_small_malformed_unit_value(unit, "PRIORITY", field_value);
        }

        if let Some(field_value) = &self.malformed.uid {
            // Anything longer than 5 characters is invalid, so it's okay to truncate to 256
            emit_small_malformed_unit_value(unit, "_UID", field_value);
        }

        if let Some(field_value) = &self.malformed.gid {
            // Anything longer than 5 characters is invalid, so it's okay to truncate to 256
            emit_small_malformed_unit_value(unit, "_GID", field_value);
        }
    }
}

// 5 minutes is more than enough to process 100k+ entries, considering that many entries *per
// second* is more realistic. At 100k entries per second, that's 30M entries within 5 minutes.
// Also, this limit is only applied when no wait is needed, and it'd take a *lot* of throughput
// to even get that far. (I'm not even sure systemd can even saturate the code enough to even hit
// a small fraction of this before the next wait on most machines.)
const FORCE_REPORT_INTERVAL_ENTRIES: usize = 100_000;

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
        if s.terminate_notify().has_notified() {
            return Ok(());
        }

        if journal.wait(Duration::from_secs(1))? {
            if s.terminate_notify().has_notified() {
                return Ok(());
            }

            let mut watchdog_counter = WatchdogCounter::<FORCE_REPORT_INTERVAL_ENTRIES>::new();

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
        let prev_cursor = resume_cursor.take();

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
                    if matches!((&resume_cursor, prev_cursor), (Some(a), Some(b)) if a == &b) {
                        s.state().add_cursor_double_retry();
                        return Err(error!("Cursor read failed after 2 attempts."));
                    }

                    if s.terminate_notify().has_notified() {
                        return Ok(());
                    }

                    log::warn!(
                        "Fatal journal processing error: {}",
                        normalize_errno(e, None)
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
