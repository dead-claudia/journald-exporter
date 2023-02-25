//! This module exists so sign extensions, truncations, and such are easier to spot.

macro_rules! define_transforms {
    ($($name:ident($from:ty) -> $to:ty),+ $(,)?) => {
        $(
            #[cfg_attr(not(test), allow(unused))]
            pub const fn $name(value: $from) -> $to {
                // That's the point of this function, to move conversions out to where I can be a
                // lot more explicit.
                #![allow(clippy::as_conversions)]

                value as $to
            }
        )+
    };
}

define_transforms! {
    zero_extend_u8_i32(u8) -> i32,
    zero_extend_u8_c_int(u8) -> i32,
    zero_extend_u8_u32(u8) -> u32,
    zero_extend_u8_usize(u8) -> usize,
    zero_extend_u16_usize(u16) -> usize,
    zero_extend_u32_usize(u32) -> usize,
    zero_extend_usize_u64(usize) -> u64,

    reinterpret_i32_c_uint(i32) -> libc::c_uint,
    reinterpret_i64_u64(i64) -> u64,
    reinterpret_u32_i32(u32) -> i32,
    reinterpret_i32_u32(i32) -> u32,
    reinterpret_u128_i128(u128) -> i128,
    reinterpret_usize_isize(usize) -> isize,

    truncate_c_long_i32(libc::c_long) -> i32,
    truncate_u16_u8(u16) -> u8,
    truncate_i32_u8(i32) -> u8,
    truncate_u32_u8(u32) -> u8,
    truncate_u64_u8(u64) -> u8,
    truncate_u128_u64(u128) -> u64,
    truncate_u128_i32(u128) -> i32,
    truncate_usize_c_int(usize) -> libc::c_int,
    truncate_usize_u8(usize) -> u8,
    truncate_usize_u16(usize) -> u16,
    truncate_usize_u32(usize) -> u32,
}
