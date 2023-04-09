use crate::prelude::*;

use super::ParentIpcMethods;
use crate::parent::key_watcher::KeyWatcherTarget;
use std::path::PathBuf;

pub struct ParentIpcDynamic {
    child_user_group: UserGroup,
    args: Box<[Box<str>]>,
    prom_environment: PromEnvironment,
    key_target: KeyWatcherTarget,
}

impl ParentIpcDynamic {
    pub fn child_user_group(&'static self) -> &'static UserGroup {
        &self.child_user_group
    }

    pub fn args(&'static self) -> &'static [Box<str>] {
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
        child_user_group: UserGroup,
        args: Box<[Box<str>]>,
        prom_environment: PromEnvironment,
        key_dir: PathBuf,
    ) {
        let dynamic = ParentIpcDynamic {
            child_user_group,
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
