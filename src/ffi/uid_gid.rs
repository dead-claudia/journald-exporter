use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uid {
    raw: libc::uid_t,
}

impl fmt::Debug for Uid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uid({})", self.raw)
    }
}

impl Uid {
    pub const ROOT: Uid = Uid { raw: 0 };
    #[cfg(not(miri))]
    pub fn current() -> Uid {
        // SAFETY: `geteuid` can never fail.
        unsafe { libc::geteuid() }.into()
    }
    #[cfg(miri)]
    pub fn current() -> Uid {
        Uid { raw: 99999 }
    }
}

impl From<libc::uid_t> for Uid {
    fn from(raw: libc::uid_t) -> Self {
        Uid { raw }
    }
}

impl From<Uid> for libc::uid_t {
    fn from(uid: Uid) -> Self {
        uid.raw
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gid {
    raw: libc::gid_t,
}

impl fmt::Debug for Gid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gid({})", self.raw)
    }
}

impl Gid {
    pub const ROOT: Gid = Gid { raw: 0 };
    #[cfg(not(miri))]
    pub fn current() -> Gid {
        // SAFETY: `getegid` can never fail.
        unsafe { libc::getegid() }.into()
    }
    #[cfg(miri)]
    pub fn current() -> Gid {
        Gid { raw: 99999 }
    }
}

impl From<libc::gid_t> for Gid {
    fn from(raw: libc::gid_t) -> Self {
        Gid { raw }
    }
}

impl From<Gid> for libc::gid_t {
    fn from(uid: Gid) -> Self {
        uid.raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uid_works() {
        assert_eq!(Uid::ROOT, Uid::from(0));
        assert_eq!(Uid::from(123), Uid::from(123));
        assert_eq!(libc::uid_t::from(Uid::from(123)), 123);
        assert_eq!(libc::uid_t::from(Uid::ROOT), 0);
        let _ = Uid::current();
    }

    #[test]
    fn gid_works() {
        assert_eq!(Gid::ROOT, Gid::from(0));
        assert_eq!(Gid::from(123), Gid::from(123));
        assert_eq!(libc::gid_t::from(Gid::from(123)), 123);
        assert_eq!(libc::gid_t::from(Gid::ROOT), 0);
        let _ = Gid::current();
    }
}
