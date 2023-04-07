use crate::prelude::*;

#[cold]
#[inline(never)]
fn syscall_check_fail(syscall_name: &'static str) -> Error {
    let e = Error::last_os_error();
    if matches!(
        e.raw_os_error(),
        Some(libc::ENOSYS | libc::EINVAL | libc::EFAULT)
    ) {
        super::panic_errno(e, syscall_name);
    }
    e
}

#[cold]
#[inline(never)]
fn sd_check_fail(syscall_name: &'static str, result: libc::c_int) -> Error {
    let code = result.wrapping_neg();
    let e = Error::from_raw_os_error(code);
    if matches!(code, libc::ENOSYS | libc::EINVAL | libc::EFAULT) {
        super::panic_errno(e, syscall_name);
    }
    e
}

pub fn syscall_check_int(
    syscall_name: &'static str,
    result: libc::c_int,
) -> io::Result<libc::c_int> {
    if result >= 0 {
        Ok(result)
    } else {
        Err(syscall_check_fail(syscall_name))
    }
}

pub fn syscall_check_long(
    syscall_name: &'static str,
    result: libc::c_long,
) -> io::Result<libc::c_long> {
    if result >= 0 {
        Ok(result)
    } else {
        Err(syscall_check_fail(syscall_name))
    }
}

pub fn sd_check(syscall_name: &'static str, result: libc::c_int) -> io::Result<libc::c_int> {
    if result >= 0 {
        Ok(result)
    } else {
        Err(sd_check_fail(syscall_name, result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sd_check_works_on_zero() {
        assert_result_eq(sd_check("invalid argument detected", 0), Ok(0));
    }

    #[test]
    fn sd_check_works_on_positive() {
        assert_result_eq(sd_check("invalid argument detected", 123), Ok(123));
    }

    #[test]
    fn sd_check_works_on_ok_errno() {
        assert_result_eq(
            sd_check("invalid argument detected", -libc::ENOENT),
            Err(Error::from_raw_os_error(libc::ENOENT)),
        );
    }

    #[test]
    #[should_panic = "invalid argument detected"]
    fn sd_check_panics_on_missing_syscall_error() {
        drop(sd_check("invalid argument detected", -libc::ENOSYS));
    }

    #[test]
    #[should_panic = "invalid argument detected"]
    fn sd_check_panics_on_invalid_argument_error() {
        drop(sd_check("invalid argument detected", -libc::EINVAL));
    }

    #[test]
    #[should_panic = "invalid argument detected"]
    fn sd_check_panics_on_system_fault_error() {
        drop(sd_check("invalid argument detected", -libc::EFAULT));
    }
}
