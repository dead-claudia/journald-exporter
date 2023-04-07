use crate::prelude::*;

// Avoid a lot of boilerplate compared to using the `systemd` crate by leveraging the API contract
// better. I'm not iterating journal values, so I don't really need as many safeguards.

// The docs specify cursors as opaque C strings. `systemd` should've just exposed these as a
// `free`able struct instance.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor(FixedCString);

impl Cursor {
    pub fn from_raw(raw: FixedCString) -> Self {
        Self(raw)
    }

    pub fn as_ptr(&self) -> *const libc::c_char {
        self.0.as_ptr()
    }

    #[cfg(test)]
    pub fn new(cursor_data: &[u8]) -> Self {
        Self(FixedCString::new(cursor_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_works() {
        let cursor = Cursor::new(b"0123456789");
        assert_eq!(cursor, Cursor::new(b"0123456789"));
        assert_ne!(cursor, Cursor::new(b"9876543210"));
        assert_ne!(cursor, Cursor::new(b"0123456"));
        assert_eq!(format!("{cursor:?}"), "Cursor(\"0123456789\")");
        // The `clone` method is specifically being tested here.
        #[allow(clippy::redundant_clone)]
        let cloned = cursor.clone();
        assert_eq!(cloned, Cursor::new(b"0123456789"));
        assert_ne!(cloned, Cursor::new(b"9876543210"));
        assert_ne!(cloned, Cursor::new(b"0123456"));
        assert_eq!(format!("{cloned:?}"), "Cursor(\"0123456789\")");
    }

    #[test]
    #[should_panic = "String data contains a null character."]
    fn cursor_new_panics_if_data_contains_zero() {
        Cursor::new(b"01234\x0056789");
    }
}
