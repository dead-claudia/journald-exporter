use crate::prelude::*;

//  ######
//  #     # #####  #  ####  #####  # ##### #   #
//  #     # #    # # #    # #    # #   #    # #
//  ######  #    # # #    # #    # #   #     #
//  #       #####  # #    # #####  #   #     #
//  #       #   #  # #    # #   #  #   #     #
//  #       #    # #  ####  #    # #   #     #

// Save a little space.
#[repr(u8)]
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
    fn arbitrary(g: &mut Gen) -> Self {
        *g.choose(&[
            Self::Emergency,
            Self::Alert,
            Self::Critical,
            Self::Error,
            Self::Warning,
            Self::Notice,
            Self::Informational,
            Self::Debug,
        ])
        .unwrap()
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let byte = self.as_severity_byte();
        Box::new((b'0'..byte).map(|byte| Self::from_severity_value(&[byte]).unwrap()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PriorityParseError {
    Empty,
    Invalid,
}

impl Priority {
    pub fn from_severity_value(s: &[u8]) -> Result<Priority, PriorityParseError> {
        match *s {
            // SAFETY: The `Priority` enum specifically represents a contiguous range from 0 to 7
            // inclusive, and the ASCII digit range is just this shifted over by a constant amount
            // (in this case, 0x30, but the specific number is irrelevant here.)
            [b @ b'0'..=b'7'] => Ok(unsafe { std::mem::transmute(b.wrapping_sub(b'0')) }),
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

//   #####
//  #     # ###### #####  #    # #  ####  ######
//  #       #      #    # #    # # #    # #
//   #####  #####  #    # #    # # #      #####
//        # #      #####  #    # # #      #
//  #     # #      #   #   #  #  # #    # #
//   #####  ###### #    #   ##   #  ####  ######

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceParseError {
    Empty,
    TooLong,
    Invalid,
}

// Ref: https://www.freedesktop.org/software/systemd/man/systemd.unit.html
pub const MAX_SERVICE_LEN: usize = 256;

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

fn is_valid_name_char(ch: u8) -> bool {
    matches!(ch,
        b'0'..=b'9'
        | b'A'..=b'Z'
        | b'a'..=b'z'
        | b':'
        | b'-'
        | b'_'
        | b'.'
        | b'\\'
        | b'@'
    )
}

impl<'a> Service<'a> {
    pub fn from_slice(s: &[u8]) -> Result<Service, ServiceParseError> {
        #[cfg(test)]
        assert_eq!(MAX_SERVICE_LEN, zero_extend_u8_usize(u8::MAX) + 1);

        match s.len() {
            0 => Err(ServiceParseError::Empty),
            1..=MAX_SERVICE_LEN if s.iter().copied().all(is_valid_name_char) => Ok(Service(s)),
            1..=MAX_SERVICE_LEN => Err(ServiceParseError::Invalid),
            _ => Err(ServiceParseError::TooLong),
        }
    }

    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }

    pub fn as_str(&self) -> &str {
        match std::str::from_utf8(self.0) {
            Ok(result) => result,
            // Shouldn't ever happen.
            Err(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
pub mod sd_arbitrary {
    use crate::prelude::*;

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct ArbitraryServiceChar(u8);

    // Note: this must remain sorted by code point.
    static SERVICE_CHARS: &[u8] =
        b"-.0123456789:@ABCDEFGHIJKLMNOPQRSTUVWXYZ\\_abcdefghijklmnopqrstuvwxyz";

    #[test]
    fn test_service_chars_are_sorted() {
        let mut sorted = SERVICE_CHARS.to_vec();
        sorted.sort();
        assert_eq!(&*sorted, SERVICE_CHARS);
    }

    #[test]
    fn test_service_chars_are_correct() {
        for &ch in SERVICE_CHARS {
            assert!(
                super::is_valid_name_char(ch) == SERVICE_CHARS.contains(&ch),
                "'{}'",
                BinaryToDisplay(&[ch])
            );
        }
    }

    impl Arbitrary for ArbitraryServiceChar {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(*g.choose(SERVICE_CHARS).unwrap())
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let max = self.0;
            Box::new(
                SERVICE_CHARS
                    .iter()
                    .cloned()
                    .take_while(move |c| *c < max)
                    .map(Self),
            )
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    pub struct ArbitraryService(Vec<u8>);

    impl ArbitraryService {
        pub fn from_unwrapped(bytes: &[u8]) -> Self {
            if !bytes.iter().cloned().all(super::is_valid_name_char) {
                panic!("{bytes:?} is not a valid service name.");
            }

            // SAFETY: `ArbitaryServiceChar` has the same layout as `u8`, and the value's checked
            // in the above loop to be correct.
            Self(bytes.to_vec())
        }

        pub fn unpack(self) -> Box<[u8]> {
            self.0.into()
        }

        fn as_bytes(&self) -> &[u8] {
            &self.0
        }

        pub fn as_service(&self) -> Service {
            Service::from_slice(self.as_bytes()).unwrap()
        }
    }

    impl fmt::Debug for ArbitraryService {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "ArbitraryService({:?})",
                String::from_utf8_lossy(self.as_bytes())
            )
        }
    }

    impl Arbitrary for ArbitraryService {
        fn arbitrary(g: &mut Gen) -> Self {
            Self(Vec::from_iter(
                (0..=zero_extend_u8_usize(<u8>::arbitrary(g)))
                    .map(|_| <ArbitraryServiceChar>::arbitrary(g).0),
            ))
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let normalized = Vec::from_iter(self.0.iter().cloned().map(ArbitraryServiceChar));
            Box::new(normalized.shrink().filter_map(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(Self(Vec::from_iter(v.into_iter().map(|c| c.0))))
                }
            }))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        let service = Service::from_slice(b"service-name").unwrap();
        assert_eq!(service.as_bytes(), b"service-name");
    }

    #[test]
    fn service_reads_from_max_len_slice() {
        let service = Service::from_slice(&[b'a'; 256]).unwrap();
        assert_eq!(service.as_bytes(), &[b'a'; 256]);
    }

    #[test]
    fn service_reads_from_valid_template_instance_slice() {
        let service = Service::from_slice(b"service-name@id").unwrap();
        assert_eq!(service.as_bytes(), b"service-name@id");
    }

    #[test]
    fn service_accepts_all_lowercase_chars() {
        let service = Service::from_slice(b"abcdefghijklmnopqrstuvwxyz").unwrap();
        assert_eq!(service.as_bytes(), b"abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn service_accepts_all_uppercase_chars() {
        let service = Service::from_slice(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ").unwrap();
        assert_eq!(service.as_bytes(), b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }

    #[test]
    fn service_accepts_all_numbers() {
        let service = Service::from_slice(b"service0123456789").unwrap();
        assert_eq!(service.as_bytes(), b"service0123456789");
    }

    #[test]
    fn service_accepts_numeric_template_instance_id() {
        let service = Service::from_slice(b"service@0123456789").unwrap();
        assert_eq!(service.as_bytes(), b"service@0123456789");
    }

    #[test]
    fn service_accepts_special_chars() {
        let service = Service::from_slice(b"s\\e:r_v.i-c@e:1.2_3\\4").unwrap();
        assert_eq!(service.as_bytes(), b"s\\e:r_v.i-c@e:1.2_3\\4");
    }

    #[test]
    fn service_rejects_empty_names() {
        assert_eq!(Service::from_slice(b""), Err(ServiceParseError::Empty));
    }

    #[test]
    fn service_rejects_too_long_names_with_only_valid_chars() {
        assert_eq!(
            Service::from_slice(&[b'a'; 300]),
            Err(ServiceParseError::TooLong)
        );
    }

    #[test]
    fn service_rejects_too_long_names_with_invalid_chars() {
        assert_eq!(
            Service::from_slice(&[b' '; 300]),
            Err(ServiceParseError::TooLong)
        );
    }

    #[test]
    fn service_rejects_semicolons() {
        assert_eq!(
            Service::from_slice(b"service;foo"),
            Err(ServiceParseError::Invalid)
        );
    }

    #[test]
    fn service_rejects_spaces() {
        assert_eq!(
            Service::from_slice(b"service foo"),
            Err(ServiceParseError::Invalid)
        );
    }
}
