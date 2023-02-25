use crate::prelude::*;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromSnapshot {
    pub entries_ingested: u64,
    pub fields_ingested: u64,
    pub data_bytes_ingested: u64,
    pub faults: u64,
    pub cursor_double_retries: u64,
    pub unreadable_fields: u64,
    pub corrupted_fields: u64,
    pub requests: u64,
    pub messages_ingested: ByteCountSnapshot,
}

#[cfg(test)]
impl Arbitrary for PromSnapshot {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            entries_ingested: Arbitrary::arbitrary(g),
            fields_ingested: Arbitrary::arbitrary(g),
            data_bytes_ingested: Arbitrary::arbitrary(g),
            faults: Arbitrary::arbitrary(g),
            cursor_double_retries: Arbitrary::arbitrary(g),
            unreadable_fields: Arbitrary::arbitrary(g),
            corrupted_fields: Arbitrary::arbitrary(g),
            requests: Arbitrary::arbitrary(g),
            messages_ingested: Arbitrary::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            (
                (
                    self.entries_ingested,
                    self.fields_ingested,
                    self.data_bytes_ingested,
                    self.faults,
                    self.cursor_double_retries,
                    self.unreadable_fields,
                    self.corrupted_fields,
                    self.requests,
                ),
                self.messages_ingested.clone(),
            )
                .shrink()
                .map(
                    |(
                        (
                            entries_ingested,
                            fields_ingested,
                            data_bytes_ingested,
                            faults,
                            cursor_double_retries,
                            unreadable_fields,
                            corrupted_fields,
                            requests,
                        ),
                        messages_ingested,
                    )| Self {
                        entries_ingested,
                        fields_ingested,
                        data_bytes_ingested,
                        faults,
                        cursor_double_retries,
                        unreadable_fields,
                        corrupted_fields,
                        requests,
                        messages_ingested,
                    },
                ),
        )
    }
}

#[derive(Debug, Clone)]
pub struct PromEnvironment {
    pub created: SystemTime,
}

#[cfg(test)]
impl Arbitrary for PromEnvironment {
    fn arbitrary(g: &mut Gen) -> Self {
        let duration = <Duration>::arbitrary(g);

        Self {
            created: SystemTime::UNIX_EPOCH + duration,
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.created.shrink().map(|created| Self { created }))
    }
}

#[derive(Debug)]
pub struct PromWriteContext<'a> {
    pub environment: &'a PromEnvironment,
    pub snapshot: &'a PromSnapshot,
    pub table: &'a UidGidTable,
}

impl<'a> Clone for PromWriteContext<'a> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> Copy for PromWriteContext<'a> {}

fn write_slice(result: &mut Vec<u8>, bytes: &[u8]) {
    result.extend_from_slice(bytes);
}

fn write_u64(result: &mut Vec<u8>, mut value: u64) {
    // This serializes a `usize` without needing to allocate anything.

    // Max integer: 18446744073709551616
    const MAX_USIZE_ASCII_BYTES: usize = 20;

    // Special case that's easier to handle outside the main loop. This needs handled specially
    // anyways for fractional parts.
    if value == 0 {
        result.push(b'0');
        return;
    }

    // Take advantage of the fact it floors to simplify the loop and stringify it in a single pass.
    let mut bytes = [b'0'; MAX_USIZE_ASCII_BYTES];
    let mut head = MAX_USIZE_ASCII_BYTES;

    while value > 0 {
        head = head.wrapping_sub(1);
        bytes[head] = truncate_u64_u8(value % 10).wrapping_add(b'0');
        value /= 10;
    }

    write_slice(result, &bytes[head..]);
}

fn write_created(result: &mut Vec<u8>, c: PromWriteContext) {
    let created = c
        .environment
        .created
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);

    // Write it as a fixed-point decimal. Less work than writing out a floating point number, and
    // it's easier to render subsecond precision correctly.
    write_u64(result, created.as_secs());

    // Don't expose a high-resolution timer over the network. Write it out at millisecond
    // resolution only. Also shaves a few bytes off the output, as this is in the part that
    // generates the per-message entry rows, and it lets me simplify the generation a bit, but
    // the security part is the most important.
    let millis = created.subsec_millis();
    result.extend_from_slice(&[
        b'.',
        truncate_u32_u8(millis / 100).wrapping_add(b'0'),
        truncate_u32_u8(millis / 10 % 10).wrapping_add(b'0'),
        truncate_u32_u8(millis % 10).wrapping_add(b'0'),
    ]);
}

fn write_data_or_question_mark(result: &mut Vec<u8>, data: Option<&[u8]>) {
    if let Some(inner) = data.as_ref() {
        write_slice(result, inner);
    } else {
        result.push(b'?');
    }
}

fn write_message_counter_prefix(
    result: &mut Vec<u8>,
    c: PromWriteContext,
    data: &ByteCountSnapshotEntry,
    name: &[u8],
    part: &[u8],
) {
    write_slice(result, name);
    write_slice(result, part);
    write_slice(result, b"{service=\"");
    write_data_or_question_mark(result, data.key.service().map(|s| s.as_bytes()));
    write_slice(result, b"\",priority=\"");
    write_slice(result, data.key.priority().as_name_bytes());
    write_slice(result, b"\",severity=\"");
    result.push(data.key.priority().as_severity_byte());
    write_slice(result, b"\",user=\"");
    write_data_or_question_mark(
        result,
        data.key
            .uid()
            .and_then(|uid| c.table.lookup_uid(uid).map(|name| &**name)),
    );
    write_slice(result, b"\",group=\"");
    write_data_or_question_mark(
        result,
        data.key
            .gid()
            .and_then(|gid| c.table.lookup_gid(gid).map(|name| &**name)),
    );
    write_slice(result, b"\"} ");
}

fn write_counter_value(result: &mut Vec<u8>, c: PromWriteContext, name: &'static [u8], value: u64) {
    write_slice(result, name);
    write_slice(result, b"_created ");
    write_created(result, c);
    result.push(b'\n');
    write_slice(result, name);
    write_slice(result, b"_total ");
    write_u64(result, value);
    result.push(b'\n');
}

fn write_global_counter(
    result: &mut Vec<u8>,
    c: PromWriteContext,
    name: &'static [u8],
    value: u64,
) {
    write_slice(result, b"# TYPE ");
    write_slice(result, name);
    write_slice(result, b" counter\n");
    write_counter_value(result, c, name, value);
}

fn write_message_counters(
    result: &mut Vec<u8>,
    c: PromWriteContext,
    name: &'static [u8],
    is_bytes: bool,
) {
    write_slice(result, b"# TYPE ");
    write_slice(result, name);
    write_slice(result, b" counter\n");

    if is_bytes {
        write_slice(result, b"# UNIT ");
        write_slice(result, name);
        write_slice(result, b" bytes\n");
    }

    if c.snapshot.messages_ingested.data.is_empty() {
        // Don't break sum
        write_counter_value(result, c, name, 0);
    } else {
        for data in c.snapshot.messages_ingested.data.iter() {
            let value = if is_bytes { data.bytes } else { data.lines };

            write_message_counter_prefix(result, c, data, name, b"_created");
            write_created(result, c);
            result.push(b'\n');

            write_message_counter_prefix(result, c, data, name, b"_total");
            write_u64(result, value);
            result.push(b'\n');
        }
    }
}

pub fn render_openapi_metrics(c: PromWriteContext) -> Box<[u8]> {
    /*
    Start with a decently large capacity of 80 KiB to avoid spammy early reallocation. It's still
    likely to reallocate once or twice after that if there's enough services. It sounds like a lot,
    but it's very realistic.

    Let's assume the following:

    - There's a moderately conservative 50 services that have emit something into the journal.
    - Each service has emit some journal entries of type WARN and some of type INFO, but none of
      any other type.
    - Each service has only one user and group combo.
    - Each service message counter is 4 digits long on average.
    - Each service message byte counter is 6 digits long on average.
    - Each service consists of 24 characters on average.
    - Each user and group consists of 8 characters on average (when weighted by service count).
    - The current time consists of 10 seconds digits. (The first date that'll require 11 digits for
      seconds is 2286-11-20T17:46:40.000Z. At the time of writing, it's obviously going to be a
      while.)

    > Yes, that's a lot of assumptions. But it's not unrealistic.

    That comes out to a little over 1 KiB per service, or just shy of 60 KiB total. 80 KiB gives
    some breathing room.
    */
    let mut result = Vec::with_capacity(80 * 1024);

    ipc::parent::init_response_metrics_header(&mut result);

    // Global counters
    write_global_counter(
        &mut result,
        c,
        b"journald_entries_ingested",
        c.snapshot.entries_ingested,
    );
    write_global_counter(
        &mut result,
        c,
        b"journald_fields_ingested",
        c.snapshot.fields_ingested,
    );
    write_global_counter(
        &mut result,
        c,
        b"journald_data_bytes_ingested",
        c.snapshot.data_bytes_ingested,
    );
    write_global_counter(&mut result, c, b"journald_faults", c.snapshot.faults);
    write_global_counter(
        &mut result,
        c,
        b"journald_cursor_double_retries",
        c.snapshot.cursor_double_retries,
    );
    write_global_counter(
        &mut result,
        c,
        b"journald_unreadable_fields",
        c.snapshot.unreadable_fields,
    );
    write_global_counter(
        &mut result,
        c,
        b"journald_corrupted_fields",
        c.snapshot.corrupted_fields,
    );
    write_global_counter(
        &mut result,
        c,
        b"journald_metrics_requests",
        c.snapshot.requests,
    );

    // Per-service counters
    // journald_messages_ingested
    write_message_counters(&mut result, c, b"journald_messages_ingested", false);
    write_message_counters(&mut result, c, b"journald_messages_ingested_bytes", true);

    write_slice(&mut result, b"# EOF\n");

    ipc::parent::finish_response_metrics(result)
}

#[cfg(test)]
mod test {
    use super::*;

    //                                               #####  #
    //  #    # #####  # ##### ######         #    # #     # #    #
    //  #    # #    # #   #   #              #    # #       #    #
    //  #    # #    # #   #   #####          #    # ######  #    #
    //  # ## # #####  #   #   #              #    # #     # #######
    //  ##  ## #   #  #   #   #              #    # #     #      #
    //  #    # #    # #   #   ######          ####   #####       #
    //                               #######

    #[test]
    fn write_u64_works_for_zero() {
        let mut result = vec![];
        write_u64(&mut result, 0);
        assert_eq!(result, b"0");
    }

    #[test]
    fn write_u64_works_for_positive_single_digits() {
        let mut result = vec![];
        write_u64(&mut result, 1);
        assert_eq!(result, b"1");
        let mut result = vec![];
        write_u64(&mut result, 2);
        assert_eq!(result, b"2");
        let mut result = vec![];
        write_u64(&mut result, 3);
        assert_eq!(result, b"3");
        let mut result = vec![];
        write_u64(&mut result, 4);
        assert_eq!(result, b"4");
        let mut result = vec![];
        write_u64(&mut result, 5);
        assert_eq!(result, b"5");
        let mut result = vec![];
        write_u64(&mut result, 6);
        assert_eq!(result, b"6");
        let mut result = vec![];
        write_u64(&mut result, 7);
        assert_eq!(result, b"7");
        let mut result = vec![];
        write_u64(&mut result, 8);
        assert_eq!(result, b"8");
        let mut result = vec![];
        write_u64(&mut result, 9);
        assert_eq!(result, b"9");
    }

    #[test]
    fn write_u64_works_for_2_digits() {
        let mut result = vec![];
        write_u64(&mut result, 12);
        assert_eq!(result, b"12");
    }

    #[test]
    fn write_u64_works_for_3_digits() {
        let mut result = vec![];
        write_u64(&mut result, 123);
        assert_eq!(result, b"123");
    }

    #[test]
    fn write_u64_works_for_4_digits() {
        let mut result = vec![];
        write_u64(&mut result, 1234);
        assert_eq!(result, b"1234");
    }

    #[test]
    fn write_u64_works_for_5_digits() {
        let mut result = vec![];
        write_u64(&mut result, 12345);
        assert_eq!(result, b"12345");
    }

    #[test]
    fn write_u64_works_for_6_digits() {
        let mut result = vec![];
        write_u64(&mut result, 123456);
        assert_eq!(result, b"123456");
    }

    #[test]
    fn write_u64_works_for_7_digits() {
        let mut result = vec![];
        write_u64(&mut result, 1234567);
        assert_eq!(result, b"1234567");
    }

    #[test]
    fn write_u64_works_for_8_digits() {
        let mut result = vec![];
        write_u64(&mut result, 12345678);
        assert_eq!(result, b"12345678");
    }

    #[test]
    fn write_u64_works_for_9_digits() {
        let mut result = vec![];
        write_u64(&mut result, 123456789);
        assert_eq!(result, b"123456789");
    }

    #[test]
    fn write_u64_works_for_10_digits() {
        let mut result = vec![];
        write_u64(&mut result, 1234567890);
        assert_eq!(result, b"1234567890");
    }

    #[test]
    fn write_u64_works_for_11_digits() {
        let mut result = vec![];
        write_u64(&mut result, 12345678901);
        assert_eq!(result, b"12345678901");
    }

    #[test]
    fn write_u64_works_for_12_digits() {
        let mut result = vec![];
        write_u64(&mut result, 123456789012);
        assert_eq!(result, b"123456789012");
    }

    #[test]
    fn write_u64_works_for_13_digits() {
        let mut result = vec![];
        write_u64(&mut result, 1234567890123);
        assert_eq!(result, b"1234567890123");
    }

    #[test]
    fn write_u64_works_for_14_digits() {
        let mut result = vec![];
        write_u64(&mut result, 12345678901234);
        assert_eq!(result, b"12345678901234");
    }

    #[test]
    fn write_u64_works_for_15_digits() {
        let mut result = vec![];
        write_u64(&mut result, 123456789012345);
        assert_eq!(result, b"123456789012345");
    }

    #[test]
    fn write_u64_works_for_16_digits() {
        let mut result = vec![];
        write_u64(&mut result, 1234567890123456);
        assert_eq!(result, b"1234567890123456");
    }

    #[test]
    fn write_u64_works_for_17_digits() {
        let mut result = vec![];
        write_u64(&mut result, 12345678901234567);
        assert_eq!(result, b"12345678901234567");
    }

    #[test]
    fn write_u64_works_for_18_digits() {
        let mut result = vec![];
        write_u64(&mut result, 123456789012345678);
        assert_eq!(result, b"123456789012345678");
    }

    #[test]
    fn write_u64_works_for_19_digits() {
        let mut result = vec![];
        write_u64(&mut result, 1234567890123456789);
        assert_eq!(result, b"1234567890123456789");
    }

    #[test]
    fn write_u64_works_for_20_digits() {
        let mut result = vec![];
        write_u64(&mut result, 12345678901234567890);
        assert_eq!(result, b"12345678901234567890");
    }

    //  #    # #####  # ##### ######          ####  #####  ######   ##   ##### ###### #####
    //  #    # #    # #   #   #              #    # #    # #       #  #    #   #      #    #
    //  #    # #    # #   #   #####          #      #    # #####  #    #   #   #####  #    #
    //  # ## # #####  #   #   #              #      #####  #      ######   #   #      #    #
    //  ##  ## #   #  #   #   #              #    # #   #  #      #    #   #   #      #    #
    //  #    # #    # #   #   ######          ####  #    # ###### #    #   #   ###### #####
    //                               #######

    // Doing this allows me to let the function better encapsulate everything.
    fn render_created(created: Duration) -> Box<[u8]> {
        let mut result = vec![];
        let context = PromWriteContext {
            environment: &PromEnvironment {
                created: SystemTime::UNIX_EPOCH + created,
            },
            snapshot: &PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot { data: Box::new([]) },
            },
            table: get_user_group_table(),
        };
        write_created(&mut result, context);
        result.into()
    }

    #[test]
    fn write_created_works_for_zero() {
        assert_eq!(&*render_created(Duration::new(0, 456000000)), b"0.456");
    }

    #[test]
    fn write_created_works_for_positive_single_digits() {
        assert_eq!(&*render_created(Duration::new(1, 456000000)), b"1.456");
        assert_eq!(&*render_created(Duration::new(2, 456000000)), b"2.456");
        assert_eq!(&*render_created(Duration::new(3, 456000000)), b"3.456");
        assert_eq!(&*render_created(Duration::new(4, 456000000)), b"4.456");
        assert_eq!(&*render_created(Duration::new(5, 456000000)), b"5.456");
        assert_eq!(&*render_created(Duration::new(6, 456000000)), b"6.456");
        assert_eq!(&*render_created(Duration::new(7, 456000000)), b"7.456");
        assert_eq!(&*render_created(Duration::new(8, 456000000)), b"8.456");
        assert_eq!(&*render_created(Duration::new(9, 456000000)), b"9.456");
    }

    #[test]
    fn write_created_works_for_2_digits() {
        assert_eq!(&*render_created(Duration::new(12, 456000000)), b"12.456");
    }

    #[test]
    fn write_created_works_for_3_digits() {
        assert_eq!(&*render_created(Duration::new(123, 456000000)), b"123.456");
    }

    #[test]
    fn write_created_works_for_4_digits() {
        assert_eq!(
            &*render_created(Duration::new(1234, 456000000)),
            b"1234.456"
        );
    }

    #[test]
    fn write_created_works_for_5_digits() {
        assert_eq!(
            &*render_created(Duration::new(12345, 456000000)),
            b"12345.456"
        );
    }

    #[test]
    fn write_created_works_for_6_digits() {
        assert_eq!(
            &*render_created(Duration::new(123456, 456000000)),
            b"123456.456"
        );
    }

    #[test]
    fn write_created_works_for_7_digits() {
        assert_eq!(
            &*render_created(Duration::new(1234567, 456000000)),
            b"1234567.456"
        );
    }

    #[test]
    fn write_created_works_for_8_digits() {
        assert_eq!(
            &*render_created(Duration::new(12345678, 456000000)),
            b"12345678.456"
        );
    }

    #[test]
    fn write_created_works_for_9_digits() {
        assert_eq!(
            &*render_created(Duration::new(123456789, 456000000)),
            b"123456789.456"
        );
    }

    #[test]
    fn write_created_works_for_10_digits() {
        assert_eq!(
            &*render_created(Duration::new(1234567890, 456000000)),
            b"1234567890.456"
        );
    }

    #[test]
    fn write_created_works_for_11_digits() {
        assert_eq!(
            &*render_created(Duration::new(12345678901, 456000000)),
            b"12345678901.456"
        );
    }

    #[test]
    fn write_created_works_for_12_digits() {
        assert_eq!(
            &*render_created(Duration::new(123456789012, 456000000)),
            b"123456789012.456"
        );
    }

    #[test]
    fn write_created_works_for_13_digits() {
        assert_eq!(
            &*render_created(Duration::new(1234567890123, 456000000)),
            b"1234567890123.456"
        );
    }

    #[test]
    fn write_created_works_for_14_digits() {
        assert_eq!(
            &*render_created(Duration::new(12345678901234, 456000000)),
            b"12345678901234.456"
        );
    }

    #[test]
    fn write_created_works_for_15_digits() {
        assert_eq!(
            &*render_created(Duration::new(123456789012345, 456000000)),
            b"123456789012345.456"
        );
    }

    #[test]
    fn write_created_works_for_16_digits() {
        assert_eq!(
            &*render_created(Duration::new(1234567890123456, 456000000)),
            b"1234567890123456.456"
        );
    }

    #[test]
    fn write_created_works_for_17_digits() {
        assert_eq!(
            &*render_created(Duration::new(12345678901234567, 456000000)),
            b"12345678901234567.456"
        );
    }

    #[test]
    fn write_created_works_for_18_digits() {
        assert_eq!(
            &*render_created(Duration::new(123456789012345678, 456000000)),
            b"123456789012345678.456"
        );
    }

    #[test]
    fn write_created_works_for_19_digits() {
        assert_eq!(
            &*render_created(Duration::new(1234567890123456789, 456000000)),
            b"1234567890123456789.456"
        );
    }

    #[test]
    fn write_created_works_for_all_millisecond_digits() {
        for (d1, c1) in (0..=9).zip(b'0'..=b'9') {
            for (d2, c2) in (0..=9).zip(b'0'..=b'9') {
                for (d3, c3) in (0..=9).zip(b'0'..=b'9') {
                    let millis = d1 * 100 + d2 * 10 + d3;
                    assert_eq!(
                        &*render_created(Duration::new(123, millis * 1000000)),
                        &[b'1', b'2', b'3', b'.', c1, c2, c3]
                    );
                }
            }
        }
    }

    //  #####  ###### #    # #####  ###### #####
    //  #    # #      ##   # #    # #      #    #
    //  #    # #####  # #  # #    # #####  #    #
    //  #####  #      #  # # #    # #      #####
    //  #   #  #      #   ## #    # #      #   #
    //  #    # ###### #    # #####  ###### #    #

    fn render_with_created(snapshot: PromSnapshot, created: Duration) -> Box<[u8]> {
        render_openapi_metrics(PromWriteContext {
            environment: &PromEnvironment {
                created: SystemTime::UNIX_EPOCH + created,
            },
            snapshot: &snapshot,
            table: get_user_group_table(),
        })
    }

    fn render(snapshot: PromSnapshot) -> Box<[u8]> {
        render_with_created(snapshot, Duration::from_millis(123456))
    }

    #[test]
    fn renders_all_empty() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_with_even_second_created() {
        let actual = render_with_created(
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot { data: Box::new([]) },
            },
            Duration::from_secs(123),
        );
        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.000
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.000
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.000
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.000
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.000
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.000
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.000
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.000
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.000
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.000
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    // Test just in case of a horribly busted system time. Main concern is it doesn't crash, but
    // I'm also testing output for consistency.
    #[test]
    fn renders_with_zero_seconds_created() {
        let actual = render_with_created(
            PromSnapshot {
                entries_ingested: 0,
                fields_ingested: 0,
                data_bytes_ingested: 0,
                faults: 0,
                cursor_double_retries: 0,
                unreadable_fields: 0,
                corrupted_fields: 0,
                requests: 0,
                messages_ingested: ByteCountSnapshot { data: Box::new([]) },
            },
            Duration::from_millis(123),
        );
        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xC4\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 0.123
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 0.123
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 0.123
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 0.123
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 0.123
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 0.123
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 0.123
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 0.123
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 0.123
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 0.123
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_1_digit_entries_ingested() {
        let actual = render(PromSnapshot {
            entries_ingested: 1,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xD8\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 1
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_max_entries_ingested() {
        let actual = render(PromSnapshot {
            entries_ingested: u64::MAX,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xEB\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 18446744073709551615
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_max_fields_ingested() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: u64::MAX,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xEB\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 18446744073709551615
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_max_data_bytes_ingested() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: u64::MAX,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xEB\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 18446744073709551615
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_max_faults() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: u64::MAX,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xEB\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 18446744073709551615
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_max_cursor_double_retries() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: u64::MAX,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xEB\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 18446744073709551615
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_max_unreadable_entries() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: u64::MAX,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xEB\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 18446744073709551615
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_max_corrupted_entries() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: u64::MAX,
            requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xEB\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 18446744073709551615
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_max_requests() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: u64::MAX,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xEB\x04\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 18446744073709551615
# TYPE journald_messages_ingested counter
journald_messages_ingested_created 123.456
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created 123.456
journald_messages_ingested_bytes_total 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_a_single_empty_message_key_ingested() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: Box::new([ByteCountSnapshotEntry {
                    key: MessageKey::build(
                        Some(123),
                        Some(123),
                        Some(b"foo"),
                        Priority::Informational,
                    ),
                    lines: 1,
                    bytes: 0,
                }]),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\x10\x06\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 0
# EOF
"
            )
        );
    }

    #[test]
    fn renders_a_single_small_message_key_ingested() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: Box::new([ByteCountSnapshotEntry {
                    key: MessageKey::build(
                        Some(123),
                        Some(123),
                        Some(b"foo"),
                        Priority::Informational,
                    ),
                    lines: 1,
                    bytes: 5,
                }]),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\x10\x06\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# EOF
"
            )
        );
    }

    #[test]
    fn renders_a_single_max_len_message_key_ingested() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: Box::new([ByteCountSnapshotEntry {
                    key: MessageKey::build(
                        Some(123),
                        Some(123),
                        Some(b"foo"),
                        Priority::Informational,
                    ),
                    lines: 1,
                    bytes: u64::MAX,
                }]),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\x23\x06\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 18446744073709551615
# EOF
"
            )
        );
    }

    #[test]
    fn renders_a_single_max_lines_message_key_ingested() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: Box::new([ByteCountSnapshotEntry {
                    key: MessageKey::build(
                        Some(123),
                        Some(123),
                        Some(b"foo"),
                        Priority::Informational,
                    ),
                    lines: u64::MAX,
                    bytes: 5,
                }]),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\x23\x06\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 18446744073709551615
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# EOF
"
            )
        );
    }

    #[test]
    fn renders_two_messages_across_two_services() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: Box::new([
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(456),
                            Some(b"foo"),
                            Priority::Informational,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(456),
                            Some(123),
                            Some(b"bar"),
                            Priority::Warning,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                ]),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xFC\x07\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 1
journald_messages_ingested_created{service=\"bar\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"bar\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_bar\"} 5
journald_messages_ingested_bytes_created{service=\"bar\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"bar\",priority=\"WARNING\",severity=\"4\",user=\"user_bar\",group=\"group_foo\"} 5
# EOF
"
            )
        );
    }

    #[test]
    fn renders_1_fault_and_1_message() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 1,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: Box::new([ByteCountSnapshotEntry {
                    key: MessageKey::build(
                        Some(123),
                        Some(123),
                        Some(b"foo"),
                        Priority::Informational,
                    ),
                    lines: 1,
                    bytes: 5,
                }]),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\x10\x06\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 1
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
# EOF
"
            )
        );
    }

    #[test]
    fn renders_multiple_priority_levels_within_same_service() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: Box::new([
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Emergency,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Alert),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Critical,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Error),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Warning,
                        ),
                        lines: 2,
                        bytes: 10,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Notice,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Informational,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Debug),
                        lines: 2,
                        bytes: 10,
                    },
                ]),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\x4E\x13\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 2
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 10
# EOF
"
            )
        );
    }

    #[test]
    fn renders_5_faults_and_multiple_priority_levels_within_same_service() {
        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 5,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: Box::new([
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Emergency,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Alert),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Critical,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Error),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Warning,
                        ),
                        lines: 2,
                        bytes: 10,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Notice,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(
                            Some(123),
                            Some(123),
                            Some(b"foo"),
                            Priority::Informational,
                        ),
                        lines: 1,
                        bytes: 5,
                    },
                    ByteCountSnapshotEntry {
                        key: MessageKey::build(Some(123), Some(123), Some(b"foo"), Priority::Debug),
                        lines: 2,
                        bytes: 10,
                    },
                ]),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\x4E\x13\x00\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 5
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 1
journald_messages_ingested_created{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 2
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 5
journald_messages_ingested_bytes_created{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"foo\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 10
# EOF
"
            )
        );
    }

    #[test]
    fn renders_500_faults_and_400_different_service_messages() {
        let service_names: [&[u8]; 20] = [
            b"service1",
            b"service2",
            b"service3",
            b"service4",
            b"service5",
            b"service6",
            b"service7",
            b"service8",
            b"service9",
            b"service10",
            b"service11",
            b"service12",
            b"service13",
            b"service14",
            b"service15",
            b"service16",
            b"service17",
            b"service18",
            b"service19",
            b"service20",
        ];

        let mut messages_ingested = vec![];

        for service in service_names {
            let ingested_message_data_params = [
                (Priority::Emergency, 2, 10),
                (Priority::Alert, 2, 10),
                (Priority::Critical, 2, 10),
                (Priority::Error, 2, 10),
                (Priority::Warning, 4, 20),
                (Priority::Notice, 2, 10),
                (Priority::Informational, 2, 10),
                (Priority::Debug, 4, 20),
            ];

            for (priority, lines, bytes) in ingested_message_data_params {
                messages_ingested.push(ByteCountSnapshotEntry {
                    key: MessageKey::build(Some(123), Some(123), Some(service), priority),
                    lines,
                    bytes,
                });
            }
        }

        let actual = render(PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_bytes_ingested: 0,
            faults: 500,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            requests: 0,
            messages_ingested: ByteCountSnapshot {
                data: messages_ingested.into(),
            },
        });

        assert_eq!(
            BinaryToDebug(&actual),
            BinaryToDebug(
                b"\x00\xE2\x40\x01\x00# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_bytes_ingested counter
journald_data_bytes_ingested_created 123.456
journald_data_bytes_ingested_total 0
# TYPE journald_faults counter
journald_faults_created 123.456
journald_faults_total 500
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created 123.456
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created 123.456
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created 123.456
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created 123.456
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created{service=\"service1\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service1\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service1\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service1\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service2\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service2\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service2\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service2\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service3\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service3\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service3\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service3\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service4\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service4\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service4\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service4\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service5\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service5\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service5\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service5\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service6\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service6\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service6\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service6\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service7\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service7\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service7\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service7\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service8\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service8\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service8\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service8\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service9\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service9\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service9\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service9\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service10\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service10\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service10\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service10\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service11\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service11\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service11\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service11\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service12\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service12\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service12\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service12\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service13\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service13\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service13\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service13\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service14\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service14\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service14\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service14\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service15\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service15\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service15\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service15\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service16\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service16\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service16\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service16\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service17\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service17\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service17\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service17\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service18\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service18\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service18\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service18\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service19\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service19\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service19\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service19\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service20\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 4
journald_messages_ingested_created{service=\"service20\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 2
journald_messages_ingested_created{service=\"service20\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_total{service=\"service20\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 4
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service1\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service1\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service2\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service2\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service3\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service3\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service4\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service4\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service5\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service5\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service6\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service6\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service7\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service7\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service8\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service8\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service9\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service9\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service10\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service10\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service11\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service11\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service12\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service12\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service13\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service13\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service14\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service14\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service15\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service15\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service16\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service16\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service17\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service17\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service18\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service18\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service19\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service19\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"EMERG\",severity=\"0\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"ALERT\",severity=\"1\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"CRIT\",severity=\"2\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"ERR\",severity=\"3\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"WARNING\",severity=\"4\",user=\"user_foo\",group=\"group_foo\"} 20
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"NOTICE\",severity=\"5\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"INFO\",severity=\"6\",user=\"user_foo\",group=\"group_foo\"} 10
journald_messages_ingested_bytes_created{service=\"service20\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 123.456
journald_messages_ingested_bytes_total{service=\"service20\",priority=\"DEBUG\",severity=\"7\",user=\"user_foo\",group=\"group_foo\"} 20
# EOF
"
            )
        );
    }
}
