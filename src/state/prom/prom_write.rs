use crate::prelude::*;

use const_str::concat_bytes;
use std::time::SystemTime;

#[derive(Debug, PartialEq, Eq)]
pub struct PromSnapshot {
    pub entries_ingested: u64,
    pub fields_ingested: u64,
    pub data_ingested_bytes: u64,
    pub faults: u64,
    pub cursor_double_retries: u64,
    pub unreadable_fields: u64,
    pub corrupted_fields: u64,
    pub metrics_requests: u64,
    pub messages_ingested: ByteCountSnapshot,
    pub monitor_hits: ByteCountSnapshot,
}

// Max integer: 18446744073709551616
pub const MAX_USIZE_ASCII_BYTES: usize = 20;
const CREATED_MILLIS_SIZE: usize = 4;
const CREATED_BUFFER_SIZE: usize = match MAX_USIZE_ASCII_BYTES.checked_add(CREATED_MILLIS_SIZE) {
    Some(size) => size,
    None => unreachable!(),
};

pub struct PromEnvironment {
    // Inline `CREATED_BUFFER_SIZE` so sizes can auto-complete.
    created_buffer: [u8; 24],
    created_len: usize,
}

fn split_created_buffer(
    target: &mut [u8; CREATED_BUFFER_SIZE],
) -> (
    &mut [u8; MAX_USIZE_ASCII_BYTES],
    &mut [u8; CREATED_MILLIS_SIZE],
) {
    #![allow(clippy::as_conversions)]

    // SAFETY: `CREATED_BUFFER_SIZE` is strictly larger than either of its parts.
    unsafe {
        let target_u64 = target.as_mut_ptr();
        let target_millis = target_u64.add(MAX_USIZE_ASCII_BYTES);

        (
            &mut *(target_u64 as *mut [u8; MAX_USIZE_ASCII_BYTES]),
            &mut *(target_millis as *mut [u8; CREATED_MILLIS_SIZE]),
        )
    }
}

/// Returns the start offset. The end offset is always implicitly the end.
pub fn write_u64(target: &mut [u8; MAX_USIZE_ASCII_BYTES], mut value: u64) -> usize {
    // This serializes a `usize` without needing to allocate anything.

    // Special case that's easier to handle outside the main loop. This needs handled specially
    // anyways for fractional parts.
    if value == 0 {
        const HEAD: usize = MAX_USIZE_ASCII_BYTES.wrapping_sub(1);
        target[HEAD] = b'0';
        return HEAD;
    }

    // Take advantage of the fact it floors to simplify the loop and stringify it in a single pass.
    let mut head = target.len();

    while value > 0 {
        head = head.wrapping_sub(1);
        target[head] = truncate_u64_u8(value % 10).wrapping_add(b'0');
        value /= 10;
    }

    head
}

impl PromEnvironment {
    pub fn new(created: SystemTime) -> Self {
        let created = created
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);

        // Write it as a fixed-point decimal. Less work than writing out a floating point number, and
        // it's easier to render subsecond precision correctly.
        let mut created_buffer = [0; CREATED_BUFFER_SIZE];

        let created_start = {
            let (target_u64, target_millis) = split_created_buffer(&mut created_buffer);
            let created_start = write_u64(target_u64, created.as_secs());

            // Don't expose a high-resolution timer over the network. Write it out at millisecond
            // resolution only. Also shaves a few bytes off the output, as this is in the part that
            // generates the per-message entry rows, and it lets me simplify the generation a bit,
            // but the security part is the most important.
            let millis = created.subsec_millis();
            target_millis[0] = b'.';
            target_millis[1] = truncate_u32_u8(millis / 100).wrapping_add(b'0');
            target_millis[2] = truncate_u32_u8(millis / 10 % 10).wrapping_add(b'0');
            target_millis[3] = truncate_u32_u8(millis % 10).wrapping_add(b'0');

            created_buffer.copy_within(created_start.., 0);

            created_start
        };

        Self {
            created_buffer,
            created_len: created_buffer.len().wrapping_sub(created_start),
        }
    }

    fn created_bytes(&self) -> &[u8] {
        // SAFETY: `self.created_len < self.created_buffer.len()` per the constructor.
        unsafe { std::slice::from_raw_parts(self.created_buffer.as_ptr(), self.created_len) }
    }
}

fn data_bytes_from_id<'a>(table: &'a IdTable, id: &Option<u32>) -> &'a [u8] {
    id.and_then(|id| table.lookup_id(id))
        .map(|name| &**name)
        .unwrap_or(b"?")
}

struct Writer {
    result: Vec<u8>,
    value_buffer: [u8; MAX_USIZE_ASCII_BYTES],
}

impl Writer {
    fn new() -> Option<Self> {
        /*
        Start with a decently large capacity of 80 KiB to avoid spammy early reallocation. It's
        still likely to reallocate once or twice after that if there's enough services. It sounds
        like a lot, but it's very realistic.

        Let's assume the following:

        - There's a moderately conservative 50 services that have emit something into the journal.
        - Each service has emit some journal entries of type WARN and some of type INFO, but none
          of any other type.
        - Each service has only one user and group combo.
        - Each service message counter is 4 digits long on average.
        - Each service message byte counter is 6 digits long on average.
        - Each service consists of 24 characters on average.
        - Each user and group consists of 8 characters on average (when weighted by service count).
        - The current time consists of 10 seconds digits. (The first date that'll require 11 digits
          for seconds is 2286-11-20T17:46:40.000Z. At the time of writing, it's obviously going to
          be a while.)

        > Yes, that's a lot of assumptions. But it's not unrealistic.

        That comes out to a little over 1 KiB per service, or just shy of 60 KiB total. 80 KiB
        gives some breathing room.
        */
        Some(Self {
            result: try_new_dynamic_vec(80 * 1024)?,
            value_buffer: [0; MAX_USIZE_ASCII_BYTES],
        })
    }

    fn write_global_counter(
        &mut self,
        constants: &'static GlobalCounterConstants,
        environment: &PromEnvironment,
        value: u64,
    ) -> bool {
        let head = write_u64(&mut self.value_buffer, value);

        write_slices(
            &mut self.result,
            &[
                constants.header,
                environment.created_bytes(),
                constants.total_label,
                &self.value_buffer[head..],
            ],
        )
    }

    fn write_message_counters(
        &mut self,
        constants: &'static MessageCounterConstants,
        environment: &PromEnvironment,
        snapshot: &ByteCountSnapshot,
        table: &UidGidTable,
    ) -> bool {
        if snapshot.is_empty() {
            // Don't break sum
            write_slices(
                &mut self.result,
                &[
                    constants.empty_fallback_header,
                    environment.created_bytes(),
                    constants.empty_fallback_total,
                ],
            )
        } else {
            if !write_slices(&mut self.result, &[constants.header]) {
                return false;
            }

            snapshot.each_while(|data| {
                let head = write_u64(
                    &mut self.value_buffer,
                    match constants.kind {
                        MessageCounterKind::Lines => data.lines,
                        MessageCounterKind::Bytes => data.bytes,
                    },
                );

                let service_bytes = data.key.service().map(|s| s.as_bytes()).unwrap_or(b"?");
                let priority_name = data.key.priority.as_name_bytes();
                let priority_severity = [data.key.priority.as_severity_byte()];
                let user_name = data_bytes_from_id(&table.uids, &data.key.uid);
                let group_name = data_bytes_from_id(&table.gids, &data.key.gid);

                let mut slice_data = [
                    // *_created key
                    constants.created_prefix,
                    service_bytes,
                    b"\",priority=\"",
                    priority_name,
                    b"\",severity=\"",
                    &priority_severity,
                    b"\",user=\"",
                    user_name,
                    b"\",group=\"",
                    group_name,
                    b"\"} ",
                    environment.created_bytes(),
                    // *_total key
                    constants.total_prefix,
                    service_bytes,
                    b"\",priority=\"",
                    priority_name,
                    b"\",severity=\"",
                    &priority_severity,
                    b"\",user=\"",
                    user_name,
                    b"\",group=\"",
                    group_name,
                    b"\"} ",
                    &self.value_buffer[head..],
                    b"",
                    b"",
                    b"",
                    b"",
                ];
                let mut len = slice_data.len().wrapping_sub(4);

                if let Some(name) = &data.name {
                    let name_label = b"\",name=\"";
                    let name_value = name.as_bytes();
                    len = slice_data.len();
                    slice_data.copy_within(22..24, 26);
                    slice_data.copy_within(10..22, 12);
                    slice_data[10] = name_label;
                    slice_data[11] = name_value;
                    slice_data[24] = name_label;
                    slice_data[25] = name_value;
                }

                write_slices(&mut self.result, &slice_data[..len])
            })
        }
    }
}

struct GlobalCounterConstants {
    header: &'static [u8],
    total_label: &'static [u8],
}

enum MessageCounterKind {
    Lines,
    Bytes,
}

struct MessageCounterConstants {
    kind: MessageCounterKind,
    header: &'static [u8],
    empty_fallback_header: &'static [u8],
    empty_fallback_total: &'static [u8],
    created_prefix: &'static [u8],
    total_prefix: &'static [u8],
}

pub fn render_openapi_metrics(
    environment: &PromEnvironment,
    snapshot: &PromSnapshot,
    table: &UidGidTable,
) -> Option<Vec<u8>> {
    let mut writer = Writer::new()?;

    // This macro hackery literally makes this reasonable.
    macro_rules! counter_header {
        ($(is_first:$is_first:expr,)? key:$key:ident, $(unit:$unit:expr,)? help:$help:expr $(,)?) => {{
            const IS_FIRST: bool = {
                #[allow(unused)]
                let is_first = false;
                $(let is_first = $is_first;)?
                is_first
            };

            const NAME: &[u8] = concat_bytes!("journald_", stringify!($key));

            const HEADER_NO_HELP: &[u8] = concat_bytes!(
                if IS_FIRST { ipc::parent::METRICS_RESPONSE_HEADER } else { b"\n" },
                b"# TYPE ", NAME, " counter",
                $("\n# UNIT ", NAME, " ", stringify!($unit),)?
            );

            #[cfg(test)]
            const HEADER: &[u8] = HEADER_NO_HELP;

            #[cfg(not(test))]
            const HEADER: &[u8] = concat_bytes!(HEADER_NO_HELP, "\n# HELP ", NAME, " ", $help);

            HEADER
        }};
    }

    macro_rules! write_global_counter {
        ($(is_first:$is_first:expr,)? key:$key:ident, $(unit:$unit:expr,)? help:$help:expr $(,)?) => {{
            const NAME: &[u8] = concat_bytes!("journald_", stringify!($key));

            static CONSTANTS: GlobalCounterConstants = GlobalCounterConstants {
                header: concat_bytes!(
                    counter_header! {
                        $(is_first: $is_first,)?
                        key: $key,
                        $(unit: $unit,)?
                        help: $help,
                    },
                    "\n", NAME, "_created "
                ),
                total_label: concat_bytes!(b"\n", NAME, "_total "),
            };
            if !writer.write_global_counter(&CONSTANTS, environment, snapshot.$key) {
                return None;
            }
        }};
    }

    macro_rules! write_message_counter {
        (kind:$kind:ident, source:$source:ident, key:$key:ident, $(unit:$unit:expr,)? help:$help:expr $(,)?) => {{
            const HEADER: &[u8] = counter_header! {
                key:$key,
                $(unit:$unit,)?
                help:$help,
            };

            const NAME: &[u8] = concat_bytes!("journald_", stringify!($key));

            static CONSTANTS: MessageCounterConstants = MessageCounterConstants {
                kind: MessageCounterKind::$kind,
                header: HEADER,
                empty_fallback_header: concat_bytes!(HEADER, "\n", NAME, "_created "),
                empty_fallback_total: concat_bytes!("\n", NAME, "_total 0"),
                created_prefix: concat_bytes!("\n", NAME, "_created{service=\""),
                total_prefix: concat_bytes!("\n", NAME, "_total{service=\""),
            };
            if !writer.write_message_counters(&CONSTANTS, environment, &snapshot.$source, table) {
                return None;
            }
        }};
    }

    // Global counters
    write_global_counter! {
        is_first: true,
        key: entries_ingested,
        help: b"\
            The total number of data field bytes ingested across all fields, including both keys \
            and their values.\
        ",
    }
    write_global_counter! {
        key: fields_ingested,
        help: b"The total number of data fields read. Normally it's 5 fields (`_SYSTEMD_UNIT`, \
        `PRIORITY`, `_UID`, `_GID`, and `MESSAGE`) for message entries, but may be fewer in the \
        case of incomplete fields and such.",
    }
    write_global_counter! {
        key: data_ingested_bytes,
        unit: bytes,
        help: b"The total number of data field bytes ingested across all fields, including both keys and \
        their values.",
    }
    write_global_counter! {
        key: faults,
        help: b"The total number of faults encountered while reading the journal.",
    }
    write_global_counter! {
        key: cursor_double_retries,
        help: b"Total number of faults encountered while recovering after a previous fault. Also \
        increments if it fails on first read. Note: too many of these in a short period of \
        time will cause entire program to crash.",
    }
    write_global_counter! {
        key: unreadable_fields,
        help: b"The total number of fields unreadable for reasons other than being corrupted \
        (usually, too large to be read).",
    }
    write_global_counter! {
        key: corrupted_fields,
        help: b"The total number of corrupted entries detected while reading the journal that could \
        still be read.",
    }
    write_global_counter! {
        key: metrics_requests,
        help: b"The total number of requests received, including requests to paths other than the \
        standard `GET /metrics` route.",
    }

    // Per-service counters
    write_message_counter! {
        kind: Lines,
        source: messages_ingested,
        key: messages_ingested,
        help: b"Number of message entries successfully processed.",
    }
    write_message_counter! {
        kind: Bytes,
        source: messages_ingested,
        key: messages_ingested_bytes,
        unit: bytes,
        help: b"Total number of `MESSAGE` field bytes ingested.",
    }
    write_message_counter! {
        kind: Lines,
        source: monitor_hits,
        key: monitor_hits,
        help: b"Number of hits corresponding to a specified monitor, denoted by its `name` attribute.",
    }
    write_message_counter! {
        kind: Bytes,
        source: monitor_hits,
        key: monitor_hits_bytes,
        unit: bytes,
        help: b"Number of bytes across all hits corresponding to a specified monitor, denoted by its `name` attribute.",
    }

    if !write_slices(&mut writer.result, &[b"\n# EOF\n"]) {
        return None;
    }

    ipc::parent::finish_response_metrics(&mut writer.result);

    Some(writer.result)
}
