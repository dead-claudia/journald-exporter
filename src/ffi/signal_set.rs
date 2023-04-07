use crate::prelude::*;

use super::syscall_utils::syscall_check_int;
use super::Signal;

pub struct SignalSet {
    sigset: libc::sigset_t,
}

impl SignalSet {
    pub fn empty() -> SignalSet {
        assert_not_miri();

        // SAFETY: Nothing is allocated, and I'm only using well-defined libc functions with valid
        // pointers.
        unsafe {
            let mut sigset = MaybeUninit::<libc::sigset_t>::uninit();
            libc::sigemptyset(sigset.as_mut_ptr());
            SignalSet {
                sigset: sigset.assume_init(),
            }
        }
    }

    pub fn add(&mut self, signal: Signal) {
        assert_not_miri();

        // SAFETY: `self.sigset` is obviously initialized here, and `sigaddset` is the standard way
        // to add signals to a given sigset.
        unsafe {
            libc::sigaddset(&mut self.sigset, signal.as_raw());
        }
    }

    pub fn into_raw(self) -> libc::sigset_t {
        self.sigset
    }

    fn update_proc_mask(s: &SignalSet, how: libc::c_int) -> io::Result<()> {
        assert_not_miri();

        // SAFETY: it's only passed in valid addresses, and the result is asserted.
        syscall_check_int("sigprocmask", unsafe {
            libc::sigprocmask(how, &s.sigset, std::ptr::null_mut())
        })?;

        Ok(())
    }

    pub fn set_blocked(s: &SignalSet) -> io::Result<()> {
        SignalSet::update_proc_mask(s, libc::SIG_BLOCK)
    }
}

impl FromIterator<Signal> for SignalSet {
    fn from_iter<I: IntoIterator<Item = Signal>>(iter: I) -> Self {
        let mut set = SignalSet::empty();
        for signal in iter {
            set.add(signal);
        }
        set
    }
}

impl<'a> FromIterator<&'a Signal> for SignalSet {
    fn from_iter<I: IntoIterator<Item = &'a Signal>>(iter: I) -> Self {
        assert_not_miri();

        let mut set = SignalSet::empty();
        for signal in iter {
            set.add(*signal);
        }
        set
    }
}
