use crate::prelude::*;

use std::backtrace::Backtrace;

fn get_test_name_from_backtrace_line(line: &str) -> Option<&str> {
    let line = line.trim();

    if line.starts_with("at ") {
        return None;
    }

    let line = line.strip_prefix(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])?;
    let line = line.strip_prefix(": ")?;

    if line.starts_with('<') {
        return None;
    }

    let line = line.strip_prefix(env!("CARGO_CRATE_NAME"))?;
    let line = line.strip_prefix("::")?;

    let line = line.strip_suffix("::{{closure}}").unwrap_or(line);

    if !line
        .split("::")
        .any(|segment| segment == "tests" || segment.ends_with("_tests"))
    {
        return None;
    }

    Some(line)
}

fn get_test_name_from_backtrace(backtrace: Backtrace) -> String {
    backtrace
        .to_string()
        .lines()
        // The backtrace starts from the end. I want to start from the start so I can avoid test
        // helpers.
        .rev()
        .find_map(get_test_name_from_backtrace_line)
        .unwrap_or("(unknown)")
        .to_owned()
}

// Output isn't perfect, but good enough to not be too noisy.
#[track_caller]
pub fn with_attempts(max_attempts: usize, reattempt_delay: f64, body: &dyn Fn()) {
    // This intentionally prints to stderr normally.
    #![allow(clippy::print_stderr)]

    if max_attempts == 0 {
        panic!("At least one attempt must be provided!");
    }

    let mut test_name = None;
    let mut attempt = 1;

    loop {
        let e = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)) {
            Ok(()) => break,
            Err(e) => e,
        };

        let test_name =
            test_name.get_or_insert_with(|| get_test_name_from_backtrace(Backtrace::capture()));

        if attempt < max_attempts {
            eprintln!("test {} attempt {} failed, retrying...", test_name, attempt);
            attempt += 1;

            if reattempt_delay != 0.0 {
                std::thread::sleep(Duration::from_secs_f64(reattempt_delay));
            }

            continue;
        }

        if attempt == 1 {
            eprintln!("test {} failed after 1 attempt.", test_name);
        } else {
            eprintln!("test {} failed after {} attempts.", test_name, attempt);
        }

        std::panic::resume_unwind(e)
    }
}

pub struct PerAttemptStatic<T, const N: usize> {
    offset: AtomicUsize,
    data: [T; N],
}

impl<T, const N: usize> PerAttemptStatic<T, N> {
    pub const fn new(data: [T; N]) -> Self {
        Self {
            offset: AtomicUsize::new(0),
            data,
        }
    }

    pub fn next(&self) -> &T {
        &self.data[self.offset.fetch_add(1, Ordering::AcqRel)]
    }
}

// Skip these tests under Miri. They're test utilities and would just slow down Miri test runs.
#[cfg(not(miri))]
mod tests {
    use super::*;

    #[test]
    fn works_on_success() {
        static CALLS: Counter = Counter::new(0);

        with_attempts(3, 0.1, &|| {
            CALLS.increment();
        });

        assert_eq!(CALLS.current(), 1);
    }

    #[test]
    fn works_on_panic_then_success() {
        static CALLS: Counter = Counter::new(0);

        with_attempts(3, 0.1, &|| {
            if CALLS.increment() < 2 {
                panic!("expected fail");
            }
        });

        assert_eq!(CALLS.current(), 2);
    }

    #[test]
    fn works_on_almost_too_many_panics_then_success() {
        static CALLS: Counter = Counter::new(0);

        with_attempts(3, 0.1, &|| {
            if CALLS.increment() < 3 {
                panic!("expected fail");
            }
        });

        assert_eq!(CALLS.current(), 3);
    }

    #[test]
    #[should_panic = "expected fail"]
    fn works_on_too_many_panics_before_success() {
        static CALLS: Counter = Counter::new(0);

        with_attempts(3, 0.1, &|| {
            if CALLS.increment() < 4 {
                panic!("expected fail");
            }
        });

        assert_eq!(CALLS.current(), 3);
    }

    #[test]
    #[should_panic = "expected fail"]
    fn works_on_unconditional_panic() {
        static CALLS: Counter = Counter::new(0);

        with_attempts(3, 0.1, &|| {
            CALLS.increment();
            panic!("expected fail");
        });

        assert_eq!(CALLS.current(), 3);
    }

    #[test]
    fn works_on_single_try_pass() {
        static CALLS: Counter = Counter::new(0);

        with_attempts(1, 0.1, &|| {
            CALLS.increment();
        });

        assert_eq!(CALLS.current(), 1);
    }

    #[test]
    #[should_panic = "expected fail"]
    fn works_on_single_try_panic() {
        static CALLS: Counter = Counter::new(0);

        with_attempts(1, 0.1, &|| {
            CALLS.increment();
            panic!("expected fail");
        });

        assert_eq!(CALLS.current(), 1);
    }
}
