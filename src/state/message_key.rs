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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteCountTableKey {
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub service_repr: ServiceRepr,
}

impl ByteCountTableKey {
    pub fn service(&self) -> Option<Service> {
        self.service_repr.as_service()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(test, derive(Clone))]
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
#[allow(clippy::as_conversions)]
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

    #[test]
    fn orders_based_on_priority() {
        propcheck::run(|&(a, b): &(Priority, Priority)| {
            let mut key1 = MessageKey::new();
            let mut key2 = MessageKey::new();
            key1.priority = a;
            key2.priority = b;
            key1.cmp(&key2) == a.cmp(&b)
        });
    }

    #[test]
    fn orders_based_on_uid() {
        propcheck::run(|&(a, b): &(libc::uid_t, libc::uid_t)| {
            let mut key1 = MessageKey::new();
            let mut key2 = MessageKey::new();
            key1.table_key.uid = Some(a);
            key2.table_key.uid = Some(b);
            key1.cmp(&key2) == a.cmp(&b)
        });
    }

    #[test]
    fn orders_based_on_gid() {
        propcheck::run(|&(a, b): &(libc::gid_t, libc::gid_t)| {
            let mut key1 = MessageKey::new();
            let mut key2 = MessageKey::new();
            key1.table_key.gid = Some(a);
            key2.table_key.gid = Some(b);
            key1.cmp(&key2) == a.cmp(&b)
        });
    }

    #[test]
    fn orders_based_on_service() {
        propcheck::run(|(a, b): &(ServiceRepr, ServiceRepr)| {
            let mut key1 = MessageKey::new();
            let mut key2 = MessageKey::new();

            key1.set_service(a.as_service().unwrap());
            key2.set_service(b.as_service().unwrap());
            key1.cmp(&key2) == service_compare(a.as_service().unwrap(), b.as_service().unwrap())
        });
    }

    #[test]
    fn orders_based_on_priority_with_all_fields_initialized() {
        propcheck::run(|&(a, b): &(Priority, Priority)| {
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
        });
    }

    #[test]
    fn orders_based_on_uid_with_all_fields_initialized() {
        propcheck::run(|&(a, b): &(libc::uid_t, libc::uid_t)| {
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
        });
    }

    #[test]
    fn orders_based_on_gid_with_all_fields_initialized() {
        propcheck::run(|&(a, b): &(libc::gid_t, libc::gid_t)| {
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
        });
    }

    #[test]
    fn orders_based_on_service_with_all_fields_initialized() {
        propcheck::run(|(a, b): &(ServiceRepr, ServiceRepr)| {
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
        });
    }

    // Absent-before-present properties

    #[test]
    fn orders_no_uid_before_uid() {
        propcheck::run(|&uid: &libc::uid_t| {
            let key1 = MessageKey::new();
            let mut key2 = MessageKey::new();
            key2.table_key.uid = Some(uid);
            key1 < key2
        });
    }

    #[test]
    fn orders_no_gid_before_gid() {
        propcheck::run(|&gid: &libc::gid_t| {
            let key1 = MessageKey::new();
            let mut key2 = MessageKey::new();
            key2.table_key.gid = Some(gid);
            key1 < key2
        });
    }

    #[test]
    fn orders_no_service_before_service() {
        propcheck::run(|service: &ServiceRepr| {
            let key1 = MessageKey::new();
            let mut key2 = MessageKey::new();
            key2.set_service(service.as_service().unwrap());
            key1 < key2
        });
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

    #[test]
    fn orders_no_uid_some_gid_before_some_uid_no_gid() {
        propcheck::run(|&(a, b): &(libc::uid_t, libc::gid_t)| {
            let mut key1 = MessageKey::new();
            let mut key2 = MessageKey::new();
            key1.table_key.gid = Some(a);
            key2.table_key.uid = Some(b);
            key1 < key2
        });
    }

    #[test]
    fn orders_some_gid_no_service_before_no_gid_some_servic() {
        propcheck::run(|(a, b): &(libc::gid_t, ServiceRepr)| {
            let mut key1 = MessageKey::new();
            let mut key2 = MessageKey::new();
            key1.set_service(b.as_service().unwrap());
            key2.table_key.gid = Some(*a);
            key1 < key2
        });
    }
}
