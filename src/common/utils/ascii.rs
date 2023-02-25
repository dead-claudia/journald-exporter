use crate::prelude::*;

pub fn to_hex_pair(byte: u8) -> (u8, u8) {
    // FIXME: remove once https://github.com/rust-lang/rust-clippy/pull/10309 is released.
    #![allow(clippy::arithmetic_side_effects)]

    fn to_hex(quad: u8) -> u8 {
        quad.wrapping_add(if quad < 10 { b'0' } else { b'A' - 10 })
    }

    (to_hex(byte >> 4), to_hex(byte & 0x0F))
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryToDebug<'a>(pub &'a [u8]);

impl fmt::Debug for BinaryToDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        BinaryToDisplay(self.0).fmt(f)?;
        f.write_char('"')?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryToDisplay<'a>(pub &'a [u8]);

impl fmt::Debug for BinaryToDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0.iter().copied() {
            let prefix = match byte {
                b'\t' => 't',
                b'\r' => 'r',
                b'\n' => 'n',
                b'\\' => '\\',
                b'\'' => '\'',
                b'"' => '"',
                b'\x20'..=b'\x7e' => {
                    f.write_char(byte.into())?;
                    continue;
                }
                _ => {
                    let (hi, lo) = to_hex_pair(byte);
                    f.write_char('\\')?;
                    f.write_char('x')?;
                    f.write_char(hi.into())?;
                    f.write_char(lo.into())?;
                    continue;
                }
            };
            f.write_char('\\')?;
            f.write_char(prefix)?;
        }
        Ok(())
    }
}

impl fmt::Display for BinaryToDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

            let (hi, lo) = to_hex_pair(byte);
            f.write_char(hi.into())?;
            f.write_char(lo.into())?;
        }

        f.write_char(']')
    }
}

// Based on Rust's version, but changed to only what I actually need. This trims two characters:
// - Non-newline whitespace, as that's semantically irrelevant for the header (newlines can't
//   appear, due to HTTP header syntax)
// - Up to 2 trailing pad characters, depending on the length of `data`, as that's optional.
pub fn trim_auth_token(mut data: &[u8]) -> &[u8] {
    while let [b'\t' | b'\x0C' | b' ', rest @ ..] = data {
        data = rest;
    }
    while let [rest @ .., b'\t' | b'\x0C' | b' '] = data {
        data = rest;
    }

    match (data.len() % 4, data) {
        // NN==
        (0, [head @ .., b'=', b'=']) => head,
        // NNN=
        (0, [head @ .., b'=']) => head,
        // NN=
        (3, [head @ .., b'=']) => head,
        // Anything else
        _ => data,
    }
}

#[cfg(test)]
mod test {
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
                let expected = (hi, lo);
                let actual = to_hex_pair(byte);

                assert_eq!(
                    to_hex_pair(byte),
                    expected,
                    "to_hex_pair(0x{:02X}) == ({:?}, {:?})",
                    byte,
                    char::from(actual.0),
                    char::from(actual.1),
                );
            }
        }
    }
}
