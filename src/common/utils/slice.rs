use crate::prelude::*;

// Optimize for code size by merging everything into here. Also makes for some simpler code.
pub fn write_slices(result: &mut Vec<u8>, slices: &[&[u8]]) -> bool {
    let len: usize = slices.iter().map(|s| s.len()).sum();

    if result.try_reserve(len).is_err() {
        false
    } else {
        for slice in slices {
            result.extend_from_slice(slice);
        }
        true
    }
}

pub fn format_cow(args: std::fmt::Arguments) -> Cow<str> {
    match args.as_str() {
        Some(s) => Cow::Borrowed(s),
        None => Cow::Owned(args.to_string()),
    }
}
