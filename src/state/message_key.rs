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
pub(super) struct ByteCountTableKey {
    pub service_len: u16,
    pub uid: Option<Uid>,
    pub gid: Option<Gid>,
    //             [u8; MAX_SERVICE_LEN]
    pub service_bytes: [u8; 256],
}

#[cfg(test)]
fn build_byte_count_key(
    uid: Option<u32>,
    gid: Option<u32>,
    service: Option<&[u8]>,
) -> ByteCountTableKey {
    let mut service_bytes = [0; MAX_SERVICE_LEN];
    let mut service_len = 0;
    if let Some(s) = service {
        service_len = truncate_usize_u16(s.len());
        copy_to_start(&mut service_bytes, s);
    }
    ByteCountTableKey {
        uid: uid.map(Uid::from),
        gid: gid.map(Gid::from),
        service_len,
        service_bytes,
    }
}

#[cfg(test)]
fn build_byte_count_key_tuple(
    (uid, gid, service): (
        Option<u32>,
        Option<u32>,
        Option<sd_arbitrary::ArbitraryService>,
    ),
) -> ByteCountTableKey {
    build_byte_count_key(uid, gid, service.map(|s| s.unpack()).as_deref())
}

#[cfg(test)]
impl Arbitrary for ByteCountTableKey {
    fn arbitrary(g: &mut Gen) -> Self {
        build_byte_count_key_tuple(Arbitrary::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let service = {
            if self.service_len > 0 {
                Some(sd_arbitrary::ArbitraryService::from_unwrapped(
                    &self.service_bytes,
                ))
            } else {
                None
            }
        };

        Box::new(
            (self.uid.map(u32::from), self.gid.map(u32::from), service)
                .shrink()
                .map(build_byte_count_key_tuple),
        )
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageKey {
    priority: Priority,
    key: ByteCountTableKey,
}

impl fmt::Debug for MessageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageKey")
            .field("uid", &self.uid())
            .field("gid", &self.gid())
            .field("priority", &self.priority())
            .field("service", &self.service())
            .finish()
    }
}

#[cfg(test)]
impl Arbitrary for MessageKey {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            priority: Arbitrary::arbitrary(g),
            key: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.priority, self.key.clone())
                .shrink()
                .map(|(priority, key)| Self { priority, key }),
        )
    }
}

impl MessageKey {
    pub fn new() -> Self {
        Self {
            priority: Priority::Emergency,
            key: ByteCountTableKey {
                service_len: 0,
                uid: None,
                gid: None,
                service_bytes: [0; MAX_SERVICE_LEN],
            },
        }
    }

    #[cfg(test)]
    pub fn build(
        uid: Option<u32>,
        gid: Option<u32>,
        service: Option<&[u8]>,
        priority: Priority,
    ) -> Self {
        Self {
            priority,
            key: build_byte_count_key(uid, gid, service),
        }
    }

    pub(super) fn from_table_key(priority: Priority, key: &ByteCountTableKey) -> Self {
        Self {
            priority,
            key: key.clone(),
        }
    }

    pub fn set_uid(&mut self, uid: Uid) {
        self.key.uid = Some(uid);
    }

    pub fn set_gid(&mut self, gid: Gid) {
        self.key.gid = Some(gid);
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }

    pub fn set_service(&mut self, service: Service) {
        let bytes = service.as_bytes();
        self.key.service_len = truncate_usize_u16(bytes.len());
        copy_to_start(&mut self.key.service_bytes, bytes);
    }

    pub fn uid(&self) -> Option<Uid> {
        self.key.uid
    }

    pub fn gid(&self) -> Option<Gid> {
        self.key.gid
    }

    pub fn priority(&self) -> Priority {
        self.priority
    }

    pub fn service(&self) -> Option<Service> {
        if self.key.service_len > 0 {
            match Service::from_slice(
                &self.key.service_bytes[..zero_extend_u16_usize(self.key.service_len)],
            ) {
                Ok(result) => Some(result),
                Err(_) => unreachable!(),
            }
        } else {
            None
        }
    }

    pub(super) fn as_table_key(&self) -> &ByteCountTableKey {
        &self.key
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use sd_arbitrary::ArbitraryService;

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
        assert_eq!(key.uid(), None);
    }

    #[test]
    fn has_correct_initial_gid() {
        let key = MessageKey::new();
        assert_eq!(key.gid(), None);
    }

    #[test]
    fn has_correct_initial_priority() {
        let key = MessageKey::new();
        assert_eq!(key.priority(), Priority::Emergency);
    }

    #[test]
    fn has_correct_initial_service() {
        let key = MessageKey::new();
        assert_eq!(key.service(), None);
    }

    #[test]
    fn returns_correct_non_zero_uid_on_set() {
        let mut key = MessageKey::new();
        key.set_uid(Uid::from(123));
        assert_eq!(key.uid(), Some(Uid::from(123)));
    }

    #[test]
    fn returns_correct_non_zero_gid_on_set() {
        let mut key = MessageKey::new();
        key.set_gid(Gid::from(123));
        assert_eq!(key.gid(), Some(Gid::from(123)));
    }

    #[test]
    fn returns_correct_non_zero_priority_on_set() {
        let mut key = MessageKey::new();
        key.set_priority(Priority::Informational);
        assert_eq!(key.priority(), Priority::Informational);
    }

    #[test]
    fn returns_correct_non_zero_service_on_set() {
        let mut key = MessageKey::new();
        key.set_service(s(b"my-service.service"));
        assert_eq!(key.service(), Some(s(b"my-service.service")));
    }

    #[test]
    fn returns_correct_zero_uid_on_set() {
        let mut key = MessageKey::new();
        key.set_uid(Uid::from(0));
        assert_eq!(key.uid(), Some(Uid::from(0)));
    }

    #[test]
    fn returns_correct_zero_gid_on_set() {
        let mut key = MessageKey::new();
        key.set_gid(Gid::from(0));
        assert_eq!(key.gid(), Some(Gid::from(0)));
    }

    #[test]
    fn returns_correct_zero_priority_on_set() {
        let mut key = MessageKey::new();
        key.set_priority(Priority::Emergency);
        assert_eq!(key.priority(), Priority::Emergency);
    }

    // Per-group properties

    #[quickcheck]
    fn orders_based_on_priority(a: Priority, b: Priority) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_priority(a);
        key2.set_priority(b);
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_uid(a: libc::uid_t, b: libc::uid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_uid(Uid::from(a));
        key2.set_uid(Uid::from(b));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_gid(a: libc::gid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_gid(Gid::from(a));
        key2.set_gid(Gid::from(b));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_service(a: ArbitraryService, b: ArbitraryService) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();

        key1.set_service(a.as_service());
        key2.set_service(b.as_service());
        key1.cmp(&key2) == service_compare(a.as_service(), b.as_service())
    }

    #[quickcheck]
    fn orders_based_on_priority_with_all_fields_initialized(a: Priority, b: Priority) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_priority(a);
        key1.set_uid(Uid::from(123));
        key1.set_gid(Gid::from(123));
        key1.set_service(s(b"test.service"));
        key2.set_priority(b);
        key2.set_uid(Uid::from(123));
        key2.set_gid(Gid::from(123));
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_uid_with_all_fields_initialized(a: libc::uid_t, b: libc::uid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_priority(Priority::Debug);
        key1.set_uid(Uid::from(a));
        key1.set_gid(Gid::from(123));
        key1.set_service(s(b"test.service"));
        key2.set_priority(Priority::Debug);
        key2.set_uid(Uid::from(b));
        key2.set_gid(Gid::from(123));
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_gid_with_all_fields_initialized(a: libc::gid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_priority(Priority::Debug);
        key1.set_uid(Uid::from(123));
        key1.set_gid(Gid::from(a));
        key1.set_service(s(b"test.service"));
        key2.set_priority(Priority::Debug);
        key2.set_uid(Uid::from(123));
        key2.set_gid(Gid::from(b));
        key2.set_service(s(b"test.service"));
        key1.cmp(&key2) == a.cmp(&b)
    }

    #[quickcheck]
    fn orders_based_on_service_with_all_fields_initialized(
        a: ArbitraryService,
        b: ArbitraryService,
    ) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();

        key1.set_priority(Priority::Debug);
        key1.set_uid(Uid::from(123));
        key1.set_gid(Gid::from(123));
        key1.set_service(a.as_service());
        key2.set_priority(Priority::Debug);
        key2.set_uid(Uid::from(123));
        key2.set_gid(Gid::from(123));
        key2.set_service(b.as_service());
        key1.cmp(&key2) == service_compare(a.as_service(), b.as_service())
    }

    // Absent-before-present properties

    #[quickcheck]
    fn orders_no_uid_before_uid(uid: libc::uid_t) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.set_uid(Uid::from(uid));
        key1 < key2
    }

    #[quickcheck]
    fn orders_no_gid_before_gid(gid: libc::gid_t) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.set_gid(Gid::from(gid));
        key1 < key2
    }

    #[quickcheck]
    fn orders_no_service_before_service(service: ArbitraryService) -> bool {
        let key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key2.set_service(service.as_service());
        key1 < key2
    }

    // Inter-group properties

    #[test]
    fn orders_min_priority_with_no_uid_before_lowest_priority_with_uid() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_priority(Priority::Emergency);
        key2.set_priority(Priority::Debug);
        key2.set_uid(Uid::from(u32::MAX));
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_no_uid_max_gid_before_min_uid_min_gid() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_gid(Gid::from(u32::MAX));
        key2.set_uid(Uid::from(0));
        key2.set_gid(Gid::from(0));
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[test]
    fn orders_max_gid_no_service_before_min_gid_min_service() {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_gid(Gid::from(u32::MAX));
        key2.set_service(s(b"-"));
        assert!(key1 < key2, "{key1:?} < {key2:?}");
    }

    #[quickcheck]
    fn orders_no_uid_some_gid_before_some_uid_no_gid(a: libc::uid_t, b: libc::gid_t) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_gid(Gid::from(a));
        key2.set_uid(Uid::from(b));
        key1 < key2
    }

    #[quickcheck]
    fn orders_no_gid_some_service_before_some_gid_no_service(
        a: libc::gid_t,
        b: ArbitraryService,
    ) -> bool {
        let mut key1 = MessageKey::new();
        let mut key2 = MessageKey::new();
        key1.set_gid(Gid::from(a));
        key2.set_service(b.as_service());
        key1 < key2
    }
}
