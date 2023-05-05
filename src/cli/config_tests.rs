use super::config::*;
use crate::cli::args::ParentArgs;
use crate::cli::args::TLSOptions;
use std::num::NonZeroU16;
use std::path::PathBuf;

#[test]
fn invalid_utf8() {
    let config = parse_config(b"1\x802");
    assert!(
        matches!(&config, Err(ConfigError::InvalidUTF8(_))),
        "found {config:?}",
    );
}

#[test]
fn invalid_syntax() {
    let config = parse_config(b"a;b");
    assert!(
        matches!(&config, Err(ConfigError::InvalidSyntax(_))),
        "found {config:?}",
    );
}

#[test]
fn empty() {
    assert_eq!(
        parse_config(b""),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
        ]))
    );
}

#[test]
fn global_boolean() {
    assert_eq!(
        parse_config(b"global = true"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidGlobalType,
        ]))
    );
}

#[test]
fn global_integer() {
    assert_eq!(
        parse_config(b"global = 123"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidGlobalType,
        ]))
    );
}

#[test]
fn global_float() {
    assert_eq!(
        parse_config(b"global = 123.0"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidGlobalType,
        ]))
    );
}

#[test]
fn global_string() {
    assert_eq!(
        parse_config(b"global = 'foo'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidGlobalType,
        ]))
    );
}

#[test]
fn global_array() {
    assert_eq!(
        parse_config(b"[[global]]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidGlobalType,
        ]))
    );
}

#[test]
fn global_table_empty() {
    assert_eq!(
        parse_config(b"[global]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_boolean() {
    assert_eq!(
        parse_config(b"[global]\nport = true"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortType,
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_float() {
    assert_eq!(
        parse_config(b"[global]\nport = 123.0"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortType,
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_string() {
    assert_eq!(
        parse_config(b"[global]\nport = 'foo'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortType,
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_array() {
    assert_eq!(
        parse_config(b"[global]\n[[global.port]]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortType,
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_table() {
    assert_eq!(
        parse_config(b"[global]\n[global.port]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortType,
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_idiom() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_alt() {
    assert_eq!(
        parse_config(b"[global]\nport = 9999"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_negative_1() {
    assert_eq!(
        parse_config(b"[global]\nport = -1"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortNumber(-1),
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_0() {
    assert_eq!(
        parse_config(b"[global]\nport = 0"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortNumber(0),
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_1() {
    assert_eq!(
        parse_config(b"[global]\nport = 1"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_65535() {
    assert_eq!(
        parse_config(b"[global]\nport = 65535"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_65536() {
    assert_eq!(
        parse_config(b"[global]\nport = 65536"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortNumber(65536),
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_100000() {
    assert_eq!(
        parse_config(b"[global]\nport = 100000"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortNumber(100000),
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_port_integer_massive() {
    assert_eq!(
        parse_config(b"[global]\nport = 1234567890123456789"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidPortNumber(1234567890123456789),
            ConfigFieldError::MissingKeyDir,
        ]))
    );
}

#[test]
fn global_key_dir_boolean() {
    assert_eq!(
        parse_config(b"[global]\nkey_dir = true"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_key_dir_integer() {
    assert_eq!(
        parse_config(b"[global]\nkey_dir = 123"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_key_dir_float() {
    assert_eq!(
        parse_config(b"[global]\nkey_dir = 123.0"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_key_dir_array() {
    assert_eq!(
        parse_config(b"[global]\n[[global.key_dir]]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_key_dir_table() {
    assert_eq!(
        parse_config(b"[global]\n[global.key_dir]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_key_dir_string_empty() {
    assert_eq!(
        parse_config(b"[global]\nkey_dir = ''"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_key_dir_string_idiom() {
    assert_eq!(
        parse_config(b"[global]\nkey_dir = '/etc/journald-exporter/keys'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
        ]))
    );
}

#[test]
fn global_key_dir_string_alt() {
    assert_eq!(
        parse_config(b"[global]\nkey_dir = '/some/other/key/dir'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingPort,
        ]))
    );
}

#[test]
fn global_port_key_dir_boolean() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345\nkey_dir = true"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_port_key_dir_integer() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345\nkey_dir = 123"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_port_key_dir_float() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345\nkey_dir = 123.0"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_port_key_dir_array() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345\n[[global.key_dir]]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_port_key_dir_table() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345\n[global.key_dir]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_port_key_dir_string_empty() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345\nkey_dir = ''"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::InvalidKeyDir,
        ]))
    );
}

#[test]
fn global_port_key_dir_string_idiom() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345\nkey_dir = '/etc/journald-exporter/keys'"),
        Ok(ParentArgs {
            port: NonZeroU16::new(12345).unwrap(),
            key_dir: PathBuf::from("/etc/journald-exporter/keys"),
            tls: None
        })
    );
}

#[test]
fn global_port_idiom_key_dir_string_alt() {
    assert_eq!(
        parse_config(b"[global]\nport = 12345\nkey_dir = '/some/other/key/dir'"),
        Ok(ParentArgs {
            port: NonZeroU16::new(12345).unwrap(),
            key_dir: PathBuf::from("/some/other/key/dir"),
            tls: None
        })
    );
}

#[test]
fn global_port_alt_key_dir_string_idiom() {
    assert_eq!(
        parse_config(b"[global]\nport = 9999\nkey_dir = '/etc/journald-exporter/keys'"),
        Ok(ParentArgs {
            port: NonZeroU16::new(9999).unwrap(),
            key_dir: PathBuf::from("/etc/journald-exporter/keys"),
            tls: None
        })
    );
}

#[test]
fn global_port_alt_key_dir_string_alt() {
    assert_eq!(
        parse_config(b"[global]\nport = 9999\nkey_dir = '/some/other/key/dir'"),
        Ok(ParentArgs {
            port: NonZeroU16::new(9999).unwrap(),
            key_dir: PathBuf::from("/some/other/key/dir"),
            tls: None
        })
    );
}

#[test]
fn https_boolean() {
    assert_eq!(
        parse_config(b"https = false"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidHttpsType,
        ]))
    );
}

#[test]
fn https_integer() {
    assert_eq!(
        parse_config(b"https = 123"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidHttpsType,
        ]))
    );
}

#[test]
fn https_float() {
    assert_eq!(
        parse_config(b"https = 123.0"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidHttpsType,
        ]))
    );
}

#[test]
fn https_string() {
    assert_eq!(
        parse_config(b"https = false"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidHttpsType,
        ]))
    );
}

#[test]
fn https_array() {
    assert_eq!(
        parse_config(b"[[https]]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidHttpsType,
        ]))
    );
}

#[test]
fn https_table_empty() {
    assert_eq!(
        parse_config(b"[https]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_certificate_boolean() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = true"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidCertificateType,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_certificate_integer() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = 123"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidCertificateType,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_certificate_float() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = 123.0"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidCertificateType,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_certificate_array() {
    assert_eq!(
        parse_config(b"[https]\n[[https.certificate]]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidCertificateType,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_certificate_table() {
    assert_eq!(
        parse_config(b"[https]\n[https.certificate]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidCertificateType,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_certificate_string_empty() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = ''"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::InvalidCertificate,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_certificate_string_idiom() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = '/etc/journald-exporter/cert.key'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_certificate_string_alt() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = '/some/other/key/file'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingPrivateKey,
        ]))
    );
}

#[test]
fn https_private_key_boolean() {
    assert_eq!(
        parse_config(b"[https]\nprivate_key = true"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
            ConfigFieldError::InvalidPrivateKeyType,
        ]))
    );
}

#[test]
fn https_private_key_integer() {
    assert_eq!(
        parse_config(b"[https]\nprivate_key = 123"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
            ConfigFieldError::InvalidPrivateKeyType,
        ]))
    );
}

#[test]
fn https_private_key_float() {
    assert_eq!(
        parse_config(b"[https]\nprivate_key = 123.0"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
            ConfigFieldError::InvalidPrivateKeyType,
        ]))
    );
}

#[test]
fn https_private_key_array() {
    assert_eq!(
        parse_config(b"[https]\n[[https.private_key]]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
            ConfigFieldError::InvalidPrivateKeyType,
        ]))
    );
}

#[test]
fn https_private_key_table() {
    assert_eq!(
        parse_config(b"[https]\n[https.private_key]"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
            ConfigFieldError::InvalidPrivateKeyType,
        ]))
    );
}

#[test]
fn https_private_key_string_empty() {
    assert_eq!(
        parse_config(b"[https]\nprivate_key = ''"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
            ConfigFieldError::InvalidPrivateKey,
        ]))
    );
}

#[test]
fn https_private_key_string_idiom() {
    assert_eq!(
        parse_config(b"[https]\nprivate_key = '/etc/journald-exporter/priv.key'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
        ]))
    );
}

#[test]
fn https_private_key_string_alt() {
    assert_eq!(
        parse_config(b"[https]\nprivate_key = '/some/other/key/file'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
            ConfigFieldError::MissingCertificate,
        ]))
    );
}

#[test]
fn https_certificate_private_key_idiom() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = '/etc/journald-exporter/cert.key'\nprivate_key = '/etc/journald-exporter/priv.key'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
        ]))
    );
}

#[test]
fn https_certificate_alt_private_key_idiom() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = '/some/other/key/file'\nprivate_key = '/etc/journald-exporter/priv.key'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
        ]))
    );
}

#[test]
fn https_certificate_idiom_private_key_alt() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = '/etc/journald-exporter/cert.key'\nprivate_key = '/some/other/key/file'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
        ]))
    );
}

#[test]
fn https_certificate_private_key_alt() {
    assert_eq!(
        parse_config(b"[https]\ncertificate = '/some/other/cert/file'\nprivate_key = '/some/other/priv/key/file'"),
        Err(ConfigError::InvalidFields(vec![
            ConfigFieldError::MissingGlobal,
        ]))
    );
}

#[test]
fn global_idiom_https_certificate_private_key_idiom() {
    assert_eq!(
        parse_config(
            b"
[global]
port = 12345
key_dir = '/etc/journald-exporter/keys'
[https]
certificate = '/etc/journald-exporter/cert.key'
private_key = '/etc/journald-exporter/priv.key'
"
        ),
        Ok(ParentArgs {
            port: NonZeroU16::new(12345).unwrap(),
            key_dir: PathBuf::from("/etc/journald-exporter/keys"),
            tls: Some(TLSOptions {
                certificate: PathBuf::from("/etc/journald-exporter/cert.key"),
                private_key: PathBuf::from("/etc/journald-exporter/priv.key"),
            })
        })
    );
}

#[test]
fn global_idiom_https_certificate_alt_private_key_idiom() {
    assert_eq!(
        parse_config(
            b"
[global]
port = 12345
key_dir = '/etc/journald-exporter/keys'
[https]
certificate = '/some/other/key/file'
private_key = '/etc/journald-exporter/priv.key'
"
        ),
        Ok(ParentArgs {
            port: NonZeroU16::new(12345).unwrap(),
            key_dir: PathBuf::from("/etc/journald-exporter/keys"),
            tls: Some(TLSOptions {
                certificate: PathBuf::from("/some/other/key/file"),
                private_key: PathBuf::from("/etc/journald-exporter/priv.key"),
            })
        })
    );
}

#[test]
fn global_idiom_https_certificate_idiom_private_key_alt() {
    assert_eq!(
        parse_config(
            b"
[global]
port = 12345
key_dir = '/etc/journald-exporter/keys'
[https]
certificate = '/etc/journald-exporter/cert.key'
private_key = '/some/other/key/file'
"
        ),
        Ok(ParentArgs {
            port: NonZeroU16::new(12345).unwrap(),
            key_dir: PathBuf::from("/etc/journald-exporter/keys"),
            tls: Some(TLSOptions {
                certificate: PathBuf::from("/etc/journald-exporter/cert.key"),
                private_key: PathBuf::from("/some/other/key/file"),
            })
        })
    );
}

#[test]
fn global_idiom_https_certificate_private_key_alt() {
    assert_eq!(
        parse_config(
            b"
[global]
port = 12345
key_dir = '/etc/journald-exporter/keys'
[https]
certificate = '/some/other/cert/file'
private_key = '/some/other/priv/key/file'
"
        ),
        Ok(ParentArgs {
            port: NonZeroU16::new(12345).unwrap(),
            key_dir: PathBuf::from("/etc/journald-exporter/keys"),
            tls: Some(TLSOptions {
                certificate: PathBuf::from("/some/other/cert/file"),
                private_key: PathBuf::from("/some/other/priv/key/file"),
            })
        })
    );
}
