use crate::prelude::*;

use super::ByteCountTableKey;
use super::MessageKey;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Clone, Copy))]
pub struct ByteCountSnapshotEntry {
    pub key: MessageKey,
    pub lines: u64,
    pub bytes: u64,
}

// `repr(C)` to ensure the order's well-defined.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct ByteCountTableEntrySnapshot {
    pub lines: u64,
    pub bytes: u64,
    pub key: ByteCountTableKey,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ByteCountSnapshot {
    // The last entry of "priority" 8 is the fallback.
    priority_table: [Box<[ByteCountTableEntrySnapshot]>; 8],
}

impl ByteCountSnapshot {
    #[cfg(test)]
    pub fn empty() -> Self {
        Self {
            priority_table: [
                Box::new([]),
                Box::new([]),
                Box::new([]),
                Box::new([]),
                Box::new([]),
                Box::new([]),
                Box::new([]),
                Box::new([]),
            ],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.priority_table.iter().all(|t| t.is_empty())
    }

    pub fn each_while(
        &self,
        mut receiver: impl FnMut(Priority, &ByteCountTableEntrySnapshot) -> bool,
    ) -> bool {
        for (i, table) in self.priority_table.iter().enumerate() {
            let priority = Priority::from_severity_index(truncate_usize_u8(i)).unwrap();
            for item in &**table {
                if !receiver(priority, item) {
                    return false;
                }
            }
        }
        true
    }

    #[cfg(test)]
    pub fn build(data: impl IntoIterator<Item = ByteCountSnapshotEntry>) -> Self {
        let mut result = [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];
        for item in data {
            let priority_index = zero_extend_u8_usize(item.key.priority.as_severity_index());
            let entry = ByteCountTableEntrySnapshot {
                key: item.key.table_key,
                lines: item.lines,
                bytes: item.bytes,
            };
            result[priority_index].push(entry);
        }

        Self {
            priority_table: result.map(Box::from),
        }
    }
}

// `repr(C)` to ensure the order's well-defined.
#[repr(C)]
struct ByteCountTableEntry {
    lines: Counter,
    bytes: Counter,
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

    pub fn snapshot(&self) -> Option<ByteCountSnapshot> {
        // Empty boxes are cheap - they don't actually allocate.
        let mut priority_table: [Box<[ByteCountTableEntrySnapshot]>; 8] = [
            Box::new([]),
            Box::new([]),
            Box::new([]),
            Box::new([]),
            Box::new([]),
            Box::new([]),
            Box::new([]),
            Box::new([]),
        ];

        for (snapshot, table) in priority_table.iter_mut().zip(&self.priority_table) {
            let table_lock = table.read().unwrap_or_else(|e| e.into_inner());

            // The idea is this:
            // 1. Bulk copy the whole thing. This is a simple `alloc` + `memcpy` - it's cheap.
            // 2. Copy the atomic loads. Will be a relatively tight loop, but less cache-friendly.
            // 3. Since it's now initialized, reinterpret it as needed. (This part requires almost
            //    no compute, and also doesn't need the guard.)

            // SAFETY: It's okay to copy the whole thing at first - the atomics can be garbage at
            // first. The atomics will be initialized after with proper atomic instructions.
            unsafe {
                use std::alloc::*;

                let len = table_lock.len();
                // If the length is zero, an empty box is desired. (Also, it'll cause memory bugs
                // if I don't special-case this.)
                if len == 0 {
                    continue;
                }

                let ptr = table_lock.as_ptr();

                let layout = match Layout::array::<ByteCountTableEntrySnapshot>(len) {
                    Ok(layout) => layout,
                    Err(_) => return None,
                };

                let byte_ptr = alloc(layout);
                if byte_ptr.is_null() {
                    return None;
                }

                std::ptr::copy_nonoverlapping(ptr.cast(), byte_ptr, layout.size());

                let result_ptr: *mut ByteCountTableEntrySnapshot = byte_ptr.cast();

                for (i, entry) in table_lock.iter().enumerate() {
                    let bytes = entry.bytes.current();
                    let lines = entry.lines.current();
                    let target = result_ptr.add(i);
                    std::ptr::write(std::ptr::addr_of_mut!((*target).bytes), bytes);
                    std::ptr::write(std::ptr::addr_of_mut!((*target).lines), lines);
                }

                let result = Box::from_raw(std::ptr::slice_from_raw_parts_mut(result_ptr, len));

                drop(table_lock);

                *snapshot = result;
            }
        }

        // To ensure there's a defined order for a much easier time testing.
        for snapshot in &mut priority_table {
            snapshot.sort_by(|a, b| a.key.cmp(&b.key));
        }

        Some(ByteCountSnapshot { priority_table })
    }

    // Take a reference to avoid a copy.
    pub fn push_line(&self, key: &MessageKey, msg_len: usize) -> bool {
        // Services are very rarely added. Try opening a read first and doing atomic updates, and
        // fall back to a write lock if the entry doesn't exist yet. Contention should already be
        // low as-is since only two threads could be accessing the map, and it's further reduced by
        // the indirection on priority/severity level.
        //
        // Using an upgradable lock so I don't need to drop and re-acquire it.
        let priority_index = zero_extend_u8_usize(key.priority.as_severity_index());
        let read_lock = self.priority_table[priority_index]
            .read()
            .unwrap_or_else(|e| e.into_inner());

        if find_and_increment(&read_lock, key, msg_len) {
            return true;
        }

        // Don't deadlock. Drop the lock before entering the fallback path.
        drop(read_lock);

        return push_line_likely_new(&self.priority_table[priority_index], key, msg_len);

        fn find_and_increment(
            entries: &[ByteCountTableEntry],
            key: &MessageKey,
            msg_len: usize,
        ) -> bool {
            match entries.iter().find(|entry| entry.key == key.table_key) {
                None => false,
                Some(entry) => {
                    entry.lines.increment();
                    entry.bytes.increment_by(zero_extend_usize_u64(msg_len));
                    true
                }
            }
        }

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
            msg_len: usize,
        ) -> bool {
            // Entry doesn't exist. Time to acquire a write lock and update the hash map with a
            // possible new key.
            let mut write_lock = priority_entry.write().unwrap_or_else(|e| e.into_inner());

            if !find_and_increment(&write_lock, key, msg_len) {
                // While this may reallocate a lot at first, it's unlikely to reallocate too much
                // after that, since there's only so many system services. This is why it doesn't
                // try to pre-allocate - it's just not needed.

                // It's not copyable in production, as it's supposed to be minimally copied in
                // general (in no small part due to its size).
                #[allow(clippy::clone_on_copy)]
                let table_key = key.table_key.clone();

                let entry = ByteCountTableEntry {
                    lines: Counter::new(1),
                    bytes: Counter::new(zero_extend_usize_u64(msg_len)),
                    key: table_key,
                };

                // Just error. It's not fatal, just results in table state issues.
                if let Err(e) = write_lock.try_reserve(1) {
                    log::error!("Failed to push new table entry: {}", e);
                    return false;
                }

                write_lock.push(entry);
            }

            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn map_key(s: &[u8]) -> MessageKey {
        MessageKey::build(None, None, Some(s), Priority::Informational)
    }

    fn actual_snapshot(s: &'static PromState) -> PromSnapshot {
        s.snapshot().unwrap()
    }

    #[test]
    fn works_on_one_thread() {
        init_logger();
        static S: PromState = PromState::new();

        S.add_message_line_ingested(&map_key(b"one"), 123);
        S.add_message_line_ingested(&map_key(b"one"), 456);
        S.add_message_line_ingested(&map_key(b"two"), 789);
        S.add_message_line_ingested(&map_key(b"three"), 555);
        S.add_message_line_ingested(&map_key(b"three"), 444);

        const EXPECTED_DATA: &[ByteCountSnapshotEntry] = &[
            ByteCountSnapshotEntry {
                key: map_key(b"one"),
                lines: 2,
                bytes: 579,
            },
            ByteCountSnapshotEntry {
                key: map_key(b"two"),
                lines: 1,
                bytes: 789,
            },
            ByteCountSnapshotEntry {
                key: map_key(b"three"),
                lines: 2,
                bytes: 999,
            },
        ];

        assert_eq!(
            actual_snapshot(&S),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_ingested_bytes: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                metrics_requests: 0,
                messages_ingested: ByteCountSnapshot::build(EXPECTED_DATA.iter().cloned()),
            }
        );
    }

    const KEYS: [MessageKey; 16] = [
        map_key(b"test_1"),
        map_key(b"test_2"),
        map_key(b"test_3"),
        map_key(b"test_4"),
        map_key(b"test_5"),
        map_key(b"test_6"),
        map_key(b"test_7"),
        map_key(b"test_8"),
        map_key(b"test_9"),
        map_key(b"test_10"),
        map_key(b"test_11"),
        map_key(b"test_12"),
        map_key(b"test_13"),
        map_key(b"test_14"),
        map_key(b"test_15"),
        map_key(b"test_16"),
    ];

    fn test_contends_inner(state: &PromState, contends_list: &[MessageKey; 10]) {
        let mut test_modulo = 9_usize;
        for i in 1..=100 {
            for key in &KEYS {
                state.add_message_line_ingested(key, 10);
            }
            if i % 10 == test_modulo {
                test_modulo = test_modulo.wrapping_sub(1);
                state.add_message_line_ingested(&contends_list[i / 10], 1);
            }
        }
    }

    #[test]
    fn works_on_two_contending_threads() {
        init_logger();
        // Pre-allocate everything so it won't take a long time to run with all the allocations.
        // This is one of the slowest tests in Miri, so it's particularly helpful here.
        static S: PromState = PromState::new();

        static CONTENDS_0: [MessageKey; 10] = [
            map_key(b"contend_0_0"),
            map_key(b"contend_0_1"),
            map_key(b"contend_0_2"),
            map_key(b"contend_0_3"),
            map_key(b"contend_0_4"),
            map_key(b"contend_0_5"),
            map_key(b"contend_0_6"),
            map_key(b"contend_0_7"),
            map_key(b"contend_0_8"),
            map_key(b"contend_0_9"),
        ];

        static CONTENDS_1: [MessageKey; 10] = [
            map_key(b"contend_1_0"),
            map_key(b"contend_1_1"),
            map_key(b"contend_1_2"),
            map_key(b"contend_1_3"),
            map_key(b"contend_1_4"),
            map_key(b"contend_1_5"),
            map_key(b"contend_1_6"),
            map_key(b"contend_1_7"),
            map_key(b"contend_1_8"),
            map_key(b"contend_1_9"),
        ];

        #[rustfmt::skip]
        const EXPECTED_DATA: &[ByteCountSnapshotEntry] = &[
            ByteCountSnapshotEntry { key: map_key(b"test_1"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_2"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_3"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_4"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_5"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_6"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_7"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_8"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_9"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_10"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_11"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_12"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_13"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_14"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_15"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"test_16"), lines: 200, bytes: 2000 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_0"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_1"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_2"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_3"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_4"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_5"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_6"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_7"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_8"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_0_9"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_0"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_1"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_2"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_3"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_4"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_5"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_6"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_7"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_8"), lines: 1, bytes: 1 },
            ByteCountSnapshotEntry { key: map_key(b"contend_1_9"), lines: 1, bytes: 1 },
        ];

        std::thread::scope(|s| {
            s.spawn(move || test_contends_inner(&S, &CONTENDS_0));
            s.spawn(move || test_contends_inner(&S, &CONTENDS_1));
        });

        assert_eq!(
            actual_snapshot(&S),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_ingested_bytes: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                metrics_requests: 0,
                messages_ingested: ByteCountSnapshot::build(EXPECTED_DATA.iter().cloned()),
            }
        );
    }

    // Complete overkill, but specifically just trying to stress-test contention.
    #[test]
    // Extremely slow in Miri (might very well take over an hour). Just skip it.
    #[cfg_attr(miri, ignore)]
    fn works_on_a_hundred_contending_threads() {
        init_logger();
        // Pre-allocate everything so it won't take a long time to run with all the allocations.
        static S: PromState = PromState::new();

        const CONTENDS_LISTS: [[MessageKey; 10]; 100] = {
            let inner = [map_key(b"placeholder"); 10];
            let mut result = [inner; 100];
            let mut i = 0;
            while i < 100 {
                let mut j = 0;
                let tens = b'0'.wrapping_add(i / 10);
                let ones = b'0'.wrapping_add(i % 10);
                let mut inner = inner;
                while j < 10 {
                    let last = b'0'.wrapping_add(j);
                    let mut base = *b"contend_00_0";
                    base[8] = tens;
                    base[9] = ones;
                    base[11] = last;
                    inner[zero_extend_u8_usize(j)] = map_key(&base);
                    j += 1;
                }
                result[zero_extend_u8_usize(i)] = inner;
                i += 1;
            }
            result
        };

        const EXPECTED_DATA: &[ByteCountSnapshotEntry] = &{
            let mut result = [ByteCountSnapshotEntry {
                key: map_key(b"placeholder"),
                lines: 1,
                bytes: 1,
            }; KEYS.len() + 1000];

            // Don't format this bit. It'll just make this less readable.
            #[rustfmt::skip]
            #[allow(clippy::let_unit_value)]
            let _ = {
                result[0] = ByteCountSnapshotEntry { key: map_key(b"test_1"), lines: 10_000, bytes: 100_000 };
                result[1] = ByteCountSnapshotEntry { key: map_key(b"test_2"), lines: 10_000, bytes: 100_000 };
                result[2] = ByteCountSnapshotEntry { key: map_key(b"test_3"), lines: 10_000, bytes: 100_000 };
                result[3] = ByteCountSnapshotEntry { key: map_key(b"test_4"), lines: 10_000, bytes: 100_000 };
                result[4] = ByteCountSnapshotEntry { key: map_key(b"test_5"), lines: 10_000, bytes: 100_000 };
                result[5] = ByteCountSnapshotEntry { key: map_key(b"test_6"), lines: 10_000, bytes: 100_000 };
                result[6] = ByteCountSnapshotEntry { key: map_key(b"test_7"), lines: 10_000, bytes: 100_000 };
                result[7] = ByteCountSnapshotEntry { key: map_key(b"test_8"), lines: 10_000, bytes: 100_000 };
                result[8] = ByteCountSnapshotEntry { key: map_key(b"test_9"), lines: 10_000, bytes: 100_000 };
                result[9] = ByteCountSnapshotEntry { key: map_key(b"test_10"), lines: 10_000, bytes: 100_000 };
                result[10] = ByteCountSnapshotEntry { key: map_key(b"test_11"), lines: 10_000, bytes: 100_000 };
                result[11] = ByteCountSnapshotEntry { key: map_key(b"test_12"), lines: 10_000, bytes: 100_000 };
                result[12] = ByteCountSnapshotEntry { key: map_key(b"test_13"), lines: 10_000, bytes: 100_000 };
                result[13] = ByteCountSnapshotEntry { key: map_key(b"test_14"), lines: 10_000, bytes: 100_000 };
                result[14] = ByteCountSnapshotEntry { key: map_key(b"test_15"), lines: 10_000, bytes: 100_000 };
                result[15] = ByteCountSnapshotEntry { key: map_key(b"test_16"), lines: 10_000, bytes: 100_000 };
            };

            let mut target = 16;
            let mut i = 0;
            while i < CONTENDS_LISTS.len() {
                let entry = &CONTENDS_LISTS[i];
                let mut j = 0;
                while j < entry.len() {
                    result[target] = ByteCountSnapshotEntry {
                        key: entry[j],
                        lines: 1,
                        bytes: 1,
                    };
                    j += 1;
                    target += 1;
                }
                i += 1;
            }

            result
        };

        std::thread::scope(|s| {
            for list in &CONTENDS_LISTS {
                s.spawn(move || test_contends_inner(&S, list));
            }
        });

        assert_eq!(
            actual_snapshot(&S),
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_ingested_bytes: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                metrics_requests: 0,
                messages_ingested: ByteCountSnapshot::build(EXPECTED_DATA.iter().cloned()),
            }
        );
    }
}
