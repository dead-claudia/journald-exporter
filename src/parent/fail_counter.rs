use crate::prelude::*;

// Respawn always on failure, but only within reason. This utility aims to prevent that, and the
// constants below are the thresholds in which it refuses to restart.
const MAX_FAILS_PER_INTERVAL: usize = 5;
const FAIL_INTERVAL: Duration = Duration::from_millis(5000);

pub struct FailCounter {
    cursor: Wrapping<usize>,
    fails: [Option<Instant>; MAX_FAILS_PER_INTERVAL - 1],
}

impl FailCounter {
    pub const fn new() -> Self {
        Self {
            cursor: Wrapping(0),
            fails: [None; MAX_FAILS_PER_INTERVAL - 1],
        }
    }

    /// Returns `true` if it reached the maximum number of tolerable failures.
    pub fn check_fail(&mut self, next: Instant) -> bool {
        let cursor = self.cursor;
        if let Some(prev) = self.fails[cursor.0] {
            if next.saturating_duration_since(prev) <= FAIL_INTERVAL {
                return true;
            }
        }

        self.fails[cursor.0] = Some(next);
        self.cursor = (cursor + Wrapping(1)) % Wrapping(self.fails.len());
        false
    }
}

#[cfg(test)]
mod test {
    // Makes the tests a little easier to follow
    #![allow(clippy::bool_assert_comparison)]

    use super::*;

    fn after(i: Instant, ms: u64) -> Instant {
        i + Duration::from_millis(ms)
    }

    #[test]
    fn fail_counter_fails_on_exact() {
        let mut counter = FailCounter::new();
        let base = Instant::now();

        assert_eq!(counter.check_fail(after(base, 10000)), false);
        assert_eq!(counter.check_fail(after(base, 20000)), false);
        assert_eq!(counter.check_fail(after(base, 22000)), false);
        assert_eq!(counter.check_fail(after(base, 23000)), false);
        assert_eq!(counter.check_fail(after(base, 24000)), false);
        assert_eq!(counter.check_fail(after(base, 25000)), true);

        // Verify it unlocks after
        assert_eq!(counter.check_fail(after(base, 30000)), false);
        assert_eq!(counter.check_fail(after(base, 30001)), false);
        assert_eq!(counter.check_fail(after(base, 30002)), false);
        assert_eq!(counter.check_fail(after(base, 30003)), false);
        assert_eq!(counter.check_fail(after(base, 30004)), true);
    }

    #[test]
    fn fail_counter_fails_on_inexact() {
        let mut counter = FailCounter::new();
        let base = Instant::now();

        assert_eq!(counter.check_fail(after(base, 10000)), false);
        assert_eq!(counter.check_fail(after(base, 20000)), false);
        assert_eq!(counter.check_fail(after(base, 22000)), false);
        assert_eq!(counter.check_fail(after(base, 23000)), false);
        assert_eq!(counter.check_fail(after(base, 24000)), false);
        assert_eq!(counter.check_fail(after(base, 24999)), true);

        // Verify it unlocks after
        assert_eq!(counter.check_fail(after(base, 30000)), false);
        assert_eq!(counter.check_fail(after(base, 30001)), false);
        assert_eq!(counter.check_fail(after(base, 30002)), false);
        assert_eq!(counter.check_fail(after(base, 30003)), false);
        assert_eq!(counter.check_fail(after(base, 30004)), true);
    }

    #[test]
    fn fail_counter_fails_fast() {
        let mut counter = FailCounter::new();
        let base = Instant::now();

        assert_eq!(counter.check_fail(after(base, 10100)), false);
        assert_eq!(counter.check_fail(after(base, 10200)), false);
        assert_eq!(counter.check_fail(after(base, 10300)), false);
        assert_eq!(counter.check_fail(after(base, 10400)), false);
        assert_eq!(counter.check_fail(after(base, 10500)), true);

        // Verify it unlocks after
        assert_eq!(counter.check_fail(after(base, 20000)), false);
        assert_eq!(counter.check_fail(after(base, 20001)), false);
        assert_eq!(counter.check_fail(after(base, 20002)), false);
        assert_eq!(counter.check_fail(after(base, 20003)), false);
        assert_eq!(counter.check_fail(after(base, 20004)), true);
    }
}
