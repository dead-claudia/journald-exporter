const fn has_zero_byte(mut s: &[u8]) -> bool {
    while let [head, tail @ ..] = s {
        if *head == 0 {
            return true;
        }
        s = tail;
    }

    false
}

// Wrapper function for an easier time defining C string constants.
pub const fn c_str(s: &'static [u8]) -> &'static std::ffi::CStr {
    match s {
        // SAFETY: this pattern case checks the invariant. Once the checked version stabilizes in
        // `const` contexts, the `unsafe` here can be removed.
        [head @ .., 0] if !has_zero_byte(head) => unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(s)
        },
        _ => panic!("C strings must end with a null (`\\0`) byte."),
    }
}

// Makes tests a little more readable.
#[cfg(all(test, not(miri)))]
pub fn c_string(s: &'static [u8]) -> std::ffi::CString {
    c_str(s).to_owned()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn c_str_works() {
        assert_eq!(
            c_str(b"test\0"),
            std::ffi::CString::new("test").unwrap().as_c_str()
        );
    }
}
