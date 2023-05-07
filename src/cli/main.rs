#![allow(clippy::exit)]
#![allow(clippy::print_stderr)]

use crate::prelude::*;

use super::args::Args;
use super::args::ParentArgs;
use super::config::parse_config;
use super::config::ConfigError;
use super::config::ParentConfig;
use crate::child::start_child;
use crate::cli::args::parse_args;
use crate::ffi::normalize_errno;
use crate::ffi::request_signal_when_parent_terminates;
use crate::ffi::ExitCode;
use crate::ffi::ExitResult;
use crate::ffi::Signal;
use crate::parent::start_parent;
use std::path::PathBuf;

fn eprintln(msg: Cow<str>) {
    let mut msg = msg.into_owned().into_bytes();
    msg.push(b'\n');
    drop(io::stderr().write_all(&msg));
}

pub fn main() {
    // Kill this if the parent dies, and do it for both the parent and child processes. Easier than
    // trying to wire up the parent somehow while testing. Should also ensure this dies when its
    // parent dies, for the E2E tests (and in general when run via `systemd-run`).
    request_signal_when_parent_terminates(Signal::SIGTERM);

    // Wire up the logger that's used everywhere to just print everything to stdout/stderr.
    log::set_max_level(log::LevelFilter::Info);
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
        Args::Child => start_child(),
        Args::Parent(args) => start_parent_using_flags(args),
        Args::ParentConfig(config) => start_parent_using_config(config),
        Args::Check(config) => check_config(config),
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
        eprintln(format_cow(*record.args()));
    }

    fn flush(&self) {
        drop(io::stderr().flush());
    }
}

fn check_config(config: PathBuf) -> io::Result<ExitResult> {
    // Just load the config. Errors will be printed out as applicable.
    load_config(config)?;
    Ok(ExitResult::Code(ExitCode(0)))
}

fn load_config(config: PathBuf) -> io::Result<ParentConfig> {
    let data = std::fs::read(&config)?;

    let e = match parse_config(&data) {
        Ok(args) => return Ok(args),
        Err(e) => e,
    };

    eprintln(format_cow(format_args!(
        "Errors were encountered while reading {}",
        config.display()
    )));

    match e {
        ConfigError::InvalidUTF8(e) => eprintln(format_cow(format_args!("{e}"))),
        ConfigError::InvalidSyntax(e) => eprintln(format_cow(format_args!("{e}"))),
        ConfigError::InvalidFields(errors) => {
            for e in errors.iter() {
                eprintln(e.as_str());
            }
        }
    }

    std::process::exit(1);
}

fn start_parent_using_config(config: PathBuf) -> io::Result<ExitResult> {
    crate::ffi::check_parent_uid_gid()?;
    start_parent(load_config(config)?)
}

fn start_parent_using_flags(args: ParentArgs) -> io::Result<ExitResult> {
    crate::ffi::check_parent_uid_gid()?;
    start_parent(ParentConfig::from_args(args))
}
