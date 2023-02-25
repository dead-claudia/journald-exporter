use crate::prelude::*;

use std::collections::VecDeque;

#[derive(Debug)]
struct CallSpyLocked<I, O> {
    args: Vec<I>,
    results: VecDeque<O>,
}

#[derive(Debug)]
pub struct CallSpy<I, O> {
    locked: Mutex<CallSpyLocked<I, O>>,
    pub(super) name: &'static str,
}

impl<I, O> CallSpy<I, O> {
    pub const fn new(name: &'static str) -> CallSpy<I, O> {
        CallSpy {
            locked: Mutex::new(CallSpyLocked {
                args: Vec::new(),
                results: VecDeque::new(),
            }),
            name,
        }
    }

    fn locked(&self) -> MutexGuard<CallSpyLocked<I, O>> {
        self.locked.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn enqueue(&self, result: O) {
        self.locked().results.push_back(result);
    }

    #[must_use]
    pub(super) fn try_call(&self, args: I) -> Option<O> {
        let mut guard = self.locked();
        if let Some(result) = guard.results.pop_front() {
            guard.args.push(args);
            return Some(result);
        }
        None
    }

    #[must_use]
    pub fn call(&self, args: I) -> O {
        match self.try_call(args) {
            Some(result) => result,
            None => panic!("No more `{}` calls expected.", self.name),
        }
    }

    #[track_caller]
    pub(super) fn assert_no_calls_remaining_inner(&self, calls: &str)
    where
        O: fmt::Debug,
    {
        let guard = self.locked();
        if !guard.results.is_empty() {
            panic!(
                "Unexpected {calls} remaining for `{}`: {:?}",
                self.name, guard.results
            );
        }
    }

    #[track_caller]
    pub fn assert_no_calls_remaining(&self)
    where
        O: fmt::Debug,
    {
        self.assert_no_calls_remaining_inner("calls");
    }

    #[track_caller]
    pub fn assert_calls(&self, expected: &[I])
    where
        I: fmt::Debug + PartialEq,
    {
        let guard = self.locked();
        if guard.args != expected {
            panic!(
                "Calls for `{}` do not match.\nExpected: {:?}\n  Actual: {:?}",
                self.name, expected, guard.args
            );
        }
    }
}

impl<I, O> CallSpy<I, io::Result<O>> {
    pub fn enqueue_ok(&self, result: O) {
        self.enqueue(Ok(result));
    }

    pub fn enqueue_err(&self, code: libc::c_int) {
        self.enqueue(Err(Error::from_raw_os_error(code)));
    }
}
