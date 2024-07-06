use crate::prelude::*;

// Hide incomplete numbers from `U32Parser`, and ensure it can only be used safely. This also
// ensures I can safely assume the tests here cover all possible external input to this module.
#[derive(Debug)]
pub struct U32Parser {
    acc: u32,
}

impl U32Parser {
    pub const fn new() -> Self {
        Self { acc: 0 }
    }

    pub const fn eat(self, byte: u8) -> Option<U32Parser> {
        // Carefully optimized for both code size and performance (as it impacts test speed a bit).
        // Per Godbolt, this ends up branch-free on x86-64.
        const D0: u32 = zero_extend_u8_u32(b'0');
        const MAX: u32 = u32::MAX / 10;
        let byte = zero_extend_u8_u32(byte);
        // Using `match` here results in a weird table lookup.
        // https://github.com/rust-lang/rust/issues/127384
        #[allow(clippy::comparison_chain)]
        let first_invalid = if self.acc < MAX {
            10
        } else if self.acc > MAX {
            0
        } else {
            6
        };
        let sub = if self.acc > MAX { byte } else { D0 };
        let v = byte.wrapping_sub(sub);
        let result = Self {
            acc: self.acc.wrapping_mul(10).wrapping_add(v),
        };

        if v < first_invalid {
            Some(result)
        } else {
            None
        }
    }

    pub const fn extract(self) -> u32 {
        self.acc
    }
}

pub const fn parse_u32(mut bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        None
    } else {
        let mut parser = U32Parser::new();

        while let &[byte, ref rest @ ..] = bytes {
            bytes = rest;
            match parser.eat(byte) {
                Some(next) => parser = next,
                None => return None,
            }
        }

        Some(parser.extract())
    }
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use super::*;

    #[test]
    fn parse_u32_works_for_special_cases() {
        assert_eq!(parse_u32(b""), None);
        assert_eq!(parse_u32(b"4294967290"), Some(4294967290));
        assert_eq!(parse_u32(b"4294967291"), Some(4294967291));
        assert_eq!(parse_u32(b"4294967292"), Some(4294967292));
        assert_eq!(parse_u32(b"4294967293"), Some(4294967293));
        assert_eq!(parse_u32(b"4294967294"), Some(4294967294));
        assert_eq!(parse_u32(b"4294967295"), Some(4294967295));
        assert_eq!(parse_u32(b"4294967296"), None);
        assert_eq!(parse_u32(b"4294967297"), None);
        assert_eq!(parse_u32(b"4294967298"), None);
        assert_eq!(parse_u32(b"4294967299"), None);
        assert_eq!(parse_u32(b"4294967300"), None);
        assert_eq!(parse_u32(b"9999999999"), None);
        assert_eq!(parse_u32(b"04294967290"), Some(4294967290));
        assert_eq!(parse_u32(b"04294967291"), Some(4294967291));
        assert_eq!(parse_u32(b"04294967292"), Some(4294967292));
        assert_eq!(parse_u32(b"04294967293"), Some(4294967293));
        assert_eq!(parse_u32(b"04294967294"), Some(4294967294));
        assert_eq!(parse_u32(b"04294967295"), Some(4294967295));
        assert_eq!(parse_u32(b"04294967296"), None);
        assert_eq!(parse_u32(b"04294967297"), None);
        assert_eq!(parse_u32(b"04294967298"), None);
        assert_eq!(parse_u32(b"04294967299"), None);
        assert_eq!(parse_u32(b"04294967300"), None);
        assert_eq!(parse_u32(b"09999999999"), None);
        assert_eq!(parse_u32(b"10000000000"), None);
        assert_eq!(parse_u32(b"20000000000"), None);
        assert_eq!(parse_u32(b"30000000000"), None);
        assert_eq!(parse_u32(b"40000000000"), None);
        assert_eq!(parse_u32(b"42949672950"), None);
    }

    #[test]
    fn parse_u32_works_for_1_digit() {
        let mut expected_table = [u32::MAX; 1 << 8];
        expected_table[zero_extend_u8_usize(b'0')] = 0;
        expected_table[zero_extend_u8_usize(b'1')] = 1;
        expected_table[zero_extend_u8_usize(b'2')] = 2;
        expected_table[zero_extend_u8_usize(b'3')] = 3;
        expected_table[zero_extend_u8_usize(b'4')] = 4;
        expected_table[zero_extend_u8_usize(b'5')] = 5;
        expected_table[zero_extend_u8_usize(b'6')] = 6;
        expected_table[zero_extend_u8_usize(b'7')] = 7;
        expected_table[zero_extend_u8_usize(b'8')] = 8;
        expected_table[zero_extend_u8_usize(b'9')] = 9;

        for (i, expected) in expected_table.into_iter().enumerate() {
            let expected = (expected != u32::MAX).then_some(expected);
            assert_eq!(parse_u32(&[truncate_usize_u8(i)]), expected, "{i:02x}");
        }
    }

    fn has_only_digits(i: u32) -> bool {
        ((i & 0xF0F0F0F0) == 0x30303030) & ((i.wrapping_add(0x76767676) & 0x10101010) == 0)
    }

    #[test]
    fn has_only_digits_is_correct() {
        for i in 0..=u8::MAX {
            assert_eq!(
                has_only_digits(u32::from_ne_bytes([i, i, i, i])),
                i.is_ascii_digit(),
                "{i:02x}"
            );
            assert_eq!(
                has_only_digits(u32::from_le_bytes([0x30, 0x30, 0x30, i])),
                i.is_ascii_digit(),
                "{i:02x}"
            );
            assert_eq!(
                has_only_digits(u32::from_le_bytes([0x30, 0x30, i, 0x30])),
                i.is_ascii_digit(),
                "{i:02x}"
            );
            assert_eq!(
                has_only_digits(u32::from_le_bytes([0x30, i, 0x30, 0x30])),
                i.is_ascii_digit(),
                "{i:02x}"
            );
            assert_eq!(
                has_only_digits(u32::from_le_bytes([i, 0x30, 0x30, 0x30])),
                i.is_ascii_digit(),
                "{i:02x}"
            );
        }
    }

    #[test]
    fn parse_u32_works_for_2_digits() {
        let mut expected_value = 0;
        for i in 0..=0xFFFF {
            let expected = has_only_digits(i | 0x30300000).then_some(expected_value);
            expected_value += u32::from(expected.is_some());
            let [.., d0, d1] = i.to_be_bytes();
            assert_eq!(parse_u32(&[d0, d1]), expected, "{i:04x}");
        }
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_works_for_3_digits() {
        let mut expected_value = 0;
        for i in 0..=0xFFFFFF {
            let expected = has_only_digits(i | 0x30000000).then_some(expected_value);
            expected_value += u32::from(expected.is_some());
            let [.., d0, d1, d2] = i.to_be_bytes();
            assert_eq!(parse_u32(&[d0, d1, d2]), expected, "{i:06x}");
        }
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_works_for_4_digits() {
        let mut expected_value = 0;
        for i in 0..=u32::MAX {
            let expected = has_only_digits(i).then_some(expected_value);
            expected_value += u32::from(expected.is_some());
            assert_eq!(parse_u32(&i.to_be_bytes()), expected, "{i:08x}");
        }
    }

    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit(acc: u32, byte: u8) -> Option<u32> {
        Some(U32Parser { acc }.eat(byte)?.extract())
    }

    // Too slow for Miri
    #[cfg(not(miri))]
    fn test_invalid_chunk(start: u32, end: u32) {
        parallel_for(start..=end, |acc| {
            for byte in 0..=u8::MAX {
                assert_eq!(parse_u32_digit(acc, byte), None, "{acc}, {byte}");
            }
        });
    }

    // Too slow for Miri
    #[cfg(not(miri))]
    fn test_valid_chunk(start: u32, end: u32) {
        parallel_for(start..=end, |acc| {
            for byte in 0..b'0' {
                assert_eq!(parse_u32_digit(acc, byte), None, "{acc}, {byte}");
            }

            let acc_10 = acc * 10;
            assert_eq!(parse_u32_digit(acc, b'0'), Some(acc_10), "{acc}, b'0'");
            assert_eq!(parse_u32_digit(acc, b'1'), Some(acc_10 + 1), "{acc}, b'1'");
            assert_eq!(parse_u32_digit(acc, b'2'), Some(acc_10 + 2), "{acc}, b'2'");
            assert_eq!(parse_u32_digit(acc, b'3'), Some(acc_10 + 3), "{acc}, b'3'");
            assert_eq!(parse_u32_digit(acc, b'4'), Some(acc_10 + 4), "{acc}, b'4'");
            assert_eq!(parse_u32_digit(acc, b'5'), Some(acc_10 + 5), "{acc}, b'5'");
            assert_eq!(parse_u32_digit(acc, b'6'), Some(acc_10 + 6), "{acc}, b'6'");
            assert_eq!(parse_u32_digit(acc, b'7'), Some(acc_10 + 7), "{acc}, b'7'");
            assert_eq!(parse_u32_digit(acc, b'8'), Some(acc_10 + 8), "{acc}, b'8'");
            assert_eq!(parse_u32_digit(acc, b'9'), Some(acc_10 + 9), "{acc}, b'9'");

            for byte in (b'9' + 1)..=u8::MAX {
                assert_eq!(parse_u32_digit(acc, byte), None, "{acc}, {byte}");
            }
        });
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_0xxxxxxxx_works() {
        test_valid_chunk(0, 99999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_1xxxxxxxx_works() {
        test_valid_chunk(100000000, 199999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_2xxxxxxxx_works() {
        test_valid_chunk(200000000, 299999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_3xxxxxxxx_works() {
        test_valid_chunk(300000000, 399999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_4xxxxxxxx_works() {
        test_valid_chunk(400000000, 429496728);

        assert_eq!(parse_u32_digit(429496729, b'0'), Some(4294967290));
        assert_eq!(parse_u32_digit(429496729, b'1'), Some(4294967291));
        assert_eq!(parse_u32_digit(429496729, b'2'), Some(4294967292));
        assert_eq!(parse_u32_digit(429496729, b'3'), Some(4294967293));
        assert_eq!(parse_u32_digit(429496729, b'4'), Some(4294967294));
        assert_eq!(parse_u32_digit(429496729, b'5'), Some(4294967295));
        assert_eq!(parse_u32_digit(429496729, b'6'), None);
        assert_eq!(parse_u32_digit(429496729, b'7'), None);
        assert_eq!(parse_u32_digit(429496729, b'8'), None);
        assert_eq!(parse_u32_digit(429496729, b'9'), None);

        test_invalid_chunk(429496730, 499999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_5xxxxxxxx_works() {
        test_invalid_chunk(500000000, 599999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_6xxxxxxxx_works() {
        test_invalid_chunk(600000000, 699999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_7xxxxxxxx_works() {
        test_invalid_chunk(700000000, 799999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_8xxxxxxxx_works() {
        test_invalid_chunk(800000000, 899999999);
    }

    #[test]
    // Too slow for Miri
    #[cfg(not(miri))]
    fn parse_u32_digit_9xxxxxxxx_works() {
        test_invalid_chunk(900000000, 999999999);
    }
}
