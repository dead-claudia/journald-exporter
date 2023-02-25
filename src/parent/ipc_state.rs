use crate::prelude::*;

use super::key_watcher::KeyWatcherTarget;
use super::types::ParentIpcMethods;
use std::path::PathBuf;

pub struct ParentIpcDynamic {
    child_uid: Uid,
    child_gid: Gid,
    args: Vec<String>,
    prom_environment: PromEnvironment,
    key_target: KeyWatcherTarget,
}

impl ParentIpcDynamic {
    pub fn child_uid(&'static self) -> Uid {
        self.child_uid
    }

    pub fn child_gid(&'static self) -> Gid {
        self.child_gid
    }

    pub fn args(&'static self) -> &'static Vec<String> {
        &self.args
    }

    pub fn prom_environment(&'static self) -> &'static PromEnvironment {
        &self.prom_environment
    }

    pub fn key_target(&'static self) -> &'static KeyWatcherTarget {
        &self.key_target
    }
}

pub struct ParentIpcState<M: ParentIpcMethods> {
    dynamic: OnceCell<ParentIpcDynamic>,
    state: PromState,
    command: &'static str,
    methods: M,
    terminate_notify: Notify,
    done_notify: Notify,
    decoder: Uncontended<ipc::child::Decoder>,
    child_input: Mutex<Option<M::ChildInput>>,
}

impl<M: ParentIpcMethods> ParentIpcState<M> {
    pub const fn new(command: &'static str, methods: M) -> ParentIpcState<M> {
        ParentIpcState {
            dynamic: OnceCell::new(),
            state: PromState::new(),
            command,
            methods,
            terminate_notify: Notify::new(),
            done_notify: Notify::new(),
            child_input: Mutex::new(None),
            decoder: Uncontended::new(ipc::child::Decoder::new()),
        }
    }

    pub fn methods(&'static self) -> &'static M {
        &self.methods
    }

    pub fn terminate_notify(&'static self) -> &'static Notify {
        &self.terminate_notify
    }

    pub fn done_notify(&'static self) -> &'static Notify {
        &self.done_notify
    }

    #[cold]
    pub fn init_dynamic(
        &'static self,
        child_uid: Uid,
        child_gid: Gid,
        args: Vec<String>,
        prom_environment: PromEnvironment,
        key_dir: PathBuf,
    ) {
        let dynamic = ParentIpcDynamic {
            child_uid,
            child_gid,
            args,
            prom_environment,
            key_target: KeyWatcherTarget::new(key_dir),
        };

        if self.dynamic.set(dynamic).is_err() {
            panic!("State already initialized.");
        }
    }

    pub fn state(&'static self) -> &'static PromState {
        &self.state
    }

    pub fn command(&'static self) -> &'static str {
        self.command
    }

    pub fn decoder(&'static self) -> &'static Uncontended<ipc::child::Decoder> {
        &self.decoder
    }

    pub fn dynamic(&'static self) -> &'static ParentIpcDynamic {
        self.dynamic.get().expect("Dynamic data not initialized")
    }

    pub fn child_input(&'static self) -> MutexGuard<'static, Option<M::ChildInput>> {
        self.child_input.lock().unwrap_or_else(|e| e.into_inner())
    }
}
