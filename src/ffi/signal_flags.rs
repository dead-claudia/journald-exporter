// FIXME: re-evaluate once https://github.com/rust-lang/rust-clippy/pull/10309 is released.
#![allow(clippy::arithmetic_side_effects)]

bitflags! {
    pub struct SignalActionFlags: libc::c_int {
        const RESET_HANDLER = libc::SA_RESETHAND;
        const REMAIN_SET_WITHIN_SIGNAL_HANDLER = libc::SA_NODEFER;
        const RESTART_SYSCALL_ON_INTERRUPT = libc::SA_RESTART;
        const IGNORE_CHILD_STOP_RESUME = libc::SA_NOCLDSTOP;
        const USE_ALTERNATIVE_STACK = libc::SA_ONSTACK;
        const IGNORE_TERMINATED_CHILDREN = libc::SA_NOCLDWAIT;
    }
}
