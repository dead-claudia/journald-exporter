use crate::prelude::*;

use super::ipc_state::ParentIpcState;
use super::key_watcher::write_current_key_set;
use super::types::*;
use crate::ffi::NormalizeErrno;

#[must_use]
fn read_request(
    s: &'static ParentIpcState<impl ParentIpcMethods>,
    buf: &[u8],
) -> ipc::child::DecoderRequest {
    let mut decoder = s.decoder().lock();
    decoder.read_bytes(buf);
    decoder.take_request()
}

pub fn write_to_child_input(s: &'static ParentIpcState<impl ParentIpcMethods>, buf: &[u8]) -> bool {
    let mut guard = s.child_input();

    let Some(stdin) = &mut *guard else {
        return false;
    };

    match try_write(s.done_notify(), stdin, buf) {
        WriteOutputRequestResult::Written => true,
        WriteOutputRequestResult::Terminated => false,
        WriteOutputRequestResult::Err(e) => {
            log::error!("{}", NormalizeErrno(&e, None));
            *guard = None;
            false
        }
    }
}

#[must_use]
fn handle_metrics_request(s: &'static ParentIpcState<impl ParentIpcMethods>) -> bool {
    let response: Box<[u8]> = match s.methods().get_user_group_table() {
        Err(e) => {
            log::error!("{}", NormalizeErrno(&e, None));
            Box::new([])
        }
        Ok(table) => render_openapi_metrics(PromWriteContext {
            environment: s.dynamic().prom_environment(),
            snapshot: &s.state().snapshot(),
            table: &table,
        }),
    };

    write_to_child_input(s, &response)
}

pub fn ipc_message_loop<M: ParentIpcMethods>(
    mut child_output: M::ChildOutput,
    s: &'static ParentIpcState<M>,
) -> io::Result<()> {
    // 4 bytes is far more than enough to read client IPC messages efficiently, since they're
    // all just one byte and they're always batched into one go. In practice, there's really
    // only going to be 1-2 bytes to read total.
    let mut read_buf = [0_u8; 4];

    loop {
        let request = match try_read2(&mut child_output, s.done_notify(), &mut read_buf) {
            ReadWriteResult::Success(buf) => read_request(s, buf),
            ReadWriteResult::Terminated => break,
            ReadWriteResult::Err(e) => return Err(e),
        };

        // This is done sequentially, as it in practice is only hit up to about once a minute.
        s.state().add_requests(request.tracked_requests());

        if request.keys_requested() && !write_current_key_set(s) {
            break;
        }

        if request.metrics_requested() && !handle_metrics_request(s) {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::parent::ipc_test_utils::*;
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

        S.enqueue_child_output_err(libc::EPIPE);

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

        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

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
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        assert_result_eq(
            S.run_ipc_message_loop(),
            Err(Error::from_raw_os_error(libc::EPIPE)),
        );

        S.assert_input_sent(&[]);

        S.assert_no_calls_remaining();
        guard.expect_logs(&[]);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn read_header_then_request_keys() {
        let guard = setup_capture_logger();

        let key_dir = write_test_key();

        static EXPECTED: &[u8] = &[
            0x01, 0x01, 0x0F, b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a',
            b'b', b'c', b'd', b'e', b'f',
        ];

        static S: StaticState = StaticState::new();
        let _watcher_guard = S.state.terminate_notify().create_guard();

        S.init_test_state_with_key_dir(key_dir.path().to_owned());

        // Also test that it's retried on interrupt.
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_KEY]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED.len());

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
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_METRICS]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());

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
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::TRACK_REQUEST, ipc::child::REQUEST_METRICS]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());

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
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::TRACK_REQUEST]);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_METRICS]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());

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
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_METRICS]);
        S.enqueue_child_output_ok(&[ipc::child::TRACK_REQUEST]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());

        assert_result_eq(
            S.run_ipc_message_loop(),
            Err(Error::from_raw_os_error(libc::EPIPE)),
        );

        S.assert_input_sent(EXPECTED_EXPOSITION);

        S.assert_no_calls_remaining();
        guard.expect_logs(&[]);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn read_header_then_in_same_chunk_request_keys_then_request_metrics() {
        let guard = setup_capture_logger();

        let key_dir = write_test_key();

        static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

        static EXPECTED_EXPOSITION: &[u8] =
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        expected_sent.extend_from_slice(EXPECTED_KEY_SET);
        expected_sent.extend_from_slice(EXPECTED_EXPOSITION);

        static S: StaticState = StaticState::new();
        let _watcher_guard = S.state.terminate_notify().create_guard();
        S.init_test_state_with_key_dir(key_dir.path().to_owned());

        // Also test that it's retried on interrupt.
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_KEY, ipc::child::REQUEST_METRICS]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_KEY_SET.len());
        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());

        assert_result_eq(
            S.run_ipc_message_loop(),
            Err(Error::from_raw_os_error(libc::EPIPE)),
        );

        S.assert_input_sent(&expected_sent);

        S.assert_no_calls_remaining();
        guard.expect_logs(&[]);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn read_header_then_in_same_chunk_keys_then_track_then_metrics_then_track() {
        let guard = setup_capture_logger();

        let key_dir = write_test_key();

        static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

        static EXPECTED_EXPOSITION: &[u8] =
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        expected_sent.extend_from_slice(EXPECTED_KEY_SET);
        expected_sent.extend_from_slice(EXPECTED_EXPOSITION);

        static S: StaticState = StaticState::new();
        let _watcher_guard = S.state.terminate_notify().create_guard();
        S.init_test_state_with_key_dir(key_dir.path().to_owned());

        // Also test that it's retried on interrupt.
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[
            ipc::child::REQUEST_KEY,
            ipc::child::TRACK_REQUEST,
            ipc::child::REQUEST_METRICS,
            ipc::child::TRACK_REQUEST,
        ]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_KEY_SET.len());
        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());

        assert_result_eq(
            S.run_ipc_message_loop(),
            Err(Error::from_raw_os_error(libc::EPIPE)),
        );

        S.assert_input_sent(&expected_sent);

        S.assert_no_calls_remaining();
        guard.expect_logs(&[]);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn read_header_then_in_same_chunk_request_metrics_then_request_keys() {
        let guard = setup_capture_logger();

        let key_dir = write_test_key();

        static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

        static EXPECTED_EXPOSITION: &[u8] =
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        expected_sent.extend_from_slice(EXPECTED_KEY_SET);
        expected_sent.extend_from_slice(EXPECTED_EXPOSITION);

        static S: StaticState = StaticState::new();
        let _watcher_guard = S.state.terminate_notify().create_guard();
        S.init_test_state_with_key_dir(key_dir.path().to_owned());

        // Also test that it's retried on interrupt.
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_METRICS, ipc::child::REQUEST_KEY]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_KEY_SET.len());
        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());

        assert_result_eq(
            S.run_ipc_message_loop(),
            Err(Error::from_raw_os_error(libc::EPIPE)),
        );

        S.assert_input_sent(&expected_sent);

        S.assert_no_calls_remaining();
        guard.expect_logs(&[]);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn read_header_then_request_keys_then_request_metrics() {
        let guard = setup_capture_logger();

        let key_dir = write_test_key();

        static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

        static EXPECTED_EXPOSITION: &[u8] =
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        expected_sent.extend_from_slice(EXPECTED_KEY_SET);
        expected_sent.extend_from_slice(EXPECTED_EXPOSITION);

        static S: StaticState = StaticState::new();
        let _watcher_guard = S.state.terminate_notify().create_guard();
        S.init_test_state_with_key_dir(key_dir.path().to_owned());

        // Also test that it's retried on interrupt.
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_KEY]);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_METRICS]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_KEY_SET.len());
        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());

        assert_result_eq(
            S.run_ipc_message_loop(),
            Err(Error::from_raw_os_error(libc::EPIPE)),
        );

        S.assert_input_sent(&expected_sent);

        S.assert_no_calls_remaining();
        guard.expect_logs(&[]);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn read_header_then_request_metrics_then_request_keys() {
        let guard = setup_capture_logger();

        let key_dir = write_test_key();

        static EXPECTED_KEY_SET: &[u8] = b"\x01\x01\x0F0123456789abcdef";

        static EXPECTED_EXPOSITION: &[u8] =
            b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
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
        expected_sent.extend_from_slice(EXPECTED_EXPOSITION);
        expected_sent.extend_from_slice(EXPECTED_KEY_SET);

        static S: StaticState = StaticState::new();
        let _watcher_guard = S.state.terminate_notify().create_guard();
        S.init_test_state_with_key_dir(key_dir.path().to_owned());

        // Also test that it's retried on interrupt.
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_METRICS]);
        S.enqueue_child_output_ok(&[ipc::child::REQUEST_KEY]);
        S.enqueue_child_output_err(libc::EINTR);
        S.enqueue_child_output_err(libc::EAGAIN);
        S.enqueue_child_output_err(libc::EPIPE);

        S.enqueue_child_input_ok(EXPECTED_EXPOSITION.len());
        S.enqueue_child_input_ok(EXPECTED_KEY_SET.len());

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

        S.enqueue_child_output_ok(&ipc::VERSION_BYTES);
        S.enqueue_child_output_ok_spy(
            &[ipc::child::REQUEST_METRICS],
            Box::new(|| S.state.done_notify().notify()),
        );

        let _stdin_lease = S.connect_stdin();

        assert_result_eq(S.run_ipc_message_loop_inner(), Ok(()));

        S.assert_no_calls_remaining();
        guard.expect_logs(&[]);
    }
}
