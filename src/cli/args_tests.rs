use super::args::*;
use std::ffi::OsString;
use std::num::NonZeroU16;
use std::path::PathBuf;

fn parse_args(args: &[&str]) -> Result<Args, ArgsError> {
    super::args::parse_args(args.iter().map(OsString::from))
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
fn help_first_arg_returns_show_help() {
    assert_eq!(
        parse_args(&["journald-exporter", "--help"]),
        Err(ArgsError::ShowHelp),
        "--help"
    );
    assert_eq!(
        parse_args(&["journald-exporter", "-help"]),
        Err(ArgsError::ShowHelp),
        "-help"
    );
    assert_eq!(
        parse_args(&["journald-exporter", "-h"]),
        Err(ArgsError::ShowHelp),
        "-h"
    );
    assert_eq!(
        parse_args(&["journald-exporter", "-?"]),
        Err(ArgsError::ShowHelp),
        "-?"
    );
}

#[test]
fn version_first_arg_returns_show_version() {
    assert_eq!(
        parse_args(&["journald-exporter", "--version"]),
        Err(ArgsError::ShowVersion),
        "--version"
    );
    assert_eq!(
        parse_args(&["journald-exporter", "-version"]),
        Err(ArgsError::ShowVersion),
        "-version"
    );
    assert_eq!(
        parse_args(&["journald-exporter", "-v"]),
        Err(ArgsError::ShowVersion),
        "-v"
    );
    assert_eq!(
        parse_args(&["journald-exporter", "-V"]),
        Err(ArgsError::ShowVersion),
        "-V"
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
fn unknown_first_arg_returns_unknown_flag() {
    assert_eq!(
        parse_args(&["journald-exporter", "-wut"]),
        Err(ArgsError::UnknownFlag("-wut".into())),
        "-wut"
    );
    assert_eq!(
        parse_args(&["journald-exporter", "--wut"]),
        Err(ArgsError::UnknownFlag("--wut".into())),
        "--wut"
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
fn port_start_returns_missing_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p]),
            Err(ArgsError::MissingPort),
            "{p}"
        );
    }
}

#[test]
fn key_dir_start_returns_missing_key_dir() {
    for k in ["-k", "--key-dir"] {
        assert_eq!(
            parse_args(&["journald-exporter", k]),
            Err(ArgsError::MissingKeyDir),
            "{k}"
        );
    }
}

#[test]
fn non_numeric_port_number_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "abc"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn partially_numeric_port_number_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "abc123"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn hex_port_number_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "0x123"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn negative_port_number_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "-123"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn negative_zero_port_number_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "-0"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn plus_port_number_for_parent_returns_missing_key_dir() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "+123"]),
            Err(ArgsError::MissingKeyDir),
            "{p}"
        );
    }
}

#[test]
fn unsigned_port_number_for_parent_returns_missing_key_dir() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "123"]),
            Err(ArgsError::MissingKeyDir),
            "{p}"
        );
    }
}

#[test]
fn contains_child_process_returns_child() {
    assert_eq!(
        parse_args(&["journald-exporter", "--child-process"]),
        Ok(Args::Child)
    );
}

#[test]
fn plus_port_number_then_key_dir_flag_returns_missing_key_dir() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", p, "+123", k]),
                Err(ArgsError::MissingKeyDir),
                "{p}"
            );
        }
    }
}

#[test]
fn plus_zero_port_number_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "+0"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn unsigned_zero_port_number_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "0"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn unsigned_port_number_out_of_range_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "100000"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn unsigned_port_number_way_out_of_range_returns_invalid_port() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "999999999999999999999999"]),
            Err(ArgsError::InvalidPort),
            "{p}"
        );
    }
}

#[test]
fn plus_port_then_unknown_second_arg_fails_with_unknown_arg() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "+123", "bad argument"]),
            Err(ArgsError::UnknownFlag("bad argument".into())),
            "{p}"
        );
    }
}

#[test]
fn unsigned_port_then_unknown_second_arg_fails_with_unknown_arg() {
    for p in ["-p", "--port"] {
        assert_eq!(
            parse_args(&["journald-exporter", p, "123", "bad argument"]),
            Err(ArgsError::UnknownFlag("bad argument".into())),
            "{p}"
        );
    }
}

#[test]
fn empty_key_dir_arg_without_port_returns_empty_key_dir() {
    for k in ["-k", "--key-dir"] {
        assert_eq!(
            parse_args(&["journald-exporter", k, ""]),
            Err(ArgsError::EmptyKeyDir),
            "{k}"
        );
    }
}

#[test]
fn key_dir_arg_ending_in_colon_without_port_returns_missing_port() {
    for k in ["-k", "--key-dir"] {
        assert_eq!(
            parse_args(&["journald-exporter", k, "blah:"]),
            Err(ArgsError::MissingPort),
            "{k}"
        );
    }
}

#[test]
fn key_dir_arg_with_special_chars_and_no_port_returns_missing_port() {
    for k in ["-k", "--key-dir"] {
        assert_eq!(
            parse_args(&["journald-exporter", k, "b/l@a!h:"]),
            Err(ArgsError::MissingPort),
            "{k}"
        );
    }
}

#[test]
fn key_dir_arg_ending_in_normal_key_dir_path_without_port_returns_missing_port() {
    for k in ["-k", "--key-dir"] {
        assert_eq!(
            parse_args(&["journald-exporter", k, "some/file"]),
            Err(ArgsError::MissingPort),
            "{k}"
        );
    }
}

#[test]
fn key_dir_arg_then_port_flag_returns_missing_port() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", k, "some/file", p]),
                Err(ArgsError::MissingPort),
                "{p}"
            );
        }
    }
}

#[test]
fn plus_port_number_key_dir_ending_in_colon_returns_success() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", p, "+123", k, "blah:"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("blah:"),
                    tls: None,
                })),
                "{p} then {k}"
            );
            assert_eq!(
                parse_args(&["journald-exporter", k, "blah:", p, "+123"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("blah:"),
                    tls: None,
                })),
                "{k} then {p}"
            );
        }
    }
}

#[test]
fn plus_port_number_empty_key_dir_arg_returns_empty_key_dir() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", p, "+123", k, ""]),
                Err(ArgsError::EmptyKeyDir),
                "{p} then {k}"
            );
            assert_eq!(
                parse_args(&["journald-exporter", k, "", p, "+123"]),
                Err(ArgsError::EmptyKeyDir),
                "{k} then {p}"
            );
        }
    }
}

#[test]
fn plus_port_number_with_file_ending_in_colon_returns_success() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", p, "+123", k, "blah:"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("blah:"),
                    tls: None,
                })),
                "{p} then {k}"
            );
            assert_eq!(
                parse_args(&["journald-exporter", k, "blah:", p, "+123"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("blah:"),
                    tls: None,
                })),
                "{k} then {p}"
            );
        }
    }
}

#[test]
fn plus_port_number_with_file_with_special_chars_returns_success() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", p, "+123", k, "b/l@a!h:"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("b/l@a!h:"),
                    tls: None,
                })),
                "{p} then {k}"
            );
            assert_eq!(
                parse_args(&["journald-exporter", k, "b/l@a!h:", p, "+123"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("b/l@a!h:"),
                    tls: None,
                })),
                "{k} then {p}"
            );
        }
    }
}

#[test]
fn plus_port_number_with_normal_key_dir_path_returns_parent() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", p, "+123", k, "some/file"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("some/file"),
                    tls: None,
                })),
                "{p} then {k}"
            );
            assert_eq!(
                parse_args(&["journald-exporter", k, "some/file", p, "+123"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("some/file"),
                    tls: None,
                })),
                "{k} then {p}"
            );
        }
    }
}

#[test]
fn unsigned_non_zero_port_number_returns_parent() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", p, "123", k, "some/file"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("some/file"),
                    tls: None,
                })),
                "{p} then {k}"
            );
            assert_eq!(
                parse_args(&["journald-exporter", k, "some/file", p, "123"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(123).unwrap(),
                    key_dir: PathBuf::from("some/file"),
                    tls: None,
                })),
                "{k} then {p}"
            );
        }
    }
}

#[test]
fn unsigned_16_bit_port_number_returns_parent() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&["journald-exporter", p, "12345", k, "some/file"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(12345).unwrap(),
                    key_dir: PathBuf::from("some/file"),
                    tls: None,
                })),
                "{p} then {k}"
            );
            assert_eq!(
                parse_args(&["journald-exporter", k, "some/file", p, "12345"]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(12345).unwrap(),
                    key_dir: PathBuf::from("some/file"),
                    tls: None,
                })),
                "{k} then {p}"
            );
        }
    }
}

#[test]
fn plus_port_number_with_file_and_unknown_third_arg_fails_with_unknown_flag() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    p,
                    "+123",
                    k,
                    "some/file",
                    "bad argument"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg last"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    k,
                    "some/file",
                    p,
                    "+123",
                    "bad argument"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg last"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    p,
                    "+123",
                    "bad argument",
                    k,
                    "some/file"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg middle"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    k,
                    "some/file",
                    "bad argument",
                    p,
                    "+123"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg middle"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    "bad argument",
                    p,
                    "+123",
                    k,
                    "some/file"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg first"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    "bad argument",
                    k,
                    "some/file",
                    p,
                    "+123"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg first"
            );
        }
    }
}

#[test]
fn unsigned_port_number_with_file_and_unknown_third_arg_fails_with_unknown_flag() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    p,
                    "123",
                    k,
                    "some/file",
                    "bad argument"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg last"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    k,
                    "some/file",
                    p,
                    "123",
                    "bad argument"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg last"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    p,
                    "123",
                    "bad argument",
                    k,
                    "some/file"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg middle"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    k,
                    "some/file",
                    "bad argument",
                    p,
                    "123"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg middle"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    "bad argument",
                    p,
                    "123",
                    k,
                    "some/file"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg first"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    "bad argument",
                    k,
                    "some/file",
                    p,
                    "123"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg first"
            );
        }
    }
}

#[test]
fn unsigned_16_bit_port_number_with_file_and_unknown_third_arg_fails_with_unknown_flag() {
    for p in ["-p", "--port"] {
        for k in ["-k", "--key-dir"] {
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    p,
                    "12345",
                    k,
                    "some/file",
                    "bad argument"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg last"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    k,
                    "some/file",
                    p,
                    "12345",
                    "bad argument"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg last"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    p,
                    "12345",
                    "bad argument",
                    k,
                    "some/file"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg middle"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    k,
                    "some/file",
                    "bad argument",
                    p,
                    "12345"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg middle"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    "bad argument",
                    p,
                    "12345",
                    k,
                    "some/file"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{p} then {k}, bad arg first"
            );
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    "bad argument",
                    k,
                    "some/file",
                    p,
                    "12345"
                ]),
                Err(ArgsError::UnknownFlag("bad argument".into())),
                "{k} then {p}, bad arg first"
            );
        }
    }
}

#[test]
fn certificate_arg_requires_value() {
    for c in ["-c", "--certificate"] {
        assert_eq!(
            parse_args(&[
                "journald-exporter",
                "--port",
                "12345",
                "--key-dir",
                "some/file",
                c,
            ]),
            Err(ArgsError::MissingCertificate),
            "{c}"
        );
    }
}

#[test]
fn certificate_arg_rejects_empty_value() {
    for c in ["-c", "--certificate"] {
        assert_eq!(
            parse_args(&[
                "journald-exporter",
                "--port",
                "12345",
                "--key-dir",
                "some/file",
                c,
                "",
            ]),
            Err(ArgsError::EmptyCertificate),
            "{c}"
        );
    }
}

#[test]
fn certificate_arg_requires_private_key_arg() {
    for c in ["-c", "--certificate"] {
        assert_eq!(
            parse_args(&[
                "journald-exporter",
                "--port",
                "12345",
                "--key-dir",
                "some/file",
                c,
                "some/cert.pem",
            ]),
            Err(ArgsError::MissingPrivateKey),
            "{c}"
        );
    }
}

#[test]
fn private_key_arg_requires_value() {
    for p in ["-P", "--private-key"] {
        assert_eq!(
            parse_args(&[
                "journald-exporter",
                "--port",
                "12345",
                "--key-dir",
                "some/file",
                p,
            ]),
            Err(ArgsError::MissingPrivateKey),
            "{p}"
        );
    }
}

#[test]
fn private_key_arg_rejects_empty_value() {
    for p in ["-P", "--private-key"] {
        assert_eq!(
            parse_args(&[
                "journald-exporter",
                "--port",
                "12345",
                "--key-dir",
                "some/file",
                p,
                "",
            ]),
            Err(ArgsError::EmptyPrivateKey),
            "{p}"
        );
    }
}

#[test]
fn private_key_arg_requires_certificate_arg() {
    for p in ["-P", "--private-key"] {
        assert_eq!(
            parse_args(&[
                "journald-exporter",
                "--port",
                "12345",
                "--key-dir",
                "some/file",
                p,
                "some/key.pem",
            ]),
            Err(ArgsError::MissingCertificate),
            "{p}"
        );
    }
}

#[test]
fn certificate_arg_and_private_key_arg_returns_parent() {
    for c in ["-c", "--certificate"] {
        for p in ["-P", "--private-key"] {
            assert_eq!(
                parse_args(&[
                    "journald-exporter",
                    "--port",
                    "12345",
                    "--key-dir",
                    "some/file",
                    c,
                    "some/cert.pem",
                    p,
                    "some/key.pem",
                ]),
                Ok(Args::Parent(ParentArgs {
                    port: NonZeroU16::new(12345).unwrap(),
                    key_dir: PathBuf::from("some/file"),
                    tls: Some(TLSOptions {
                        certificate: PathBuf::from("some/cert.pem"),
                        private_key: PathBuf::from("some/key.pem"),
                    })
                })),
                "{p}"
            );
        }
    }
}
