use crate::prelude::*;

#[track_caller]
pub fn assert_error_eq(left: Error, right: Error) {
    if !error_eq(&left, &right) {
        assert_eq_fail(&left, &right);
    }
}

#[track_caller]
pub fn assert_result_eq<T>(left: io::Result<T>, right: io::Result<T>)
where
    T: PartialEq + fmt::Debug,
{
    if !result_eq(&left, &right) {
        assert_eq_fail(&left, &right);
    }
}

// Make monomorphic to reduce how much code is generated.
#[track_caller]
fn assert_eq_fail(left: &dyn fmt::Debug, right: &dyn fmt::Debug) -> ! {
    panic!(
        "assertion failed: `(left != right)`\n  left: {:?}\n right: {:?}",
        left, right
    );
}

// Skip these tests under Miri. They're test utilities and would just slow down Miri test runs.
#[cfg(not(miri))]
mod tests {
    use super::*;

    #[test]
    fn result_eq_assertion_works_on_pass() {
        assert_result_eq(Ok(123), Ok(123));
    }

    #[test]
    #[should_panic = "assertion failed: `(left != right)`\n  left: Ok(123)\n right: Err(Kind(NotFound))"]
    fn result_eq_assertion_works_on_fail() {
        assert_result_eq(Ok(123), Err(Error::from(ErrorKind::NotFound)));
    }
}
