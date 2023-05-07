use crate::prelude::*;
use std::collections::HashMap;

// TODO: optimize the layout of `FilterEntry`.

struct FilterEntry {
    monitor_name: Arc<str>,
    priority: Option<Priority>,
    uid: Option<Option<u32>>,
    gid: Option<Option<u32>>,
    service: Option<ServiceRepr>,
    map: ByteCountMap,
}

impl FilterEntry {
    fn build(entry: &MonitorFilterResolved) -> Self {
        Self {
            monitor_name: Arc::clone(&entry.monitor_name),
            priority: entry.priority,
            uid: entry.uid,
            gid: entry.gid,
            service: entry.service.clone(),
            map: ByteCountMap::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MonitorFilterResolved {
    pub monitor_name: Arc<str>,
    pub priority: Option<Priority>,
    pub uid: Option<Option<u32>>,
    pub gid: Option<Option<u32>>,
    pub service: Option<ServiceRepr>,
    pub message_pattern: Option<Box<str>>,
}

fn try_hit_slice(entries: &[FilterEntry], key: &MessageKey, msg_len: usize, faults: &Counter) {
    for entry in entries.iter() {
        if let Some(priority) = entry.priority {
            if priority != key.priority {
                return;
            }
        }

        if let Some(uid) = entry.uid {
            if uid != key.uid {
                return;
            }
        }

        if let Some(gid) = entry.gid {
            if gid != key.gid {
                return;
            }
        }

        if let Some(service) = &entry.service {
            if service.matches(&key.service) {
                return;
            }
        }

        if !entry.map.push_line(key, msg_len) {
            faults.increment();
        };
    }
}

pub struct MonitorFilter {
    message_set: regex::bytes::RegexSet,
    // The last entry is the catch-all map.
    filter_entries: Box<[FilterEntry]>,
    message_map_ranges: Box<[std::ops::Range<usize>]>,
    fallback_start: usize,
}

impl MonitorFilter {
    pub fn new(entries: &[MonitorFilterResolved]) -> Self {
        let mut filter_entry_lists = Vec::new();
        let mut non_message_entries = Vec::new();
        let mut message_pattern_indices: HashMap<Box<str>, usize> = HashMap::new();
        let message_set = regex::bytes::RegexSet::new(entries.iter().filter_map(|e| {
            if let Some(pattern) = &e.message_pattern {
                let index = match message_pattern_indices.get(pattern) {
                    Some(value) => *value,
                    None => {
                        let value = filter_entry_lists.len();
                        message_pattern_indices.insert(pattern.clone(), value);
                        filter_entry_lists.push(Vec::new());
                        value
                    }
                };

                filter_entry_lists[index].push(FilterEntry::build(e));
                Some(&**pattern)
            } else {
                non_message_entries.push(FilterEntry::build(e));
                None
            }
        }))
        .expect("failed to build message regex set");

        let mut filter_entries = Vec::with_capacity(entries.len());

        let message_map_ranges = Box::from_iter(filter_entry_lists.into_iter().map(|list| {
            let start = filter_entries.len();
            let end = start.wrapping_add(list.len());
            filter_entries.extend(list);
            start..end
        }));

        let fallback_start = filter_entries.len();
        filter_entries.extend(non_message_entries);

        Self {
            message_set,
            filter_entries: filter_entries.into(),
            message_map_ranges,
            fallback_start,
        }
    }

    // Take a reference to avoid a copy.
    pub fn try_hit(&self, key: &MessageKey, msg: &[u8], faults: &Counter) {
        for index in self.message_set.matches(msg) {
            try_hit_slice(
                &self.filter_entries[self.message_map_ranges[index].clone()],
                key,
                msg.len(),
                faults,
            );
        }

        try_hit_slice(
            &self.filter_entries[..self.fallback_start],
            key,
            msg.len(),
            faults,
        );
    }

    pub fn snapshot(&self) -> Option<ByteCountSnapshot> {
        let mut result = Vec::new();

        for entry in self.filter_entries.iter() {
            let mut snapshot = entry.map.snapshot()?.into_inner();

            for item in snapshot.iter_mut() {
                item.name = Some(Arc::clone(&entry.monitor_name));
            }

            result.try_reserve_exact(snapshot.len()).ok()?;
            result.extend(snapshot.into_vec());
        }

        Some(ByteCountSnapshot::new(result.into()))
    }
}
