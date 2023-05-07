use crate::prelude::*;

use super::args::ParentArgs;
use super::args::TLSOptions;
use std::num::NonZeroU16;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct ParentConfig {
    pub port: std::num::NonZeroU16,
    pub key_dir: std::path::PathBuf,
    pub tls: Option<TLSOptions>,
    pub monitor_filter: Option<Vec<MonitorFilterEntry>>,
}

#[derive(Debug, PartialEq)]
pub enum IdOrName {
    Placeholder,
    Id(u32),
    Name(Box<str>),
}

#[derive(Debug, PartialEq)]
pub struct MonitorFilterEntry {
    pub monitor_name: Arc<str>,
    pub priority: Option<Priority>,
    pub user: Option<IdOrName>,
    pub group: Option<IdOrName>,
    pub service: Option<ServiceRepr>,
    pub message_pattern: Option<Box<str>>,
}

impl ParentConfig {
    pub fn from_args(args: ParentArgs) -> Self {
        Self {
            port: args.port,
            key_dir: args.key_dir,
            tls: args.tls,
            monitor_filter: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigMonitorFieldError {
    InvalidMonitorType,
    InvalidPriorityType,
    InvalidPriorityIntegerValue,
    InvalidPriorityNameValue,
    PriorityNumericName,
    InvalidUserType,
    InvalidUserId,
    InvalidUserName,
    InvalidGroupType,
    InvalidGroupId,
    InvalidGroupName,
    InvalidServiceType,
    InvalidMessageType,
    MessageRegexTooLarge,
    InvalidMessageRegex(Box<str>),
}

impl ConfigMonitorFieldError {
    fn push_str(&self, result: &mut String) {
        match self {
            ConfigMonitorFieldError::InvalidMonitorType => {
                result.push_str("`Monitor must be a table if given.");
            }
            ConfigMonitorFieldError::InvalidPriorityType => {
                result.push_str("`priority` must be either an integer priority between 0 and 7 inclusive or a priority name string if provided.");
            }
            ConfigMonitorFieldError::InvalidPriorityIntegerValue => {
                result.push_str("Integer `priority` must be between 0 and 7 inclusive.");
            }
            ConfigMonitorFieldError::InvalidPriorityNameValue => {
                result.push_str("String `priority` must be a valid syslog priority name.");
            }
            ConfigMonitorFieldError::PriorityNumericName => {
                result.push_str("String `priority` must be a valid syslog priority name. Consider removing the quotes.");
            }
            ConfigMonitorFieldError::InvalidUserType => {
                result.push_str("`user` must be either a string username, an integer ID, or the literal string `'?'` if provided.");
            }
            ConfigMonitorFieldError::InvalidUserId => {
                result.push_str(
                    "Integer `user` must be within the range 0 (root) to 4294967295 inclusive.",
                );
            }
            ConfigMonitorFieldError::InvalidUserName => {
                result.push_str(r"String `user` is neither a valid username nor the literal string `'?'`. Valid usernames match the regular expression /^[_A-Za-z][_0-9A-Za-z-]*\$?$/ and are at most 32 characters long.");
            }
            ConfigMonitorFieldError::InvalidGroupType => {
                result.push_str("`group` must be either a string group name, an integer ID, or the literal string `'?'` if provided.");
            }
            ConfigMonitorFieldError::InvalidGroupId => {
                result.push_str(
                    "Integer `group` must be within the range 0 (root) to 4294967295 inclusive.",
                );
            }
            ConfigMonitorFieldError::InvalidGroupName => {
                result.push_str(r"String `group` is neither a valid group name nor the literal string `'?'`. Valid group names match the regular expression /^[_A-Za-z][_0-9A-Za-z-]*\$?$/ and are at most 32 characters long.");
            }
            ConfigMonitorFieldError::InvalidServiceType => {
                result.push_str(r"`service` must be a valid service name string if provided. Valid service names match the regular expression /^[0-9A-Za-z:_.\\-]+(@[0-9A-Za-z:_.\\-]+)?$/ and are at most 32 characters long.");
            }
            ConfigMonitorFieldError::InvalidMessageType => {
                result.push_str("`message` must be a valid Rust regular expression. See https://docs.rs/regex/latest/regex/#syntax for more information.");
            }
            ConfigMonitorFieldError::MessageRegexTooLarge => {
                result.push_str(
                    "An error was encountered while validating `message`: Compiled regex too big",
                );
            }
            ConfigMonitorFieldError::InvalidMessageRegex(message) => {
                result.push_str("An error was encountered while validating `message`: ");
                result.push_str(message);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    InvalidMonitorTableType,
    InvalidMonitorField(Box<str>, ConfigMonitorFieldError),
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
            ConfigFieldError::InvalidMonitorTableType => {
                Cow::Borrowed("`[monitor]` must be a table if given.")
            }
            ConfigFieldError::InvalidMonitorField(name, field_error) => {
                let mut result = String::new();
                result.push_str("In `[monitor.");
                result.push_str(name);
                result.push_str("]`: ");
                field_error.push_str(&mut result);
                Cow::Owned(result)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    InvalidUTF8(std::str::Utf8Error),
    InvalidSyntax(toml::de::Error),
    InvalidFields(Box<[ConfigFieldError]>),
}

struct MonitorContext<'a, 'b> {
    name: &'a str,
    table: &'a toml::Table,
    errors: &'b mut Vec<ConfigFieldError>,
}

struct ExtractIdData {
    field: &'static str,
    invalid_type: ConfigMonitorFieldError,
    invalid_id: ConfigMonitorFieldError,
    invalid_name: ConfigMonitorFieldError,
}

impl MonitorContext<'_, '_> {
    fn report(&mut self, error: ConfigMonitorFieldError) {
        self.errors.push(ConfigFieldError::InvalidMonitorField(
            self.name.to_owned().into(),
            error,
        ))
    }

    fn parse_monitor_priority(&mut self, value: &str) -> Option<Priority> {
        match value.as_bytes() {
            // `EMERG`, deprecated `PANIC`
            [b'e' | b'E', b'm' | b'M', b'e' | b'E', b'r' | b'R', b'g' | b'G']
            | [b'p' | b'P', b'a' | b'A', b'n' | b'N', b'i' | b'I', b'c' | b'C'] => {
                Some(Priority::Emergency)
            }
            // `ALERT`
            [b'a' | b'A', b'l' | b'L', b'e' | b'E', b'r' | b'R', b't' | b'T'] => {
                Some(Priority::Alert)
            }
            // `CRIT`
            [b'c' | b'C', b'r' | b'R', b'i' | b'I', b't' | b'T'] => Some(Priority::Critical),
            // `ERR`, deprecated `ERROR`
            [b'e' | b'E', b'r' | b'R', b'r' | b'R']
            | [b'e' | b'E', b'r' | b'R', b'r' | b'R', b'o' | b'O', b'r' | b'R'] => {
                Some(Priority::Error)
            }
            // `WARNING`, deprecated `WARN`
            [b'w' | b'W', b'a' | b'A', b'r' | b'R', b'n' | b'N', b'i' | b'I', b'n' | b'N', b'g' | b'G']
            | [b'w' | b'W', b'a' | b'A', b'r' | b'R', b'n' | b'N'] => Some(Priority::Warning),
            // `NOTICE`
            [b'n' | b'N', b'o' | b'O', b't' | b'T', b'i' | b'I', b'c' | b'C', b'e' | b'E'] => {
                Some(Priority::Notice)
            }
            // `INFO`
            [b'i' | b'I', b'n' | b'N', b'f' | b'F', b'o' | b'O'] => Some(Priority::Informational),
            // `DEBUG`
            [b'd' | b'D', b'e' | b'E', b'b' | b'B', b'u' | b'U', b'g' | b'G'] => {
                Some(Priority::Debug)
            }
            // String values `0` through `7`
            bytes => {
                self.report(match Priority::from_severity_value(bytes) {
                    Ok(_) => ConfigMonitorFieldError::PriorityNumericName,
                    Err(_) => ConfigMonitorFieldError::InvalidPriorityNameValue,
                });
                None
            }
        }
    }

    fn extract_id(&mut self, data: &ExtractIdData, target: &mut Option<IdOrName>) {
        match self.table.get(data.field) {
            None => {}
            Some(toml::Value::Integer(num)) => match u32::try_from(*num) {
                Ok(id) => *target = Some(IdOrName::Id(id)),
                Err(_) => self.report(data.invalid_id.clone()),
            },
            Some(toml::Value::String(username)) => {
                match validate_name_or_placeholder(username.as_bytes()) {
                    NameOrPlaceholderResult::Invalid => self.report(data.invalid_name.clone()),
                    NameOrPlaceholderResult::ValidName => {
                        *target = Some(IdOrName::Name(username.clone().into()))
                    }
                    NameOrPlaceholderResult::ValidPlaceholder => {
                        *target = Some(IdOrName::Placeholder)
                    }
                }
            }
            Some(_) => self.report(data.invalid_type.clone()),
        }
    }

    fn parse_monitor_field(mut self) -> MonitorFilterEntry {
        let mut priority = None;
        let mut user = None;
        let mut group = None;
        let mut service = None;
        let mut message_pattern = None;

        match self.table.get("priority") {
            None => {}
            Some(toml::Value::Integer(num @ 0..=7)) => {
                priority = Some(Priority::from_severity_index(truncate_i64_u8(*num)).unwrap());
            }
            Some(toml::Value::Integer(_)) => {
                self.report(ConfigMonitorFieldError::InvalidPriorityIntegerValue);
            }
            Some(toml::Value::String(value)) => priority = self.parse_monitor_priority(value),
            Some(_) => self.report(ConfigMonitorFieldError::InvalidPriorityType),
        }

        static USER_DATA: ExtractIdData = ExtractIdData {
            field: "user",
            invalid_type: ConfigMonitorFieldError::InvalidUserType,
            invalid_id: ConfigMonitorFieldError::InvalidUserId,
            invalid_name: ConfigMonitorFieldError::InvalidUserName,
        };

        static GROUP_DATA: ExtractIdData = ExtractIdData {
            field: "group",
            invalid_type: ConfigMonitorFieldError::InvalidGroupType,
            invalid_id: ConfigMonitorFieldError::InvalidGroupId,
            invalid_name: ConfigMonitorFieldError::InvalidGroupName,
        };

        self.extract_id(&USER_DATA, &mut user);
        self.extract_id(&GROUP_DATA, &mut group);

        match self.table.get("service") {
            None => {}
            Some(toml::Value::String(slice)) => {
                if slice.is_empty() {
                    service = Some(ServiceRepr::empty());
                } else {
                    match ServiceRepr::from_slice(slice.as_bytes()) {
                        Ok(s) => service = Some(s),
                        Err(_) => self.report(ConfigMonitorFieldError::InvalidServiceType),
                    }
                }
            }
            Some(_) => self.report(ConfigMonitorFieldError::InvalidServiceType),
        }

        match self.table.get("message") {
            None => {}
            Some(toml::Value::String(slice)) => match regex::bytes::Regex::new(slice) {
                Ok(_) => message_pattern = Some(slice.as_str().into()),
                Err(e) => match e {
                    regex::Error::Syntax(message) => {
                        self.report(ConfigMonitorFieldError::InvalidMessageRegex(message.into()))
                    }
                    regex::Error::CompiledTooBig(_) => {
                        self.report(ConfigMonitorFieldError::MessageRegexTooLarge)
                    }
                    // Should get compiled out.
                    e => self.report(ConfigMonitorFieldError::InvalidMessageRegex(
                        e.to_string().into(),
                    )),
                },
            },
            Some(_) => self.report(ConfigMonitorFieldError::InvalidMessageType),
        }

        MonitorFilterEntry {
            monitor_name: self.name.to_owned().into(),
            priority,
            user,
            group,
            service,
            message_pattern,
        }
    }
}

struct GlobalSection {
    port: Option<std::num::NonZeroU16>,
    key_dir: Option<std::path::PathBuf>,
}

struct SectionData {
    name: &'static str,
    invalid_type: ConfigFieldError,
    missing: Option<ConfigFieldError>,
}

struct ConfigContext<'a> {
    table: &'a toml::Table,
    errors: Vec<ConfigFieldError>,
}

impl<'a> ConfigContext<'a> {
    fn report(&mut self, error: ConfigFieldError) {
        self.errors.push(error);
    }

    fn try_open(&mut self, data: &SectionData) -> Option<&'a toml::Table> {
        match self.table.get(data.name) {
            Some(toml::Value::Table(table)) => Some(table),
            Some(_) => {
                self.report(data.invalid_type.clone());
                None
            }
            None => {
                if let Some(missing) = data.missing.as_ref() {
                    self.report(missing.clone());
                }
                None
            }
        }
    }

    fn parse_global(&mut self) -> GlobalSection {
        let mut section = GlobalSection {
            port: None,
            key_dir: None,
        };

        static DATA: SectionData = SectionData {
            name: "global",
            invalid_type: ConfigFieldError::InvalidGlobalType,
            missing: Some(ConfigFieldError::MissingGlobal),
        };

        let global = match self.try_open(&DATA) {
            Some(global) => global,
            None => return section,
        };

        match global.get("port") {
            None => self.report(ConfigFieldError::MissingPort),
            Some(toml::Value::Integer(value)) => {
                match u16::try_from(*value).ok().and_then(NonZeroU16::new) {
                    Some(port) => section.port = Some(port),
                    None => self.report(ConfigFieldError::InvalidPortNumber(*value)),
                }
            }
            Some(_) => self.report(ConfigFieldError::InvalidPortType),
        };

        match global.get("key_dir") {
            Some(toml::Value::String(s)) if !s.is_empty() => {
                section.key_dir = Some(PathBuf::from(s))
            }
            Some(_) => self.report(ConfigFieldError::InvalidKeyDir),
            None => self.report(ConfigFieldError::MissingKeyDir),
        };

        section
    }

    fn parse_https(&mut self) -> Option<TLSOptions> {
        static DATA: SectionData = SectionData {
            name: "https",
            invalid_type: ConfigFieldError::InvalidHttpsType,
            missing: None,
        };

        let https = self.try_open(&DATA)?;

        let mut certificate = None;
        let mut private_key = None;

        match https.get("certificate") {
            None => self.report(ConfigFieldError::MissingCertificate),
            Some(toml::Value::String(s)) if s.is_empty() => {
                self.report(ConfigFieldError::InvalidCertificate);
            }
            Some(toml::Value::String(s)) => {
                certificate = Some(PathBuf::from(s));
            }
            Some(_) => self.report(ConfigFieldError::InvalidCertificateType),
        };

        match https.get("private_key") {
            None => self.report(ConfigFieldError::MissingPrivateKey),
            Some(toml::Value::String(s)) if s.is_empty() => {
                self.report(ConfigFieldError::InvalidPrivateKey);
            }
            Some(toml::Value::String(s)) => {
                private_key = Some(PathBuf::from(s));
            }
            Some(_) => self.report(ConfigFieldError::InvalidPrivateKeyType),
        };

        if let (Some(certificate), Some(private_key)) = (certificate, private_key) {
            Some(TLSOptions {
                certificate,
                private_key,
            })
        } else {
            None
        }
    }

    fn parse_monitor(&mut self) -> Option<Vec<MonitorFilterEntry>> {
        static DATA: SectionData = SectionData {
            name: "monitor",
            invalid_type: ConfigFieldError::InvalidMonitorTableType,
            missing: None,
        };

        let monitor = self.try_open(&DATA)?;
        let mut entries = Vec::with_capacity(monitor.len());

        for (name, value) in monitor.iter() {
            match value {
                toml::Value::Table(table) => entries.push(
                    MonitorContext {
                        name,
                        table,
                        errors: &mut self.errors,
                    }
                    .parse_monitor_field(),
                ),
                _ => self.report(ConfigFieldError::InvalidMonitorField(
                    name.to_owned().into(),
                    ConfigMonitorFieldError::InvalidMonitorType,
                )),
            };
        }

        Some(entries)
    }
}

pub fn parse_config(data: &[u8]) -> Result<ParentConfig, ConfigError> {
    match std::str::from_utf8(data) {
        Err(e) => Err(ConfigError::InvalidUTF8(e)),
        Ok(s) => match s.parse() {
            Err(e) => Err(ConfigError::InvalidSyntax(e)),
            Ok(table) => {
                let mut ctx = ConfigContext {
                    table: &table,
                    errors: Vec::new(),
                };

                let global = ctx.parse_global();
                let tls = ctx.parse_https();
                let monitor_filter = ctx.parse_monitor();

                if ctx.errors.is_empty() {
                    Ok(ParentConfig {
                        port: global.port.unwrap(),
                        key_dir: global.key_dir.unwrap(),
                        tls,
                        monitor_filter,
                    })
                } else {
                    Err(ConfigError::InvalidFields(ctx.errors.into()))
                }
            }
        },
    }
}
