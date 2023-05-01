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
    }

    let mut state = ArgState::Initial;

    let mut parent_port = None::<NonZeroU16>;
    let mut key_dir = None::<PathBuf>;
    let mut certificate = None::<PathBuf>;
    let mut private_key = None::<PathBuf>;

    // Skip the first argument - it's the executable.
    for arg in args.into_iter().skip(1) {
        match state {
            ArgState::Initial => match arg.to_str() {
                Some("-help" | "--help" | "-h" | "-?") => return Err(ArgsError::ShowHelp),
                Some("-version" | "--version" | "-v" | "-V") => return Err(ArgsError::ShowVersion),
                Some("-p" | "--port") => state = ArgState::ExpectParentPort,
                Some("-k" | "--key-dir") => state = ArgState::ExpectKeyDir,
                Some("-c" | "--certificate") => state = ArgState::ExpectCertificate,
                Some("-P" | "--private-key") => state = ArgState::ExpectPrivateKey,
                Some("--child-process") => return Ok(Args::Child),
                _ => return Err(ArgsError::UnknownFlag(arg)),
            },
            ArgState::ExpectParentPort => match arg.to_str() {
                None => return Err(ArgsError::InvalidPort),
                // Parse it as a port number
                Some(value) => match value.parse() {
                    Err(_) => return Err(ArgsError::InvalidPort),
                    Ok(port) => {
                        parent_port = Some(port);
                        state = ArgState::Initial;
                    }
                },
            },
            ArgState::ExpectKeyDir => {
                if arg.is_empty() {
                    return Err(ArgsError::EmptyKeyDir);
                }

                key_dir = Some(PathBuf::from(arg));
                state = ArgState::Initial;
            }
            ArgState::ExpectCertificate => {
                if arg.is_empty() {
                    return Err(ArgsError::EmptyCertificate);
                }

                certificate = Some(PathBuf::from(arg));
                state = ArgState::Initial;
            }
            ArgState::ExpectPrivateKey => {
                if arg.is_empty() {
                    return Err(ArgsError::EmptyPrivateKey);
                }

                private_key = Some(PathBuf::from(arg));
                state = ArgState::Initial;
            }
        }
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

            match (parent_port, key_dir) {
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
    }
}
