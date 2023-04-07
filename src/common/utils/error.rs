use crate::prelude::*;

struct StaticError(&'static str);

impl fmt::Debug for StaticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for StaticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for StaticError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.0
    }
}

pub fn err(msg: &'static str) -> Error {
    // Avoid an extra allocation here.
    Error::new(ErrorKind::Other, StaticError(msg))
}

pub fn string_err(msg: String) -> Error {
    Error::new(ErrorKind::Other, msg)
}
