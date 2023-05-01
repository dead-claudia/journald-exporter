// Miri-compatible versions of a few libc routines.

use crate::prelude::*;

use std::ptr::NonNull;

// Use `NonNull` to let it be optimized within enums.
pub struct FixedCString(NonNull<libc::c_char>);

// SAFETY: This is safe as it's effectively constant and it's still memory-managed.
unsafe impl Send for FixedCString {}
// SAFETY: This is safe as it's effectively constant and it's still memory-managed.
unsafe impl Sync for FixedCString {}

#[cfg(test)]
fn allocation_failure() -> ! {
    panic!("allocation failure");
}

impl FixedCString {
    #[cfg(test)]
    // Make stack traces stop at the right spot.
    #[track_caller]
    pub fn new(value: &[u8]) -> Self {
        if value.contains(&b'\0') {
            panic!(
                "String data contains a null character: {:?}",
                BinaryToDebug(value)
            );
        }

        // SAFETY: asserts memory is allocated. `value` is verified to not contain nulls.
        unsafe {
            // Allocate it using standard C mechanisms, to align with what systemd does for its
            // field retrieval. I'd just do `CString::new(value).unwrap().into_raw()` except that
            // I can't use C's `free` to free Rust-allocated pointers.
            let result = match NonNull::new(libc::malloc(value.len().wrapping_add(1))) {
                Some(result) => result.cast::<i8>(),
                _ => allocation_failure(),
            };

            value
                .as_ptr()
                .copy_to_nonoverlapping(result.cast().as_ptr(), value.len());

            *result.as_ptr().offset(reinterpret_usize_isize(value.len())) = 0;

            Self::from_ptr(result)
        }
    }

    pub const unsafe fn from_ptr(ptr: NonNull<libc::c_char>) -> Self {
        Self(ptr)
    }

    pub const fn as_ptr(&self) -> *const libc::c_char {
        self.0.as_ptr()
    }

    /// Note: unlike most Rust collections, this is *not* O(1).
    pub fn len(&self) -> usize {
        // SAFETY: Caller is responsible for ensuring the pointer is allocated correctly.
        unsafe {
            let mut p = self.as_ptr();

            if !cfg!(miri) {
                return libc::strlen(p);
            }

            while p.read() != 0 {
                p = p.add(1);
            }

            reinterpret_isize_usize(p.offset_from(self.as_ptr()))
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: Caller is responsible for ensuring the pointer is allocated correctly.
        unsafe { std::slice::from_raw_parts(self.as_ptr().cast(), self.len()) }
    }
}

impl PartialEq for FixedCString {
    fn eq(&self, other: &FixedCString) -> bool {
        // SAFETY: Strings are guaranteed to be null terminated, and this is wrapping a C string.
        unsafe {
            let mut a = self.0.as_ptr();
            let mut b = other.0.as_ptr();

            if !cfg!(miri) {
                return libc::strcmp(a, b) == 0;
            }

            loop {
                let ac = a.read();
                let bc = b.read();

                // This catches both the case where one terminates before the other and when two
                // inner characters are inequal.
                if ac != bc {
                    return false;
                }

                // This implies `bc == 0` as well since `ac == bc`
                if ac == 0 {
                    return true;
                }

                a = a.add(1);
                b = b.add(1);
            }
        }
    }
}

impl Eq for FixedCString {}

impl std::hash::Hash for FixedCString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes())
    }
}

impl Drop for FixedCString {
    fn drop(&mut self) {
        // SAFETY: Only called on drop and just releases memory.
        unsafe {
            // Free it using standard C mechanisms, as it's usually allocated by systemd via the
            // system's native libc.
            libc::free(self.0.as_ptr().cast());
        }
    }
}

#[cfg(test)]
impl Clone for FixedCString {
    fn clone(&self) -> Self {
        // SAFETY: asserts memory is allocated. `data` is verified to not contain nulls.
        unsafe {
            let len = self.len().wrapping_add(1);
            let new_str = match NonNull::new(libc::malloc(len)) {
                Some(result) => result.cast(),
                _ => allocation_failure(),
            };

            self.0
                .as_ptr()
                .copy_to_nonoverlapping(new_str.as_ptr(), len);

            Self(new_str.cast())
        }
    }
}

impl fmt::Debug for FixedCString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        BinaryToDebug(self.as_bytes()).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_works() {
        let cstr_ptr = FixedCString::new(b"0123456789");
        assert_eq!(cstr_ptr, FixedCString::new(b"0123456789"));
        assert_ne!(cstr_ptr, FixedCString::new(b"9876543210"));
        assert_ne!(cstr_ptr, FixedCString::new(b"0123456"));
        assert_eq!(format!("{cstr_ptr:?}"), "\"0123456789\"");
        // The `clone` method is specifically being tested here.
        #[allow(clippy::redundant_clone)]
        let cloned_ptr = cstr_ptr.clone();
        assert_eq!(cloned_ptr, FixedCString::new(b"0123456789"));
        assert_ne!(cloned_ptr, FixedCString::new(b"9876543210"));
        assert_ne!(cloned_ptr, FixedCString::new(b"0123456"));
        assert_eq!(format!("{cloned_ptr:?}"), "\"0123456789\"");
    }

    #[test]
    #[should_panic = "String data contains a null character: \"01234\\x0056789\""]
    fn from_panics_if_data_contains_zero() {
        FixedCString::new(b"01234\x0056789");
    }
}
