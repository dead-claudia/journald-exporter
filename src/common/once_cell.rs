// A simple cell that can be atomically initialized once.

use crate::prelude::*;

use std::cell::UnsafeCell;

pub struct OnceCell<T> {
    once: Once,
    value: UnsafeCell<MaybeUninit<T>>,
}

// SAFETY: Cells can always be safely sent across boundaries.
unsafe impl<T: Send> Send for OnceCell<T> {}

// SAFETY: Cells can always be safely sent across boundaries. The `Send` bound here is because one
// thread could create it, send the cell to another thread, and the receiving thread would be the
// one dropping it. Thus, it's dropping a sent value.
unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}

impl<T> OnceCell<T> {
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        self.once.call_once_force(|_| {
            self.initialize(f);
        });

        // SAFETY: The value's initalized before `self.ready` is set.
        unsafe { (*self.value.get()).assume_init_ref() }
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        let mut result = Err(value);

        self.once.call_once_force(|_| {
            self.set_inner(replace(&mut result, Ok(())).unwrap_err());
        });

        result
    }

    #[cold]
    fn initialize(&self, f: impl FnOnce() -> T) {
        // SAFETY: It's locked to only one thread here.
        unsafe {
            self.value.get().write(MaybeUninit::new(f()));
        }
    }

    #[cold]
    fn set_inner(&self, value: T) {
        // SAFETY: It's locked to only one thread here.
        unsafe {
            self.value.get().write(MaybeUninit::new(value));
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.once.is_completed() {
            // SAFETY: The value's initalized before `self.ready` is set.
            Some(unsafe { (*self.value.get()).assume_init_ref() })
        } else {
            None
        }
    }
}
