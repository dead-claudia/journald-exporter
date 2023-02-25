// IPC server parsing and generation tests

mod common;

// Test groups
mod read_contiguous_after_key_request;
mod read_contiguous_after_metrics_request;
mod read_contiguous_immediate;
mod read_header;
mod read_split_after_key_request;
mod read_split_after_metrics_request;
mod read_split_immediate;
