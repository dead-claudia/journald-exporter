use crate::prelude::*;

use crate::ffi::PollFlags;
use crate::ffi::PollResult;
use crate::ffi::Pollable;

struct SpyFn {
    f: Box<dyn FnOnce() + Send>,
}

impl fmt::Debug for SpyFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpyFn").finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct ReadSpy {
    inner: CallSpy<(), (io::Result<&'static [u8]>, Option<SpyFn>)>,
}

impl ReadSpy {
    pub const fn new(name: &'static str) -> ReadSpy {
        ReadSpy {
            inner: CallSpy::new(name),
        }
    }

    pub fn enqueue_read_ok_spy(&self, result: &'static [u8], spy: Box<dyn FnOnce() + Send>) {
        self.inner.enqueue((Ok(result), Some(SpyFn { f: spy })));
    }

    pub fn enqueue_read(&self, result: Result<&'static [u8], libc::c_int>) {
        self.inner
            .enqueue((result.map_err(Error::from_raw_os_error), None));
    }

    pub fn enqueue_read_err(&self, code: libc::c_int) {
        self.enqueue_read(Err(code));
    }

    #[track_caller]
    pub fn assert_no_calls_remaining(&self) {
        self.inner.assert_no_calls_remaining_inner("`read` calls");
    }
}

impl Read for &ReadSpy {
    #[track_caller]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some((result, opt_f)) = self.inner.try_call(()) {
            if let Some(spy_fn) = opt_f {
                (spy_fn.f)();
            }
            let data = result?;
            let len = buf.len().min(data.len());
            buf[..len].copy_from_slice(&data[..len]);
            return Ok(len);
        }

        panic!("No more `read` calls expected for `{}`.", self.inner.name)
    }
}

impl Pollable for &ReadSpy {
    fn poll(&self, _: PollFlags, _: Option<Duration>) -> io::Result<PollResult> {
        Ok(PollResult::IN)
    }
}
