//! Unlike the `quickcheck` crate, this doesn't hardly allocate at all.

use rand::Rng;
use std::env::VarError;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

fn max_dynamic_gen_size() -> usize {
    static VALUE: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *VALUE.get_or_init(|| {
        match std::env::var("QUICKCHECK_DYNAMIC_GEN_MAX") {
            Ok(v) => {
                if let Ok(result) = v.parse() {
                    return result;
                }
            }
            Err(VarError::NotPresent) => return 100,
            Err(_) => {}
        }

        panic!("Invalid value for QUICKCHECK_DYNAMIC_GEN_MAX")
    })
}

pub fn random_entry<T: Copy>(options: &[T]) -> T {
    let index = rand::thread_rng().gen_range(0..options.len());
    // SAFETY: it's always present.
    unsafe { *options.get_unchecked(index) }
}

pub fn random_entry_or_gen<T: Copy>(options: &[T]) -> T
where
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    let mut gen = rand::thread_rng();
    options
        .get(gen.gen_range(0..options.len() * 10))
        .copied()
        .unwrap_or_else(|| gen.gen())
}

pub trait Shrinker {
    type Item: Arbitrary;
    fn next(&mut self) -> Option<&Self::Item>;
}

/// `Arbitrary` describes types whose values can be randomly generated and
/// shrunk.
///
/// Aside from shrinking, `Arbitrary` is different from typical RNGs in that
/// it respects `Gen::size()` for controlling how much memory a particular
/// value uses, for practical purposes. For example, `Vec::arbitrary()`
/// respects `Gen::size()` to decide the maximum `len()` of the vector.
/// This behavior is necessary due to practical speed and size limitations.
/// Conversely, `i32::arbitrary()` ignores `size()` since all `i32` values
/// require `O(1)` memory and operations between `i32`s require `O(1)` time
/// (with the exception of exponentiation).
///
/// Additionally, all types that implement `Arbitrary` must also implement
/// `Clone`.
pub trait Arbitrary: 'static {
    type Shrinker: Shrinker<Item = Self>;

    /// Return an arbitrary value.
    ///
    /// Implementations should respect `Gen::size()` when decisions about how
    /// big a particular value should be. Implementations should generally
    /// defer to other `Arbitrary` implementations to generate other random
    /// values when necessary. The `Gen` type also offers a few RNG helper
    /// routines.
    fn arbitrary() -> Self;

    /// Clone the value. Usually, this is `Clone::clone`, but it may need modified for certain
    /// arbitrary values.
    fn clone(&self) -> Self;

    /// Return an iterator of values that are smaller than itself.
    ///
    /// The way in which a value is "smaller" is implementation defined. In
    /// some cases, the interpretation is obvious: shrinking an integer should
    /// produce integers smaller than itself. Others are more complex, for
    /// example, shrinking a `Vec` should both shrink its size and shrink its
    /// component values.
    ///
    /// The iterator returned should be bounded to some reasonable size.
    ///
    /// It is always correct to return an empty iterator, and indeed, this
    /// is the default implementation. The downside of this approach is that
    /// witnesses to failures in properties will be more inscrutable.
    fn shrink(&self) -> Self::Shrinker;
}

pub struct UnitShrinker;
impl Shrinker for UnitShrinker {
    type Item = ();
    fn next(&mut self) -> Option<&Self::Item> {
        None
    }
}

impl Arbitrary for () {
    type Shrinker = UnitShrinker;
    fn arbitrary() -> Self {}
    fn clone(&self) -> Self {}
    fn shrink(&self) -> Self::Shrinker {
        UnitShrinker
    }
}

pub struct BoolShrinker(bool);

impl Shrinker for BoolShrinker {
    type Item = bool;
    fn next(&mut self) -> Option<&Self::Item> {
        std::mem::replace(&mut self.0, false).then_some(&self.0)
    }
}

impl Arbitrary for bool {
    type Shrinker = BoolShrinker;
    fn arbitrary() -> Self {
        rand::random()
    }
    fn clone(&self) -> Self {
        *self
    }
    fn shrink(&self) -> Self::Shrinker {
        BoolShrinker(*self)
    }
}

pub struct OptionShrinker<A: Arbitrary> {
    ready: bool,
    state: Option<A::Shrinker>,
    stored: Option<A>,
}

impl<A: Arbitrary> Shrinker for OptionShrinker<A> {
    type Item = Option<A>;
    fn next(&mut self) -> Option<&Self::Item> {
        let state = self.state.as_mut()?;
        if std::mem::replace(&mut self.ready, true) {
            self.stored = Some(state.next()?.clone());
        }
        Some(&self.stored)
    }
}

impl<A: Arbitrary> Arbitrary for Option<A> {
    type Shrinker = OptionShrinker<A>;
    fn arbitrary() -> Self {
        if rand::random() {
            None
        } else {
            Some(<A>::arbitrary())
        }
    }
    fn clone(&self) -> Self {
        Some(self.as_ref()?.clone())
    }
    fn shrink(&self) -> Self::Shrinker {
        OptionShrinker {
            ready: false,
            state: self.as_ref().map(|v| v.shrink()),
            stored: None,
        }
    }
}

macro_rules! wrapper_shrinker {
    ($shrinker:ident; $wrapper:ident<$param:ident>; $clone:item) => {
        pub struct $shrinker<$param: Shrinker> {
            state: $param,
            stored: Option<$wrapper<<$param>::Item>>,
        }

        impl<$param: Shrinker> Shrinker for $shrinker<$param>
        where $param::Item: Arbitrary {
            type Item = $wrapper<<$param>::Item>;
            fn next(&mut self) -> Option<&Self::Item> {
                Some(
                    self.stored
                        .insert(Self::Item::new(self.state.next()?.clone())),
                )
            }
        }

        /// Note: internal clones bump the reference count. They do not do a deep clone
        impl<$param: Arbitrary> Arbitrary for $wrapper<$param> {
            type Shrinker = $shrinker<<$param>::Shrinker>;
            fn arbitrary() -> Self {
                Self::new(<$param>::arbitrary())
            }
            $clone
            fn shrink(&self) -> Self::Shrinker {
                $shrinker {
                    state: <$param>::shrink(self),
                    stored: None,
                }
            }
        }
    };
}

wrapper_shrinker! {
    ArcShrinker; Arc<A>;
    fn clone(&self) -> Self {
        Clone::clone(self)
    }
}

wrapper_shrinker! {
    BoxShrinker; Box<A>;
    fn clone(&self) -> Self {
        Self::new(<A>::clone(self))
    }
}

pub struct ResultShrinker<T: Arbitrary, E: Arbitrary> {
    state: Result<T::Shrinker, E::Shrinker>,
    stored: Option<Result<T, E>>,
}

impl<T: Arbitrary, E: Arbitrary> Shrinker for ResultShrinker<T, E> {
    type Item = Result<T, E>;
    fn next(&mut self) -> Option<&Self::Item> {
        let next = match &mut self.state {
            Ok(v) => Ok(v.next()?.clone()),
            Err(v) => Err(v.next()?.clone()),
        };
        Some(self.stored.insert(next))
    }
}

impl<T: Arbitrary, E: Arbitrary> Arbitrary for Result<T, E> {
    type Shrinker = ResultShrinker<T, E>;

    fn arbitrary() -> Self {
        if rand::random() {
            Ok(<T>::arbitrary())
        } else {
            Err(<E>::arbitrary())
        }
    }
    fn clone(&self) -> Self {
        match self {
            Ok(v) => Ok(v.clone()),
            Err(e) => Err(e.clone()),
        }
    }
    fn shrink(&self) -> Self::Shrinker {
        ResultShrinker {
            state: match self {
                Ok(v) => Ok(v.shrink()),
                Err(v) => Err(v.shrink()),
            },
            stored: None,
        }
    }
}

macro_rules! arbitrary_tuple {
    ($shrinker_ty:ident ; $shrinker_union_ty:ident ; $($ty_param:ident => $idx:tt),+) => {
        pub struct $shrinker_ty<$($ty_param: Arbitrary),+> {
            state: ($($ty_param::Shrinker,)+),
            template: ($($ty_param,)+),
            result: ($($ty_param,)+),
            index: usize,
        }

        impl<$($ty_param: Arbitrary),+> Shrinker for $shrinker_ty<$($ty_param),+> {
            type Item = ($($ty_param,)+);
            fn next(&mut self) -> Option<&Self::Item> {
                loop {
                    match self.index {
                        $($idx => {
                            if let Some(v) = self.state.$idx.next() {
                                self.result.$idx = v.clone();
                                return Some(&self.result);
                            } else {
                                self.result.$idx = self.template.$idx.clone();
                                self.index = $idx + 1;
                            }
                        })+
                        _ => return None,
                    }
                }
            }
        }

        impl<$($ty_param: Arbitrary),+> Arbitrary for ($($ty_param,)+) {
            type Shrinker = $shrinker_ty<$($ty_param),+>;
            fn arbitrary() -> Self {
                ($(<$ty_param>::arbitrary(),)+)
            }
            fn clone(&self) -> Self {
                ($(self.$idx.clone(),)+)
            }
            fn shrink(&self) -> Self::Shrinker {
                $shrinker_ty {
                    state: ($(self.$idx.shrink(),)+),
                    template: self.clone(),
                    result: self.clone(),
                    index: 0,
                }
            }
        }
    }
}

arbitrary_tuple!(Tuple1Shrinker; Tuple1ShrinkerUnion; A => 0);
arbitrary_tuple!(Tuple2Shrinker; Tuple2ShrinkerUnion; A => 0, B => 1);
arbitrary_tuple!(Tuple3Shrinker; Tuple3ShrinkerUnion; A => 0, B => 1, C => 2);
arbitrary_tuple!(Tuple4Shrinker; Tuple4ShrinkerUnion; A => 0, B => 1, C => 2, D => 3);

pub struct ArrayShrinkIterator<const N: usize, A: Arbitrary> {
    state: Option<A::Shrinker>,
    template: Option<A>,
    result: [A; N],
    index: usize,
}

impl<const N: usize, A: Arbitrary> Shrinker for ArrayShrinkIterator<N, A> {
    type Item = [A; N];
    fn next(&mut self) -> Option<&Self::Item> {
        if N == 0 {
            return None;
        }

        match &mut self.state {
            None => None,
            Some(state) => loop {
                let current = self.index;

                if let Some(next) = state.next() {
                    self.result[self.index] = next.clone();
                    return Some(&self.result);
                }

                let next = current + 1;
                self.index = next;

                if next == N {
                    self.state = None;
                    return None;
                }

                self.result[current] = self.template.clone().unwrap();
                *state = self.result[next].shrink();
            },
        }
    }
}

impl<const N: usize, A: Arbitrary> Arbitrary for [A; N] {
    type Shrinker = ArrayShrinkIterator<N, A>;

    fn arbitrary() -> Self {
        std::array::from_fn(|_| <A>::arbitrary())
    }

    fn clone(&self) -> Self {
        self.each_ref().map(<A>::clone)
    }

    // This can likely be made a lot more efficient by using a binary adder, but it's good
    // enough for now.
    fn shrink(&self) -> Self::Shrinker {
        ArrayShrinkIterator {
            state: self.first().map(<A>::shrink),
            template: self.first().map(<A>::clone),
            result: self.clone(),
            index: 0,
        }
    }
}

impl<A: Arbitrary> Arbitrary for Vec<A> {
    type Shrinker = VecShrinker<A>;

    fn arbitrary() -> Vec<A> {
        let size = rand::thread_rng().gen_range(0..max_dynamic_gen_size());
        Self::from_iter((0..size).map(|_| <A>::arbitrary()))
    }
    fn clone(&self) -> Self {
        Self::from_iter(self.iter().map(<A>::clone))
    }
    fn shrink(&self) -> Self::Shrinker {
        VecShrinker::new(self.clone())
    }
}

///Iterator which returns successive attempts to shrink the vector `seed`
pub struct VecShrinker<A: Arbitrary> {
    seed: Vec<A>,
    /// How much which is removed when trying with smaller vectors
    size: usize,
    /// The end of the removed elements
    offset: usize,
    /// The shrinker for the element at `offset` once shrinking of individual
    /// elements are attempted
    element_shrinker: Option<A::Shrinker>,
    result: Vec<A>,
}

impl<A: Arbitrary> VecShrinker<A> {
    fn new(seed: Vec<A>) -> Self {
        let size = seed.len();
        Self {
            size,
            offset: size,
            element_shrinker: seed.first().map(|s| s.shrink()),
            result: Vec::new(),
            seed,
        }
    }
}

impl<A: Arbitrary> Shrinker for VecShrinker<A> {
    type Item = Vec<A>;
    fn next(&mut self) -> Option<&Vec<A>> {
        let seed = &*self.seed;
        let mut size = self.size;
        let mut offset = self.offset;
        let len = seed.len();

        // Try with an empty vector first
        if size == len {
            size /= 2;
            self.offset = size / 2;
            self.size = size / 2;
            self.result.clear();
            return Some(&self.result);
        }

        let (drop_size, insert_item) = if size != 0 {
            // Generate a smaller vector by removing the elements between
            // (offset - size) and offset

            (size, None)
        } else {
            // A smaller vector did not work so try to shrink each element of
            // the vector instead Reuse `offset` as the index determining which
            // element to shrink

            // The first element shrinker is already created so skip the first
            // offset (self.offset == 0 only on first entry to this part of the
            // iterator)
            if offset == 0 {
                offset = 1;
            }

            // Get the next shrunk element if any. `offset` points to the index after the returned
            // element after the loop terminates.
            loop {
                if let Some(shrinker) = &mut self.element_shrinker {
                    if let Some(e) = shrinker.next() {
                        break (1, Some(e));
                    }
                }

                if let Some(e) = seed.get(offset) {
                    self.element_shrinker = Some(e.shrink());
                    offset += 1;
                } else {
                    self.offset = offset;
                    self.size = size;
                    return None;
                }
            }
        };

        self.result.clear();
        self.result
            .reserve(len - drop_size + usize::from(insert_item.is_some()));

        let hole_start = offset - drop_size;
        let hole_end = offset;

        fn extend_in_place<A: Arbitrary>(result: &mut Vec<A>, source: &[A]) {
            let mut len = result.len();

            debug_assert!(
                result.capacity() - len >= source.len(),
                "spare={}, source={}",
                result.capacity() - len,
                source.len()
            );

            // SAFETY: It updates the length each time it inserts, to ensure previous entries are
            // dropped if subsequent clones panic. And for the memory invariant, that's asserted
            // in the above `debug_assert!`.
            unsafe {
                let mut spare = result.as_mut_ptr();
                for i in source {
                    len += 1;
                    *spare = i.clone();
                    result.set_len(len);
                    spare = spare.add(1)
                }
            }
        }

        fn push_in_place<A: Arbitrary>(result: &mut Vec<A>, source: &A) {
            debug_assert!(result.capacity() - result.len() >= 1);

            // SAFETY: See `extend_in_place` comment. Same applies here, including with the
            // `debug_assert!` above.
            unsafe {
                *result.as_mut_ptr() = source.clone();
                result.set_len(result.len() + 1);
            }
        }

        extend_in_place(&mut self.result, &seed[..hole_start]);

        if let Some(item) = insert_item {
            push_in_place(&mut self.result, item);
        } else {
            offset += size;

            // Try to reduce the amount removed from the vector once all
            // previous sizes tried
            if offset > len {
                size /= 2;
                offset = size;
            }
        }

        extend_in_place(&mut self.result, &seed[hole_end..]);

        self.offset = offset;
        self.size = size;

        Some(&self.result)
    }
}

#[allow(clippy::as_conversions)]
#[cfg(target_pointer_width = "32")]
static PROBLEM_VALUES_SMALL: &[usize] = &[
    0,
    1,
    i8::MAX as _,
    u8::MAX as _,
    i16::MAX as _,
    u16::MAX as _,
    i32::MAX as _,
    u32::MAX as _,
];

#[allow(clippy::as_conversions)]
#[cfg(target_pointer_width = "64")]
static PROBLEM_VALUES_SMALL: [usize; 10] = [
    0,
    1,
    i8::MAX as _,
    u8::MAX as _,
    i16::MAX as _,
    u16::MAX as _,
    i32::MAX as _,
    u32::MAX as _,
    i64::MAX as _,
    u64::MAX as _,
];

#[allow(clippy::as_conversions)]
static PROBLEM_VALUES_LARGE: [u128; 12] = [
    0,
    1,
    i8::MAX as _,
    u8::MAX as _,
    i16::MAX as _,
    u16::MAX as _,
    i32::MAX as _,
    u32::MAX as _,
    i64::MAX as _,
    u64::MAX as _,
    i128::MAX as _,
    u128::MAX as _,
];

macro_rules! int_arbitrary {
    ($name:ident, $ty:ty, $problem_source:expr, $problem_limit:expr) => {
        pub struct $name {
            x: $ty,
            i: $ty,
            r: $ty,
        }

        impl $name {
            fn new(x: $ty) -> Self {
                Self { x, i: x, r: 0 }
            }
        }

        impl Shrinker for $name {
            type Item = $ty;
            fn next(&mut self) -> Option<&Self::Item> {
                let i = self.i;
                if i != 0 {
                    self.i = i / 2;
                    self.r = self.x - i;
                    Some(&self.r)
                } else {
                    None
                }
            }
        }

        #[allow(clippy::as_conversions)]
        impl Arbitrary for $ty {
            type Shrinker = $name;
            fn arbitrary() -> Self {
                random_entry_or_gen(&$problem_source[..$problem_limit]) as _
            }
            fn clone(&self) -> Self {
                *self
            }
            fn shrink(&self) -> Self::Shrinker {
                $name::new(*self)
            }
        }
    };
}

macro_rules! transmute_arbitrary {
    ($name:ident, $ty:ty, $source:ty) => {
        transmute_arbitrary! {
            $name, $ty, $source;
            // SAFETY: The whole point of this is to provide a transmuted view. Safety is on
            // the caller to ensure such a transmute is sound generally.
            unsafe { std::mem::transmute(<$source>::arbitrary()) }
        }
    };

    ($name:ident, $ty:ty, $source:ty; $($factory:stmt);+) => {
        pub struct $name(<$source as Arbitrary>::Shrinker);

        impl Shrinker for $name {
            type Item = $ty;
            #[allow(clippy::as_conversions)]
            fn next(&mut self) -> Option<&Self::Item> {
                // SAFETY: The whole point of this is to provide a transmuted view. Safety is on
                // the caller to ensure such a transmute is sound generally.
                unsafe { std::mem::transmute(self.0.next()) }
            }
        }

        #[allow(clippy::as_conversions)]
        impl Arbitrary for $ty {
            type Shrinker = $name;
            fn arbitrary() -> Self {
                $($factory)+
            }
            fn clone(&self) -> Self {
                *self
            }
            fn shrink(&self) -> Self::Shrinker {
                // SAFETY: The whole point of this is to provide a transmuted view. Safety is on
                // the caller to ensure such a transmute is sound generally.
                $name(unsafe { std::mem::transmute::<_, $source>(*self) }.shrink())
            }
        }
    };
}

int_arbitrary!(U8Shrinker, u8, PROBLEM_VALUES_SMALL, 4);
int_arbitrary!(U16Shrinker, u16, PROBLEM_VALUES_SMALL, 6);
int_arbitrary!(U32Shrinker, u32, PROBLEM_VALUES_SMALL, 8);
#[cfg(target_pointer_width = "32")]
int_arbitrary!(U64Shrinker, u64, PROBLEM_VALUES_64);
#[cfg(target_pointer_width = "64")]
int_arbitrary!(U64Shrinker, u64, PROBLEM_VALUES_SMALL, 10);
int_arbitrary!(U128Shrinker, u128, PROBLEM_VALUES_LARGE, 12);

#[cfg(target_pointer_width = "32")]
transmute_arbitrary!(UsizeShrinker, usize, u32);
#[cfg(target_pointer_width = "64")]
transmute_arbitrary!(UsizeShrinker, usize, u64);
transmute_arbitrary!(I8Shrinker, i8, u8);
transmute_arbitrary!(I16Shrinker, i16, u16);
transmute_arbitrary!(I32Shrinker, i32, u32);
transmute_arbitrary!(I64Shrinker, i64, u64);
transmute_arbitrary!(I128Shrinker, i128, u128);
transmute_arbitrary!(IsizeShrinker, isize, usize);

// Inspired by `rand`'s `impl Standard for Distribution<Char>` implementation. What they do is
// remove the surrogate range from the list of possible results, and just convert to and from that
// representation.
const CHAR_GAP_OFFSET: u32 = 0xD800;
const CHAR_GAP_SIZE: u32 = 0xDFFF - 0xD800 + 1;

pub struct CharShrinker {
    inner: U32Shrinker,
    current: char,
}

impl Shrinker for CharShrinker {
    type Item = char;
    fn next(&mut self) -> Option<&char> {
        let mut next = *self.inner.next()?;
        if next > CHAR_GAP_OFFSET {
            next += CHAR_GAP_SIZE;
        }
        // SAFETY: `next` is within `char` range, and the above condition ensures it's never a
        // surrogate code point.
        self.current = unsafe { char::from_u32_unchecked(next) };
        Some(&self.current)
    }
}

impl Arbitrary for char {
    type Shrinker = CharShrinker;
    fn arbitrary() -> char {
        static COMMON_PL_CHARS: &[char] = &[
            ' ', ' ', ' ', '\t', '\n', '~', '`', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')',
            '_', '-', '=', '+', '[', ']', '{', '}', ':', ';', '\'', '"', '\\', '|', ',', '<', '>',
            '.', '/', '?', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
        ];

        static TRICKY_UNICODE_1: &[char] = &[
            '\u{0149}', // a deprecated character
            '\u{fff0}', // some of "Other, format" category:
            '\u{fff1}',
            '\u{fff2}',
            '\u{fff3}',
            '\u{fff4}',
            '\u{fff5}',
            '\u{fff6}',
            '\u{fff7}',
            '\u{fff8}',
            '\u{fff9}',
            '\u{fffA}',
            '\u{fffB}',
            '\u{fffC}',
            '\u{fffD}',
            '\u{fffE}',
            '\u{fffF}',
            '\u{0600}',
            '\u{0601}',
            '\u{0602}',
            '\u{0603}',
            '\u{0604}',
            '\u{0605}',
            '\u{061C}',
            '\u{06DD}',
            '\u{070F}',
            '\u{180E}',
            '\u{110BD}',
            '\u{1D173}',
            '\u{e0001}', // tag
            '\u{e0020}', //  tag space
            '\u{e000}',
            '\u{e001}',
            '\u{ef8ff}', // private use
            '\u{f0000}',
            '\u{ffffd}',
            '\u{ffffe}',
            '\u{fffff}',
            '\u{100000}',
            '\u{10FFFD}',
            '\u{10FFFE}',
            '\u{10FFFF}',
            // "Other, surrogate" characters are so that very special
            // that they are not even allowed in safe Rust,
            //so omitted here
            '\u{3000}', // ideographic space
            '\u{1680}',
            // other space characters are already covered by two next
            // branches
        ];

        let mut rng = rand::thread_rng();
        match rng.gen_range(0..100) {
            // ASCII + some control characters
            0..=49 => rng.gen_range('\0'..'\u{00B0}'),
            // Unicode BMP characters, including possible surrogates
            50..=59 => rng.gen_range('\0'..'\u{10000}'),
            // Characters often used in programming languages
            60..=84 => random_entry(COMMON_PL_CHARS),
            85..=89 => random_entry(TRICKY_UNICODE_1),
            90..=94 => rng.gen_range('\u{2000}'..'\u{2070}'),
            _ => rng.gen(),
        }
    }
    fn clone(&self) -> Self {
        *self
    }
    fn shrink(&self) -> Self::Shrinker {
        #[allow(clippy::as_conversions)]
        let mut x = *self as u32;
        if x > CHAR_GAP_OFFSET {
            x -= CHAR_GAP_SIZE;
        }
        CharShrinker {
            inner: x.shrink(),
            current: '0',
        }
    }
}

macro_rules! float_arbitrary {
    ($ty:ty, $ity:ty, $shrinker:ident) => {
        pub struct $shrinker {
            template: $ty,
            instance: $ty,
            result: $ty,
        }

        impl Shrinker for $shrinker {
            type Item = $ty;
            fn next(&mut self) -> Option<&Self::Item> {
                #[allow(clippy::as_conversions)]
                const SIGN_OFFSET: u32 = (std::mem::size_of::<$ty>() * 8 - 1) as u32;
                const EXP_OFFSET: u32 = <$ty>::MANTISSA_DIGITS - 1;
                const EXP_MASK: $ity = (1 << SIGN_OFFSET) - (1 << EXP_OFFSET);
                const EXP_PMASK: $ity = (1 << (SIGN_OFFSET - 1)) - (1 << EXP_OFFSET);
                const MANTISSA_MASK: $ity = (1 << EXP_OFFSET) - 1;
                const MANTISSA_PMASK: $ity = (1 << (EXP_OFFSET - 1)) - 1;

                let instance = self.instance;

                if instance.is_sign_negative() {
                    debug_assert_eq!(self.instance.to_bits(), self.template.to_bits());
                    let instance = instance.abs();
                    self.instance = instance;
                    self.result = instance;
                } else {
                    let instance = instance.to_bits();
                    let mut mask = EXP_MASK;
                    let mut pmask = EXP_PMASK;

                    if instance & mask == 0 {
                        mask = MANTISSA_MASK;
                        pmask = MANTISSA_PMASK;
                        if instance & mask == 0 {
                            return None;
                        }
                    }

                    let i = (instance >> 1) & pmask;
                    self.result = <$ty>::from_bits(self.template.to_bits() - i);
                    self.instance = <$ty>::from_bits(instance & !mask | i);
                }

                Some(&self.result)
            }
        }

        impl Arbitrary for $ty {
            type Shrinker = $shrinker;
            fn arbitrary() -> Self {
                static PROBLEM_FLOAT: &[$ty] = &[-0.0, -1.0, <$ty>::NEG_INFINITY, <$ty>::NAN];
                random_entry_or_gen(PROBLEM_FLOAT)
            }
            fn clone(&self) -> Self {
                *self
            }
            fn shrink(&self) -> Self::Shrinker {
                $shrinker {
                    template: *self,
                    instance: *self,
                    result: 0.0,
                }
            }
        }
    };
}

float_arbitrary!(f32, u32, Float32Shrinker);
float_arbitrary!(f64, u64, Float64Shrinker);

enum StringShrinkerInner {
    Ascii(VecShrinker<u8>),
    Full(String, VecShrinker<char>),
}

pub struct StringShrinker {
    inner: StringShrinkerInner,
}

impl Shrinker for StringShrinker {
    type Item = String;
    fn next(&mut self) -> Option<&Self::Item> {
        match &mut self.inner {
            // SAFETY: `inner` is a pure-ASCII byte buffer. Also, `String`s have the same layout as
            // `Vec<u8>` under the hood, so it's safe to transmute.
            StringShrinkerInner::Ascii(inner) => unsafe {
                const _: () = {
                    const STRING_SIZE: usize = std::mem::size_of::<String>();
                    const VEC_SIZE: usize = std::mem::size_of::<Vec<u8>>();
                    if STRING_SIZE != VEC_SIZE {
                        panic!("`String` size != `Vec` size");
                    }
                };
                std::mem::transmute(inner.next())
            },
            StringShrinkerInner::Full(current, inner) => {
                let next = inner.next()?;
                current.clear();
                current.extend(next);
                Some(current)
            }
        }
    }
}

impl Arbitrary for String {
    type Shrinker = StringShrinker;

    fn arbitrary() -> String {
        String::from_iter(<Vec<char>>::arbitrary())
    }

    fn clone(&self) -> Self {
        Clone::clone(self)
    }

    fn shrink(&self) -> Self::Shrinker {
        // Iterate the string list twice. First to count characters, and second to copy into the
        // vec. This avoids needing to reallocate.

        // The idea is this: counting characters is as simple as counting character start bytes.
        // ASCII characters can count as character start bytes, as can multi-byte character start
        // bytes.
        // - 00xxxxxx = ASCII single-byte character
        // - 01xxxxxx = ASCII single-byte character
        // - 10xxxxxx = Multi-byte character continuation, doesn't start a character.
        // - 11xxxxxx = Multi-byte character start, doesn't
        //
        // This can and should (easily) be vectorized in an optimized build. Not sure why Rust core
        // doesn't just do this.
        //
        // TODO: file a PR to change `core::str::count::count_chars` to do this.
        let mut char_count = 0;
        for byte in self.as_bytes() {
            char_count += usize::from((*byte & 0xC0) != 0x80);
        }

        // Assuming `self` is valid UTF-8 (as `String` asserts by design), the character count can
        // only equal the self length if only single-byte ASCII characters are present.
        if char_count == self.len() {
            StringShrinker {
                inner: StringShrinkerInner::Ascii(VecShrinker::new(self.as_bytes().to_vec())),
            }
        } else {
            let mut v = Vec::<char>::with_capacity(char_count);
            v.extend(self.chars());
            StringShrinker {
                inner: StringShrinkerInner::Full(String::new(), VecShrinker::new(v)),
            }
        }
    }
}

const _: () = {
    const TUPLE_SIZE: usize = std::mem::size_of::<(u64, u32)>();
    const DURATION_SIZE: usize = std::mem::size_of::<std::time::Duration>();
    if TUPLE_SIZE != DURATION_SIZE {
        panic!("`(u64, u32)` size != `Duration` size");
    }
};

transmute_arbitrary! {
    DurationShrinker, std::time::Duration, (u64, u32);
    let mut rng = rand::thread_rng();
    let secs = rng.gen();
    let nanos = rng.gen_range(0..999999999);
    std::time::Duration::new(secs, nanos)
}

pub struct SystemTimeShrinker {
    shrinker: DurationShrinker,
    result: std::time::SystemTime,
    reuse_value: bool,
}

impl Shrinker for SystemTimeShrinker {
    type Item = std::time::SystemTime;

    fn next(&mut self) -> Option<&Self::Item> {
        match self.reuse_value {
            false => {
                self.reuse_value = true;
                self.result = std::time::UNIX_EPOCH + *self.shrinker.next()?;
            }
            true => {
                let (secs, nanos) = self.shrinker.0.result;
                self.reuse_value = false;
                self.result = std::time::UNIX_EPOCH - std::time::Duration::new(secs, nanos);
            }
        }

        Some(&self.result)
    }
}

// Make sure I know how transmuting between durations and system times work, and that it's safe
// around edge cases.
mod system_time_transmute_tests {
    use super::*;
    use std::time::Duration;
    use std::time::SystemTime;

    fn transmute(duration: Duration) -> SystemTime {
        // SAFETY: It's the same size, and both are plain values.
        unsafe { std::mem::transmute::<_, SystemTime>(duration) }
    }

    #[test]
    fn works() {
        assert_eq!(UNIX_EPOCH, transmute(Duration::ZERO));
        assert_eq!(
            UNIX_EPOCH + Duration::new(1, 123),
            transmute(Duration::new(1, 123))
        );
        assert_eq!(
            UNIX_EPOCH + Duration::new(0x7FFFFFFFFFFFFFFF, 0),
            transmute(Duration::new(0x7FFFFFFFFFFFFFFF, 0))
        );
        assert_eq!(
            UNIX_EPOCH + Duration::new(0x7FFFFFFFFFFFFFFF, 999999999),
            transmute(Duration::new(0x7FFFFFFFFFFFFFFF, 999999999))
        );
        assert_eq!(
            UNIX_EPOCH - Duration::new(0x8000000000000000, 0),
            transmute(Duration::new(0x8000000000000000, 0))
        );
        assert_eq!(UNIX_EPOCH - Duration::new(0, 1), transmute(Duration::MAX));
        assert_eq!(
            UNIX_EPOCH - Duration::new(0, 999999999),
            transmute(Duration::new(0xFFFFFFFFFFFFFFFF, 1))
        );
    }
}

impl Arbitrary for std::time::SystemTime {
    type Shrinker = SystemTimeShrinker;

    fn arbitrary() -> Self {
        // SAFETY: See above test module.
        unsafe { std::mem::transmute(<std::time::Duration>::arbitrary()) }
    }

    fn clone(&self) -> Self {
        *self
    }

    fn shrink(&self) -> Self::Shrinker {
        let duration = self
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| e.duration());

        SystemTimeShrinker {
            shrinker: duration.shrink(),
            result: std::time::UNIX_EPOCH,
            reuse_value: false,
        }
    }
}

pub struct KindShrinker(UsizeShrinker, std::io::ErrorKind);

impl Shrinker for KindShrinker {
    type Item = std::io::ErrorKind;
    fn next(&mut self) -> Option<&Self::Item> {
        self.1 = crate::test_utils::ERROR_KINDS[*self.0.next()?];
        Some(&self.1)
    }
}

impl Arbitrary for std::io::ErrorKind {
    type Shrinker = KindShrinker;

    fn arbitrary() -> Self {
        random_entry(crate::test_utils::ERROR_KINDS)
    }

    fn clone(&self) -> Self {
        *self
    }

    fn shrink(&self) -> Self::Shrinker {
        let index = crate::test_utils::ERROR_KINDS
            .iter()
            .position(|c| c == self)
            .unwrap();
        KindShrinker(index.shrink(), std::io::ErrorKind::NotFound)
    }
}

// To avoid capturing `error` (and by proxy, `self`).
enum ErrorShrinkerKind {
    Code(UsizeShrinker),
    Kind(UsizeShrinker),
    Custom(Tuple2Shrinker<usize, String>),
}

pub struct ErrorShrinker {
    inner: ErrorShrinkerKind,
    error: std::io::Error,
}

impl Shrinker for ErrorShrinker {
    type Item = std::io::Error;
    fn next(&mut self) -> Option<&Self::Item> {
        self.error = match &mut self.inner {
            ErrorShrinkerKind::Code(code) => {
                std::io::Error::from_raw_os_error(crate::ffi::ERRNO_LIST[*code.next()?])
            }
            ErrorShrinkerKind::Kind(kind) => {
                std::io::Error::from(crate::test_utils::ERROR_KINDS[*kind.next()?])
            }
            ErrorShrinkerKind::Custom(tuple) => {
                let (kind, msg) = tuple.next()?;
                std::io::Error::new(crate::test_utils::ERROR_KINDS[*kind], Clone::clone(msg))
            }
        };
        Some(&self.error)
    }
}

impl Arbitrary for std::io::Error {
    type Shrinker = ErrorShrinker;

    fn arbitrary() -> Self {
        match rand::thread_rng().gen_range(0..3) {
            0 => Self::from_raw_os_error(random_entry(crate::ffi::ERRNO_LIST)),
            1 => Self::from(random_entry(crate::test_utils::ERROR_KINDS)),
            _ => Self::new(
                random_entry(crate::test_utils::ERROR_KINDS),
                <String as Arbitrary>::arbitrary(),
            ),
        }
    }

    /// This is why `Arbitrary` doesn't subtype `Clone`.
    fn clone(&self) -> Self {
        match self.raw_os_error() {
            Some(code) => Self::from_raw_os_error(code),
            None => match self.get_ref() {
                Some(inner) => Self::new(self.kind(), inner.to_string()),
                None => Self::from(self.kind()),
            },
        }
    }

    fn shrink(&self) -> Self::Shrinker {
        ErrorShrinker {
            error: std::io::Error::from(std::io::ErrorKind::NotFound),
            inner: match self.raw_os_error() {
                Some(code) => ErrorShrinkerKind::Code(
                    crate::ffi::ERRNO_LIST
                        .iter()
                        .position(|c| *c == code)
                        .unwrap()
                        .shrink(),
                ),
                None => {
                    let index = crate::test_utils::ERROR_KINDS
                        .iter()
                        .position(|k| *k == self.kind())
                        .unwrap();

                    match self.get_ref() {
                        Some(inner) => {
                            ErrorShrinkerKind::Custom((index, inner.to_string()).shrink())
                        }
                        None => ErrorShrinkerKind::Kind(index.shrink()),
                    }
                }
            },
        }
    }
}
