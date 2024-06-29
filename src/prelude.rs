// A nice, pretty prelude to cut down on the vast number of `use` statements commonly used.

pub use crate::common::*;
pub use crate::ffi::normalize_errno;
pub use crate::state::*;
pub use once_cell::sync::OnceCell;
pub use std::borrow::Cow;
pub use std::fmt;
pub use std::fmt::Write as _;
pub use std::io;
pub use std::io::BufRead;
pub use std::io::Error;
pub use std::io::ErrorKind;
pub use std::io::Read;
pub use std::io::Write;
pub use std::mem::replace;
pub use std::mem::take;
pub use std::mem::MaybeUninit;
pub use std::num::Wrapping;
pub use std::sync::atomic::*;
pub use std::sync::Arc;
pub use std::sync::Condvar;
pub use std::sync::Mutex;
pub use std::sync::MutexGuard;
pub use std::sync::Once;
pub use std::sync::RwLock;
pub use std::time::Duration;
pub use std::time::Instant;

#[cfg(test)]
pub use test_prelude::*;

#[cfg(test)]
mod test_prelude {
    pub use crate::test_utils::*;
    pub use quickcheck::Arbitrary;
    pub use quickcheck::Gen;
    pub use quickcheck_macros::quickcheck;
}
