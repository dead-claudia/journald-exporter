// IPC server parsing and generation tests

mod common;

// Test groups
mod read_contiguous_after_key_set;
mod read_contiguous_after_metrics_response;
mod read_contiguous_immediate;
mod read_header;
mod read_split_after_key_set;
mod read_split_after_metrics_response;
mod read_split_immediate;
mod write_receive_key_set;
