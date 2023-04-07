use crate::prelude::*;

use super::ipc::write_to_child_input;
use super::ipc::ParentIpcMethods;
use super::ipc::ParentIpcState;
use crate::ffi::current_uid;
use notify::recommended_watcher;
use notify::Watcher;
use std::os::unix::prelude::MetadataExt;
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;

pub struct KeyWatcherTarget {
    key_dir: PathBuf,
}

impl KeyWatcherTarget {
    pub const fn new(key_dir: PathBuf) -> KeyWatcherTarget {
        KeyWatcherTarget { key_dir }
    }

    fn get_current_key_set(&self) -> io::Result<Box<[u8]>> {
        let mut next = Vec::new();

        let dir_metadata = std::fs::metadata(&self.key_dir)?;

        fn perms_are_insecure(metadata: &std::fs::Metadata) -> bool {
            metadata.mode() & (GROUP_WO | OTHER_WO) != 0
        }

        if perms_are_insecure(&dir_metadata) {
            return Err(report_insecure(&self.key_dir, dir_metadata));
        }

        for entry in std::fs::read_dir(&self.key_dir)? {
            let entry = entry?;
            match process_path(entry) {
                Ok(key) => next.push(key),
                // Tolerate it. May have just been removed right after the threshold.
                Err(e) if e.kind() == ErrorKind::NotFound => {}
                // Log it and silently ignore it. Easier to debug and less intrusive overall, and
                // also by side effect plugs a potential DoS hole in case arbitrary files get added
                // to it by a non-root process.
                Err(e) => log::error!("Key watcher error: {}", &normalize_errno(e, None)),
            }
        }

        Ok(ipc::parent::receive_key_set_bytes(KeySet::new(next.into())))
    }
}

#[must_use]
pub fn write_current_key_set(s: &'static ParentIpcState<impl ParentIpcMethods>) -> bool {
    match s.dynamic().key_target().get_current_key_set() {
        // If it's terminating, it's pointless to report errors here. Also suppresses a bunch of
        // useless error logs in the tests.
        Err(_) if s.child_input().is_none() => false,
        Err(e) => {
            log::error!("Key watcher error: {}", &normalize_errno(e, None));
            // Just in case something changed in the meantime.
            s.child_input().is_none()
        }
        Ok(msg) => write_to_child_input(s, &msg),
    }
}

fn to_io_error(error: notify::Error) -> Error {
    match error.kind {
        notify::ErrorKind::Io(e) => e,
        _ => string_err(error.to_string()),
    }
}

// Assert not writable by group or other, and not readable by group.
const OTHER_RO: u32 = libc::S_IROTH;
const OTHER_WO: u32 = libc::S_IWOTH;
const OTHER_XO: u32 = libc::S_IXOTH;
const OTHER_RW: u32 = libc::S_IROTH | libc::S_IWOTH;
const OTHER_RX: u32 = libc::S_IROTH | libc::S_IXOTH;
const OTHER_WX: u32 = libc::S_IWOTH | libc::S_IXOTH;
const OTHER_RWX: u32 = libc::S_IROTH | libc::S_IWOTH | libc::S_IXOTH;

const GROUP_RO: u32 = libc::S_IRGRP;
const GROUP_WO: u32 = libc::S_IWGRP;
const GROUP_XO: u32 = libc::S_IXGRP;
const GROUP_RW: u32 = libc::S_IRGRP | libc::S_IWGRP;
const GROUP_RX: u32 = libc::S_IRGRP | libc::S_IXGRP;
const GROUP_WX: u32 = libc::S_IWGRP | libc::S_IXGRP;
const GROUP_RWX: u32 = libc::S_IRGRP | libc::S_IWGRP | libc::S_IXGRP;

fn insecure_message(metadata: std::fs::Metadata) -> &'static str {
    let perm_bits = metadata.permissions().mode();
    match perm_bits & OTHER_RWX {
        OTHER_RO => "readable by everyone",
        OTHER_WO => "writable by everyone",
        OTHER_XO => "executable by everyone",
        OTHER_RW => "readable and writable by everyone",
        OTHER_RX => "readable and executable by everyone",
        OTHER_WX => "writable and executable by everyone",
        OTHER_RWX => "readable, writable, and executable by everyone",
        _ => match perm_bits & GROUP_RWX {
            GROUP_RO => "readable by everyone in owning group",
            GROUP_WO => "writable by everyone in owning group",
            GROUP_XO => "executable by everyone in owning group",
            GROUP_RW => "readable and writable by everyone in owning group",
            GROUP_RX => "readable and executable by everyone in owning group",
            GROUP_WX => "writable and executable by everyone in owning group",
            GROUP_RWX => "readable, writable, and executable by everyone in owning group",
            _ => unreachable!(),
        },
    }
}

// Only show the immediate path in test as it's only seeing a temporary directory, but show
// everything in release for easier debugging.
fn resolve_file_name(entry: std::fs::DirEntry) -> std::path::PathBuf {
    if cfg!(test) {
        entry.file_name().into()
    } else {
        entry.path()
    }
}

#[cold]
fn report_insecure(path: &std::path::Path, metadata: std::fs::Metadata) -> Error {
    string_err(format!(
        "{} has insecure permissions: {}",
        path.display(),
        insecure_message(metadata),
    ))
}

fn process_path(entry: std::fs::DirEntry) -> io::Result<Key> {
    let metadata = entry.metadata()?;

    if !metadata.file_type().is_file() {
        return Err(string_err(format!(
            "{} is not a file",
            resolve_file_name(entry).display()
        )));
    }

    // Make polymorphic based on the current user. Easier to clean up that way in test, and in prod
    // it always checks against the root user anyways.
    if metadata.uid() != current_uid() {
        return Err(string_err(format!(
            "{} is not owned by the root user",
            resolve_file_name(entry).display()
        )));
    }

    fn perms_are_insecure(metadata: &std::fs::Metadata) -> bool {
        metadata.mode() & (GROUP_RWX | OTHER_RWX) != 0
    }

    if perms_are_insecure(&metadata) {
        return Err(report_insecure(&resolve_file_name(entry), metadata));
    }

    let key_data = std::fs::read_to_string(entry.path())?;
    Key::from_hex(key_data.trim().as_bytes()).ok_or_else(|| {
        string_err(format!(
            "{} does not contain a valid key",
            resolve_file_name(entry).display()
        ))
    })
}

fn is_update_event(event: &notify::Event) -> bool {
    use notify::event::*;

    matches!(
        event.kind,
        EventKind::Any
            | EventKind::Create(CreateKind::Any)
            | EventKind::Create(CreateKind::File)
            | EventKind::Modify(ModifyKind::Any)
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any))
            | EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime))
            | EventKind::Modify(ModifyKind::Name(_))
            | EventKind::Remove(RemoveKind::Any)
            | EventKind::Remove(RemoveKind::File)
    )
}

#[derive(Debug)]
enum EventState {
    None,
    Error(Vec<notify::Error>),
    Event,
    ErrorAndEvent(Vec<notify::Error>),
    Drop,
    ErrorAndDrop(Vec<notify::Error>),
    Locked,
}

struct EventHandler(Arc<Checkpoint<EventState>>);

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.0
            .notify(|state| match replace(state, EventState::Drop) {
                EventState::None => {}
                EventState::Error(errors) => *state = EventState::ErrorAndDrop(errors),
                EventState::Event => {}
                EventState::ErrorAndEvent(errors) => *state = EventState::ErrorAndDrop(errors),
                EventState::Drop => {}
                EventState::ErrorAndDrop(errors) => *state = EventState::ErrorAndDrop(errors),
                EventState::Locked => {}
            });
    }
}

impl notify::EventHandler for EventHandler {
    fn handle_event(&mut self, event: notify::Result<notify::Event>) {
        self.0.notify(|state| match (event, state) {
            (Ok(event), state @ EventState::None) if is_update_event(&event) => {
                *state = EventState::Event;
            }
            (Ok(event), state @ EventState::Error(_)) if is_update_event(&event) => {
                let EventState::Error(errors) = replace(&mut *state, EventState::Locked) else {
                    unreachable!();
                };
                *state = EventState::ErrorAndEvent(errors);
            }
            (Ok(_), _) => {}
            (Err(e), state @ EventState::None) => {
                *state = EventState::Error(vec![e]);
            }
            (Err(e), state @ EventState::Event) => {
                *state = EventState::ErrorAndEvent(vec![e]);
            }
            (Err(e), state @ EventState::Drop) => {
                *state = EventState::ErrorAndDrop(vec![e]);
            }
            (
                Err(e),
                EventState::Error(errors)
                | EventState::ErrorAndEvent(errors)
                | EventState::ErrorAndDrop(errors),
            ) => {
                errors.push(e);
            }
            (Err(_), EventState::Locked) => {}
        });
    }
}

pub fn run_watcher(s: &'static ParentIpcState<impl ParentIpcMethods>) -> io::Result<()> {
    let checkpoint = Arc::new(Checkpoint::new(EventState::None));

    let mut notify_watcher =
        recommended_watcher(EventHandler(Arc::clone(&checkpoint))).map_err(to_io_error)?;

    notify_watcher
        .watch(
            &s.dynamic().key_target().key_dir,
            notify::RecursiveMode::NonRecursive,
        )
        .map_err(to_io_error)?;

    // To handle non-atomic writes correctly.
    const ATOMIC_DEBOUNCE_TIMEOUT: Duration = Duration::from_millis(100);
    const TERMINATE_TIMEOUT: Duration = Duration::from_secs(1);

    let mut has_update = false;

    while !s.terminate_notify().has_notified() {
        let timeout = if has_update {
            ATOMIC_DEBOUNCE_TIMEOUT
        } else {
            TERMINATE_TIMEOUT
        };

        let mut guard = checkpoint.wait_for(timeout);

        let state = replace(&mut *guard, EventState::Locked);

        *guard = match &state {
            EventState::Drop | EventState::ErrorAndDrop(_) | EventState::Locked => {
                EventState::Locked
            }
            _ => EventState::None,
        };

        drop(guard);

        fn report_errors(errors: &[notify::Error]) {
            for e in errors {
                log::error!("Key watcher error: {e}");
            }
        }

        match state {
            EventState::None => {
                if replace(&mut has_update, false) && !write_current_key_set(s) {
                    break;
                }
            }
            EventState::Error(errors) => {
                report_errors(&errors);
                if replace(&mut has_update, false) && !write_current_key_set(s) {
                    break;
                }
            }
            EventState::Event => has_update = true,
            EventState::ErrorAndEvent(errors) => {
                report_errors(&errors);
                has_update = true;
            }
            // Watcher dropped or some other error occurred.
            EventState::Drop => break,
            // Watcher dropped or some other error occurred.
            EventState::ErrorAndDrop(errors) => {
                report_errors(&errors);
                break;
            }
            // Checked for earlier.
            EventState::Locked => break,
        }
    }

    Ok(())
}

// Skip in Miri. Until the complex filesystem interactions can be shimmed, it's worthless to even
// try.
#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;

    use crate::parent::ipc::test_utils::*;
    use std::fs::File;

    struct TestKeyState {
        start_checkpoint: ThreadCheckpoint,
        static_state: StaticState,
    }

    struct TestRuntimeState {
        key_dir: tempfile::TempDir,
        // The guard must be dropped before the handle.
        _watcher_guard: NotifyGuard<'static>,
        _watcher_handle: ThreadHandle,
    }

    const NO_FILES: u8 = 0;
    const FILE_TEST_KEY: u8 = 1 << 0;
    const FILE_TEST_KEY_2: u8 = 1 << 1;
    const FILE_OTHER_KEY: u8 = 1 << 2;

    fn atomic_prepare(contents: &[u8]) -> tempfile::NamedTempFile {
        atomic_prepare_mode(0o600, contents)
    }

    fn atomic_prepare_mode(mode: u32, contents: &[u8]) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(mode))
            .unwrap();
        file.write_all(contents).unwrap();
        file
    }

    fn to_key_set(files: u8) -> Vec<u8> {
        let mut expected = Vec::new();

        fn select_file(files: u8, flag: u8, contents: &[u8]) -> &[u8] {
            if (files & flag) != 0 {
                contents
            } else {
                b""
            }
        }

        write_slices(
            &mut expected,
            &[
                &[0x01, truncate_u32_u8(files.count_ones())],
                select_file(files, FILE_TEST_KEY, b"\x0F0123456789abcdef"),
                select_file(files, FILE_TEST_KEY_2, b"\x0F76543210fedcba98"),
                select_file(files, FILE_OTHER_KEY, b"\x0Ffedcba9876543210"),
            ],
        );

        expected
    }

    impl TestKeyState {
        const fn new() -> Self {
            Self {
                start_checkpoint: ThreadCheckpoint::new(),
                static_state: StaticState::new(),
            }
        }

        fn start(&'static self, guard: &LoggerGuard) -> TestRuntimeState {
            let key_dir = tempfile::tempdir().unwrap();

            std::fs::set_permissions(key_dir.path(), std::fs::Permissions::from_mode(0o755))
                .unwrap();

            let _checkpoint_guard = self.start_checkpoint.drop_guard();

            // Spawn before creating the guard so the guard gets dropped first, but synchronize on a
            // checkpoint to ensure it's actually registered first.
            let watcher_handle = guard.spawn(Box::new(|| {
                if self.start_checkpoint.try_wait() {
                    run_watcher(&self.static_state.state)
                } else {
                    Ok(())
                }
            }));

            let watcher_guard = self.static_state.state.terminate_notify().create_guard();
            self.static_state
                .init_test_state_with_key_dir(key_dir.path().to_owned());
            self.start_checkpoint.resume();

            // Wait for the listener to get registered.
            std::thread::sleep(Duration::from_millis(250));

            TestRuntimeState {
                key_dir,
                _watcher_guard: watcher_guard,
                _watcher_handle: watcher_handle,
            }
        }
    }

    struct TestManipulate {
        state: [TestKeyState; 3],
        index: AtomicUsize,
    }

    impl TestManipulate {
        const fn new() -> Self {
            Self {
                state: [
                    TestKeyState::new(),
                    TestKeyState::new(),
                    TestKeyState::new(),
                ],
                index: AtomicUsize::new(0),
            }
        }

        fn run(
            &'static self,
            files: u8,
            setup: Option<&dyn Fn(&TestRuntimeState)>,
            update: &dyn Fn(&TestRuntimeState),
        ) {
            with_attempts(3, 0.5, &|| {
                let index = self.index.fetch_add(1, Ordering::AcqRel);
                let guard = setup_capture_logger();
                let t = &self.state[index];
                let rt = t.start(&guard);

                if let Some(f) = setup {
                    f(&rt);
                }

                let expected = to_key_set(files);
                let state = &t.static_state.state;

                let target = &state.methods().child_input;
                target.reset_data_written();
                target.enqueue_write_ok(expected.len());

                let _stdin_lease = t.static_state.connect_stdin();

                update(&rt);
                std::thread::sleep(Duration::from_millis(500));

                t.static_state.assert_input_sent(&expected);
                guard.expect_logs(&[]);
                t.static_state.assert_no_calls_remaining();
            });
        }
    }

    struct TestMode {
        name: &'static str,
        mode: u32,
        logs: &'static [&'static str],
        state: [TestKeyState; 3],
        index: AtomicUsize,
    }

    impl TestMode {
        const fn new(name: &'static str, mode: u32, logs: &'static [&'static str]) -> Self {
            Self {
                name,
                mode,
                logs,
                state: [
                    TestKeyState::new(),
                    TestKeyState::new(),
                    TestKeyState::new(),
                ],
                index: AtomicUsize::new(0),
            }
        }

        fn run(&'static self) {
            with_attempts(3, 0.5, &|| {
                let index = self.index.fetch_add(1, Ordering::AcqRel);
                let guard = setup_capture_logger();
                let rt = self.state[index].start(&guard);

                rt.atomic_persist(
                    self.name,
                    atomic_prepare_mode(self.mode, b"0123456789abcdef"),
                );
                std::thread::sleep(Duration::from_millis(500));

                guard.expect_logs(self.logs);
                self.state[index].static_state.assert_no_calls_remaining();
            });
        }
    }

    impl TestRuntimeState {
        fn remove_key(&self, name: &str) {
            std::fs::remove_file(self.key_dir.path().join(name)).unwrap();
        }

        fn non_atomic_write_key(&self, name: &str, contents: &[u8]) {
            let mut file = File::create(self.key_dir.path().join(name)).unwrap();
            file.set_permissions(std::fs::Permissions::from_mode(0o600))
                .unwrap();
            file.write_all(contents).unwrap();
        }

        fn atomic_persist(&self, name: &str, file: tempfile::NamedTempFile) {
            file.persist(self.key_dir.path().join(name)).unwrap();
        }
    }

    #[test]
    fn observes_non_atomic_add_one_from_empty() {
        static T: TestManipulate = TestManipulate::new();
        T.run(FILE_TEST_KEY, None, &|rt| {
            rt.non_atomic_write_key("test.key", b"0123456789abcdef");
        });
    }

    #[test]
    fn observes_non_atomic_add_one_from_non_empty() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            FILE_TEST_KEY | FILE_OTHER_KEY,
            Some(&|rt| {
                rt.non_atomic_write_key("test.key", b"0123456789abcdef");
            }),
            &|rt| {
                rt.non_atomic_write_key("other.key", b"fedcba9876543210");
            },
        );
    }

    #[test]
    fn observes_non_atomic_add_two_from_non_empty() {
        static T: TestManipulate = TestManipulate::new();
        T.run(FILE_TEST_KEY | FILE_OTHER_KEY, None, &|rt| {
            rt.non_atomic_write_key("test.key", b"0123456789abcdef");
            rt.non_atomic_write_key("other.key", b"fedcba9876543210");
        });
    }

    #[test]
    fn observes_non_atomic_update_without_other_files() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            FILE_TEST_KEY_2,
            Some(&|rt| {
                rt.non_atomic_write_key("test.key", b"0123456789abcdef");
            }),
            &|rt| {
                rt.non_atomic_write_key("test.key", b"76543210fedcba98");
            },
        );
    }

    #[test]
    fn observes_non_atomic_update_with_other_files() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            FILE_TEST_KEY_2 | FILE_OTHER_KEY,
            Some(&|rt| {
                rt.non_atomic_write_key("test.key", b"0123456789abcdef");
                rt.non_atomic_write_key("other.key", b"fedcba9876543210");
            }),
            &|rt| {
                rt.non_atomic_write_key("test.key", b"76543210fedcba98");
            },
        );
    }

    #[test]
    fn observes_atomic_add_one_from_empty() {
        static T: TestManipulate = TestManipulate::new();
        T.run(FILE_TEST_KEY, None, &|rt| {
            rt.atomic_persist("test.key", atomic_prepare(b"0123456789abcdef"));
        });
    }

    #[test]
    fn observes_atomic_add_one_from_non_empty() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            FILE_TEST_KEY | FILE_OTHER_KEY,
            Some(&|rt| {
                rt.atomic_persist("test.key", atomic_prepare(b"0123456789abcdef"));
            }),
            &|rt| {
                rt.atomic_persist("other.key", atomic_prepare(b"fedcba9876543210"));
            },
        );
    }

    #[test]
    fn observes_atomic_add_two_from_non_empty() {
        static T: TestManipulate = TestManipulate::new();
        T.run(FILE_TEST_KEY | FILE_OTHER_KEY, None, &|rt| {
            rt.atomic_persist("test.key", atomic_prepare(b"0123456789abcdef"));
            rt.atomic_persist("other.key", atomic_prepare(b"fedcba9876543210"));
        });
    }

    #[test]
    fn observes_atomic_update_without_other_files() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            FILE_TEST_KEY_2,
            Some(&|rt| {
                rt.atomic_persist("test.key", atomic_prepare(b"0123456789abcdef"));
            }),
            &|rt| {
                rt.atomic_persist("test.key", atomic_prepare(b"76543210fedcba98"));
            },
        );
    }

    #[test]
    fn observes_atomic_update_with_other_files() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            FILE_TEST_KEY_2 | FILE_OTHER_KEY,
            Some(&|rt| {
                rt.atomic_persist("test.key", atomic_prepare(b"0123456789abcdef"));
                rt.atomic_persist("other.key", atomic_prepare(b"fedcba9876543210"));
            }),
            &|rt| {
                rt.atomic_persist("test.key", atomic_prepare(b"76543210fedcba98"));
            },
        );
    }

    #[test]
    fn observes_remove_one_with_no_remaining_file() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            NO_FILES,
            Some(&|rt| {
                rt.non_atomic_write_key("test.key", b"0123456789abcdef");
            }),
            &|rt| {
                rt.remove_key("test.key");
            },
        );
    }

    #[test]
    fn observes_remove_one_with_remaining_file() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            FILE_OTHER_KEY,
            Some(&|rt| {
                rt.non_atomic_write_key("test.key", b"0123456789abcdef");
                rt.non_atomic_write_key("other.key", b"fedcba9876543210");
            }),
            &|rt| {
                rt.remove_key("test.key");
            },
        );
    }

    #[test]
    fn observes_remove_two_with_no_remaining_files() {
        static T: TestManipulate = TestManipulate::new();
        T.run(
            NO_FILES,
            Some(&|rt| {
                rt.non_atomic_write_key("test.key", b"0123456789abcdef");
                rt.non_atomic_write_key("other.key", b"fedcba9876543210");
            }),
            &|rt| {
                rt.remove_key("test.key");
                rt.remove_key("other.key");
            },
        );
    }

    #[test]
    fn allows_mode_700() {
        static T: TestMode = TestMode::new("test.key", 0o700, &[]);
        T.run();
    }

    #[test]
    fn rejects_mode_670() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o670,
            &["Key watcher error: test.key has insecure permissions: readable, writable, and executable by everyone in owning group"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_660() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o660,
            &["Key watcher error: test.key has insecure permissions: readable and writable by everyone in owning group"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_650() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o650,
            &["Key watcher error: test.key has insecure permissions: readable and executable by everyone in owning group"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_640() {
        static T: TestMode = TestMode::new("test.key", 0o640, &["Key watcher error: test.key has insecure permissions: readable by everyone in owning group"]);
        T.run();
    }

    #[test]
    fn rejects_mode_630() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o630,
            &["Key watcher error: test.key has insecure permissions: writable and executable by everyone in owning group"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_620() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o620,
            &["Key watcher error: test.key has insecure permissions: writable by everyone in owning group"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_610() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o610,
            &["Key watcher error: test.key has insecure permissions: executable by everyone in owning group"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_607() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o607,
            &["Key watcher error: test.key has insecure permissions: readable, writable, and executable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_606() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o606,
            &["Key watcher error: test.key has insecure permissions: readable and writable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_605() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o605,
            &["Key watcher error: test.key has insecure permissions: readable and executable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_604() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o604,
            &["Key watcher error: test.key has insecure permissions: readable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_603() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o603,
            &["Key watcher error: test.key has insecure permissions: writable and executable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_602() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o602,
            &["Key watcher error: test.key has insecure permissions: writable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_601() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o601,
            &["Key watcher error: test.key has insecure permissions: executable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_677() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o677,
            &["Key watcher error: test.key has insecure permissions: readable, writable, and executable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_676() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o676,
            &["Key watcher error: test.key has insecure permissions: readable and writable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_675() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o675,
            &["Key watcher error: test.key has insecure permissions: readable and executable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_674() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o674,
            &["Key watcher error: test.key has insecure permissions: readable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_673() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o673,
            &["Key watcher error: test.key has insecure permissions: writable and executable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_672() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o672,
            &["Key watcher error: test.key has insecure permissions: writable by everyone"],
        );
        T.run();
    }

    #[test]
    fn rejects_mode_671() {
        static T: TestMode = TestMode::new(
            "test.key",
            0o671,
            &["Key watcher error: test.key has insecure permissions: executable by everyone"],
        );
        T.run();
    }
}
