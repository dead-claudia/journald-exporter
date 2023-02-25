use crate::ffi::Signal;
use crate::prelude::*;

// This is so I don't cross up this with simple integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCode(pub u8);

#[cfg(test)]
impl Arbitrary for ExitCode {
    fn arbitrary(g: &mut Gen) -> Self {
        Self(Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.0.shrink().map(ExitCode))
    }
}

impl ExitCode {
    pub const fn from_raw(code: i32) -> ExitCode {
        ExitCode(truncate_i32_u8(code))
    }

    pub fn as_raw(&self) -> i32 {
        zero_extend_u8_i32(self.0)
    }
}

impl fmt::Display for ExitCode {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

// This is so I don't cross up this with simple integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitResult {
    Code(ExitCode),
    Signal(Signal),
}

#[cfg(test)]
impl Arbitrary for ExitResult {
    fn arbitrary(g: &mut Gen) -> Self {
        match <bool>::arbitrary(g) {
            true => ExitResult::Code(Arbitrary::arbitrary(g)),
            false => ExitResult::Signal(Arbitrary::arbitrary(g)),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            ExitResult::Code(code) => Box::new(code.shrink().map(ExitResult::Code)),
            ExitResult::Signal(signal) => Box::new(
                signal
                    .shrink()
                    .map(ExitResult::Signal)
                    .chain(ExitCode(u8::MAX).shrink().map(ExitResult::Code)),
            ),
        }
    }
}

impl ExitResult {
    pub fn as_exit_code(&self) -> ExitCode {
        match self {
            ExitResult::Code(code) => *code,
            ExitResult::Signal(signal) => {
                ExitCode(truncate_i32_u8(signal.as_raw().wrapping_add(128)))
            }
        }
    }
}

impl fmt::Display for ExitResult {
    #[cold]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExitResult::Code(code) => {
                f.write_str("code ")?;
                fmt::Display::fmt(&code, f)
            }
            ExitResult::Signal(signal) => {
                f.write_str("signal ")?;
                fmt::Display::fmt(signal, f)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::ffi::sigrtmin;

    #[test]
    fn exit_code_returns_right_exit_code_for_code() {
        for i in 0..=u8::MAX {
            assert_eq!(ExitCode(i).as_raw(), zero_extend_u8_i32(i));
        }
    }

    #[test]
    fn exit_code_from_raw_works() {
        for i in 0..=u8::MAX {
            assert_eq!(
                ExitCode::from_raw(zero_extend_u8_i32(i)).as_raw(),
                zero_extend_u8_i32(i)
            );
        }
    }

    #[test]
    fn exit_result_returns_right_exit_code_for_code() {
        for i in 0..=u8::MAX {
            assert_eq!(ExitResult::Code(ExitCode(i)).as_exit_code(), ExitCode(i));
        }
    }

    #[test]
    fn exit_result_returns_right_exit_code_for_signal() {
        for i in Signal::all_signals() {
            assert_eq!(
                ExitResult::Signal(i).as_exit_code().as_raw(),
                i.as_raw() + 128
            );
        }
    }

    #[test]
    fn exit_result_prints_correct_display_for_exit_result() {
        for i in 0..=u8::MAX {
            assert_eq!(
                ExitResult::Code(ExitCode(i)).to_string(),
                format!("code {i}")
            );
        }
    }

    #[test]
    fn exit_result_prints_correct_display_for_known_signals() {
        // Commented-out lines are either duplicates or unused entries.
        const PAIRS: &[(Signal, &str)] = &[
            (Signal::SIGHUP, "signal SIGHUP"),
            (Signal::SIGINT, "signal SIGINT"),
            (Signal::SIGQUIT, "signal SIGQUIT"),
            (Signal::SIGILL, "signal SIGILL"),
            (Signal::SIGTRAP, "signal SIGTRAP"),
            (Signal::SIGABRT, "signal SIGABRT"),
            // (Signal::SIGIOT, "signal SIGIOT"),
            (Signal::SIGBUS, "signal SIGBUS"),
            #[cfg(target_arch = "mips")]
            (Signal::SIGEMT, "signal SIGEMT"),
            (Signal::SIGFPE, "signal SIGFPE"),
            (Signal::SIGKILL, "signal SIGKILL"),
            (Signal::SIGUSR1, "signal SIGUSR1"),
            (Signal::SIGSEGV, "signal SIGSEGV"),
            (Signal::SIGUSR2, "signal SIGUSR2"),
            (Signal::SIGPIPE, "signal SIGPIPE"),
            (Signal::SIGALRM, "signal SIGALRM"),
            (Signal::SIGTERM, "signal SIGTERM"),
            #[cfg(not(target_arch = "mips"))]
            (Signal::SIGSTKFLT, "signal SIGSTKFLT"),
            (Signal::SIGCHLD, "signal SIGCHLD"),
            // #[cfg(target_arch = "mips")]
            // (Signal::SIGCLD, "signal SIGCLD"),
            (Signal::SIGCONT, "signal SIGCONT"),
            (Signal::SIGSTOP, "signal SIGSTOP"),
            (Signal::SIGTSTP, "signal SIGTSTP"),
            (Signal::SIGTTIN, "signal SIGTTIN"),
            (Signal::SIGTTOU, "signal SIGTTOU"),
            (Signal::SIGURG, "signal SIGURG"),
            (Signal::SIGXCPU, "signal SIGXCPU"),
            (Signal::SIGXFSZ, "signal SIGXFSZ"),
            (Signal::SIGVTALRM, "signal SIGVTALRM"),
            (Signal::SIGPROF, "signal SIGPROF"),
            (Signal::SIGWINCH, "signal SIGWINCH"),
            (Signal::SIGIO, "signal SIGIO"),
            // (Signal::SIGPOLL, "signal SIGPOLL"),
            (Signal::SIGPWR, "signal SIGPWR"),
            // (Signal::SIGINFO, "signal SIGINFO"),
            // (Signal::SIGLOST, "signal SIGLOST"),
            (Signal::SIGSYS, "signal SIGSYS"),
        ];

        for (signal, expected) in PAIRS.iter().copied() {
            assert_eq!(ExitResult::Signal(signal).to_string(), expected);
        }
    }

    #[test]
    fn exit_result_prints_correct_display_for_realtime_signals() {
        for i in Signal::rt_signals() {
            assert_eq!(
                ExitResult::Signal(i).to_string(),
                format!("signal SIGRTMIN+{}", i.as_raw() - sigrtmin().as_raw())
            );
        }
    }
}
