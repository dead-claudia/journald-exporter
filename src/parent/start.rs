use crate::prelude::*;

use super::child_spawn_manager::ChildSpawnManager;
use super::child_spawn_manager::ChildSpawnResult;
use super::ipc::ipc_message_loop;
use super::ipc_state::ParentIpcState;
use super::journal::run_journal_loop;
use super::key_watcher::run_watcher;
use super::native_ipc::NativeIpcMethods;
use super::types::IpcError;
use super::types::IpcExitStatus;
use super::types::ParentIpcMethods;
use crate::cli::args::ParentArgs;
use crate::ffi::ExitCode;
use crate::ffi::ExitResult;
use crate::ffi::NativeJournalRef;
use crate::ffi::NativeSystemdProvider;
use std::time::SystemTime;

static IPC_STATE: ParentIpcState<NativeIpcMethods> =
    ParentIpcState::new("/proc/self/exe", NativeIpcMethods::new());

static NATIVE_JOURNAL_PROVIDER: OnceCell<NativeSystemdProvider> = OnceCell::new();

struct NotifyGuard;

impl Drop for NotifyGuard {
    fn drop(&mut self) {
        IPC_STATE.terminate_notify().notify();
        IPC_STATE.done_notify().notify();
    }
}

pub fn start_parent(args: ParentArgs) -> io::Result<ExitResult> {
    let (child_uid, child_gid) = get_child_uid_gid()?;
    let provider = NativeSystemdProvider::open_provider()?;

    NATIVE_JOURNAL_PROVIDER.get_or_init(|| provider);

    let _notify_guard = NotifyGuard;

    IPC_STATE.init_dynamic(
        child_uid,
        child_gid,
        vec![String::from("--child-process"), args.port.to_string()],
        PromEnvironment {
            created: SystemTime::now(),
        },
        args.key_dir,
    );

    resolve_parent_return()
}

fn get_child_uid_gid() -> io::Result<(Uid, Gid)> {
    let table = IPC_STATE.methods().get_user_group_table()?;

    // Verify it's running as root and then get the UID and GID of the child.

    if Uid::current() != Uid::ROOT || Gid::current() != Gid::ROOT {
        return Err(Error::new(
            ErrorKind::Other,
            "This program is intended to be run as root.",
        ));
    }

    match table.lookup_user_group(b"journald-exporter", b"journald-exporter") {
        (Some(uid), Some(gid)) => Ok((uid, gid)),
        _ => Err(Error::new(
            ErrorKind::Other,
            "Expected a `journald-exporter` user must be present.",
        )),
    }
}

// Here's the intent:
// - If any high-level task errors, the function should just fail entirely and immediately.
// - If any high-level task ends, the token should notify so everything gets shut down safely.
//   This part isn't handled by this function, but by a `NOTIFY_EXIT` global.
// - If the parent IPC task terminates, its exit result should be used. Otherwise, it should just
//   default to an exit result of 1.

fn resolve_parent_return() -> io::Result<ExitResult> {
    // FIXME: re-evaluate once https://github.com/rust-lang/rust-clippy/pull/10309 is released.
    #![allow(clippy::arithmetic_side_effects)]

    bitflags! {
        #[derive(Clone, Copy)]
        struct WaitFlags: u8 {
            const PARENT_IPC  = 1 << 0;
            const JOURNAL     = 1 << 1;
            const KEY_UPDATER = 1 << 2;
        }
    }

    const READY_PARENT_IPC: u8 = 1 << 0;
    const READY_BACKGROUND: u8 = 1 << 1;

    #[derive(Clone, Copy)]
    struct WaitState {
        flags: u8,
        exit_result: ExitResult,
    }

    static WAIT_CHECKPOINT: Checkpoint<WaitState> = Checkpoint::new(WaitState {
        flags: 0,
        exit_result: ExitResult::Code(ExitCode(1)),
    });

    struct ParentIpcTaskGuard(Option<ExitResult>);

    impl Drop for ParentIpcTaskGuard {
        fn drop(&mut self) {
            WAIT_CHECKPOINT.notify(|state| {
                state.flags |= READY_PARENT_IPC;
                if let Some(exit_result) = self.0 {
                    state.exit_result = exit_result;
                }
            });
        }
    }

    fn parent_ipc_task() -> ThreadTask {
        Box::new(move || {
            let mut task_guard = ParentIpcTaskGuard(None);
            task_guard.0 = Some(parent_ipc()?);
            Ok(())
        })
    }

    struct BackgroundTaskGuard;

    impl Drop for BackgroundTaskGuard {
        fn drop(&mut self) {
            WAIT_CHECKPOINT.notify(|state| {
                state.flags |= READY_BACKGROUND;
            });
        }
    }

    fn journal_task() -> ThreadTask {
        Box::new(move || {
            let _task_guard = BackgroundTaskGuard;
            run_journal_loop::<NativeJournalRef>(&IPC_STATE, NATIVE_JOURNAL_PROVIDER.get().unwrap())
        })
    }

    fn key_updater_task() -> ThreadTask {
        Box::new(move || {
            let _task_guard = BackgroundTaskGuard;
            run_watcher(&IPC_STATE)
        })
    }

    let parent_ipc_handle = ThreadHandle::spawn(parent_ipc_task());
    let journal_handle = ThreadHandle::spawn(journal_task());
    let key_updater_handle = ThreadHandle::spawn(key_updater_task());

    let mut result = Ok(WAIT_CHECKPOINT.wait().exit_result);

    if let Err(e) = parent_ipc_handle.join() {
        result = Err(e);
    }

    if let Err(e) = journal_handle.join() {
        result = Err(e);
    }

    if let Err(e) = key_updater_handle.join() {
        result = Err(e);
    }

    result
}

pub fn parent_ipc() -> io::Result<ExitResult> {
    let mut ipc = ChildSpawnManager::new(&IPC_STATE);
    let mut resume = None;

    loop {
        match ipc.update_spawn(resume) {
            ChildSpawnResult::Ready(child_output) => {
                struct DropStdin;
                impl Drop for DropStdin {
                    fn drop(&mut self) {
                        *IPC_STATE.child_input() = None;
                    }
                }

                let _stdin_guard = DropStdin;
                resume = Some(Ok(spawn_ipc_and_wait_for_child_exit(child_output)));
            }
            ChildSpawnResult::Err(e) => resume = Some(Err(e)),
            ChildSpawnResult::Break(result) => break result,
        }
    }
}

fn ipc_message_loop_task(
    child_output: std::process::ChildStdout,
    child_exited: Arc<Notify>,
) -> ThreadTask {
    Box::new(move || {
        let guard = IPC_STATE.done_notify().create_guard();
        let result = ipc_message_loop(child_output, &IPC_STATE);
        drop(guard);
        if !child_exited.has_notified() {
            IPC_STATE.methods().child_terminate()?;
        }
        result
    })
}

fn spawn_ipc_and_wait_for_child_exit(child_output: std::process::ChildStdout) -> IpcExitStatus {
    IPC_STATE.done_notify().reset();
    let child_exited = Arc::new(Notify::new());
    let done_guard = IPC_STATE.done_notify().create_guard();
    let ipc_thread = ThreadHandle::spawn(ipc_message_loop_task(
        child_output,
        Arc::clone(&child_exited),
    ));

    let mut status = IPC_STATE.methods().child_wait();
    child_exited.notify();
    drop(done_guard);

    if let Err(e) = ipc_thread.join() {
        status.errors.push(IpcError::Parent(e));
    }

    status
}
