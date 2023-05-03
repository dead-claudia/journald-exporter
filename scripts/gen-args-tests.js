"use strict"

const fs = require("fs")
const path = require("path")

const root = path.dirname(__dirname)

const toParams = ([short, long]) => ({
    split: [
        ["short", short],
        ["long", long],
    ],
    all: [
        ["short", `${short}", "`],
        ["short_eq", `${short}=`],
        ["long", `${long}", "`],
        ["long_eq", `${long}=`],
    ],
})
const header = "// WARNING: This file is auto-generated by `scripts/gen-args-tests.js`. Do not modify directly.\n"

const testNames = []

fs.rmSync(`${root}/src/cli/args_tests/gen`, {recursive: true, force: true})
fs.mkdirSync(`${root}/src/cli/args_tests/gen`, {recursive: true})

function generate(name, tests) {
    if (testNames.includes(name)) {
        throw new Error(`${name} already generated.`)
    }

    testNames.push(name)

    const source = `${header}
use crate::cli::args::*;

fn parse_args(args: &[&str]) -> Result<Args, ArgsError> {
    crate::cli::args::parse_args(args.iter().map(std::ffi::OsString::from))
}
${tests.map(t => `
#[test]
fn ${t.name}() {
    assert_eq!(
        parse_args(${t.rawTest || `&["journald-exporter", ${t.test}]`}),
        ${t.expect},
    );
}
`).join("")}`

    fs.writeFileSync(`${root}/src/cli/args_tests/gen/${name}.rs`, source)
}

const portParams = toParams(["-p", "--port"])
const keyDirParams = toParams(["-k", "--key-dir"])
const certificateParams = toParams(["-C", "--certificate"])
const privateKeyParams = toParams(["-K", "--private-key"])

generate("port", [
    ...portParams.split.map(([name, value]) => ({
        name: `${name}_start_returns_missing_port`,
        test: `"${value}"`,
        expect: `Err(ArgsError::MissingPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_empty_port_returns_invalid_port`,
        test: `"${source}"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_non_numeric_port_number_returns_invalid_port`,
        test: `"${source}abc"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_partially_numeric_port_number_returns_invalid_port`,
        test: `"${source}abc123"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_hex_port_number_returns_invalid_port`,
        test: `"${source}0x123"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_negative_port_number_returns_invalid_port`,
        test: `"${source}-123"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_negative_zero_port_number_returns_invalid_port`,
        test: `"${source}-0"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_plus_zero_port_number_returns_invalid_port`,
        test: `"${source}+0"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_plus_port_number_for_parent_returns_invalid_port`,
        test: `"${source}+123"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...portParams.all.map(([name, source]) => ({
        name: `${name}_unsigned_port_number_for_parent_returns_missing_key_dir`,
        test: `"${source}123"`,
        expect: `Err(ArgsError::MissingKeyDir)`,
    })),
])

generate("key_dir", [
    ...keyDirParams.split.map(([name, value]) => ({
        name: `${name}_start_returns_missing_key_dir`,
        test: `"${value}"`,
        expect: `Err(ArgsError::MissingKeyDir)`,
    })),
    ...keyDirParams.all.map(([name, source]) => ({
        name: `${name}_arg_without_port_returns_empty_key_dir`,
        test: `"${source}"`,
        expect: `Err(ArgsError::EmptyKeyDir)`,
    })),
    ...keyDirParams.all.map(([name, source]) => ({
        name: `${name}_arg_ending_in_colon_without_port_returns_missing_port`,
        test: `"${source}blah:"`,
        expect: `Err(ArgsError::MissingPort)`,
    })),
    ...keyDirParams.all.map(([name, source]) => ({
        name: `${name}_arg_with_special_chars_and_no_port_returns_missing_port`,
        test: `"${source}b/l@a!h:"`,
        expect: `Err(ArgsError::MissingPort)`,
    })),
    ...keyDirParams.all.map(([name, source]) => ({
        name: `${name}_normal_key_dir_path_without_port_returns_missing_port`,
        test: `"${source}some/dir"`,
        expect: `Err(ArgsError::MissingPort)`,
    })),
])

generate("certificate", [
    ...certificateParams.split.map(([name, value]) => ({
        name: `${name}_start_returns_missing_certificate`,
        test: `"-p", "123", "-k", "some/dir", "${value}"`,
        expect: `Err(ArgsError::MissingCertificate)`,
    })),
    ...certificateParams.all.map(([name, source]) => ({
        name: `${name}_arg_without_private_key_returns_empty_certificate`,
        test: `"-p", "123", "-k", "some/dir", "${source}"`,
        expect: `Err(ArgsError::EmptyCertificate)`,
    })),
    ...certificateParams.all.map(([name, source]) => ({
        name: `${name}_arg_ending_in_colon_without_private_key_returns_missing_private_key`,
        test: `"-p", "123", "-k", "some/dir", "${source}blah:"`,
        expect: `Err(ArgsError::MissingPrivateKey)`,
    })),
    ...certificateParams.all.map(([name, source]) => ({
        name: `${name}_arg_with_special_chars_and_no_private_key_returns_missing_private_key`,
        test: `"-p", "123", "-k", "some/dir", "${source}b/l@a!h:"`,
        expect: `Err(ArgsError::MissingPrivateKey)`,
    })),
    ...certificateParams.all.map(([name, source]) => ({
        name: `${name}_normal_certificate_path_without_private_key_returns_missing_private_key`,
        test: `"-p", "123", "-k", "some/dir", "${source}some/cert.pem"`,
        expect: `Err(ArgsError::MissingPrivateKey)`,
    })),
])

generate("private_key", [
    ...privateKeyParams.split.map(([name, value]) => ({
        name: `${name}_start_returns_missing_private_key`,
        test: `"-p", "123", "-k", "some/dir", "${value}"`,
        expect: `Err(ArgsError::MissingPrivateKey)`,
    })),
    ...privateKeyParams.all.map(([name, source]) => ({
        name: `${name}_arg_without_port_returns_empty_private_key`,
        test: `"-p", "123", "-k", "some/dir", "${source}"`,
        expect: `Err(ArgsError::EmptyPrivateKey)`,
    })),
    ...privateKeyParams.all.map(([name, source]) => ({
        name: `${name}_arg_ending_in_colon_without_certificate_returns_missing_certificate`,
        test: `"-p", "123", "-k", "some/dir", "${source}blah:"`,
        expect: `Err(ArgsError::MissingCertificate)`,
    })),
    ...privateKeyParams.all.map(([name, source]) => ({
        name: `${name}_arg_with_special_chars_and_no_certificate_returns_missing_certificate`,
        test: `"-p", "123", "-k", "some/dir", "${source}b/l@a!h:"`,
        expect: `Err(ArgsError::MissingCertificate)`,
    })),
    ...privateKeyParams.all.map(([name, source]) => ({
        name: `${name}_normal_private_key_path_without_certificate_returns_missing_certificate`,
        test: `"-p", "123", "-k", "some/dir", "${source}some/key.pem"`,
        expect: `Err(ArgsError::MissingCertificate)`,
    })),
])

const joinPortKeyDir = portParams.all.flatMap(([pn, pv]) => (
    keyDirParams.all.map(([kn, kv]) => [pn, pv, kn, kv])
))

generate("port_and_key_dir", [
    ...portParams.all.flatMap(([pn, pv]) => keyDirParams.split.map(([kn, kv]) => ({
        name: `${pn}_port_then_${kn}_key_dir_start_returns_missing_key_dir`,
        test: `"${pv}123", "${kv}"`,
        expect: `Err(ArgsError::MissingKeyDir)`,
    }))),
    ...portParams.split.flatMap(([pn, pv]) => keyDirParams.all.map(([kn, kv]) => ({
        name: `${kn}_key_dir_then_${pn}_port_start_returns_missing_port`,
        test: `"${kv}some/dir", "${pv}"`,
        expect: `Err(ArgsError::MissingPort)`,
    }))),
    ...joinPortKeyDir.map(([pn, pv, kn, kv]) => ({
        name: `${pn}_port_then_${kn}_empty_key_dir_returns_empty_key_dir`,
        test: `"${pv}123", "${kv}"`,
        expect: `Err(ArgsError::EmptyKeyDir)`,
    })),
    ...joinPortKeyDir.map(([pn, pv, kn, kv]) => ({
        name: `${kn}_empty_key_dir_then_${pn}_port_returns_empty_key_dir`,
        test: `"${kv}", "${pv}123"`,
        expect: `Err(ArgsError::EmptyKeyDir)`,
    })),
    ...joinPortKeyDir.map(([pn, pv, kn, kv]) => ({
        name: `${pn}_empty_port_then_${kn}_key_dir_returns_invalid_port`,
        test: `"${pv}", "${kv}some/dir"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...joinPortKeyDir.map(([pn, pv, kn, kv]) => ({
        name: `${kn}_key_dir_then_${pn}_empty_port_returns_invalid_port`,
        test: `"${kv}some/dir", "${pv}"`,
        expect: `Err(ArgsError::InvalidPort)`,
    })),
    ...joinPortKeyDir.map(([pn, pv, kn, kv]) => ({
        name: `${pn}_port_then_${kn}_normal_key_dir_returns_success`,
        test: `"${pv}123", "${kv}some/dir"`,
        expect: `Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        }))`,
    })),
    ...joinPortKeyDir.map(([pn, pv, kn, kv]) => ({
        name: `${kn}_normal_key_dir_then_${pn}_port_returns_success`,
        test: `"${kv}some/dir", "${pv}123"`,
        expect: `Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: None,
        }))`,
    })),
])

const joinCertificatePrivateKey = certificateParams.all.flatMap(([cn, cv]) => (
    privateKeyParams.all.map(([pn, pv]) => [cn, cv, pn, pv])
))

generate("certificate_and_private_key", [
    ...certificateParams.all.flatMap(([cn, cv]) => privateKeyParams.split.map(([pn, pv]) => ({
        name: `${cn}_normal_certificate_then_${pn}_private_key_start_returns_missing_private_key`,
        test: `"-p", "123", "-k", "some/dir", "${cv}some/key.pem", "${pv}"`,
        expect: `Err(ArgsError::MissingPrivateKey)`,
    }))),
    ...certificateParams.split.flatMap(([cn, cv]) => privateKeyParams.all.map(([pn, pv]) => ({
        name: `${pn}_normal_private_key_then_${cn}_certificate_start_returns_missing_certificate`,
        test: `"-p", "123", "-k", "some/dir", "${pv}some/key.pem", "${cv}"`,
        expect: `Err(ArgsError::MissingCertificate)`,
    }))),
    ...joinCertificatePrivateKey.map(([cn, cv, pn, pv]) => ({
        name: `${cn}_normal_certificate_then_${pn}_empty_private_key_returns_empty_private_key`,
        test: `"-p", "123", "-k", "some/dir", "${cv}some/cert.pem", "${pv}"`,
        expect: `Err(ArgsError::EmptyPrivateKey)`,
    })),
    ...joinCertificatePrivateKey.map(([cn, cv, pn, pv]) => ({
        name: `${pn}_empty_private_key_then_${cn}_normal_certificate_returns_empty_private_key`,
        test: `"-p", "123", "-k", "some/dir", "${pv}", "${cv}some/cert.pem"`,
        expect: `Err(ArgsError::EmptyPrivateKey)`,
    })),
    ...joinCertificatePrivateKey.map(([cn, cv, pn, pv]) => ({
        name: `${cn}_empty_certificate_then_${pn}_normal_private_key_returns_empty_certificate`,
        test: `"-p", "123", "-k", "some/dir", "${cv}", "${pv}some/key.pem"`,
        expect: `Err(ArgsError::EmptyCertificate)`,
    })),
    ...joinCertificatePrivateKey.map(([cn, cv, pn, pv]) => ({
        name: `${pn}_normal_private_key_then_${cn}_empty_certificate_returns_empty_certificate`,
        test: `"-p", "123", "-k", "some/dir", "${pv}some/key.pem", "${cv}"`,
        expect: `Err(ArgsError::EmptyCertificate)`,
    })),
    ...joinCertificatePrivateKey.map(([cn, cv, pn, pv]) => ({
        name: `${cn}_normal_certificate_then_${pn}_normal_private_key_returns_success`,
        test: `"-p", "123", "-k", "some/dir", "${cv}some/cert.pem", "${pv}some/key.pem"`,
        expect: `Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: Some(TLSOptions {
                certificate: std::path::PathBuf::from("some/cert.pem"),
                private_key: std::path::PathBuf::from("some/key.pem"),
            }),
        }))`,
    })),
    ...joinCertificatePrivateKey.map(([cn, cv, pn, pv]) => ({
        name: `${pn}_normal_private_key_then_${cn}_normal_certificate_returns_success`,
        test: `"-p", "123", "-k", "some/dir", "${pv}some/key.pem", "${cv}some/cert.pem"`,
        expect: `Ok(Args::Parent(ParentArgs {
            port: std::num::NonZeroU16::new(123).unwrap(),
            key_dir: std::path::PathBuf::from("some/dir"),
            tls: Some(TLSOptions {
                certificate: std::path::PathBuf::from("some/cert.pem"),
                private_key: std::path::PathBuf::from("some/key.pem"),
            }),
        }))`,
    })),
])

fs.writeFileSync(
    `${root}/src/cli/args_tests/gen/mod.rs`,
    `${header}\n${testNames.map(name => `mod ${name};\n`).join("")}`,
)

console.log("Be sure to:")
console.log("1. Verify the output (important).")
console.log("2. Run `cargo fmt -- src/cli/args_tests/gen/*.rs` after.")
