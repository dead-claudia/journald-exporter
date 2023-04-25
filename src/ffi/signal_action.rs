use crate::prelude::*;

use super::syscall_utils::syscall_assert_int;
use super::Signal;
use std::ptr::addr_of_mut;

pub fn request_signal_when_parent_terminates(signal: Signal) {
    assert_not_miri();

    // SAFETY: doesn't impact any Rust-visible memory.
    unsafe {
        // The only result that could happen in practice is `EINVAL`, which results in a panic.
        // The condition for this is the signal number being invalid, and no named signal here
        // should ever trigger that.
        syscall_assert_int(
            "prctl",
            libc::prctl(libc::PR_SET_PDEATHSIG, signal.as_raw()),
        );
    }
}

// This is only used in setup code. It's okay for it to panic.
pub fn install_handler(
    signals: &[Signal],
    extra_flags: libc::c_int,
    handler: extern "C" fn(signum: Signal),
) {
    #[allow(clippy::as_conversions)]
    let action = handler as usize;

    assert_not_miri();

    if signals.is_empty() {
        return;
    }

    // I'd normally initialize this directly, but the layout of this is architecture-dependent
    // and it's more portable to just use libc. (It's *far* beyond me why this isn't the same
    // across all architectures, and just as much why `sigset_t` isn't publicly defined.)
    let mut sigaction = MaybeUninit::<libc::sigaction>::zeroed();

    // SAFETY: It's valid and doesn't do any pointer arithmetic that could make things unsafe.
    // It's all ultimately operating on a reference to a stack value.
    let sigaction = unsafe {
        let sigaction_ptr = sigaction.as_mut_ptr();

        let mask = addr_of_mut!((*sigaction_ptr).sa_mask);
        libc::sigemptyset(mask);
        for signal in signals {
            libc::sigaddset(mask, signal.as_raw());
        }

        *addr_of_mut!((*sigaction_ptr).sa_flags) = extra_flags;
        *addr_of_mut!((*sigaction_ptr).sa_sigaction) = action;
        sigaction.assume_init()
    };

    for signal in signals {
        // SAFETY: it's only passed in valid addresses, and the result is asserted.
        syscall_assert_int("sigaction", unsafe {
            libc::sigaction(signal.as_raw(), &sigaction, std::ptr::null_mut())
        });
    }

    // SAFETY: it's only passed in valid addresses, and the result is asserted.
    syscall_assert_int("sigprocmask", unsafe {
        libc::sigprocmask(libc::SIG_BLOCK, &sigaction.sa_mask, std::ptr::null_mut())
    });
}
