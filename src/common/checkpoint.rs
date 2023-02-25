use crate::prelude::*;

#[derive(Debug)]
pub struct Checkpoint<T> {
    cvar: Condvar,
    mutex: Mutex<T>,
}

impl<T> Checkpoint<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            mutex: Mutex::new(inner),
            cvar: Condvar::new(),
        }
    }

    pub fn try_notify<R>(&self, f: impl FnOnce(&mut T) -> (bool, R)) -> R {
        // Why hold the lock for the whole block? See: https://stackoverflow.com/a/66162551
        let mut guard = self.mutex.lock().unwrap_or_else(|e| e.into_inner());
        let (should_notify, result) = f(&mut *guard);
        if should_notify {
            self.cvar.notify_all();
        }
        result
    }

    pub fn notify<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.try_notify(|guard| (true, f(guard)))
    }

    pub fn wait(&self) -> MutexGuard<T> {
        self.resume_wait(self.lock())
    }

    pub fn wait_for(&self, timeout: Duration) -> MutexGuard<T> {
        self.resume_wait_for(timeout, self.lock())
    }

    pub fn resume_wait<'a>(&'a self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.cvar.wait(guard).unwrap_or_else(|e| e.into_inner())
    }

    pub fn resume_wait_for<'a>(
        &'a self,
        timeout: Duration,
        guard: MutexGuard<'a, T>,
    ) -> MutexGuard<'a, T> {
        let (guard, _) = self
            .cvar
            .wait_timeout(guard, timeout)
            .unwrap_or_else(|e| e.into_inner());

        guard
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.mutex.lock().unwrap_or_else(|e| e.into_inner())
    }
}
