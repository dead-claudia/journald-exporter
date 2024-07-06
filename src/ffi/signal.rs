use crate::prelude::*;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signal(libc::c_int);

// This isn't authoritative. This just represents the max signal number *across all platforms* in
// Linux. `libc::SIGRTMAX()` is still checked as well in all cases. It's just used so I can
// pre-populate the signum lookup table.

// Ref: https://elixir.bootlin.com/linux/latest/source/arch/mips/include/uapi/asm/signal.h#L15
#[cfg(target_arch = "mips")]
const MAX_SIGNUM: usize = 128;

// All others have this maximum instead.
#[cfg(not(target_arch = "mips"))]
const MAX_SIGNUM: usize = 64;

// Not authoritative. Just the first signal reserved for them by POSIX.
const RT_SIGNUM_START: libc::c_int = 32;

#[cfg(test)]
pub struct SignalShrinker(propcheck::I32Shrinker);

#[cfg(test)]
impl propcheck::Shrinker for SignalShrinker {
    type Item = Signal;
    fn next(&mut self) -> Option<&Self::Item> {
        // SAFETY: Same layout
        unsafe { std::mem::transmute(self.0.next()) }
    }
}

#[cfg(test)]
impl propcheck::Arbitrary for Signal {
    type Shrinker = SignalShrinker;

    fn arbitrary() -> Self {
        // static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| Signal::all_signals().collect());
        static SIGNALS: OnceCell<Vec<Signal>> = OnceCell::new();
        propcheck::random_entry(SIGNALS.get_or_init(|| Signal::all_signals().collect()))
    }

    fn clone(&self) -> Self {
        *self
    }

    fn shrink(&self) -> Self::Shrinker {
        SignalShrinker(self.0.shrink())
    }
}

pub fn sigrtmax() -> Signal {
    const CAP: libc::c_int = reinterpret_u32_c_int(truncate_usize_u32(MAX_SIGNUM));
    Signal(libc::SIGRTMAX().max(CAP))
}

pub fn sigrtmin() -> Signal {
    Signal(libc::SIGRTMIN().min(RT_SIGNUM_START))
}

impl Signal {
    // Use only when it's *known* to be a valid signal.
    pub const fn from_raw(signum: i32) -> Signal {
        Signal(signum)
    }

    pub const fn as_raw(&self) -> i32 {
        self.0
    }

    #[allow(unused)]
    pub const SIGHUP: Signal = Signal(libc::SIGHUP);
    #[allow(unused)]
    pub const SIGINT: Signal = Signal(libc::SIGINT);
    #[allow(unused)]
    pub const SIGQUIT: Signal = Signal(libc::SIGQUIT);
    #[allow(unused)]
    pub const SIGILL: Signal = Signal(libc::SIGILL);
    #[allow(unused)]
    pub const SIGTRAP: Signal = Signal(libc::SIGTRAP);
    #[allow(unused)]
    pub const SIGABRT: Signal = Signal(libc::SIGABRT);
    #[allow(unused)]
    pub const SIGIOT: Signal = Signal(libc::SIGIOT);
    #[allow(unused)]
    pub const SIGBUS: Signal = Signal(libc::SIGBUS);
    #[cfg(target_arch = "mips")]
    #[allow(unused)]
    pub const SIGEMT: Signal = Signal(libc::SIGEMT);
    #[allow(unused)]
    pub const SIGFPE: Signal = Signal(libc::SIGFPE);
    #[allow(unused)]
    pub const SIGKILL: Signal = Signal(libc::SIGKILL);
    #[allow(unused)]
    pub const SIGUSR1: Signal = Signal(libc::SIGUSR1);
    #[allow(unused)]
    pub const SIGSEGV: Signal = Signal(libc::SIGSEGV);
    #[allow(unused)]
    pub const SIGUSR2: Signal = Signal(libc::SIGUSR2);
    #[allow(unused)]
    pub const SIGPIPE: Signal = Signal(libc::SIGPIPE);
    #[allow(unused)]
    pub const SIGALRM: Signal = Signal(libc::SIGALRM);
    #[allow(unused)]
    pub const SIGTERM: Signal = Signal(libc::SIGTERM);
    #[allow(unused)]
    pub const SIGSTKFLT: Signal = Signal(libc::SIGSTKFLT);
    #[allow(unused)]
    pub const SIGCHLD: Signal = Signal(libc::SIGCHLD);
    #[cfg(target_arch = "mips")]
    #[allow(unused)]
    pub const SIGCLD: Signal = Signal(libc::SIGCLD);
    #[allow(unused)]
    pub const SIGCONT: Signal = Signal(libc::SIGCONT);
    #[allow(unused)]
    pub const SIGSTOP: Signal = Signal(libc::SIGSTOP);
    #[allow(unused)]
    pub const SIGTSTP: Signal = Signal(libc::SIGTSTP);
    #[allow(unused)]
    pub const SIGTTIN: Signal = Signal(libc::SIGTTIN);
    #[allow(unused)]
    pub const SIGTTOU: Signal = Signal(libc::SIGTTOU);
    #[allow(unused)]
    pub const SIGURG: Signal = Signal(libc::SIGURG);
    #[allow(unused)]
    pub const SIGXCPU: Signal = Signal(libc::SIGXCPU);
    #[allow(unused)]
    pub const SIGXFSZ: Signal = Signal(libc::SIGXFSZ);
    #[allow(unused)]
    pub const SIGVTALRM: Signal = Signal(libc::SIGVTALRM);
    #[allow(unused)]
    pub const SIGPROF: Signal = Signal(libc::SIGPROF);
    #[allow(unused)]
    pub const SIGWINCH: Signal = Signal(libc::SIGWINCH);
    #[allow(unused)]
    pub const SIGIO: Signal = Signal(libc::SIGIO);
    #[allow(unused)]
    pub const SIGPOLL: Signal = Signal(libc::SIGPOLL);
    #[allow(unused)]
    pub const SIGPWR: Signal = Signal(libc::SIGPWR);
    // Not actually defined in the `libc` crate.
    // #[allow(unused)]
    // pub const SIGINFO: Signal = Signal(libc::SIGINFO);
    // #[allow(unused)]
    // pub const SIGLOST: Signal = Signal(libc::SIGLOST);
    #[allow(unused)]
    pub const SIGSYS: Signal = Signal(libc::SIGSYS);
    // Deprecated alias of `SIGSYS`
    // #[allow(unused)]
    // pub const SIGUNUSED: Signal = Signal(libc::SIGUNUSED);

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

static SIGNAL_NAME_STRINGS: [[u8; 11]; MAX_SIGNUM + 1] = {
    let mut result = [[0; 11]; MAX_SIGNUM + 1];

    let mut i = 0;
    while i <= MAX_SIGNUM {
        let mut current = *b"SIGRTMIN+00";
        let mut rt_signum =
            reinterpret_usize_isize(i).wrapping_sub(sign_extend_c_int_isize(RT_SIGNUM_START));

        if rt_signum < 0 {
            rt_signum = -rt_signum;
            current[8] = b'-';
        }
        let rt_signum = truncate_usize_u8(reinterpret_isize_usize(rt_signum));

        if rt_signum < 10 {
            current[9] = rt_signum + b'0';
            current[10] = 0;
        } else {
            current[9] = (rt_signum / 10) + b'0';
            current[10] = (rt_signum % 10) + b'0';
        }

        result[i] = current;
        i += 1;
    }

    // Commented-out entries aren't defined by `libc`.
    // result[zero_extend_c_int_usize(libc::SIGUNUSED)] = *b"SIGUNUSED\0\0";
    result[zero_extend_c_int_usize(libc::SIGSYS)] = *b"SIGSYS\0\0\0\0\0";
    // result[zero_extend_c_int_usize(libc::SIGLOST)] = *b"SIGLOST\0\0\0\0";
    // result[zero_extend_c_int_usize(libc::SIGINFO)] = *b"SIGINFO\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGPWR)] = *b"SIGPWR\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGPOLL)] = *b"SIGPOLL\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGIO)] = *b"SIGIO\0\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGWINCH)] = *b"SIGWINCH\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGPROF)] = *b"SIGPROF\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGVTALRM)] = *b"SIGVTALRM\0\0";
    result[zero_extend_c_int_usize(libc::SIGXFSZ)] = *b"SIGXFSZ\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGXCPU)] = *b"SIGXCPU\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGURG)] = *b"SIGURG\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGTTOU)] = *b"SIGTTOU\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGTTIN)] = *b"SIGTTIN\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGTSTP)] = *b"SIGTSTP\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGSTOP)] = *b"SIGSTOP\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGCONT)] = *b"SIGCONT\0\0\0\0";
    #[cfg(target_arch = "mips")]
    let _ = result[zero_extend_c_int_usize(libc::SIGCLD)] = *b"SIGCLD\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGCHLD)] = *b"SIGCHLD\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGSTKFLT)] = *b"SIGSTKFLT\0\0";
    result[zero_extend_c_int_usize(libc::SIGTERM)] = *b"SIGTERM\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGALRM)] = *b"SIGALRM\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGPIPE)] = *b"SIGPIPE\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGUSR2)] = *b"SIGUSR2\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGSEGV)] = *b"SIGSEGV\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGUSR1)] = *b"SIGUSR1\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGKILL)] = *b"SIGKILL\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGFPE)] = *b"SIGFPE\0\0\0\0\0";
    #[cfg(target_arch = "mips")]
    let _ = result[zero_extend_c_int_usize(libc::SIGEMT)] = *b"SIGEMT\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGBUS)] = *b"SIGBUS\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGIOT)] = *b"SIGIOT\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGABRT)] = *b"SIGABRT\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGTRAP)] = *b"SIGTRAP\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGILL)] = *b"SIGILL\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGQUIT)] = *b"SIGQUIT\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGINT)] = *b"SIGINT\0\0\0\0\0";
    result[zero_extend_c_int_usize(libc::SIGHUP)] = *b"SIGHUP\0\0\0\0\0";

    result
};

static SIGNAL_NAMES: [&str; MAX_SIGNUM + 1] = {
    let mut result = [""; MAX_SIGNUM + 1];
    let mut i = 0;
    while i <= MAX_SIGNUM {
        let mut current: &[u8] = &SIGNAL_NAME_STRINGS[i];
        while let [head @ .., 0] = current {
            current = head;
        }
        // SAFETY: The string is known to be valid UTF-8. The `from_raw_parts` is to work
        // around the fact array slicing isn't available in `const` contexts.
        result[i] = match std::str::from_utf8(current) {
            Ok(s) => s,
            Err(_) => panic!("Invalid UTF-8 detected."),
        };
        i += 1;
    }
    result
};

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 <= sigrtmax().0 {
            let mut signum = self.0;
            if signum >= RT_SIGNUM_START {
                signum = signum.wrapping_sub(sigrtmin().0.wrapping_sub(RT_SIGNUM_START));
            }
            if let Some(name) = SIGNAL_NAMES.get(zero_extend_c_int_usize(signum)) {
                // SAFETY: All strings are supposed to be valid UTF-8.
                return f.write_str(name);
            }
        }

        f.write_str("unknown (")?;
        self.0.fmt(f)?;
        f.write_char(')')
    }
}

impl fmt::Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}
