use crate::prelude::*;

/*
Precedence for ordering, where `a < b` implies `a` comes before `b`:
- Lower priority value < higher priority value
- Lower priority value < same priority value + lower UID
- Lower UID < same priority value + same UID + lower GID
- Lower GID < same priority value + same UID + same GID + lexicographically later service
- No UID < specified UID
- No GID < specified GID
- No service < specified service
- No UID < lower priority + some UID
- No GID < higher UID + some GID
- No service < higher GID + some service
- Priority < UID < GID < service
*/

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(test, derive(Copy))]
pub struct ByteCountTableKey {
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub service_repr: ServiceRepr,
}

#[cfg(test)]
fn build_byte_count_key_tuple(
    (uid, gid, service_repr): (Option<u32>, Option<u32>, ServiceRepr),
) -> ByteCountTableKey {
    ByteCountTableKey {
        uid,
        gid,
        service_repr,
    }
}

#[cfg(test)]
impl Arbitrary for ByteCountTableKey {
    fn arbitrary(g: &mut Gen) -> Self {
        build_byte_count_key_tuple(Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.uid, self.gid, self.service_repr)
                .shrink()
                .map(build_byte_count_key_tuple),
        )
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(test, derive(Clone, Copy))]
pub struct MessageKey {
    pub priority: Priority,
    pub table_key: ByteCountTableKey,
}

impl fmt::Debug for MessageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageKey")
            .field("uid", &self.table_key.uid)
            .field("gid", &self.table_key.gid)
            .field("priority", &self.priority)
            .field("service", &self.service())
            .finish()
    }
}

#[cfg(test)]
impl Arbitrary for MessageKey {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            priority: Arbitrary::arbitrary(g),
            table_key: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.priority, self.table_key)
                .shrink()
                .map(|(priority, key)| Self {
                    priority,
                    table_key: key,
                }),
        )
    }
}

impl MessageKey {
    pub fn new() -> Self {
        Self {
            priority: Priority::Emergency,
            table_key: ByteCountTableKey {
                uid: None,
                gid: None,
                service_repr: ServiceRepr::EMPTY,
            },
        }
    }

    pub fn copy_from_table_entry(priority: Priority, key: &ByteCountTableKey) -> Self {
        // It's not copyable in production, as it's supposed to be minimally copied in general
        // (in no small part due to its size).
        #![allow(clippy::clone_on_copy)]
        Self {
            priority,
            table_key: key.clone(),
        }
    }

    #[cfg(test)]
    pub const fn build(
        uid: Option<u32>,
        gid: Option<u32>,
        service: Option<&[u8]>,
        priority: Priority,
    ) -> Self {
        Self {
            priority,
            table_key: ByteCountTableKey {
                uid,
                gid,
                service_repr: match ServiceRepr::new(service) {
                    Ok(service_repr) => service_repr,
                    Err(ServiceParseError::Empty) => panic!("Service name is empty."),
                    Err(ServiceParseError::TooLong) => panic!("Service name is too long."),
                    Err(ServiceParseError::Invalid) => panic!("Service name is invalid."),
                },
            },
        }
    }

    pub fn set_service(&mut self, service: Service) {
        self.table_key.service_repr.set_service(service);
    }

    pub fn service(&self) -> Option<Service> {
        self.table_key.service_repr.as_service()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test utilities

    fn s(s: &[u8]) -> Service {
        Service::from_slice(s).unwrap()
    }

    fn service_compare(a: Service, b: Service) -> std::cmp::Ordering {
        match a.as_bytes().len().cmp(&b.as_bytes().len()) {
            std::cmp::Ordering::Equal => a.as_bytes().cmp(b.as_bytes()),
            not_equal => not_equal,
        }
    }

    // Smoke tests for test utilities
    #[test]
    fn test_service_compare_works() {
        use std::cmp::Ordering;

        // Single-char vs single-char
        assert_eq!(service_compare(s(b"a"), s(b"a")), Ordering::Equal);
        assert_eq!(service_compare(s(b"a"), s(b"b")), Ordering::Less);
        assert_eq!(service_compare(s(b"b"), s(b"a")), Ordering::Greater);
        assert_eq!(service_compare(s(b"b"), s(b"b")), Ordering::Equal);

        // Single-char vs multi-char
        assert_eq!(service_compare(s(b"a"), s(b"aa")), Ordering::Less);
        assert_eq!(service_compare(s(b"a"), s(b"ab")), Ordering::Less);
        assert_eq!(service_compare(s(b"a"), s(b"ba")), Ordering::Less);
        assert_eq!(service_compare(s(b"a"), s(b"bb")), Ordering::Less);
        assert_eq!(service_compare(s(b"b"), s(b"aa")), Ordering::Less);
        assert_eq!(service_compare(s(b"b"), s(b"ab")), Ordering::Less);
        assert_eq!(service_compare(s(b"b"), s(b"ba")), Ordering::Less);
        assert_eq!(service_compare(s(b"b"), s(b"bb")), Ordering::Less);

        // Multi-char, equal length
        assert_eq!(service_compare(s(b"aa"), s(b"aa")), Ordering::Equal);
        assert_eq!(service_compare(s(b"aa"), s(b"ab")), Ordering::Less);
        assert_eq!(service_compare(s(b"aa"), s(b"ba")), Ordering::Less);
        assert_eq!(service_compare(s(b"aa"), s(b"bb")), Ordering::Less);
        assert_eq!(service_compare(s(b"ab"), s(b"aa")), Ordering::Greater);
        assert_eq!(service_compare(s(b"ab"), s(b"ab")), Ordering::Equal);
        assert_eq!(service_compare(s(b"ab"), s(b"ba")), Ordering::Less);
        assert_eq!(service_compare(s(b"ab"), s(b"bb")), Ordering::Less);
        assert_eq!(service_compare(s(b"ba"), s(b"aa")), Ordering::Greater);
        assert_eq!(service_compare(s(b"ba"), s(b"ab")), Ordering::Greater);
        assert_eq!(service_compare(s(b"ba"), s(b"ba")), Ordering::Equal);
        assert_eq!(service_compare(s(b"ba"), s(b"bb")), Ordering::Less);
        assert_eq!(service_compare(s(b"bb"), s(b"aa")), Ordering::Greater);
        assert_eq!(service_compare(s(b"bb"), s(b"ab")), Ordering::Greater);
        assert_eq!(service_compare(s(b"bb"), s(b"ba")), Ordering::Greater);
        assert_eq!(service_compare(s(b"bb"), s(b"bb")), Ordering::Equal);
    }

    // Basic functionality

    #[test]
    fn has_correct_initial_uid() {
        let key = MessageKey::new();
        assert_eq!(key.table_key.uid, None);
    }

    #[test]
    fn has_correct_initial_gid() {
        let key = MessageKey::new();
        assert_eq!(key.table_key.gid, None);
    }

    #[test]
    fn has_correct_initial_priority() {
        let key = MessageKey::new();
        assert_eq!(key.priority, Priority::Emergency);
    }

    #[test]
    fn has_correct_initial_service() {
        let key = MessageKey::new();
        assert_eq!(key.service(), None);
    }

    #[test]
    fn returns_correct_non_zero_service_on_set() {
        let mut key = MessageKey::new();
        key.set_service(s(b"my-service.service"));
        assert_eq!(key.service(), Some(s(b"my-service.service")));
    }

    // Per-group properties

    #[quickcheck]
    fn orders_based_on_priority(a: Priority, b: Priority) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.priority = a;
        key2.priority = b;
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_uid(a: libc::uid_t, b: libc::uid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.table_key.uid = Some(a);
        key2.table_key.uid = Some(b);
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_gid(a: libc::gid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.table_key.gid = Some(a);
        key2.table_key.gid = Some(b);
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_service(a: ServiceRepr, b: ServiceRepr) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();

        key1.set_service(a.as_service().unwrap());
        key2.set_service(b.as_service().unwrap());
        key1.cmp(&key2) == service_compare(a.as_service().unwrap(), b.as_service().unwrap())
    }

    #[quickcheck]
    fn orders_based_on_priority_with_all_fields_initialized(a: Priority, b: Priority) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.priority = a;
        key1.table_key.uid = Some(123);
        key1.table_key.gid = Some(123);
        key1.set_service(s(b"test.service"));
        key2.priority = b;
        key2.table_key.uid = Some(123);
        key2.table_key.gid = Some(123);
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_uid_with_all_fields_initialized(a: libc::uid_t, b: libc::uid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.priority = Priority::Debug;
        key1.table_key.uid = Some(a);
        key1.table_key.gid = Some(123);
        key1.set_service(s(b"test.service"));
        key2.priority = Priority::Debug;
        key2.table_key.uid = Some(b);
        key2.table_key.gid = Some(123);
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_gid_with_all_fields_initialized(a: libc::gid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.priority = Priority::Debug;
        key1.table_key.uid = Some(123);
        key1.table_key.gid = Some(a);
        key1.set_service(s(b"test.service"));
        key2.priority = Priority::Debug;
        key2.table_key.uid = Some(123);
        key2.table_key.gid = Some(b);
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_service_with_all_fields_initialized(a: ServiceRepr, b: ServiceRepr) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();

        key1.priority = Priority::Debug;
        key1.table_key.uid = Some(123);
        key1.table_key.gid = Some(123);
        key1.set_service(a.as_service().unwrap());
        key2.priority = Priority::Debug;
        key2.table_key.uid = Some(123);
        key2.table_key.gid = Some(123);
        key2.set_service(b.as_service().unwrap());
        key1.cmp(&key2) == service_compare(a.as_service().unwrap(), b.as_service().unwrap())
    }

    // Absent-before-present properties

    #[quickcheck]
    fn orders_no_uid_before_uid(uid: libc::uid_t) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.table_key.uid = Some(uid);
        key1 < key2
    }

    #[quickcheck]
    fn orders_no_gid_before_gid(gid: libc::gid_t) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.table_key.gid = Some(gid);
        key1 < key2
    }

    #[quickcheck]
    fn orders_no_service_before_service(service: ServiceRepr) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.set_service(service.as_service().unwrap());
        key1 < key2
    }

    // Inter-group properties

    #[test]
    fn orders_highest_priority_with_no_uid_before_lowest_priority_with_uid() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.priority = Priority::Emergency;
        key2.priority = Priority::Debug;
        key2.table_key.uid = Some(u32::MAX);
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_no_uid_max_gid_before_min_uid_min_gid() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.table_key.gid = Some(u32::MAX);
        key2.table_key.uid = Some(0);
        key2.table_key.gid = Some(0);
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_max_gid_min_service_before_min_gid_no_service() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_service(s(b"-"));
        key2.table_key.gid = Some(u32::MAX);
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[quickcheck]
    fn orders_no_uid_some_gid_before_some_uid_no_gid(a: libc::uid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.table_key.gid = Some(a);
        key2.table_key.uid = Some(b);
        key1 < key2
    }

    #[quickcheck]
    fn orders_some_gid_no_service_before_no_gid_some_service(
        a: libc::gid_t,
        b: ServiceRepr,
    ) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_service(b.as_service().unwrap());
        key2.table_key.gid = Some(a);
        key1 < key2
    }
}
