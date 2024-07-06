use crate::prelude::*;
use std::marker::PhantomData;

#[test]
fn unit_works() {
    let mut called = false;

    propcheck::run(|()| {
        called = true;
        true
    });

    assert!(called);
}

#[test]
#[should_panic = "[FAIL] ()"]
fn unit_fails() {
    propcheck::run(|()| false);
}

#[test]
#[should_panic = "panic message"]
fn unit_panics() {
    propcheck::run(|()| panic!("panic message"));
}

macro_rules! smoke_test_arbitrary {
    ($($works:ident/$fails:ident/$panics:ident => $ty:ty,)+) => {
        $(
            #[test]
            fn $works() {
                let mut called = false;
                propcheck::run(|_: &$ty| {
                    called = true;
                    true
                });
                assert!(called);
            }

            #[test]
            #[should_panic = "[FAIL]"]
            fn $fails() {
                propcheck::run(|_: &$ty| false);
            }

            #[test]
            #[should_panic = "panic message"]
            fn $panics() {
                propcheck::run(|_: &$ty| panic!("panic message"));
            }
        )+
    };
}

smoke_test_arbitrary! {
    bool_works/bool_fails/bool_panics => bool,
    char_works/char_fails/char_panics => char,
    u8_works/u8_fails/u8_panics => u8,
    u16_works/u16_fails/u16_panics => u16,
    u32_works/u32_fails/u32_panics => u32,
    u64_works/u64_fails/u64_panics => u64,
    u128_works/u128_fails/u128_panics => u128,
    usize_works/usize_fails/usize_panics => usize,
    f32_works/f32_fails/f32_panics => f32,
    f64_works/f64_fails/f64_panics => f64,
    option_works/option_fails/option_panics => Option<u32>,
    result_works/result_fails/result_panics => Result<u32, bool>,
    arc_works/arc_fails/arc_panics => Arc<u32>,
    box_works/box_fails/box_panics => Box<u32>,
    tuple_1_works/tuple_1_fails/tuple_1_panics => (u8,),
    tuple_2_works/tuple_2_fails/tuple_2_panics => (u8, u8),
    tuple_3_works/tuple_3_fails/tuple_3_panics => (u8, u8, u8),
    tuple_4_works/tuple_4_fails/tuple_4_panics => (u8, u8, u8, u8),
    array_0_works/array_0_fails/array_0_panics => [u8; 0],
    array_1_works/array_1_fails/array_1_panics => [u8; 1],
    array_2_works/array_2_fails/array_2_panics => [u8; 2],
    array_3_works/array_3_fails/array_3_panics => [u8; 3],
    array_4_works/array_4_fails/array_4_panics => [u8; 4],
    array_5_works/array_5_fails/array_5_panics => [u8; 5],
    array_6_works/array_6_fails/array_6_panics => [u8; 6],
    array_7_works/array_7_fails/array_7_panics => [u8; 7],
    array_8_works/array_8_fails/array_8_panics => [u8; 8],
    array_9_works/array_9_fails/array_9_panics => [u8; 9],
    array_16_works/array_16_fails/array_16_panics => [u8; 16],
    vec_works/vec_fails/vec_panics => Vec<u8>,
    string_works/string_fails/string_panics => String,
    duration_works/duration_fails/duration_panics => std::time::Duration,
    system_time_works/system_time_fails/system_time_panics => std::time::SystemTime,
    io_error_works/io_error_fails/io_error_panics => std::io::Error,
}

#[test]
#[should_panic = "panic message"]
fn panic_in_arbitrary_works() {
    let mut called = false;
    propcheck::run(|_: &PanicGenerate| {
        called = true;
        true
    });
    assert!(called);
}

#[test]
#[should_panic = "panic message"]
fn panic_in_arbitrary_fails() {
    propcheck::run(|_: &PanicGenerate| false);
}

#[test]
#[should_panic = "panic message"]
fn panic_in_arbitrary_panics() {
    propcheck::run(|_: &PanicGenerate| panic!("other message"));
}

#[test]
fn panic_in_clone_works() {
    let mut called = false;
    propcheck::run(|_: &[PanicClone; 2]| {
        called = true;
        true
    });
    assert!(called);
}

#[test]
#[should_panic = "panic message"]
fn panic_in_clone_fails() {
    propcheck::run(|_: &[PanicClone; 2]| false);
}

#[test]
#[should_panic = "other message"]
fn panic_in_clone_panics() {
    propcheck::run(|_: &[PanicClone; 2]| panic!("other message"));
}

#[test]
fn panic_in_shrink_works() {
    let mut called = false;
    propcheck::run(|_: &PanicShrink| {
        called = true;
        true
    });
    assert!(called);
}

#[test]
#[should_panic = "panic message"]
fn panic_in_shrink_fails() {
    propcheck::run(|_: &PanicShrink| false);
}

#[test]
#[should_panic = "other message"]
fn panic_in_shrink_panics() {
    propcheck::run(|_: &PanicShrink| panic!("other message"));
}

#[test]
fn panic_in_shrink_next_works() {
    let mut called = false;
    propcheck::run(|_: &PanicShrinkNext| {
        called = true;
        true
    });
    assert!(called);
}

#[test]
#[should_panic = "panic message"]
fn panic_in_shrink_next_fails() {
    propcheck::run(|_: &PanicShrinkNext| false);
}

#[test]
#[should_panic = "other message"]
fn panic_in_shrink_next_panics() {
    propcheck::run(|_: &PanicShrinkNext| panic!("other message"));
}

//  #     #
//  #     # ##### # #      # ##### # ######  ####
//  #     #   #   # #      #   #   # #      #
//  #     #   #   # #      #   #   # #####   ####
//  #     #   #   # #      #   #   # #           #
//  #     #   #   # #      #   #   # #      #    #
//   #####    #   # ###### #   #   # ######  ####

#[derive(Debug)]
struct PanicGenerate;
#[derive(Debug)]
struct PanicClone;
#[derive(Debug)]
struct PanicShrink;
#[derive(Debug)]
struct PanicShrinkNext;

struct EmptyShrinker<T>(PhantomData<T>);
impl<T: super::Arbitrary> super::Shrinker for EmptyShrinker<T> {
    type Item = T;
    fn next(&mut self) -> Option<&Self::Item> {
        None
    }
}

struct PanicNextShrinker<T>(PhantomData<T>);
impl<T: super::Arbitrary> super::Shrinker for PanicNextShrinker<T> {
    type Item = T;
    fn next(&mut self) -> Option<&Self::Item> {
        panic!("panic message")
    }
}

impl super::Arbitrary for PanicGenerate {
    type Shrinker = EmptyShrinker<Self>;

    fn arbitrary() -> Self {
        panic!("panic message")
    }

    fn clone(&self) -> Self {
        Self
    }

    fn shrink(&self) -> Self::Shrinker {
        EmptyShrinker(PhantomData)
    }
}

impl super::Arbitrary for PanicClone {
    type Shrinker = EmptyShrinker<Self>;

    fn arbitrary() -> Self {
        Self
    }

    fn clone(&self) -> Self {
        panic!("panic message")
    }

    fn shrink(&self) -> Self::Shrinker {
        EmptyShrinker(PhantomData)
    }
}

impl super::Arbitrary for PanicShrink {
    type Shrinker = EmptyShrinker<Self>;

    fn arbitrary() -> Self {
        Self
    }

    fn clone(&self) -> Self {
        Self
    }

    fn shrink(&self) -> Self::Shrinker {
        panic!("panic message")
    }
}

impl super::Arbitrary for PanicShrinkNext {
    type Shrinker = PanicNextShrinker<Self>;

    fn arbitrary() -> Self {
        Self
    }

    fn clone(&self) -> Self {
        Self
    }

    fn shrink(&self) -> Self::Shrinker {
        PanicNextShrinker(PhantomData)
    }
}
