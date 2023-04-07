use crate::prelude::*;

use crate::state::ByteCountMap;
use crate::state::MessageKey;

pub struct PromState {
    entries_ingested: Counter,
    fields_ingested: Counter,
    data_ingested_bytes: Counter,
    faults: Counter,
    cursor_double_retries: Counter,
    unreadable_fields: Counter,
    corrupted_fields: Counter,
    metrics_requests: Counter,
    messages_ingested: ByteCountMap,
}

impl PromState {
    pub const fn new() -> PromState {
        PromState {
            entries_ingested: Counter::new(0),
            fields_ingested: Counter::new(0),
            data_ingested_bytes: Counter::new(0),
            faults: Counter::new(0),
            cursor_double_retries: Counter::new(0),
            unreadable_fields: Counter::new(0),
            corrupted_fields: Counter::new(0),
            metrics_requests: Counter::new(0),
            messages_ingested: ByteCountMap::new(),
        }
    }

    #[cold]
    pub fn add_fault(&self) {
        self.faults.increment();
    }

    #[cold]
    pub fn add_cursor_double_retry(&self) {
        self.cursor_double_retries.increment();
    }

    #[cold]
    pub fn add_unreadable_field(&self) {
        self.unreadable_fields.increment();
    }

    #[cold]
    pub fn add_corrupted_field(&self) {
        self.corrupted_fields.increment();
    }

    pub fn add_entry_ingested(&self) {
        self.entries_ingested.increment();
    }

    pub fn add_field_ingested(&self, bytes: usize) {
        self.fields_ingested.increment();
        self.data_ingested_bytes
            .increment_by(zero_extend_usize_u64(bytes));
    }

    pub fn add_metrics_requests(&self, requests: usize) {
        self.metrics_requests
            .increment_by(zero_extend_usize_u64(requests));
    }

    pub fn add_message_line_ingested(&self, key: &MessageKey, msg_len: usize) {
        self.messages_ingested.push_line(key, msg_len);
    }

    pub fn snapshot(&self) -> PromSnapshot {
        PromSnapshot {
            entries_ingested: self.entries_ingested.current(),
            fields_ingested: self.fields_ingested.current(),
            data_ingested_bytes: self.data_ingested_bytes.current(),
            faults: self.faults.current(),
            cursor_double_retries: self.cursor_double_retries.current(),
            unreadable_fields: self.unreadable_fields.current(),
            corrupted_fields: self.corrupted_fields.current(),
            metrics_requests: self.metrics_requests.current(),
            messages_ingested: self.messages_ingested.snapshot(),
        }
    }
}
