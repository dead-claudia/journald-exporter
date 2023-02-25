use crate::prelude::*;

enum State {
    Init,
    Dropped,
    Ready,
}

pub struct ThreadCheckpoint {
    inner: Checkpoint<State>,
}

pub struct ThreadCheckpointDropGuard<'a>(&'a ThreadCheckpoint);

impl Drop for ThreadCheckpointDropGuard<'_> {
    fn drop(&mut self) {
        self.0.inner.notify(|state| *state = State::Dropped);
    }
}

impl ThreadCheckpoint {
    pub const fn new() -> Self {
        Self {
            inner: Checkpoint::new(State::Init),
        }
    }

    pub fn drop_guard(&self) -> ThreadCheckpointDropGuard {
        ThreadCheckpointDropGuard(self)
    }

    pub fn resume(&self) {
        self.inner.notify(|state| {
            if matches!(state, State::Init) {
                *state = State::Ready;
            }
        });
    }

    /// Returns `false` if this resumed due to abort.
    pub fn try_wait(&self) -> bool {
        let mut guard = self.inner.lock();
        loop {
            match &*guard {
                State::Init => guard = self.inner.resume_wait(guard),
                State::Dropped => return false,
                State::Ready => return true,
            }
        }
    }

    /// Returns `false` if this resumed due to abort.
    pub fn wait(&self) {
        if !self.try_wait() {
            panic!("Wait failed due to checkpoint dropped.");
        }
    }
}
