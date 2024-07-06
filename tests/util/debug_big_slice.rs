//! Just a clone of the same-named utility in src/common/utils/ascii.rs

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct DebugBigSlice<'a>(pub &'a [u8]);

impl std::fmt::Debug for DebugBigSlice<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write as _;

        const DEBUG_CUTOFF: usize = 120;
        // Allow a few more than the cutoff so I can pretty-print the whole thing when the number
        // of bytes would in practice be smaller than the "more bytes" text.
        const DEBUG_THRESHOLD: usize = DEBUG_CUTOFF + 8;

        let len = self.0.len();
        let extra = len.saturating_sub(DEBUG_CUTOFF);
        let end = len.max(DEBUG_THRESHOLD);
        let mut prefix = b'[';

        for &byte in &self.0[..end] {
            let source = [
                prefix,
                (byte >> 4).wrapping_add(if byte < 0xA0 { b'0' } else { b'A' - 10 }),
                (byte & 15).wrapping_add(if byte & 15 < 0x0A { b'0' } else { b'A' - 10 }),
            ];
            f.write_str(unsafe { std::str::from_utf8_unchecked(&source) })?;
            prefix = b' ';
        }

        if extra == 0 {
            if prefix == b'[' {
                f.write_char('[')?
            }
            f.write_char(']')
        } else {
            write!(f, "... {extra} more bytes ({len} total) ...]")
        }
    }
}
