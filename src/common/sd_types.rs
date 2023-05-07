use crate::prelude::*;

//  ######
//  #     # #####  #  ####  #####  # ##### #   #
//  #     # #    # # #    # #    # #   #    # #
//  ######  #    # # #    # #    # #   #     #
//  #       #####  # #    # #####  #   #     #
//  #       #   #  # #    # #   #  #   #     #
//  #       #    # #  ####  #    # #   #     #

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    #[allow(dead_code)]
    Emergency = 0,
    #[allow(dead_code)]
    Alert = 1,
    #[allow(dead_code)]
    Critical = 2,
    #[allow(dead_code)]
    Error = 3,
    #[allow(dead_code)]
    Warning = 4,
    #[allow(dead_code)]
    Notice = 5,
    #[allow(dead_code)]
    Informational = 6,
    #[allow(dead_code)]
    Debug = 7,
}

#[cfg(test)]
impl Arbitrary for Priority {
    fn arbitrary(_: &mut Gen) -> Self {
        // Don't bother with the ceremony of `quickcheck::Gen` and just use `rand` directly.
        Self::from_severity_index(truncate_usize_u8(sample_to(7))).unwrap()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(PriorityShrinker::new(*self))
    }
}

#[derive(Debug, PartialEq)]
pub enum PriorityParseError {
    Empty,
    Invalid,
}

impl Priority {
    pub fn from_severity_value(s: &[u8]) -> Result<Priority, PriorityParseError> {
        match *s {
            [b @ b'0'..=b'7'] => {
                Self::from_severity_index(b.wrapping_sub(b'0')).ok_or(PriorityParseError::Invalid)
            }
            [] => Err(PriorityParseError::Empty),
            _ => Err(PriorityParseError::Invalid),
        }
    }

    pub fn from_severity_index(byte: u8) -> Option<Priority> {
        match byte {
            // SAFETY: The `Priority` enum specifically represents a contiguous range from 0 to 7
            // inclusive, and the ASCII digit range is just this shifted over by a constant amount
            // (in this case, 0x30, but the specific number is irrelevant here.)
            0..=7 => Some(unsafe { std::mem::transmute(byte) }),
            _ => None,
        }
    }

    pub fn as_name_bytes(self) -> &'static [u8] {
        match self {
            Priority::Emergency => b"EMERG",
            Priority::Alert => b"ALERT",
            Priority::Critical => b"CRIT",
            Priority::Error => b"ERR",
            Priority::Warning => b"WARNING",
            Priority::Notice => b"NOTICE",
            Priority::Informational => b"INFO",
            Priority::Debug => b"DEBUG",
        }
    }

    pub fn as_severity_index(self) -> u8 {
        // Part of the point of this method. It's practically unavoidable.
        #![allow(clippy::as_conversions)]

        self as u8
    }

    pub fn as_severity_byte(self) -> u8 {
        self.as_severity_index().wrapping_add(b'0')
    }
}

#[cfg(test)]
pub struct PriorityShrinker(u8);

#[cfg(test)]
impl PriorityShrinker {
    pub fn new(priority: Priority) -> Self {
        Self(priority.as_severity_index())
    }
}

#[cfg(test)]
impl Iterator for PriorityShrinker {
    type Item = Priority;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.checked_sub(1) {
            None => None,
            Some(next) => {
                self.0 = next;
                // SAFETY: The mapped byte obviously can't be out of range of valid `Priority`
                // values as it's always decremented, and priority indices are contiguous.
                Some(unsafe { std::mem::transmute(next) })
            }
        }
    }
}

//   #####
//  #     # ###### #####  #    # #  ####  ######
//  #       #      #    # #    # # #    # #
//   #####  #####  #    # #    # # #      #####
//        # #      #####  #    # # #      #
//  #     # #      #   #   #  #  # #    # #
//   #####  ###### #    #   ##   #  ####  ######

#[derive(Debug, PartialEq, Eq)]
pub enum ServiceParseError {
    Empty,
    TooLong,
    Invalid,
}

// Ref: https://www.freedesktop.org/software/systemd/man/systemd.unit.html
pub const MAX_SERVICE_LEN: usize = 256;
const MAX_UNITLESS_SERVICE_LEN: usize = MAX_SERVICE_LEN.wrapping_sub(8);

const _: () = {
    if MAX_SERVICE_LEN != zero_extend_u8_usize(u8::MAX) + 1 {
        panic!("Assumption invalid: `MAX_SERVICE_LEN == u8::MAX + 1`");
    }
};

pub struct Service<'a>(&'a [u8]);

impl fmt::Debug for Service<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Service({:?})", self.as_str())
    }
}

impl PartialEq for Service<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for Service<'_> {}

/*
From the `systemd.unit(5)` man page:

> Valid unit names consist of a "name prefix" and a dot and a suffix specifying the unit type. The
> "unit prefix" must consist of one or more valid characters (ASCII letters, digits, ":", "-", "_",
> ".", and "\"). The total length of the unit name including the suffix must not exceed 256
> characters. The type suffix must be one of ".service", ".socket", ".device", ".mount",
> ".automount", ".swap", ".target", ".path", ".timer", ".slice", or ".scope".

I'm not enforcing the specific type suffixes (I'm not updating this every time systemd adds a new
unit type), but I am enforcing the rest strictly.
*/

#[derive(Clone, Copy, PartialEq, Eq)]
enum ServiceSyntaxClassification {
    Empty,
    TooLong,
    Invalid,
    Unitful,
    Unitless,
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClassifyState {
    Fail,
    NameStart,
    NamePart,
    SuffixStart,
    SuffixPart,
    InstanceStart,
    InstancePart,
    InstanceSuffixStart,
    InstanceSuffixPart,
}

impl From<ClassifyState> for usize {
    #[allow(clippy::as_conversions)]
    fn from(value: ClassifyState) -> Self {
        value as usize
    }
}

// Goal is to have the classification hit L1 for these lookup tables, falling back to L2 in most
// cases where it isn't in L1. It's intentionally as small as reasonably possible, and I even
// verify it's only ASCII first to further cut down on table size.

#[repr(align(64))]
struct CacheAligned<T>(T);
impl<T> std::ops::Deref for CacheAligned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(clippy::as_conversions)]
static CLASSIFY_LUT: CacheAligned<[[ClassifyState; 256]; 9]> = CacheAligned({
    let mut result = [[ClassifyState::Fail; 256]; 9];

    macro_rules! set {
        ($state:ident, $ch:expr, $next:ident) => {{
            result[ClassifyState::$state as usize][zero_extend_u8_usize($ch)] =
                ClassifyState::$next;
        }};
        ($state:ident, $start:expr, $end:expr, $next:ident) => {{
            let mut ch = $start;
            while ch <= $end {
                set!($state, ch, $next);
                ch = ch.wrapping_add(1);
            }
        }};
        ($state:ident, @valid, $next:ident) => {{
            set!($state, b'0', b'9', $next);
            set!($state, b'A', b'Z', $next);
            set!($state, b'a', b'z', $next);
            set!($state, b':', $next);
            set!($state, b'-', $next);
            set!($state, b'_', $next);
            set!($state, b'\\', $next);
        }};
    }

    // Inital state: `NAME_START`
    set!(NameStart, b'@', InstanceStart);
    set!(NameStart, @valid, NamePart);

    set!(NamePart, b'.', SuffixStart);
    set!(NamePart, b'@', InstanceStart);
    set!(NamePart, @valid, NamePart);

    set!(SuffixStart, b'.', SuffixStart);
    set!(SuffixStart, b'@', InstanceStart);
    set!(SuffixStart, @valid, SuffixPart);

    set!(SuffixPart, b'.', SuffixStart);
    set!(SuffixPart, b'@', InstanceStart);
    set!(SuffixPart, @valid, SuffixPart);

    set!(InstanceStart, b'.', InstanceSuffixStart);
    set!(InstanceStart, @valid, InstancePart);

    set!(InstancePart, b'.', InstanceSuffixStart);
    set!(InstancePart, @valid, InstancePart);

    set!(InstanceSuffixStart, b'.', InstanceSuffixStart);
    set!(InstanceSuffixStart, @valid, InstanceSuffixPart);

    set!(InstanceSuffixPart, b'.', InstanceSuffixStart);
    set!(InstanceSuffixPart, @valid, InstanceSuffixPart);

    result
});

fn classify_advance(state: ClassifyState, ch: u8) -> ClassifyState {
    debug_assert!(usize::from(state) < CLASSIFY_LUT.len());
    debug_assert!(zero_extend_u8_usize(u8::MAX) < CLASSIFY_LUT[0].len());
    // SAFETY: Preconditions are asserted above. The whole point here is to just eliminate bounds
    // checks. It also makes code gen a bit smaller.
    unsafe {
        *CLASSIFY_LUT
            .get_unchecked(usize::from(state))
            .get_unchecked(zero_extend_u8_usize(ch))
    }
}

// This is in what's probably the hottest path of the entire program aside from the regex matching
// and mere memory copying, so performance is a bit critical here.
fn classify_service_syntax(s: &[u8]) -> ServiceSyntaxClassification {
    if s.len() > MAX_SERVICE_LEN {
        return ServiceSyntaxClassification::TooLong;
    }

    let mut state = ClassifyState::NameStart;

    for ch in s {
        state = classify_advance(state, *ch);
    }

    match state {
        ClassifyState::NameStart => ServiceSyntaxClassification::Empty,
        ClassifyState::NamePart | ClassifyState::InstancePart => {
            if s.len() <= MAX_UNITLESS_SERVICE_LEN {
                ServiceSyntaxClassification::Unitless
            } else {
                ServiceSyntaxClassification::TooLong
            }
        }
        ClassifyState::SuffixPart | ClassifyState::InstanceSuffixPart => {
            ServiceSyntaxClassification::Unitful
        }
        _ => ServiceSyntaxClassification::Invalid,
    }
}

impl<'a> Service<'a> {
    pub fn from_full_service(s: &[u8]) -> Result<Service, ServiceParseError> {
        match classify_service_syntax(s) {
            ServiceSyntaxClassification::Empty => Err(ServiceParseError::Empty),
            ServiceSyntaxClassification::TooLong => Err(ServiceParseError::TooLong),
            ServiceSyntaxClassification::Invalid => Err(ServiceParseError::Invalid),
            ServiceSyntaxClassification::Unitful => Ok(Service(s)),
            ServiceSyntaxClassification::Unitless => Err(ServiceParseError::Invalid),
        }
    }

    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(self.0).unwrap()
    }
}

pub struct ServiceRepr {
    service_len: u16,
    //             [u8; MAX_SERVICE_LEN]
    service_bytes: [u8; 256],
}

impl PartialEq for ServiceRepr {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for ServiceRepr {}

impl PartialOrd for ServiceRepr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ServiceRepr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_bytes = self.as_bytes();
        let other_bytes = other.as_bytes();

        match self_bytes.len().cmp(&other_bytes.len()) {
            std::cmp::Ordering::Equal => self_bytes.cmp(other_bytes),
            not_equal => not_equal,
        }
    }
}

impl std::hash::Hash for ServiceRepr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state)
    }
}

impl Clone for ServiceRepr {
    fn clone(&self) -> Self {
        let mut result = ServiceRepr::empty();
        result.service_len = self.service_len;
        // SAFETY: It's only copying the initialized parts.
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.service_bytes.as_ptr(),
                result.service_bytes.as_mut_ptr(),
                zero_extend_u16_usize(self.service_len),
            )
        }
        result
    }
}

impl ServiceRepr {
    #[cfg(test)]
    pub fn new(s: Option<&[u8]>) -> Result<Self, ServiceParseError> {
        match s {
            Some(s) => Self::from_slice(s),
            None => Ok(Self::empty()),
        }
    }

    pub const fn empty() -> Self {
        Self {
            service_len: 0,
            service_bytes: [0; MAX_SERVICE_LEN],
        }
    }

    pub fn matches(&self, received: &Self) -> bool {
        let mut self_bytes = self.as_bytes().iter();
        let mut received_bytes = received.as_bytes().iter();

        // Read for exact matches, with one exception: consider received `foo@123.service` to
        // match `foo@.service` (but not `foo.service`).
        let mut prev = 0;

        loop {
            match (self_bytes.next(), received_bytes.next()) {
                (None, None) => return true,
                (Some(&a), Some(&b)) if a == b => prev = a,
                (Some(&a @ b'.'), Some(_)) if prev == b'@' => loop {
                    match received_bytes.next() {
                        None => return false,
                        Some(b'.') => break,
                        Some(_) => prev = a,
                    }
                },
                _ => return false,
            }
        }
    }

    pub fn from_slice(s: &[u8]) -> Result<Self, ServiceParseError> {
        fn build(s: &[u8], add_unit: bool) -> ServiceRepr {
            let mut repr = ServiceRepr::empty();

            repr.service_len = truncate_usize_u16(s.len());
            repr.service_bytes[..s.len()].copy_from_slice(s);

            if add_unit {
                let extra = b".service";

                repr.service_len = repr
                    .service_len
                    .wrapping_add(truncate_usize_u16(extra.len()));

                repr.service_bytes[s.len()..s.len().wrapping_add(extra.len())]
                    .copy_from_slice(extra);
            }

            repr
        }

        match classify_service_syntax(s) {
            ServiceSyntaxClassification::Empty => Err(ServiceParseError::Empty),
            ServiceSyntaxClassification::TooLong => Err(ServiceParseError::TooLong),
            ServiceSyntaxClassification::Invalid => Err(ServiceParseError::Invalid),
            ServiceSyntaxClassification::Unitful => Ok(build(s, false)),
            ServiceSyntaxClassification::Unitless => Ok(build(s, true)),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: The first `self.service_len` bytes are always initialized.
        unsafe {
            std::slice::from_raw_parts(
                self.service_bytes.as_ptr().cast(),
                zero_extend_u16_usize(self.service_len),
            )
        }
    }

    pub fn as_service(&self) -> Option<Service> {
        let bytes = self.as_bytes();
        if bytes.is_empty() {
            None
        } else {
            Some(Service(bytes))
        }
    }

    pub fn set_service(&mut self, service: Service) {
        let bytes = service.as_bytes();
        self.service_len = truncate_usize_u16(bytes.len());
        // SAFETY:
        // 1. The memory ranges can't overlap since separate mutable and immutable borrows cannot
        //    overlap.
        // 2. `Service` enforces a size limit of `MAX_SERVICE_LEN`, so it can be trusted here.
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                self.service_bytes.as_mut_ptr().cast(),
                bytes.len(),
            )
        }
    }
}

impl fmt::Debug for ServiceRepr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_service().fmt(f)
    }
}

#[cfg(test)]
static SERVICE_CHARS: &[u8] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-.:@\\_";

#[cfg(test)]
static SERVICE_CHAR_OFFSETS: [u8; 256] = {
    let mut results = [u8::MAX; 256];
    let mut i = 0;
    while i < SERVICE_CHARS.len() {
        results[zero_extend_u8_usize(SERVICE_CHARS[i])] = truncate_usize_u8(i);
        i += 1;
    }
    results
};

#[cfg(test)]
static SERVICE_CHAR_COUNT: usize = SERVICE_CHARS.len();

#[cfg(test)]
fn offset_to_service_char(offset: usize) -> u8 {
    SERVICE_CHARS[offset]
}

#[cfg(test)]
fn service_char_to_offset(ch: u8) -> usize {
    let offset = SERVICE_CHAR_OFFSETS[zero_extend_u8_usize(ch)];
    if offset == u8::MAX {
        panic!("unexpected character: {ch:02x} {:?}", BinaryToDebug(&[ch]))
    }
    zero_extend_u8_usize(offset)
}

#[cfg(test)]
static UNIT_TYPE_EXTS: &[&[u8]] = &[
    b".automount",
    b".device",
    b".mount",
    b".path",
    b".scope",
    b".service",
    b".slice",
    b".socket",
    b".swap",
    b".target",
    b".timer",
];

#[cfg(test)]
static UNIT_EXT_COUNT: usize = UNIT_TYPE_EXTS.len();

#[cfg(test)]
static MAX_UNIT_EXT_LEN: usize = 10;

#[cfg(test)]
fn offset_to_unit_ext(offset: usize) -> &'static [u8] {
    UNIT_TYPE_EXTS[offset]
}

#[cfg(test)]
fn unit_ext_to_offset(unit_ext: &[u8]) -> usize {
    // Similar to above, but inlines `UNIT_TYPE_EXTS.iter().position(|c| *c == unit_ext).unwrap()`.
    match unit_ext {
        b".automount" => 0,
        b".device" => 1,
        b".mount" => 2,
        b".path" => 3,
        b".scope" => 4,
        b".service" => 5,
        b".slice" => 6,
        b".socket" => 7,
        b".swap" => 8,
        b".target" => 9,
        b".timer" => 10,
        _ => panic!("unexpected ext: {:?}", BinaryToDebug(unit_ext)),
    }
}

// Based on `quickcheck`'s `VecShrinker`, but adapted to this.
#[cfg(test)]
pub fn service_shrinker(repr: &ServiceRepr) -> impl Iterator<Item = ServiceRepr> {
    #[derive(Clone, Copy)]
    struct ServiceCharShrinker(u8);

    impl Arbitrary for ServiceCharShrinker {
        fn arbitrary(_: &mut Gen) -> Self {
            unreachable!()
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let offset = zero_extend_u8_usize(SERVICE_CHAR_OFFSETS[zero_extend_u8_usize(self.0)]);
            Box::new(SERVICE_CHARS[..offset].iter().cloned().map(Self))
        }
    }

    let bytes = repr.as_bytes();
    let period = bytes.iter().rposition(|c| *c == b'.').unwrap();

    debug_assert!(
        period <= MAX_SERVICE_LEN - MAX_UNIT_EXT_LEN,
        "{} <= {}",
        period,
        MAX_SERVICE_LEN - MAX_UNIT_EXT_LEN
    );

    let (name_bytes, ext_bytes) = bytes.split_at(period);

    let mut unit_ext_shrink = unit_ext_to_offset(ext_bytes);
    let unit_ext = offset_to_unit_ext(unit_ext_shrink);

    let name_data = ServiceRepr {
        service_len: truncate_usize_u16(period),
        service_bytes: repr.service_bytes,
    };

    // SAFETY: The input bytes can only be valid service character bytes.
    let name_bytes: &[ServiceCharShrinker] = unsafe { std::mem::transmute(name_bytes) };

    fn build_repr(name_bytes: &[u8], unit_ext: &[u8]) -> ServiceRepr {
        let mut repr = ServiceRepr::empty();
        let total_len = unit_ext.len() + name_bytes.len();

        repr.service_len = truncate_usize_u16(total_len);
        repr.service_bytes[..name_bytes.len()].copy_from_slice(name_bytes);
        repr.service_bytes[name_bytes.len()..total_len].copy_from_slice(unit_ext);

        repr
    }

    let iter = Vec::from(name_bytes).shrink().filter_map(|name_bytes| {
        if name_bytes.is_empty() {
            None
        } else {
            Some(build_repr(
                // SAFETY: It's always safe to transmute from `ServiceCharShrinker` to `u8`, as
                // they have the same underlying byte structure.
                unsafe { std::mem::transmute(&*name_bytes) },
                unit_ext,
            ))
        }
    });

    iter.chain(std::iter::from_fn(move || {
        let next_offset = unit_ext_shrink.checked_sub(1)?;
        unit_ext_shrink = next_offset;
        Some(build_repr(
            name_data.as_bytes(),
            offset_to_unit_ext(next_offset),
        ))
    }))
}

#[cfg(test)]
impl Arbitrary for ServiceRepr {
    fn arbitrary(_: &mut Gen) -> Self {
        // This is a significant bottleneck for Miri, hence why it doesn't use `g` and instead uses
        // `rand` directly. Only reason it's optimized like this is because it's actually digging
        // into my productivity.

        static PREFIX_LEN: usize = MAX_SERVICE_LEN - MAX_UNIT_EXT_LEN;
        let mut repr = Self::empty();

        let prefix_len = sample_to(PREFIX_LEN).wrapping_add(1);
        let unit_ext = offset_to_unit_ext(sample_to(UNIT_EXT_COUNT));

        repr.service_len = truncate_usize_u16(prefix_len + unit_ext.len());
        // SAFETY: `prefix_len + unit_ext.len()` is always less than `MAX_SERVICE_LEN` because
        // `prefix_len` is constrained such that it always leaves room for the unit name.
        unsafe {
            let mut repr_ptr = repr.service_bytes.as_mut_ptr().cast::<u8>();
            let end = repr_ptr.add(prefix_len);

            while repr_ptr < end {
                *repr_ptr = offset_to_service_char(sample_to(SERVICE_CHAR_COUNT));
                repr_ptr = repr_ptr.offset(1);
            }

            std::ptr::copy_nonoverlapping(unit_ext.as_ptr(), end, unit_ext.len());
        }

        repr
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(service_shrinker(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_chars_is_defined_correctly() {
        static SERVICE_CHARS: &[u8] =
            b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-.:@\\_";

        for (i, &ch) in SERVICE_CHARS.iter().enumerate() {
            assert_eq!(
                service_char_to_offset(ch),
                i,
                "{ch:02x} {:?} -> {i}",
                BinaryToDebug(&[ch])
            );
            assert_eq!(
                offset_to_service_char(i),
                ch,
                "{i} -> {ch:02x} {:?}",
                BinaryToDebug(&[ch])
            );
        }
    }

    #[test]
    fn unit_exts_is_defined_correctly() {
        for (i, &unit_ext) in UNIT_TYPE_EXTS.iter().enumerate() {
            assert_eq!(
                unit_ext_to_offset(unit_ext),
                i,
                "{:?} -> {i}",
                BinaryToDebug(unit_ext)
            );
            assert_eq!(
                offset_to_unit_ext(i),
                unit_ext,
                "{i} -> {:?}",
                BinaryToDebug(unit_ext)
            );
        }
    }

    //  ######
    //  #     # #####  #  ####  #####  # ##### #   #
    //  #     # #    # # #    # #    # #   #    # #
    //  ######  #    # # #    # #    # #   #     #
    //  #       #####  # #    # #####  #   #     #
    //  #       #   #  # #    # #   #  #   #     #
    //  #       #    # #  ####  #    # #   #     #

    #[test]
    fn priority_decodes_empty_severity_to_empty_error() {
        assert_eq!(
            Priority::from_severity_value(b""),
            Err(PriorityParseError::Empty)
        );
    }

    #[test]
    fn priority_decodes_non_numeric_severity_to_invalid_error() {
        assert_eq!(
            Priority::from_severity_value(b"DEBUG"),
            Err(PriorityParseError::Invalid)
        );
    }

    #[test]
    fn priority_decodes_severity_8_to_invalid_error() {
        assert_eq!(
            Priority::from_severity_value(b"8"),
            Err(PriorityParseError::Invalid)
        );
    }

    #[test]
    fn priority_decodes_severity_9_to_invalid_error() {
        assert_eq!(
            Priority::from_severity_value(b"9"),
            Err(PriorityParseError::Invalid)
        );
    }

    #[test]
    fn priority_decodes_multi_digit_severity_to_invalid_error() {
        assert_eq!(
            Priority::from_severity_value(b"123"),
            Err(PriorityParseError::Invalid)
        );
    }

    #[test]
    fn priority_decodes_severity_index_8_to_none() {
        assert_eq!(Priority::from_severity_index(8), None);
    }

    #[test]
    fn priority_decodes_severity_index_9_to_none() {
        assert_eq!(Priority::from_severity_index(9), None);
    }

    #[test]
    fn priority_emerg_returns_right_name() {
        assert_eq!(Priority::Emergency.as_name_bytes(), b"EMERG");
    }

    #[test]
    fn priority_emerg_returns_right_severity_byte() {
        assert_eq!(Priority::Emergency.as_severity_byte(), b'0');
    }

    #[test]
    fn priority_emerg_returns_right_severity_index() {
        assert_eq!(Priority::Emergency.as_severity_index(), 0);
    }

    #[test]
    fn priority_emerg_is_decoded_from_severity_value() {
        assert_eq!(Priority::from_severity_value(b"0"), Ok(Priority::Emergency));
    }

    #[test]
    fn priority_emerg_is_decoded_from_severity_index() {
        assert_eq!(Priority::from_severity_index(0), Some(Priority::Emergency));
    }

    #[test]
    fn priority_alert_returns_right_name() {
        assert_eq!(Priority::Alert.as_name_bytes(), b"ALERT");
    }

    #[test]
    fn priority_alert_returns_right_severity_byte() {
        assert_eq!(Priority::Alert.as_severity_byte(), b'1');
    }

    #[test]
    fn priority_alert_returns_right_severity_index() {
        assert_eq!(Priority::Alert.as_severity_index(), 1);
    }

    #[test]
    fn priority_alert_is_decoded_from_severity_value() {
        assert_eq!(Priority::from_severity_value(b"1"), Ok(Priority::Alert));
    }

    #[test]
    fn priority_alert_is_decoded_from_severity_index() {
        assert_eq!(Priority::from_severity_index(1), Some(Priority::Alert));
    }

    #[test]
    fn priority_crit_returns_right_name() {
        assert_eq!(Priority::Critical.as_name_bytes(), b"CRIT");
    }

    #[test]
    fn priority_crit_returns_right_severity_byte() {
        assert_eq!(Priority::Critical.as_severity_byte(), b'2');
    }

    #[test]
    fn priority_crit_returns_right_severity_index() {
        assert_eq!(Priority::Critical.as_severity_index(), 2);
    }

    #[test]
    fn priority_crit_is_decoded_from_severity_value() {
        assert_eq!(Priority::from_severity_value(b"2"), Ok(Priority::Critical));
    }

    #[test]
    fn priority_crit_is_decoded_from_severity_index() {
        assert_eq!(Priority::from_severity_index(2), Some(Priority::Critical));
    }

    #[test]
    fn priority_error_returns_right_name() {
        assert_eq!(Priority::Error.as_name_bytes(), b"ERR");
    }

    #[test]
    fn priority_error_returns_right_severity_byte() {
        assert_eq!(Priority::Error.as_severity_byte(), b'3');
    }

    #[test]
    fn priority_error_returns_right_severity_index() {
        assert_eq!(Priority::Error.as_severity_index(), 3);
    }

    #[test]
    fn priority_error_is_decoded_from_severity_value() {
        assert_eq!(Priority::from_severity_value(b"3"), Ok(Priority::Error));
    }

    #[test]
    fn priority_error_is_decoded_from_severity_index() {
        assert_eq!(Priority::from_severity_index(3), Some(Priority::Error));
    }

    #[test]
    fn priority_warning_returns_right_name() {
        assert_eq!(Priority::Warning.as_name_bytes(), b"WARNING");
    }

    #[test]
    fn priority_warning_returns_right_severity_byte() {
        assert_eq!(Priority::Warning.as_severity_byte(), b'4');
    }

    #[test]
    fn priority_warning_returns_right_severity_index() {
        assert_eq!(Priority::Warning.as_severity_index(), 4);
    }

    #[test]
    fn priority_warning_is_decoded_from_severity_value() {
        assert_eq!(Priority::from_severity_value(b"4"), Ok(Priority::Warning));
    }

    #[test]
    fn priority_warning_is_decoded_from_severity_index() {
        assert_eq!(Priority::from_severity_index(4), Some(Priority::Warning));
    }

    #[test]
    fn priority_notice_returns_right_name() {
        assert_eq!(Priority::Notice.as_name_bytes(), b"NOTICE");
    }

    #[test]
    fn priority_notice_returns_right_severity_byte() {
        assert_eq!(Priority::Notice.as_severity_byte(), b'5');
    }

    #[test]
    fn priority_notice_returns_right_severity_index() {
        assert_eq!(Priority::Notice.as_severity_index(), 5);
    }

    #[test]
    fn priority_notice_is_decoded_from_severity_value() {
        assert_eq!(Priority::from_severity_value(b"5"), Ok(Priority::Notice));
    }

    #[test]
    fn priority_notice_is_decoded_from_severity_index() {
        assert_eq!(Priority::from_severity_index(5), Some(Priority::Notice));
    }

    #[test]
    fn priority_info_returns_right_name() {
        assert_eq!(Priority::Informational.as_name_bytes(), b"INFO");
    }

    #[test]
    fn priority_info_returns_right_severity_byte() {
        assert_eq!(Priority::Informational.as_severity_byte(), b'6');
    }

    #[test]
    fn priority_info_returns_right_severity_index() {
        assert_eq!(Priority::Informational.as_severity_index(), 6);
    }

    #[test]
    fn priority_info_is_decoded_from_severity_value() {
        assert_eq!(
            Priority::from_severity_value(b"6"),
            Ok(Priority::Informational)
        );
    }

    #[test]
    fn priority_info_is_decoded_from_severity_index() {
        assert_eq!(
            Priority::from_severity_index(6),
            Some(Priority::Informational)
        );
    }

    #[test]
    fn priority_debug_returns_right_name() {
        assert_eq!(Priority::Debug.as_name_bytes(), b"DEBUG");
    }

    #[test]
    fn priority_debug_returns_right_severity_byte() {
        assert_eq!(Priority::Debug.as_severity_byte(), b'7');
    }

    #[test]
    fn priority_debug_returns_right_severity_index() {
        assert_eq!(Priority::Debug.as_severity_index(), 7);
    }

    #[test]
    fn priority_debug_is_decoded_from_severity_value() {
        assert_eq!(Priority::from_severity_value(b"7"), Ok(Priority::Debug));
    }

    #[test]
    fn priority_debug_is_decoded_from_severity_index() {
        assert_eq!(Priority::from_severity_index(7), Some(Priority::Debug));
    }

    //   #####
    //  #     # ###### #####  #    # #  ####  ######
    //  #       #      #    # #    # # #    # #
    //   #####  #####  #    # #    # # #      #####
    //        # #      #####  #    # # #      #
    //  #     # #      #   #   #  #  # #    # #
    //   #####  ###### #    #   ##   #  ####  ######

    #[test]
    fn service_reads_from_valid_slice() {
        let service = Service::from_full_service(b"service-name.unit").unwrap();
        assert_eq!(service.as_bytes(), b"service-name.unit");
    }

    #[test]
    fn service_reads_from_max_len_slice() {
        let mut raw = [b'a'; 256];
        raw[251..].copy_from_slice(b".unit");
        let service = Service::from_full_service(&raw).unwrap();
        assert_eq!(service.as_bytes(), &raw);
    }

    #[test]
    fn service_reads_from_valid_template_instance_slice() {
        let service = Service::from_full_service(b"service-name@id.unit").unwrap();
        assert_eq!(service.as_bytes(), b"service-name@id.unit");
    }

    #[test]
    fn service_accepts_all_lowercase_chars() {
        let service = Service::from_full_service(b"abcdefghijklmnopqrstuvwxyz.service").unwrap();
        assert_eq!(service.as_bytes(), b"abcdefghijklmnopqrstuvwxyz.service");
    }

    #[test]
    fn service_accepts_all_uppercase_chars() {
        let service = Service::from_full_service(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ.unit").unwrap();
        assert_eq!(service.as_bytes(), b"ABCDEFGHIJKLMNOPQRSTUVWXYZ.unit");
    }

    #[test]
    fn service_accepts_all_numbers() {
        let service = Service::from_full_service(b"service0123456789.timer").unwrap();
        assert_eq!(service.as_bytes(), b"service0123456789.timer");
    }

    #[test]
    fn service_accepts_numeric_template_instance_id() {
        let service = Service::from_full_service(b"service@0123456789.service").unwrap();
        assert_eq!(service.as_bytes(), b"service@0123456789.service");
    }

    #[test]
    fn service_accepts_special_chars() {
        let service = Service::from_full_service(b"s\\e:r_v.i-c@e:1.2_3\\4.service").unwrap();
        assert_eq!(service.as_bytes(), b"s\\e:r_v.i-c@e:1.2_3\\4.service");
    }

    #[test]
    fn service_rejects_empty_names() {
        assert_eq!(
            Service::from_full_service(b""),
            Err(ServiceParseError::Empty)
        );
    }

    #[test]
    fn service_rejects_single_hyphen_names() {
        assert_eq!(
            Service::from_full_service(b"-"),
            Err(ServiceParseError::Invalid)
        );
    }

    #[test]
    fn service_rejects_single_char_names() {
        assert_eq!(
            Service::from_full_service(b"a"),
            Err(ServiceParseError::Invalid)
        );
    }

    #[test]
    fn service_rejects_only_type() {
        assert_eq!(
            Service::from_full_service(b".service"),
            Err(ServiceParseError::Invalid)
        );
    }

    #[test]
    fn service_rejects_too_long_names_with_only_valid_chars() {
        assert_eq!(
            Service::from_full_service(&[b'a'; 300]),
            Err(ServiceParseError::TooLong)
        );
    }

    #[test]
    fn service_rejects_too_long_names_with_invalid_chars() {
        assert_eq!(
            Service::from_full_service(&[b' '; 300]),
            Err(ServiceParseError::TooLong)
        );
    }

    #[test]
    fn service_rejects_semicolons() {
        assert_eq!(
            Service::from_full_service(b"service;foo.service"),
            Err(ServiceParseError::Invalid)
        );
    }

    #[test]
    fn service_rejects_spaces() {
        assert_eq!(
            Service::from_full_service(b"service foo.unit"),
            Err(ServiceParseError::Invalid)
        );
    }

    fn service_full_regex() -> &'static regex::Regex {
        static SERVICE_FULL_REGEX: OnceCell<regex::Regex> = OnceCell::new();
        SERVICE_FULL_REGEX.get_or_init(|| {
            regex::Regex::new(r"^[0-9A-Za-z:_.\\-]+(@[0-9A-Za-z:_.\\-]+)?\.[0-9A-Za-z:_\\-]+$")
                .unwrap()
        })
    }

    #[quickcheck]
    fn service_validation_works(a: String) -> bool {
        match Service::from_full_service(a.as_bytes()) {
            Ok(s) => s.as_bytes() == a.as_bytes(),
            Err(ServiceParseError::Empty) => a.is_empty(),
            Err(ServiceParseError::TooLong) => a.len() > MAX_SERVICE_LEN,
            Err(ServiceParseError::Invalid) => !service_full_regex().is_match(&a),
        }
    }

    fn service_tolerant_regex() -> &'static regex::Regex {
        static SERVICE_FULL_REGEX: OnceCell<regex::Regex> = OnceCell::new();
        SERVICE_FULL_REGEX.get_or_init(|| {
            regex::Regex::new(r"^[0-9A-Za-z:_.\\-]+(@[0-9A-Za-z:_.\\-]+)?$").unwrap()
        })
    }

    #[quickcheck]
    fn service_repr_validation_works(a: String) -> bool {
        match ServiceRepr::from_slice(a.as_bytes()) {
            Ok(s) => s.as_bytes() == a.as_bytes(),
            Err(ServiceParseError::Empty) => a.is_empty(),
            Err(ServiceParseError::TooLong) => a.len() > MAX_SERVICE_LEN,
            Err(ServiceParseError::Invalid) => !service_tolerant_regex().is_match(&a),
        }
    }
}
