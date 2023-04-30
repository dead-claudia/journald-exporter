use crate::prelude::*;

use super::ParentIpcMethods;
use crate::parent::key_watcher::KeyWatcherTarget;
use std::ffi::OsStr;
use std::num::NonZeroU16;

pub struct TLSConfig {
    pub certificate: Box<OsStr>,
    pub private_key: Box<OsStr>,
}

pub struct ParentIpcDynamic {
    pub port: NonZeroU16,
    pub child_user_group: UserGroup,
    pub prom_environment: PromEnvironment,
    pub key_target: KeyWatcherTarget,
    pub tls_config: Option<TLSConfig>,
}

pub struct ParentIpcState<M: ParentIpcMethods> {
    dynamic: OnceCell<ParentIpcDynamic>,
    state: PromState,
    methods: M,
    terminate_notify: Notify,
    done_notify: Notify,
    decoder: Uncontended<ipc::child::Decoder>,
    child_input: Mutex<Option<M::ChildInput>>,
}

impl<M: ParentIpcMethods> ParentIpcState<M> {
    pub const fn new(methods: M) -> ParentIpcState<M> {
        ParentIpcState {
            dynamic: OnceCell::new(),
            state: PromState::new(),
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
    pub fn init_dynamic(&'static self, dynamic: ParentIpcDynamic) {
        if self.dynamic.set(dynamic).is_err() {
            panic!("State already initialized.");
        }
    }

    pub fn state(&'static self) -> &'static PromState {
        &self.state
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
