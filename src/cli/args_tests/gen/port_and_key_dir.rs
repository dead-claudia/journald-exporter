// WARNING: This file is auto-generated by `scripts/gen-args-tests.js`. Do not modify directly.

use crate::cli::args::*;

fn parse_args(args: &[&str]) -> Result<Args, ArgsError> {
    crate::cli::args::parse_args(args.iter().map(std::ffi::OsString::from))
}

#[test]
fn short_port_then_short_key_dir_start_returns_missing_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "-k"]),
        Err(ArgsError::MissingKeyDir),
    );
}

#[test]
fn short_port_then_long_key_dir_start_returns_missing_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "--key-dir"]),
        Err(ArgsError::MissingKeyDir),
    );
}

#[test]
fn short_eq_port_then_short_key_dir_start_returns_missing_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "-k"]),
        Err(ArgsError::MissingKeyDir),
    );
}

#[test]
fn short_eq_port_then_long_key_dir_start_returns_missing_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "--key-dir"]),
        Err(ArgsError::MissingKeyDir),
    );
}

#[test]
fn long_port_then_short_key_dir_start_returns_missing_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "-k"]),
        Err(ArgsError::MissingKeyDir),
    );
}

#[test]
fn long_port_then_long_key_dir_start_returns_missing_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "--key-dir"]),
        Err(ArgsError::MissingKeyDir),
    );
}

#[test]
fn long_eq_port_then_short_key_dir_start_returns_missing_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "-k"]),
        Err(ArgsError::MissingKeyDir),
    );
}

#[test]
fn long_eq_port_then_long_key_dir_start_returns_missing_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "--key-dir"]),
        Err(ArgsError::MissingKeyDir),
    );
}

#[test]
fn short_key_dir_then_short_port_start_returns_missing_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "-p"]),
        Err(ArgsError::MissingPort),
    );
}

#[test]
fn short_eq_key_dir_then_short_port_start_returns_missing_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "-p"]),
        Err(ArgsError::MissingPort),
    );
}

#[test]
fn long_key_dir_then_short_port_start_returns_missing_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "-p"]),
        Err(ArgsError::MissingPort),
    );
}

#[test]
fn long_eq_key_dir_then_short_port_start_returns_missing_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "-p"]),
        Err(ArgsError::MissingPort),
    );
}

#[test]
fn short_key_dir_then_long_port_start_returns_missing_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "--port"]),
        Err(ArgsError::MissingPort),
    );
}

#[test]
fn short_eq_key_dir_then_long_port_start_returns_missing_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "--port"]),
        Err(ArgsError::MissingPort),
    );
}

#[test]
fn long_key_dir_then_long_port_start_returns_missing_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "--port"]),
        Err(ArgsError::MissingPort),
    );
}

#[test]
fn long_eq_key_dir_then_long_port_start_returns_missing_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "--port"]),
        Err(ArgsError::MissingPort),
    );
}

#[test]
fn short_port_then_short_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "-k", ""]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_port_then_short_eq_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "-k="]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_port_then_long_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "--key-dir", ""]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_port_then_long_eq_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "--key-dir="]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_eq_port_then_short_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "-k", ""]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_eq_port_then_short_eq_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "-k="]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_eq_port_then_long_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "--key-dir", ""]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_eq_port_then_long_eq_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "--key-dir="]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_port_then_short_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "-k", ""]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_port_then_short_eq_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "-k="]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_port_then_long_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "--key-dir", ""]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_port_then_long_eq_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "--key-dir="]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_eq_port_then_short_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "-k", ""]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_eq_port_then_short_eq_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "-k="]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_eq_port_then_long_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "--key-dir", ""]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_eq_port_then_long_eq_empty_key_dir_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "--key-dir="]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_empty_key_dir_then_short_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "", "-p", "123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_eq_empty_key_dir_then_short_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=", "-p", "123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_empty_key_dir_then_short_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "", "-p", "123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_eq_empty_key_dir_then_short_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=", "-p", "123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_empty_key_dir_then_short_eq_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "", "-p=123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_eq_empty_key_dir_then_short_eq_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=", "-p=123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_empty_key_dir_then_short_eq_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "", "-p=123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_eq_empty_key_dir_then_short_eq_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=", "-p=123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_empty_key_dir_then_long_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "", "--port", "123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_eq_empty_key_dir_then_long_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=", "--port", "123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_empty_key_dir_then_long_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "", "--port", "123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_eq_empty_key_dir_then_long_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=", "--port", "123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_empty_key_dir_then_long_eq_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "", "--port=123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_eq_empty_key_dir_then_long_eq_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=", "--port=123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_empty_key_dir_then_long_eq_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "", "--port=123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn long_eq_empty_key_dir_then_long_eq_port_returns_empty_key_dir() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=", "--port=123"]),
        Err(ArgsError::EmptyKeyDir),
    );
}

#[test]
fn short_empty_port_then_short_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "", "-k", "some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_empty_port_then_short_eq_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "", "-k=some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_empty_port_then_long_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "", "--key-dir", "some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_empty_port_then_long_eq_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "", "--key-dir=some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_eq_empty_port_then_short_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=", "-k", "some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_eq_empty_port_then_short_eq_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=", "-k=some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_eq_empty_port_then_long_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=", "--key-dir", "some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_eq_empty_port_then_long_eq_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=", "--key-dir=some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_empty_port_then_short_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "", "-k", "some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_empty_port_then_short_eq_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "", "-k=some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_empty_port_then_long_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "", "--key-dir", "some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_empty_port_then_long_eq_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "", "--key-dir=some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_eq_empty_port_then_short_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=", "-k", "some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_eq_empty_port_then_short_eq_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=", "-k=some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_eq_empty_port_then_long_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=", "--key-dir", "some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_eq_empty_port_then_long_eq_key_dir_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=", "--key-dir=some/dir"]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_key_dir_then_short_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "-p", ""]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_eq_key_dir_then_short_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "-p", ""]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_key_dir_then_short_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "-p", ""]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_eq_key_dir_then_short_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "-p", ""]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_key_dir_then_short_eq_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "-p="]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_eq_key_dir_then_short_eq_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "-p="]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_key_dir_then_short_eq_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "-p="]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_eq_key_dir_then_short_eq_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "-p="]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_key_dir_then_long_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "--port", ""]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_eq_key_dir_then_long_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "--port", ""]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_key_dir_then_long_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "--port", ""]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_eq_key_dir_then_long_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "--port", ""]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_key_dir_then_long_eq_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "--port="]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_eq_key_dir_then_long_eq_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "--port="]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_key_dir_then_long_eq_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "--port="]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn long_eq_key_dir_then_long_eq_empty_port_returns_invalid_port() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "--port="]),
        Err(ArgsError::InvalidPort),
    );
}

#[test]
fn short_port_then_short_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "-k", "some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_port_then_short_eq_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "-k=some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_port_then_long_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "--key-dir", "some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_port_then_long_eq_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p", "123", "--key-dir=some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_eq_port_then_short_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "-k", "some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_eq_port_then_short_eq_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "-k=some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_eq_port_then_long_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "--key-dir", "some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_eq_port_then_long_eq_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-p=123", "--key-dir=some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_port_then_short_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "-k", "some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_port_then_short_eq_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "-k=some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_port_then_long_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&[
            "journald-exporter",
            "--port",
            "123",
            "--key-dir",
            "some/dir"
        ]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_port_then_long_eq_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port", "123", "--key-dir=some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_eq_port_then_short_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "-k", "some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_eq_port_then_short_eq_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "-k=some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_eq_port_then_long_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "--key-dir", "some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_eq_port_then_long_eq_normal_key_dir_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--port=123", "--key-dir=some/dir"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_normal_key_dir_then_short_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "-p", "123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_eq_normal_key_dir_then_short_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "-p", "123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_normal_key_dir_then_short_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "-p", "123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_eq_normal_key_dir_then_short_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "-p", "123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_normal_key_dir_then_short_eq_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "-p=123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_eq_normal_key_dir_then_short_eq_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "-p=123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_normal_key_dir_then_short_eq_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "-p=123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_eq_normal_key_dir_then_short_eq_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "-p=123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_normal_key_dir_then_long_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "--port", "123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_eq_normal_key_dir_then_long_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "--port", "123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_normal_key_dir_then_long_port_returns_success() {
    assert_eq!(
        parse_args(&[
            "journald-exporter",
            "--key-dir",
            "some/dir",
            "--port",
            "123"
        ]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_eq_normal_key_dir_then_long_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "--port", "123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_normal_key_dir_then_long_eq_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k", "some/dir", "--port=123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn short_eq_normal_key_dir_then_long_eq_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "-k=some/dir", "--port=123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_normal_key_dir_then_long_eq_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir", "some/dir", "--port=123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}

#[test]
fn long_eq_normal_key_dir_then_long_eq_port_returns_success() {
    assert_eq!(
        parse_args(&["journald-exporter", "--key-dir=some/dir", "--port=123"]),
        Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        })),
    );
}
