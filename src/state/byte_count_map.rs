use crate::prelude::*;

use super::ByteCountTableKey;
use super::MessageKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteCountSnapshotEntry {
    pub key: MessageKey,
    pub lines: u64,
    pub bytes: u64,
}

#[cfg(test)]
impl Arbitrary for ByteCountSnapshotEntry {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            key: Arbitrary::arbitrary(g),
            lines: Arbitrary::arbitrary(g),
            bytes: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (self.key.clone(), self.lines, self.bytes)
                .shrink()
                .map(|(key, lines, bytes)| Self { key, lines, bytes }),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteCountSnapshot {
    pub data: Box<[ByteCountSnapshotEntry]>,
}

impl ByteCountSnapshot {
    #[cfg(test)]
    pub fn empty() -> Self {
        Self { data: Box::new([]) }
    }
}

#[cfg(test)]
impl Arbitrary for ByteCountSnapshot {
    fn arbitrary(g: &mut Gen) -> Self {
        let data: Vec<_> = Arbitrary::arbitrary(g);
        Self { data: data.into() }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.data
                .to_vec()
                .shrink()
                .map(|data| Self { data: data.into() }),
        )
    }
}

#[derive(Clone)]
struct ByteCountData {
    lines: Counter,
    bytes: Counter,
}

struct ByteCountTableEntry {
    data: ByteCountData,
    key: ByteCountTableKey,
}

pub struct ByteCountMap {
    // The last entry of "priority" 8 is the fallback.
    priority_table: [RwLock<Vec<ByteCountTableEntry>>; 8],
}

impl ByteCountMap {
    pub const fn new() -> ByteCountMap {
        ByteCountMap {
            priority_table: [
                RwLock::new(Vec::new()),
                RwLock::new(Vec::new()),
                RwLock::new(Vec::new()),
                RwLock::new(Vec::new()),
                RwLock::new(Vec::new()),
                RwLock::new(Vec::new()),
                RwLock::new(Vec::new()),
                RwLock::new(Vec::new()),
            ],
        }
    }

    pub fn snapshot(&self) -> ByteCountSnapshot {
        // This is just a speculative size estimate. It's very unlikely to be wrong, since keys are
        // very rarely added. (Also why I'm not bothering to make this fully wait-free via `mmap`.)
        let mut size = 0_usize;
        for entry in self.priority_table.iter() {
            let read_lock = entry.read().unwrap_or_else(|e| e.into_inner());
            size = size.saturating_add(read_lock.len());
        }
        let mut result = Vec::with_capacity(size);
        for (i, entry) in self.priority_table.iter().enumerate() {
            let priority = Priority::from_severity_index(truncate_usize_u8(i)).unwrap();
            let read_lock = entry.read().unwrap_or_else(|e| e.into_inner());
            for ByteCountTableEntry { key, data } in read_lock.iter() {
                result.push(ByteCountSnapshotEntry {
                    key: MessageKey::from_table_key(priority, key),
                    lines: data.lines.current(),
                    bytes: data.bytes.current(),
                })
            }
        }
        ByteCountSnapshot {
            data: result.into(),
        }
    }

    // Take a reference to avoid a copy.
    pub fn push_line(&self, key: &MessageKey, msg_len: usize) {
        let msg_len = zero_extend_usize_u64(msg_len);

        // Services are very rarely added. Try opening a read first and doing atomic updates, and
        // fall back to a write lock if the entry doesn't exist yet. Contention should already be
        // low as-is since only two threads could be accessing the map, and it's further reduced by
        // the indirection on priority/severity level.
        //
        // Using an upgradable lock so I don't need to drop and re-acquire it.
        let priority_index = zero_extend_u8_usize(key.priority().as_severity_index());
        let read_lock = self.priority_table[priority_index]
            .read()
            .unwrap_or_else(|e| e.into_inner());

        for entry in read_lock.iter() {
            if &entry.key == key.as_table_key() {
                entry.data.lines.increment();
                entry.data.bytes.increment_by(msg_len);
                return;
            }
        }

        // Don't deadlock. Drop the lock before entering the fallback path.
        drop(read_lock);

        push_line_likely_new(&self.priority_table[priority_index], key, msg_len);

        // Here's why I want to keep this fully out of the hot path:
        // - There's normally only like a few hundred services active. *Maybe* a thousand on machines
        //   with a somewhat extreme number of services.
        // - There are only a few extra dimensions in `MessageKey`: UID, GID, and priority. UIDs and
        //   GIDs normally align one-to-one with service names, and only a few priorities are normally
        //   used by each service. Worst case scenario, all 8 are used, but even that doesn't increase
        //   cardinality much. If you multiply it all together, you're seeing up to at most a few
        //   thousand keys added during the lifetime of the application, with most of these just being
        //   added near the exporter's startup.
        // - The key itself could be relatively large (like 250+ bytes), and it's just wasteful to
        //   allocate that much stack space on an infrequent call path.
        #[cold]
        #[inline(never)]
        fn push_line_likely_new(
            priority_entry: &RwLock<Vec<ByteCountTableEntry>>,
            key: &MessageKey,
            msg_len: u64,
        ) {
            // Entry doesn't exist. Time to acquire a write lock and update the hash map with a
            // possible new key.
            let mut write_lock = priority_entry.write().unwrap_or_else(|e| e.into_inner());

            for entry in write_lock.iter() {
                if &entry.key == key.as_table_key() {
                    entry.data.lines.increment();
                    entry.data.bytes.increment_by(msg_len);
                    return;
                }
            }

            // While this may reallocate a lot at first, it's unlikely to reallocate too much after
            // that, since there's only so many system services. This is why it doesn't try to
            // pre-allocate - it's just not needed.

            write_lock.push(ByteCountTableEntry {
                data: ByteCountData {
                    lines: Counter::new(1),
                    bytes: Counter::new(msg_len),
                },
                key: key.as_table_key().clone(),
            });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn map_key(s: &str) -> MessageKey {
        let mut key = MessageKey::new();
        key.set_service(Service::from_slice(s.as_bytes()).unwrap());
        key.set_priority(crate::common::Priority::Informational);
        key
    }

    fn actual_snapshot(s: &'static PromState) -> PromSnapshot {
        s.snapshot()
    }

    fn expected_snapshot(expected: &[(MessageKey, u64, u64)]) -> PromSnapshot {
        let mut expected = Vec::from_iter(
            expected
                .iter()
                .cloned()
                .map(|(key, lines, bytes)| ByteCountSnapshotEntry { key, lines, bytes }),
        );
        expected.sort_by(|a, b| a.key.cmp(&b.key));

        PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: expected.into(),
            },
        }
    }

    #[test]
    fn byte_count_map_works_on_one_thread() {
        static S: PromState = PromState::new();

        S.add_message_line_ingested(&map_key("one"), 123);
        S.add_message_line_ingested(&map_key("one"), 456);
        S.add_message_line_ingested(&map_key("two"), 789);
        S.add_message_line_ingested(&map_key("three"), 555);
        S.add_message_line_ingested(&map_key("three"), 444);

        assert_eq!(
            actual_snapshot(&S),
            expected_snapshot(&[
                (map_key("one"), 2, 579),
                (map_key("two"), 1, 789),
                (map_key("three"), 2, 999),
            ])
        );
    }

    fn test_contends(state: &'static PromState, thread_count: u64) {
        let keys = [
            map_key("test_1"),
            map_key("test_2"),
            map_key("test_3"),
            map_key("test_4"),
            map_key("test_5"),
            map_key("test_6"),
            map_key("test_7"),
            map_key("test_8"),
            map_key("test_9"),
            map_key("test_10"),
            map_key("test_11"),
            map_key("test_12"),
            map_key("test_13"),
            map_key("test_14"),
            map_key("test_15"),
            map_key("test_16"),
        ];

        let mut contends = Vec::new();

        for id in 1..=thread_count {
            std::thread::scope(|s| {
                // Ensure each borrow is unique, so I can `move` the contends list into the
                // closure.
                let keys_list = &keys;
                let contends_list = [
                    map_key(&format!("contend_{id}_1")),
                    map_key(&format!("contend_{id}_2")),
                    map_key(&format!("contend_{id}_3")),
                    map_key(&format!("contend_{id}_4")),
                    map_key(&format!("contend_{id}_5")),
                    map_key(&format!("contend_{id}_6")),
                    map_key(&format!("contend_{id}_7")),
                    map_key(&format!("contend_{id}_8")),
                    map_key(&format!("contend_{id}_9")),
                    map_key(&format!("contend_{id}_10")),
                ];

                contends.extend(contends_list.iter().cloned());

                s.spawn(move || {
                    let mut contends = contends_list.iter().cycle();
                    for i in 1..=100 {
                        for key in keys_list.iter() {
                            state.add_message_line_ingested(key, 10);
                        }
                        if i % 10 == 5 {
                            state.add_message_line_ingested(contends.next().unwrap(), 1);
                        }
                    }
                });
            });
        }

        let mut expected = Vec::with_capacity(keys.len() + contends.len());

        for key in keys {
            expected.push((key, 100 * thread_count, 1000 * thread_count));
        }

        for key in contends {
            expected.push((key, 1, 1));
        }

        assert_eq!(actual_snapshot(state), expected_snapshot(&expected));
    }

    #[test]
    fn byte_count_map_works_on_two_contending_threads() {
        static S: PromState = PromState::new();
        test_contends(&S, 2);
    }

    // Complete overkill, but specifically just trying to stress-test contention.
    #[test]
    // Extremely slow in Miri. Just skip it.
    #[cfg_attr(miri, ignore)]
    fn byte_count_map_works_on_a_hundred_contending_threads() {
        static S: PromState = PromState::new();
        test_contends(&S, 100);
    }
}
