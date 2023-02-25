pub fn parse_u32_digit(acc: u32, byte: u8) -> Option<u32> {
    match (acc.checked_mul(10), byte.checked_sub(b'0')) {
        (Some(next), Some(v @ 0..=9)) => next.checked_add(super::zero_extend_u8_u32(v)),
        _ => None,
    }
}

pub fn parse_u32(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        None
    } else {
        bytes.iter().copied().try_fold(0, parse_u32_digit)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    use super::*;

    #[quickcheck]
    fn parse_u32_digit_works(a: u32, b: u8) -> bool {
        match (a, b) {
            #[allow(clippy::identity_op)]
            (a @ 0..=429496728, b @ b'0') => parse_u32_digit(a, b) == Some(a * 10 + 0),
            (a @ 0..=429496728, b @ b'1') => parse_u32_digit(a, b) == Some(a * 10 + 1),
            (a @ 0..=429496728, b @ b'2') => parse_u32_digit(a, b) == Some(a * 10 + 2),
            (a @ 0..=429496728, b @ b'3') => parse_u32_digit(a, b) == Some(a * 10 + 3),
            (a @ 0..=429496728, b @ b'4') => parse_u32_digit(a, b) == Some(a * 10 + 4),
            (a @ 0..=429496728, b @ b'5') => parse_u32_digit(a, b) == Some(a * 10 + 5),
            (a @ 0..=429496728, b @ b'6') => parse_u32_digit(a, b) == Some(a * 10 + 6),
            (a @ 0..=429496728, b @ b'7') => parse_u32_digit(a, b) == Some(a * 10 + 7),
            (a @ 0..=429496728, b @ b'8') => parse_u32_digit(a, b) == Some(a * 10 + 8),
            (a @ 0..=429496728, b @ b'9') => parse_u32_digit(a, b) == Some(a * 10 + 9),
            (a @ 429496729, b @ b'0') => parse_u32_digit(a, b) == Some(4294967290),
            (a @ 429496729, b @ b'1') => parse_u32_digit(a, b) == Some(4294967291),
            (a @ 429496729, b @ b'2') => parse_u32_digit(a, b) == Some(4294967292),
            (a @ 429496729, b @ b'3') => parse_u32_digit(a, b) == Some(4294967293),
            (a @ 429496729, b @ b'4') => parse_u32_digit(a, b) == Some(4294967294),
            (a @ 429496729, b @ b'5') => parse_u32_digit(a, b) == Some(4294967295),
            (a, b) => parse_u32_digit(a, b).is_none(),
        }
    }

    // Test the special cases directly to ensure they're checked
    #[test]
    fn parse_u32_digit_special_case_works() {
        assert_eq!(parse_u32_digit(0, b'0'), Some(0));
        assert_eq!(parse_u32_digit(10, b'0'), Some(100));
        assert_eq!(parse_u32_digit(100, b'0'), Some(1000));
        assert_eq!(parse_u32_digit(1000, b'0'), Some(10000));
        assert_eq!(parse_u32_digit(10000, b'0'), Some(100000));
        assert_eq!(parse_u32_digit(100000, b'0'), Some(1000000));
        assert_eq!(parse_u32_digit(1000000, b'0'), Some(10000000));
        assert_eq!(parse_u32_digit(10000000, b'0'), Some(100000000));
        assert_eq!(parse_u32_digit(100000000, b'0'), Some(1000000000));
        assert_eq!(parse_u32_digit(0, b'1'), Some(1));
        assert_eq!(parse_u32_digit(10, b'1'), Some(101));
        assert_eq!(parse_u32_digit(100, b'1'), Some(1001));
        assert_eq!(parse_u32_digit(1000, b'1'), Some(10001));
        assert_eq!(parse_u32_digit(10000, b'1'), Some(100001));
        assert_eq!(parse_u32_digit(100000, b'1'), Some(1000001));
        assert_eq!(parse_u32_digit(1000000, b'1'), Some(10000001));
        assert_eq!(parse_u32_digit(10000000, b'1'), Some(100000001));
        assert_eq!(parse_u32_digit(100000000, b'1'), Some(1000000001));
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
        assert_eq!(parse_u32_digit(0, b'-'), None);
        assert_eq!(parse_u32_digit(10, b'-'), None);
        assert_eq!(parse_u32_digit(100, b'-'), None);
        assert_eq!(parse_u32_digit(1000, b'-'), None);
        assert_eq!(parse_u32_digit(10000, b'-'), None);
        assert_eq!(parse_u32_digit(100000, b'-'), None);
        assert_eq!(parse_u32_digit(1000000, b'-'), None);
        assert_eq!(parse_u32_digit(10000000, b'-'), None);
        assert_eq!(parse_u32_digit(100000000, b'-'), None);
        assert_eq!(parse_u32_digit(429496729, b'-'), None);
    }

    const fn d(v: u8) -> u32 {
        zero_extend_u8_u32(v - b'0')
    }

    #[test]
    fn d_helper_works() {
        assert_eq!(d(b'0'), 0);
        assert_eq!(d(b'1'), 1);
        assert_eq!(d(b'2'), 2);
        assert_eq!(d(b'3'), 3);
        assert_eq!(d(b'4'), 4);
        assert_eq!(d(b'5'), 5);
        assert_eq!(d(b'6'), 6);
        assert_eq!(d(b'7'), 7);
        assert_eq!(d(b'8'), 8);
        assert_eq!(d(b'9'), 9);
    }

    #[test]
    fn parse_u32_works_for_0_digits() {
        assert_eq!(parse_u32(b""), None);
    }

    #[quickcheck]
    fn parse_u32_works_for_1_digit(d1: u8) -> bool {
        match d1 {
            b'0'..=b'9' => parse_u32(&[d1]) == Some(d(d1)),
            _ => parse_u32(&[d1]).is_none(),
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_2_digits(digits: (u8, u8)) -> bool {
        match digits {
            (d2 @ b'0'..=b'9', d1 @ b'0'..=b'9') => {
                parse_u32(&[d2, d1]) == Some(d(d2) * 10 + d(d1))
            }
            (d2, d1) => parse_u32(&[d2, d1]).is_none(),
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_3_digits(digits: (u8, u8, u8)) -> bool {
        match digits {
            (d3 @ b'0'..=b'9', d2 @ b'0'..=b'9', d1 @ b'0'..=b'9') => {
                parse_u32(&[d3, d2, d1]) == Some(d(d3) * 100 + d(d2) * 10 + d(d1))
            }
            (d3, d2, d1) => parse_u32(&[d3, d2, d1]).is_none(),
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_4_digits(digits: (u8, u8, u8, u8)) -> bool {
        match digits {
            (d4 @ b'0'..=b'9', d3 @ b'0'..=b'9', d2 @ b'0'..=b'9', d1 @ b'0'..=b'9') => {
                parse_u32(&[d4, d3, d2, d1])
                    == Some(d(d4) * 1000 + d(d3) * 100 + d(d2) * 10 + d(d1))
            }
            (d4, d3, d2, d1) => parse_u32(&[d4, d3, d2, d1]).is_none(),
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_5_digits(digits: (u8, u8, u8, u8, u8)) -> bool {
        match digits {
            (
                d5 @ b'0'..=b'9',
                d4 @ b'0'..=b'9',
                d3 @ b'0'..=b'9',
                d2 @ b'0'..=b'9',
                d1 @ b'0'..=b'9',
            ) => {
                parse_u32(&[d5, d4, d3, d2, d1])
                    == Some(d(d5) * 10000 + d(d4) * 1000 + d(d3) * 100 + d(d2) * 10 + d(d1))
            }
            (d5, d4, d3, d2, d1) => parse_u32(&[d5, d4, d3, d2, d1]).is_none(),
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_6_digits(digits: (u8, u8, u8, u8, u8, u8)) -> bool {
        match digits {
            (
                d6 @ b'0'..=b'9',
                d5 @ b'0'..=b'9',
                d4 @ b'0'..=b'9',
                d3 @ b'0'..=b'9',
                d2 @ b'0'..=b'9',
                d1 @ b'0'..=b'9',
            ) => {
                parse_u32(&[d6, d5, d4, d3, d2, d1])
                    == Some(
                        d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (d6, d5, d4, d3, d2, d1) => parse_u32(&[d6, d5, d4, d3, d2, d1]).is_none(),
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_7_digits(digits: (u8, u8, u8, u8, u8, u8, u8)) -> bool {
        match digits {
            (
                d7 @ b'0'..=b'9',
                d6 @ b'0'..=b'9',
                d5 @ b'0'..=b'9',
                d4 @ b'0'..=b'9',
                d3 @ b'0'..=b'9',
                d2 @ b'0'..=b'9',
                d1 @ b'0'..=b'9',
            ) => {
                parse_u32(&[d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        d(d7) * 1000000
                            + d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (d7, d6, d5, d4, d3, d2, d1) => parse_u32(&[d7, d6, d5, d4, d3, d2, d1]).is_none(),
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_8_digits(digits: (u8, u8, u8, u8, u8, u8, u8, u8)) -> bool {
        match digits {
            (
                d8 @ b'0'..=b'9',
                d7 @ b'0'..=b'9',
                d6 @ b'0'..=b'9',
                d5 @ b'0'..=b'9',
                d4 @ b'0'..=b'9',
                d3 @ b'0'..=b'9',
                d2 @ b'0'..=b'9',
                d1 @ b'0'..=b'9',
            ) => {
                parse_u32(&[d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        d(d8) * 10000000
                            + d(d7) * 1000000
                            + d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (d8, d7, d6, d5, d4, d3, d2, d1) => {
                parse_u32(&[d8, d7, d6, d5, d4, d3, d2, d1]).is_none()
            }
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_9_digits(
        digits1: (u8, u8, u8, u8, u8, u8, u8, u8),
        digits2: (u8,),
    ) -> bool {
        match (
            digits1.0, digits1.1, digits1.2, digits1.3, digits1.4, digits1.5, digits1.6, digits1.7,
            digits2.0,
        ) {
            (
                d9 @ b'0'..=b'9',
                d8 @ b'0'..=b'9',
                d7 @ b'0'..=b'9',
                d6 @ b'0'..=b'9',
                d5 @ b'0'..=b'9',
                d4 @ b'0'..=b'9',
                d3 @ b'0'..=b'9',
                d2 @ b'0'..=b'9',
                d1 @ b'0'..=b'9',
            ) => {
                parse_u32(&[d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        d(d9) * 100000000
                            + d(d8) * 10000000
                            + d(d7) * 1000000
                            + d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (d9, d8, d7, d6, d5, d4, d3, d2, d1) => {
                parse_u32(&[d9, d8, d7, d6, d5, d4, d3, d2, d1]).is_none()
            }
        }
    }

    #[quickcheck]
    fn parse_u32_works_for_10_digits(
        digits1: (u8, u8, u8, u8, u8, u8, u8, u8),
        digits2: (u8, u8),
    ) -> bool {
        // Note: 2^32 = 4294967296
        let (d10, d9, d8, d7, d6, d5, d4, d3) = digits1;
        let (d2, d1) = digits2;
        match (d10, d9, d8, d7, d6, d5, d4, d3, d2, d1) {
            (
                b'0'..=b'3',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
            ) => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        d(d10) * 1000000000
                            + d(d9) * 100000000
                            + d(d8) * 10000000
                            + d(d7) * 1000000
                            + d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (
                b'4',
                b'0'..=b'1',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
            ) => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        4000000000
                            + d(d9) * 100000000
                            + d(d8) * 10000000
                            + d(d7) * 1000000
                            + d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (
                b'4',
                b'2',
                b'0'..=b'8',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
            ) => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        4200000000
                            + d(d8) * 10000000
                            + d(d7) * 1000000
                            + d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (
                b'4',
                b'2',
                b'9',
                b'0'..=b'3',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
            ) => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        4290000000
                            + d(d7) * 1000000
                            + d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (
                b'4',
                b'2',
                b'9',
                b'4',
                b'0'..=b'8',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
            ) => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        4294000000
                            + d(d6) * 100000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (
                b'4',
                b'2',
                b'9',
                b'4',
                b'9',
                b'0'..=b'5',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
            ) => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(
                        4294900000
                            + d(d5) * 10000
                            + d(d4) * 1000
                            + d(d3) * 100
                            + d(d2) * 10
                            + d(d1),
                    )
            }
            (
                b'4',
                b'2',
                b'9',
                b'4',
                b'9',
                b'6',
                b'0'..=b'6',
                b'0'..=b'9',
                b'0'..=b'9',
                b'0'..=b'9',
            ) => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(4294960000 + d(d4) * 1000 + d(d3) * 100 + d(d2) * 10 + d(d1))
            }
            (b'4', b'2', b'9', b'4', b'9', b'6', b'7', b'0'..=b'1', b'0'..=b'9', b'0'..=b'9') => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(4294967000 + d(d3) * 100 + d(d2) * 10 + d(d1))
            }
            (b'4', b'2', b'9', b'4', b'9', b'6', b'7', b'2', b'0'..=b'8', b'0'..=b'9') => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1])
                    == Some(4294967200 + d(d2) * 10 + d(d1))
            }
            (b'4', b'2', b'9', b'4', b'9', b'6', b'7', b'2', b'9', b'0'..=b'5') => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1]) == Some(4294967200 + d(d1))
            }
            (d10, d9, d8, d7, d6, d5, d4, d3, d2, d1) => {
                parse_u32(&[d10, d9, d8, d7, d6, d5, d4, d3, d2, d1]).is_none()
            }
        }
    }

    // Test the special cases directly to ensure they're checked

    #[test]
    fn parse_u32_works_for_unpadded_powers_of_10() {
        assert_eq!(parse_u32(b"10"), Some(10));
        assert_eq!(parse_u32(b"100"), Some(100));
        assert_eq!(parse_u32(b"1000"), Some(1000));
        assert_eq!(parse_u32(b"10000"), Some(10000));
        assert_eq!(parse_u32(b"100000"), Some(100000));
        assert_eq!(parse_u32(b"1000000"), Some(1000000));
        assert_eq!(parse_u32(b"10000000"), Some(10000000));
        assert_eq!(parse_u32(b"100000000"), Some(100000000));
        assert_eq!(parse_u32(b"1000000000"), Some(1000000000));
    }

    #[test]
    fn parse_u32_works_for_padded_powers_of_10() {
        assert_eq!(parse_u32(b"0000000001"), Some(1));
        assert_eq!(parse_u32(b"0000000010"), Some(10));
        assert_eq!(parse_u32(b"0000000100"), Some(100));
        assert_eq!(parse_u32(b"0000001000"), Some(1000));
        assert_eq!(parse_u32(b"0000010000"), Some(10000));
        assert_eq!(parse_u32(b"0000100000"), Some(100000));
        assert_eq!(parse_u32(b"0001000000"), Some(1000000));
        assert_eq!(parse_u32(b"0010000000"), Some(10000000));
        assert_eq!(parse_u32(b"0100000000"), Some(100000000));
        assert_eq!(parse_u32(b"1000000000"), Some(1000000000));
    }

    #[test]
    fn parse_u32_works_for_unpadded_powers_of_10_plus_1() {
        assert_eq!(parse_u32(b"11"), Some(11));
        assert_eq!(parse_u32(b"101"), Some(101));
        assert_eq!(parse_u32(b"1001"), Some(1001));
        assert_eq!(parse_u32(b"10001"), Some(10001));
        assert_eq!(parse_u32(b"100001"), Some(100001));
        assert_eq!(parse_u32(b"1000001"), Some(1000001));
        assert_eq!(parse_u32(b"10000001"), Some(10000001));
        assert_eq!(parse_u32(b"100000001"), Some(100000001));
        assert_eq!(parse_u32(b"1000000001"), Some(1000000001));
    }

    #[test]
    fn parse_u32_works_for_padded_powers_of_10_plus_1() {
        assert_eq!(parse_u32(b"0000000011"), Some(11));
        assert_eq!(parse_u32(b"0000000101"), Some(101));
        assert_eq!(parse_u32(b"0000001001"), Some(1001));
        assert_eq!(parse_u32(b"0000010001"), Some(10001));
        assert_eq!(parse_u32(b"0000100001"), Some(100001));
        assert_eq!(parse_u32(b"0001000001"), Some(1000001));
        assert_eq!(parse_u32(b"0010000001"), Some(10000001));
        assert_eq!(parse_u32(b"0100000001"), Some(100000001));
        assert_eq!(parse_u32(b"1000000001"), Some(1000000001));
    }

    #[test]
    fn parse_u32_works_near_u32_representation_limit() {
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
    }

    #[test]
    fn parse_u32_rejects_numbers_suffixed_with_hyphens() {
        assert_eq!(parse_u32(b"0-"), None);
        assert_eq!(parse_u32(b"10-"), None);
        assert_eq!(parse_u32(b"100-"), None);
        assert_eq!(parse_u32(b"1000-"), None);
        assert_eq!(parse_u32(b"10000-"), None);
        assert_eq!(parse_u32(b"100000-"), None);
        assert_eq!(parse_u32(b"1000000-"), None);
        assert_eq!(parse_u32(b"10000000-"), None);
        assert_eq!(parse_u32(b"100000000-"), None);
        assert_eq!(parse_u32(b"429496729-"), None);
        assert_eq!(parse_u32(b"4294967295-"), None);
    }

    #[test]
    fn parse_u32_rejects_numbers_prefixed_with_hyphens() {
        assert_eq!(parse_u32(b"-0"), None);
        assert_eq!(parse_u32(b"-10"), None);
        assert_eq!(parse_u32(b"-100"), None);
        assert_eq!(parse_u32(b"-1000"), None);
        assert_eq!(parse_u32(b"-10000"), None);
        assert_eq!(parse_u32(b"-100000"), None);
        assert_eq!(parse_u32(b"-1000000"), None);
        assert_eq!(parse_u32(b"-10000000"), None);
        assert_eq!(parse_u32(b"-100000000"), None);
        assert_eq!(parse_u32(b"-429496729"), None);
        assert_eq!(parse_u32(b"-4294967295"), None);
    }
}
