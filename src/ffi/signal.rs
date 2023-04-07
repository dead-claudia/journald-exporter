use super::syscall_utils::syscall_check_int;
use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signal(u8);

#[cfg(test)]
impl Arbitrary for Signal {
    fn arbitrary(g: &mut Gen) -> Self {
        // static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| Signal::all_signals().collect());
        static SIGNALS: OnceCell<Vec<Signal>> = OnceCell::new();
        let signals = SIGNALS.get_or_init(|| Signal::all_signals().collect());

        *g.choose(signals).unwrap()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.0
                .shrink()
                .filter(|i| *i > sigrtmin().0 || *i <= 31)
                .map(Signal),
        )
    }
}

#[cfg(test)]
pub fn sigrtmax() -> Signal {
    let result = libc::SIGRTMAX();
    if result > zero_extend_u8_i32(u8::MAX) {
        panic!("Unexpectedly high max real-time signal: {}", result);
    }
    Signal(truncate_i32_u8(result))
}

pub fn sigrtmin() -> Signal {
    if cfg!(miri) {
        // Hack: expose glibc's normal value for this, since Miri doesn't implement the underlying
        // libc call. Ref: https://github.com/rust-lang/miri/issues/2832
        Signal(35)
    } else {
        let result = libc::SIGRTMIN();
        if result > zero_extend_u8_i32(u8::MAX) {
            panic!("Unexpectedly high min real-time signal: {}", result);
        }
        Signal(truncate_i32_u8(result))
    }
}

impl Signal {
    // Use only when it's *known* to be a valid signal.
    pub const fn from_raw(signum: i32) -> Signal {
        Signal(truncate_i32_u8(signum))
    }

    pub const fn as_raw(&self) -> i32 {
        zero_extend_u8_c_int(self.0)
    }

    pub fn request_signal_when_parent_terminates(signal: Signal) {
        if cfg!(miri) {
            return;
        }

        // SAFETY: doesn't impact any Rust-visible memory.
        unsafe {
            // The only result that could happen in practice is `EINVAL`, which results in a panic.
            // The condition for this is the signal number being invalid, and no named signal here
            // should ever trigger that.
            drop(syscall_check_int(
                "prctl",
                libc::prctl(libc::PR_SET_PDEATHSIG, signal.as_raw()),
            ));
        }
    }

    #[allow(unused)]
    pub const SIGHUP: Signal = Signal(truncate_i32_u8(libc::SIGHUP));
    #[allow(unused)]
    pub const SIGINT: Signal = Signal(truncate_i32_u8(libc::SIGINT));
    #[allow(unused)]
    pub const SIGQUIT: Signal = Signal(truncate_i32_u8(libc::SIGQUIT));
    #[allow(unused)]
    pub const SIGILL: Signal = Signal(truncate_i32_u8(libc::SIGILL));
    #[allow(unused)]
    pub const SIGTRAP: Signal = Signal(truncate_i32_u8(libc::SIGTRAP));
    #[allow(unused)]
    pub const SIGABRT: Signal = Signal(truncate_i32_u8(libc::SIGABRT));
    #[allow(unused)]
    pub const SIGIOT: Signal = Signal(truncate_i32_u8(libc::SIGIOT));
    #[allow(unused)]
    pub const SIGBUS: Signal = Signal(truncate_i32_u8(libc::SIGBUS));
    #[cfg(target_arch = "mips")]
    #[allow(unused)]
    pub const SIGEMT: Signal = Signal(truncate_i32_u8(libc::SIGEMT));
    #[allow(unused)]
    pub const SIGFPE: Signal = Signal(truncate_i32_u8(libc::SIGFPE));
    #[allow(unused)]
    pub const SIGKILL: Signal = Signal(truncate_i32_u8(libc::SIGKILL));
    #[allow(unused)]
    pub const SIGUSR1: Signal = Signal(truncate_i32_u8(libc::SIGUSR1));
    #[allow(unused)]
    pub const SIGSEGV: Signal = Signal(truncate_i32_u8(libc::SIGSEGV));
    #[allow(unused)]
    pub const SIGUSR2: Signal = Signal(truncate_i32_u8(libc::SIGUSR2));
    #[allow(unused)]
    pub const SIGPIPE: Signal = Signal(truncate_i32_u8(libc::SIGPIPE));
    #[allow(unused)]
    pub const SIGALRM: Signal = Signal(truncate_i32_u8(libc::SIGALRM));
    #[allow(unused)]
    pub const SIGTERM: Signal = Signal(truncate_i32_u8(libc::SIGTERM));
    #[allow(unused)]
    pub const SIGSTKFLT: Signal = Signal(truncate_i32_u8(libc::SIGSTKFLT));
    #[allow(unused)]
    pub const SIGCHLD: Signal = Signal(truncate_i32_u8(libc::SIGCHLD));
    #[cfg(target_arch = "mips")]
    #[allow(unused)]
    pub const SIGCLD: Signal = Signal(truncate_i32_u8(libc::SIGCLD));
    #[allow(unused)]
    pub const SIGCONT: Signal = Signal(truncate_i32_u8(libc::SIGCONT));
    #[allow(unused)]
    pub const SIGSTOP: Signal = Signal(truncate_i32_u8(libc::SIGSTOP));
    #[allow(unused)]
    pub const SIGTSTP: Signal = Signal(truncate_i32_u8(libc::SIGTSTP));
    #[allow(unused)]
    pub const SIGTTIN: Signal = Signal(truncate_i32_u8(libc::SIGTTIN));
    #[allow(unused)]
    pub const SIGTTOU: Signal = Signal(truncate_i32_u8(libc::SIGTTOU));
    #[allow(unused)]
    pub const SIGURG: Signal = Signal(truncate_i32_u8(libc::SIGURG));
    #[allow(unused)]
    pub const SIGXCPU: Signal = Signal(truncate_i32_u8(libc::SIGXCPU));
    #[allow(unused)]
    pub const SIGXFSZ: Signal = Signal(truncate_i32_u8(libc::SIGXFSZ));
    #[allow(unused)]
    pub const SIGVTALRM: Signal = Signal(truncate_i32_u8(libc::SIGVTALRM));
    #[allow(unused)]
    pub const SIGPROF: Signal = Signal(truncate_i32_u8(libc::SIGPROF));
    #[allow(unused)]
    pub const SIGWINCH: Signal = Signal(truncate_i32_u8(libc::SIGWINCH));
    #[allow(unused)]
    pub const SIGIO: Signal = Signal(truncate_i32_u8(libc::SIGIO));
    #[allow(unused)]
    pub const SIGPOLL: Signal = Signal(truncate_i32_u8(libc::SIGPOLL));
    #[allow(unused)]
    pub const SIGPWR: Signal = Signal(truncate_i32_u8(libc::SIGPWR));
    // Not actually defined in the `libc` crate.
    // #[allow(unused)]
    // pub const SIGINFO: Signal = Signal(truncate_i32_u8(libc::SIGINFO));
    // #[allow(unused)]
    // pub const SIGLOST: Signal = Signal(truncate_i32_u8(libc::SIGLOST));
    #[allow(unused)]
    pub const SIGSYS: Signal = Signal(truncate_i32_u8(libc::SIGSYS));
    // Deprecated alias of `SIGSYS`
    // #[allow(unused)]
    // pub const SIGUNUSED: Signal = Signal(truncate_i32_u8(libc::SIGUNUSED));

    #[cfg(test)]
    pub fn rt_signals() -> impl Iterator<Item = Signal> {
        (sigrtmin().0..=sigrtmax().0).map(Signal)
    }

    #[cfg(test)]
    pub fn all_signals() -> impl Iterator<Item = Signal> {
        static STATIC_SIGNALS: &[Signal] = &[
            Signal::SIGHUP,
            Signal::SIGINT,
            Signal::SIGQUIT,
            Signal::SIGILL,
            Signal::SIGTRAP,
            Signal::SIGABRT,
            Signal::SIGIOT,
            Signal::SIGBUS,
            #[cfg(target_arch = "mips")]
            Signal::SIGEMT,
            Signal::SIGFPE,
            Signal::SIGKILL,
            Signal::SIGUSR1,
            Signal::SIGSEGV,
            Signal::SIGUSR2,
            Signal::SIGPIPE,
            Signal::SIGALRM,
            Signal::SIGTERM,
            Signal::SIGSTKFLT,
            Signal::SIGCHLD,
            #[cfg(target_arch = "mips")]
            Signal::SIGCLD,
            Signal::SIGCONT,
            Signal::SIGSTOP,
            Signal::SIGTSTP,
            Signal::SIGTTIN,
            Signal::SIGTTOU,
            Signal::SIGURG,
            Signal::SIGXCPU,
            Signal::SIGXFSZ,
            Signal::SIGVTALRM,
            Signal::SIGPROF,
            Signal::SIGWINCH,
            Signal::SIGIO,
            Signal::SIGPOLL,
            Signal::SIGPWR,
            // Not actually defined in the `libc` crate.
            // Signal::SIGINFO,
            // Signal::SIGLOST,
            Signal::SIGSYS,
        ];

        STATIC_SIGNALS.iter().copied().chain(Signal::rt_signals())
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // It's okay if some of them are unreachable, as signal numbers differ across architectures
        // and are sometimes aliased.
        #![allow(unreachable_patterns)]

        let (prefix, offset) = match *self {
            Signal::SIGHUP => ("SIGHUP", None),
            Signal::SIGINT => ("SIGINT", None),
            Signal::SIGQUIT => ("SIGQUIT", None),
            Signal::SIGILL => ("SIGILL", None),
            Signal::SIGTRAP => ("SIGTRAP", None),
            Signal::SIGABRT => ("SIGABRT", None),
            Signal::SIGIOT => ("SIGIOT", None),
            Signal::SIGBUS => ("SIGBUS", None),
            #[cfg(target_arch = "mips")]
            Signal::SIGEMT => ("SIGEMT", None),
            Signal::SIGFPE => ("SIGFPE", None),
            Signal::SIGKILL => ("SIGKILL", None),
            Signal::SIGUSR1 => ("SIGUSR1", None),
            Signal::SIGSEGV => ("SIGSEGV", None),
            Signal::SIGUSR2 => ("SIGUSR2", None),
            Signal::SIGPIPE => ("SIGPIPE", None),
            Signal::SIGALRM => ("SIGALRM", None),
            Signal::SIGTERM => ("SIGTERM", None),
            Signal::SIGSTKFLT => ("SIGSTKFLT", None),
            Signal::SIGCHLD => ("SIGCHLD", None),
            #[cfg(target_arch = "mips")]
            Signal::SIGCLD => ("SIGCLD", None),
            Signal::SIGCONT => ("SIGCONT", None),
            Signal::SIGSTOP => ("SIGSTOP", None),
            Signal::SIGTSTP => ("SIGTSTP", None),
            Signal::SIGTTIN => ("SIGTTIN", None),
            Signal::SIGTTOU => ("SIGTTOU", None),
            Signal::SIGURG => ("SIGURG", None),
            Signal::SIGXCPU => ("SIGXCPU", None),
            Signal::SIGXFSZ => ("SIGXFSZ", None),
            Signal::SIGVTALRM => ("SIGVTALRM", None),
            Signal::SIGPROF => ("SIGPROF", None),
            Signal::SIGWINCH => ("SIGWINCH", None),
            Signal::SIGIO => ("SIGIO", None),
            Signal::SIGPOLL => ("SIGPOLL", None),
            Signal::SIGPWR => ("SIGPWR", None),
            // Signal::SIGINFO => ("SIGINFO", None),
            // Signal::SIGLOST => ("SIGLOST", None),
            Signal::SIGSYS => ("SIGSYS", None),
            // Signal::SIGUNUSED => ("SIGUNUSED", None),
            Signal(inner) => {
                let (offset, is_negative) =
                    zero_extend_u8_i32(inner).overflowing_sub(sigrtmin().0.into());

                (
                    if is_negative { "SIGRTMIN" } else { "SIGRTMIN+" },
                    Some(offset),
                )
            }
        };

        f.write_str(prefix)?;
        offset.map_or(Ok(()), |o| o.fmt(f))
    }
}

impl fmt::Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
