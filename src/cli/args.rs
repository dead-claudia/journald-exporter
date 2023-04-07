use crate::prelude::*;

use std::ffi::OsString;
use std::num::NonZeroU16;
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct ChildArgs {
    pub port: NonZeroU16,
}

#[derive(Debug, PartialEq)]
pub struct ParentArgs {
    pub port: NonZeroU16,
    pub key_dir: PathBuf,
}

#[derive(Debug, PartialEq)]
pub enum Args {
    Child(ChildArgs),
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
    UnknownFlag(OsString),
}

impl ArgsError {
    pub fn as_str(&self) -> std::borrow::Cow<'static, str> {
        match self {
            ArgsError::ShowHelp => std::borrow::Cow::Borrowed(super::help::HELP_STRING),
            ArgsError::ShowVersion => std::borrow::Cow::Borrowed(super::help::VERSION_STRING),
            ArgsError::MissingPort => std::borrow::Cow::Borrowed("A port is required."),
            ArgsError::InvalidPort => std::borrow::Cow::Borrowed("Port is invalid."),
            ArgsError::MissingKeyDir => std::borrow::Cow::Borrowed("Key directory missing."),
            ArgsError::EmptyKeyDir => std::borrow::Cow::Borrowed("Key directory cannot be empty."),
            ArgsError::UnknownFlag(option) => std::borrow::Cow::Owned(format!(
                "Unknown flag or option: '{}'",
                BinaryToDisplay(option.as_bytes())
            )),
        }
    }
}

pub fn parse_args(args: impl IntoIterator<Item = OsString>) -> Result<Args, ArgsError> {
    enum ArgState {
        Initial,
        ExpectParentPort,
        ExpectKeyDir,
        ExpectChildPort,
    }

    let mut state = ArgState::Initial;

    let mut child_port = None::<NonZeroU16>;
    let mut parent_port = None::<NonZeroU16>;
    let mut key_dir = None::<PathBuf>;

    // Skip the first argument - it's the executable.
    for arg in args.into_iter().skip(1) {
        match state {
            ArgState::Initial => match arg.to_str() {
                Some("-help" | "--help" | "-h" | "-?") => return Err(ArgsError::ShowHelp),
                Some("-version" | "--version" | "-v" | "-V") => return Err(ArgsError::ShowVersion),
                Some("-p" | "--port") => state = ArgState::ExpectParentPort,
                Some("-k" | "--key-dir") => state = ArgState::ExpectKeyDir,
                Some("--child-process") => state = ArgState::ExpectChildPort,
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
            ArgState::ExpectChildPort => match arg.to_str() {
                None => return Err(ArgsError::InvalidPort),
                // Parse it as a port number
                Some(value) => match value.parse() {
                    Err(_) => return Err(ArgsError::InvalidPort),
                    Ok(port) => {
                        child_port = Some(port);
                        state = ArgState::Initial;
                    }
                },
            },
        }
    }

    match state {
        ArgState::Initial => match child_port {
            Some(port) => Ok(Args::Child(ChildArgs { port })),
            None => match (parent_port, key_dir) {
                // Show help if no arguments are given.
                (None, None) => Err(ArgsError::ShowHelp),
                (None, Some(_)) => Err(ArgsError::MissingPort),
                (Some(_), None) => Err(ArgsError::MissingKeyDir),
                (Some(port), Some(key_dir)) => Ok(Args::Parent(ParentArgs { port, key_dir })),
            },
        },
        ArgState::ExpectParentPort => Err(ArgsError::MissingPort),
        ArgState::ExpectKeyDir => Err(ArgsError::MissingKeyDir),
        ArgState::ExpectChildPort => Err(ArgsError::MissingPort),
    }
}
