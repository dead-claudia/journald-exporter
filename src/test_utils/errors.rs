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

pub(super) static ERROR_KINDS: &[ErrorKind] = &[
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

// Skip these tests under Miri. They're test utilities and would just slow down Miri test runs.
#[cfg(not(miri))]
#[allow(clippy::as_conversions)]
#[allow(clippy::bool_assert_comparison)]
mod tests {

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ErrnoSelect(libc::c_int);

    struct ErrnoShrinker(propcheck::UsizeShrinker, ErrnoSelect);

    impl propcheck::Shrinker for ErrnoShrinker {
        type Item = ErrnoSelect;
        fn next(&mut self) -> Option<&Self::Item> {
            self.1 = ErrnoSelect(crate::ffi::ERRNO_LIST[*self.0.next()?]);
            Some(&self.1)
        }
    }

    impl propcheck::Arbitrary for ErrnoSelect {
        type Shrinker = ErrnoShrinker;

        fn arbitrary() -> Self {
            Self(rand::random())
        }

        fn clone(&self) -> Self {
            Self(self.0)
        }

        fn shrink(&self) -> Self::Shrinker {
            let index = crate::ffi::ERRNO_LIST
                .iter()
                .position(|c| *c == self.0)
                .unwrap();
            ErrnoShrinker(index.shrink(), Self(0))
        }
    }

    #[test]
    fn error_eq_holds_for_raw_os_errors() {
        propcheck::run(|&[ErrnoSelect(code_a), ErrnoSelect(code_b)]: &[_; 2]| {
            let a = Error::from_raw_os_error(code_a);
            let b = Error::from_raw_os_error(code_b);
            error_eq(&a, &b) == (code_a == code_b)
        });
    }

    #[test]
    fn error_eq_holds_for_same_kinds() {
        propcheck::run(|&[kind_a, kind_b]: &[std::io::ErrorKind; 2]| {
            let a = Error::from(kind_a);
            let b = Error::from(kind_b);
            error_eq(&a, &b) == (kind_a == kind_b)
        });
    }

    #[test]
    fn error_eq_holds_for_same_customs() {
        propcheck::run(
            |[(kind_a, msg_a), (kind_b, msg_b)]: &[(std::io::ErrorKind, String); 2]| {
                let a = Error::new(*kind_a, msg_a.clone());
                let b = Error::new(*kind_b, msg_b.clone());
                error_eq(&a, &b) == (kind_a == kind_b && msg_a == msg_b)
            },
        );
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

    #[test]
    fn error_eq_equality_is_reflexive() {
        propcheck::run(|a: &std::io::Error| error_eq(a, a));
    }

    #[test]
    fn error_eq_equality_is_reflexive_across_clones() {
        use propcheck::Arbitrary as _;
        propcheck::run(|a: &std::io::Error| error_eq(&a.clone(), a));
    }

    #[test]
    fn error_eq_equality_is_symmetric() {
        propcheck::run(|[a, b]: &[std::io::Error; 2]| error_eq(a, b) == error_eq(b, a));
    }

    #[test]
    fn error_eq_equality_is_transitive() {
        propcheck::run(|[a, b, c]: &[std::io::Error; 3]| {
            if error_eq(a, b) && error_eq(b, c) {
                error_eq(a, c)
            } else {
                true // just pass the test
            }
        });
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

    #[test]
    fn result_eq_holds_for_ok() {
        propcheck::run(|&[a, b]: &[u8; 2]| result_eq(&Ok(a), &Ok(b)) == (a == b));
    }

    #[test]
    fn result_eq_holds_for_errors() {
        use propcheck::Arbitrary as _;
        propcheck::run(|[a, b]: &[std::io::Error; 2]| {
            error_eq(a, b) == result_eq::<()>(&Err(a.clone()), &Err(b.clone()))
        });
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

    #[test]
    fn result_eq_equality_is_reflexive() {
        propcheck::run(|a: &std::io::Result<u8>| result_eq(a, a));
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn result_eq_equality_is_reflexive_across_clones() {
        use propcheck::Arbitrary as _;
        propcheck::run(|a: &std::io::Result<u8>| result_eq(&a.clone(), a));
    }

    #[test]
    fn result_eq_equality_is_symmetric() {
        propcheck::run(|[a, b]: &[std::io::Result<u8>; 2]| result_eq(a, b) == result_eq(b, a));
    }

    #[test]
    fn result_eq_equality_is_transitive() {
        propcheck::run(|[a, b, c]: &[std::io::Result<u8>; 3]| {
            if result_eq(a, b) && result_eq(b, c) {
                result_eq(a, c)
            } else {
                true // just pass the test
            }
        });
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
