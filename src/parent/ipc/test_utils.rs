use crate::prelude::*;

use super::mocks::*;
use super::*;
use crate::ffi::*;

pub enum ExpectedSpawnResult {
    #[allow(dead_code)]
    Ready,
    #[allow(dead_code)]
    Err(libc::c_int),
    #[allow(dead_code)]
    BreakOk(ExitResult),
    BreakErr(libc::c_int),
}

impl fmt::Debug for ExpectedSpawnResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_as_break_io_result(
            f: &mut fmt::Formatter,
            result: Result<ExitResult, libc::c_int>,
        ) -> fmt::Result {
            f.debug_tuple("Break")
                .field(&result.map_err(Error::from_raw_os_error))
                .finish()
        }

        match self {
            Self::Ready => write!(f, "Ready"),
            Self::Err(code) => f
                .debug_tuple("Err")
                .field(&Error::from_raw_os_error(*code))
                .finish(),
            Self::BreakOk(result) => fmt_as_break_io_result(f, Ok(*result)),
            Self::BreakErr(code) => fmt_as_break_io_result(f, Err(*code)),
        }
    }
}

pub static EXIT_STATUS_TERMINATED: IpcExitStatus = IpcExitStatus {
    result: Some(ExitResult::Signal(Signal::SIGTERM)),
    parent_error: None,
    child_wait_error: None,
};

#[must_use = "This type must be retained for the duration that the child stdin should be kept open."]
pub struct StdinLease(&'static StaticState);
impl Drop for StdinLease {
    fn drop(&mut self) {
        *self.0.state.child_input() = None;
    }
}

#[must_use]
pub struct StaticState {
    pub state: ParentIpcState<FakeIpcChildHandle>,
}

impl StaticState {
    pub const fn new() -> StaticState {
        StaticState {
            state: ParentIpcState::new("/bin/cat", FakeIpcChildHandle::new()),
        }
    }

    pub fn connect_stdin(&'static self) -> StdinLease {
        *self.state.child_input() = Some(&self.state.methods().child_input);
        StdinLease(self)
    }

    pub fn init_test_state(&'static self) {
        self.init_test_state_with_key_dir(std::path::PathBuf::new());
    }

    pub fn init_test_state_with_key_dir(&'static self, key_dir: std::path::PathBuf) {
        self.state.init_dynamic(
            UserGroup {
                uid: current_uid(),
                gid: current_gid(),
            },
            Box::new([]),
            PromEnvironment::new(mock_system_time(123, 456)),
            key_dir,
        );
    }

    pub fn run_ipc_spawn(
        &'static self,
        ipc: &mut ChildSpawnManager<FakeIpcChildHandle>,
        maybe_status: Option<io::Result<IpcExitStatus>>,
        expected_result: ExpectedSpawnResult,
        expected_logs: &[&'static str],
    ) {
        let guard = setup_capture_logger();

        match (ipc.update_spawn(maybe_status), expected_result) {
            (ChildSpawnResult::Break(Ok(a)), ExpectedSpawnResult::BreakOk(b)) if a == b => {}
            (ChildSpawnResult::Break(Err(e)), ExpectedSpawnResult::BreakErr(code))
                if e.raw_os_error() == Some(code) => {}
            (ChildSpawnResult::Err(e), ExpectedSpawnResult::Err(code))
                if e.raw_os_error() == Some(code) => {}
            (ChildSpawnResult::Ready(_), ExpectedSpawnResult::Ready) => {}
            (result, expected_result) => {
                panic!(
                    "assertion failed: `left` does not match `right`\n  left: {:?}\n right: {:?}",
                    result, expected_result,
                );
            }
        }

        guard.expect_logs(expected_logs);
    }

    /// Note: always enqueue an error or drop the receiver, or the mock output reader will panic.
    pub fn run_ipc_message_loop_inner(&'static self) -> io::Result<()> {
        super::ipc_message_loop(&self.state.methods().child_output, &self.state)
    }

    /// Note: always enqueue an error, or the mock output reader will panic.
    pub fn run_ipc_message_loop(&'static self) -> io::Result<()> {
        let _stdin_lease = self.connect_stdin();
        self.run_ipc_message_loop_inner()
    }

    pub fn enqueue_next_instant(&'static self, result: Instant) {
        self.state.methods().next_instant.enqueue(result);
    }

    pub fn enqueue_child_spawn(
        &'static self,
        result: Result<(&'static ChildStateNotify, IpcExitStatus), libc::c_int>,
    ) {
        self.state.methods().child_spawn.enqueue_io(result);
    }

    pub fn enqueue_child_input(&'static self, result: Result<usize, libc::c_int>) {
        self.state.methods().child_input.enqueue_write(result);
    }

    pub fn enqueue_child_output(&'static self, result: Result<&'static [u8], libc::c_int>) {
        self.state.methods().child_output.enqueue_read(result);
    }

    pub fn enqueue_child_output_ok_spy(
        &'static self,
        result: &'static [u8],
        spy: Box<dyn FnOnce() + Send>,
    ) {
        self.state
            .methods()
            .child_output
            .enqueue_read_ok_spy(result, spy);
    }

    pub fn assert_input_sent(&'static self, expected: &[u8]) {
        self.state
            .methods()
            .child_input
            .assert_data_str_written(expected);
    }

    pub fn assert_no_calls_remaining(&'static self) {
        self.state.methods().assert_no_calls_remaining();
    }
}
