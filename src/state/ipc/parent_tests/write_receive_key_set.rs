use crate::prelude::*;

use super::common::*;

#[test]
fn encodes_empty_receive_key_set() {
    assert_eq!(
        &*receive_key_set_bytes(KeySet::build(&[])),
        &[
            0x01, // Operation ID
            0x00, // Key set length
        ]
    );
}

#[test]
fn encodes_single_item_receive_key_set() {
    assert_eq!(
        &*receive_key_set_bytes(KeySet::build(&[b"0123456789ABCDEF"])),
        &[
            0x01, // Operation ID
            0x01, // Key set length
            0x10, // Key 1: all hex digits (length: 16)
            b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd',
            b'e', b'f',
        ]
    );
}

#[test]
fn encodes_max_len_receive_key_set() {
    #[rustfmt::skip]
    let expected = &[
        0x01, // Operation ID
        0xFF, // Key set length
        // Keys, of the following pattern (so it's easier to type):
        // - 2 `a`s
        // - 4 `b`s
        // - 2 `c`s
        // - 4 `d`s
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c', 0x04, b'd', b'd', b'd', b'd',
        0x02, b'a', b'a', 0x04, b'b', b'b', b'b', b'b', 0x02, b'c', b'c',
    ];

    // Keys, of the following pattern (so it's easier to type):
    // - 2 `A`s
    // - 4 `B`s
    // - 2 `C`s
    // - 4 `D`s
    #[rustfmt::skip]
    static KEYS_TO_ADD: &[&[u8]] = &[
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC", b"DDDD",
        b"AA", b"BBBB", b"CC", b"DDDD", b"AA", b"BBBB", b"CC",

    ];

    assert_eq!(
        &*receive_key_set_bytes(KeySet::build(KEYS_TO_ADD)),
        &expected[..]
    );
}
