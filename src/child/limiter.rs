use std::net::IpAddr;
use std::net::Ipv6Addr;

// A simple once-per-second IP rate limiter. Concurrency is controlled externally.

// Since request rate is unlikely to be high, it just uses a vector.

#[derive(Debug)]
pub struct Limiter {
    second: u64,
    throttle_set: Option<Vec<Ipv6Addr>>,
}

impl Limiter {
    pub const fn new() -> Limiter {
        Limiter {
            second: 0,
            throttle_set: None,
        }
    }

    pub fn reap(&mut self, second: u64) {
        if self.second < second {
            self.second = second;
            self.throttle_set = None;
        }
    }

    pub fn check_throttled(&mut self, second: u64, key: IpAddr) -> bool {
        if self.second < second {
            self.second = second;
            self.throttle_set = None;
        }

        let throttle_set = self.throttle_set.get_or_insert_with(Vec::new);

        // Start from the end, as more recent entries are the most likely to repeat. Also will help
        // hide the fact this isn't constant-time.
        if throttle_set.iter().rev().any(|k| *k == key) {
            true
        } else {
            throttle_set.push(match key {
                IpAddr::V4(v4) => v4.to_ipv6_mapped(),
                IpAddr::V6(v6) => v6,
            });
            false
        }
    }
}

#[cfg(test)]
mod test {
    // Makes the tests a little easier to follow
    #![allow(clippy::bool_assert_comparison)]

    use super::*;

    const fn ip(low: u16) -> IpAddr {
        IpAddr::V6(Ipv6Addr::new(2001, 0, 0, 0, 0, 0, 0, low))
    }

    #[test]
    fn fails_to_throttle_on_one_call_per_second_with_one_key() {
        let mut limiter = Limiter::new();

        assert_eq!(limiter.check_throttled(1, ip(1111)), false);
    }

    #[test]
    fn throttles_on_two_calls_per_second_with_one_key() {
        let mut limiter = Limiter::new();

        limiter.check_throttled(1, ip(1111));
        assert_eq!(limiter.check_throttled(1, ip(1111)), true);
    }

    #[test]
    fn throttles_on_three_calls_per_second_with_one_key() {
        let mut limiter = Limiter::new();

        limiter.check_throttled(1, ip(1111));
        limiter.check_throttled(1, ip(1111));
        assert_eq!(limiter.check_throttled(1, ip(1111)), true);
    }

    #[test]
    fn fails_to_throttle_on_one_call_per_second_with_two_keys() {
        let mut limiter = Limiter::new();

        assert_eq!(limiter.check_throttled(1, ip(1111)), false);
        assert_eq!(limiter.check_throttled(1, ip(2222)), false);
    }

    #[test]
    fn throttles_on_two_calls_per_second_with_two_keys() {
        let mut limiter = Limiter::new();

        limiter.check_throttled(1, ip(1111));
        limiter.check_throttled(1, ip(2222));
        assert_eq!(limiter.check_throttled(1, ip(1111)), true);
        assert_eq!(limiter.check_throttled(1, ip(2222)), true);
    }

    #[test]
    fn throttles_on_three_calls_per_second_with_two_keys() {
        let mut limiter = Limiter::new();

        limiter.check_throttled(1, ip(1111));
        limiter.check_throttled(1, ip(2222));
        limiter.check_throttled(1, ip(1111));
        limiter.check_throttled(1, ip(2222));
        assert_eq!(limiter.check_throttled(1, ip(1111)), true);
        assert_eq!(limiter.check_throttled(1, ip(2222)), true);
    }

    #[test]
    fn after_reap_fails_to_throttle_on_one_call_per_second_with_one_key() {
        let mut limiter = Limiter::new();

        limiter.reap(1);
        assert_eq!(limiter.check_throttled(1, ip(1111)), false);
    }

    #[test]
    fn after_reap_throttles_on_two_calls_per_second_with_one_key() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        limiter.check_throttled(1, ip(1111));
        assert_eq!(limiter.check_throttled(1, ip(1111)), true);
    }

    #[test]
    fn after_reap_throttles_on_three_calls_per_second_with_one_key() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        limiter.check_throttled(1, ip(1111));
        limiter.check_throttled(1, ip(1111));
        assert_eq!(limiter.check_throttled(1, ip(1111)), true);
    }

    #[test]
    fn after_reap_fails_to_throttle_on_one_call_per_second_with_two_keys() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        assert_eq!(limiter.check_throttled(1, ip(1111)), false);
        assert_eq!(limiter.check_throttled(1, ip(2222)), false);
    }

    #[test]
    fn after_reap_throttles_on_two_calls_per_second_with_two_keys() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        limiter.check_throttled(1, ip(1111));
        limiter.check_throttled(1, ip(2222));
        assert_eq!(limiter.check_throttled(1, ip(1111)), true);
        assert_eq!(limiter.check_throttled(1, ip(2222)), true);
    }

    #[test]
    fn after_reap_throttles_on_three_calls_per_second_with_two_keys() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        limiter.check_throttled(1, ip(1111));
        limiter.check_throttled(1, ip(2222));
        limiter.check_throttled(1, ip(1111));
        limiter.check_throttled(1, ip(2222));
        assert_eq!(limiter.check_throttled(1, ip(1111)), true);
        assert_eq!(limiter.check_throttled(1, ip(2222)), true);
    }
}
