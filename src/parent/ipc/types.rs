use crate::prelude::*;

use super::ParentIpcState;
use crate::ffi::ExitResult;
use crate::ffi::Pollable;

pub trait ParentIpcMethods: Send + Sync + Sized {
    type ChildInput: Pollable + Write + Unpin + Send;
    type ChildOutput: Pollable + Read + Unpin + Send;

    fn next_instant(&'static self) -> Instant;

    fn get_user_group_table(&'static self) -> io::Result<Arc<UidGidTable>>;

    fn child_spawn(
        &'static self,
        ipc_state: &'static ParentIpcState<Self>,
    ) -> io::Result<(Self::ChildInput, Self::ChildOutput)>;

    fn child_terminate(&'static self) -> io::Result<()>;

    fn child_wait(&self) -> IpcExitStatus;
}

#[derive(Debug)]
pub struct IpcExitStatus {
    pub result: Option<ExitResult>,
    pub parent_error: Option<Error>,
    pub child_wait_error: Option<Error>,
}

// Only implement it for test. In prod, it should never be cloned like this. Some tests do rely on
// this definition, though, and it's a lot more convenient to implement it here than to add a
// `use propcheck::Arbitrary as _` before about every use.
#[cfg(test)]
impl Clone for IpcExitStatus {
    fn clone(&self) -> Self {
        propcheck::Arbitrary::clone(self)
    }
}

#[cfg(test)]
impl PartialEq for IpcExitStatus {
    fn eq(&self, other: &Self) -> bool {
        self.result == other.result
            && match (&self.parent_error, &other.parent_error) {
                (None, None) => true,
                (Some(a), Some(b)) => error_eq(a, b),
                _ => false,
            }
            && match (&self.child_wait_error, &other.child_wait_error) {
                (None, None) => true,
                (Some(a), Some(b)) => error_eq(a, b),
                _ => false,
            }
    }
}

#[cfg(test)]
impl Eq for IpcExitStatus {}

#[cfg(test)]
pub struct IpcExitStatusShrinker(
    propcheck::Tuple3Shrinker<Option<ExitResult>, Option<Error>, Option<Error>>,
);

#[cfg(test)]
impl propcheck::Shrinker for IpcExitStatusShrinker {
    type Item = IpcExitStatus;

    fn next(&mut self) -> Option<&Self::Item> {
        // SAFETY: same layout
        unsafe { std::mem::transmute(self.0.next()) }
    }
}

#[cfg(test)]
impl propcheck::Arbitrary for IpcExitStatus {
    type Shrinker = IpcExitStatusShrinker;

    fn arbitrary() -> Self {
        let (result, parent_error, child_wait_error) =
            <(Option<ExitResult>, Option<Error>, Option<Error>)>::arbitrary();

        Self {
            result,
            parent_error,
            child_wait_error,
        }
    }

    fn clone(&self) -> Self {
        Self {
            result: self.result,
            parent_error: self.parent_error.clone(),
            child_wait_error: self.child_wait_error.clone(),
        }
    }

    fn shrink(&self) -> Self::Shrinker {
        // SAFETY: same layout
        IpcExitStatusShrinker(unsafe {
            std::mem::transmute::<_, &(Option<ExitResult>, Option<Error>, Option<Error>)>(self)
                .shrink()
        })
    }
}

// Skip all these tests in Miri. They take a while and are just testing test utilities.
#[cfg(all(test, not(miri)))]
#[allow(clippy::as_conversions)]
mod tests {
    use super::*;

    use crate::ffi::ExitCode;
    use crate::ffi::Signal;

    #[test]
    fn ipc_exit_status_equality_is_reflexive() {
        propcheck::run(|a: &IpcExitStatus| a == a);
    }

    #[test]
    fn ipc_exit_status_equality_is_symmetric() {
        propcheck::run(|[a, b]: &[IpcExitStatus; 2]| (a == b) == (b == a));
    }

    #[test]
    fn ipc_exit_status_equality_is_transitive() {
        propcheck::run(|[a, b, c]: &[IpcExitStatus; 3]| {
            if a == b && b == c {
                a == c
            } else {
                true // just pass the test
            }
        });
    }

    #[test]
    fn ipc_exit_status_assertion_works_on_pass() {
        assert_eq!(
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(0))),
                parent_error: None,
                child_wait_error: None
            },
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(0))),
                parent_error: None,
                child_wait_error: None
            },
        )
    }

    #[test]
    fn ipc_exit_status_assertion_works_on_fail() {
        assert_ne!(
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(0))),
                parent_error: None,
                child_wait_error: None
            },
            IpcExitStatus {
                result: Some(ExitResult::Signal(Signal::SIGINT)),
                parent_error: Some(ErrorKind::NotFound.into()),
                child_wait_error: None,
            },
        );
    }
}
