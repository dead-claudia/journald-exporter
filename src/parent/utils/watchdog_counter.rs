/// This accepts its top-level bound as a compile-time constant so it can inline it into the code
/// rather than wasting an entire register for it.
pub struct WatchdogCounter<const N: usize> {
    remaining: usize,
}

impl<const N: usize> WatchdogCounter<N> {
    const INITIAL: usize = N.wrapping_sub(1);

    pub const fn new() -> Self {
        Self {
            remaining: Self::INITIAL,
        }
    }

    pub fn hit(&mut self) -> bool {
        let result = self.remaining.checked_sub(1);
        self.remaining = result.unwrap_or(Self::INITIAL);
        result.is_none()
    }
}

#[cfg(test)]
mod tests {
    // Makes the tests a little easier to follow
    #![allow(clippy::bool_assert_comparison)]

    use super::*;

    #[test]
    fn watchdog_counter_works_once() {
        let mut counter = WatchdogCounter::<5>::new();
        for _ in 0..4 {
            assert_eq!(counter.hit(), false);
        }
        assert_eq!(counter.hit(), true);
    }

    #[test]
    fn watchdog_counter_wraps_around() {
        let mut counter = WatchdogCounter::<5>::new();
        for _ in 0..5 {
            counter.hit();
        }
        for _ in 0..4 {
            assert_eq!(counter.hit(), false);
        }
        assert_eq!(counter.hit(), true);
    }
}
