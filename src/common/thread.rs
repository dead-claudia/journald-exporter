//! A wrapper around `std::thread::*` that doesn't leak. This is important in the face of process
//! termination and crashes, since threads don't join by default. (I've had numerous test failures
//! as well due to threads unexpectedly hanging, and I don't want to have too many active threads
//! during the tests.)

use crate::prelude::*;

use std::thread::JoinHandle;

pub struct ThreadHandle(Option<JoinHandle<io::Result<()>>>);

impl ThreadHandle {
    // Reduce the polymorphism involved here by accepting a boxed closure instead.
    pub fn spawn(init: impl FnOnce() -> io::Result<()> + Send + 'static) -> Self {
        Self(Some(std::thread::spawn(init)))
    }

    pub fn join(mut self) -> io::Result<()> {
        match self.0.take().unwrap().join() {
            Ok(result) => result,
            Err(e) => std::panic::resume_unwind(e),
        }
    }
}

impl Drop for ThreadHandle {
    fn drop(&mut self) {
        if let Some(inner) = self.0.take() {
            match inner.join() {
                Err(e) => std::panic::resume_unwind(e),
                // If the thread isn't already panicking, upgrade errors to panics.
                Ok(Err(e)) if !std::thread::panicking() => {
                    panic!(
                        "Uncaught error from joined thread: {}",
                        normalize_errno(e, None)
                    )
                }
                Ok(_) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Improves readability of assertions
    #![allow(clippy::bool_assert_comparison)]

    use super::*;

    #[test]
    fn join_returns_ok_on_ok() {
        static DONE_NOTIFY: Notify = Notify::new();
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static RESUME_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();

        let _start_guard = START_CHECKPOINT.drop_guard();

        let handle = ThreadHandle::spawn(Box::new(|| {
            let _resume_guard = RESUME_CHECKPOINT.drop_guard();
            let _done_guard = DONE_NOTIFY.create_guard();
            START_CHECKPOINT.resume();
            RESUME_CHECKPOINT.wait();
            Ok(())
        }));

        START_CHECKPOINT.wait();
        assert_eq!(DONE_NOTIFY.has_notified(), false);
        RESUME_CHECKPOINT.resume();

        assert_result_eq(handle.join(), Ok(()));
        assert_eq!(DONE_NOTIFY.has_notified(), true);
    }

    #[test]
    fn join_returns_err_on_err() {
        static DONE_NOTIFY: Notify = Notify::new();
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static RESUME_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();

        let _start_guard = START_CHECKPOINT.drop_guard();

        let handle = ThreadHandle::spawn(Box::new(|| {
            let _resume_guard = RESUME_CHECKPOINT.drop_guard();
            let _done_guard = DONE_NOTIFY.create_guard();
            START_CHECKPOINT.resume();
            if !RESUME_CHECKPOINT.try_wait() {
                return Ok(());
            }
            Err(Error::from_raw_os_error(libc::ENOENT))
        }));

        START_CHECKPOINT.wait();
        assert_eq!(DONE_NOTIFY.has_notified(), false);
        RESUME_CHECKPOINT.resume();

        assert_result_eq(handle.join(), Err(Error::from_raw_os_error(libc::ENOENT)));
        assert_eq!(DONE_NOTIFY.has_notified(), true);
    }

    #[test]
    #[should_panic = "test panic"]
    fn join_panics_on_panic() {
        static DONE_NOTIFY: Notify = Notify::new();
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static RESUME_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();

        let _start_guard = START_CHECKPOINT.drop_guard();

        // Catch the panic so I can assert the guard was still ultimately called.
        let handle_result = std::panic::catch_unwind(|| {
            let handle = ThreadHandle::spawn(Box::new(|| {
                let _resume_guard = RESUME_CHECKPOINT.drop_guard();
                let _done_guard = DONE_NOTIFY.create_guard();
                START_CHECKPOINT.resume();
                if !RESUME_CHECKPOINT.try_wait() {
                    return Ok(());
                }
                std::panic::panic_any("test panic");
            }));

            START_CHECKPOINT.wait();
            assert_eq!(DONE_NOTIFY.has_notified(), false);
            RESUME_CHECKPOINT.resume();

            handle.join()
        });

        assert_eq!(DONE_NOTIFY.has_notified(), true);
        std::panic::resume_unwind(
            handle_result.expect_err("Expected a panic, but received a result"),
        )
    }

    #[test]
    fn drop_returns_on_ok() {
        static DONE_NOTIFY: Notify = Notify::new();
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static RESUME_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();

        let _start_guard = START_CHECKPOINT.drop_guard();

        let handle = ThreadHandle::spawn(Box::new(|| {
            let _resume_guard = RESUME_CHECKPOINT.drop_guard();
            let _done_guard = DONE_NOTIFY.create_guard();
            START_CHECKPOINT.resume();
            RESUME_CHECKPOINT.wait();
            Ok(())
        }));

        START_CHECKPOINT.wait();
        assert_eq!(DONE_NOTIFY.has_notified(), false);
        RESUME_CHECKPOINT.resume();

        drop(handle);
        assert_eq!(DONE_NOTIFY.has_notified(), true);
    }

    #[test]
    #[should_panic = "Uncaught error from joined thread: ENOENT: No such file or directory"]
    fn drop_panics_on_err() {
        static DONE_NOTIFY: Notify = Notify::new();
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static RESUME_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();

        let _start_guard = START_CHECKPOINT.drop_guard();

        // Catch the panic so I can assert the guard was still ultimately called.
        let handle_result = std::panic::catch_unwind(|| {
            let handle = ThreadHandle::spawn(Box::new(|| {
                let _resume_guard = RESUME_CHECKPOINT.drop_guard();
                let _done_guard = DONE_NOTIFY.create_guard();
                START_CHECKPOINT.resume();
                if !RESUME_CHECKPOINT.try_wait() {
                    return Ok(());
                }
                Err(Error::from_raw_os_error(libc::ENOENT))
            }));

            START_CHECKPOINT.wait();
            assert_eq!(DONE_NOTIFY.has_notified(), false);
            RESUME_CHECKPOINT.resume();

            drop(handle)
        });

        assert_eq!(DONE_NOTIFY.has_notified(), true);
        std::panic::resume_unwind(
            handle_result.expect_err("Expected a panic, but received a result"),
        )
    }

    #[test]
    #[should_panic = "test panic"]
    fn drop_panics_on_panic() {
        static DONE_NOTIFY: Notify = Notify::new();
        static START_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();
        static RESUME_CHECKPOINT: ThreadCheckpoint = ThreadCheckpoint::new();

        let _start_guard = START_CHECKPOINT.drop_guard();

        // Catch the panic so I can assert the guard was still ultimately called.
        let handle_result = std::panic::catch_unwind(|| {
            let handle = ThreadHandle::spawn(Box::new(|| {
                let _resume_guard = RESUME_CHECKPOINT.drop_guard();
                let _done_guard = DONE_NOTIFY.create_guard();
                START_CHECKPOINT.resume();
                if !RESUME_CHECKPOINT.try_wait() {
                    return Ok(());
                }
                std::panic::panic_any("test panic");
            }));

            START_CHECKPOINT.wait();
            assert_eq!(DONE_NOTIFY.has_notified(), false);
            RESUME_CHECKPOINT.resume();

            handle.join()
        });

        assert_eq!(DONE_NOTIFY.has_notified(), true);
        std::panic::resume_unwind(
            handle_result.expect_err("Expected a panic, but received a result"),
        )
    }
}
