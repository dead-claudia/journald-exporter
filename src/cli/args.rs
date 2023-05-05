use crate::prelude::*;

use std::ffi::OsString;
use std::num::NonZeroU16;
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct TLSOptions {
    pub certificate: PathBuf,
    pub private_key: PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct ChildArgs {
    pub port: NonZeroU16,
}

#[derive(Debug, PartialEq)]
pub struct ParentArgs {
    pub port: NonZeroU16,
    pub key_dir: PathBuf,
    pub tls: Option<TLSOptions>,
}

#[derive(Debug, PartialEq)]
pub enum Args {
    Child,
    Parent(ParentArgs),
    ParentConfig(PathBuf),
    Check(PathBuf),
}

#[derive(Debug, PartialEq)]
pub enum ArgsError {
    ShowHelp,
    ShowVersion,
    MissingPort,
    InvalidPort,
    MissingKeyDir,
    EmptyKeyDir,
    MissingCertificate,
    EmptyCertificate,
    MissingPrivateKey,
    EmptyPrivateKey,
    MissingConfig,
    EmptyConfig,
    UnknownFlag(OsString),
}

impl ArgsError {
    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            ArgsError::ShowHelp => Cow::Borrowed(super::help::HELP_STRING),
            ArgsError::ShowVersion => Cow::Borrowed(super::help::VERSION_STRING),
            ArgsError::MissingPort => Cow::Borrowed("A port is required."),
            ArgsError::InvalidPort => Cow::Borrowed("Port is invalid."),
            ArgsError::MissingKeyDir => Cow::Borrowed("Key directory missing."),
            ArgsError::EmptyKeyDir => Cow::Borrowed("Key directory cannot be empty."),
            ArgsError::MissingCertificate => Cow::Borrowed("Certificate file missing."),
            ArgsError::EmptyCertificate => Cow::Borrowed("Certificate file cannot be empty."),
            ArgsError::MissingPrivateKey => Cow::Borrowed("Private key file missing."),
            ArgsError::EmptyPrivateKey => Cow::Borrowed("Private key file cannot be empty."),
            ArgsError::MissingConfig => Cow::Borrowed("Config file missing."),
            ArgsError::EmptyConfig => Cow::Borrowed("Config file cannot be empty."),
            ArgsError::UnknownFlag(option) => {
                let mut result = String::new();
                result.push_str("Unknown flag or option: '");
                binary_to_display(&mut result, option.as_bytes());
                result.push('\'');
                Cow::Owned(result)
            }
        }
    }
}

pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Args, ArgsError> {
    enum ArgState {
        Initial,
        ExpectParentPort,
        ExpectKeyDir,
        ExpectCertificate,
        ExpectPrivateKey,
        ExpectConfig,
        ExpectCheck,
    }

    let mut state = ArgState::Initial;

    let mut port = None::<NonZeroU16>;
    let mut key_dir = None::<PathBuf>;
    let mut certificate = None::<PathBuf>;
    let mut private_key = None::<PathBuf>;

    fn parse_port(arg: &[u8]) -> Result<NonZeroU16, ArgsError> {
        parse_u32(arg)
            .and_then(|v| u16::try_from(v).ok())
            .and_then(NonZeroU16::new)
            .ok_or(ArgsError::InvalidPort)
    }

    fn parse_path(arg: &[u8], error: ArgsError) -> Result<PathBuf, ArgsError> {
        if arg.is_empty() {
            Err(error)
        } else {
            Ok(PathBuf::from(std::ffi::OsStr::from_bytes(arg)))
        }
    }

    let mut has_arg = false;

    // Skip the first argument - it's the executable.
    for arg in args.into_iter().skip(1) {
        has_arg = true;
        match state {
            ArgState::Initial => match arg.as_bytes() {
                b"-help" | b"--help" | b"-h" | b"-?" => return Err(ArgsError::ShowHelp),
                b"-version" | b"--version" | b"-v" | b"-V" => return Err(ArgsError::ShowVersion),
                b"-p" | b"--port" => state = ArgState::ExpectParentPort,
                b"-k" | b"--key-dir" => state = ArgState::ExpectKeyDir,
                b"-C" | b"--certificate" => state = ArgState::ExpectCertificate,
                b"-K" | b"--private-key" => state = ArgState::ExpectPrivateKey,
                b"--child-process" => return Ok(Args::Child),
                b"--config" => state = ArgState::ExpectConfig,
                b"-c" | b"--check" => state = ArgState::ExpectCheck,

                // Short option equals
                [b'-', b'p', b'=', arg @ ..] => {
                    port = Some(parse_port(arg)?);
                }
                [b'-', b'k', b'=', arg @ ..] => {
                    key_dir = Some(parse_path(arg, ArgsError::EmptyKeyDir)?);
                }
                [b'-', b'C', b'=', arg @ ..] => {
                    certificate = Some(parse_path(arg, ArgsError::EmptyCertificate)?);
                }
                [b'-', b'K', b'=', arg @ ..] => {
                    private_key = Some(parse_path(arg, ArgsError::EmptyPrivateKey)?);
                }
                [b'-', b'c', b'=', arg @ ..] => {
                    return Ok(Args::Check(parse_path(arg, ArgsError::EmptyConfig)?));
                }

                // `--port=`
                [b'-', b'-', b'p', b'o', b'r', b't', b'=', arg @ ..] => {
                    port = Some(parse_port(arg)?);
                }
                // `--key-dir=`
                [b'-', b'-', b'k', b'e', b'y', b'-', b'd', b'i', b'r', b'=', arg @ ..] => {
                    key_dir = Some(parse_path(arg, ArgsError::EmptyKeyDir)?);
                }
                // `--certificate=`
                [b'-', b'-', b'c', b'e', b'r', b't', b'i', b'f', b'i', b'c', b'a', b't', b'e', b'=', arg @ ..] =>
                {
                    certificate = Some(parse_path(arg, ArgsError::EmptyCertificate)?);
                }
                // `--private-key=`
                [b'-', b'-', b'p', b'r', b'i', b'v', b'a', b't', b'e', b'-', b'k', b'e', b'y', b'=', arg @ ..] =>
                {
                    private_key = Some(parse_path(arg, ArgsError::EmptyPrivateKey)?);
                }
                // `--config=`
                [b'-', b'-', b'c', b'o', b'n', b'f', b'i', b'g', b'=', arg @ ..] => {
                    return Ok(Args::ParentConfig(parse_path(arg, ArgsError::EmptyConfig)?));
                }
                // `--check=`
                [b'-', b'-', b'c', b'h', b'e', b'c', b'k', b'=', arg @ ..] => {
                    return Ok(Args::Check(parse_path(arg, ArgsError::EmptyConfig)?));
                }

                _ => return Err(ArgsError::UnknownFlag(arg)),
            },
            ArgState::ExpectParentPort => {
                state = ArgState::Initial;
                port = Some(parse_port(arg.as_bytes())?);
            }
            ArgState::ExpectKeyDir => {
                state = ArgState::Initial;
                key_dir = Some(parse_path(arg.as_bytes(), ArgsError::EmptyKeyDir)?);
            }
            ArgState::ExpectCertificate => {
                state = ArgState::Initial;
                certificate = Some(parse_path(arg.as_bytes(), ArgsError::EmptyCertificate)?);
            }
            ArgState::ExpectPrivateKey => {
                state = ArgState::Initial;
                private_key = Some(parse_path(arg.as_bytes(), ArgsError::EmptyPrivateKey)?);
            }
            ArgState::ExpectConfig => {
                return Ok(Args::ParentConfig(parse_path(
                    arg.as_bytes(),
                    ArgsError::EmptyConfig,
                )?))
            }
            ArgState::ExpectCheck => {
                return Ok(Args::Check(parse_path(
                    arg.as_bytes(),
                    ArgsError::EmptyConfig,
                )?))
            }
        }
    }

    if !has_arg {
        return Err(ArgsError::ShowHelp);
    }

    match state {
        ArgState::Initial => {
            let tls = match (certificate, private_key) {
                (None, None) => None,
                (None, Some(_)) => return Err(ArgsError::MissingCertificate),
                (Some(_), None) => return Err(ArgsError::MissingPrivateKey),
                (Some(certificate), Some(private_key)) => Some(TLSOptions {
                    certificate,
                    private_key,
                }),
            };

            match (port, key_dir) {
                // Show help if no arguments are given.
                (None, None) => Err(ArgsError::ShowHelp),
                (None, Some(_)) => Err(ArgsError::MissingPort),
                (Some(_), None) => Err(ArgsError::MissingKeyDir),
                (Some(port), Some(key_dir)) => Ok(Args::Parent(ParentArgs { port, key_dir, tls })),
            }
        }
        ArgState::ExpectParentPort => Err(ArgsError::MissingPort),
        ArgState::ExpectKeyDir => Err(ArgsError::MissingKeyDir),
        ArgState::ExpectCertificate => Err(ArgsError::MissingCertificate),
        ArgState::ExpectPrivateKey => Err(ArgsError::MissingPrivateKey),
        ArgState::ExpectConfig => Err(ArgsError::MissingConfig),
        ArgState::ExpectCheck => Err(ArgsError::MissingConfig),
    }
}
