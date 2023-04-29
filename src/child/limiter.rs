use std::net::IpAddr;
use std::net::Ipv6Addr;

// A simple once-per-second IP rate limiter. Concurrency is controlled externally.

// Since request rate is unlikely to be high, it just uses a vector.

#[derive(Debug)]
pub struct Limiter {
    second: u64,
    throttle_set: Vec<Ipv6Addr>,
}

impl Limiter {
    pub const fn new() -> Limiter {
        Limiter {
            second: 0,
            throttle_set: Vec::new(),
        }
    }

    pub fn reap(&mut self, second: u64) {
        if self.second < second {
            self.second = second;
            self.throttle_set = Vec::new();
        }
    }

    pub fn check_throttled(&mut self, second: u64, key: IpAddr) -> bool {
        self.reap(second);

        // Start from the end, as more recent entries are the most likely to repeat. Also will help
        // hide the fact this isn't constant-time.
        if self.throttle_set.iter().rev().any(|k| *k == key) {
            true
        } else {
            self.throttle_set.push(match key {
                IpAddr::V4(v4) => v4.to_ipv6_mapped(),
                IpAddr::V6(v6) => v6,
            });
            false
        }
    }
}

#[cfg(test)]
mod tests {
    // Makes the tests a little easier to follow
    #![allow(clippy::bool_assert_comparison)]

    use super::*;
    use const_str::ip_addr;

    #[test]
    fn fails_to_throttle_on_one_call_per_second_with_one_key() {
        let mut limiter = Limiter::new();

        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), false,);
    }

    #[test]
    fn throttles_on_two_calls_per_second_with_one_key() {
        let mut limiter = Limiter::new();

        limiter.check_throttled(1, ip_addr!("2001::1111"));
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), true);
    }

    #[test]
    fn throttles_on_three_calls_per_second_with_one_key() {
        let mut limiter = Limiter::new();

        limiter.check_throttled(1, ip_addr!("2001::1111"));
        limiter.check_throttled(1, ip_addr!("2001::1111"));
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), true);
    }

    #[test]
    fn fails_to_throttle_on_one_call_per_second_with_two_keys() {
        let mut limiter = Limiter::new();

        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), false,);
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::2222")), false,);
    }

    #[test]
    fn throttles_on_two_calls_per_second_with_two_keys() {
        let mut limiter = Limiter::new();

        limiter.check_throttled(1, ip_addr!("2001::1111"));
        limiter.check_throttled(1, ip_addr!("2001::2222"));
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), true);
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::2222")), true);
    }

    #[test]
    fn throttles_on_three_calls_per_second_with_two_keys() {
        let mut limiter = Limiter::new();

        limiter.check_throttled(1, ip_addr!("2001::1111"));
        limiter.check_throttled(1, ip_addr!("2001::2222"));
        limiter.check_throttled(1, ip_addr!("2001::1111"));
        limiter.check_throttled(1, ip_addr!("2001::2222"));
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), true);
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::2222")), true);
    }

    #[test]
    fn after_reap_fails_to_throttle_on_one_call_per_second_with_one_key() {
        let mut limiter = Limiter::new();

        limiter.reap(1);
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), false,);
    }

    #[test]
    fn after_reap_throttles_on_two_calls_per_second_with_one_key() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        limiter.check_throttled(1, ip_addr!("2001::1111"));
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), true);
    }

    #[test]
    fn after_reap_throttles_on_three_calls_per_second_with_one_key() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        limiter.check_throttled(1, ip_addr!("2001::1111"));
        limiter.check_throttled(1, ip_addr!("2001::1111"));
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), true);
    }

    #[test]
    fn after_reap_fails_to_throttle_on_one_call_per_second_with_two_keys() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), false,);
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::2222")), false,);
    }

    #[test]
    fn after_reap_throttles_on_two_calls_per_second_with_two_keys() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        limiter.check_throttled(1, ip_addr!("2001::1111"));
        limiter.check_throttled(1, ip_addr!("2001::2222"));
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), true);
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::2222")), true);
    }

    #[test]
    fn after_reap_throttles_on_three_calls_per_second_with_two_keys() {
        let mut limiter = Limiter::new();
        limiter.reap(1);

        limiter.check_throttled(1, ip_addr!("2001::1111"));
        limiter.check_throttled(1, ip_addr!("2001::2222"));
        limiter.check_throttled(1, ip_addr!("2001::1111"));
        limiter.check_throttled(1, ip_addr!("2001::2222"));
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::1111")), true);
        assert_eq!(limiter.check_throttled(1, ip_addr!("2001::2222")), true);
    }
}
