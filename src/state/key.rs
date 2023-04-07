//! This specifically exists to ensure that passwords are:
//! 1. Treated as opaque as pragmatically possible, to allow for a better bead on security.
//! 2. Checked only in timing-resistant ways, to avoid leaking the password through the endpoint.

use crate::prelude::*;

// Represents a key strength of 256 bits. Should be enough for the foreseeable future.
pub const MAX_KEY_LEN: usize = 64;
pub const MAX_KEY_SET_LEN: usize = zero_extend_u8_usize(u8::MAX);

#[must_use = "Keys should not be ignored, as that could create security holes."]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Key {
    // Allocate a fixed-length pointer here to avoid potentially leaking the password length.
    // Otherwise, one could pass in small and increasingly larger keys and just benchmark latencies
    // to figure it out.
    //
    // Also has a secondary benefit in allowing it to be sized.
    raw: [u8; MAX_KEY_LEN],
}

impl zeroize::Zeroize for Key {
    fn zeroize(&mut self) {
        self.raw.zeroize();
    }
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key(\"{}\")", BinaryToDisplay(self.insecure_get_value()))
    }
}

fn is_valid_hex_string(bytes: &[u8]) -> bool {
    matches!(bytes.len(), len @ 1..=MAX_KEY_LEN if len % 2 == 0)
        && bytes
            .iter()
            .all(|b| matches!(b, b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f'))
}

fn normalize_hex(byte: u8) -> u8 {
    // Normalize for fast case-insensitive matching. This assumes it's already validated as a hex
    // character, so input can only be one of the following:
    // - `0-9`: `0x30-0x39` = `0b0011_xxxx`
    // - `A-F`: `0x41-0x46` = `0b0100_xxxx`
    // - `a-f`: `0x61-0x66` = `0b0110_xxxx`
    // What I want is digits (the first line) to remain the same, but upper and lower letters to be
    // normalized to an equivalent form. Conveniently, all I have to do is set the second bit, and
    // it's exactly what I need.
    byte | 0b0010_0000
}

impl Key {
    // This function has this name for a reason. Don't use it unless there's a very good reason,
    // like serializing it over the IPC channel.
    pub fn insecure_get_value(&self) -> &[u8] {
        let key_len = self.raw.iter().position(|c| *c == 0).unwrap_or(MAX_KEY_LEN);
        &self.raw[..key_len]
    }

    pub fn from_hex(hex: &[u8]) -> Option<Key> {
        if is_valid_hex_string(hex) {
            let mut raw = [0_u8; MAX_KEY_LEN];
            copy_to_start(&mut raw, hex);
            for r in raw[..hex.len()].iter_mut() {
                *r = normalize_hex(*r);
            }
            Some(Key { raw })
        } else {
            None
        }
    }

    #[track_caller]
    pub fn from_raw(b: &[u8]) -> Key {
        match Self::from_hex(b) {
            Some(key) => key,
            None => panic_invalid_length(b.len()),
        }
    }
}

#[cold]
#[inline(never)]
#[track_caller]
fn panic_invalid_length(len: usize) -> ! {
    if len == 0 {
        panic!("Key must not be empty.");
    }
    if len > MAX_KEY_LEN {
        panic!("Key length must be at most 256 bytes, but found {len}.");
    }
    if len % 2 != 0 {
        panic!("Key length must be even, but found {len}.");
    }
    panic!("Key length must only contain hex characters.");
}

#[derive(Debug)]
pub struct KeySet {
    key_set: Option<zeroize::Zeroizing<Box<[Key]>>>,
}

impl KeySet {
    pub const fn empty() -> KeySet {
        KeySet { key_set: None }
    }

    pub fn new(b: Box<[Key]>) -> KeySet {
        assert!(b.len() <= MAX_KEY_SET_LEN);
        KeySet {
            key_set: if b.is_empty() {
                None
            } else {
                Some(zeroize::Zeroizing::new(b))
            },
        }
    }

    pub fn into_insecure_view_keys(self) -> zeroize::Zeroizing<Box<[Key]>> {
        self.key_set
            .unwrap_or_else(|| zeroize::Zeroizing::new(Box::new([])))
    }

    pub fn check_key(&self, key: &[u8]) -> bool {
        // Check for correct syntax. This part isn't security-critical.
        let Some(input_key) = Key::from_hex(key) else {
            return false;
        };

        let Some(key_set) = &self.key_set else {
            return false;
        };

        // This is specially designed to avoid detection of both key length and matched key (if
        // multiple keys are available).

        let mut match_found = false;

        for trusted_key in key_set.iter() {
            let mut current_matched = true;

            for i in 0..MAX_KEY_LEN {
                let left = input_key.raw[i];
                let right = trusted_key.raw[i];
                current_matched = std::hint::black_box(current_matched & (left == right));
            }

            match_found = std::hint::black_box(match_found | current_matched);
        }

        std::hint::black_box(match_found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_create_key_from_invalid_hex_string_of_even_length() {
        assert!(Key::from_hex(b"definitely a non-hex string with even length").is_none());
    }

    #[test]
    fn does_not_create_key_from_invalid_hex_string_of_odd_length() {
        assert!(Key::from_hex(b"definitely a non-hex string with odd length").is_none());
    }

    #[test]
    fn does_not_create_key_from_hex_only_string_of_odd_length() {
        assert!(Key::from_hex(b"0123456789abcdef1").is_none());
    }

    #[test]
    fn rejects_if_empty_set() {
        let key_set = KeySet::empty();
        assert!(!key_set.check_key(b"definitely a non-hex string with even length"));
    }

    #[test]
    fn rejects_non_hex_keys_with_even_length() {
        let key_set = KeySet::new(Box::new([Key::from_hex(b"abcdef0123456789").unwrap()]));
        assert!(!key_set.check_key(b"definitely a non-hex string with even length"));
    }

    #[test]
    fn rejects_non_hex_keys_with_odd_length() {
        let key_set = KeySet::new(Box::new([Key::from_hex(b"abcdef0123456789").unwrap()]));
        assert!(!key_set.check_key(b"definitely a non-hex string with odd length"));
    }

    #[test]
    fn rejects_hex_keys_with_odd_length() {
        let key_set = KeySet::new(Box::new([Key::from_hex(b"abcdef0123456789").unwrap()]));
        assert!(!key_set.check_key(b"0123456789abcdef1"));
    }

    #[test]
    fn checks_hex_lower_against_single_hex_lower_match() {
        let key_set = KeySet::new(Box::new([Key::from_hex(b"0123456789abcdef").unwrap()]));
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_hex_lower_against_single_hex_upper_match() {
        let key_set = KeySet::new(Box::new([Key::from_hex(b"0123456789ABCDEF").unwrap()]));
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_hex_upper_against_single_hex_lower_match() {
        let key_set = KeySet::new(Box::new([Key::from_hex(b"0123456789abcdef").unwrap()]));
        assert!(key_set.check_key(b"0123456789ABCDEF"));
    }

    #[test]
    fn checks_hex_upper_against_single_hex_upper_match() {
        let key_set = KeySet::new(Box::new([Key::from_hex(b"0123456789ABCDEF").unwrap()]));
        assert!(key_set.check_key(b"0123456789ABCDEF"));
    }

    #[test]
    fn rejects_against_single_mismatch() {
        let key_set = KeySet::new(Box::new([Key::from_hex(b"abcdef0123456789").unwrap()]));
        assert!(!key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_against_multi_match_one() {
        let key_set = KeySet::new(Box::new([
            Key::from_hex(b"0123456789abcdef").unwrap(),
            Key::from_hex(b"aaaaaaaaaaaaaaaa").unwrap(),
        ]));
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_against_multi_match_second() {
        let key_set = KeySet::new(Box::new([
            Key::from_hex(b"aaaaaaaaaaaaaaaa").unwrap(),
            Key::from_hex(b"0123456789abcdef").unwrap(),
        ]));
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_against_multi_match_all() {
        let key_set = KeySet::new(Box::new([
            Key::from_hex(b"0123456789abcdef").unwrap(),
            Key::from_hex(b"0123456789abcdef").unwrap(),
        ]));
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn rejects_against_multi_match_none() {
        let key_set = KeySet::new(Box::new([
            Key::from_hex(b"abcdef0123456789").unwrap(),
            Key::from_hex(b"aaaaaaaaaaaaaaaa").unwrap(),
        ]));
        assert!(!key_set.check_key(b"0123456789abcdef"));
    }
}
