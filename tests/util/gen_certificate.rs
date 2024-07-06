use std::os::unix::process::CommandExt as _;

pub fn gen_certificate(target_dir: &std::path::Path) -> std::io::Result<()> {
    /*
    Commands:
    ```
    openssl req -x509 \
        -newkey rsa:4096 \
        -keyout key.pem \
        -out cert.pem \
        -sha256 \
        -days 3650 \
        -nodes \
        -subj '/C=US/ST=Oregon/L=Portland/O=Company Name/OU=Org/CN=localhost' \
        -addext 'subjectAltName=DNS:example.com,DNS:*.example.com,IP:10.0.0.1'

    openssl x509 -req \
        -days 365 \
        -in cert.pem \
        -signkey key.pem \
        -out pub.pem
    ```
    */

    #[rustfmt::skip]
    static COMMANDS: &[&[&str]] = &[
        &[
            "req",
            "-x509",
            "-newkey", "rsa:4096",
            "-keyout", "key.pem",
            "-out", "cert.pem",
            "-sha256",
            "-days", "3650",
            "-nodes",
            "-subj", "/C=US/ST=Oregon/L=Portland/O=Company Name/OU=Org/CN=localhost",
            "-addext", "subjectAltName=DNS:example.com,DNS:*.example.com,IP:10.0.0.1",
        ],
        &[
            "x509",
            "-req",
            "-days", "365",
            "-in", "cert.pem",
            "-signkey", "key.pem",
            "-out", "pub.pem",
        ]
    ];

    for args in COMMANDS.iter().copied() {
        openssl(target_dir, args)?;
    }

    Ok(())
}

fn openssl(target_dir: &std::path::Path, args: &[&str]) -> std::io::Result<()> {
    let mut command = std::process::Command::new("openssl");
    command.current_dir(target_dir);
    command.args(args);
    command.stdin(std::process::Stdio::null());
    command.stdout(std::process::Stdio::null());
    command.stderr(std::process::Stdio::inherit());
    unsafe {
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
            Ok(())
        });
    }
    let output = command.output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("Failed to create certificates."))
    }
}
