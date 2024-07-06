use std::fmt::Debug;

use super::Arbitrary;
use super::Shrinker;

pub struct TestOptions {
    max_runs: usize,
    max_fails: usize,
    max_fail_reports: usize,
}

impl Default for TestOptions {
    fn default() -> Self {
        TestOptions {
            #[cfg(miri)]
            max_runs: 100,
            #[cfg(not(miri))]
            max_runs: 100000,
            max_fails: 100,
            max_fail_reports: 20,
        }
    }
}

// This needs to be monomorphic, so share everything by `Arbitrary`.

#[track_caller]
pub fn run<A: Arbitrary + Debug>(body: impl FnMut(&A) -> bool) {
    run_with_opts(TestOptions::default(), body)
}

#[track_caller]
pub fn run_with_opts<A: Arbitrary + Debug>(options: TestOptions, mut body: impl FnMut(&A) -> bool) {
    propcheck_inner(options, &mut body)
}

struct TestState<'a, A> {
    options: TestOptions,
    tests_run: usize,
    test_failures: usize,
    body: &'a mut dyn FnMut(&A) -> bool,
    arg_failures: Vec<A>,
}

impl<A: Debug> std::fmt::Display for TestState<'_, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} tests passed, {} tests failed.",
            self.tests_run - self.test_failures,
            self.test_failures
        )?;

        for arg in &self.arg_failures {
            write!(f, "\n[FAIL] {arg:?}")?;
        }

        Ok(())
    }
}

#[allow(clippy::print_stderr)]
#[track_caller]
fn propcheck_inner<A: Arbitrary + Debug>(options: TestOptions, body: &mut dyn FnMut(&A) -> bool) {
    crate::test_utils::init_logger();

    let mut state = TestState {
        options,
        body,
        arg_failures: Vec::new(),
        tests_run: 0,
        test_failures: 0,
    };

    while run_test(&mut state, &<A>::arbitrary()) {
        // Loop logic's in `run_test`
    }

    if state.test_failures != 0 {
        panic!("{state}");
    }
}

#[track_caller]
fn run_test<A: Arbitrary + Debug>(state: &mut TestState<A>, arg: &A) -> bool {
    if state.tests_run == state.options.max_runs || state.test_failures == state.options.max_fails {
        return false;
    }

    state.tests_run += 1;

    if (state.body)(arg) {
        return true;
    }

    state.test_failures += 1;

    if state.arg_failures.len() < state.options.max_fail_reports {
        state.arg_failures.push(arg.clone());
    }

    let mut shrinker = arg.shrink();
    while let Some(arg) = shrinker.next() {
        if !run_test(state, arg) {
            return false;
        }
    }

    true
}
