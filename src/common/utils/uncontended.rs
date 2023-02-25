//! Utility to avoid going through the ceremony of an entire mutex just for cases where a value in
//! practice won't actually be accessed by multiple threads. It has the same memory overhead as a
//! mutex, but only to assert at runtime that it's uncontended - it won't even attempt a spinlock.

use crate::prelude::*;

use std::cell::UnsafeCell;

pub struct Uncontended<T> {
    contended: AtomicBool,
    value: UnsafeCell<T>,
}

// SAFETY: It provides mutex-like guarantees
unsafe impl<T: Send> Send for Uncontended<T> {}
// SAFETY: It provides mutex-like guarantees
unsafe impl<T: Send> Sync for Uncontended<T> {}

pub struct UncontendedGuard<'a, T> {
    guarded: &'a Uncontended<T>,
}

#[cold]
fn fail_contended() -> ! {
    panic!("Unexpected lock contention");
}

impl<T> Uncontended<T> {
    pub const fn new(value: T) -> Self {
        Self {
            contended: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> UncontendedGuard<T> {
        if self.contended.swap(true, Ordering::AcqRel) {
            fail_contended();
        }

        UncontendedGuard { guarded: self }
    }

    // pub fn get_mut(&mut self) -> &mut T {
    //     if *self.contended.get_mut() {
    //         fail_contended();
    //     }

    //     self.value.get_mut()
    // }
}

impl<T> Drop for UncontendedGuard<'_, T> {
    fn drop(&mut self) {
        // Release the lock.
        self.guarded.contended.store(false, Ordering::Release);
    }
}

impl<T> std::ops::Deref for UncontendedGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: Exclusive access is enforced when acquiring the guard by panicking if it's
        // already locked, and the returned pointer always points to a valid reference.
        unsafe { &*self.guarded.value.get() }
    }
}

impl<T> std::ops::DerefMut for UncontendedGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Exclusive access is enforced when acquiring the guard by panicking if it's
        // already locked, and the returned pointer always points to a valid reference.
        unsafe { &mut *self.guarded.value.get() }
    }
}

impl<T: fmt::Debug> fmt::Debug for Uncontended<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.lock().fmt(f)
    }
}

impl<T: fmt::Debug> fmt::Debug for UncontendedGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}
