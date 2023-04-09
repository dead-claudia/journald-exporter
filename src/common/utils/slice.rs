use crate::prelude::*;

// 1. It's a little less boilerplate than `target[..source.len()].copy_from_slice(source)`, which
//    in this app is a lot more common.
// 2. `.copy_from_slice` for some reason isn't forcibly inlined, even though one would think it
//    should be (since it'll end up delegating to `memcpy` if needed anyways). Resolving this here
//    doesn't seem to do much, though.
#[inline(always)]
#[track_caller]
pub fn copy_to_start(target: &mut [u8], source: &[u8]) {
    let source_len = source.len();
    let target_len = target.len();

    if source_len > target_len {
        len_mismatch_fail(target_len, source_len);
    }

    // SAFETY: `source` cannot point to `target` as mutable references are exclusive. `source` and
    // `target` by definition have `source.len()` and `target.len()` elements initialized, and
    // as per the above check, `target.len()` is no smaller than `source.len()`.
    unsafe {
        std::ptr::copy_nonoverlapping(source.as_ptr(), target.as_mut_ptr(), source_len);
    }
}

// The panic code path was put into a cold function to not bloat the call site. It also exists to
// align with Rust's behavior.
#[inline(never)]
#[cold]
#[track_caller]
fn len_mismatch_fail(target_len: usize, source_len: usize) -> ! {
    panic!(
        "source slice length ({}) exceeds destination slice length ({})",
        source_len, target_len,
    );
}

// Optimize for code size by merging everything into here. Also makes for some simpler code.
pub fn write_slices(result: &mut Vec<u8>, slices: &[&[u8]]) {
    let mut len = 0_usize;

    for slice in slices {
        len = len.wrapping_add(slice.len());
    }

    result.reserve(len);

    for slice in slices {
        result.extend_from_slice(slice)
    }
}

pub enum CowStr<'a> {
    Borrowed(&'a str),
    Owned(Box<str>),
}

impl<'a> CowStr<'a> {
    pub fn into_owned(self) -> Box<str> {
        match self {
            CowStr::Borrowed(s) => s.into(),
            CowStr::Owned(s) => s,
        }
    }

    pub fn format(args: std::fmt::Arguments<'a>) -> CowStr<'a> {
        match args.as_str() {
            Some(s) => CowStr::Borrowed(s),
            None => CowStr::Owned(args.to_string().into()),
        }
    }
}

impl std::ops::Deref for CowStr<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            CowStr::Borrowed(s) => s,
            CowStr::Owned(s) => s,
        }
    }
}

impl fmt::Debug for CowStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl fmt::Display for CowStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl PartialEq for CowStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
