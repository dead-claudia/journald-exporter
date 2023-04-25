use crate::prelude::*;

use super::test_utils::*;

use std::os::unix::fs::PermissionsExt;

fn write_test_key() -> tempfile::TempDir {
    let key_dir = tempfile::tempdir().unwrap();
    let mut file = std::fs::File::create(key_dir.path().join("test.key")).unwrap();
    file.set_permissions(std::fs::Permissions::from_mode(0o600))
        .unwrap();
    file.write_all(b"0123456789abcdef").unwrap();
    drop(file);
    key_dir
}

#[test]
fn reads_immediate_error() {
    let guard = setup_capture_logger();

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state();

    S.enqueue_child_output(Err(libc::EPIPE));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(&[]);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
fn reads_interrupt_then_immediate_error() {
    let guard = setup_capture_logger();

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state();

    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(&[]);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
fn read_header_then_empty_request() {
    let guard = setup_capture_logger();

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state();

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(&[]);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
// Skip in Miri due to filesystem access
#[cfg_attr(miri, ignore)]
fn read_header_then_request_keys() {
    let guard = setup_capture_logger();

    let key_dir = write_test_key();

    static EXPECTED: &[u8] = &[
        0x01, 0x01, 0x0F, b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b',
        b'c', b'd', b'e', b'f',
    ];

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();

    S.init_test_state_with_key_dir(key_dir.path().to_owned());

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_KEY]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(EXPECTED);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
fn read_header_then_request_metrics() {
    let guard = setup_capture_logger();

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state();

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_METRICS]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(EXPECTED_EXPOSITION);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
fn read_header_then_in_same_chunk_track_request_then_request_metrics() {
    let guard = setup_capture_logger();

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_metrics_requests_total 1
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state();

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[
        ipc::child::TRACK_REQUEST,
        ipc::child::REQUEST_METRICS,
    ]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(EXPECTED_EXPOSITION);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
fn read_header_then_track_request_then_request_metrics() {
    let guard = setup_capture_logger();

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_metrics_requests_total 1
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state();

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[ipc::child::TRACK_REQUEST]));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_METRICS]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(EXPECTED_EXPOSITION);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
fn read_header_then_request_metrics_then_track_request() {
    let guard = setup_capture_logger();

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state();

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_METRICS]));
    S.enqueue_child_output(Ok(&[ipc::child::TRACK_REQUEST]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(EXPECTED_EXPOSITION);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
// Skip in Miri due to filesystem access
#[cfg_attr(miri, ignore)]
fn read_header_then_in_same_chunk_request_keys_then_request_metrics() {
    let guard = setup_capture_logger();

    let key_dir = write_test_key();

    static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    let mut expected_sent = Vec::new();
    write_slices(&mut expected_sent, &[EXPECTED_KEY_SET, EXPECTED_EXPOSITION]);

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state_with_key_dir(key_dir.path().to_owned());

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_KEY, ipc::child::REQUEST_METRICS]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_KEY_SET.len()));
    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(&expected_sent);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
// Skip in Miri due to filesystem access
#[cfg_attr(miri, ignore)]
fn read_header_then_in_same_chunk_keys_then_track_then_metrics_then_track() {
    let guard = setup_capture_logger();

    let key_dir = write_test_key();

    static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_metrics_requests_total 2
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    let mut expected_sent = Vec::new();
    write_slices(&mut expected_sent, &[EXPECTED_KEY_SET, EXPECTED_EXPOSITION]);

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state_with_key_dir(key_dir.path().to_owned());

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[
        ipc::child::REQUEST_KEY,
        ipc::child::TRACK_REQUEST,
        ipc::child::REQUEST_METRICS,
        ipc::child::TRACK_REQUEST,
    ]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_KEY_SET.len()));
    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(&expected_sent);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
// Skip in Miri due to filesystem access
#[cfg_attr(miri, ignore)]
fn read_header_then_in_same_chunk_request_metrics_then_request_keys() {
    let guard = setup_capture_logger();

    let key_dir = write_test_key();

    static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    let mut expected_sent = Vec::new();
    write_slices(&mut expected_sent, &[EXPECTED_KEY_SET, EXPECTED_EXPOSITION]);

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state_with_key_dir(key_dir.path().to_owned());

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_METRICS, ipc::child::REQUEST_KEY]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_KEY_SET.len()));
    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(&expected_sent);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
// Skip in Miri due to filesystem access
#[cfg_attr(miri, ignore)]
fn read_header_then_request_keys_then_request_metrics() {
    let guard = setup_capture_logger();

    let key_dir = write_test_key();

    static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    let mut expected_sent = Vec::new();
    write_slices(&mut expected_sent, &[EXPECTED_KEY_SET, EXPECTED_EXPOSITION]);

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state_with_key_dir(key_dir.path().to_owned());

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_KEY]));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_METRICS]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_KEY_SET.len()));
    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(&expected_sent);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
// Skip in Miri due to filesystem access
#[cfg_attr(miri, ignore)]
fn read_header_then_request_metrics_then_request_keys() {
    let guard = setup_capture_logger();

    let key_dir = write_test_key();

    static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

    static EXPECTED_EXPOSITION: &[u8] =
        b"\x00\x02\x05\x00\x00# TYPE journald_entries_ingested counter
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
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
";

    let mut expected_sent = Vec::new();
    write_slices(&mut expected_sent, &[EXPECTED_EXPOSITION, EXPECTED_KEY_SET]);

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state_with_key_dir(key_dir.path().to_owned());

    // Also test that it's retried on interrupt.
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_METRICS]));
    S.enqueue_child_output(Ok(&[ipc::child::REQUEST_KEY]));
    S.enqueue_child_output(Err(libc::EINTR));
    S.enqueue_child_output(Err(libc::EAGAIN));
    S.enqueue_child_output(Err(libc::EPIPE));

    S.enqueue_child_input(Ok(EXPECTED_EXPOSITION.len()));
    S.enqueue_child_input(Ok(EXPECTED_KEY_SET.len()));

    assert_result_eq(
        S.run_ipc_message_loop(),
        Err(Error::from_raw_os_error(libc::EPIPE)),
    );

    S.assert_input_sent(&expected_sent);

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}

#[test]
fn bails_on_immediately_disconnected_receiver_when_handling_request_metrics() {
    let guard = setup_capture_logger();

    static S: StaticState = StaticState::new();
    let _watcher_guard = S.state.terminate_notify().create_guard();
    S.init_test_state();

    S.enqueue_child_output(Ok(&ipc::VERSION_BYTES));
    S.enqueue_child_output_ok_spy(
        &[ipc::child::REQUEST_METRICS],
        Box::new(|| {
            S.state.done_notify().notify();
        }),
    );

    let _stdin_lease = S.connect_stdin();

    assert_result_eq(S.run_ipc_message_loop_inner(), Ok(()));

    S.assert_no_calls_remaining();
    guard.expect_logs(&[]);
}
