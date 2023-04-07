use crate::prelude::*;

use crate::common::UidGidTable;

// Avoid needing to import a trait for this common case.
pub fn get_user_group_table() -> Arc<UidGidTable> {
    #[rustfmt::skip]
    static SHARED_UID_GID_TABLE: OnceCell<Arc<UidGidTable>> = OnceCell::new();

    SHARED_UID_GID_TABLE
        .get_or_init(|| {
            Arc::new(UidGidTable::new(
                IdTable::from_entries(&[
                    (123_u32, IdName::new(b"user_foo")),
                    (456_u32, IdName::new(b"user_bar")),
                    (789_u32, IdName::new(b"user_baz")),
                ]),
                IdTable::from_entries(&[
                    (123_u32, IdName::new(b"group_foo")),
                    (456_u32, IdName::new(b"group_bar")),
                    (789_u32, IdName::new(b"group_baz")),
                ]),
            ))
        })
        .clone()
}

// Skip these tests under Miri. They're test utilities and would just slow down Miri test runs.
#[cfg(not(miri))]
mod tests {
    use super::*;

    #[test]
    fn uid_123_can_be_accessed() {
        assert_eq!(
            *get_user_group_table().uids.lookup_id(123).unwrap(),
            IdName::new(b"user_foo")
        );
    }

    #[test]
    fn uid_456_can_be_accessed() {
        assert_eq!(
            *get_user_group_table().uids.lookup_id(456).unwrap(),
            IdName::new(b"user_bar")
        );
    }

    #[test]
    fn uid_789_can_be_accessed() {
        assert_eq!(
            *get_user_group_table().uids.lookup_id(789).unwrap(),
            IdName::new(b"user_baz")
        );
    }

    #[test]
    fn gid_123_can_be_accessed() {
        assert_eq!(
            *get_user_group_table().gids.lookup_id(123).unwrap(),
            IdName::new(b"group_foo")
        );
    }

    #[test]
    fn gid_456_can_be_accessed() {
        assert_eq!(
            *get_user_group_table().gids.lookup_id(456).unwrap(),
            IdName::new(b"group_bar")
        );
    }

    #[test]
    fn gid_789_can_be_accessed() {
        assert_eq!(
            *get_user_group_table().gids.lookup_id(789).unwrap(),
            IdName::new(b"group_baz")
        );
    }

    #[test]
    fn lookup_identifies_missing_user() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_bad", b"group_foo"),
            None,
        );
    }

    #[test]
    fn lookup_identifies_missing_group() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_foo", b"group_bad"),
            None,
        );
    }

    #[test]
    fn lookup_identifies_missing_user_group() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_bad", b"group_bad"),
            None,
        );
    }

    #[test]
    fn lookup_identifies_uid_gid_123() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_foo", b"group_foo"),
            Some(UserGroup { uid: 123, gid: 123 }),
        );
    }

    #[test]
    fn lookup_identifies_uid_gid_456() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_bar", b"group_bar"),
            Some(UserGroup { uid: 456, gid: 456 }),
        );
    }

    #[test]
    fn lookup_identifies_uid_gid_789() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_baz", b"group_baz"),
            Some(UserGroup { uid: 789, gid: 789 }),
        );
    }

    #[test]
    fn lookup_identifies_uid_123_gid_456() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_foo", b"group_bar"),
            Some(UserGroup { uid: 123, gid: 456 }),
        );
    }

    #[test]
    fn lookup_identifies_uid_456_gid_789() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_bar", b"group_baz"),
            Some(UserGroup { uid: 456, gid: 789 }),
        );
    }

    #[test]
    fn lookup_identifies_uid_789_gid_123() {
        assert_eq!(
            get_user_group_table().lookup_user_group(b"user_baz", b"group_foo"),
            Some(UserGroup { uid: 789, gid: 123 }),
        );
    }
}
