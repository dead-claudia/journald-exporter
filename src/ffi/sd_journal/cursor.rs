use crate::prelude::*;

use std::ffi::CStr;
use std::ptr::NonNull;

// Avoid a lot of boilerplate compared to using the `systemd` crate by leveraging the API contract
// better. I'm not iterating journal values, so I don't really need as many safeguards.

// The docs specify cursors as opaque C strings. `systemd` shouldn't be exposing these literally.

pub struct Cursor {
    // Let it be optimized within enums.
    raw: NonNull<libc::c_char>,
}

// SAFETY: This is safe as it's effectively a constant string and it's still memory-managed.
unsafe impl Send for Cursor {}
// SAFETY: This is safe as it's effectively a constant string and it's still memory-managed.
unsafe impl Sync for Cursor {}

impl Cursor {
    pub unsafe fn from_ptr(raw: NonNull<libc::c_char>) -> Cursor {
        Cursor { raw }
    }

    fn as_ptr(&self) -> *const libc::c_char {
        self.raw.as_ptr()
    }

    pub fn as_c_str(&self) -> &CStr {
        // SAFETY: it's pulled from a known C string, and new data is checked to end with a null.
        unsafe { CStr::from_ptr(self.as_ptr()) }
    }

    #[cfg(test)]
    pub fn new(data: &[u8]) -> Cursor {
        if data.contains(&b'\0') {
            panic!("Cursor data contains a null character.");
        }

        // SAFETY: asserts memory is allocated. `data` is verified to not contain nulls.
        unsafe {
            // Allocate it using standard C mechanisms, to align with what systemd does for its
            // field retrieval. I'd just do `CString::new(data).unwrap().into_raw()` except that
            // I can't use C's `free` to free Rust-allocated pointers.
            let result = NonNull::<i8>::new(libc::malloc(data.len().wrapping_add(1)).cast())
                .expect("Could not allocate memory for cursor.");

            data.as_ptr()
                .copy_to_nonoverlapping(result.cast().as_ptr(), data.len());

            *result.as_ptr().offset(reinterpret_usize_isize(data.len())) = 0;

            Cursor::from_ptr(result)
        }
    }
}

impl fmt::Debug for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cursor")
            // SAFETY: It normally comes from systemd, but should always be a valid C string
            // provided everything else here is correct.
            .field("raw", &unsafe {
                std::ffi::CStr::from_ptr(self.raw.as_ptr())
            })
            .finish()
    }
}

impl Clone for Cursor {
    fn clone(&self) -> Cursor {
        // SAFETY: asserts memory is allocated. `data` is verified to not contain nulls.
        unsafe {
            // Allocate it using standard C mechanisms, to align with what systemd does for its
            // field retrieval.
            Cursor::from_ptr(
                NonNull::new(libc::strdup(self.as_ptr()))
                    .expect("Could not allocate memory for cursor."),
            )
        }
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        // SAFETY: Only called once and just releases memory.
        unsafe {
            // Free it using standard C mechanisms.
            libc::free(self.as_ptr().cast_mut().cast());
        }
    }
}

impl PartialEq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        // SAFETY: Strings are guaranteed to be null terminated, and this is wrapping a C string.
        unsafe { libc::strcmp(self.as_ptr(), other.as_ptr()) == 0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    fn cursor_works() {
        let cursor = Cursor::new(b"0123456789");
        assert_eq!(cursor, Cursor::new(b"0123456789"));
        assert_ne!(cursor, Cursor::new(b"9876543210"));
        assert_ne!(cursor, Cursor::new(b"0123456"));
        assert_eq!(format!("{cursor:?}"), r#"Cursor { raw: "0123456789" }"#);
        // The `clone` method is specifically being tested here.
        #[allow(clippy::redundant_clone)]
        let cloned = cursor.clone();
        assert_eq!(cloned, Cursor::new(b"0123456789"));
        assert_ne!(cloned, Cursor::new(b"9876543210"));
        assert_ne!(cloned, Cursor::new(b"0123456"));
        assert_eq!(format!("{cloned:?}"), r#"Cursor { raw: "0123456789" }"#);
    }

    #[test]
    // Skip in Miri due to FFI calls
    #[cfg_attr(miri, ignore)]
    #[should_panic = "Cursor data contains a null character."]
    fn cursor_new_panics_if_data_contains_zero() {
        Cursor::new(b"01234\x0056789");
    }
}
