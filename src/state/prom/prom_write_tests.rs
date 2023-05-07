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

fn build_entry(
    service: &[u8],
    priority: Priority,
    lines: u64,
    bytes: u64,
) -> ByteCountSnapshotEntry {
    ByteCountSnapshotEntry {
        name: None,
        key: MessageKey::build(Some(123), Some(123), Some(service), priority),
        lines,
        bytes,
    }
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        monitor_hits: ByteCountSnapshot::empty(),
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
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        messages_ingested: ByteCountSnapshot::build([build_entry(
            b"foo.service",
            Priority::Informational,
            1,
            0,
        )]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 0
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        messages_ingested: ByteCountSnapshot::build([build_entry(
            b"foo.service",
            Priority::Informational,
            1,
            5,
        )]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        messages_ingested: ByteCountSnapshot::build([build_entry(
            b"foo.service",
            Priority::Informational,
            1,
            u64::MAX,
        )]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 18446744073709551615
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        messages_ingested: ByteCountSnapshot::build([build_entry(
            b"foo.service",
            Priority::Informational,
            u64::MAX,
            5,
        )]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 18446744073709551615
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
                name: None,
                key: MessageKey::build(
                    Some(123),
                    Some(456),
                    Some(b"foo.service"),
                    Priority::Informational,
                ),
                lines: 1,
                bytes: 5,
            },
            ByteCountSnapshotEntry {
                name: None,
                key: MessageKey::build(
                    Some(456),
                    Some(123),
                    Some(b"bar.service"),
                    Priority::Warning,
                ),
                lines: 1,
                bytes: 5,
            },
        ]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"bar.service\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"bar.service\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"bar.service\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"bar.service\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 5
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
        messages_ingested: ByteCountSnapshot::build([build_entry(
            b"foo.service",
            Priority::Informational,
            1,
            5,
        )]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
            build_entry(b"foo.service", Priority::Emergency, 1, 5),
            build_entry(b"foo.service", Priority::Alert, 1, 5),
            build_entry(b"foo.service", Priority::Critical, 1, 5),
            build_entry(b"foo.service", Priority::Error, 1, 5),
            build_entry(b"foo.service", Priority::Warning, 2, 10),
            build_entry(b"foo.service", Priority::Notice, 1, 5),
            build_entry(b"foo.service", Priority::Informational, 1, 5),
            build_entry(b"foo.service", Priority::Debug, 2, 10),
        ]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"foo.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"foo.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 2
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 10
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
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
            build_entry(b"foo.service", Priority::Emergency, 1, 5),
            build_entry(b"foo.service", Priority::Alert, 1, 5),
            build_entry(b"foo.service", Priority::Critical, 1, 5),
            build_entry(b"foo.service", Priority::Error, 1, 5),
            build_entry(b"foo.service", Priority::Warning, 2, 10),
            build_entry(b"foo.service", Priority::Notice, 1, 5),
            build_entry(b"foo.service", Priority::Informational, 1, 5),
            build_entry(b"foo.service", Priority::Debug, 2, 10),
        ]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"foo.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"foo.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 2
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 10
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
# EOF
"
    );
}

#[test]
fn renders_500_faults_and_400_different_service_messages() {
    // Move most the overhead to compile-time for Miri.
    macro_rules! ingested_entry_sets {
        ($($service:literal),* $(,)?) => {{
            [
                $(
                    build_entry($service, Priority::Emergency, 2, 10),
                    build_entry($service, Priority::Alert, 2, 10),
                    build_entry($service, Priority::Critical, 2, 10),
                    build_entry($service, Priority::Error, 2, 10),
                    build_entry($service, Priority::Warning, 4, 20),
                    build_entry($service, Priority::Notice, 2, 10),
                    build_entry($service, Priority::Informational, 2, 10),
                    build_entry($service, Priority::Debug, 4, 20),
                )*
            ]
        }}
    }

    let actual = render(PromSnapshot {
        entries_ingested: 0,
        fields_ingested: 0,
        data_ingested_bytes: 0,
        faults: 500,
        cursor_double_retries: 0,
        unreadable_fields: 0,
        corrupted_fields: 0,
        metrics_requests: 0,
        messages_ingested: ByteCountSnapshot::build(ingested_entry_sets![
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
        ]),
        monitor_hits: ByteCountSnapshot::empty(),
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
journald_messages_ingested_created{service=\"service1.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service2.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service3.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service4.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service5.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service6.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service7.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service8.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service9.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service10.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service11.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service12.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service13.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service14.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service15.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service16.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service17.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service18.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service19.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service20.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service1.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service2.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service3.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service4.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service5.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service6.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service7.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service8.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service9.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service10.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service11.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service12.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service13.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service14.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service15.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service16.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service17.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service18.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service19.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service20.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"service1.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20.service\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20.service\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20.service\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20.service\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service2.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service3.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service4.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service5.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service6.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service7.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service8.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service9.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service10.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service11.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service12.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service13.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service14.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service15.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service16.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service17.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service18.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service19.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service20.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20.service\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service1.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20.service\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20.service\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service2.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service3.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service4.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service5.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service6.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service7.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service8.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service9.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service10.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service11.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service12.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service13.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service14.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service15.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service16.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service17.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service18.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service19.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service20.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20.service\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
# TYPE journald_monitor_hits counter
journald_monitor_hits_created 123.456
journald_monitor_hits_total 0
# TYPE journald_monitor_hits_bytes counter
# UNIT journald_monitor_hits_bytes bytes
journald_monitor_hits_bytes_created 123.456
journald_monitor_hits_bytes_total 0
# EOF
"
    );
}
