use crate::prelude::*;

use crate::ffi::Cursor;
use crate::ffi::Id128;
use std::ffi::CStr;

pub trait SystemdProvider {
    fn watchdog_notify(&'static self) -> io::Result<()>;
    fn boot_id(&'static self) -> Id128;
    fn get_monotonic_time_usec(&'static self) -> SystemdMonotonicUsec;
}

pub trait JournalRef
where
    Self: Sized,
{
    type Provider: SystemdProvider;
    fn open(provider: &'static Self::Provider) -> io::Result<Self>;
    fn set_data_threshold(&mut self, threshold: usize) -> io::Result<()>;
    fn seek_monotonic_usec(
        &mut self,
        boot_id: Id128,
        start_usec: SystemdMonotonicUsec,
    ) -> io::Result<()>;
    fn seek_cursor(&mut self, cursor: &Cursor) -> io::Result<()>;
    fn wait(&mut self, duration: Duration) -> io::Result<bool>;
    fn next(&mut self) -> io::Result<bool>;
    fn cursor(&mut self) -> io::Result<Cursor>;
    fn get_data<'a>(&'a mut self, field: &CStr) -> io::Result<&'a [u8]>;
}

// This is distinct from Rust's `Instant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemdMonotonicUsec(pub u64);
