use crate::prelude::*;

/// Returns a value such that `result.to_ne_bytes()` returns `[hi, lo]`
pub fn to_hex_pair(byte: u8) -> [u8; 2] {
    fn to_hex(quad: u8) -> u8 {
        quad.wrapping_add(if quad < 10 { b'0' } else { b'A' - 10 })
    }

    // FIXME: Switch `wrapping_*` calls to literal operators where possible once
    // https://github.com/rust-lang/rust-clippy/pull/10309 is released.
    [to_hex(byte.wrapping_shr(4)), to_hex(byte & 0x0F)]
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryToDebug<'a>(pub &'a [u8]);

impl fmt::Debug for BinaryToDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        result.push('"');
        binary_to_display(&mut result, self.0);
        result.push('"');
        f.write_str(&result)
    }
}

pub fn binary_to_display(result: &mut String, buf: &[u8]) {
    for &byte in buf {
        match byte {
            b'\t' => result.push_str("\\t"),
            b'\r' => result.push_str("\\r"),
            b'\n' => result.push_str("\\n"),
            b'\\' => result.push_str("\\\\"),
            b'\'' => result.push_str("\\'"),
            b'"' => result.push_str("\\\""),
            b'\x20'..=b'\x7e' => result.push(byte.into()),
            _ => {
                let [hi, lo] = to_hex_pair(byte);
                let chars = [b'\\', b'x', hi, lo];
                // SAFETY: `chars` is pure ASCII.
                result.push_str(unsafe { std::str::from_utf8_unchecked(&chars) });
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct DebugBigSlice<'a>(pub &'a [u8]);

impl fmt::Debug for DebugBigSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const DEBUG_CUTOFF: usize = 120;
        // Allow a few more than the cutoff so I can just always pluralize it (and so I can
        // pretty-print when the number of bytes would in practice be smaller anyways).
        const DEBUG_THRESHOLD: usize = DEBUG_CUTOFF + 8;

        f.write_char('[')?;

        for (i, byte) in self.0.iter().cloned().enumerate() {
            if i > DEBUG_THRESHOLD {
                f.write_str(" ... ")?;
                fmt::Debug::fmt(&self.0.len().checked_sub(DEBUG_CUTOFF).unwrap(), f)?;
                f.write_str(" more bytes (")?;
                fmt::Debug::fmt(&self.0.len(), f)?;
                f.write_str(" total) ...]")?;
                return Ok(());
            }

            if i != 0 {
                f.write_char(' ')?;
            }

            let [hi, lo] = to_hex_pair(byte);
            f.write_char(hi.into())?;
            f.write_char(lo.into())?;
        }

        f.write_char(']')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //  #####  ####          #    # ###### #    #         #####    ##   # #####
    //    #   #    #         #    # #       #  #          #    #  #  #  # #    #
    //    #   #    #         ###### #####    ##           #    # #    # # #    #
    //    #   #    #         #    # #        ##           #####  ###### # #####
    //    #   #    #         #    # #       #  #          #      #    # # #   #
    //    #    ####          #    # ###### #    #         #      #    # # #    #
    //               #######                      #######

    #[test]
    fn to_hex_pair_works() {
        const VALUES: [(u8, u8); 16] = [
            (0x0, b'0'),
            (0x1, b'1'),
            (0x2, b'2'),
            (0x3, b'3'),
            (0x4, b'4'),
            (0x5, b'5'),
            (0x6, b'6'),
            (0x7, b'7'),
            (0x8, b'8'),
            (0x9, b'9'),
            (0xA, b'A'),
            (0xB, b'B'),
            (0xC, b'C'),
            (0xD, b'D'),
            (0xE, b'E'),
            (0xF, b'F'),
        ];

        for (i, hi) in VALUES {
            for (j, lo) in VALUES {
                let byte = i << 4 | j;
                let expected = [hi, lo];
                let actual = to_hex_pair(byte);

                assert_eq!(
                    to_hex_pair(byte),
                    expected,
                    "to_hex_pair(0x{:02X}) == {:?}",
                    byte,
                    actual.map(char::from),
                );
            }
        }
    }
}
