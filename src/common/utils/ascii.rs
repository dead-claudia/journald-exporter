use crate::prelude::*;

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
    #[repr(align(4))]
    struct Lut([u8; 1024]);

    static LUT: Lut = Lut(*b"\
\\x00\\x01\\x02\\x03\\x04\\x05\\x06\\x07\\x08\\t\0\0\\n\0\0\\x0B\\x0C\\r\0\0\\x0E\\x0F\
\\x10\\x11\\x12\\x13\\x14\\x15\\x16\\x17\\x18\\x19\\x1A\\x1B\\x1C\\x1D\\x1E\\x1F\
\x20\0\0\0\x21\0\0\0\\\"\0\0\x23\0\0\0\x24\0\0\0\x25\0\0\0\x26\0\0\0\\'\0\0\
\x28\0\0\0\x29\0\0\0\x2A\0\0\0\x2B\0\0\0\x2C\0\0\0\x2D\0\0\0\x2E\0\0\0\x2F\0\0\0\
\x30\0\0\0\x31\0\0\0\x32\0\0\0\x33\0\0\0\x34\0\0\0\x35\0\0\0\x36\0\0\0\x37\0\0\0\
\x38\0\0\0\x39\0\0\0\x3A\0\0\0\x3B\0\0\0\x3C\0\0\0\x3D\0\0\0\x3E\0\0\0\x3F\0\0\0\
\x40\0\0\0\x41\0\0\0\x42\0\0\0\x43\0\0\0\x44\0\0\0\x45\0\0\0\x46\0\0\0\x47\0\0\0\
\x48\0\0\0\x49\0\0\0\x4A\0\0\0\x4B\0\0\0\x4C\0\0\0\x4D\0\0\0\x4E\0\0\0\x4F\0\0\0\
\x50\0\0\0\x51\0\0\0\x52\0\0\0\x53\0\0\0\x54\0\0\0\x55\0\0\0\x56\0\0\0\x57\0\0\0\
\x58\0\0\0\x59\0\0\0\x5A\0\0\0\x5B\0\0\0\\\\\0\0\x5D\0\0\0\x5E\0\0\0\x5F\0\0\0\
\x60\0\0\0\x61\0\0\0\x62\0\0\0\x63\0\0\0\x64\0\0\0\x65\0\0\0\x66\0\0\0\x67\0\0\0\
\x68\0\0\0\x69\0\0\0\x6A\0\0\0\x6B\0\0\0\x6C\0\0\0\x6D\0\0\0\x6E\0\0\0\x6F\0\0\0\
\x70\0\0\0\x71\0\0\0\x72\0\0\0\x73\0\0\0\x74\0\0\0\x75\0\0\0\x76\0\0\0\x77\0\0\0\
\x78\0\0\0\x79\0\0\0\x7A\0\0\0\x7B\0\0\0\x7C\0\0\0\x7D\0\0\0\x7E\0\0\0\\x7F\
\\x80\\x81\\x82\\x83\\x84\\x85\\x86\\x87\\x88\\x89\\x8A\\x8B\\x8C\\x8D\\x8E\\x8F\
\\x90\\x91\\x92\\x93\\x94\\x95\\x96\\x97\\x98\\x99\\x9A\\x9B\\x9C\\x9D\\x9E\\x9F\
\\xA0\\xA1\\xA2\\xA3\\xA4\\xA5\\xA6\\xA7\\xA8\\xA9\\xAA\\xAB\\xAC\\xAD\\xAE\\xAF\
\\xB0\\xB1\\xB2\\xB3\\xB4\\xB5\\xB6\\xB7\\xB8\\xB9\\xBA\\xBB\\xBC\\xBD\\xBE\\xBF\
\\xC0\\xC1\\xC2\\xC3\\xC4\\xC5\\xC6\\xC7\\xC8\\xC9\\xCA\\xCB\\xCC\\xCD\\xCE\\xCF\
\\xD0\\xD1\\xD2\\xD3\\xD4\\xD5\\xD6\\xD7\\xD8\\xD9\\xDA\\xDB\\xDC\\xDD\\xDE\\xDF\
\\xE0\\xE1\\xE2\\xE3\\xE4\\xE5\\xE6\\xE7\\xE8\\xE9\\xEA\\xEB\\xEC\\xED\\xEE\\xEF\
\\xF0\\xF1\\xF2\\xF3\\xF4\\xF5\\xF6\\xF7\\xF8\\xF9\\xFA\\xFB\\xFC\\xFD\\xFE\\xFF");

    let std::ops::Range { mut start, end } = buf.as_ptr_range();

    // SAFETY: The length is computed from the LUT entry. Underflow is impossible as the LUT
    // never has a zero 32-bit word. And every entry is valid ASCII, even if it's not fully
    // used.
    //
    // As for other reads, it's from `buf`, and I iterate a pointer range. And `result` is only
    // pushed valid ASCII as per the above.
    unsafe {
        let result = result.as_mut_vec();

        while start != end {
            let base = LUT.0.as_ptr().cast::<u32>();

            let addr = base.add(usize::from(*start));
            start = start.add(1);

            #[cfg(target_endian = "little")]
            let len = 4_u32.wrapping_sub(addr.read_unaligned().leading_zeros() >> 3);
            #[cfg(target_endian = "big")]
            let len = 4_u32.wrapping_sub(addr.read_unaligned().trailing_zeros() >> 3);

            #[allow(clippy::as_conversions)]
            let chunk = std::slice::from_raw_parts(addr.cast(), len as usize);

            result.extend_from_slice(chunk);
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct DebugBigSlice<'a>(pub &'a [u8]);

impl fmt::Debug for DebugBigSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Write as _;

        const DEBUG_CUTOFF: usize = 120;
        // Allow a few more than the cutoff so I can pretty-print the whole thing when the number
        // of bytes would in practice be smaller than the "more bytes" text.
        const DEBUG_THRESHOLD: usize = DEBUG_CUTOFF + 8;

        let len = self.0.len();
        let mut end = len;
        let mut extra = 0;
        let mut prefix = b'[';

        if len > DEBUG_THRESHOLD {
            end = DEBUG_CUTOFF;
            extra = len.wrapping_sub(end);
        }

        for &byte in &self.0[..end] {
            let source = [
                prefix,
                (byte >> 4).wrapping_add(if byte < 0xA0 { b'0' } else { b'A' - 10 }),
                (byte & 15).wrapping_add(if byte & 15 < 0x0A { b'0' } else { b'A' - 10 }),
            ];
            // SAFETY: it's pure ASCII
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

// Based on Rust's version, but works on byte slices instead.
// TODO: remove once https://github.com/rust-lang/rust/issues/94035 stabilizes.
pub const fn trim_ascii(mut data: &[u8]) -> &[u8] {
    while let [b'\x09' | b'\x0A' | b'\x0C' | b'\x0D' | b' ', rest @ ..] = data {
        data = rest;
    }

    while let [rest @ .., b'\x09' | b'\x0A' | b'\x0C' | b'\x0D' | b' '] = data {
        data = rest;
    }

    data
}

// Like above, but for the `Authorization` field for headers. This trims two characters:
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
mod tests {
    use super::*;

    //  ######                              ######            #####
    //  #     # ###### #####  #    #  ####  #     # #  ####  #     # #      #  ####  ######
    //  #     # #      #    # #    # #    # #     # # #    # #       #      # #    # #
    //  #     # #####  #####  #    # #      ######  # #       #####  #      # #      #####
    //  #     # #      #    # #    # #  ### #     # # #  ###       # #      # #      #
    //  #     # #      #    # #    # #    # #     # # #    # #     # #      # #    # #
    //  ######  ###### #####   ####   ####  ######  #  ####   #####  ###### #  ####  ######

    #[test]
    fn debug_big_slice_works_for_empty() {
        assert_eq!(format!("{:?}", DebugBigSlice(&[])), "[]");
    }

    #[test]
    fn debug_big_slice_works_for_one_byte() {
        static VALUES: [&str; 256] = [
            "[00]", "[01]", "[02]", "[03]", "[04]", "[05]", "[06]", "[07]", "[08]", "[09]", "[0A]",
            "[0B]", "[0C]", "[0D]", "[0E]", "[0F]", "[10]", "[11]", "[12]", "[13]", "[14]", "[15]",
            "[16]", "[17]", "[18]", "[19]", "[1A]", "[1B]", "[1C]", "[1D]", "[1E]", "[1F]", "[20]",
            "[21]", "[22]", "[23]", "[24]", "[25]", "[26]", "[27]", "[28]", "[29]", "[2A]", "[2B]",
            "[2C]", "[2D]", "[2E]", "[2F]", "[30]", "[31]", "[32]", "[33]", "[34]", "[35]", "[36]",
            "[37]", "[38]", "[39]", "[3A]", "[3B]", "[3C]", "[3D]", "[3E]", "[3F]", "[40]", "[41]",
            "[42]", "[43]", "[44]", "[45]", "[46]", "[47]", "[48]", "[49]", "[4A]", "[4B]", "[4C]",
            "[4D]", "[4E]", "[4F]", "[50]", "[51]", "[52]", "[53]", "[54]", "[55]", "[56]", "[57]",
            "[58]", "[59]", "[5A]", "[5B]", "[5C]", "[5D]", "[5E]", "[5F]", "[60]", "[61]", "[62]",
            "[63]", "[64]", "[65]", "[66]", "[67]", "[68]", "[69]", "[6A]", "[6B]", "[6C]", "[6D]",
            "[6E]", "[6F]", "[70]", "[71]", "[72]", "[73]", "[74]", "[75]", "[76]", "[77]", "[78]",
            "[79]", "[7A]", "[7B]", "[7C]", "[7D]", "[7E]", "[7F]", "[80]", "[81]", "[82]", "[83]",
            "[84]", "[85]", "[86]", "[87]", "[88]", "[89]", "[8A]", "[8B]", "[8C]", "[8D]", "[8E]",
            "[8F]", "[90]", "[91]", "[92]", "[93]", "[94]", "[95]", "[96]", "[97]", "[98]", "[99]",
            "[9A]", "[9B]", "[9C]", "[9D]", "[9E]", "[9F]", "[A0]", "[A1]", "[A2]", "[A3]", "[A4]",
            "[A5]", "[A6]", "[A7]", "[A8]", "[A9]", "[AA]", "[AB]", "[AC]", "[AD]", "[AE]", "[AF]",
            "[B0]", "[B1]", "[B2]", "[B3]", "[B4]", "[B5]", "[B6]", "[B7]", "[B8]", "[B9]", "[BA]",
            "[BB]", "[BC]", "[BD]", "[BE]", "[BF]", "[C0]", "[C1]", "[C2]", "[C3]", "[C4]", "[C5]",
            "[C6]", "[C7]", "[C8]", "[C9]", "[CA]", "[CB]", "[CC]", "[CD]", "[CE]", "[CF]", "[D0]",
            "[D1]", "[D2]", "[D3]", "[D4]", "[D5]", "[D6]", "[D7]", "[D8]", "[D9]", "[DA]", "[DB]",
            "[DC]", "[DD]", "[DE]", "[DF]", "[E0]", "[E1]", "[E2]", "[E3]", "[E4]", "[E5]", "[E6]",
            "[E7]", "[E8]", "[E9]", "[EA]", "[EB]", "[EC]", "[ED]", "[EE]", "[EF]", "[F0]", "[F1]",
            "[F2]", "[F3]", "[F4]", "[F5]", "[F6]", "[F7]", "[F8]", "[F9]", "[FA]", "[FB]", "[FC]",
            "[FD]", "[FE]", "[FF]",
        ];

        for byte in 0..=255 {
            assert_eq!(
                format!("{:?}", DebugBigSlice(&[byte])),
                VALUES[usize::from(byte)]
            );
        }
    }

    #[test]
    fn debug_big_slice_works_for_two_bytes() {
        static PAIRS: [&str; 256] = [
            "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "0A", "0B", "0C", "0D",
            "0E", "0F", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1A", "1B",
            "1C", "1D", "1E", "1F", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29",
            "2A", "2B", "2C", "2D", "2E", "2F", "30", "31", "32", "33", "34", "35", "36", "37",
            "38", "39", "3A", "3B", "3C", "3D", "3E", "3F", "40", "41", "42", "43", "44", "45",
            "46", "47", "48", "49", "4A", "4B", "4C", "4D", "4E", "4F", "50", "51", "52", "53",
            "54", "55", "56", "57", "58", "59", "5A", "5B", "5C", "5D", "5E", "5F", "60", "61",
            "62", "63", "64", "65", "66", "67", "68", "69", "6A", "6B", "6C", "6D", "6E", "6F",
            "70", "71", "72", "73", "74", "75", "76", "77", "78", "79", "7A", "7B", "7C", "7D",
            "7E", "7F", "80", "81", "82", "83", "84", "85", "86", "87", "88", "89", "8A", "8B",
            "8C", "8D", "8E", "8F", "90", "91", "92", "93", "94", "95", "96", "97", "98", "99",
            "9A", "9B", "9C", "9D", "9E", "9F", "A0", "A1", "A2", "A3", "A4", "A5", "A6", "A7",
            "A8", "A9", "AA", "AB", "AC", "AD", "AE", "AF", "B0", "B1", "B2", "B3", "B4", "B5",
            "B6", "B7", "B8", "B9", "BA", "BB", "BC", "BD", "BE", "BF", "C0", "C1", "C2", "C3",
            "C4", "C5", "C6", "C7", "C8", "C9", "CA", "CB", "CC", "CD", "CE", "CF", "D0", "D1",
            "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9", "DA", "DB", "DC", "DD", "DE", "DF",
            "E0", "E1", "E2", "E3", "E4", "E5", "E6", "E7", "E8", "E9", "EA", "EB", "EC", "ED",
            "EE", "EF", "F0", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "FA", "FB",
            "FC", "FD", "FE", "FF",
        ];

        let mut expected = String::with_capacity(7);

        for first in 0..=255 {
            for second in 0..=255 {
                expected.clear();
                expected.push('[');
                expected.push_str(PAIRS[usize::from(first)]);
                expected.push(' ');
                expected.push_str(PAIRS[usize::from(second)]);
                expected.push(']');

                assert_eq!(format!("{:?}", DebugBigSlice(&[first, second])), expected);
            }
        }
    }

    #[test]
    fn debug_big_slice_works_for_16_bytes() {
        static SLICE: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

        assert_eq!(
            format!("{:?}", DebugBigSlice(&SLICE)),
            "[00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F]"
        );
    }

    #[test]
    fn debug_big_slice_works_for_127_bytes() {
        static SLICE: [u8; 127] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
            0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
            0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
            0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45,
            0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53,
            0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61,
            0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
            0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D,
            0x7E,
        ];

        assert_eq!(
            format!("{:?}", DebugBigSlice(&SLICE)),
            "[00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F \
10 11 12 13 14 15 16 17 18 19 1A 1B 1C 1D 1E 1F \
20 21 22 23 24 25 26 27 28 29 2A 2B 2C 2D 2E 2F \
30 31 32 33 34 35 36 37 38 39 3A 3B 3C 3D 3E 3F \
40 41 42 43 44 45 46 47 48 49 4A 4B 4C 4D 4E 4F \
50 51 52 53 54 55 56 57 58 59 5A 5B 5C 5D 5E 5F \
60 61 62 63 64 65 66 67 68 69 6A 6B 6C 6D 6E 6F \
70 71 72 73 74 75 76 77 78 79 7A 7B 7C 7D 7E]"
        );
    }

    #[test]
    fn debug_big_slice_works_for_256_bytes() {
        static SLICE: [u8; 256] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
            0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
            0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
            0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45,
            0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53,
            0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61,
            0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
            0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D,
            0x7E, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B,
            0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
            0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
            0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF, 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5,
            0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0, 0xC1, 0xC2, 0xC3,
            0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF, 0xD0, 0xD1,
            0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
            0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED,
            0xEE, 0xEF, 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB,
            0xFC, 0xFD, 0xFE, 0xFF,
        ];

        assert_eq!(
            format!("{:?}", DebugBigSlice(&SLICE)),
            "[00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F \
10 11 12 13 14 15 16 17 18 19 1A 1B 1C 1D 1E 1F \
20 21 22 23 24 25 26 27 28 29 2A 2B 2C 2D 2E 2F \
30 31 32 33 34 35 36 37 38 39 3A 3B 3C 3D 3E 3F \
40 41 42 43 44 45 46 47 48 49 4A 4B 4C 4D 4E 4F \
50 51 52 53 54 55 56 57 58 59 5A 5B 5C 5D 5E 5F \
60 61 62 63 64 65 66 67 68 69 6A 6B 6C 6D 6E 6F \
70 71 72 73 74 75 76 77 ... 136 more bytes (256 total) ...]"
        );
    }

    //  #####  # #    #   ##   #####  #   #         #####  ####          #####  #  ####  #####  #        ##   #   #
    //  #    # # ##   #  #  #  #    #  # #            #   #    #         #    # # #      #    # #       #  #   # #
    //  #####  # # #  # #    # #    #   #             #   #    #         #    # #  ####  #    # #      #    #   #
    //  #    # # #  # # ###### #####    #             #   #    #         #    # #      # #####  #      ######   #
    //  #    # # #   ## #    # #   #    #             #   #    #         #    # # #    # #      #      #    #   #
    //  #####  # #    # #    # #    #   #             #    ####          #####  #  ####  #      ###### #    #   #
    //                                      #######              #######

    #[test]
    fn binary_to_display_works_for_empty() {
        let mut result = String::new();
        binary_to_display(&mut result, &[]);
        assert_eq!(result, "");
    }

    #[test]
    fn binary_to_display_works_for_one_byte() {
        static VALUES: [&str; 256] = [
            "\\x00", "\\x01", "\\x02", "\\x03", "\\x04", "\\x05", "\\x06", "\\x07", "\\x08", "\\t",
            "\\n", "\\x0B", "\\x0C", "\\r", "\\x0E", "\\x0F", "\\x10", "\\x11", "\\x12", "\\x13",
            "\\x14", "\\x15", "\\x16", "\\x17", "\\x18", "\\x19", "\\x1A", "\\x1B", "\\x1C",
            "\\x1D", "\\x1E", "\\x1F", "\x20", "\x21", "\\\"", "\x23", "\x24", "\x25", "\x26",
            "\\'", "\x28", "\x29", "\x2A", "\x2B", "\x2C", "\x2D", "\x2E", "\x2F", "\x30", "\x31",
            "\x32", "\x33", "\x34", "\x35", "\x36", "\x37", "\x38", "\x39", "\x3A", "\x3B", "\x3C",
            "\x3D", "\x3E", "\x3F", "\x40", "\x41", "\x42", "\x43", "\x44", "\x45", "\x46", "\x47",
            "\x48", "\x49", "\x4A", "\x4B", "\x4C", "\x4D", "\x4E", "\x4F", "\x50", "\x51", "\x52",
            "\x53", "\x54", "\x55", "\x56", "\x57", "\x58", "\x59", "\x5A", "\x5B", "\\\\", "\x5D",
            "\x5E", "\x5F", "\x60", "\x61", "\x62", "\x63", "\x64", "\x65", "\x66", "\x67", "\x68",
            "\x69", "\x6A", "\x6B", "\x6C", "\x6D", "\x6E", "\x6F", "\x70", "\x71", "\x72", "\x73",
            "\x74", "\x75", "\x76", "\x77", "\x78", "\x79", "\x7A", "\x7B", "\x7C", "\x7D", "\x7E",
            "\\x7F", "\\x80", "\\x81", "\\x82", "\\x83", "\\x84", "\\x85", "\\x86", "\\x87",
            "\\x88", "\\x89", "\\x8A", "\\x8B", "\\x8C", "\\x8D", "\\x8E", "\\x8F", "\\x90",
            "\\x91", "\\x92", "\\x93", "\\x94", "\\x95", "\\x96", "\\x97", "\\x98", "\\x99",
            "\\x9A", "\\x9B", "\\x9C", "\\x9D", "\\x9E", "\\x9F", "\\xA0", "\\xA1", "\\xA2",
            "\\xA3", "\\xA4", "\\xA5", "\\xA6", "\\xA7", "\\xA8", "\\xA9", "\\xAA", "\\xAB",
            "\\xAC", "\\xAD", "\\xAE", "\\xAF", "\\xB0", "\\xB1", "\\xB2", "\\xB3", "\\xB4",
            "\\xB5", "\\xB6", "\\xB7", "\\xB8", "\\xB9", "\\xBA", "\\xBB", "\\xBC", "\\xBD",
            "\\xBE", "\\xBF", "\\xC0", "\\xC1", "\\xC2", "\\xC3", "\\xC4", "\\xC5", "\\xC6",
            "\\xC7", "\\xC8", "\\xC9", "\\xCA", "\\xCB", "\\xCC", "\\xCD", "\\xCE", "\\xCF",
            "\\xD0", "\\xD1", "\\xD2", "\\xD3", "\\xD4", "\\xD5", "\\xD6", "\\xD7", "\\xD8",
            "\\xD9", "\\xDA", "\\xDB", "\\xDC", "\\xDD", "\\xDE", "\\xDF", "\\xE0", "\\xE1",
            "\\xE2", "\\xE3", "\\xE4", "\\xE5", "\\xE6", "\\xE7", "\\xE8", "\\xE9", "\\xEA",
            "\\xEB", "\\xEC", "\\xED", "\\xEE", "\\xEF", "\\xF0", "\\xF1", "\\xF2", "\\xF3",
            "\\xF4", "\\xF5", "\\xF6", "\\xF7", "\\xF8", "\\xF9", "\\xFA", "\\xFB", "\\xFC",
            "\\xFD", "\\xFE", "\\xFF",
        ];

        let mut result = String::with_capacity(4);

        for byte in 0..=255 {
            result.clear();
            binary_to_display(&mut result, &[byte]);
            assert_eq!(result, VALUES[usize::from(byte)]);
        }
    }

    #[test]
    fn binary_to_display_works_for_two_bytes() {
        static PAIRS: [&str; 256] = [
            "\\x00", "\\x01", "\\x02", "\\x03", "\\x04", "\\x05", "\\x06", "\\x07", "\\x08", "\\t",
            "\\n", "\\x0B", "\\x0C", "\\r", "\\x0E", "\\x0F", "\\x10", "\\x11", "\\x12", "\\x13",
            "\\x14", "\\x15", "\\x16", "\\x17", "\\x18", "\\x19", "\\x1A", "\\x1B", "\\x1C",
            "\\x1D", "\\x1E", "\\x1F", "\x20", "\x21", "\\\"", "\x23", "\x24", "\x25", "\x26",
            "\\'", "\x28", "\x29", "\x2A", "\x2B", "\x2C", "\x2D", "\x2E", "\x2F", "\x30", "\x31",
            "\x32", "\x33", "\x34", "\x35", "\x36", "\x37", "\x38", "\x39", "\x3A", "\x3B", "\x3C",
            "\x3D", "\x3E", "\x3F", "\x40", "\x41", "\x42", "\x43", "\x44", "\x45", "\x46", "\x47",
            "\x48", "\x49", "\x4A", "\x4B", "\x4C", "\x4D", "\x4E", "\x4F", "\x50", "\x51", "\x52",
            "\x53", "\x54", "\x55", "\x56", "\x57", "\x58", "\x59", "\x5A", "\x5B", "\\\\", "\x5D",
            "\x5E", "\x5F", "\x60", "\x61", "\x62", "\x63", "\x64", "\x65", "\x66", "\x67", "\x68",
            "\x69", "\x6A", "\x6B", "\x6C", "\x6D", "\x6E", "\x6F", "\x70", "\x71", "\x72", "\x73",
            "\x74", "\x75", "\x76", "\x77", "\x78", "\x79", "\x7A", "\x7B", "\x7C", "\x7D", "\x7E",
            "\\x7F", "\\x80", "\\x81", "\\x82", "\\x83", "\\x84", "\\x85", "\\x86", "\\x87",
            "\\x88", "\\x89", "\\x8A", "\\x8B", "\\x8C", "\\x8D", "\\x8E", "\\x8F", "\\x90",
            "\\x91", "\\x92", "\\x93", "\\x94", "\\x95", "\\x96", "\\x97", "\\x98", "\\x99",
            "\\x9A", "\\x9B", "\\x9C", "\\x9D", "\\x9E", "\\x9F", "\\xA0", "\\xA1", "\\xA2",
            "\\xA3", "\\xA4", "\\xA5", "\\xA6", "\\xA7", "\\xA8", "\\xA9", "\\xAA", "\\xAB",
            "\\xAC", "\\xAD", "\\xAE", "\\xAF", "\\xB0", "\\xB1", "\\xB2", "\\xB3", "\\xB4",
            "\\xB5", "\\xB6", "\\xB7", "\\xB8", "\\xB9", "\\xBA", "\\xBB", "\\xBC", "\\xBD",
            "\\xBE", "\\xBF", "\\xC0", "\\xC1", "\\xC2", "\\xC3", "\\xC4", "\\xC5", "\\xC6",
            "\\xC7", "\\xC8", "\\xC9", "\\xCA", "\\xCB", "\\xCC", "\\xCD", "\\xCE", "\\xCF",
            "\\xD0", "\\xD1", "\\xD2", "\\xD3", "\\xD4", "\\xD5", "\\xD6", "\\xD7", "\\xD8",
            "\\xD9", "\\xDA", "\\xDB", "\\xDC", "\\xDD", "\\xDE", "\\xDF", "\\xE0", "\\xE1",
            "\\xE2", "\\xE3", "\\xE4", "\\xE5", "\\xE6", "\\xE7", "\\xE8", "\\xE9", "\\xEA",
            "\\xEB", "\\xEC", "\\xED", "\\xEE", "\\xEF", "\\xF0", "\\xF1", "\\xF2", "\\xF3",
            "\\xF4", "\\xF5", "\\xF6", "\\xF7", "\\xF8", "\\xF9", "\\xFA", "\\xFB", "\\xFC",
            "\\xFD", "\\xFE", "\\xFF",
        ];

        let mut expected = String::with_capacity(8);
        let mut result = String::with_capacity(8);

        for first in 0..=255 {
            for second in 0..=255 {
                expected.clear();
                expected.push_str(PAIRS[usize::from(first)]);
                expected.push_str(PAIRS[usize::from(second)]);

                result.clear();
                binary_to_display(&mut result, &[first, second]);

                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn binary_to_display_works_for_16_bytes() {
        static SLICE: [u8; 16] = *b"0123456789ABCDEF";

        let mut result = String::with_capacity(16);
        binary_to_display(&mut result, &SLICE);
        assert_eq!(result, "0123456789ABCDEF");
    }

    #[test]
    fn binary_to_display_works_for_256_bytes() {
        static SLICE: [u8; 256] = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
            0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29,
            0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
            0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45,
            0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50, 0x51, 0x52, 0x53,
            0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x61,
            0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
            0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D,
            0x7E, 0x7F, 0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B,
            0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99,
            0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F, 0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
            0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF, 0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5,
            0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF, 0xC0, 0xC1, 0xC2, 0xC3,
            0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xCB, 0xCC, 0xCD, 0xCE, 0xCF, 0xD0, 0xD1,
            0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xDB, 0xDC, 0xDD, 0xDE, 0xDF,
            0xE0, 0xE1, 0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED,
            0xEE, 0xEF, 0xF0, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA, 0xFB,
            0xFC, 0xFD, 0xFE, 0xFF,
        ];

        let mut result = String::with_capacity(1024);
        binary_to_display(&mut result, &SLICE);

        assert_eq!(
            result,
            "\\x00\\x01\\x02\\x03\\x04\\x05\\x06\\x07\\x08\\t\\n\\x0B\\x0C\\r\\x0E\\x0F\
\\x10\\x11\\x12\\x13\\x14\\x15\\x16\\x17\\x18\\x19\\x1A\\x1B\\x1C\\x1D\\x1E\\x1F\
\x20\x21\\\"\x23\x24\x25\x26\\'\x28\x29\x2A\x2B\x2C\x2D\x2E\x2F\
\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3A\x3B\x3C\x3D\x3E\x3F\
\x40\x41\x42\x43\x44\x45\x46\x47\x48\x49\x4A\x4B\x4C\x4D\x4E\x4F\
\x50\x51\x52\x53\x54\x55\x56\x57\x58\x59\x5A\x5B\\\\\x5D\x5E\x5F\
\x60\x61\x62\x63\x64\x65\x66\x67\x68\x69\x6A\x6B\x6C\x6D\x6E\x6F\
\x70\x71\x72\x73\x74\x75\x76\x77\x78\x79\x7A\x7B\x7C\x7D\x7E\\x7F\
\\x80\\x81\\x82\\x83\\x84\\x85\\x86\\x87\\x88\\x89\\x8A\\x8B\\x8C\\x8D\\x8E\\x8F\
\\x90\\x91\\x92\\x93\\x94\\x95\\x96\\x97\\x98\\x99\\x9A\\x9B\\x9C\\x9D\\x9E\\x9F\
\\xA0\\xA1\\xA2\\xA3\\xA4\\xA5\\xA6\\xA7\\xA8\\xA9\\xAA\\xAB\\xAC\\xAD\\xAE\\xAF\
\\xB0\\xB1\\xB2\\xB3\\xB4\\xB5\\xB6\\xB7\\xB8\\xB9\\xBA\\xBB\\xBC\\xBD\\xBE\\xBF\
\\xC0\\xC1\\xC2\\xC3\\xC4\\xC5\\xC6\\xC7\\xC8\\xC9\\xCA\\xCB\\xCC\\xCD\\xCE\\xCF\
\\xD0\\xD1\\xD2\\xD3\\xD4\\xD5\\xD6\\xD7\\xD8\\xD9\\xDA\\xDB\\xDC\\xDD\\xDE\\xDF\
\\xE0\\xE1\\xE2\\xE3\\xE4\\xE5\\xE6\\xE7\\xE8\\xE9\\xEA\\xEB\\xEC\\xED\\xEE\\xEF\
\\xF0\\xF1\\xF2\\xF3\\xF4\\xF5\\xF6\\xF7\\xF8\\xF9\\xFA\\xFB\\xFC\\xFD\\xFE\\xFF,"
        );
    }
}
