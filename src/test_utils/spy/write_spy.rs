use crate::prelude::*;

use crate::ffi::ImmutableWrite;
use crate::ffi::PollFlags;
use crate::ffi::PollResult;
use crate::ffi::Pollable;

#[derive(Debug)]
pub struct WriteSpy {
    data: Mutex<Vec<u8>>,
    inner: CallSpy<(), io::Result<usize>>,
}

impl WriteSpy {
    pub const fn new(name: &'static str) -> WriteSpy {
        WriteSpy {
            data: Mutex::new(Vec::new()),
            inner: CallSpy::new(name),
        }
    }

    pub fn get_data_written(&self) -> Vec<u8> {
        self.data.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn reset_data_written(&self) {
        self.data.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }

    /// Note: the result is merely the maximum allowed to be returned. The actual result may be
    /// lower based on the buffer sent to be written with.
    pub fn enqueue_write_ok(&self, result: usize) {
        self.inner.enqueue_ok(result);
    }

    pub fn enqueue_write_err(&self, code: libc::c_int) {
        self.inner.enqueue_err(code);
    }

    #[track_caller]
    pub fn assert_data_written(&self, expected: &[u8]) {
        let guard = self.data.lock().unwrap_or_else(|e| e.into_inner());
        if *guard != expected {
            panic!(
                "Calls for `{}` do not match.\n  Actual: {:?}\nExpected: {:?}",
                self.inner.name,
                DebugBigSlice(&guard),
                DebugBigSlice(expected)
            );
        }
    }

    #[track_caller]
    pub fn assert_data_str_written(&self, expected: &[u8]) {
        let guard = self.data.lock().unwrap_or_else(|e| e.into_inner());
        if *guard != expected {
            panic!(
                "Calls for `{}` do not match.\n  Actual: {:?}\nExpected: {:?}",
                self.inner.name,
                BinaryToDebug(&guard),
                BinaryToDebug(expected)
            );
        }
    }

    #[track_caller]
    pub fn assert_no_calls_remaining(&self) {
        self.inner.assert_no_calls_remaining_inner("`write` calls");
    }
}

impl io::Write for &WriteSpy {
    #[track_caller]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut data = self.data.lock().unwrap_or_else(|e| e.into_inner());

        match self.inner.try_call(()) {
            None => panic!(
                "No more `write` calls expected for `{}`, tried to write {:?}.",
                self.inner.name,
                BinaryToDebug(buf),
            ),
            Some(Err(e)) => Err(e),
            Some(Ok(len)) if len == buf.len() => {
                data.extend_from_slice(buf);
                Ok(len)
            }
            Some(Ok(len)) => panic!(
                "Write length mismatch for `{}`. Expected {} bytes, found {}-byte source {:?}.",
                self.inner.name,
                len,
                buf.len(),
                BinaryToDebug(buf),
            ),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    #[track_caller]
    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        let mut joined = Vec::new();
        for buf in bufs {
            joined.extend_from_slice(buf);
        }
        io::Write::write(self, &joined)
    }
}

impl Pollable for &WriteSpy {
    fn poll(&self, _: PollFlags, _: Option<Duration>) -> io::Result<PollResult> {
        Ok(PollResult::OUT)
    }
}

impl ImmutableWrite for &WriteSpy {
    type Inner<'a> = &'a WriteSpy where Self: 'a;
    fn inner(&self) -> Self::Inner<'_> {
        self
    }
}
