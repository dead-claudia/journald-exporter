use crate::prelude::*;

struct CowStrError(CowStr<'static>);

impl fmt::Debug for CowStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for CowStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for CowStrError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match &self.0 {
            CowStr::Borrowed(s) => s,
            CowStr::Owned(s) => s,
        }
    }
}

pub fn err(msg: &'static str) -> Error {
    cow_err(CowStr::Borrowed(msg))
}

pub fn string_err(msg: Box<str>) -> Error {
    cow_err(CowStr::Owned(msg))
}

pub fn cow_err(msg: CowStr<'static>) -> Error {
    // Avoid an extra allocation here.
    Error::new(ErrorKind::Other, CowStrError(msg))
}
