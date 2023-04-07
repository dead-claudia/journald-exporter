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
pub enum IpcError {
    Parent(Error),
    ChildWait(Error),
}

// Only implement it for test, so it can satisfy `Arbitrary`. In prod, it should never be cloned
// like this.
#[cfg(test)]
impl Clone for IpcError {
    fn clone(&self) -> Self {
        match self {
            IpcError::Parent(e) => IpcError::Parent(error_clone(e)),
            IpcError::ChildWait(e) => IpcError::ChildWait(error_clone(e)),
        }
    }
}

#[cfg(test)]
impl PartialEq for IpcError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (IpcError::Parent(left), IpcError::Parent(right)) => error_eq(left, right),
            (IpcError::ChildWait(left), IpcError::ChildWait(right)) => error_eq(left, right),
            _ => false,
        }
    }
}

#[cfg(test)]
impl Eq for IpcError {}

#[cfg(test)]
impl Arbitrary for IpcError {
    fn arbitrary(g: &mut Gen) -> Self {
        enum S {
            Parent,
            ChildWait,
        }

        match g.choose(&[S::Parent, S::ChildWait]).unwrap() {
            S::Parent => Self::Parent(error_arbitrary(g)),
            S::ChildWait => Self::ChildWait(error_arbitrary(g)),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            IpcError::Parent(e) => Box::new(error_shrink(e).map(Self::Parent)),
            IpcError::ChildWait(e) => Box::new(error_shrink(e).map(Self::ChildWait)),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
pub struct IpcExitStatus {
    pub result: Option<ExitResult>,
    pub errors: Vec<IpcError>,
}

#[cfg(test)]
impl Arbitrary for IpcExitStatus {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            result: Arbitrary::arbitrary(g),
            errors: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let results = Vec::from_iter(self.result.shrink());
        Box::new(self.errors.shrink().flat_map(move |errors| {
            results.clone().into_iter().map(move |result| Self {
                errors: errors.clone(),
                result,
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
                errors: Vec::new(),
            },
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(0))),
                errors: Vec::new(),
            },
        )
    }

    #[test]
    fn ipc_exit_status_assertion_works_on_fail() {
        assert_ne!(
            IpcExitStatus {
                result: Some(ExitResult::Code(ExitCode(0))),
                errors: Vec::new(),
            },
            IpcExitStatus {
                result: Some(ExitResult::Signal(Signal::SIGINT)),
                errors: vec![IpcError::Parent(ErrorKind::NotFound.into())],
            },
        );
    }
}
