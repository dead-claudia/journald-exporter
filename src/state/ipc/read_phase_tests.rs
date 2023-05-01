#![allow(clippy::bool_assert_comparison)]

use super::common::*;

#[test]
fn read_iter_phase_32_next_offset_0_returns_zero_values() {
    let mut iter1 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut iter2 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut iter3 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_0_returns_1_byte_values() {
    let mut iter1 = ReadIter::new(&[0x12, 0x00, 0x00, 0x00]);
    let mut iter2 = ReadIter::new(&[0x42, 0x00, 0x00, 0x00]);
    let mut iter3 = ReadIter::new(&[0x12, 0x00, 0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x12));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x42));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x12));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_0_returns_2_byte_values() {
    let mut iter1 = ReadIter::new(&[0x12, 0x34, 0x00, 0x00]);
    let mut iter2 = ReadIter::new(&[0x9A, 0xBC, 0x00, 0x00]);
    let mut iter3 = ReadIter::new(&[0x12, 0x34, 0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x00003412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x0000BC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x00003412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_0_returns_3_byte_values() {
    let mut iter1 = ReadIter::new(&[0x12, 0x34, 0x56, 0x00]);
    let mut iter2 = ReadIter::new(&[0x9A, 0xBC, 0xDE, 0x00]);
    let mut iter3 = ReadIter::new(&[0x12, 0x34, 0x56, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x00563412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x00DEBC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x00563412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_0_returns_4_byte_values() {
    let mut iter1 = ReadIter::new(&[0x12, 0x34, 0x56, 0x78]);
    let mut iter2 = ReadIter::new(&[0x9A, 0xBC, 0xDE, 0xF0]);
    let mut iter3 = ReadIter::new(&[0x12, 0x34, 0x56, 0x78]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x78563412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0xF0DEBC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x78563412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_1_returns_zero_values() {
    let mut iter0 = ReadIter::new(&[0x00]);
    let mut iter1 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut iter2 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut iter3 = ReadIter::new(&[0x00, 0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_1_returns_1_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12]);
    let mut iter1 = ReadIter::new(&[0x00, 0x00, 0x00, 0x42]);
    let mut iter2 = ReadIter::new(&[0x00, 0x00, 0x00, 0x12]);
    let mut iter3 = ReadIter::new(&[0x00, 0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x12));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x42));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x12));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_1_returns_2_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12]);
    let mut iter1 = ReadIter::new(&[0x34, 0x00, 0x00, 0x9A]);
    let mut iter2 = ReadIter::new(&[0xBC, 0x00, 0x00, 0x12]);
    let mut iter3 = ReadIter::new(&[0x34, 0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x00003412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x0000BC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x00003412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_1_returns_3_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12]);
    let mut iter1 = ReadIter::new(&[0x34, 0x56, 0x00, 0x9A]);
    let mut iter2 = ReadIter::new(&[0xBC, 0xDE, 0x00, 0x12]);
    let mut iter3 = ReadIter::new(&[0x34, 0x56, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x00563412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x00DEBC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x00563412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_1_returns_4_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12]);
    let mut iter1 = ReadIter::new(&[0x34, 0x56, 0x78, 0x9A]);
    let mut iter2 = ReadIter::new(&[0xBC, 0xDE, 0xF0, 0x12]);
    let mut iter3 = ReadIter::new(&[0x34, 0x56, 0x78]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x78563412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0xF0DEBC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x78563412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_2_returns_zero_values() {
    let mut iter0 = ReadIter::new(&[0x00, 0x00]);
    let mut iter1 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut iter2 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut iter3 = ReadIter::new(&[0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_2_returns_1_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12, 0x00]);
    let mut iter1 = ReadIter::new(&[0x00, 0x00, 0x42, 0x00]);
    let mut iter2 = ReadIter::new(&[0x00, 0x00, 0x12, 0x00]);
    let mut iter3 = ReadIter::new(&[0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x12));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x42));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x12));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_2_returns_2_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12, 0x34]);
    let mut iter1 = ReadIter::new(&[0x00, 0x00, 0x9A, 0xBC]);
    let mut iter2 = ReadIter::new(&[0x00, 0x00, 0x12, 0x34]);
    let mut iter3 = ReadIter::new(&[0x00, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x00003412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x0000BC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x00003412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_2_returns_3_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12, 0x34]);
    let mut iter1 = ReadIter::new(&[0x56, 0x00, 0x9A, 0xBC]);
    let mut iter2 = ReadIter::new(&[0xDE, 0x00, 0x12, 0x34]);
    let mut iter3 = ReadIter::new(&[0x56, 0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x00563412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x00DEBC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x00563412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_2_returns_4_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12, 0x34]);
    let mut iter1 = ReadIter::new(&[0x56, 0x78, 0x9A, 0xBC]);
    let mut iter2 = ReadIter::new(&[0xDE, 0xF0, 0x12, 0x34]);
    let mut iter3 = ReadIter::new(&[0x56, 0x78]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x78563412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0xF0DEBC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x78563412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_3_returns_zero_values() {
    let mut iter0 = ReadIter::new(&[0x00, 0x00, 0x00]);
    let mut iter1 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut iter2 = ReadIter::new(&[0x00, 0x00, 0x00, 0x00]);
    let mut iter3 = ReadIter::new(&[0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_3_returns_1_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12, 0x00, 0x00]);
    let mut iter1 = ReadIter::new(&[0x00, 0x42, 0x00, 0x00]);
    let mut iter2 = ReadIter::new(&[0x00, 0x12, 0x00, 0x00]);
    let mut iter3 = ReadIter::new(&[0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x12));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x42));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x12));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_3_returns_2_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12, 0x34, 0x00]);
    let mut iter1 = ReadIter::new(&[0x00, 0x9A, 0xBC, 0x00]);
    let mut iter2 = ReadIter::new(&[0x00, 0x12, 0x34, 0x00]);
    let mut iter3 = ReadIter::new(&[0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x00003412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x0000BC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x00003412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_3_returns_3_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12, 0x34, 0x56]);
    let mut iter1 = ReadIter::new(&[0x00, 0x9A, 0xBC, 0xDE]);
    let mut iter2 = ReadIter::new(&[0x00, 0x12, 0x34, 0x56]);
    let mut iter3 = ReadIter::new(&[0x00]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x00563412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0x00DEBC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x00563412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}

#[test]
fn read_iter_phase_32_next_offset_3_returns_4_byte_values() {
    let mut iter0 = ReadIter::new(&[0x12, 0x34, 0x56]);
    let mut iter1 = ReadIter::new(&[0x78, 0x9A, 0xBC, 0xDE]);
    let mut iter2 = ReadIter::new(&[0xF0, 0x12, 0x34, 0x56]);
    let mut iter3 = ReadIter::new(&[0x78]);
    let mut phase = ReadPhase::new();
    assert_eq!(iter0.phase_next_32(&mut phase), None);
    assert_eq!(iter1.phase_next_32(&mut phase), Some(0x78563412));
    assert_eq!(iter1.phase_next_32(&mut phase), None);
    assert_eq!(iter2.phase_next_32(&mut phase), Some(0xF0DEBC9A));
    assert_eq!(iter2.phase_next_32(&mut phase), None);
    assert_eq!(iter3.phase_next_32(&mut phase), Some(0x78563412));
    assert_eq!(iter3.phase_next_32(&mut phase), None);
}
