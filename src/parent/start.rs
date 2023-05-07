use crate::prelude::*;

use super::ipc::*;
use super::journal::run_journal_loop;
use super::key_watcher::run_watcher;
use crate::cli::args::TLSOptions;
use crate::cli::config::IdOrName;
use crate::cli::config::MonitorFilterEntry;
use crate::cli::config::ParentConfig;
use crate::ffi::*;
use crate::parent::key_watcher::KeyWatcherTarget;
use const_str::cstr;
use std::time::SystemTime;

const READY_PARENT_IPC: u8 = 1 << 0;
const READY_BACKGROUND: u8 = 1 << 1;

struct WaitState {
    flags: u8,
    exit_result: ExitResult,
}

static NATIVE_JOURNALD_PROVIDER: OnceCell<NativeSystemdProvider> = OnceCell::new();

static IPC_STATE: ParentIpcState<NativeIpcMethods> = ParentIpcState::new(NativeIpcMethods::new());

pub fn start_parent(args: ParentConfig) -> io::Result<ExitResult> {
    let user_table = IPC_STATE.methods().get_user_group_table()?;
    let child_user_group = get_child_uid_gid(&user_table)?;

    let monitor_filter = match args.monitor_filter {
        None => None,
        Some(entries) => Some(resolve_monitor_filter(&user_table, entries)?),
    };

    // SAFETY: The lease is forgotten, so the risky `drop` bits will never execute.
    unsafe {
        #[allow(clippy::mem_forget)]
        std::mem::forget(IPC_STATE.state().initialize_monitor_filter(monitor_filter));
    }

    let provider = NativeSystemdProvider::open_provider()?;

    NATIVE_JOURNALD_PROVIDER.get_or_init(|| provider);

    let _notify_guard = IPC_STATE.terminate_notify().create_guard();
    let _notify_guard = IPC_STATE.done_notify().create_guard();

    IPC_STATE.init_dynamic(ParentIpcDynamic {
        port: args.port,
        child_user_group,
        prom_environment: PromEnvironment::new(SystemTime::now()),
        key_target: KeyWatcherTarget::new(args.key_dir),
        tls_config: load_tls_config(args.tls)?,
    });

    resolve_parent_return()
}

fn load_tls_file(path: &std::path::Path) -> io::Result<Box<std::ffi::OsStr>> {
    use std::os::unix::prelude::OsStringExt;

    match std::fs::read(path) {
        Ok(data) => Ok(std::ffi::OsString::from_vec(data).into()),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            Err(error!(ErrorKind::NotFound, "{} not found.", path.display()))
        }
        Err(e) => Err(error!(
            "An error occurred while loading {}: {}.",
            path.display(),
            normalize_errno(e, Some("open"))
        )),
    }
}

fn load_tls_config(tls: Option<TLSOptions>) -> io::Result<Option<TLSConfig>> {
    match tls {
        None => Ok(None),
        Some(tls) => {
            let config = TLSConfig {
                certificate: load_tls_file(&tls.certificate)?,
                private_key: load_tls_file(&tls.private_key)?,
            };
            log::info!("TLS config loaded.");
            Ok(Some(config))
        }
    }
}

fn get_child_uid_gid(user_table: &UidGidTable) -> io::Result<UserGroup> {
    user_table
        .lookup_user_group(b"journald-exporter", b"journald-exporter")
        .ok_or_else(|| error!("A `journald-exporter` user must be present."))
}

fn resolve_monitor_filter(
    user_table: &UidGidTable,
    entries: Vec<MonitorFilterEntry>,
) -> io::Result<MonitorFilter> {
    fn check_id(
        id_table: &IdTable,
        id_kind: &str,
        monitor_name: &str,
        id: Option<IdOrName>,
    ) -> io::Result<Option<Option<u32>>> {
        match id {
            None => Ok(None),
            Some(IdOrName::Placeholder) => Ok(Some(None)),
            Some(IdOrName::Id(id)) => {
                if id_table.lookup_id(id).is_none() {
                    log::warn!("In `[monitor.{monitor_name}]`: {id_kind} ID `{id}` doesn't seem to exist. Filter will remain live, but won't likely receive hits.");
                }
                Ok(Some(Some(id)))
            }
            Some(IdOrName::Name(name)) => match id_table.lookup_name(name.as_bytes()) {
                Some(id) => Ok(Some(Some(id))),
                None => Err(error!(
                    "In `[monitor.{monitor_name}]`: {id_kind} `{name}` could not be resolved."
                )),
            },
        }
    }

    let mut resolved = Vec::with_capacity(entries.len());

    for entry in entries {
        resolved.push(MonitorFilterResolved {
            uid: check_id(&user_table.uids, "User", &entry.monitor_name, entry.user)?,
            gid: check_id(&user_table.gids, "Group", &entry.monitor_name, entry.group)?,
            monitor_name: entry.monitor_name,
            priority: entry.priority,
            service: entry.service,
            message_pattern: entry.message_pattern,
        })
    }

    Ok(MonitorFilter::new(&resolved))
}

// Here's the intent:
// - If any high-level task errors, the function should just fail entirely and immediately.
// - If any high-level task ends, the token should notify so everything gets shut down safely.
//   This part isn't handled by this function, but by a `NOTIFY_EXIT` global.
// - If the parent IPC task terminates, its exit result should be used. Otherwise, it should just
//   default to an exit result of 1.

fn resolve_parent_return() -> io::Result<ExitResult> {
    struct ParentIpcTaskGuard(Option<ExitResult>);

    static WAIT_CHECKPOINT: Checkpoint<WaitState> = Checkpoint::new(WaitState {
        flags: 0,
        exit_result: ExitResult::Code(ExitCode(1)),
    });

    impl Drop for ParentIpcTaskGuard {
        fn drop(&mut self) {
            WAIT_CHECKPOINT.notify(|state| {
                state.flags |= READY_PARENT_IPC;
                if let Some(exit_result) = self.0.take() {
                    state.exit_result = exit_result;
                }
            });
        }
    }

    fn parent_ipc_task() -> io::Result<()> {
        let mut task_guard = ParentIpcTaskGuard(None);
        log::info!("Parent IPC setup started.");
        task_guard.0 = Some(parent_ipc()?);
        Ok(())
    }

    struct BackgroundTaskGuard;

    impl Drop for BackgroundTaskGuard {
        fn drop(&mut self) {
            WAIT_CHECKPOINT.notify(|state| {
                state.flags |= READY_BACKGROUND;
            });
        }
    }

    fn journal_task() -> io::Result<()> {
        let _task_guard = BackgroundTaskGuard;
        log::info!("Journal iteration started.");
        run_journal_loop::<NativeJournalRef>(&IPC_STATE, NATIVE_JOURNALD_PROVIDER.get().unwrap())
    }

    fn key_updater_task() -> io::Result<()> {
        let _task_guard = BackgroundTaskGuard;
        log::info!("Key watcher started.");
        run_watcher(&IPC_STATE)
    }

    let parent_ipc_handle = ThreadHandle::spawn(parent_ipc_task);
    let journal_handle = ThreadHandle::spawn(journal_task);
    let key_updater_handle = ThreadHandle::spawn(key_updater_task);

    static READY_MSG: &std::ffi::CStr = cstr!("READY=1");

    NATIVE_JOURNALD_PROVIDER
        .get()
        .unwrap()
        .sd_notify(READY_MSG)?;

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
                log::info!("Child spawned.");
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
) -> impl FnOnce() -> io::Result<()> + Send {
    move || {
        log::info!("Parent IPC ready.");
        let guard = IPC_STATE.done_notify().create_guard();
        let result = ipc_message_loop(child_output, &IPC_STATE);
        drop(guard);
        if !child_exited.has_notified() {
            IPC_STATE.methods().child_terminate()?;
        }
        result
    }
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
        status.parent_error = Some(e);
    }

    status
}
