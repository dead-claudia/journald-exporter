#![allow(clippy::exit)]
#![allow(clippy::print_stderr)]

use crate::prelude::*;

use super::args::Args;
use crate::child::start_child;
use crate::cli::args::parse_args;
use crate::ffi::normalize_errno;
use crate::ffi::Signal;
use crate::parent::start_parent;

fn eprintln(msg: CowStr) {
    let mut msg = msg.into_owned().into_string().into_bytes();
    msg.push(b'\n');
    drop(io::stderr().write_all(&msg));
}

pub fn main() {
    // Kill this if the parent dies, and do it for both the parent and child processes. Easier than
    // trying to wire up the parent somehow while testing. Should also ensure this dies when its
    // parent dies, for the E2E tests (and in general when run via `systemd-run`).
    Signal::request_signal_when_parent_terminates(Signal::SIGTERM);

    // Wire up the logger that's used everywhere to just print everything to stdout/stderr.
    log::set_max_level(log::LevelFilter::Trace);
    // SAFETY: Obviously no threads have been spawned yet, so it's fine to not go through all the
    // ceremony of thread safety.
    if unsafe { log::set_logger_racy(&StderrLogger) }.is_err() {
        // Shouldn't ever happen.
        std::process::abort();
    }

    let args = match parse_args(std::env::args_os()) {
        Err(e) => {
            eprintln(e.as_str());
            std::process::exit(1)
        }
        Ok(args) => args,
    };

    let result = match args {
        Args::Child(args) => start_child(args),
        Args::Parent(args) => start_parent(args),
    };

    match result {
        Ok(result) => std::process::exit(result.as_exit_code().as_raw()),
        Err(e) => {
            eprintln(normalize_errno(e, None));
            std::process::exit(1);
        }
    }
}

struct StderrLogger;

impl log::Log for StderrLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        // It's okay to print everything to stderr. This doesn't log informative messages anyways.
        eprintln(CowStr::format(*record.args()));
    }

    fn flush(&self) {
        drop(io::stderr().flush());
    }
}
