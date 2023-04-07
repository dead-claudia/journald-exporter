use crate::prelude::*;

pub struct Counter {
    current: AtomicU64,
}

impl Counter {
    pub const fn new(initial: u64) -> Counter {
        Counter {
            current: AtomicU64::new(initial),
        }
    }

    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Acquire)
    }

    // Returns the current post-increment count for convenience.
    pub fn increment(&self) -> u64 {
        self.increment_by(1)
    }

    // Returns the current post-increment count for convenience.
    pub fn increment_by(&self, n: u64) -> u64 {
        // Why is this relaxed? See this page in the Boost documentation:
        // https://www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html
        self.current.fetch_add(n, Ordering::Relaxed).wrapping_add(n)
    }
}
