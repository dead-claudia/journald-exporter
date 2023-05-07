//! This module exists so sign extensions, truncations, and such are easier to spot.

// That's the point of this function, to move conversions out to where I can be a
// lot more explicit.
#![allow(clippy::as_conversions)]
// Allow test-only transforms
#![cfg_attr(not(test), allow(unused))]

pub const fn zero_extend_u8_i32(value: u8) -> i32 {
    value as i32
}

pub const fn zero_extend_u8_u32(value: u8) -> u32 {
    value as u32
}

pub const fn zero_extend_u8_usize(value: u8) -> usize {
    value as usize
}

pub const fn zero_extend_u16_u64(value: u16) -> u64 {
    value as u64
}

pub const fn zero_extend_u16_usize(value: u16) -> usize {
    value as usize
}

pub const fn zero_extend_u32_usize(value: u32) -> usize {
    value as usize
}

pub const fn zero_extend_c_int_usize(value: libc::c_int) -> usize {
    value as usize
}

pub const fn zero_extend_usize_u64(value: usize) -> u64 {
    value as u64
}

pub const fn sign_extend_c_int_isize(value: libc::c_int) -> isize {
    value as isize
}

pub const fn reinterpret_i32_c_uint(value: i32) -> libc::c_uint {
    value as libc::c_uint
}

pub const fn reinterpret_i64_u64(value: i64) -> u64 {
    value as u64
}

pub const fn reinterpret_u32_i32(value: u32) -> i32 {
    value as i32
}

pub const fn reinterpret_i32_u32(value: i32) -> u32 {
    value as u32
}

pub const fn reinterpret_u32_c_int(value: u32) -> libc::c_int {
    value as libc::c_int
}

pub const fn reinterpret_usize_isize(value: usize) -> isize {
    value as isize
}

pub const fn reinterpret_isize_usize(value: isize) -> usize {
    value as usize
}

pub const fn truncate_c_long_i32(value: libc::c_long) -> i32 {
    value as i32
}

pub const fn truncate_i32_u8(value: i32) -> u8 {
    value as u8
}

pub const fn truncate_i64_u8(value: i64) -> u8 {
    value as u8
}

pub const fn truncate_u32_u8(value: u32) -> u8 {
    value as u8
}

pub const fn truncate_u64_u8(value: u64) -> u8 {
    value as u8
}

pub const fn truncate_u128_u64(value: u128) -> u64 {
    value as u64
}

pub const fn truncate_u128_i32(value: u128) -> i32 {
    value as i32
}

pub const fn truncate_usize_c_int(value: usize) -> libc::c_int {
    value as libc::c_int
}

pub const fn truncate_usize_u8(value: usize) -> u8 {
    value as u8
}

pub const fn truncate_usize_u16(value: usize) -> u16 {
    value as u16
}

pub const fn truncate_usize_u32(value: usize) -> u32 {
    value as u32
}

pub const fn truncate_u64_usize(value: u64) -> usize {
    value as usize
}
