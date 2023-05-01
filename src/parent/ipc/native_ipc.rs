use crate::prelude::*;

use super::*;
use crate::ffi::PidFd;
use std::os::unix::prelude::OsStrExt;
use std::os::unix::process::CommandExt;
use std::process::Stdio;
use std::time::SystemTime;

// Don't leak the process in case of spawn error.
struct KillChildOnDrop<'a>(Option<&'a mut std::process::Child>);

#[cold]
fn panic_if_real_error(e: Error) {
    if e.kind() != ErrorKind::InvalidInput {
        panic!("Error occurred while attempting to kill child: {}", e);
    }
}

fn terminate_raw_child(child: &mut std::process::Child) {
    if let Err(e) = child.kill() {
        panic_if_real_error(e);
    }

    if let Err(e) = child.wait() {
        panic_if_real_error(e);
    }
}

impl Drop for KillChildOnDrop<'_> {
    fn drop(&mut self) {
        if let Some(child) = self.0.take() {
            terminate_raw_child(child)
        }
    }
}

type ChildStdio = (std::process::ChildStdin, std::process::ChildStdout);

struct UserGroupTableCacheEntry {
    table: Arc<UidGidTable>,
    expiry: Instant,
    last_updated: SystemTime,
}

pub struct NativeIpcMethods {
    child_state: Mutex<Option<PidFd>>,
    user_group_table_cache: Uncontended<Option<UserGroupTableCacheEntry>>,
}

impl NativeIpcMethods {
    pub const fn new() -> NativeIpcMethods {
        NativeIpcMethods {
            child_state: Mutex::new(None),
            user_group_table_cache: Uncontended::new(None),
        }
    }
}

impl ParentIpcMethods for NativeIpcMethods {
    type ChildInput = std::process::ChildStdin;
    type ChildOutput = std::process::ChildStdout;

    fn next_instant(&'static self) -> Instant {
        Instant::now()
    }

    fn get_user_group_table(&'static self) -> io::Result<Arc<UidGidTable>> {
        // Refresh only every 10 minutes, to not spam the file system every request (or request
        // batch).
        const USER_GROUP_REFRESH_RATE: Duration = Duration::from_secs(600);

        // This should only be called single-threaded. It's okay to hold the lock for an extended
        // period of time.
        let mut guard = self.user_group_table_cache.lock();

        if let Some(entry) = &*guard {
            if Instant::now() < entry.expiry {
                return Ok(entry.table.clone());
            }
        }

        let uid_file = std::fs::File::open("/etc/passwd")?;
        let gid_file = std::fs::File::open("/etc/group")?;

        let uid_updated = uid_file.metadata()?.modified()?;
        let gid_updated = gid_file.metadata()?.modified()?;
        let last_updated = uid_updated.max(gid_updated);

        if let Some(entry) = &mut *guard {
            if last_updated == entry.last_updated {
                entry.last_updated = last_updated;
                entry.expiry = Instant::now() + USER_GROUP_REFRESH_RATE;
                return Ok(entry.table.clone());
            }
        }

        fn read_file(mut file: std::fs::File) -> io::Result<IdTable> {
            // This is enough buffer for about anything realistic.
            let mut buf = [0; 4096];

            let mut parser = PasswdGroupParser::new();
            loop {
                let len = file.read(&mut buf)?;
                if len == 0 {
                    break;
                }
                if !parser.consume(&buf[..len]) {
                    return Err(Error::from_raw_os_error(libc::ENOMEM));
                }
            }
            Ok(parser.extract())
        }

        let uid_table = read_file(uid_file)?;
        let gid_table = read_file(gid_file)?;

        let table = Arc::new(UidGidTable::new(uid_table, gid_table));

        *guard = Some(UserGroupTableCacheEntry {
            table: table.clone(),
            expiry: Instant::now() + USER_GROUP_REFRESH_RATE,
            last_updated,
        });

        Ok(table)
    }

    fn child_spawn(
        &'static self,
        ipc_state: &'static ParentIpcState<Self>,
    ) -> io::Result<ChildStdio> {
        // TODO: switch from this `ipc::ChildWrap` + `PidFd` wrapper to `.create_pidfd(true)` +
        // `.pidfd()` once https://github.com/rust-lang/rust/issues/82971 gets stabilized. In
        // particular, this is technically racy as the `PidFd` is added after spawning, and
        // fixing that is non-trivial and requires some temporary Unix sockets to coordinate.
        let mut command = std::process::Command::new("/proc/self/exe");

        let ipc_dynamic = ipc_state.dynamic();

        let mut port_bytes = [0; MAX_USIZE_ASCII_BYTES];
        let port_bytes_start = write_u64(
            &mut port_bytes,
            zero_extend_u16_u64(ipc_dynamic.port.into()),
        );

        command.arg("--child-process");
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::inherit());
        command.uid(ipc_dynamic.child_user_group.uid);
        command.gid(ipc_dynamic.child_user_group.gid);

        command.env(
            "PORT",
            std::ffi::OsStr::from_bytes(&port_bytes[port_bytes_start..]),
        );

        if let Some(tls_options) = &ipc_dynamic.tls_config {
            command.env("TLS_CERTIFICATE", &tls_options.certificate);
            command.env("TLS_PRIVATE_KEY", &tls_options.private_key);
        }

        let mut child = command.spawn()?;
        drop(command);

        // Child reference itself not needed - I've got the pid FD to send signals, and I've got both
        // the input and output streams as handles.
        let pidfd = match PidFd::open_from_child(&child) {
            Ok(pidfd) => pidfd,
            Err(e) => {
                terminate_raw_child(&mut child);
                return Err(e);
            }
        };

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        stdin.write_all(&crate::state::ipc::VERSION_BYTES)?;

        {
            let mut guard = self.child_state.lock().unwrap_or_else(|e| e.into_inner());
            *guard = Some(pidfd);
        }

        Ok((stdin, stdout))
    }

    fn child_terminate(&'static self) -> io::Result<()> {
        let guard = self.child_state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(pidfd) = &*guard {
            pidfd.terminate()?;
        }
        Ok(())
    }

    fn child_wait(&self) -> IpcExitStatus {
        let pidfd = self
            .child_state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .expect("No child spawned.");

        let mut status = IpcExitStatus {
            result: None,
            parent_error: None,
            child_wait_error: None,
        };

        match pidfd.wait() {
            Ok(r) => status.result = Some(r),
            // If the child's already terminated (`ESRCH`), that's the desired end state, so no need to
            // complain about that. Other errors are worth complaining about.
            Err(e) if e.raw_os_error() == Some(libc::ESRCH) => {}
            Err(e) => status.child_wait_error = Some(e),
        }

        status
    }
}
