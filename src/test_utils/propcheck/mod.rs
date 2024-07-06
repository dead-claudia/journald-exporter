//! A simplified port of quickcheck to this. I'm tired of dealing with its limitations, both around
//! performance (it's dog slow) and just general random generation and such.

mod arbitrary;
mod runner;
mod tests;

pub use arbitrary::*;
pub use runner::run;
