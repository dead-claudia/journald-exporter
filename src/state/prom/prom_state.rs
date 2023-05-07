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
    monitor_hits: OnceCell<Option<MonitorFilter>>,
}

pub struct MonitorFilterLease<'a>(&'a PromState);

impl Drop for MonitorFilterLease<'_> {
    fn drop(&mut self) {
        if self.0.monitor_hits.get().is_some() {
            // SAFETY: This is safe provided all the requirements are met with
            // `initialize_monitor_filter`.
            unsafe {
                let ptr: *const OnceCell<_> = &self.0.monitor_hits;
                ptr.cast_mut().as_mut().unwrap().take();
            }
        }
    }
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
            monitor_hits: OnceCell::new(),
        }
    }

    /// SAFETY: Make sure to always keep the handle alive for the duration of the state's usage.
    #[must_use]
    pub unsafe fn initialize_monitor_filter(
        &self,
        filter: Option<MonitorFilter>,
    ) -> MonitorFilterLease {
        if self.monitor_hits.set(filter).is_err() {
            std::panic::panic_any("Filter already initialized");
        }
        MonitorFilterLease(self)
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

    pub fn add_message_line_ingested(&self, key: &MessageKey, msg: &[u8]) {
        if !self.messages_ingested.push_line(key, msg.len()) {
            self.add_fault();
        }

        if let Some(Some(filter)) = self.monitor_hits.get() {
            filter.try_hit(key, msg, &self.faults);
        }
    }

    pub fn snapshot(&self) -> Option<PromSnapshot> {
        Some(PromSnapshot {
            entries_ingested: self.entries_ingested.current(),
            fields_ingested: self.fields_ingested.current(),
            data_ingested_bytes: self.data_ingested_bytes.current(),
            faults: self.faults.current(),
            cursor_double_retries: self.cursor_double_retries.current(),
            unreadable_fields: self.unreadable_fields.current(),
            corrupted_fields: self.corrupted_fields.current(),
            metrics_requests: self.metrics_requests.current(),
            messages_ingested: self.messages_ingested.snapshot()?,
            monitor_hits: match self.monitor_hits.get() {
                None => std::panic::panic_any("Monitor filter not initialized."),
                Some(None) => ByteCountSnapshot::empty(),
                Some(Some(hits)) => hits.snapshot()?,
            },
        })
    }
}
