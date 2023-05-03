use crate::cli::args::*;
use std::ffi::OsString;

fn parse_args(args: &[&str]) -> Result<Args, ArgsError> {
    crate::cli::args::parse_args(args.iter().map(OsString::from))
}

#[test]
fn no_argv0_or_first_arg_returns_show_help() {
    assert_eq!(parse_args(&[]), Err(ArgsError::ShowHelp));
}

#[test]
fn no_first_arg_returns_show_help() {
    assert_eq!(parse_args(&["journald-exporter"]), Err(ArgsError::ShowHelp));
}

#[test]
fn dash_question_arg_returns_show_help() {
    assert_eq!(
        parse_args(&["journald-exporter", "-?"]),
        Err(ArgsError::ShowHelp),
    );
}

#[test]
fn dash_h_arg_returns_show_help() {
    assert_eq!(
        parse_args(&["journald-exporter", "-h"]),
        Err(ArgsError::ShowHelp),
    );
}

#[test]
fn dash_help_arg_returns_show_help() {
    assert_eq!(
        parse_args(&["journald-exporter", "-help"]),
        Err(ArgsError::ShowHelp),
    );
}

#[test]
fn dash_dash_help_arg_returns_show_help() {
    assert_eq!(
        parse_args(&["journald-exporter", "--help"]),
        Err(ArgsError::ShowHelp),
    );
}

#[test]
fn dash_lower_v_arg_returns_show_version() {
    assert_eq!(
        parse_args(&["journald-exporter", "-v"]),
        Err(ArgsError::ShowVersion),
    );
}

#[test]
fn dash_upper_v_arg_returns_show_version() {
    assert_eq!(
        parse_args(&["journald-exporter", "-V"]),
        Err(ArgsError::ShowVersion),
    );
}

#[test]
fn dash_version_arg_returns_show_version() {
    assert_eq!(
        parse_args(&["journald-exporter", "-version"]),
        Err(ArgsError::ShowVersion),
    );
}

#[test]
fn dash_dash_version_arg_returns_show_version() {
    assert_eq!(
        parse_args(&["journald-exporter", "--version"]),
        Err(ArgsError::ShowVersion),
    );
}

#[test]
fn non_flag_first_arg_returns_unknown_flag() {
    assert_eq!(
        parse_args(&["journald-exporter", "abc"]),
        Err(ArgsError::UnknownFlag("abc".into()))
    );
}

#[test]
fn unknown_single_dash_first_arg_returns_unknown_flag() {
    assert_eq!(
        parse_args(&["journald-exporter", "-wut"]),
        Err(ArgsError::UnknownFlag("-wut".into())),
    );
}

#[test]
fn unknown_double_dash_first_arg_returns_unknown_flag() {
    assert_eq!(
        parse_args(&["journald-exporter", "--wut"]),
        Err(ArgsError::UnknownFlag("--wut".into())),
    );
}

#[test]
fn single_hyphen_returns_unknown_flag() {
    assert_eq!(
        parse_args(&["journald-exporter", "-"]),
        Err(ArgsError::UnknownFlag("-".into()))
    );
}

#[test]
fn double_hyphen_returns_unknown_flag() {
    assert_eq!(
        parse_args(&["journald-exporter", "--"]),
        Err(ArgsError::UnknownFlag("--".into()))
    );
}

#[test]
fn contains_child_process_returns_child() {
    assert_eq!(
        parse_args(&["journald-exporter", "--child-process"]),
        Ok(Args::Child)
    );
}
