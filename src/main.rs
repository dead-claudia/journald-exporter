// Clippy rules
#![deny(clippy::exit)]
#![deny(clippy::as_conversions)]
#![deny(clippy::assertions_on_result_states)]
#![deny(clippy::dbg_macro)]
#![warn(clippy::deref_by_slicing)]
#![warn(let_underscore_drop)]
#![warn(clippy::let_underscore_lock)]
#![warn(clippy::let_underscore_must_use)]
#![warn(clippy::lossy_float_literal)]
#![deny(clippy::mem_forget)]
#![deny(clippy::mutex_atomic)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::print_stdout)]
#![deny(clippy::print_stderr)]
#![deny(clippy::self_named_module_files)]
#![deny(clippy::string_add)]
#![deny(clippy::string_add_assign)]
#![deny(clippy::string_slice)]
#![deny(clippy::todo)]
#![deny(clippy::try_err)]
#![deny(clippy::unimplemented)]
// Enable once stabilized.
// #![deny(clippy::unnecessary_safety_comment)]
#![deny(clippy::unseparated_literal_suffix)]
#![cfg_attr(not(test), deny(clippy::use_debug))]
// Panicking in tests is okay.
#![cfg_attr(not(test), deny(clippy::arithmetic_side_effects))]

// Now for all the module specifications

mod child;
mod cli;
mod common;
mod ffi;
mod parent;
mod prelude;
mod state;
#[cfg(test)]
mod test_utils;

pub fn main() {
    cli::main::main()
}
