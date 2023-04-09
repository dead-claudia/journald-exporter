use crate::prelude::*;
use std::cell::UnsafeCell;

struct LogStderr;
static LOG_STDERR: LogStderr = LogStderr;

impl log::Log for LogStderr {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    // This intentionally prints to stderr.
    #[allow(clippy::print_stderr)]
    fn log(&self, record: &log::Record) {
        eprintln!(
            "{} {}:{}: {}",
            record.level(),
            record.file().unwrap_or("unknown"),
            record.line().map_or("???".into(), |line| line.to_string()),
            record.args()
        );
    }

    fn flush(&self) {
        io::Write::flush(&mut io::stderr()).unwrap();
    }
}

struct LogCapture {
    // Using a `String` so it'll compare against `&str` correctly. Not sure why `PartialEq<&T>`
    // isn't implemented for `Box<T>` or vice versa.
    inner: Mutex<Vec<String>>,
}

impl log::Log for LogCapture {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(record.args().to_string())
    }

    fn flush(&self) {
        // ignore
    }
}

enum LoggerKind {
    LogStderr,
    LoggerVec(LogCapture),
    LogProxy(TestLoggerState),
}

#[derive(Clone)]
struct TestLoggerState {
    kind: Arc<Mutex<LoggerKind>>,
}

impl TestLoggerState {
    fn kind(&self) -> MutexGuard<LoggerKind> {
        self.kind.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn with_logger<R>(&self, f: impl Fn(&dyn log::Log) -> R) -> R {
        match &*self.kind() {
            LoggerKind::LogStderr => f(&LOG_STDERR),
            LoggerKind::LoggerVec(logger) => f(logger),
            LoggerKind::LogProxy(logger) => logger.with_logger(f),
        }
    }
}

thread_local! {
    static LOGGER_STATE: TestLoggerState = TestLoggerState {
        kind: Arc::new(Mutex::new(LoggerKind::LogStderr)),
    };
}

struct TestLogger;

static TEST_LOGGER: TestLogger = TestLogger;

impl log::Log for TestLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        LOGGER_STATE.with(|state| state.with_logger(|l| l.enabled(metadata)))
    }

    fn log(&self, record: &log::Record) {
        LOGGER_STATE.with(|state| state.with_logger(|l| l.log(record)))
    }

    fn flush(&self) {
        LOGGER_STATE.with(|state| state.with_logger(|l| l.flush()))
    }
}

// I'd also include an `impl !Send for LoggerGuard {}` here, but that's pending a feature that's
// been pending stabilization for years: https://github.com/rust-lang/rust/issues/68318
pub struct LoggerGuard(LoggerKind, UnsafeCell<()>);
pub struct LoggerProxyHandle(TestLoggerState);

impl LoggerGuard {
    fn new(kind: LoggerKind) -> Self {
        LOGGER_STATE.with(|state| Self(replace(&mut *state.kind(), kind), UnsafeCell::new(())))
    }

    pub fn proxy(&self) -> LoggerProxyHandle {
        LoggerProxyHandle(LOGGER_STATE.with(|state| state.clone()))
    }

    pub fn spawn(&self, task: impl FnOnce() -> io::Result<()> + Send + 'static) -> ThreadHandle {
        let proxy = self.proxy();
        ThreadHandle::spawn(Box::new(move || {
            let _guard = proxy.install();
            task()
        }))
    }
}

impl LoggerProxyHandle {
    pub fn install(self) -> LoggerGuard {
        LoggerGuard::new(LoggerKind::LogProxy(self.0))
    }
}

impl Drop for LoggerGuard {
    fn drop(&mut self) {
        let kind = replace(&mut self.0, LoggerKind::LogStderr);
        LOGGER_STATE.with(|state| *state.kind() = kind);
    }
}

#[must_use]
pub struct LoggerCaptureGuard(LoggerGuard);

impl LoggerCaptureGuard {
    #[track_caller]
    pub fn expect_logs(self, lines: &[&'static str]) {
        // FIXME: figure out why logs aren't appearing in Miri. It works in `cargo test`.
        if cfg!(miri) {
            return;
        }

        LOGGER_STATE.with(|state| match &*state.kind() {
            LoggerKind::LoggerVec(v) => {
                let guard = v.inner.lock().unwrap_or_else(|e| e.into_inner());
                assert_eq!(&*guard, lines);
            }
            _ => panic!("Logger is not a capture logger"),
        });
    }
}

impl std::ops::Deref for LoggerCaptureGuard {
    type Target = LoggerGuard;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Initialize this for all tests. Easier than trying to pump it through literally everything, and
// I can also use this to track down any/all stray logs.
#[ctor::ctor]
fn init_logger() {
    // Don't log `trace!(...)` stuff.
    if std::env::var("RUST_TRACE") == Ok("1".into()) {
        log::set_max_level(log::LevelFilter::Trace);
    } else {
        log::set_max_level(log::LevelFilter::Debug);
    }
    log::set_logger(&TEST_LOGGER).unwrap();
}

#[must_use = "The returned guard must be live for the whole test to ensure all logs are captured."]
pub fn setup_capture_logger() -> LoggerCaptureGuard {
    LoggerCaptureGuard(LoggerGuard::new(LoggerKind::LoggerVec(LogCapture {
        inner: Mutex::new(Vec::new()),
    })))
}
