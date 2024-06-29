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

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key({:?})", BinaryToDebug(self.insecure_get_value()))
    }
}

fn is_valid_hex_string(bytes: &[u8]) -> bool {
    matches!(bytes.len(), len @ 1..=MAX_KEY_LEN if len % 2 == 0)
        && bytes
            .iter()
            .all(|b| b.is_hex_digit())
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
}

#[derive(Debug)]
pub struct KeySetBuilder {
    key_set: Vec<Key>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPushResult {
    Success,
    Invalid,
    TooManyKeys,
}

impl KeySetBuilder {
    pub fn new() -> Self {
        Self {
            key_set: Vec::new(),
        }
    }

    pub fn try_reserve(len: usize) -> Option<Self> {
        Some(Self {
            key_set: try_new_dynamic_vec(len)?,
        })
    }

    #[must_use]
    pub fn push_hex(&mut self, key: &[u8]) -> KeyPushResult {
        if !is_valid_hex_string(key) {
            KeyPushResult::Invalid
        } else if self.key_set.len() == MAX_KEY_SET_LEN {
            KeyPushResult::TooManyKeys
        } else {
            // SAFETY: It's validated to be correct.
            unsafe { self.push_raw(key) }
            KeyPushResult::Success
        }
    }

    /// Safety note: `key` must be validated to satisfy the following constraints:
    /// - The slice itself is non-empty.
    /// - The slice contains at most 64 characters.
    /// - The slice is of even length.
    /// - The slice's contents consist of only hexadecimal digits.
    pub unsafe fn push_raw(&mut self, key: &[u8]) {
        debug_assert!(key.len() < MAX_KEY_LEN);

        let tail = self.key_set.len();
        if tail == MAX_KEY_SET_LEN {
            unreachable!();
        }

        self.key_set.reserve(1);
        // SAFETY: Push the key to the set while ensuring it's never written to the stack.
        let mut dest = self.key_set.as_mut_ptr().add(tail).cast::<u8>();
        debug_assert_eq!(dest.align_offset(std::mem::align_of::<Key>()), 0);
        for byte in key.iter() {
            *dest = normalize_hex(*byte);
            dest = dest.add(1);
        }
        for _ in key.len()..MAX_KEY_LEN {
            *dest = 0;
            dest = dest.add(1);
        }
        self.key_set.set_len(tail.wrapping_add(1));
    }

    pub fn finish(self) -> KeySet {
        // SAFETY: Bypassing the drop logic, to avoid clearing the inner vector.
        let key_set = unsafe {
            let this = std::mem::ManuallyDrop::new(self);
            std::ptr::read(&this.key_set)
        };
        KeySet {
            key_set: key_set.into(),
        }
    }
}

impl Drop for KeySetBuilder {
    fn drop(&mut self) {
        // SAFETY: All zeroes are valid for keys.
        unsafe {
            secure_clear(&mut self.key_set);
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct KeySet {
    key_set: Box<[Key]>,
}

impl Drop for KeySet {
    fn drop(&mut self) {
        // SAFETY: All zeroes are valid for keys.
        unsafe {
            secure_clear(&mut self.key_set);
        }
    }
}

impl KeySet {
    #[cfg(test)]
    pub fn build(keys: &[&[u8]]) -> Self {
        let mut builder = KeySetBuilder::new();
        assert!(
            keys.len() <= MAX_KEY_SET_LEN,
            "Too many keys: {}",
            keys.len()
        );
        for key in keys {
            match builder.push_hex(key) {
                KeyPushResult::Success => {}
                KeyPushResult::Invalid => panic!("Key is invalid: {:?}", BinaryToDebug(key)),
                // Should've been validated already.
                KeyPushResult::TooManyKeys => unreachable!(),
            }
        }
        builder.finish()
    }

    pub fn insecure_view_keys(&self) -> &[Key] {
        &self.key_set
    }

    pub fn check_key(&self, key: &[u8]) -> bool {
        // Check for correct syntax. This part isn't security-critical.
        if !is_valid_hex_string(key) {
            return false;
        }

        // This is specially designed to avoid detection of both key length and matched key (if
        // multiple keys are available).

        let mut match_found = false;

        for trusted_key in self.key_set.iter() {
            let mut current_matched = true;

            for (&left, &right) in key.iter().zip(&trusted_key.raw) {
                current_matched =
                    std::hint::black_box(current_matched & (normalize_hex(left) == right));
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
        let mut builder = KeySetBuilder::new();
        assert_eq!(
            builder.push_hex(b"definitely a non-hex string with even length"),
            KeyPushResult::Invalid
        );
    }

    #[test]
    fn does_not_create_key_from_invalid_hex_string_of_odd_length() {
        let mut builder = KeySetBuilder::new();
        assert_eq!(
            builder.push_hex(b"definitely a non-hex string with odd length"),
            KeyPushResult::Invalid
        );
    }

    #[test]
    fn does_not_create_key_from_hex_only_string_of_odd_length() {
        let mut builder = KeySetBuilder::new();
        assert_eq!(
            builder.push_hex(b"0123456789abcdef1"),
            KeyPushResult::Invalid
        );
    }

    #[test]
    fn rejects_if_empty_set() {
        let key_set = KeySet::build(&[]);
        assert!(!key_set.check_key(b"definitely a non-hex string with even length"));
    }

    #[test]
    fn rejects_non_hex_keys_with_even_length() {
        let key_set = KeySet::build(&[b"abcdef0123456789"]);
        assert!(!key_set.check_key(b"definitely a non-hex string with even length"));
    }

    #[test]
    fn rejects_non_hex_keys_with_odd_length() {
        let key_set = KeySet::build(&[b"abcdef0123456789"]);
        assert!(!key_set.check_key(b"definitely a non-hex string with odd length"));
    }

    #[test]
    fn rejects_hex_keys_with_odd_length() {
        let key_set = KeySet::build(&[b"abcdef0123456789"]);
        assert!(!key_set.check_key(b"0123456789abcdef1"));
    }

    #[test]
    fn checks_hex_lower_against_single_hex_lower_match() {
        let key_set = KeySet::build(&[b"0123456789abcdef"]);
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_hex_lower_against_single_hex_upper_match() {
        let key_set = KeySet::build(&[b"0123456789ABCDEF"]);
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_hex_upper_against_single_hex_lower_match() {
        let key_set = KeySet::build(&[b"0123456789abcdef"]);
        assert!(key_set.check_key(b"0123456789ABCDEF"));
    }

    #[test]
    fn checks_hex_upper_against_single_hex_upper_match() {
        let key_set = KeySet::build(&[b"0123456789ABCDEF"]);
        assert!(key_set.check_key(b"0123456789ABCDEF"));
    }

    #[test]
    fn rejects_against_single_mismatch() {
        let key_set = KeySet::build(&[b"abcdef0123456789"]);
        assert!(!key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_against_multi_match_one() {
        let key_set = KeySet::build(&[b"0123456789abcdef", b"aaaaaaaaaaaaaaaa"]);
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_against_multi_match_second() {
        let key_set = KeySet::build(&[b"aaaaaaaaaaaaaaaa", b"0123456789abcdef"]);
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn checks_against_multi_match_all() {
        let key_set = KeySet::build(&[b"0123456789abcdef", b"0123456789abcdef"]);
        assert!(key_set.check_key(b"0123456789abcdef"));
    }

    #[test]
    fn rejects_against_multi_match_none() {
        let key_set = KeySet::build(&[b"abcdef0123456789", b"aaaaaaaaaaaaaaaa"]);
        assert!(!key_set.check_key(b"0123456789abcdef"));
    }
}
