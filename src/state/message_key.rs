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

#[cfg(test)]
fn build_message_key_tuple(
    (priority, uid, gid, service): (Priority, Option<u32>, Option<u32>, ServiceRepr),
) -> MessageKey {
    MessageKey {
        priority,
        uid,
        gid,
        service,
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord, Hash)]
pub struct MessageKey {
    pub priority: Priority,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub service: ServiceRepr,
}

#[cfg(test)]
impl Arbitrary for MessageKey {
    fn arbitrary(g: &mut Gen) -> Self {
        build_message_key_tuple(Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.priority, self.uid, self.gid, self.service)
                .shrink()
                .map(build_message_key_tuple),
        )
    }
}

impl MessageKey {
    pub fn new() -> Self {
        Self {
            priority: Priority::Emergency,
            uid: None,
            gid: None,
            service: ServiceRepr::EMPTY,
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
            uid,
            gid,
            service: match ServiceRepr::new(service) {
                Ok(service_repr) => service_repr,
                Err(ServiceParseError::Empty) => panic!("Service name is empty."),
                Err(ServiceParseError::TooLong) => panic!("Service name is too long."),
                Err(ServiceParseError::Invalid) => panic!("Service name is invalid."),
            },
        }
    }

    pub fn set_service(&mut self, service: Service) {
        self.service.set_service(service);
    }

    pub fn service(&self) -> Option<Service> {
        self.service.as_service()
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
        assert_eq!(key.uid, None);
    }

    #[test]
    fn has_correct_initial_gid() {
        let key = MessageKey::new();
        assert_eq!(key.gid, None);
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
        key1.uid = Some(a);
        key2.uid = Some(b);
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_gid(a: libc::gid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.gid = Some(a);
        key2.gid = Some(b);
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
        key1.uid = Some(123);
        key1.gid = Some(123);
        key1.set_service(s(b"test.service"));
        key2.priority = b;
        key2.uid = Some(123);
        key2.gid = Some(123);
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_uid_with_all_fields_initialized(a: libc::uid_t, b: libc::uid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.priority = Priority::Debug;
        key1.uid = Some(a);
        key1.gid = Some(123);
        key1.set_service(s(b"test.service"));
        key2.priority = Priority::Debug;
        key2.uid = Some(b);
        key2.gid = Some(123);
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_gid_with_all_fields_initialized(a: libc::gid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.priority = Priority::Debug;
        key1.uid = Some(123);
        key1.gid = Some(a);
        key1.set_service(s(b"test.service"));
        key2.priority = Priority::Debug;
        key2.uid = Some(123);
        key2.gid = Some(b);
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_service_with_all_fields_initialized(a: ServiceRepr, b: ServiceRepr) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();

        key1.priority = Priority::Debug;
        key1.uid = Some(123);
        key1.gid = Some(123);
        key1.set_service(a.as_service().unwrap());
        key2.priority = Priority::Debug;
        key2.uid = Some(123);
        key2.gid = Some(123);
        key2.set_service(b.as_service().unwrap());
        key1.cmp(&key2) == service_compare(a.as_service().unwrap(), b.as_service().unwrap())
    }

    // Absent-before-present properties

    #[quickcheck]
    fn orders_no_uid_before_uid(uid: libc::uid_t) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.uid = Some(uid);
        key1 < key2
    }

    #[quickcheck]
    fn orders_no_gid_before_gid(gid: libc::gid_t) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.gid = Some(gid);
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
        key2.uid = Some(u32::MAX);
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_lowest_priority_with_no_uid_before_highest_priority_with_uid() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.priority = Priority::Emergency;
        key1.uid = Some(u32::MAX);
        key2.priority = Priority::Debug;
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_no_uid_max_gid_before_min_uid_min_gid() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.gid = Some(u32::MAX);
        key2.uid = Some(0);
        key2.gid = Some(0);
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_no_uid_min_gid_before_min_uid_max_gid() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.gid = Some(0);
        key2.uid = Some(0);
        key2.gid = Some(u32::MAX);
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_min_gid_no_service_before_max_gid_min_service() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.gid = Some(0);
        key2.gid = Some(u32::MAX);
        key2.set_service(s(b"-"));
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_min_gid_min_service_before_max_gid_no_service() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.gid = Some(0);
        key1.set_service(s(b"-"));
        key2.gid = Some(u32::MAX);
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[quickcheck]
    fn orders_no_uid_some_gid_before_some_uid_no_gid(a: libc::uid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.gid = Some(a);
        key2.uid = Some(b);
        key1 < key2
    }

    #[quickcheck]
    fn orders_no_uid_no_gid_before_some_uid_some_gid(a: libc::uid_t, b: libc::gid_t) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.uid = Some(b);
        key2.gid = Some(a);
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
        key2.gid = Some(a);
        key1 < key2
    }

    #[quickcheck]
    fn orders_no_gid_no_service_before_some_gid_some_service(
        a: libc::gid_t,
        b: ServiceRepr,
    ) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.gid = Some(a);
        key2.set_service(b.as_service().unwrap());
        key1 < key2
    }
}
