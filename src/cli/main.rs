#![allow(clippy::exit)]
#![allow(clippy::print_stderr)]

use crate::prelude::*;

use super::args::Args;
use crate::child::start_child;
use crate::cli::args::parse_args;
use crate::ffi::NormalizeErrno;
use crate::parent::start_parent;

fn fail(e: &dyn std::fmt::Display) {
    eprintln!("{e}");
    std::process::exit(1);
}

pub fn main() {
    // Wire up the logger that's used everywhere to just print everything to stdout/stderr.
    log::set_max_level(log::LevelFilter::Trace);
    // SAFETY: Obviously no threads have been spawned yet, so it's fine to not go through all the
    // ceremony of thread safety.
    if unsafe { log::set_logger_racy(&StderrLogger) }.is_err() {
        fail(&"Could not set logger.");
    }

    match parse_args(std::env::args_os()) {
        Err(e) => fail(&e),
        Ok(Args::Child(args)) => match start_child(args) {
            Ok(result) => std::process::exit(result.as_exit_code().as_raw()),
            Err(e) => fail(&NormalizeErrno(&e, None)),
        },
        Ok(Args::Parent(args)) => match start_parent(args) {
            Ok(result) => std::process::exit(result.as_exit_code().as_raw()),
            Err(e) => fail(&NormalizeErrno(&e, None)),
        },
    }
}

struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let time_duration = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());

        let time =
            time::OffsetDateTime::from_unix_timestamp_nanos(reinterpret_u128_i128(time_duration))
                .unwrap();

        // I don't care about year 10000. If this is really going to still be running by that
        // point, I'll just assume it wraps around.
        let mut prefix = *b"-0000-00-00T00:00:00.000";

        static DIGIT_PAIRS: [[u8; 2]; 100] = {
            let mut result = [[0, 0]; 100];
            let mut i = 0;
            while i < result.len() {
                result[i] = [
                    truncate_usize_u8(i / 10) + b'0',
                    truncate_usize_u8(i % 10) + b'0',
                ];
                i += 1;
            }
            result
        };

        // Ensure this actually gets lowered to a single 16-bit copy. For some reason, it doesn't
        // get optimized to that when using either the `copy_to_start` defined here or the
        // `prefix.copy_from_slice(...)`.
        #[inline(always)]
        fn copy_digit(prefix: &mut [u8; 24], index: usize, digit: u8) {
            let [a, b] = DIGIT_PAIRS[zero_extend_u8_usize(digit)];
            prefix[index] = a;
            prefix[index.wrapping_add(1)] = b;
        }

        let raw_year = time.year();
        let year = raw_year.abs();
        copy_digit(&mut prefix, 1, truncate_i32_u8(year / 100));
        copy_digit(&mut prefix, 3, truncate_i32_u8(year % 100));
        copy_digit(&mut prefix, 6, u8::from(time.month()));
        copy_digit(&mut prefix, 9, time.day());
        copy_digit(&mut prefix, 12, time.hour());
        copy_digit(&mut prefix, 15, time.minute());
        copy_digit(&mut prefix, 18, time.second());
        let millis = time.millisecond();
        copy_digit(&mut prefix, 21, truncate_u16_u8(millis / 10));
        prefix[23] = truncate_u16_u8(millis % 10).wrapping_add(b'0');

        let time_str =
            std::str::from_utf8(if raw_year < 0 { &prefix } else { &prefix[1..] }).unwrap();

        // It's okay to print everything to stderr. This doesn't log informative messages anyways.
        drop(io::stderr().write_fmt(format_args!("[{}Z] {}\n", time_str, record.args())));
    }

    fn flush(&self) {
        drop(io::stderr().flush());
    }
}
