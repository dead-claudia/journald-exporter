use crate::prelude::*;

use super::ipc::ParentIpcState;
use super::journal::run_journal_loop;
use crate::ffi::Cursor;
use crate::ffi::FakeJournalRef;
use crate::ffi::FakeSystemdProvider;
use crate::ffi::Id128;
use crate::parent::ipc::mocks::FakeIpcChildHandle;

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

    #[track_caller]
    fn push_field(&'static self, key: &'static [u8], value: Result<&'static [u8], i32>) {
        self.provider
            .journal
            .get_data
            .enqueue_io(FixedCString::new(key), value);
    }

    fn push_entry(&'static self, entry: Entry) {
        self.push_field(b"_SYSTEMD_UNIT", entry.unit);
        self.push_field(b"PRIORITY", entry.priority);
        self.push_field(b"_UID", entry.uid);
        self.push_field(b"_GID", entry.gid);
        self.push_field(b"MESSAGE", entry.message);
    }
}

#[test]
fn aborts_on_initial_watchdog_error() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Err(libc::EIO));

    assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
    logger_guard.expect_logs(&[]);

    assert_eq!(
        T.snapshot(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
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
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn aborts_on_early_fatal_open_failure() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Err(libc::EIO));

    assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::EIO)));
    logger_guard.expect_logs(&[]);
    assert_eq!(
        T.snapshot(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn aborts_on_early_fatal_set_data_threshold() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider
        .journal
        .set_data_threshold
        .enqueue_io(Err(libc::ENOMEM));

    assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::ENOMEM)));
    logger_guard.expect_logs(&[]);
    assert_eq!(
        T.snapshot(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn aborts_on_early_fatal_seek() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider
        .journal
        .seek_monotonic_usec
        .enqueue_io(Err(libc::ECHILD));

    assert_result_eq(T.start(), Err(Error::from_raw_os_error(libc::ECHILD)));
    logger_guard.expect_logs(&[]);
    assert_eq!(
        T.snapshot(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn retries_on_out_of_descriptors() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Err(libc::EMFILE));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Err(libc::ENFILE));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 0,
            faults: 2,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn retries_on_connection_error() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::ECONNRESET));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::ECONNABORTED));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 0,
            faults: 2,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn aborts_on_io_error_during_wait() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn aborts_on_memory_error_during_next() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Err(libc::ENOMEM));

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
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn aborts_on_missing_cursor_after_next() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider.journal.cursor.enqueue_io(Err(libc::ENOENT));

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
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty()
        },
    );
    T.provider.assert_no_calls_remaining();
}

#[test]
fn pushes_entry_then_aborts_on_wait_error() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"some text"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 34,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
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

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b""),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 25,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
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

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 1")));
    T.push_entry(Entry {
        unit: Err(libc::ENOENT),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 2")));
    T.push_entry(Entry {
        unit: Err(libc::E2BIG),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 3")));
    T.push_entry(Entry {
        unit: Err(libc::ENOBUFS),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 4")));
    T.push_entry(Entry {
        unit: Err(libc::EBADMSG),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 56,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 2,
            corrupted_fields: 1,
            metrics_requests: 0,
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

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 1")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Err(libc::ENOENT),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 2")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Err(libc::E2BIG),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 3")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Err(libc::ENOBUFS),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 4")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        uid: Ok(b"123"),
        priority: Err(libc::EBADMSG),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 124,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 2,
            corrupted_fields: 1,
            metrics_requests: 0,
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
fn processes_invalid_priority_then_aborts_on_wait_error() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 1")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"wut"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 34,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 1,
            corrupted_fields: 0,
            metrics_requests: 0,
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
fn processes_unreadable_uids_then_aborts_on_wait_error() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 1")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Err(libc::ENOENT),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 2")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Err(libc::E2BIG),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 3")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Err(libc::ENOBUFS),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 4")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Err(libc::EBADMSG),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 116,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 2,
            corrupted_fields: 1,
            metrics_requests: 0,
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
fn processes_invalid_uids_then_aborts_on_wait_error() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 1")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"wut"),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 2")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Err(libc::E2BIG),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 3")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Err(libc::ENOBUFS),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 4")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Err(libc::EBADMSG),
        gid: Ok(b"123"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 119,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 3,
            corrupted_fields: 1,
            metrics_requests: 0,
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

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 1")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Err(libc::ENOENT),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 2")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Err(libc::E2BIG),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 3")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Err(libc::ENOBUFS),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 4")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Err(libc::EBADMSG),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 116,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 2,
            corrupted_fields: 1,
            metrics_requests: 0,
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
fn processes_invalid_gids_then_aborts_on_wait_error() {
    let logger_guard = setup_capture_logger();
    static T: TestState = TestState::init();

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 1")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"wut"),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 2")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Err(libc::E2BIG),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 3")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Err(libc::ENOBUFS),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 4")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Err(libc::EBADMSG),
        message: Ok(b"message"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 119,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 3,
            corrupted_fields: 1,
            metrics_requests: 0,
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

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 1")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Err(libc::ENOENT),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 2")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Err(libc::E2BIG),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 3")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Err(libc::ENOBUFS),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor 4")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"123"),
        message: Err(libc::EBADMSG),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 100,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 2,
            corrupted_fields: 1,
            metrics_requests: 0,
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

    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.open.enqueue_io(Ok(()));
    T.provider.journal.set_data_threshold.enqueue_io(Ok(()));
    T.provider.get_monotonic_time_usec.enqueue(123_000_000_000);
    T.provider.journal.seek_monotonic_usec.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Ok(true));
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"456"),
        message: Ok(b"some text 1"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"4"),
        uid: Ok(b"123"),
        gid: Ok(b"456"),
        message: Ok(b"some text 2"),
    });
    T.provider.journal.next.enqueue_io(Ok(true));
    T.provider
        .journal
        .cursor
        .enqueue_io(Ok(Cursor::new(b"test cursor")));
    T.push_entry(Entry {
        unit: Ok(b"my-service.service"),
        priority: Ok(b"6"),
        uid: Ok(b"456"),
        gid: Ok(b"123"),
        message: Ok(b"some text 3"),
    });
    T.provider.journal.next.enqueue_io(Ok(false));
    T.provider.watchdog_notify.enqueue_io(Ok(()));
    T.provider.journal.wait.enqueue_io(Err(libc::EIO));

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
            data_ingested_bytes: 108,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
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
