use crate::prelude::*;

use super::*;

fn render(snapshot: PromSnapshot) -> Vec<u8> {
    render_openapi_metrics(
        &PromEnvironment::new(mock_system_time(123, 456)),
        &snapshot,
        &get_user_group_table(),
    )
    .unwrap()
}

// Get this noise out. Also gets tedious editing the length every time I want to add an entry or
// modify an existing one.
fn assert_snapshot_eq(actual: Vec<u8>, expected: &'static [u8]) {
    // This is optimized somewhat to try to speed up Miri in one of the slowest parts. Won't make
    // literally any difference for the standard `cargo test`.

    let mut real_expected = Vec::new();

    real_expected.push(0);
    real_expected.extend_from_slice(&truncate_usize_u32(expected.len()).to_le_bytes());
    real_expected.extend_from_slice(expected);

    assert_eq!(BinaryToDebug(&actual), BinaryToDebug(&real_expected));
}

#[test]
fn renders_1_digit_entries_ingested() {
    let actual = render(PromSnapshot {
        entries_ingested: 1,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 0,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 1
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_max_entries_ingested() {
    let actual = render(PromSnapshot {
        entries_ingested: u64::MAX,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 0,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 18446744073709551615
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_max_fields_ingested() {
    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: u64::MAX,
        data_ingested_bytes: 0,
        faults: 0,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 18446744073709551615
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_max_data_ingested_bytes() {
    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: u64::MAX,
        faults: 0,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 18446744073709551615
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_max_faults() {
    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: u64::MAX,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 18446744073709551615
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_max_cursor_double_retries() {
    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 0,
        cursor_double_retries: u64::MAX,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 18446744073709551615
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_max_unreadable_entries() {
    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 0,
        cursor_double_retries: 0,
        unreadable_fields: u64::MAX,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 18446744073709551615
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_max_corrupted_entries() {
    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 0,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: u64::MAX,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 18446744073709551615
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_max_requests() {
    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 0,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: u64::MAX,
        messages_ingested: ByteCountSnapshot::empty(),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 18446744073709551615
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
",
    );
}

#[test]
fn renders_a_single_empty_message_key_ingested() {
    let actual = render(PromSnapshot {
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
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 0
# EOF
"
    );
}

#[test]
fn renders_a_single_small_message_key_ingested() {
    let actual = render(PromSnapshot {
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
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# EOF
"
    );
}

#[test]
fn renders_a_single_max_len_message_key_ingested() {
    let actual = render(PromSnapshot {
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
            bytes: u64::MAX,
        }]),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 18446744073709551615
# EOF
"
    );
}

#[test]
fn renders_a_single_max_lines_message_key_ingested() {
    let actual = render(PromSnapshot {
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
            lines: u64::MAX,
            bytes: 5,
        }]),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 18446744073709551615
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# EOF
"
    );
}

#[test]
fn renders_two_messages_across_two_services() {
    let actual = render(PromSnapshot {
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
                key: MessageKey::build(Some(123), Some(456), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(456), Some(123), Some(b"bar"), Priority::Warning),
                lines: 1,
                bytes: 5,
            },
        ]),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"bar\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"bar\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"bar\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"bar\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 5
# EOF
"
    );
}

#[test]
fn renders_1_fault_and_1_message() {
    let actual = render(PromSnapshot {
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
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 1
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# EOF
"
    );
}

#[test]
fn renders_multiple_priority_levels_within_same_service() {
    let actual = render(PromSnapshot {
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
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Emergency),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Alert),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Critical),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Error),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Warning),
                lines: 2,
                bytes: 10,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Notice),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Debug),
                lines: 2,
                bytes: 10,
            },
        ]),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 2
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 10
# EOF
"
    );
}

#[test]
fn renders_5_faults_and_multiple_priority_levels_within_same_service() {
    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 5,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::build([
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Emergency),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Alert),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Critical),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Error),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Warning),
                lines: 2,
                bytes: 10,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Notice),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Informational),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Debug),
                lines: 2,
                bytes: 10,
            },
        ]),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 5
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 2
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 10
# EOF
"
    );
}

#[test]
fn renders_500_faults_and_400_different_service_messages() {
    // Move most the overhead to compile-time.
    static MESSAGES_INGESTED: [ByteCountSnapshotEntry; 160] = {
        let mut result = [const {
            ByteCountSnapshotEntry {
                key: MessageKey::build(None, None, None, Priority::Emergency),
                lines: 0,
                bytes: 0,
            }
        }; 160];

        let service_names: [&[u8]; 20] = [
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

        let ingested_message_data_params: [(Priority, u64, u64); 8] = [
            (Priority::Emergency, 2, 10),
            (Priority::Alert, 2, 10),
            (Priority::Critical, 2, 10),
            (Priority::Error, 2, 10),
            (Priority::Warning, 4, 20),
            (Priority::Notice, 2, 10),
            (Priority::Informational, 2, 10),
            (Priority::Debug, 4, 20),
        ];

        let mut i = 0;
        let mut target = 0;
        while i < service_names.len() {
            let service = service_names[i];
            let mut j = 0;
            while j < ingested_message_data_params.len() {
                let (priority, lines, bytes) = ingested_message_data_params[j];

                result[target] = ByteCountSnapshotEntry {
                    key: MessageKey::build(Some(123), Some(123), Some(service), priority),
                    lines,
                    bytes,
                };

                j += 1;
                target += 1;
            }
            i += 1;
        }

        result
    };

    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 500,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::build(MESSAGES_INGESTED.iter().cloned()),
    });

    assert_snapshot_eq(
        actual,
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 500
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"service1\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service2\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service3\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service4\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service5\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service6\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service7\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service8\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service9\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service10\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service11\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service12\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service13\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service14\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service15\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service16\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service17\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service18\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service19\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service20\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service1\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service2\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service3\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service4\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service5\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service6\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service7\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service8\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service9\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service10\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service11\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service12\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service13\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service14\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service15\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service16\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service17\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service18\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service19\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service20\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
# EOF
"
    );
}
