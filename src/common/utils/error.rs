use crate::prelude::*;

#[doc(hidden)]
#[inline(always)]
pub fn __error_fmt(kind: ErrorKind, args: fmt::Arguments) -> Error {
    match args.as_str() {
        Some(error) => Error::new(kind, error),
        None => Error::new(kind, args.to_string()),
    }
}

macro_rules! error {
    ($kind:path, $fmt:literal $($tt:tt)*) => {{
        $crate::common::__error_fmt($kind, ::std::format_args!($fmt $($tt)*))
    }};
    ($fmt:literal $($tt:tt)*) => {{
        $crate::common::__error_fmt(::std::io::ErrorKind::Other, ::std::format_args!($fmt $($tt)*))
    }};
}
pub(crate) use error;
