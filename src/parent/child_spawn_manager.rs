use crate::prelude::*;

use super::fail_counter::FailCounter;
use super::ipc_state::ParentIpcState;
use super::types::*;
use crate::ffi::ExitResult;
use crate::ffi::NormalizeErrno;

pub struct ChildSpawnManager<M: ParentIpcMethods + 'static> {
    fail_counter: FailCounter,
    state: &'static ParentIpcState<M>,
}

#[derive(Debug)]
pub enum ChildSpawnResult<M: ParentIpcMethods> {
    Ready(M::ChildOutput),
    Err(Error),
    Break(io::Result<ExitResult>),
}

impl<M: ParentIpcMethods> ChildSpawnManager<M> {
    pub const fn new(state: &'static ParentIpcState<M>) -> Self {
        Self {
            fail_counter: FailCounter::new(),
            state,
        }
    }

    fn handle_prev_status(&mut self, status: IpcExitStatus) -> Option<io::Result<ExitResult>> {
        for e in status.errors {
            match e {
                IpcError::Parent(e) => log::error!(
                    "Parent IPC loop failed with error: {}",
                    NormalizeErrno(&e, None)
                ),
                IpcError::ChildWait(e) => {
                    log::error!("Child wait failed with error: {}", NormalizeErrno(&e, None))
                }
            }
        }

        let Some(result) = status.result else {
            return Some(Err(Error::new(
                ErrorKind::Other,
                "Child errored during termination.",
            )));
        };

        if self
            .fail_counter
            .check_fail(self.state.methods().next_instant())
        {
            return Some(Ok(result));
        }
        log::error!("Child exited prematurely with {}", result);

        // Simpler to do here.
        *self.state.decoder().lock() = ipc::child::Decoder::new();
        None
    }

    fn handle_prev_error(&mut self, error: Error) -> Option<io::Result<ExitResult>> {
        if self
            .fail_counter
            .check_fail(self.state.methods().next_instant())
        {
            return Some(Err(error));
        }

        log::error!(
            "Child errored during spawn: {}",
            NormalizeErrno(&error, None)
        );
        None
    }

    fn handle_prev_result(
        &mut self,
        result: io::Result<IpcExitStatus>,
    ) -> Option<io::Result<ExitResult>> {
        match result {
            Ok(status) => self.handle_prev_status(status),
            Err(error) => self.handle_prev_error(error),
        }
    }

    pub fn update_spawn(
        &mut self,
        maybe_status: Option<io::Result<IpcExitStatus>>,
    ) -> ChildSpawnResult<M> {
        match maybe_status.and_then(|result| self.handle_prev_result(result)) {
            Some(result) => ChildSpawnResult::Break(result),
            // Easier to just restart the IPC altogether anew if the child itself fails. If the parent
            // fails, it's fatal and should be restarted. Of course, the child also will need started
            // as well.
            //
            // The state is still intact, so consistency errors aren't a risk.
            None => match self.state.methods().child_spawn(self.state) {
                Ok((child_stdin, child_stdout)) => {
                    *self.state.child_input() = Some(child_stdin);
                    ChildSpawnResult::Ready(child_stdout)
                }
                Err(e) => ChildSpawnResult::Err(e),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::parent::ipc_mocks::*;
    use crate::parent::ipc_test_utils::*;

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn manager_tracks_retries_on_spawn_error() {
        static S: StaticState = StaticState::new();
        S.init_test_state();

        let instant = Instant::now();
        S.enqueue_child_spawn_err(libc::ENOENT);
        S.enqueue_next_instant(instant + Duration::from_millis(0));
        S.enqueue_child_spawn_err(libc::EACCES);
        S.enqueue_next_instant(instant + Duration::from_millis(100));
        S.enqueue_child_spawn_err(libc::EBUSY);
        S.enqueue_next_instant(instant + Duration::from_millis(200));
        S.enqueue_child_spawn_err(libc::EMFILE);
        S.enqueue_next_instant(instant + Duration::from_millis(300));
        S.enqueue_child_spawn_err(libc::ELOOP);
        S.enqueue_next_instant(instant + Duration::from_millis(400));

        let mut ipc = ChildSpawnManager::new(&S.state);
        S.run_ipc_spawn(&mut ipc, None, ExpectedSpawnResult::Err(libc::ENOENT), &[]);
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::ENOENT))),
            ExpectedSpawnResult::Err(libc::EACCES),
            &["Child errored during spawn: ENOENT: No such file or directory"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EACCES))),
            ExpectedSpawnResult::Err(libc::EBUSY),
            &["Child errored during spawn: EACCES: Permission denied"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EBUSY))),
            ExpectedSpawnResult::Err(libc::EMFILE),
            &["Child errored during spawn: EBUSY: Device or resource busy"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EMFILE))),
            ExpectedSpawnResult::Err(libc::ELOOP),
            &["Child errored during spawn: EMFILE: Too many open files"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::ELOOP))),
            ExpectedSpawnResult::BreakErr(libc::ELOOP),
            &[],
        );

        S.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn manager_allows_ready_from_start() {
        static S: StaticState = StaticState::new();
        static CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();
        S.init_test_state();

        S.enqueue_child_spawn_ok(&CHILD_NOTIFY, EXIT_STATUS_TERMINATED.clone());

        let mut ipc = ChildSpawnManager::new(&S.state);
        S.run_ipc_spawn(&mut ipc, None, ExpectedSpawnResult::Ready, &[]);
        assert_eq!(S.child_wait(), EXIT_STATUS_TERMINATED);

        S.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn manager_allows_ready_from_1_spawn_error() {
        static S: StaticState = StaticState::new();
        static CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();
        S.init_test_state();

        let instant = Instant::now();
        S.enqueue_child_spawn_err(libc::ENOENT);
        S.enqueue_next_instant(instant + Duration::from_millis(0));
        S.enqueue_child_spawn_ok(&CHILD_NOTIFY, EXIT_STATUS_TERMINATED.clone());

        let mut ipc = ChildSpawnManager::new(&S.state);
        S.run_ipc_spawn(&mut ipc, None, ExpectedSpawnResult::Err(libc::ENOENT), &[]);
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::ENOENT))),
            ExpectedSpawnResult::Ready,
            &["Child errored during spawn: ENOENT: No such file or directory"],
        );
        assert_eq!(S.child_wait(), EXIT_STATUS_TERMINATED);

        S.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn manager_allows_ready_from_2_spawn_errors() {
        static S: StaticState = StaticState::new();
        static CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();
        S.init_test_state();

        let instant = Instant::now();
        S.enqueue_child_spawn_err(libc::ENOENT);
        S.enqueue_next_instant(instant + Duration::from_millis(0));
        S.enqueue_child_spawn_err(libc::EACCES);
        S.enqueue_next_instant(instant + Duration::from_millis(100));
        S.enqueue_child_spawn_ok(&CHILD_NOTIFY, EXIT_STATUS_TERMINATED.clone());

        let mut ipc = ChildSpawnManager::new(&S.state);
        S.run_ipc_spawn(&mut ipc, None, ExpectedSpawnResult::Err(libc::ENOENT), &[]);
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::ENOENT))),
            ExpectedSpawnResult::Err(libc::EACCES),
            &["Child errored during spawn: ENOENT: No such file or directory"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EACCES))),
            ExpectedSpawnResult::Ready,
            &["Child errored during spawn: EACCES: Permission denied"],
        );

        assert_eq!(S.child_wait(), EXIT_STATUS_TERMINATED);

        S.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn manager_allows_ready_from_3_spawn_errors() {
        static S: StaticState = StaticState::new();
        static CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();
        S.init_test_state();

        let instant = Instant::now();
        S.enqueue_child_spawn_err(libc::ENOENT);
        S.enqueue_next_instant(instant + Duration::from_millis(0));
        S.enqueue_child_spawn_err(libc::EACCES);
        S.enqueue_next_instant(instant + Duration::from_millis(100));
        S.enqueue_child_spawn_err(libc::EBUSY);
        S.enqueue_next_instant(instant + Duration::from_millis(200));
        S.enqueue_child_spawn_ok(&CHILD_NOTIFY, EXIT_STATUS_TERMINATED.clone());

        let mut ipc = ChildSpawnManager::new(&S.state);
        S.run_ipc_spawn(&mut ipc, None, ExpectedSpawnResult::Err(libc::ENOENT), &[]);
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::ENOENT))),
            ExpectedSpawnResult::Err(libc::EACCES),
            &["Child errored during spawn: ENOENT: No such file or directory"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EACCES))),
            ExpectedSpawnResult::Err(libc::EBUSY),
            &["Child errored during spawn: EACCES: Permission denied"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EBUSY))),
            ExpectedSpawnResult::Ready,
            &["Child errored during spawn: EBUSY: Device or resource busy"],
        );

        assert_eq!(S.child_wait(), EXIT_STATUS_TERMINATED);

        S.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn manager_allows_ready_from_4_spawn_errors() {
        static S: StaticState = StaticState::new();
        static CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();
        S.init_test_state();

        let instant = Instant::now();
        S.enqueue_child_spawn_err(libc::ENOENT);
        S.enqueue_next_instant(instant + Duration::from_millis(0));
        S.enqueue_child_spawn_err(libc::EACCES);
        S.enqueue_next_instant(instant + Duration::from_millis(100));
        S.enqueue_child_spawn_err(libc::EBUSY);
        S.enqueue_next_instant(instant + Duration::from_millis(200));
        S.enqueue_child_spawn_err(libc::EMFILE);
        S.enqueue_next_instant(instant + Duration::from_millis(300));
        S.enqueue_child_spawn_ok(&CHILD_NOTIFY, EXIT_STATUS_TERMINATED.clone());

        let mut ipc = ChildSpawnManager::new(&S.state);
        S.run_ipc_spawn(&mut ipc, None, ExpectedSpawnResult::Err(libc::ENOENT), &[]);
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::ENOENT))),
            ExpectedSpawnResult::Err(libc::EACCES),
            &["Child errored during spawn: ENOENT: No such file or directory"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EACCES))),
            ExpectedSpawnResult::Err(libc::EBUSY),
            &["Child errored during spawn: EACCES: Permission denied"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EBUSY))),
            ExpectedSpawnResult::Err(libc::EMFILE),
            &["Child errored during spawn: EBUSY: Device or resource busy"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EMFILE))),
            ExpectedSpawnResult::Ready,
            &["Child errored during spawn: EMFILE: Too many open files"],
        );

        assert_eq!(S.child_wait(), EXIT_STATUS_TERMINATED);

        S.assert_no_calls_remaining();
    }

    #[test]
    // FIXME: figure out why logs aren't appearing in Miri. It passes in `cargo test`.
    #[cfg_attr(miri, ignore)]
    fn manager_tracks_retries_on_5_spawn_errors_if_spaced_far_enough_apart() {
        static S: StaticState = StaticState::new();
        static CHILD_NOTIFY: ChildStateNotify = ChildStateNotify::new();
        S.init_test_state();

        let instant = Instant::now();
        S.enqueue_child_spawn_err(libc::ENOENT);
        S.enqueue_next_instant(instant + Duration::from_millis(0));
        S.enqueue_child_spawn_err(libc::EACCES);
        S.enqueue_next_instant(instant + Duration::from_millis(100));
        S.enqueue_child_spawn_err(libc::EBUSY);
        S.enqueue_next_instant(instant + Duration::from_millis(200));
        S.enqueue_child_spawn_err(libc::EMFILE);
        S.enqueue_next_instant(instant + Duration::from_millis(300));
        S.enqueue_child_spawn_err(libc::ELOOP);
        S.enqueue_next_instant(instant + Duration::from_millis(10000));
        S.enqueue_child_spawn_ok(&CHILD_NOTIFY, EXIT_STATUS_TERMINATED.clone());

        let mut ipc = ChildSpawnManager::new(&S.state);
        S.run_ipc_spawn(&mut ipc, None, ExpectedSpawnResult::Err(libc::ENOENT), &[]);
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::ENOENT))),
            ExpectedSpawnResult::Err(libc::EACCES),
            &["Child errored during spawn: ENOENT: No such file or directory"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EACCES))),
            ExpectedSpawnResult::Err(libc::EBUSY),
            &["Child errored during spawn: EACCES: Permission denied"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EBUSY))),
            ExpectedSpawnResult::Err(libc::EMFILE),
            &["Child errored during spawn: EBUSY: Device or resource busy"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::EMFILE))),
            ExpectedSpawnResult::Err(libc::ELOOP),
            &["Child errored during spawn: EMFILE: Too many open files"],
        );
        S.run_ipc_spawn(
            &mut ipc,
            Some(Err(Error::from_raw_os_error(libc::ELOOP))),
            ExpectedSpawnResult::Ready,
            &["Child errored during spawn: ELOOP: Too many levels of symbolic links"],
        );

        assert_eq!(S.child_wait(), EXIT_STATUS_TERMINATED);

        S.assert_no_calls_remaining();
    }
}
