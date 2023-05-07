use crate::prelude::*;

use super::*;

fn message_key(
    uid: Option<u32>,
    gid: Option<u32>,
    priority: Priority,
    service: Option<Service>,
) -> MessageKey {
    let mut key = MessageKey::new();

    key.uid = uid;
    key.gid = gid;
    key.priority = priority;

    if let Some(service) = service {
        key.set_service(service);
    }

    key
}

#[test]
fn returns_correct_initial_metrics() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty(),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_fault() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_fault();

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 1,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty(),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_cursor_double_retry() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_cursor_double_retry();

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 1,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty(),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_unreadable_entry() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_unreadable_field();

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 1,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty(),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_corrupted_entry() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_corrupted_field();

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 1,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::empty(),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_set_of_requests() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_metrics_requests(123);

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 123,
            messages_ingested: ByteCountSnapshot::empty(),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_zero_byte_value() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 0],
    );

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build([ByteCountSnapshotEntry {
                name: None,
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 0,
            }]),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_message_without_a_service() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_message_line_ingested(
        &message_key(Some(123), Some(123), Priority::Informational, None),
        &[0; 5],
    );

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build([ByteCountSnapshotEntry {
                name: None,
                key: MessageKey::build(Some(123), Some(123), None, Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_message_without_a_user() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_message_line_ingested(
        &message_key(
            None,
            Some(123),
            Priority::Informational,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build([ByteCountSnapshotEntry {
                name: None,
                key: MessageKey::build(None, Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_message_without_a_group() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            None,
            Priority::Informational,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build([ByteCountSnapshotEntry {
                name: None,
                key: MessageKey::build(Some(123), None, Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_a_single_message() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build([ByteCountSnapshotEntry {
                name: None,
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_two_messages_across_two_services() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(456),
            Priority::Informational,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(456),
            Some(123),
            Priority::Warning,
            Some(Service::from_full_service(b"bar.service").unwrap()),
        ),
        &[0; 7],
    );

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build([
                ByteCountSnapshotEntry {
                    name: None,
                    key: MessageKey::build(
                        Some(456),
                        Some(123),
                        Some(b"bar.service"),
                        Priority::Warning
                    ),
                    lines: 1,
                    bytes: 7,
                },
                ByteCountSnapshotEntry {
                    name: None,
                    key: MessageKey::build(
                        Some(123),
                        Some(456),
                        Some(b"foo"),
                        Priority::Informational
                    ),
                    lines: 1,
                    bytes: 5,
                },
            ]),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_1_fault_and_1_message() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 1,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build([ByteCountSnapshotEntry {
                name: None,
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_10_same_service_messages() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Warning,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Debug,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Emergency,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Critical,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Notice,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Debug,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Alert,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Error,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Warning,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );

    let expected_ingested_message_data_params = [
        (Priority::Emergency, 1, 5),
        (Priority::Alert, 1, 5),
        (Priority::Critical, 1, 5),
        (Priority::Error, 1, 5),
        (Priority::Warning, 2, 10),
        (Priority::Notice, 1, 5),
        (Priority::Informational, 1, 5),
        (Priority::Debug, 2, 10),
    ];

    let expected_messages_ingested = Vec::from_iter(expected_ingested_message_data_params.map(
        |(priority, lines, bytes)| ByteCountSnapshotEntry {
            name: None,
            key: MessageKey::build(Some(123), Some(123), Some(b"foo"), priority),
            lines,
            bytes,
        },
    ));

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build(expected_messages_ingested),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_5_faults_and_10_same_service_messages() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Warning,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Debug,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Emergency,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Critical,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Notice,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_fault();
    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Debug,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Alert,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Error,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Warning,
            Some(Service::from_full_service(b"foo.service").unwrap()),
        ),
        &[0; 5],
    );

    let expected_ingested_message_data_params = [
        (Priority::Emergency, 1, 5),
        (Priority::Alert, 1, 5),
        (Priority::Critical, 1, 5),
        (Priority::Error, 1, 5),
        (Priority::Warning, 2, 10),
        (Priority::Notice, 1, 5),
        (Priority::Informational, 1, 5),
        (Priority::Debug, 2, 10),
    ];

    let expected_messages_ingested = Vec::from_iter(expected_ingested_message_data_params.map(
        |(priority, lines, bytes)| ByteCountSnapshotEntry {
            name: None,
            key: MessageKey::build(Some(123), Some(123), Some(b"foo"), priority),

            lines,
            bytes,
        },
    ));

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 5,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build(expected_messages_ingested),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}

#[test]
fn correctly_tracks_500_faults_and_400_different_service_messages() {
    static STATE: PromState = PromState::new();
    // SAFETY: It's held for the full test, and `S` is only accessible inside the test.
    let _lease = unsafe { STATE.initialize_monitor_filter(None) };

    static SERVICE_NAMES: &[&[u8]] = &[
        b"service1.service",
        b"service2.service",
        b"service3.service",
        b"service4.service",
        b"service5.service",
        b"service6.service",
        b"service7.service",
        b"service8.service",
        b"service9.service",
        b"service10.service",
        b"service11.service",
        b"service12.service",
        b"service13.service",
        b"service14.service",
        b"service15.service",
        b"service16.service",
        b"service17.service",
        b"service18.service",
        b"service19.service",
        b"service20.service",
    ];

    static EXPECTED_INGESTED_MESSAGE_DATA_PARAMS: &[(Priority, u64, u64)] = &[
        (Priority::Emergency, 2, 10),
        (Priority::Alert, 2, 10),
        (Priority::Critical, 2, 10),
        (Priority::Error, 2, 10),
        (Priority::Warning, 4, 20),
        (Priority::Notice, 2, 10),
        (Priority::Informational, 2, 10),
        (Priority::Debug, 4, 20),
    ];

    let mut expected_messages_ingested = vec![];

    for &(priority, lines, bytes) in EXPECTED_INGESTED_MESSAGE_DATA_PARAMS {
        for &service in SERVICE_NAMES {
            expected_messages_ingested.push(ByteCountSnapshotEntry {
                name: None,
                key: MessageKey::build(Some(123), Some(123), Some(service), priority),

                lines,
                bytes,
            })
        }
    }

    for &name in SERVICE_NAMES {
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Warning,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Informational,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Debug,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Emergency,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Critical,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Notice,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Debug,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Alert,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Error,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Warning,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_fault();
    }

    for &name in SERVICE_NAMES.iter().rev() {
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Warning,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Informational,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Debug,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Emergency,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Critical,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Notice,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Debug,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Alert,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Error,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Warning,
                Some(Service::from_full_service(name).unwrap()),
            ),
            &[0; 5],
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_fault();
    }

    assert_eq!(
        STATE.snapshot().unwrap(),
        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 500,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot::build(expected_messages_ingested),
            monitor_hits: ByteCountSnapshot::empty(),
        }
    );
}
