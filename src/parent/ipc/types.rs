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

// Only implement it for test, so it can satisfy `Arbitrary`. In prod, it should never be cloned
// like this.
#[cfg(test)]
impl Clone for IpcExitStatus {
    fn clone(&self) -> Self {
        Self {
            result: self.result,
            parent_error: self.parent_error.as_ref().map(error_clone),
            child_wait_error: self.child_wait_error.as_ref().map(error_clone),
        }
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
impl Arbitrary for IpcExitStatus {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            result: Arbitrary::arbitrary(g),
            parent_error: <bool>::arbitrary(g).then(|| error_arbitrary(g)),
            child_wait_error: <bool>::arbitrary(g).then(|| error_arbitrary(g)),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let result = self.result;
        let child_wait_error = self.child_wait_error.as_ref().map(error_clone);

        let parent_shrink = match &self.parent_error {
            None => {
                let iter: Box<dyn Iterator<Item = _>> = Box::new(std::iter::once(None));
                iter
            }
            Some(e) => Box::new(std::iter::once(None).chain(error_shrink(e).map(Some))),
        };

        let child_wait_shrink = parent_shrink.flat_map(move |parent| match &child_wait_error {
            None => {
                let iter: Box<dyn Iterator<Item = _>> = Box::new(std::iter::once((parent, None)));
                iter
            }
            Some(e) => Box::new(
                std::iter::once((parent.as_ref().map(error_clone), None)).chain(
                    error_shrink(e).map(move |child_wait| {
                        (parent.as_ref().map(error_clone), Some(child_wait))
                    }),
                ),
            ),
        });

        Box::new(child_wait_shrink.flat_map(move |(parent, child_wait)| {
            result.shrink().map(move |r| IpcExitStatus {
                result: r,
                parent_error: parent.as_ref().map(error_clone),
                child_wait_error: child_wait.as_ref().map(error_clone),
            })
        }))
    }
}

// Skip all these tests in Miri. They take a while and are just testing test utilities.
#[cfg(all(test, not(miri)))]
mod tests {
    use super::*;

    use crate::ffi::ExitCode;
    use crate::ffi::Signal;

    #[quickcheck]
    fn ipc_exit_status_equality_is_reflexive(a: IpcExitStatus) -> bool {
        a == a
    }

    #[quickcheck]
    fn ipc_exit_status_equality_is_symmetric(a: IpcExitStatus, b: IpcExitStatus) -> bool {
        (a == b) == (b == a)
    }

    #[quickcheck]
    fn ipc_exit_status_equality_is_transitive(
        a: IpcExitStatus,
        b: IpcExitStatus,
        c: IpcExitStatus,
    ) -> bool {
        if a == b && b == c {
            a == c
        } else {
            true // just pass the test
        }
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
