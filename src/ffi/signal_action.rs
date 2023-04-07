use crate::prelude::*;

use super::syscall_utils::syscall_check_int;
use super::Signal;
use super::SignalSet;

pub trait SignalHandler {
    fn on_signal(signal: Signal);
}

pub struct SignalAction {
    sigaction: libc::sigaction,
}

impl SignalAction {
    pub fn new<H: SignalHandler>(additional_signals: SignalSet) -> SignalAction {
        extern "C" fn handler_wrap<H: SignalHandler>(
            signum: libc::c_int,
            _info: *const libc::siginfo_t,
            _ucontext: *const libc::c_void,
        ) {
            // SAFETY: It's a valid signal coming in, and I don't want unnecessary panics.
            H::on_signal(Signal::from_raw(signum))
        }

        // SAFETY: zeroed is a valid initialization pattern. I'd normally initialize this directly,
        // but the layout of this differs based on architecture and it's more portable this way.
        let mut sigaction = unsafe { MaybeUninit::<libc::sigaction>::zeroed().assume_init() };
        sigaction.sa_mask = additional_signals.into_raw();
        sigaction.sa_flags = libc::SA_SIGINFO;
        #[allow(clippy::as_conversions)]
        let action = handler_wrap::<H> as usize;
        sigaction.sa_sigaction = action;
        SignalAction { sigaction }
    }

    pub fn install(&self, signal: Signal) -> io::Result<()> {
        assert_not_miri();

        // SAFETY: it's only passed in valid addresses, and the result is asserted.
        syscall_check_int("sigaction", unsafe {
            libc::sigaction(signal.as_raw(), &self.sigaction, std::ptr::null_mut())
        })?;
        Ok(())
    }
}
