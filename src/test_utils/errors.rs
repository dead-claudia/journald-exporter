use crate::prelude::*;

pub fn error_eq(left: &Error, right: &Error) -> bool {
    match (left.raw_os_error(), right.raw_os_error()) {
        (Some(a), Some(b)) if a == b => true,
        (None, None) => match (left.kind(), right.kind()) {
            (a, b) if a == b => match (left.get_ref(), right.get_ref()) {
                (Some(a), Some(b)) if a.to_string() == b.to_string() => true,
                (None, None) => true,
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

pub fn result_eq<T: PartialEq>(left: &io::Result<T>, right: &io::Result<T>) -> bool {
    match (left, right) {
        (Ok(a), Ok(b)) => a == b,
        (Err(left), Err(right)) => error_eq(left, right),
        _ => false,
    }
}

static ERROR_KINDS: [ErrorKind; 20] = [
    ErrorKind::NotFound,
    ErrorKind::PermissionDenied,
    ErrorKind::ConnectionRefused,
    ErrorKind::ConnectionReset,
    ErrorKind::ConnectionAborted,
    ErrorKind::NotConnected,
    ErrorKind::AddrInUse,
    ErrorKind::AddrNotAvailable,
    ErrorKind::BrokenPipe,
    ErrorKind::AlreadyExists,
    ErrorKind::WouldBlock,
    ErrorKind::InvalidInput,
    ErrorKind::InvalidData,
    ErrorKind::TimedOut,
    ErrorKind::WriteZero,
    ErrorKind::Interrupted,
    ErrorKind::Unsupported,
    ErrorKind::UnexpectedEof,
    ErrorKind::OutOfMemory,
    ErrorKind::Other,
];

fn kind_arbitrary(g: &mut Gen) -> ErrorKind {
    *g.choose(&ERROR_KINDS).unwrap()
}

fn kind_shrink(kind: ErrorKind) -> impl Iterator<Item = ErrorKind> {
    ERROR_KINDS.iter().copied().filter(move |k| *k < kind)
}

pub fn error_arbitrary(g: &mut Gen) -> Error {
    enum S {
        Code,
        Kind,
        Custom,
    }

    match g.choose(&[S::Code, S::Kind, S::Custom]).unwrap() {
        S::Code => Error::from_raw_os_error(crate::ffi::errno_arbitrary(g)),
        S::Kind => Error::from(kind_arbitrary(g)),
        S::Custom => Error::new(kind_arbitrary(g), <String>::arbitrary(g)),
    }
}

pub fn error_clone(error: &Error) -> Error {
    match error.raw_os_error() {
        Some(code) => Error::from_raw_os_error(code),
        None => match error.get_ref() {
            Some(inner) => Error::new(error.kind(), inner.to_string()),
            None => Error::from(error.kind()),
        },
    }
}

pub fn error_shrink(error: &Error) -> Box<dyn Iterator<Item = Error>> {
    // To avoid capturing `error` (and by proxy, `self`).
    #[derive(Clone)]
    enum E {
        Code(libc::c_int),
        Kind(ErrorKind),
        Custom(ErrorKind, String),
    }

    let error_inspect = match error.raw_os_error() {
        Some(code) => E::Code(code),
        None => match error.get_ref() {
            Some(inner) => E::Custom(error.kind(), inner.to_string()),
            None => E::Kind(error.kind()),
        },
    };

    match error_inspect {
        E::Code(code) => Box::new(crate::ffi::errno_shrink(code).map(Error::from_raw_os_error)),
        E::Kind(kind) => Box::new(kind_shrink(kind).map(Error::from)),
        E::Custom(kind, inner) => Box::new(
            kind_shrink(kind).flat_map(move |k| inner.shrink().map(move |i| Error::new(k, i))),
        ),
    }
}

// Skip these tests under Miri. They're test utilities and would just slow down Miri test runs.
#[cfg(not(miri))]
mod tests {
    #![allow(clippy::bool_assert_comparison)]

    use super::*;

    #[derive(Debug)]
    struct ErrorSelect(Error);

    impl Clone for ErrorSelect {
        fn clone(&self) -> Self {
            Self(error_clone(&self.0))
        }
    }

    impl Arbitrary for ErrorSelect {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(error_arbitrary(g))
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(error_shrink(&self.0).map(Self))
        }
    }

    #[derive(Debug)]
    struct ResultSelect(io::Result<u8>);

    impl Clone for ResultSelect {
        fn clone(&self) -> Self {
            Self(match &self.0 {
                Ok(v) => Ok(*v),
                Err(e) => Err(error_clone(e)),
            })
        }
    }

    impl Arbitrary for ResultSelect {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(<Result<u8, ErrorSelect>>::arbitrary(g).map_err(|e| e.0))
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            match &self.0 {
                Ok(v) => Box::new(v.shrink().map(|v| Self(Ok(v)))),
                Err(e) => Box::new(error_shrink(e).map(|e| Self(Err(e)))),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ErrnoSelect(libc::c_int);

    impl Arbitrary for ErrnoSelect {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(crate::ffi::errno_arbitrary(g))
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(crate::ffi::errno_shrink(self.0).map(Self))
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct KindSelect(ErrorKind);

    impl Arbitrary for KindSelect {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(kind_arbitrary(g))
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            Box::new(kind_shrink(self.0).map(Self))
        }
    }

    #[quickcheck]
    fn error_eq_holds_for_raw_os_errors(code_a: ErrnoSelect, code_b: ErrnoSelect) -> bool {
        let a = Error::from_raw_os_error(code_a.0);
        let b = Error::from_raw_os_error(code_b.0);
        error_eq(&a, &b) == (code_a == code_b)
    }

    #[quickcheck]
    fn error_eq_holds_for_same_kinds(kind_a: KindSelect, kind_b: KindSelect) -> bool {
        let a = Error::from(kind_a.0);
        let b = Error::from(kind_b.0);
        error_eq(&a, &b) == (kind_a == kind_b)
    }

    #[quickcheck]
    fn error_eq_holds_for_same_customs(
        kind_a: KindSelect,
        msg_a: String,
        kind_b: KindSelect,
        msg_b: String,
    ) -> bool {
        let a = Error::new(kind_a.0, msg_a.clone());
        let b = Error::new(kind_b.0, msg_b.clone());
        error_eq(&a, &b) == (kind_a == kind_b && msg_a == msg_b)
    }

    #[test]
    fn error_eq_fails_for_raw_os_error_vs_kind() {
        let a = Error::from_raw_os_error(libc::ENOENT);
        let b = Error::from(ErrorKind::NotFound);
        assert_eq!(error_eq(&a, &b), false);
    }

    #[test]
    fn error_eq_fails_for_kind_vs_raw_os_error() {
        let a = Error::from(ErrorKind::NotFound);
        let b = Error::from_raw_os_error(libc::ENOENT);
        assert_eq!(error_eq(&a, &b), false);
    }

    #[test]
    fn error_eq_fails_for_raw_os_error_vs_custom() {
        let a = Error::from_raw_os_error(libc::ENOENT);
        let b = Error::new(ErrorKind::NotFound, "test");
        assert_eq!(error_eq(&a, &b), false);
    }

    #[test]
    fn error_eq_fails_for_custom_vs_raw_os_error() {
        let a = Error::new(ErrorKind::NotFound, "test");
        let b = Error::from_raw_os_error(libc::ENOENT);
        assert_eq!(error_eq(&a, &b), false);
    }

    #[test]
    fn error_eq_fails_for_kind_vs_custom() {
        let a = Error::from(ErrorKind::NotFound);
        let b = Error::new(ErrorKind::NotFound, "test");
        assert_eq!(error_eq(&a, &b), false);
    }

    #[test]
    fn error_eq_fails_for_custom_vs_kind() {
        let a = Error::new(ErrorKind::NotFound, "test");
        let b = Error::from(ErrorKind::NotFound);
        assert_eq!(error_eq(&a, &b), false);
    }

    #[quickcheck]
    fn error_eq_equality_is_reflexive(a: ErrorSelect) -> bool {
        error_eq(&a.0, &a.0)
    }

    #[quickcheck]
    fn error_eq_equality_is_reflexive_across_clones(a: ErrorSelect) -> bool {
        error_eq(&error_clone(&a.0), &a.0)
    }

    #[quickcheck]
    fn error_eq_equality_is_symmetric(a: ErrorSelect, b: ErrorSelect) -> bool {
        error_eq(&a.0, &b.0) == error_eq(&b.0, &a.0)
    }

    #[quickcheck]
    fn error_eq_equality_is_transitive(a: ErrorSelect, b: ErrorSelect, c: ErrorSelect) -> bool {
        if error_eq(&a.0, &b.0) && error_eq(&b.0, &c.0) {
            error_eq(&a.0, &c.0)
        } else {
            true // just pass the test
        }
    }

    #[test]
    fn error_eq_assertion_works_on_pass() {
        assert_error_eq(
            Error::from_raw_os_error(libc::ENOENT),
            Error::from_raw_os_error(libc::ENOENT),
        );
    }

    #[test]
    fn error_eq_assertion_works_on_fail() {
        assert!(!error_eq(
            &Error::from_raw_os_error(libc::ENOENT),
            &Error::from_raw_os_error(libc::EISDIR),
        ));
    }

    #[quickcheck]
    fn result_eq_holds_for_ok(a: u8, b: u8) -> bool {
        result_eq(&Ok(a), &Ok(b)) == (a == b)
    }

    #[quickcheck]
    fn result_eq_holds_for_errors(a: ErrorSelect, b: ErrorSelect) -> bool {
        error_eq(&a.0, &b.0) == result_eq::<u8>(&Err(a.0), &Err(b.0))
    }

    #[test]
    fn error_eq_fails_for_ok_vs_err() {
        let a = Ok(123);
        let b = Err(Error::from(ErrorKind::NotFound));
        assert_eq!(result_eq(&a, &b), false);
    }

    #[test]
    fn error_eq_fails_for_err_vs_ok() {
        let a = Err(Error::from(ErrorKind::NotFound));
        let b = Ok(123);
        assert_eq!(result_eq(&a, &b), false);
    }

    #[quickcheck]
    fn result_eq_equality_is_reflexive(a: ResultSelect) -> bool {
        result_eq(&a.0, &a.0)
    }

    #[quickcheck]
    #[allow(clippy::redundant_clone)]
    fn result_eq_equality_is_reflexive_across_clones(a: ResultSelect) -> bool {
        result_eq(&a.clone().0, &a.0)
    }

    #[quickcheck]
    fn result_eq_equality_is_symmetric(a: ResultSelect, b: ResultSelect) -> bool {
        result_eq(&a.0, &b.0) == result_eq(&b.0, &a.0)
    }

    #[quickcheck]
    fn result_eq_equality_is_transitive(a: ResultSelect, b: ResultSelect, c: ResultSelect) -> bool {
        if result_eq(&a.0, &b.0) && result_eq(&b.0, &c.0) {
            result_eq(&a.0, &c.0)
        } else {
            true // just pass the test
        }
    }

    #[test]
    fn result_eq_assertion_works_on_pass() {
        assert_result_eq(Ok(123), Ok(123));
    }

    #[test]
    fn result_eq_assertion_works_on_fail() {
        assert!(!result_eq(
            &Ok(123),
            &Err(Error::from_raw_os_error(libc::ENOENT))
        ));
    }
}
