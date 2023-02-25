pub mod child;
pub mod common;
pub mod parent;

mod child_flags;
#[cfg(test)]
mod child_tests;
#[cfg(test)]
mod parent_tests;

pub const VERSION: u32 = 0;
pub static VERSION_BYTES: [u8; 4] = VERSION.to_le_bytes();
