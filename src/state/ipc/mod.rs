pub mod child;
pub mod common;
pub mod parent;

#[cfg(test)]
mod child_tests;
#[cfg(test)]
mod parent_tests;
#[cfg(test)]
mod read_phase_tests;

pub const VERSION: u32 = 0;
pub static VERSION_BYTES: [u8; 4] = VERSION.to_le_bytes();
