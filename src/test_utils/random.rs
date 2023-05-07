use crate::prelude::*;

use rand::RngCore;

pub fn sample_to(n: usize) -> usize {
    thread_local! {
        static RNG: std::cell::RefCell<rand::rngs::SmallRng> =
            std::cell::RefCell::new(rand::SeedableRng::from_entropy());
    }

    RNG.with(|rng| {
        let mut rng = rng.borrow_mut();

        // Ref: https://stackoverflow.com/a/10984975
        let max = usize::MAX - usize::MAX % n;
        loop {
            let x = truncate_u64_usize(rng.next_u64());
            if x < max {
                break x % n;
            }
        }
    })
}
