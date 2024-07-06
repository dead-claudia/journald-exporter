use super::debug_big_slice::DebugBigSlice;
use super::gen_certificate::gen_certificate;
use super::get_binary::get_binary_path;
use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;
use std::process::Stdio;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

pub const REQUEST_METRICS: u8 = 0x00;
pub const REQUEST_KEY: u8 = 0x01;
pub const TRACK_REQUEST: u8 = 0x02;

struct SpawnState {
    port: u16,
    child_stdin: std::process::ChildStdin,
    consumed_stdout: Arc<Mutex<Vec<u8>>>,
    cert_dir: Option<tempfile::TempDir>,
}

pub struct Server {
    is_secure: bool,
    state: std::sync::OnceLock<io::Result<SpawnState>>,
}

fn kill_on_parent_termination(command: &mut std::process::Command) {
    unsafe {
        command.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
            Ok(())
        });
    }
}

fn try_read_port(line: &str) -> Option<u16> {
    let remaining = line.strip_prefix("Server listener bound at port ")?;
    if remaining.is_empty() {
        return None;
    }
    let port_end = remaining
        .find(|p: char| !p.is_ascii_digit())
        .unwrap_or(remaining.len());
    if port_end == 0 {
        return None;
    }
    remaining[..port_end].parse().ok()
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<T> {
    match m.lock() {
        Ok(guard) => guard,
        Err(e) => e.into_inner(),
    }
}

fn do_spawn(is_secure: bool) -> io::Result<SpawnState> {
    let mut command = std::process::Command::new(get_binary_path());
    command.arg("--child_process");
    command.env("PORT", "0");

    let mut cert_dir = None;

    if is_secure {
        let cert_dir = cert_dir.insert(tempfile::tempdir()?);

        gen_certificate(cert_dir.path())?;

        command.env(
            "TLS_CERTIFICATE",
            std::fs::read_to_string(cert_dir.path().join("cert.pem"))?,
        );
        command.env(
            "TLS_PRIVATE_KEY",
            std::fs::read_to_string(cert_dir.path().join("key.pem"))?,
        );
    }

    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    kill_on_parent_termination(&mut command);

    let child = command.spawn()?;

    let mut child_stdin = child.stdin.unwrap();
    let consumed_stdout = Arc::new(Mutex::new(Vec::new()));

    let (port_send, port_recv) = mpsc::sync_channel(1);

    {
        let consumed_stdout = Arc::clone(&consumed_stdout);
        let mut child_stdout = child.stdout.unwrap();
        let child_stderr = child.stderr.unwrap();

        std::thread::spawn(move || {
            let mut reader = std::io::BufReader::new(child_stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() {
                if let Some(port) = try_read_port(&line) {
                    let _ = port_send.send(port);
                }
                eprintln!("{line}");
                line.clear();
            }
        });

        std::thread::spawn(move || {
            let mut chunk = [0_u8; 65536];

            loop {
                let read = child_stdout.read(&mut chunk).unwrap();

                if read == 0 {
                    break;
                }

                lock(&consumed_stdout).extend_from_slice(&chunk[..read]);
            }
        });
    }

    let port = port_recv
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("Spawn timed out");

    child_stdin.write_all(&[
        0x01, 16, b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c',
        b'd', b'e', b'f',
    ])?;

    Ok(SpawnState {
        port,
        child_stdin,
        consumed_stdout,
        cert_dir,
    })
}

impl Server {
    pub const fn new(is_secure: bool) -> Self {
        Self {
            is_secure,
            state: std::sync::OnceLock::new(),
        }
    }

    fn spawn_state(&self) -> &SpawnState {
        let is_secure = self.is_secure;
        self.state
            .get_or_init(move || do_spawn(is_secure))
            .as_ref()
            .unwrap()
    }

    pub fn send_request(&self, route: &str, additional_curl_args: &[&str]) -> Request {
        let spawn_state = self.spawn_state();

        let mut command = std::process::Command::new("curl");

        if let Some(dir) = &spawn_state.cert_dir {
            command.arg(format!("https://localhost:{}{}", spawn_state.port, route));
            command.arg("--cacert");
            command.arg(dir.path().join("cert.pem"));
            command.arg("--pinnedpubkey");
            command.arg(dir.path().join("pub.pem"));
        } else {
            command.arg(format!("http://localhost:{}", spawn_state.port));
        }

        command.arg("--max-time");
        command.arg("5");
        command.args(additional_curl_args);
        command.arg("--include");
        command.arg("--no-fail");

        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::inherit());

        kill_on_parent_termination(&mut command);

        let child = command.spawn().unwrap();

        Request { child }
    }

    pub fn write_stdin(&self, input: &[u8]) {
        let spawn_state = self.spawn_state();
        (&spawn_state.child_stdin).write_all(input).unwrap();
    }

    pub fn assert_stdout(&self, expected: &[u8]) {
        let stdout = std::mem::take(&mut *lock(&self.spawn_state().consumed_stdout));
        if stdout != expected {
            panic!(
                "  Actual: {:?}\nExpected: {:?}",
                DebugBigSlice(&stdout),
                DebugBigSlice(expected)
            );
        }
    }
}

pub struct Request {
    child: std::process::Child,
}

impl Request {
    #[track_caller]
    pub fn assert_response(self, mut response: &[u8]) {
        if let [b'\n', ref rest @ ..] = response {
            response = rest;
        }

        let mut output = self.child.wait_with_output().unwrap();

        assert!(
            output.status.success(),
            "curl command failed with code {:?}, signal {:?}",
            output.status.code(),
            output.status.signal(),
        );

        output.stdout.retain(|c| *c != b'\r');
        assert_eq!(output.stdout, response);
    }
}
