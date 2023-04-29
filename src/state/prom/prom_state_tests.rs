use crate::prelude::*;

use super::*;

fn message_key(
    uid: Option<u32>,
    gid: Option<u32>,
    priority: Priority,
    service: Option<Service>,
) -> MessageKey {
    let mut key = MessageKey::new();

    if let Some(raw) = uid {
        key.table_key.uid = Some(raw);
    }

    if let Some(raw) = gid {
        key.table_key.gid = Some(raw);
    }

    key.priority = priority;

    if let Some(service) = service {
        key.set_service(service);
    }

    key
}

#[test]
fn returns_correct_initial_metrics() {
    static STATE: PromState = PromState::new();

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
        }
    );
}

#[test]
fn correctly_tracks_a_single_fault() {
    static STATE: PromState = PromState::new();

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
        }
    );
}

#[test]
fn correctly_tracks_a_single_cursor_double_retry() {
    static STATE: PromState = PromState::new();

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
        }
    );
}

#[test]
fn correctly_tracks_a_single_unreadable_entry() {
    static STATE: PromState = PromState::new();

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
        }
    );
}

#[test]
fn correctly_tracks_a_single_corrupted_entry() {
    static STATE: PromState = PromState::new();

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
        }
    );
}

#[test]
fn correctly_tracks_a_single_set_of_requests() {
    static STATE: PromState = PromState::new();

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
        }
    );
}

#[test]
fn correctly_tracks_a_single_zero_byte_value() {
    static STATE: PromState = PromState::new();

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        0,
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
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 0,
            }]),
        }
    );
}

#[test]
fn correctly_tracks_a_single_message_without_a_service() {
    static STATE: PromState = PromState::new();

    STATE.add_message_line_ingested(
        &message_key(Some(123), Some(123), Priority::Informational, None),
        5,
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
                key: MessageKey::build(Some(123), Some(123), None, Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
        }
    );
}

#[test]
fn correctly_tracks_a_single_message_without_a_user() {
    static STATE: PromState = PromState::new();

    STATE.add_message_line_ingested(
        &message_key(
            None,
            Some(123),
            Priority::Informational,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
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
                key: MessageKey::build(None, Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
        }
    );
}

#[test]
fn correctly_tracks_a_single_message_without_a_group() {
    static STATE: PromState = PromState::new();

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            None,
            Priority::Informational,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
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
                key: MessageKey::build(Some(123), None, Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
        }
    );
}

#[test]
fn correctly_tracks_a_single_message() {
    static STATE: PromState = PromState::new();

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
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
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
        }
    );
}

#[test]
fn correctly_tracks_two_messages_across_two_services() {
    static STATE: PromState = PromState::new();

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(456),
            Priority::Informational,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(456),
            Some(123),
            Priority::Warning,
            Some(Service::from_slice(b"bar").unwrap()),
        ),
        7,
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
                    key: MessageKey::build(Some(456), Some(123), Some(b"bar"), Priority::Warning),
                    lines: 1,
                    bytes: 7,
                },
                ByteCountSnapshotEntry {
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
        }
    );
}

#[test]
fn correctly_tracks_1_fault_and_1_message() {
    static STATE: PromState = PromState::new();

    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
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
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            }]),
        }
    );
}

#[test]
fn correctly_tracks_10_same_service_messages() {
    static STATE: PromState = PromState::new();

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Warning,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Debug,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Emergency,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Critical,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Notice,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Debug,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Alert,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Error,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Warning,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
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
        }
    );
}

#[test]
fn correctly_tracks_5_faults_and_10_same_service_messages() {
    static STATE: PromState = PromState::new();

    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Warning,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Informational,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Debug,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Emergency,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Critical,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Notice,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_fault();
    STATE.add_fault();
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Debug,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Alert,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Error,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
    );
    STATE.add_message_line_ingested(
        &message_key(
            Some(123),
            Some(123),
            Priority::Warning,
            Some(Service::from_slice(b"foo").unwrap()),
        ),
        5,
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
        }
    );
}

#[test]
fn correctly_tracks_500_faults_and_400_different_service_messages() {
    static STATE: PromState = PromState::new();

    static SERVICE_NAMES: &[&[u8]] = &[
        b"service1",
        b"service2",
        b"service3",
        b"service4",
        b"service5",
        b"service6",
        b"service7",
        b"service8",
        b"service9",
        b"service10",
        b"service11",
        b"service12",
        b"service13",
        b"service14",
        b"service15",
        b"service16",
        b"service17",
        b"service18",
        b"service19",
        b"service20",
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
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Informational,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Debug,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Emergency,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Critical,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Notice,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Debug,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Alert,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Error,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Warning,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
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
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Informational,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Debug,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Emergency,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Critical,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Notice,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Debug,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Alert,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Error,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
        );
        STATE.add_fault();
        STATE.add_message_line_ingested(
            &message_key(
                Some(123),
                Some(123),
                Priority::Warning,
                Some(Service::from_slice(name).unwrap()),
            ),
            5,
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
        }
    );
}
