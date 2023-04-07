use crate::prelude::*;

use super::*;

fn render_with_created(snapshot: &PromSnapshot, seconds: u64, millis: u32) -> Box<[u8]> {
    render_openapi_metrics(
        &PromEnvironment::new(mock_system_time(seconds, millis)),
        snapshot,
        &get_user_group_table(),
    )
}

struct Template {
    data_len: usize,
    parts: &'static [&'static [u8]],
}

impl Template {
    const fn new(parts: &'static [&'static [u8]]) -> Self {
        let mut data_len = 0;
        let mut i = 0;

        while i < parts.len() {
            data_len += parts[i].len();
            i += 1;
        }

        Self { data_len, parts }
    }
}

// Get this noise out. Also gets tedious editing the length every time I want to add an entry or
// modify an existing one.
fn assert_snapshot_eq(actual: Box<[u8]>, template: &'static Template, joiner: &[u8]) {
    // This is optimized somewhat to try to speed up Miri in one of the slowest parts. Won't make
    // literally any difference for the standard `cargo test`.

    let expected_len = (template.parts.len() - 1) * joiner.len() + template.data_len;
    let mut real_expected: Box<[u8]> = vec![0; expected_len + 5].into();

    copy_to_start(
        &mut real_expected[1..],
        &truncate_usize_u32(expected_len).to_le_bytes(),
    );

    let mut target = 5;
    let mut has_previous = false;

    for part in template.parts {
        if has_previous {
            copy_to_start(&mut real_expected[target..], joiner);
            target += joiner.len();
        }
        copy_to_start(&mut real_expected[target..], part);
        target += part.len();
        has_previous = true;
    }

    assert_eq!(BinaryToDebug(&actual), BinaryToDebug(&real_expected));
}

//   ####   ####  #    # #    # ##### ###### #####     #    #   ##   #      #    # ######  ####
//  #    # #    # #    # ##   #   #   #      #    #    #    #  #  #  #      #    # #      #
//  #      #    # #    # # #  #   #   #####  #    #    #    # #    # #      #    # #####   ####
//  #      #    # #    # #  # #   #   #      #####     #    # ###### #      #    # #           #
//  #    # #    # #    # #   ##   #   #      #   #      #  #  #    # #      #    # #      #    #
//   ####   ####   ####  #    #   #   ###### #    #      ##   #    # ######  ####  ######  ####

fn test_write_counter_value(value: u64, expected: &[u8]) {
    static EXPECTED_PARTS: Template = Template::new(&[
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created 123.456
journald_entries_ingested_total ",
        b"
# TYPE journald_fields_ingested counter
journald_fields_ingested_created 123.456
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created 123.456
journald_data_ingested_bytes_total 0
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
",
    ]);

    let actual = render_with_created(
        &PromSnapshot {
            entries_ingested: value,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        },
        123,
        456,
    );

    assert_snapshot_eq(actual, &EXPECTED_PARTS, expected);
}

#[test]
fn counter_value_written_correctly_for_zero() {
    test_write_counter_value(0, b"0");
}

#[test]
fn counter_value_written_correctly_for_positive_single_digits() {
    test_write_counter_value(1, b"1");
    test_write_counter_value(2, b"2");
    test_write_counter_value(3, b"3");
    test_write_counter_value(4, b"4");
    test_write_counter_value(5, b"5");
    test_write_counter_value(6, b"6");
    test_write_counter_value(7, b"7");
    test_write_counter_value(8, b"8");
    test_write_counter_value(9, b"9");
}

#[test]
fn counter_value_written_correctly_for_2_digits() {
    test_write_counter_value(12, b"12");
}

#[test]
fn counter_value_written_correctly_for_3_digits() {
    test_write_counter_value(123, b"123");
}

#[test]
fn counter_value_written_correctly_for_4_digits() {
    test_write_counter_value(1234, b"1234");
}

#[test]
fn counter_value_written_correctly_for_5_digits() {
    test_write_counter_value(12345, b"12345");
}

#[test]
fn counter_value_written_correctly_for_6_digits() {
    test_write_counter_value(123456, b"123456");
}

#[test]
fn counter_value_written_correctly_for_7_digits() {
    test_write_counter_value(1234567, b"1234567");
}

#[test]
fn counter_value_written_correctly_for_8_digits() {
    test_write_counter_value(12345678, b"12345678");
}

#[test]
fn counter_value_written_correctly_for_9_digits() {
    test_write_counter_value(123456789, b"123456789");
}

#[test]
fn counter_value_written_correctly_for_10_digits() {
    test_write_counter_value(1234567890, b"1234567890");
}

#[test]
fn counter_value_written_correctly_for_11_digits() {
    test_write_counter_value(12345678901, b"12345678901");
}

#[test]
fn counter_value_written_correctly_for_12_digits() {
    test_write_counter_value(123456789012, b"123456789012");
}

#[test]
fn counter_value_written_correctly_for_13_digits() {
    test_write_counter_value(1234567890123, b"1234567890123");
}

#[test]
fn counter_value_written_correctly_for_14_digits() {
    test_write_counter_value(12345678901234, b"12345678901234");
}

#[test]
fn counter_value_written_correctly_for_15_digits() {
    test_write_counter_value(123456789012345, b"123456789012345");
}

#[test]
fn counter_value_written_correctly_for_16_digits() {
    test_write_counter_value(1234567890123456, b"1234567890123456");
}

#[test]
fn counter_value_written_correctly_for_17_digits() {
    test_write_counter_value(12345678901234567, b"12345678901234567");
}

#[test]
fn counter_value_written_correctly_for_18_digits() {
    test_write_counter_value(123456789012345678, b"123456789012345678");
}

#[test]
fn counter_value_written_correctly_for_19_digits() {
    test_write_counter_value(1234567890123456789, b"1234567890123456789");
}

#[test]
fn counter_value_written_correctly_for_20_digits() {
    test_write_counter_value(12345678901234567890, b"12345678901234567890");
}

//   ####  #####  ######   ##   ##### ###### #####     #####  #   # ##### ######  ####
//  #    # #    # #       #  #    #   #      #    #    #    #  # #    #   #      #
//  #      #    # #####  #    #   #   #####  #    #    #####    #     #   #####   ####
//  #      #####  #      ######   #   #      #    #    #    #   #     #   #           #
//  #    # #   #  #      #    #   #   #      #    #    #    #   #     #   #      #    #
//   ####  #    # ###### #    #   #   ###### #####     #####    #     #   ######  ####

#[track_caller]
fn test_write_created_value(secs: u64, millis: u32, created: &[u8]) {
    static EXPECTED_PARTS: Template = Template::new(&[
        b"# TYPE journald_entries_ingested counter
journald_entries_ingested_created ",
        b"
journald_entries_ingested_total 0
# TYPE journald_fields_ingested counter
journald_fields_ingested_created ",
        b"
journald_fields_ingested_total 0
# TYPE journald_data_ingested_bytes counter
# UNIT journald_data_ingested_bytes bytes
journald_data_ingested_bytes_created ",
        b"
journald_data_ingested_bytes_total 0
# TYPE journald_faults counter
journald_faults_created ",
        b"
journald_faults_total 0
# TYPE journald_cursor_double_retries counter
journald_cursor_double_retries_created ",
        b"
journald_cursor_double_retries_total 0
# TYPE journald_unreadable_fields counter
journald_unreadable_fields_created ",
        b"
journald_unreadable_fields_total 0
# TYPE journald_corrupted_fields counter
journald_corrupted_fields_created ",
        b"
journald_corrupted_fields_total 0
# TYPE journald_metrics_requests counter
journald_metrics_requests_created ",
        b"
journald_metrics_requests_total 0
# TYPE journald_messages_ingested counter
journald_messages_ingested_created ",
        b"
journald_messages_ingested_total 0
# TYPE journald_messages_ingested_bytes counter
# UNIT journald_messages_ingested_bytes bytes
journald_messages_ingested_bytes_created ",
        b"
journald_messages_ingested_bytes_total 0
# EOF
",
    ]);

    let actual = render_with_created(
        &PromSnapshot {
            entries_ingested: 0,
            fields_ingested: 0,
            data_ingested_bytes: 0,
            faults: 0,
            cursor_double_retries: 0,
            unreadable_fields: 0,
            corrupted_fields: 0,
            metrics_requests: 0,
            messages_ingested: ByteCountSnapshot { data: Box::new([]) },
        },
        secs,
        millis,
    );

    assert_snapshot_eq(actual, &EXPECTED_PARTS, created);
}

#[test]
fn created_value_written_correctly_for_zero() {
    test_write_created_value(0, 456, b"0.456");
}

// Test just in case of a horribly busted system time. Main concern is it doesn't crash, but I'm
// also testing output for consistency.
#[test]
fn created_value_written_correctly_for_seconds_0_to_9() {
    test_write_created_value(1, 456, b"1.456");
    test_write_created_value(2, 456, b"2.456");
    test_write_created_value(3, 456, b"3.456");
    test_write_created_value(4, 456, b"4.456");
    test_write_created_value(5, 456, b"5.456");
    test_write_created_value(6, 456, b"6.456");
    test_write_created_value(7, 456, b"7.456");
    test_write_created_value(8, 456, b"8.456");
    test_write_created_value(9, 456, b"9.456");
}

// Test 2 digit seconds exhaustively as well, just for completeness

#[test]
fn created_value_written_correctly_for_seconds_10_to_19() {
    test_write_created_value(10, 456, b"10.456");
    test_write_created_value(11, 456, b"11.456");
    test_write_created_value(12, 456, b"12.456");
    test_write_created_value(13, 456, b"13.456");
    test_write_created_value(14, 456, b"14.456");
    test_write_created_value(15, 456, b"15.456");
    test_write_created_value(16, 456, b"16.456");
    test_write_created_value(17, 456, b"17.456");
    test_write_created_value(18, 456, b"18.456");
    test_write_created_value(19, 456, b"19.456");
}

#[test]
fn created_value_written_correctly_for_seconds_20_to_29() {
    test_write_created_value(20, 456, b"20.456");
    test_write_created_value(21, 456, b"21.456");
    test_write_created_value(22, 456, b"22.456");
    test_write_created_value(23, 456, b"23.456");
    test_write_created_value(24, 456, b"24.456");
    test_write_created_value(25, 456, b"25.456");
    test_write_created_value(26, 456, b"26.456");
    test_write_created_value(27, 456, b"27.456");
    test_write_created_value(28, 456, b"28.456");
    test_write_created_value(29, 456, b"29.456");
}

#[test]
fn created_value_written_correctly_for_seconds_30_to_39() {
    test_write_created_value(30, 456, b"30.456");
    test_write_created_value(31, 456, b"31.456");
    test_write_created_value(32, 456, b"32.456");
    test_write_created_value(33, 456, b"33.456");
    test_write_created_value(34, 456, b"34.456");
    test_write_created_value(35, 456, b"35.456");
    test_write_created_value(36, 456, b"36.456");
    test_write_created_value(37, 456, b"37.456");
    test_write_created_value(38, 456, b"38.456");
    test_write_created_value(39, 456, b"39.456");
}

#[test]
fn created_value_written_correctly_for_seconds_40_to_49() {
    test_write_created_value(40, 456, b"40.456");
    test_write_created_value(41, 456, b"41.456");
    test_write_created_value(42, 456, b"42.456");
    test_write_created_value(43, 456, b"43.456");
    test_write_created_value(44, 456, b"44.456");
    test_write_created_value(45, 456, b"45.456");
    test_write_created_value(46, 456, b"46.456");
    test_write_created_value(47, 456, b"47.456");
    test_write_created_value(48, 456, b"48.456");
    test_write_created_value(49, 456, b"49.456");
}

#[test]
fn created_value_written_correctly_for_seconds_50_to_59() {
    test_write_created_value(50, 456, b"50.456");
    test_write_created_value(51, 456, b"51.456");
    test_write_created_value(52, 456, b"52.456");
    test_write_created_value(53, 456, b"53.456");
    test_write_created_value(54, 456, b"54.456");
    test_write_created_value(55, 456, b"55.456");
    test_write_created_value(56, 456, b"56.456");
    test_write_created_value(57, 456, b"57.456");
    test_write_created_value(58, 456, b"58.456");
    test_write_created_value(59, 456, b"59.456");
}

#[test]
fn created_value_written_correctly_for_seconds_60_to_69() {
    test_write_created_value(60, 456, b"60.456");
    test_write_created_value(61, 456, b"61.456");
    test_write_created_value(62, 456, b"62.456");
    test_write_created_value(63, 456, b"63.456");
    test_write_created_value(64, 456, b"64.456");
    test_write_created_value(65, 456, b"65.456");
    test_write_created_value(66, 456, b"66.456");
    test_write_created_value(67, 456, b"67.456");
    test_write_created_value(68, 456, b"68.456");
    test_write_created_value(69, 456, b"69.456");
}

#[test]
fn created_value_written_correctly_for_seconds_70_to_79() {
    test_write_created_value(70, 456, b"70.456");
    test_write_created_value(71, 456, b"71.456");
    test_write_created_value(72, 456, b"72.456");
    test_write_created_value(73, 456, b"73.456");
    test_write_created_value(74, 456, b"74.456");
    test_write_created_value(75, 456, b"75.456");
    test_write_created_value(76, 456, b"76.456");
    test_write_created_value(77, 456, b"77.456");
    test_write_created_value(78, 456, b"78.456");
    test_write_created_value(79, 456, b"79.456");
}

#[test]
fn created_value_written_correctly_for_seconds_80_to_89() {
    test_write_created_value(80, 456, b"80.456");
    test_write_created_value(81, 456, b"81.456");
    test_write_created_value(82, 456, b"82.456");
    test_write_created_value(83, 456, b"83.456");
    test_write_created_value(84, 456, b"84.456");
    test_write_created_value(85, 456, b"85.456");
    test_write_created_value(86, 456, b"86.456");
    test_write_created_value(87, 456, b"87.456");
    test_write_created_value(88, 456, b"88.456");
    test_write_created_value(89, 456, b"89.456");
}

#[test]
fn created_value_written_correctly_for_seconds_90_to_99() {
    test_write_created_value(90, 456, b"90.456");
    test_write_created_value(91, 456, b"91.456");
    test_write_created_value(92, 456, b"92.456");
    test_write_created_value(93, 456, b"93.456");
    test_write_created_value(94, 456, b"94.456");
    test_write_created_value(95, 456, b"95.456");
    test_write_created_value(96, 456, b"96.456");
    test_write_created_value(97, 456, b"97.456");
    test_write_created_value(98, 456, b"98.456");
    test_write_created_value(99, 456, b"99.456");
}

// More digits aren't tested exhaustively, since it's just impractical.

#[test]
fn created_value_written_correctly_for_3_digits() {
    test_write_created_value(123, 456, b"123.456");
}

#[test]
fn created_value_written_correctly_for_4_digits() {
    test_write_created_value(1234, 456, b"1234.456");
}

#[test]
fn created_value_written_correctly_for_5_digits() {
    test_write_created_value(12345, 456, b"12345.456");
}

#[test]
fn created_value_written_correctly_for_6_digits() {
    test_write_created_value(123456, 456, b"123456.456");
}

#[test]
fn created_value_written_correctly_for_7_digits() {
    test_write_created_value(1234567, 456, b"1234567.456");
}

#[test]
fn created_value_written_correctly_for_8_digits() {
    test_write_created_value(12345678, 456, b"12345678.456");
}

#[test]
fn created_value_written_correctly_for_9_digits() {
    test_write_created_value(123456789, 456, b"123456789.456");
}

#[test]
fn created_value_written_correctly_for_10_digits() {
    test_write_created_value(1234567890, 456, b"1234567890.456");
}

#[test]
fn created_value_written_correctly_for_11_digits() {
    test_write_created_value(12345678901, 456, b"12345678901.456");
}

#[test]
fn created_value_written_correctly_for_12_digits() {
    test_write_created_value(123456789012, 456, b"123456789012.456");
}

#[test]
fn created_value_written_correctly_for_13_digits() {
    test_write_created_value(1234567890123, 456, b"1234567890123.456");
}

#[test]
fn created_value_written_correctly_for_14_digits() {
    test_write_created_value(12345678901234, 456, b"12345678901234.456");
}

#[test]
fn created_value_written_correctly_for_15_digits() {
    test_write_created_value(123456789012345, 456, b"123456789012345.456");
}

#[test]
fn created_value_written_correctly_for_16_digits() {
    test_write_created_value(1234567890123456, 456, b"1234567890123456.456");
}

#[test]
fn created_value_written_correctly_for_17_digits() {
    test_write_created_value(12345678901234567, 456, b"12345678901234567.456");
}

#[test]
fn created_value_written_correctly_for_18_digits() {
    test_write_created_value(123456789012345678, 456, b"123456789012345678.456");
}

#[test]
fn created_value_written_correctly_for_19_digits() {
    test_write_created_value(1234567890123456789, 456, b"1234567890123456789.456");
}

// The milliseconds are checked exhaustively since it's only 1K possibilities.

#[test]
fn created_value_written_correctly_for_millisecond_000_to_009() {
    test_write_created_value(123, 0, b"123.000");
    test_write_created_value(123, 1, b"123.001");
    test_write_created_value(123, 2, b"123.002");
    test_write_created_value(123, 3, b"123.003");
    test_write_created_value(123, 4, b"123.004");
    test_write_created_value(123, 5, b"123.005");
    test_write_created_value(123, 6, b"123.006");
    test_write_created_value(123, 7, b"123.007");
    test_write_created_value(123, 8, b"123.008");
    test_write_created_value(123, 9, b"123.009");
}

#[test]
fn created_value_written_correctly_for_millisecond_010_to_019() {
    test_write_created_value(123, 10, b"123.010");
    test_write_created_value(123, 11, b"123.011");
    test_write_created_value(123, 12, b"123.012");
    test_write_created_value(123, 13, b"123.013");
    test_write_created_value(123, 14, b"123.014");
    test_write_created_value(123, 15, b"123.015");
    test_write_created_value(123, 16, b"123.016");
    test_write_created_value(123, 17, b"123.017");
    test_write_created_value(123, 18, b"123.018");
    test_write_created_value(123, 19, b"123.019");
}

#[test]
fn created_value_written_correctly_for_millisecond_020_to_029() {
    test_write_created_value(123, 20, b"123.020");
    test_write_created_value(123, 21, b"123.021");
    test_write_created_value(123, 22, b"123.022");
    test_write_created_value(123, 23, b"123.023");
    test_write_created_value(123, 24, b"123.024");
    test_write_created_value(123, 25, b"123.025");
    test_write_created_value(123, 26, b"123.026");
    test_write_created_value(123, 27, b"123.027");
    test_write_created_value(123, 28, b"123.028");
    test_write_created_value(123, 29, b"123.029");
}

#[test]
fn created_value_written_correctly_for_millisecond_030_to_039() {
    test_write_created_value(123, 30, b"123.030");
    test_write_created_value(123, 31, b"123.031");
    test_write_created_value(123, 32, b"123.032");
    test_write_created_value(123, 33, b"123.033");
    test_write_created_value(123, 34, b"123.034");
    test_write_created_value(123, 35, b"123.035");
    test_write_created_value(123, 36, b"123.036");
    test_write_created_value(123, 37, b"123.037");
    test_write_created_value(123, 38, b"123.038");
    test_write_created_value(123, 39, b"123.039");
}

#[test]
fn created_value_written_correctly_for_millisecond_040_to_049() {
    test_write_created_value(123, 40, b"123.040");
    test_write_created_value(123, 41, b"123.041");
    test_write_created_value(123, 42, b"123.042");
    test_write_created_value(123, 43, b"123.043");
    test_write_created_value(123, 44, b"123.044");
    test_write_created_value(123, 45, b"123.045");
    test_write_created_value(123, 46, b"123.046");
    test_write_created_value(123, 47, b"123.047");
    test_write_created_value(123, 48, b"123.048");
    test_write_created_value(123, 49, b"123.049");
}

#[test]
fn created_value_written_correctly_for_millisecond_050_to_059() {
    test_write_created_value(123, 50, b"123.050");
    test_write_created_value(123, 51, b"123.051");
    test_write_created_value(123, 52, b"123.052");
    test_write_created_value(123, 53, b"123.053");
    test_write_created_value(123, 54, b"123.054");
    test_write_created_value(123, 55, b"123.055");
    test_write_created_value(123, 56, b"123.056");
    test_write_created_value(123, 57, b"123.057");
    test_write_created_value(123, 58, b"123.058");
    test_write_created_value(123, 59, b"123.059");
}

#[test]
fn created_value_written_correctly_for_millisecond_060_to_069() {
    test_write_created_value(123, 60, b"123.060");
    test_write_created_value(123, 61, b"123.061");
    test_write_created_value(123, 62, b"123.062");
    test_write_created_value(123, 63, b"123.063");
    test_write_created_value(123, 64, b"123.064");
    test_write_created_value(123, 65, b"123.065");
    test_write_created_value(123, 66, b"123.066");
    test_write_created_value(123, 67, b"123.067");
    test_write_created_value(123, 68, b"123.068");
    test_write_created_value(123, 69, b"123.069");
}

#[test]
fn created_value_written_correctly_for_millisecond_070_to_079() {
    test_write_created_value(123, 70, b"123.070");
    test_write_created_value(123, 71, b"123.071");
    test_write_created_value(123, 72, b"123.072");
    test_write_created_value(123, 73, b"123.073");
    test_write_created_value(123, 74, b"123.074");
    test_write_created_value(123, 75, b"123.075");
    test_write_created_value(123, 76, b"123.076");
    test_write_created_value(123, 77, b"123.077");
    test_write_created_value(123, 78, b"123.078");
    test_write_created_value(123, 79, b"123.079");
}

#[test]
fn created_value_written_correctly_for_millisecond_080_to_089() {
    test_write_created_value(123, 80, b"123.080");
    test_write_created_value(123, 81, b"123.081");
    test_write_created_value(123, 82, b"123.082");
    test_write_created_value(123, 83, b"123.083");
    test_write_created_value(123, 84, b"123.084");
    test_write_created_value(123, 85, b"123.085");
    test_write_created_value(123, 86, b"123.086");
    test_write_created_value(123, 87, b"123.087");
    test_write_created_value(123, 88, b"123.088");
    test_write_created_value(123, 89, b"123.089");
}

#[test]
fn created_value_written_correctly_for_millisecond_090_to_099() {
    test_write_created_value(123, 90, b"123.090");
    test_write_created_value(123, 91, b"123.091");
    test_write_created_value(123, 92, b"123.092");
    test_write_created_value(123, 93, b"123.093");
    test_write_created_value(123, 94, b"123.094");
    test_write_created_value(123, 95, b"123.095");
    test_write_created_value(123, 96, b"123.096");
    test_write_created_value(123, 97, b"123.097");
    test_write_created_value(123, 98, b"123.098");
    test_write_created_value(123, 99, b"123.099");
}

#[test]
fn created_value_written_correctly_for_millisecond_100_to_109() {
    test_write_created_value(123, 100, b"123.100");
    test_write_created_value(123, 101, b"123.101");
    test_write_created_value(123, 102, b"123.102");
    test_write_created_value(123, 103, b"123.103");
    test_write_created_value(123, 104, b"123.104");
    test_write_created_value(123, 105, b"123.105");
    test_write_created_value(123, 106, b"123.106");
    test_write_created_value(123, 107, b"123.107");
    test_write_created_value(123, 108, b"123.108");
    test_write_created_value(123, 109, b"123.109");
}

#[test]
fn created_value_written_correctly_for_millisecond_110_to_119() {
    test_write_created_value(123, 110, b"123.110");
    test_write_created_value(123, 111, b"123.111");
    test_write_created_value(123, 112, b"123.112");
    test_write_created_value(123, 113, b"123.113");
    test_write_created_value(123, 114, b"123.114");
    test_write_created_value(123, 115, b"123.115");
    test_write_created_value(123, 116, b"123.116");
    test_write_created_value(123, 117, b"123.117");
    test_write_created_value(123, 118, b"123.118");
    test_write_created_value(123, 119, b"123.119");
}

#[test]
fn created_value_written_correctly_for_millisecond_120_to_129() {
    test_write_created_value(123, 120, b"123.120");
    test_write_created_value(123, 121, b"123.121");
    test_write_created_value(123, 122, b"123.122");
    test_write_created_value(123, 123, b"123.123");
    test_write_created_value(123, 124, b"123.124");
    test_write_created_value(123, 125, b"123.125");
    test_write_created_value(123, 126, b"123.126");
    test_write_created_value(123, 127, b"123.127");
    test_write_created_value(123, 128, b"123.128");
    test_write_created_value(123, 129, b"123.129");
}

#[test]
fn created_value_written_correctly_for_millisecond_130_to_139() {
    test_write_created_value(123, 130, b"123.130");
    test_write_created_value(123, 131, b"123.131");
    test_write_created_value(123, 132, b"123.132");
    test_write_created_value(123, 133, b"123.133");
    test_write_created_value(123, 134, b"123.134");
    test_write_created_value(123, 135, b"123.135");
    test_write_created_value(123, 136, b"123.136");
    test_write_created_value(123, 137, b"123.137");
    test_write_created_value(123, 138, b"123.138");
    test_write_created_value(123, 139, b"123.139");
}

#[test]
fn created_value_written_correctly_for_millisecond_140_to_149() {
    test_write_created_value(123, 140, b"123.140");
    test_write_created_value(123, 141, b"123.141");
    test_write_created_value(123, 142, b"123.142");
    test_write_created_value(123, 143, b"123.143");
    test_write_created_value(123, 144, b"123.144");
    test_write_created_value(123, 145, b"123.145");
    test_write_created_value(123, 146, b"123.146");
    test_write_created_value(123, 147, b"123.147");
    test_write_created_value(123, 148, b"123.148");
    test_write_created_value(123, 149, b"123.149");
}

#[test]
fn created_value_written_correctly_for_millisecond_150_to_159() {
    test_write_created_value(123, 150, b"123.150");
    test_write_created_value(123, 151, b"123.151");
    test_write_created_value(123, 152, b"123.152");
    test_write_created_value(123, 153, b"123.153");
    test_write_created_value(123, 154, b"123.154");
    test_write_created_value(123, 155, b"123.155");
    test_write_created_value(123, 156, b"123.156");
    test_write_created_value(123, 157, b"123.157");
    test_write_created_value(123, 158, b"123.158");
    test_write_created_value(123, 159, b"123.159");
}

#[test]
fn created_value_written_correctly_for_millisecond_160_to_169() {
    test_write_created_value(123, 160, b"123.160");
    test_write_created_value(123, 161, b"123.161");
    test_write_created_value(123, 162, b"123.162");
    test_write_created_value(123, 163, b"123.163");
    test_write_created_value(123, 164, b"123.164");
    test_write_created_value(123, 165, b"123.165");
    test_write_created_value(123, 166, b"123.166");
    test_write_created_value(123, 167, b"123.167");
    test_write_created_value(123, 168, b"123.168");
    test_write_created_value(123, 169, b"123.169");
}

#[test]
fn created_value_written_correctly_for_millisecond_170_to_179() {
    test_write_created_value(123, 170, b"123.170");
    test_write_created_value(123, 171, b"123.171");
    test_write_created_value(123, 172, b"123.172");
    test_write_created_value(123, 173, b"123.173");
    test_write_created_value(123, 174, b"123.174");
    test_write_created_value(123, 175, b"123.175");
    test_write_created_value(123, 176, b"123.176");
    test_write_created_value(123, 177, b"123.177");
    test_write_created_value(123, 178, b"123.178");
    test_write_created_value(123, 179, b"123.179");
}

#[test]
fn created_value_written_correctly_for_millisecond_180_to_189() {
    test_write_created_value(123, 180, b"123.180");
    test_write_created_value(123, 181, b"123.181");
    test_write_created_value(123, 182, b"123.182");
    test_write_created_value(123, 183, b"123.183");
    test_write_created_value(123, 184, b"123.184");
    test_write_created_value(123, 185, b"123.185");
    test_write_created_value(123, 186, b"123.186");
    test_write_created_value(123, 187, b"123.187");
    test_write_created_value(123, 188, b"123.188");
    test_write_created_value(123, 189, b"123.189");
}

#[test]
fn created_value_written_correctly_for_millisecond_190_to_199() {
    test_write_created_value(123, 190, b"123.190");
    test_write_created_value(123, 191, b"123.191");
    test_write_created_value(123, 192, b"123.192");
    test_write_created_value(123, 193, b"123.193");
    test_write_created_value(123, 194, b"123.194");
    test_write_created_value(123, 195, b"123.195");
    test_write_created_value(123, 196, b"123.196");
    test_write_created_value(123, 197, b"123.197");
    test_write_created_value(123, 198, b"123.198");
    test_write_created_value(123, 199, b"123.199");
}

#[test]
fn created_value_written_correctly_for_millisecond_200_to_209() {
    test_write_created_value(123, 200, b"123.200");
    test_write_created_value(123, 201, b"123.201");
    test_write_created_value(123, 202, b"123.202");
    test_write_created_value(123, 203, b"123.203");
    test_write_created_value(123, 204, b"123.204");
    test_write_created_value(123, 205, b"123.205");
    test_write_created_value(123, 206, b"123.206");
    test_write_created_value(123, 207, b"123.207");
    test_write_created_value(123, 208, b"123.208");
    test_write_created_value(123, 209, b"123.209");
}

#[test]
fn created_value_written_correctly_for_millisecond_210_to_219() {
    test_write_created_value(123, 210, b"123.210");
    test_write_created_value(123, 211, b"123.211");
    test_write_created_value(123, 212, b"123.212");
    test_write_created_value(123, 213, b"123.213");
    test_write_created_value(123, 214, b"123.214");
    test_write_created_value(123, 215, b"123.215");
    test_write_created_value(123, 216, b"123.216");
    test_write_created_value(123, 217, b"123.217");
    test_write_created_value(123, 218, b"123.218");
    test_write_created_value(123, 219, b"123.219");
}

#[test]
fn created_value_written_correctly_for_millisecond_220_to_229() {
    test_write_created_value(123, 220, b"123.220");
    test_write_created_value(123, 221, b"123.221");
    test_write_created_value(123, 222, b"123.222");
    test_write_created_value(123, 223, b"123.223");
    test_write_created_value(123, 224, b"123.224");
    test_write_created_value(123, 225, b"123.225");
    test_write_created_value(123, 226, b"123.226");
    test_write_created_value(123, 227, b"123.227");
    test_write_created_value(123, 228, b"123.228");
    test_write_created_value(123, 229, b"123.229");
}

#[test]
fn created_value_written_correctly_for_millisecond_230_to_239() {
    test_write_created_value(123, 230, b"123.230");
    test_write_created_value(123, 231, b"123.231");
    test_write_created_value(123, 232, b"123.232");
    test_write_created_value(123, 233, b"123.233");
    test_write_created_value(123, 234, b"123.234");
    test_write_created_value(123, 235, b"123.235");
    test_write_created_value(123, 236, b"123.236");
    test_write_created_value(123, 237, b"123.237");
    test_write_created_value(123, 238, b"123.238");
    test_write_created_value(123, 239, b"123.239");
}

#[test]
fn created_value_written_correctly_for_millisecond_240_to_249() {
    test_write_created_value(123, 240, b"123.240");
    test_write_created_value(123, 241, b"123.241");
    test_write_created_value(123, 242, b"123.242");
    test_write_created_value(123, 243, b"123.243");
    test_write_created_value(123, 244, b"123.244");
    test_write_created_value(123, 245, b"123.245");
    test_write_created_value(123, 246, b"123.246");
    test_write_created_value(123, 247, b"123.247");
    test_write_created_value(123, 248, b"123.248");
    test_write_created_value(123, 249, b"123.249");
}

#[test]
fn created_value_written_correctly_for_millisecond_250_to_259() {
    test_write_created_value(123, 250, b"123.250");
    test_write_created_value(123, 251, b"123.251");
    test_write_created_value(123, 252, b"123.252");
    test_write_created_value(123, 253, b"123.253");
    test_write_created_value(123, 254, b"123.254");
    test_write_created_value(123, 255, b"123.255");
    test_write_created_value(123, 256, b"123.256");
    test_write_created_value(123, 257, b"123.257");
    test_write_created_value(123, 258, b"123.258");
    test_write_created_value(123, 259, b"123.259");
}

#[test]
fn created_value_written_correctly_for_millisecond_260_to_269() {
    test_write_created_value(123, 260, b"123.260");
    test_write_created_value(123, 261, b"123.261");
    test_write_created_value(123, 262, b"123.262");
    test_write_created_value(123, 263, b"123.263");
    test_write_created_value(123, 264, b"123.264");
    test_write_created_value(123, 265, b"123.265");
    test_write_created_value(123, 266, b"123.266");
    test_write_created_value(123, 267, b"123.267");
    test_write_created_value(123, 268, b"123.268");
    test_write_created_value(123, 269, b"123.269");
}

#[test]
fn created_value_written_correctly_for_millisecond_270_to_279() {
    test_write_created_value(123, 270, b"123.270");
    test_write_created_value(123, 271, b"123.271");
    test_write_created_value(123, 272, b"123.272");
    test_write_created_value(123, 273, b"123.273");
    test_write_created_value(123, 274, b"123.274");
    test_write_created_value(123, 275, b"123.275");
    test_write_created_value(123, 276, b"123.276");
    test_write_created_value(123, 277, b"123.277");
    test_write_created_value(123, 278, b"123.278");
    test_write_created_value(123, 279, b"123.279");
}

#[test]
fn created_value_written_correctly_for_millisecond_280_to_289() {
    test_write_created_value(123, 280, b"123.280");
    test_write_created_value(123, 281, b"123.281");
    test_write_created_value(123, 282, b"123.282");
    test_write_created_value(123, 283, b"123.283");
    test_write_created_value(123, 284, b"123.284");
    test_write_created_value(123, 285, b"123.285");
    test_write_created_value(123, 286, b"123.286");
    test_write_created_value(123, 287, b"123.287");
    test_write_created_value(123, 288, b"123.288");
    test_write_created_value(123, 289, b"123.289");
}

#[test]
fn created_value_written_correctly_for_millisecond_290_to_299() {
    test_write_created_value(123, 290, b"123.290");
    test_write_created_value(123, 291, b"123.291");
    test_write_created_value(123, 292, b"123.292");
    test_write_created_value(123, 293, b"123.293");
    test_write_created_value(123, 294, b"123.294");
    test_write_created_value(123, 295, b"123.295");
    test_write_created_value(123, 296, b"123.296");
    test_write_created_value(123, 297, b"123.297");
    test_write_created_value(123, 298, b"123.298");
    test_write_created_value(123, 299, b"123.299");
}

#[test]
fn created_value_written_correctly_for_millisecond_300_to_309() {
    test_write_created_value(123, 300, b"123.300");
    test_write_created_value(123, 301, b"123.301");
    test_write_created_value(123, 302, b"123.302");
    test_write_created_value(123, 303, b"123.303");
    test_write_created_value(123, 304, b"123.304");
    test_write_created_value(123, 305, b"123.305");
    test_write_created_value(123, 306, b"123.306");
    test_write_created_value(123, 307, b"123.307");
    test_write_created_value(123, 308, b"123.308");
    test_write_created_value(123, 309, b"123.309");
}

#[test]
fn created_value_written_correctly_for_millisecond_310_to_319() {
    test_write_created_value(123, 310, b"123.310");
    test_write_created_value(123, 311, b"123.311");
    test_write_created_value(123, 312, b"123.312");
    test_write_created_value(123, 313, b"123.313");
    test_write_created_value(123, 314, b"123.314");
    test_write_created_value(123, 315, b"123.315");
    test_write_created_value(123, 316, b"123.316");
    test_write_created_value(123, 317, b"123.317");
    test_write_created_value(123, 318, b"123.318");
    test_write_created_value(123, 319, b"123.319");
}

#[test]
fn created_value_written_correctly_for_millisecond_320_to_329() {
    test_write_created_value(123, 320, b"123.320");
    test_write_created_value(123, 321, b"123.321");
    test_write_created_value(123, 322, b"123.322");
    test_write_created_value(123, 323, b"123.323");
    test_write_created_value(123, 324, b"123.324");
    test_write_created_value(123, 325, b"123.325");
    test_write_created_value(123, 326, b"123.326");
    test_write_created_value(123, 327, b"123.327");
    test_write_created_value(123, 328, b"123.328");
    test_write_created_value(123, 329, b"123.329");
}

#[test]
fn created_value_written_correctly_for_millisecond_330_to_339() {
    test_write_created_value(123, 330, b"123.330");
    test_write_created_value(123, 331, b"123.331");
    test_write_created_value(123, 332, b"123.332");
    test_write_created_value(123, 333, b"123.333");
    test_write_created_value(123, 334, b"123.334");
    test_write_created_value(123, 335, b"123.335");
    test_write_created_value(123, 336, b"123.336");
    test_write_created_value(123, 337, b"123.337");
    test_write_created_value(123, 338, b"123.338");
    test_write_created_value(123, 339, b"123.339");
}

#[test]
fn created_value_written_correctly_for_millisecond_340_to_349() {
    test_write_created_value(123, 340, b"123.340");
    test_write_created_value(123, 341, b"123.341");
    test_write_created_value(123, 342, b"123.342");
    test_write_created_value(123, 343, b"123.343");
    test_write_created_value(123, 344, b"123.344");
    test_write_created_value(123, 345, b"123.345");
    test_write_created_value(123, 346, b"123.346");
    test_write_created_value(123, 347, b"123.347");
    test_write_created_value(123, 348, b"123.348");
    test_write_created_value(123, 349, b"123.349");
}

#[test]
fn created_value_written_correctly_for_millisecond_350_to_359() {
    test_write_created_value(123, 350, b"123.350");
    test_write_created_value(123, 351, b"123.351");
    test_write_created_value(123, 352, b"123.352");
    test_write_created_value(123, 353, b"123.353");
    test_write_created_value(123, 354, b"123.354");
    test_write_created_value(123, 355, b"123.355");
    test_write_created_value(123, 356, b"123.356");
    test_write_created_value(123, 357, b"123.357");
    test_write_created_value(123, 358, b"123.358");
    test_write_created_value(123, 359, b"123.359");
}

#[test]
fn created_value_written_correctly_for_millisecond_360_to_369() {
    test_write_created_value(123, 360, b"123.360");
    test_write_created_value(123, 361, b"123.361");
    test_write_created_value(123, 362, b"123.362");
    test_write_created_value(123, 363, b"123.363");
    test_write_created_value(123, 364, b"123.364");
    test_write_created_value(123, 365, b"123.365");
    test_write_created_value(123, 366, b"123.366");
    test_write_created_value(123, 367, b"123.367");
    test_write_created_value(123, 368, b"123.368");
    test_write_created_value(123, 369, b"123.369");
}

#[test]
fn created_value_written_correctly_for_millisecond_370_to_379() {
    test_write_created_value(123, 370, b"123.370");
    test_write_created_value(123, 371, b"123.371");
    test_write_created_value(123, 372, b"123.372");
    test_write_created_value(123, 373, b"123.373");
    test_write_created_value(123, 374, b"123.374");
    test_write_created_value(123, 375, b"123.375");
    test_write_created_value(123, 376, b"123.376");
    test_write_created_value(123, 377, b"123.377");
    test_write_created_value(123, 378, b"123.378");
    test_write_created_value(123, 379, b"123.379");
}

#[test]
fn created_value_written_correctly_for_millisecond_380_to_389() {
    test_write_created_value(123, 380, b"123.380");
    test_write_created_value(123, 381, b"123.381");
    test_write_created_value(123, 382, b"123.382");
    test_write_created_value(123, 383, b"123.383");
    test_write_created_value(123, 384, b"123.384");
    test_write_created_value(123, 385, b"123.385");
    test_write_created_value(123, 386, b"123.386");
    test_write_created_value(123, 387, b"123.387");
    test_write_created_value(123, 388, b"123.388");
    test_write_created_value(123, 389, b"123.389");
}

#[test]
fn created_value_written_correctly_for_millisecond_390_to_399() {
    test_write_created_value(123, 390, b"123.390");
    test_write_created_value(123, 391, b"123.391");
    test_write_created_value(123, 392, b"123.392");
    test_write_created_value(123, 393, b"123.393");
    test_write_created_value(123, 394, b"123.394");
    test_write_created_value(123, 395, b"123.395");
    test_write_created_value(123, 396, b"123.396");
    test_write_created_value(123, 397, b"123.397");
    test_write_created_value(123, 398, b"123.398");
    test_write_created_value(123, 399, b"123.399");
}

#[test]
fn created_value_written_correctly_for_millisecond_400_to_409() {
    test_write_created_value(123, 400, b"123.400");
    test_write_created_value(123, 401, b"123.401");
    test_write_created_value(123, 402, b"123.402");
    test_write_created_value(123, 403, b"123.403");
    test_write_created_value(123, 404, b"123.404");
    test_write_created_value(123, 405, b"123.405");
    test_write_created_value(123, 406, b"123.406");
    test_write_created_value(123, 407, b"123.407");
    test_write_created_value(123, 408, b"123.408");
    test_write_created_value(123, 409, b"123.409");
}

#[test]
fn created_value_written_correctly_for_millisecond_410_to_419() {
    test_write_created_value(123, 410, b"123.410");
    test_write_created_value(123, 411, b"123.411");
    test_write_created_value(123, 412, b"123.412");
    test_write_created_value(123, 413, b"123.413");
    test_write_created_value(123, 414, b"123.414");
    test_write_created_value(123, 415, b"123.415");
    test_write_created_value(123, 416, b"123.416");
    test_write_created_value(123, 417, b"123.417");
    test_write_created_value(123, 418, b"123.418");
    test_write_created_value(123, 419, b"123.419");
}

#[test]
fn created_value_written_correctly_for_millisecond_420_to_429() {
    test_write_created_value(123, 420, b"123.420");
    test_write_created_value(123, 421, b"123.421");
    test_write_created_value(123, 422, b"123.422");
    test_write_created_value(123, 423, b"123.423");
    test_write_created_value(123, 424, b"123.424");
    test_write_created_value(123, 425, b"123.425");
    test_write_created_value(123, 426, b"123.426");
    test_write_created_value(123, 427, b"123.427");
    test_write_created_value(123, 428, b"123.428");
    test_write_created_value(123, 429, b"123.429");
}

#[test]
fn created_value_written_correctly_for_millisecond_430_to_439() {
    test_write_created_value(123, 430, b"123.430");
    test_write_created_value(123, 431, b"123.431");
    test_write_created_value(123, 432, b"123.432");
    test_write_created_value(123, 433, b"123.433");
    test_write_created_value(123, 434, b"123.434");
    test_write_created_value(123, 435, b"123.435");
    test_write_created_value(123, 436, b"123.436");
    test_write_created_value(123, 437, b"123.437");
    test_write_created_value(123, 438, b"123.438");
    test_write_created_value(123, 439, b"123.439");
}

#[test]
fn created_value_written_correctly_for_millisecond_440_to_449() {
    test_write_created_value(123, 440, b"123.440");
    test_write_created_value(123, 441, b"123.441");
    test_write_created_value(123, 442, b"123.442");
    test_write_created_value(123, 443, b"123.443");
    test_write_created_value(123, 444, b"123.444");
    test_write_created_value(123, 445, b"123.445");
    test_write_created_value(123, 446, b"123.446");
    test_write_created_value(123, 447, b"123.447");
    test_write_created_value(123, 448, b"123.448");
    test_write_created_value(123, 449, b"123.449");
}

#[test]
fn created_value_written_correctly_for_millisecond_450_to_459() {
    test_write_created_value(123, 450, b"123.450");
    test_write_created_value(123, 451, b"123.451");
    test_write_created_value(123, 452, b"123.452");
    test_write_created_value(123, 453, b"123.453");
    test_write_created_value(123, 454, b"123.454");
    test_write_created_value(123, 455, b"123.455");
    test_write_created_value(123, 456, b"123.456");
    test_write_created_value(123, 457, b"123.457");
    test_write_created_value(123, 458, b"123.458");
    test_write_created_value(123, 459, b"123.459");
}

#[test]
fn created_value_written_correctly_for_millisecond_460_to_469() {
    test_write_created_value(123, 460, b"123.460");
    test_write_created_value(123, 461, b"123.461");
    test_write_created_value(123, 462, b"123.462");
    test_write_created_value(123, 463, b"123.463");
    test_write_created_value(123, 464, b"123.464");
    test_write_created_value(123, 465, b"123.465");
    test_write_created_value(123, 466, b"123.466");
    test_write_created_value(123, 467, b"123.467");
    test_write_created_value(123, 468, b"123.468");
    test_write_created_value(123, 469, b"123.469");
}

#[test]
fn created_value_written_correctly_for_millisecond_470_to_479() {
    test_write_created_value(123, 470, b"123.470");
    test_write_created_value(123, 471, b"123.471");
    test_write_created_value(123, 472, b"123.472");
    test_write_created_value(123, 473, b"123.473");
    test_write_created_value(123, 474, b"123.474");
    test_write_created_value(123, 475, b"123.475");
    test_write_created_value(123, 476, b"123.476");
    test_write_created_value(123, 477, b"123.477");
    test_write_created_value(123, 478, b"123.478");
    test_write_created_value(123, 479, b"123.479");
}

#[test]
fn created_value_written_correctly_for_millisecond_480_to_489() {
    test_write_created_value(123, 480, b"123.480");
    test_write_created_value(123, 481, b"123.481");
    test_write_created_value(123, 482, b"123.482");
    test_write_created_value(123, 483, b"123.483");
    test_write_created_value(123, 484, b"123.484");
    test_write_created_value(123, 485, b"123.485");
    test_write_created_value(123, 486, b"123.486");
    test_write_created_value(123, 487, b"123.487");
    test_write_created_value(123, 488, b"123.488");
    test_write_created_value(123, 489, b"123.489");
}

#[test]
fn created_value_written_correctly_for_millisecond_490_to_499() {
    test_write_created_value(123, 490, b"123.490");
    test_write_created_value(123, 491, b"123.491");
    test_write_created_value(123, 492, b"123.492");
    test_write_created_value(123, 493, b"123.493");
    test_write_created_value(123, 494, b"123.494");
    test_write_created_value(123, 495, b"123.495");
    test_write_created_value(123, 496, b"123.496");
    test_write_created_value(123, 497, b"123.497");
    test_write_created_value(123, 498, b"123.498");
    test_write_created_value(123, 499, b"123.499");
}

#[test]
fn created_value_written_correctly_for_millisecond_500_to_509() {
    test_write_created_value(123, 500, b"123.500");
    test_write_created_value(123, 501, b"123.501");
    test_write_created_value(123, 502, b"123.502");
    test_write_created_value(123, 503, b"123.503");
    test_write_created_value(123, 504, b"123.504");
    test_write_created_value(123, 505, b"123.505");
    test_write_created_value(123, 506, b"123.506");
    test_write_created_value(123, 507, b"123.507");
    test_write_created_value(123, 508, b"123.508");
    test_write_created_value(123, 509, b"123.509");
}

#[test]
fn created_value_written_correctly_for_millisecond_510_to_519() {
    test_write_created_value(123, 510, b"123.510");
    test_write_created_value(123, 511, b"123.511");
    test_write_created_value(123, 512, b"123.512");
    test_write_created_value(123, 513, b"123.513");
    test_write_created_value(123, 514, b"123.514");
    test_write_created_value(123, 515, b"123.515");
    test_write_created_value(123, 516, b"123.516");
    test_write_created_value(123, 517, b"123.517");
    test_write_created_value(123, 518, b"123.518");
    test_write_created_value(123, 519, b"123.519");
}

#[test]
fn created_value_written_correctly_for_millisecond_520_to_529() {
    test_write_created_value(123, 520, b"123.520");
    test_write_created_value(123, 521, b"123.521");
    test_write_created_value(123, 522, b"123.522");
    test_write_created_value(123, 523, b"123.523");
    test_write_created_value(123, 524, b"123.524");
    test_write_created_value(123, 525, b"123.525");
    test_write_created_value(123, 526, b"123.526");
    test_write_created_value(123, 527, b"123.527");
    test_write_created_value(123, 528, b"123.528");
    test_write_created_value(123, 529, b"123.529");
}

#[test]
fn created_value_written_correctly_for_millisecond_530_to_539() {
    test_write_created_value(123, 530, b"123.530");
    test_write_created_value(123, 531, b"123.531");
    test_write_created_value(123, 532, b"123.532");
    test_write_created_value(123, 533, b"123.533");
    test_write_created_value(123, 534, b"123.534");
    test_write_created_value(123, 535, b"123.535");
    test_write_created_value(123, 536, b"123.536");
    test_write_created_value(123, 537, b"123.537");
    test_write_created_value(123, 538, b"123.538");
    test_write_created_value(123, 539, b"123.539");
}

#[test]
fn created_value_written_correctly_for_millisecond_540_to_549() {
    test_write_created_value(123, 540, b"123.540");
    test_write_created_value(123, 541, b"123.541");
    test_write_created_value(123, 542, b"123.542");
    test_write_created_value(123, 543, b"123.543");
    test_write_created_value(123, 544, b"123.544");
    test_write_created_value(123, 545, b"123.545");
    test_write_created_value(123, 546, b"123.546");
    test_write_created_value(123, 547, b"123.547");
    test_write_created_value(123, 548, b"123.548");
    test_write_created_value(123, 549, b"123.549");
}

#[test]
fn created_value_written_correctly_for_millisecond_550_to_559() {
    test_write_created_value(123, 550, b"123.550");
    test_write_created_value(123, 551, b"123.551");
    test_write_created_value(123, 552, b"123.552");
    test_write_created_value(123, 553, b"123.553");
    test_write_created_value(123, 554, b"123.554");
    test_write_created_value(123, 555, b"123.555");
    test_write_created_value(123, 556, b"123.556");
    test_write_created_value(123, 557, b"123.557");
    test_write_created_value(123, 558, b"123.558");
    test_write_created_value(123, 559, b"123.559");
}

#[test]
fn created_value_written_correctly_for_millisecond_560_to_569() {
    test_write_created_value(123, 560, b"123.560");
    test_write_created_value(123, 561, b"123.561");
    test_write_created_value(123, 562, b"123.562");
    test_write_created_value(123, 563, b"123.563");
    test_write_created_value(123, 564, b"123.564");
    test_write_created_value(123, 565, b"123.565");
    test_write_created_value(123, 566, b"123.566");
    test_write_created_value(123, 567, b"123.567");
    test_write_created_value(123, 568, b"123.568");
    test_write_created_value(123, 569, b"123.569");
}

#[test]
fn created_value_written_correctly_for_millisecond_570_to_579() {
    test_write_created_value(123, 570, b"123.570");
    test_write_created_value(123, 571, b"123.571");
    test_write_created_value(123, 572, b"123.572");
    test_write_created_value(123, 573, b"123.573");
    test_write_created_value(123, 574, b"123.574");
    test_write_created_value(123, 575, b"123.575");
    test_write_created_value(123, 576, b"123.576");
    test_write_created_value(123, 577, b"123.577");
    test_write_created_value(123, 578, b"123.578");
    test_write_created_value(123, 579, b"123.579");
}

#[test]
fn created_value_written_correctly_for_millisecond_580_to_589() {
    test_write_created_value(123, 580, b"123.580");
    test_write_created_value(123, 581, b"123.581");
    test_write_created_value(123, 582, b"123.582");
    test_write_created_value(123, 583, b"123.583");
    test_write_created_value(123, 584, b"123.584");
    test_write_created_value(123, 585, b"123.585");
    test_write_created_value(123, 586, b"123.586");
    test_write_created_value(123, 587, b"123.587");
    test_write_created_value(123, 588, b"123.588");
    test_write_created_value(123, 589, b"123.589");
}

#[test]
fn created_value_written_correctly_for_millisecond_590_to_599() {
    test_write_created_value(123, 590, b"123.590");
    test_write_created_value(123, 591, b"123.591");
    test_write_created_value(123, 592, b"123.592");
    test_write_created_value(123, 593, b"123.593");
    test_write_created_value(123, 594, b"123.594");
    test_write_created_value(123, 595, b"123.595");
    test_write_created_value(123, 596, b"123.596");
    test_write_created_value(123, 597, b"123.597");
    test_write_created_value(123, 598, b"123.598");
    test_write_created_value(123, 599, b"123.599");
}

#[test]
fn created_value_written_correctly_for_millisecond_600_to_609() {
    test_write_created_value(123, 600, b"123.600");
    test_write_created_value(123, 601, b"123.601");
    test_write_created_value(123, 602, b"123.602");
    test_write_created_value(123, 603, b"123.603");
    test_write_created_value(123, 604, b"123.604");
    test_write_created_value(123, 605, b"123.605");
    test_write_created_value(123, 606, b"123.606");
    test_write_created_value(123, 607, b"123.607");
    test_write_created_value(123, 608, b"123.608");
    test_write_created_value(123, 609, b"123.609");
}

#[test]
fn created_value_written_correctly_for_millisecond_610_to_619() {
    test_write_created_value(123, 610, b"123.610");
    test_write_created_value(123, 611, b"123.611");
    test_write_created_value(123, 612, b"123.612");
    test_write_created_value(123, 613, b"123.613");
    test_write_created_value(123, 614, b"123.614");
    test_write_created_value(123, 615, b"123.615");
    test_write_created_value(123, 616, b"123.616");
    test_write_created_value(123, 617, b"123.617");
    test_write_created_value(123, 618, b"123.618");
    test_write_created_value(123, 619, b"123.619");
}

#[test]
fn created_value_written_correctly_for_millisecond_620_to_629() {
    test_write_created_value(123, 620, b"123.620");
    test_write_created_value(123, 621, b"123.621");
    test_write_created_value(123, 622, b"123.622");
    test_write_created_value(123, 623, b"123.623");
    test_write_created_value(123, 624, b"123.624");
    test_write_created_value(123, 625, b"123.625");
    test_write_created_value(123, 626, b"123.626");
    test_write_created_value(123, 627, b"123.627");
    test_write_created_value(123, 628, b"123.628");
    test_write_created_value(123, 629, b"123.629");
}

#[test]
fn created_value_written_correctly_for_millisecond_630_to_639() {
    test_write_created_value(123, 630, b"123.630");
    test_write_created_value(123, 631, b"123.631");
    test_write_created_value(123, 632, b"123.632");
    test_write_created_value(123, 633, b"123.633");
    test_write_created_value(123, 634, b"123.634");
    test_write_created_value(123, 635, b"123.635");
    test_write_created_value(123, 636, b"123.636");
    test_write_created_value(123, 637, b"123.637");
    test_write_created_value(123, 638, b"123.638");
    test_write_created_value(123, 639, b"123.639");
}

#[test]
fn created_value_written_correctly_for_millisecond_640_to_649() {
    test_write_created_value(123, 640, b"123.640");
    test_write_created_value(123, 641, b"123.641");
    test_write_created_value(123, 642, b"123.642");
    test_write_created_value(123, 643, b"123.643");
    test_write_created_value(123, 644, b"123.644");
    test_write_created_value(123, 645, b"123.645");
    test_write_created_value(123, 646, b"123.646");
    test_write_created_value(123, 647, b"123.647");
    test_write_created_value(123, 648, b"123.648");
    test_write_created_value(123, 649, b"123.649");
}

#[test]
fn created_value_written_correctly_for_millisecond_650_to_659() {
    test_write_created_value(123, 650, b"123.650");
    test_write_created_value(123, 651, b"123.651");
    test_write_created_value(123, 652, b"123.652");
    test_write_created_value(123, 653, b"123.653");
    test_write_created_value(123, 654, b"123.654");
    test_write_created_value(123, 655, b"123.655");
    test_write_created_value(123, 656, b"123.656");
    test_write_created_value(123, 657, b"123.657");
    test_write_created_value(123, 658, b"123.658");
    test_write_created_value(123, 659, b"123.659");
}

#[test]
fn created_value_written_correctly_for_millisecond_660_to_669() {
    test_write_created_value(123, 660, b"123.660");
    test_write_created_value(123, 661, b"123.661");
    test_write_created_value(123, 662, b"123.662");
    test_write_created_value(123, 663, b"123.663");
    test_write_created_value(123, 664, b"123.664");
    test_write_created_value(123, 665, b"123.665");
    test_write_created_value(123, 666, b"123.666");
    test_write_created_value(123, 667, b"123.667");
    test_write_created_value(123, 668, b"123.668");
    test_write_created_value(123, 669, b"123.669");
}

#[test]
fn created_value_written_correctly_for_millisecond_670_to_679() {
    test_write_created_value(123, 670, b"123.670");
    test_write_created_value(123, 671, b"123.671");
    test_write_created_value(123, 672, b"123.672");
    test_write_created_value(123, 673, b"123.673");
    test_write_created_value(123, 674, b"123.674");
    test_write_created_value(123, 675, b"123.675");
    test_write_created_value(123, 676, b"123.676");
    test_write_created_value(123, 677, b"123.677");
    test_write_created_value(123, 678, b"123.678");
    test_write_created_value(123, 679, b"123.679");
}

#[test]
fn created_value_written_correctly_for_millisecond_680_to_689() {
    test_write_created_value(123, 680, b"123.680");
    test_write_created_value(123, 681, b"123.681");
    test_write_created_value(123, 682, b"123.682");
    test_write_created_value(123, 683, b"123.683");
    test_write_created_value(123, 684, b"123.684");
    test_write_created_value(123, 685, b"123.685");
    test_write_created_value(123, 686, b"123.686");
    test_write_created_value(123, 687, b"123.687");
    test_write_created_value(123, 688, b"123.688");
    test_write_created_value(123, 689, b"123.689");
}

#[test]
fn created_value_written_correctly_for_millisecond_690_to_699() {
    test_write_created_value(123, 690, b"123.690");
    test_write_created_value(123, 691, b"123.691");
    test_write_created_value(123, 692, b"123.692");
    test_write_created_value(123, 693, b"123.693");
    test_write_created_value(123, 694, b"123.694");
    test_write_created_value(123, 695, b"123.695");
    test_write_created_value(123, 696, b"123.696");
    test_write_created_value(123, 697, b"123.697");
    test_write_created_value(123, 698, b"123.698");
    test_write_created_value(123, 699, b"123.699");
}

#[test]
fn created_value_written_correctly_for_millisecond_700_to_709() {
    test_write_created_value(123, 700, b"123.700");
    test_write_created_value(123, 701, b"123.701");
    test_write_created_value(123, 702, b"123.702");
    test_write_created_value(123, 703, b"123.703");
    test_write_created_value(123, 704, b"123.704");
    test_write_created_value(123, 705, b"123.705");
    test_write_created_value(123, 706, b"123.706");
    test_write_created_value(123, 707, b"123.707");
    test_write_created_value(123, 708, b"123.708");
    test_write_created_value(123, 709, b"123.709");
}

#[test]
fn created_value_written_correctly_for_millisecond_710_to_719() {
    test_write_created_value(123, 710, b"123.710");
    test_write_created_value(123, 711, b"123.711");
    test_write_created_value(123, 712, b"123.712");
    test_write_created_value(123, 713, b"123.713");
    test_write_created_value(123, 714, b"123.714");
    test_write_created_value(123, 715, b"123.715");
    test_write_created_value(123, 716, b"123.716");
    test_write_created_value(123, 717, b"123.717");
    test_write_created_value(123, 718, b"123.718");
    test_write_created_value(123, 719, b"123.719");
}

#[test]
fn created_value_written_correctly_for_millisecond_720_to_729() {
    test_write_created_value(123, 720, b"123.720");
    test_write_created_value(123, 721, b"123.721");
    test_write_created_value(123, 722, b"123.722");
    test_write_created_value(123, 723, b"123.723");
    test_write_created_value(123, 724, b"123.724");
    test_write_created_value(123, 725, b"123.725");
    test_write_created_value(123, 726, b"123.726");
    test_write_created_value(123, 727, b"123.727");
    test_write_created_value(123, 728, b"123.728");
    test_write_created_value(123, 729, b"123.729");
}

#[test]
fn created_value_written_correctly_for_millisecond_730_to_739() {
    test_write_created_value(123, 730, b"123.730");
    test_write_created_value(123, 731, b"123.731");
    test_write_created_value(123, 732, b"123.732");
    test_write_created_value(123, 733, b"123.733");
    test_write_created_value(123, 734, b"123.734");
    test_write_created_value(123, 735, b"123.735");
    test_write_created_value(123, 736, b"123.736");
    test_write_created_value(123, 737, b"123.737");
    test_write_created_value(123, 738, b"123.738");
    test_write_created_value(123, 739, b"123.739");
}

#[test]
fn created_value_written_correctly_for_millisecond_740_to_749() {
    test_write_created_value(123, 740, b"123.740");
    test_write_created_value(123, 741, b"123.741");
    test_write_created_value(123, 742, b"123.742");
    test_write_created_value(123, 743, b"123.743");
    test_write_created_value(123, 744, b"123.744");
    test_write_created_value(123, 745, b"123.745");
    test_write_created_value(123, 746, b"123.746");
    test_write_created_value(123, 747, b"123.747");
    test_write_created_value(123, 748, b"123.748");
    test_write_created_value(123, 749, b"123.749");
}

#[test]
fn created_value_written_correctly_for_millisecond_750_to_759() {
    test_write_created_value(123, 750, b"123.750");
    test_write_created_value(123, 751, b"123.751");
    test_write_created_value(123, 752, b"123.752");
    test_write_created_value(123, 753, b"123.753");
    test_write_created_value(123, 754, b"123.754");
    test_write_created_value(123, 755, b"123.755");
    test_write_created_value(123, 756, b"123.756");
    test_write_created_value(123, 757, b"123.757");
    test_write_created_value(123, 758, b"123.758");
    test_write_created_value(123, 759, b"123.759");
}

#[test]
fn created_value_written_correctly_for_millisecond_760_to_769() {
    test_write_created_value(123, 760, b"123.760");
    test_write_created_value(123, 761, b"123.761");
    test_write_created_value(123, 762, b"123.762");
    test_write_created_value(123, 763, b"123.763");
    test_write_created_value(123, 764, b"123.764");
    test_write_created_value(123, 765, b"123.765");
    test_write_created_value(123, 766, b"123.766");
    test_write_created_value(123, 767, b"123.767");
    test_write_created_value(123, 768, b"123.768");
    test_write_created_value(123, 769, b"123.769");
}

#[test]
fn created_value_written_correctly_for_millisecond_770_to_779() {
    test_write_created_value(123, 770, b"123.770");
    test_write_created_value(123, 771, b"123.771");
    test_write_created_value(123, 772, b"123.772");
    test_write_created_value(123, 773, b"123.773");
    test_write_created_value(123, 774, b"123.774");
    test_write_created_value(123, 775, b"123.775");
    test_write_created_value(123, 776, b"123.776");
    test_write_created_value(123, 777, b"123.777");
    test_write_created_value(123, 778, b"123.778");
    test_write_created_value(123, 779, b"123.779");
}

#[test]
fn created_value_written_correctly_for_millisecond_780_to_789() {
    test_write_created_value(123, 780, b"123.780");
    test_write_created_value(123, 781, b"123.781");
    test_write_created_value(123, 782, b"123.782");
    test_write_created_value(123, 783, b"123.783");
    test_write_created_value(123, 784, b"123.784");
    test_write_created_value(123, 785, b"123.785");
    test_write_created_value(123, 786, b"123.786");
    test_write_created_value(123, 787, b"123.787");
    test_write_created_value(123, 788, b"123.788");
    test_write_created_value(123, 789, b"123.789");
}

#[test]
fn created_value_written_correctly_for_millisecond_790_to_799() {
    test_write_created_value(123, 790, b"123.790");
    test_write_created_value(123, 791, b"123.791");
    test_write_created_value(123, 792, b"123.792");
    test_write_created_value(123, 793, b"123.793");
    test_write_created_value(123, 794, b"123.794");
    test_write_created_value(123, 795, b"123.795");
    test_write_created_value(123, 796, b"123.796");
    test_write_created_value(123, 797, b"123.797");
    test_write_created_value(123, 798, b"123.798");
    test_write_created_value(123, 799, b"123.799");
}

#[test]
fn created_value_written_correctly_for_millisecond_800_to_809() {
    test_write_created_value(123, 800, b"123.800");
    test_write_created_value(123, 801, b"123.801");
    test_write_created_value(123, 802, b"123.802");
    test_write_created_value(123, 803, b"123.803");
    test_write_created_value(123, 804, b"123.804");
    test_write_created_value(123, 805, b"123.805");
    test_write_created_value(123, 806, b"123.806");
    test_write_created_value(123, 807, b"123.807");
    test_write_created_value(123, 808, b"123.808");
    test_write_created_value(123, 809, b"123.809");
}

#[test]
fn created_value_written_correctly_for_millisecond_810_to_819() {
    test_write_created_value(123, 810, b"123.810");
    test_write_created_value(123, 811, b"123.811");
    test_write_created_value(123, 812, b"123.812");
    test_write_created_value(123, 813, b"123.813");
    test_write_created_value(123, 814, b"123.814");
    test_write_created_value(123, 815, b"123.815");
    test_write_created_value(123, 816, b"123.816");
    test_write_created_value(123, 817, b"123.817");
    test_write_created_value(123, 818, b"123.818");
    test_write_created_value(123, 819, b"123.819");
}

#[test]
fn created_value_written_correctly_for_millisecond_820_to_829() {
    test_write_created_value(123, 820, b"123.820");
    test_write_created_value(123, 821, b"123.821");
    test_write_created_value(123, 822, b"123.822");
    test_write_created_value(123, 823, b"123.823");
    test_write_created_value(123, 824, b"123.824");
    test_write_created_value(123, 825, b"123.825");
    test_write_created_value(123, 826, b"123.826");
    test_write_created_value(123, 827, b"123.827");
    test_write_created_value(123, 828, b"123.828");
    test_write_created_value(123, 829, b"123.829");
}

#[test]
fn created_value_written_correctly_for_millisecond_830_to_839() {
    test_write_created_value(123, 830, b"123.830");
    test_write_created_value(123, 831, b"123.831");
    test_write_created_value(123, 832, b"123.832");
    test_write_created_value(123, 833, b"123.833");
    test_write_created_value(123, 834, b"123.834");
    test_write_created_value(123, 835, b"123.835");
    test_write_created_value(123, 836, b"123.836");
    test_write_created_value(123, 837, b"123.837");
    test_write_created_value(123, 838, b"123.838");
    test_write_created_value(123, 839, b"123.839");
}

#[test]
fn created_value_written_correctly_for_millisecond_840_to_849() {
    test_write_created_value(123, 840, b"123.840");
    test_write_created_value(123, 841, b"123.841");
    test_write_created_value(123, 842, b"123.842");
    test_write_created_value(123, 843, b"123.843");
    test_write_created_value(123, 844, b"123.844");
    test_write_created_value(123, 845, b"123.845");
    test_write_created_value(123, 846, b"123.846");
    test_write_created_value(123, 847, b"123.847");
    test_write_created_value(123, 848, b"123.848");
    test_write_created_value(123, 849, b"123.849");
}

#[test]
fn created_value_written_correctly_for_millisecond_850_to_859() {
    test_write_created_value(123, 850, b"123.850");
    test_write_created_value(123, 851, b"123.851");
    test_write_created_value(123, 852, b"123.852");
    test_write_created_value(123, 853, b"123.853");
    test_write_created_value(123, 854, b"123.854");
    test_write_created_value(123, 855, b"123.855");
    test_write_created_value(123, 856, b"123.856");
    test_write_created_value(123, 857, b"123.857");
    test_write_created_value(123, 858, b"123.858");
    test_write_created_value(123, 859, b"123.859");
}

#[test]
fn created_value_written_correctly_for_millisecond_860_to_869() {
    test_write_created_value(123, 860, b"123.860");
    test_write_created_value(123, 861, b"123.861");
    test_write_created_value(123, 862, b"123.862");
    test_write_created_value(123, 863, b"123.863");
    test_write_created_value(123, 864, b"123.864");
    test_write_created_value(123, 865, b"123.865");
    test_write_created_value(123, 866, b"123.866");
    test_write_created_value(123, 867, b"123.867");
    test_write_created_value(123, 868, b"123.868");
    test_write_created_value(123, 869, b"123.869");
}

#[test]
fn created_value_written_correctly_for_millisecond_870_to_879() {
    test_write_created_value(123, 870, b"123.870");
    test_write_created_value(123, 871, b"123.871");
    test_write_created_value(123, 872, b"123.872");
    test_write_created_value(123, 873, b"123.873");
    test_write_created_value(123, 874, b"123.874");
    test_write_created_value(123, 875, b"123.875");
    test_write_created_value(123, 876, b"123.876");
    test_write_created_value(123, 877, b"123.877");
    test_write_created_value(123, 878, b"123.878");
    test_write_created_value(123, 879, b"123.879");
}

#[test]
fn created_value_written_correctly_for_millisecond_880_to_889() {
    test_write_created_value(123, 880, b"123.880");
    test_write_created_value(123, 881, b"123.881");
    test_write_created_value(123, 882, b"123.882");
    test_write_created_value(123, 883, b"123.883");
    test_write_created_value(123, 884, b"123.884");
    test_write_created_value(123, 885, b"123.885");
    test_write_created_value(123, 886, b"123.886");
    test_write_created_value(123, 887, b"123.887");
    test_write_created_value(123, 888, b"123.888");
    test_write_created_value(123, 889, b"123.889");
}

#[test]
fn created_value_written_correctly_for_millisecond_890_to_899() {
    test_write_created_value(123, 890, b"123.890");
    test_write_created_value(123, 891, b"123.891");
    test_write_created_value(123, 892, b"123.892");
    test_write_created_value(123, 893, b"123.893");
    test_write_created_value(123, 894, b"123.894");
    test_write_created_value(123, 895, b"123.895");
    test_write_created_value(123, 896, b"123.896");
    test_write_created_value(123, 897, b"123.897");
    test_write_created_value(123, 898, b"123.898");
    test_write_created_value(123, 899, b"123.899");
}

#[test]
fn created_value_written_correctly_for_millisecond_900_to_909() {
    test_write_created_value(123, 900, b"123.900");
    test_write_created_value(123, 901, b"123.901");
    test_write_created_value(123, 902, b"123.902");
    test_write_created_value(123, 903, b"123.903");
    test_write_created_value(123, 904, b"123.904");
    test_write_created_value(123, 905, b"123.905");
    test_write_created_value(123, 906, b"123.906");
    test_write_created_value(123, 907, b"123.907");
    test_write_created_value(123, 908, b"123.908");
    test_write_created_value(123, 909, b"123.909");
}

#[test]
fn created_value_written_correctly_for_millisecond_910_to_919() {
    test_write_created_value(123, 910, b"123.910");
    test_write_created_value(123, 911, b"123.911");
    test_write_created_value(123, 912, b"123.912");
    test_write_created_value(123, 913, b"123.913");
    test_write_created_value(123, 914, b"123.914");
    test_write_created_value(123, 915, b"123.915");
    test_write_created_value(123, 916, b"123.916");
    test_write_created_value(123, 917, b"123.917");
    test_write_created_value(123, 918, b"123.918");
    test_write_created_value(123, 919, b"123.919");
}

#[test]
fn created_value_written_correctly_for_millisecond_920_to_929() {
    test_write_created_value(123, 920, b"123.920");
    test_write_created_value(123, 921, b"123.921");
    test_write_created_value(123, 922, b"123.922");
    test_write_created_value(123, 923, b"123.923");
    test_write_created_value(123, 924, b"123.924");
    test_write_created_value(123, 925, b"123.925");
    test_write_created_value(123, 926, b"123.926");
    test_write_created_value(123, 927, b"123.927");
    test_write_created_value(123, 928, b"123.928");
    test_write_created_value(123, 929, b"123.929");
}

#[test]
fn created_value_written_correctly_for_millisecond_930_to_939() {
    test_write_created_value(123, 930, b"123.930");
    test_write_created_value(123, 931, b"123.931");
    test_write_created_value(123, 932, b"123.932");
    test_write_created_value(123, 933, b"123.933");
    test_write_created_value(123, 934, b"123.934");
    test_write_created_value(123, 935, b"123.935");
    test_write_created_value(123, 936, b"123.936");
    test_write_created_value(123, 937, b"123.937");
    test_write_created_value(123, 938, b"123.938");
    test_write_created_value(123, 939, b"123.939");
}

#[test]
fn created_value_written_correctly_for_millisecond_940_to_949() {
    test_write_created_value(123, 940, b"123.940");
    test_write_created_value(123, 941, b"123.941");
    test_write_created_value(123, 942, b"123.942");
    test_write_created_value(123, 943, b"123.943");
    test_write_created_value(123, 944, b"123.944");
    test_write_created_value(123, 945, b"123.945");
    test_write_created_value(123, 946, b"123.946");
    test_write_created_value(123, 947, b"123.947");
    test_write_created_value(123, 948, b"123.948");
    test_write_created_value(123, 949, b"123.949");
}

#[test]
fn created_value_written_correctly_for_millisecond_950_to_959() {
    test_write_created_value(123, 950, b"123.950");
    test_write_created_value(123, 951, b"123.951");
    test_write_created_value(123, 952, b"123.952");
    test_write_created_value(123, 953, b"123.953");
    test_write_created_value(123, 954, b"123.954");
    test_write_created_value(123, 955, b"123.955");
    test_write_created_value(123, 956, b"123.956");
    test_write_created_value(123, 957, b"123.957");
    test_write_created_value(123, 958, b"123.958");
    test_write_created_value(123, 959, b"123.959");
}

#[test]
fn created_value_written_correctly_for_millisecond_960_to_969() {
    test_write_created_value(123, 960, b"123.960");
    test_write_created_value(123, 961, b"123.961");
    test_write_created_value(123, 962, b"123.962");
    test_write_created_value(123, 963, b"123.963");
    test_write_created_value(123, 964, b"123.964");
    test_write_created_value(123, 965, b"123.965");
    test_write_created_value(123, 966, b"123.966");
    test_write_created_value(123, 967, b"123.967");
    test_write_created_value(123, 968, b"123.968");
    test_write_created_value(123, 969, b"123.969");
}

#[test]
fn created_value_written_correctly_for_millisecond_970_to_979() {
    test_write_created_value(123, 970, b"123.970");
    test_write_created_value(123, 971, b"123.971");
    test_write_created_value(123, 972, b"123.972");
    test_write_created_value(123, 973, b"123.973");
    test_write_created_value(123, 974, b"123.974");
    test_write_created_value(123, 975, b"123.975");
    test_write_created_value(123, 976, b"123.976");
    test_write_created_value(123, 977, b"123.977");
    test_write_created_value(123, 978, b"123.978");
    test_write_created_value(123, 979, b"123.979");
}

#[test]
fn created_value_written_correctly_for_millisecond_980_to_989() {
    test_write_created_value(123, 980, b"123.980");
    test_write_created_value(123, 981, b"123.981");
    test_write_created_value(123, 982, b"123.982");
    test_write_created_value(123, 983, b"123.983");
    test_write_created_value(123, 984, b"123.984");
    test_write_created_value(123, 985, b"123.985");
    test_write_created_value(123, 986, b"123.986");
    test_write_created_value(123, 987, b"123.987");
    test_write_created_value(123, 988, b"123.988");
    test_write_created_value(123, 989, b"123.989");
}

#[test]
fn created_value_written_correctly_for_millisecond_990_to_999() {
    test_write_created_value(123, 990, b"123.990");
    test_write_created_value(123, 991, b"123.991");
    test_write_created_value(123, 992, b"123.992");
    test_write_created_value(123, 993, b"123.993");
    test_write_created_value(123, 994, b"123.994");
    test_write_created_value(123, 995, b"123.995");
    test_write_created_value(123, 996, b"123.996");
    test_write_created_value(123, 997, b"123.997");
    test_write_created_value(123, 998, b"123.998");
    test_write_created_value(123, 999, b"123.999");
}
