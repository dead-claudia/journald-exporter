use crate::prelude::*;

use super::args::ParentArgs;
use super::args::TLSOptions;
use std::num::NonZeroU16;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigFieldError {
    MissingGlobal,
    InvalidGlobalType,
    MissingPort,
    InvalidPortType,
    InvalidPortNumber(i64),
    MissingKeyDir,
    InvalidKeyDir,
    InvalidHttpsType,
    MissingCertificate,
    MissingPrivateKey,
    InvalidCertificateType,
    InvalidPrivateKeyType,
    InvalidCertificate,
    InvalidPrivateKey,
}

impl ConfigFieldError {
    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            ConfigFieldError::MissingGlobal => Cow::Borrowed("`[global]` must exist as a table."),
            ConfigFieldError::InvalidGlobalType => Cow::Borrowed("`[global]` must be a table."),
            ConfigFieldError::MissingPort => {
                Cow::Borrowed("`global.port` must exist as an integer port number.")
            }
            ConfigFieldError::InvalidPortType => {
                Cow::Borrowed("`global.port` must be an integer port number.")
            }
            ConfigFieldError::InvalidPortNumber(num) => {
                Cow::Owned(format!("{num} is not a valid port number. Ports must be within the range 1 and 65535 inclusive."))
            }
            ConfigFieldError::MissingKeyDir => {
                Cow::Borrowed("`global.key_dir` must exist as a non-empty file name string.")
            }
            ConfigFieldError::InvalidKeyDir => {
                Cow::Borrowed("`global.key_dir` must be a non-empty file name string.")
            }
            ConfigFieldError::InvalidHttpsType => {
                Cow::Borrowed("`[https]` must be a table if specified.")
            }
            ConfigFieldError::MissingCertificate => {
                Cow::Borrowed("`https.certificate` must exist as a non-empty file name string.")
            }
            ConfigFieldError::MissingPrivateKey => {
                Cow::Borrowed("`https.private_key` must exist as a non-empty file name string.")
            }
            ConfigFieldError::InvalidCertificateType => {
                Cow::Borrowed("`https.certificate` must be a non-empty file name string.")
            }
            ConfigFieldError::InvalidPrivateKeyType => {
                Cow::Borrowed("`https.private_key` must be a non-empty file name string.")
            }
            ConfigFieldError::InvalidCertificate => {
                Cow::Borrowed("`https.certificate` must be a non-empty file name.")
            }
            ConfigFieldError::InvalidPrivateKey => {
                Cow::Borrowed("`https.private_key` must be a non-empty file name.")
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    InvalidUTF8(std::str::Utf8Error),
    InvalidSyntax(toml::de::Error),
    InvalidFields(Vec<ConfigFieldError>),
}

pub fn parse_config(data: &[u8]) -> Result<ParentArgs, ConfigError> {
    let data_str = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(e) => return Err(ConfigError::InvalidUTF8(e)),
    };

    let table: toml::Table = match data_str.parse() {
        Ok(table) => table,
        Err(e) => return Err(ConfigError::InvalidSyntax(e)),
    };

    let mut errors = Vec::new();
    let mut port = None;
    let mut key_dir = None;
    let mut tls = None;

    match table.get("global") {
        None => errors.push(ConfigFieldError::MissingGlobal),
        Some(toml::Value::Table(global)) => {
            match global.get("port") {
                None => errors.push(ConfigFieldError::MissingPort),
                Some(toml::Value::Integer(port_num @ 1..=65535)) => {
                    port = Some(
                        u16::try_from(*port_num)
                            .ok()
                            .and_then(NonZeroU16::new)
                            .unwrap(),
                    );
                }
                Some(toml::Value::Integer(num)) => {
                    errors.push(ConfigFieldError::InvalidPortNumber(*num))
                }
                Some(_) => errors.push(ConfigFieldError::InvalidPortType),
            };

            match global.get("key_dir") {
                None => errors.push(ConfigFieldError::MissingKeyDir),
                Some(toml::Value::String(s)) if !s.is_empty() => key_dir = Some(PathBuf::from(s)),
                Some(_) => errors.push(ConfigFieldError::InvalidKeyDir),
            };
        }
        Some(_) => errors.push(ConfigFieldError::InvalidGlobalType),
    };

    match table.get("https") {
        None => {}
        Some(toml::Value::Table(https)) => {
            let mut certificate = None;
            let mut private_key = None;

            match https.get("certificate") {
                None => errors.push(ConfigFieldError::MissingCertificate),
                Some(toml::Value::String(s)) if s.is_empty() => {
                    errors.push(ConfigFieldError::InvalidCertificate);
                }
                Some(toml::Value::String(s)) => {
                    certificate = Some(PathBuf::from(s));
                }
                Some(_) => errors.push(ConfigFieldError::InvalidCertificateType),
            };

            match https.get("private_key") {
                None => errors.push(ConfigFieldError::MissingPrivateKey),
                Some(toml::Value::String(s)) if s.is_empty() => {
                    errors.push(ConfigFieldError::InvalidPrivateKey);
                }
                Some(toml::Value::String(s)) => {
                    private_key = Some(PathBuf::from(s));
                }
                Some(_) => errors.push(ConfigFieldError::InvalidPrivateKeyType),
            };

            if let (Some(certificate), Some(private_key)) = (certificate, private_key) {
                tls = Some(TLSOptions {
                    certificate,
                    private_key,
                });
            }
        }
        Some(_) => errors.push(ConfigFieldError::InvalidHttpsType),
    };

    if errors.is_empty() {
        Ok(ParentArgs {
            port: port.unwrap(),
            key_dir: key_dir.unwrap(),
            tls,
        })
    } else {
        Err(ConfigError::InvalidFields(errors))
    }
}
