use crate::prelude::*;

use super::syscall_utils::syscall_check_long;
use super::ExitResult;
use super::Signal;
use super::SignalAction;
use super::SignalActionFlags;
use super::SignalHandler;
use super::SignalSet;
use crate::ffi::panic_errno;
use crate::ffi::syscall_utils::syscall_check_int;
use crate::ffi::ExitCode;
use crate::ffi::PollFlags;
use crate::ffi::Pollable;
use std::os::unix::prelude::*;

#[derive(Debug)]
pub struct PidFd {
    fd: OwnedFd,
}

impl PidFd {
    pub fn open_from_child(child: &std::process::Child) -> io::Result<PidFd> {
        // SAFETY: the syscall is ensured correct by passing in a known child ID, and it returns a
        // valid PID FD to plug into `PidFd::from_raw_fd`. The underlying syscall doesn't read any
        // Rust-observable code, and it's thread-safe, so there aren't any memory ramifications.
        unsafe {
            let result = syscall_check_long(
                "pidfd_open",
                libc::syscall(libc::SYS_pidfd_open, reinterpret_u32_i32(child.id()), 0),
            )?;
            Ok(PidFd::from_raw_fd(truncate_c_long_i32(result)))
        }
    }

    pub fn terminate(&self) -> io::Result<()> {
        let fd = self.fd.as_raw_fd();
        // SAFETY: No pointers here are actually being provided aside from an (accepted) null
        // pointer to represent no `siginfo_t` record being passed. There's no memory safety
        // concerns here.
        unsafe {
            syscall_check_long(
                "pidfd_send_signal",
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    fd,
                    Signal::SIGTERM.as_raw(),
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                ),
            )?;
            Ok(())
        }
    }

    pub fn wait(&self) -> io::Result<ExitResult> {
        static SETUP_WAITID_HACK: Once = Once::new();

        SETUP_WAITID_HACK.call_once(|| {
            /*
            Work around a very subtle quirk around `waitid`/etc. that causes `ECHILD` to be
            returned for child processes.

            From the `waitid` man page:

            > Errors:
            >     [...]
            >     `ECHILD` (for `waitpid()` or `waitid()`) The process specified by `pid`
            >     (`waitpid()`) or `idtype` and `id` (`waitid()`) does not exist or is not a child
            >     of the calling process. (This can happen for one's own child if the action for
            >     `SIGCHLD` is set to `SIG_IGN`. See also the Linux Notes section about threads.)
            >
            > [...]
            >
            > Notes:
            >     [...]
            >     POSIX.1-2001 specifies that if the disposition of `SIGCHLD` is set to `SIG_IGN`
            >     or the `SA_NOCLDWAIT` flag is set for `SIGCHLD` (see `sigaction(2)`), then
            >     children that terminate do not become zombies and a call to `wait()` or
            >     `waitpid()` will block until all children have terminated, and then fail with
            >     errno set to `ECHILD`. (The original POSIX standard left the behavior of setting
            >     `SIGCHLD` to `SIG_IGN` unspecified. Note that even though the default disposition
            >     of `SIGCHLD` is "ignore", explicitly setting the disposition to `SIG_IGN` results
            >     in different treatment of zombie process children.)

            Unfortunately, the default action for this signal is, in fact, `SIG_IGN`, and so I have
            to employ this hack to ensure child processes are retained as proper `wait`able
            zombies.
            */

            struct NoopHandler;
            impl SignalHandler for NoopHandler {
                fn on_signal(_: Signal) {
                    // do nothing
                }
            }

            let action =
                SignalAction::new::<NoopHandler>(SignalSet::empty(), SignalActionFlags::empty());

            if let Err(e) = action.install(Signal::SIGCHLD) {
                panic_errno(e, "sigaction")
            }

            let signal_set = SignalSet::from_iter([Signal::SIGCHLD]);

            if let Err(e) = SignalSet::set_blocked(&signal_set) {
                panic_errno(e, "sigprocmask")
            };
        });

        // Per the man page, polling `in` is what's needed to poll for child exit.
        self.fd.poll(PollFlags::IN, None)?;

        // SAFETY: The `info` pointer is first initialized via `waitid`, then `info.si_code` is
        // checked to verify the correct meaning of `info.si_status` before reading that. Other
        // fields aren't used, so they can be safely assumed "initialized" to anything, even
        // garbage.
        unsafe {
            let mut info = MaybeUninit::uninit();

            syscall_check_int(
                "pidfd_send_signal",
                libc::waitid(
                    libc::P_PIDFD,
                    reinterpret_i32_c_uint(self.fd.as_raw_fd()),
                    info.as_mut_ptr(),
                    libc::WEXITED,
                ),
            )?;

            let info = info.assume_init();
            match info.si_code {
                libc::CLD_EXITED => Ok(ExitResult::Code(ExitCode::from_raw(info.si_status()))),
                libc::CLD_KILLED | libc::CLD_DUMPED => {
                    Ok(ExitResult::Signal(Signal::from_raw(info.si_status())))
                }
                _ => Err(Error::new(
                    ErrorKind::Other,
                    format!("Unexpected `si_code` from wait: {}", info.si_code),
                )),
            }
        }
    }
}

impl FromRawFd for PidFd {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            fd: OwnedFd::from_raw_fd(fd),
        }
    }
}

impl AsRawFd for PidFd {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl AsFd for PidFd {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl IntoRawFd for PidFd {
    fn into_raw_fd(self) -> RawFd {
        self.fd.into_raw_fd()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // So I can actually read and check this external to the program.
    fn get_pidfd_pid(pidfd: &PidFd) -> u32 {
        use io::BufRead as _;

        let f = std::fs::File::open(format!("/proc/self/fdinfo/{}", pidfd.as_raw_fd())).unwrap();

        let mut lines = io::BufReader::new(f);
        let mut line = String::new();

        while lines.read_line(&mut line).unwrap() > 0 {
            if let Some(suffix) = line.strip_prefix("Pid:") {
                let suffix = suffix.trim();
                match suffix.parse() {
                    Ok(pidfd) => return pidfd,
                    Err(_) => panic!("Invalid PID line: {}", suffix),
                }
            }
            line.clear();
        }

        panic!("Could not find PID!");
    }

    // Don't leak the process in case of spawn error.
    struct TestProcess(Option<std::process::Child>);

    impl TestProcess {
        fn spawn() -> Self {
            TestProcess(Some(
                std::process::Command::new("/bin/cat")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .unwrap(),
            ))
        }

        fn inner(&mut self) -> &mut std::process::Child {
            self.0.as_mut().unwrap()
        }
    }

    impl Drop for TestProcess {
        fn drop(&mut self) {
            let Some(child) = self.0.as_mut() else {
                return;
            };

            match child.kill() {
                Ok(()) => {}
                // It's okay if the child doesn't exist. That's the desired state.
                Err(e) if matches!(e.raw_os_error(), Some(libc::ESRCH | libc::EINVAL)) => {}
                Err(e) => panic!("Error occurred while attempting to kill child: {}", e),
            }

            match child.wait() {
                Ok(_) => {}
                // It's okay if the child doesn't exist. That's the desired state.
                Err(e) if matches!(e.raw_os_error(), Some(libc::ESRCH | libc::EINVAL)) => {}
                Err(e) => panic!("Error occurred while attempting to wait child: {}", e),
            }
        }
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn pidfd_open_works() {
        let mut child = TestProcess::spawn();
        let inner = child.inner();

        let id = inner.id();

        let pidfd = PidFd::open_from_child(inner).unwrap();

        assert_eq!(id, get_pidfd_pid(&pidfd));

        // Close the input so it dies naturally.
        drop(inner.stdin.take());

        let result = pidfd.wait().unwrap();
        child.0 = None;

        assert_eq!(result, ExitResult::Code(ExitCode(0)));
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn pidfd_open_can_kill_pidfd() {
        let mut child = TestProcess::spawn();
        let inner = child.inner();

        let id = inner.id();

        let pidfd = PidFd::open_from_child(inner).unwrap();

        assert_eq!(id, get_pidfd_pid(&pidfd));

        pidfd.terminate().unwrap();

        // Close the input so it dies naturally.
        drop(inner.stdin.take());

        let result = pidfd.wait().unwrap();
        child.0 = None;

        assert_eq!(result, ExitResult::Signal(Signal::SIGTERM));
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn pidfd_passes_through_raw_fd_correctly() {
        let mut child = TestProcess::spawn();
        let pidfd = PidFd::open_from_child(child.inner()).unwrap();

        let id = pidfd.into_raw_fd();

        // SAFETY: Was just pulled from a known-valid FD.
        let pidfd = unsafe { PidFd::from_raw_fd(id) };
        assert_eq!(pidfd.as_raw_fd(), id);
        assert_eq!(pidfd.as_fd().as_raw_fd(), id);
        assert_eq!(pidfd.into_raw_fd(), id);

        // SAFETY: Was just pulled from a known-valid FD.
        let pidfd = unsafe { PidFd::from_raw_fd(id) };
        pidfd.terminate().unwrap();
        pidfd.wait().unwrap();
        child.0 = None;
    }
}
