use crate::prelude::*;

use super::*;
use crate::test_utils::CallSpy;
use crate::test_utils::ReadSpy;
use crate::test_utils::WriteSpy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildState {
    Active,
    Terminated,
    Exited,
}

impl ChildState {
    const fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Active,
            1 => Self::Terminated,
            2 => Self::Exited,
            _ => unreachable!(),
        }
    }

    const fn to_u8(self) -> u8 {
        match self {
            ChildState::Active => 0,
            ChildState::Terminated => 1,
            ChildState::Exited => 2,
        }
    }
}

pub struct ChildStateNotify {
    inner: AtomicU8,
}

impl fmt::Debug for ChildStateNotify {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.get().fmt(f)
    }
}

impl PartialEq for ChildStateNotify {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl ChildStateNotify {
    pub const fn new() -> ChildStateNotify {
        ChildStateNotify {
            inner: AtomicU8::new(ChildState::Active.to_u8()),
        }
    }

    pub fn get(&self) -> ChildState {
        ChildState::from_u8(self.inner.load(Ordering::Acquire))
    }

    fn set_if_active(&self, state: ChildState) {
        // I just want the conditional assignment bit. It's okay if it fails. I'm just not using
        // `compare_exchange_weak` because I can't deal with *spurious* failures, only actual
        // failures.
        let _ignore = self.inner.compare_exchange(
            ChildState::Active.to_u8(),
            state.to_u8(),
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
}

pub type SpawnChildResult = (&'static ChildStateNotify, IpcExitStatus);

#[derive(Debug)]
enum SpawnStatus {
    None,
    Active(&'static ChildStateNotify, IpcExitStatus),
    Terminated(IpcExitStatus),
}

#[derive(Debug)]
pub struct FakeIpcChildHandle {
    pub next_instant: CallSpy<(), Instant>,
    pub child_spawn: CallSpy<(), io::Result<SpawnChildResult>>,
    pub child_input: WriteSpy,
    pub child_output: ReadSpy,
    spawn_result: Mutex<SpawnStatus>,
}

impl FakeIpcChildHandle {
    pub const fn new() -> FakeIpcChildHandle {
        FakeIpcChildHandle {
            next_instant: CallSpy::new("get_monotonic_time"),
            child_spawn: CallSpy::new("child_spawn"),
            child_input: WriteSpy::new("child_state.stdio.input"),
            child_output: ReadSpy::new("child_state.stdio.output"),
            spawn_result: Mutex::new(SpawnStatus::None),
        }
    }

    pub fn assert_no_calls_remaining(&self) {
        self.next_instant.assert_no_calls_remaining();
        self.child_spawn.assert_no_calls_remaining();
        self.child_input.assert_no_calls_remaining();
        self.child_output.assert_no_calls_remaining();
    }

    fn lock_spawn_result(&self) -> MutexGuard<SpawnStatus> {
        self.spawn_result.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn child_is_spawned(&self) -> bool {
        matches!(&*self.lock_spawn_result(), SpawnStatus::Active(_, _))
    }
}

impl ParentIpcMethods for FakeIpcChildHandle {
    type ChildInput = &'static WriteSpy;
    type ChildOutput = &'static ReadSpy;

    fn next_instant(&'static self) -> Instant {
        self.next_instant.call(())
    }

    fn get_user_group_table(&'static self) -> io::Result<Arc<UidGidTable>> {
        Ok(get_user_group_table())
    }

    fn child_spawn(&self, _: &'static ParentIpcState<Self>) -> io::Result<(&WriteSpy, &ReadSpy)> {
        let spy_result = self.child_spawn.call(());
        let mut guard = self.lock_spawn_result();

        match (spy_result, &*guard) {
            (Ok((ctrl_guard, status)), SpawnStatus::None) => {
                *guard = SpawnStatus::Active(ctrl_guard, status);
                Ok((&self.child_input, &self.child_output))
            }
            (Err(e), SpawnStatus::None) => Err(e),
            (_, _) => panic!("Child already spawned."),
        }
    }

    fn child_terminate(&self) -> io::Result<()> {
        let mut guard = self.lock_spawn_result();

        match replace(&mut *guard, SpawnStatus::None) {
            SpawnStatus::None => panic!("No child spawned."),
            SpawnStatus::Active(ctrl_guard, status) => {
                ctrl_guard.set_if_active(ChildState::Terminated);
                *guard = SpawnStatus::Terminated(status);
            }
            SpawnStatus::Terminated(status) => *guard = SpawnStatus::Terminated(status),
        }

        Ok(())
    }

    fn child_wait(&self) -> IpcExitStatus {
        match replace(&mut *self.lock_spawn_result(), SpawnStatus::None) {
            SpawnStatus::None => panic!("No child spawned."),
            SpawnStatus::Active(ctrl_guard, status) => {
                ctrl_guard.set_if_active(ChildState::Exited);
                status
            }
            SpawnStatus::Terminated(status) => status,
        }
    }
}

mod tests {
    use super::*;

    use crate::ffi::current_gid;
    use crate::ffi::current_uid;
    use crate::ffi::ExitCode;
    use crate::ffi::ExitResult;

    fn init_mock_ipc_dynamic(s: &'static ParentIpcState<FakeIpcChildHandle>) {
        s.init_dynamic(
            UserGroup {
                uid: current_uid(),
                gid: current_gid(),
            },
            Box::new([]),
            PromEnvironment::new(mock_system_time(123, 456)),
            std::path::PathBuf::new(),
        );
    }

    #[test]
    fn spawn_child_ok_works() {
        static S: ParentIpcState<FakeIpcChildHandle> =
            ParentIpcState::new("command", FakeIpcChildHandle::new());

        init_mock_ipc_dynamic(&S);
        static SPAWN_CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();

        S.methods().child_spawn.enqueue_io(Ok((
            &SPAWN_CHILD_NOTIFY,
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(1))),
                parent_error: None,
                child_wait_error: None,
            },
        )));

        S.methods()
            .child_spawn(&S)
            .expect("`spawn_child` returned an error");

        assert!(S.methods().child_is_spawned());

        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Active);
        S.methods().child_terminate().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Terminated);

        assert_eq!(
            S.methods().child_wait(),
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(1))),
                parent_error: None,
                child_wait_error: None,
            },
        );

        std::thread::sleep(Duration::from_millis(10));

        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Terminated);
        S.methods().child_input.assert_data_written(b"");
        S.methods().child_input.assert_no_calls_remaining();
        S.methods().child_output.assert_no_calls_remaining();

        S.methods().assert_no_calls_remaining();
    }

    #[test]
    fn spawn_child_join_fail_works() {
        static S: ParentIpcState<FakeIpcChildHandle> =
            ParentIpcState::new("command", FakeIpcChildHandle::new());
        init_mock_ipc_dynamic(&S);
        static SPAWN_CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();

        S.methods().child_spawn.enqueue_io(Ok((
            &SPAWN_CHILD_NOTIFY,
            IpcExitStatus {
                result: None,
                parent_error: None,
                child_wait_error: None,
            },
        )));

        S.methods()
            .child_spawn(&S)
            .expect("`spawn_child` returned an error");

        assert!(S.methods().child_is_spawned());

        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Active);
        S.methods().child_terminate().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Terminated);

        assert_eq!(
            S.methods().child_wait(),
            IpcExitStatus {
                result: None,
                parent_error: None,
                child_wait_error: None,
            },
        );

        std::thread::sleep(Duration::from_millis(10));

        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Terminated);
        S.methods().child_input.assert_data_written(b"");
        S.methods().child_input.assert_no_calls_remaining();
        S.methods().child_output.assert_no_calls_remaining();

        S.methods().assert_no_calls_remaining();
    }

    #[test]
    fn spawn_child_spawn_fail_works() {
        static S: ParentIpcState<FakeIpcChildHandle> =
            ParentIpcState::new("command", FakeIpcChildHandle::new());
        init_mock_ipc_dynamic(&S);

        S.methods().child_spawn.enqueue_io(Err(libc::ENOENT));

        match S.methods().child_spawn(&S) {
            Ok(_) => panic!("Expected `spawn_child` to return an error"),
            Err(e) => assert_error_eq(e, Error::from_raw_os_error(libc::ENOENT)),
        };

        S.methods().assert_no_calls_remaining();
    }

    #[test]
    fn spawn_child_handles_input_close() {
        static S: ParentIpcState<FakeIpcChildHandle> =
            ParentIpcState::new("command", FakeIpcChildHandle::new());
        init_mock_ipc_dynamic(&S);
        static SPAWN_CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();

        S.methods().child_spawn.enqueue_io(Ok((
            &SPAWN_CHILD_NOTIFY,
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(1))),
                parent_error: None,
                child_wait_error: None,
            },
        )));

        S.methods()
            .child_spawn(&S)
            .expect("`spawn_child` returned an error");

        assert!(S.methods().child_is_spawned());

        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Active);
        S.methods().child_terminate().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Terminated);

        S.methods().child_wait();

        std::thread::sleep(Duration::from_millis(10));

        S.methods().child_input.assert_data_written(b"");
        S.methods().child_input.assert_no_calls_remaining();
        S.methods().child_output.assert_no_calls_remaining();

        S.methods().assert_no_calls_remaining();
    }

    #[test]
    fn spawn_child_handles_write() {
        static S: ParentIpcState<FakeIpcChildHandle> =
            ParentIpcState::new("command", FakeIpcChildHandle::new());
        init_mock_ipc_dynamic(&S);
        static SPAWN_CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();

        S.methods().child_spawn.enqueue_io(Ok((
            &SPAWN_CHILD_NOTIFY,
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(1))),
                parent_error: None,
                child_wait_error: None,
            },
        )));

        let (mut child_input, _) = S
            .methods()
            .child_spawn(&S)
            .expect("`spawn_child` returned an error");

        assert!(S.methods().child_is_spawned());

        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Active);
        S.methods().child_terminate().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Terminated);

        S.methods().child_input.enqueue_write(Ok(8));
        child_input.write_all(b"12345678").unwrap();

        S.methods().child_wait();

        S.methods().child_input.assert_data_written(b"12345678");
        S.methods().child_input.assert_no_calls_remaining();
        S.methods().child_output.assert_no_calls_remaining();

        S.methods().assert_no_calls_remaining();
    }

    #[test]
    fn spawn_child_enqueues_read_ok() {
        static S: ParentIpcState<FakeIpcChildHandle> =
            ParentIpcState::new("command", FakeIpcChildHandle::new());
        init_mock_ipc_dynamic(&S);
        static SPAWN_CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();

        S.methods().child_spawn.enqueue_io(Ok((
            &SPAWN_CHILD_NOTIFY,
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(1))),
                parent_error: None,
                child_wait_error: None,
            },
        )));

        S.methods()
            .child_spawn(&S)
            .expect("`spawn_child` returned an error");

        assert!(S.methods().child_is_spawned());

        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Active);
        S.methods().child_terminate().unwrap();
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Terminated);

        S.methods()
            .child_output
            .enqueue_read(Ok(b"0123456789abcdef"));

        let mut output = [0_u8; 16];
        assert_result_eq((&S.methods().child_output).read(&mut output), Ok(16));

        assert_eq!(&output, b"0123456789abcdef");
        S.methods().child_wait();

        S.methods().child_input.assert_data_written(b"");
        S.methods().child_input.assert_no_calls_remaining();
        S.methods().child_output.assert_no_calls_remaining();

        S.methods().assert_no_calls_remaining();
    }

    #[test]
    fn spawn_child_output_enqueues_read_error() {
        static S: ParentIpcState<FakeIpcChildHandle> =
            ParentIpcState::new("command", FakeIpcChildHandle::new());
        init_mock_ipc_dynamic(&S);
        static SPAWN_CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();

        S.methods().child_spawn.enqueue_io(Ok((
            &SPAWN_CHILD_NOTIFY,
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(1))),
                parent_error: None,
                child_wait_error: None,
            },
        )));

        S.methods()
            .child_spawn(&S)
            .expect("`spawn_child` returned an error");

        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(SPAWN_CHILD_NOTIFY.get(), ChildState::Active);

        let mut output = [0; 16];

        S.methods().child_output.enqueue_read(Err(libc::EPIPE));

        assert_result_eq(
            (&S.methods().child_output).read(&mut output),
            Err(Error::from_raw_os_error(libc::EPIPE)),
        );

        S.methods().child_wait();

        S.methods().child_input.assert_data_written(b"");
        S.methods().child_input.assert_no_calls_remaining();
        S.methods().child_output.assert_no_calls_remaining();

        S.methods().assert_no_calls_remaining();
    }
}
