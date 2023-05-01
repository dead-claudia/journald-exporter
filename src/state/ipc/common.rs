use crate::prelude::*;

pub fn unknown_byte(byte: u8) -> ! {
    let mut result = *b"Unknown IPC byte '  '";
    [result[18], result[19]] = to_hex_pair(byte);
    // It's guaranteed pure ASCII.
    match std::str::from_utf8(&result) {
        Ok(result) => panic!("{}", result),
        Err(_) => unreachable!(),
    }
}

pub fn unknown_version(version: u32) -> ! {
    panic!("Bad version ID: {}", version)
}

pub struct ReadPhase(u32);

impl fmt::Debug for ReadPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // I want it to be small output always.
        write!(f, "ReadPhase({:#018X})", self.0)
    }
}

impl ReadPhase {
    pub const fn new() -> ReadPhase {
        ReadPhase(0x01)
    }
}

#[derive(Debug)]
pub struct KeyAccumulator {
    remaining: usize,
    // Use an option to mask when the vec failed to allocate.
    data: Option<KeySetBuilder>,
}

impl KeyAccumulator {
    pub fn new(len: u32) -> Self {
        Self {
            remaining: zero_extend_u32_usize(len),
            data: KeySetBuilder::try_reserve(zero_extend_u32_usize(len)),
        }
    }

    pub fn has_remaining(&self) -> bool {
        self.remaining != 0
    }

    pub fn finish(self) -> Option<KeySet> {
        debug_assert_eq!(self.remaining, 0);
        self.data.map(|d| d.finish())
    }

    pub fn push_raw(&mut self, value: &[u8]) {
        match self.remaining.checked_sub(1) {
            None => panic!("Overflowed slice buffer!"),
            Some(remaining) => self.remaining = remaining,
        }

        if let Some(data) = &mut self.data {
            // FIXME: change this to https://github.com/rust-lang/rust/issues/100486 to ensure the
            // invariant that it never re-allocates is asserted.
            // SAFETY: It's assumed valid.
            unsafe {
                data.push_raw(value);
            }
        }
    }
}

#[derive(Debug)]
pub struct ByteAccumulator {
    remaining: usize,
    // Use an option to mask when the vec failed to allocate.
    data: Option<Vec<u8>>,
}

impl ByteAccumulator {
    pub fn new(len: u32) -> Self {
        Self {
            remaining: zero_extend_u32_usize(len),
            data: try_new_dynamic_vec(zero_extend_u32_usize(len)),
        }
    }

    #[cfg(test)]
    fn bytes_initialized(&self) -> usize {
        self.data.as_ref().map_or(0, |d| d.len())
    }

    pub fn has_remaining(&self) -> bool {
        self.remaining != 0
    }

    pub fn initialized(&self) -> &[u8] {
        self.data.as_deref().unwrap_or(&[])
    }

    pub fn finish(self) -> Option<Box<[u8]>> {
        debug_assert_eq!(self.remaining, 0);
        self.data.map(|d| d.into())
    }

    pub fn push_from_iter(&mut self, iter: &mut ReadIter) -> bool {
        if iter.buf.is_empty() {
            return false;
        }

        let remaining_len = self.remaining;

        // Nothing to consume, so just return an empty boxed slice.
        if remaining_len > 0 {
            let extend_len = iter.buf.len().min(remaining_len);
            let (head, tail) = iter.buf.split_at(extend_len);

            if let Some(data) = &mut self.data {
                data.extend_from_slice(head);
            }

            iter.buf = tail;
            self.remaining = remaining_len.wrapping_sub(extend_len);
        }

        true
    }
}

pub struct ReadIter<'a> {
    buf: &'a [u8],
}

impl fmt::Debug for ReadIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadIter")
            // Only the recent-ish bytes matter anyways.
            .field("buf", &DebugBigSlice(self.buf))
            .finish()
    }
}

impl<'a> ReadIter<'a> {
    pub const fn new(buf: &'a [u8]) -> Self {
        ReadIter { buf }
    }

    pub fn remaining(self) -> &'a [u8] {
        self.buf
    }

    pub fn next(&mut self) -> Option<u8> {
        let (&value, tail) = self.buf.split_first()?;
        self.buf = tail;
        Some(value)
    }

    // Technically speaking, `ReadPhase` *could* be reused, but it's easier to guarantee safety
    // without it.
    pub fn phase_next_32(&mut self, phase: &mut ReadPhase) -> Option<u32> {
        // FIXME: remove once https://github.com/rust-lang/rust-clippy/pull/10309 is released.
        #![allow(clippy::arithmetic_side_effects)]

        let ReadPhase(mut current) = *phase;

        while let Some(byte) = self.next() {
            // Why this works:
            // Step 0: acc = DD????, byte = ??, step = 1
            // Step 1: acc = CCDD??, byte = ??, step = 2
            // Step 2: acc = BBCCDD, byte = ??, step = 3
            // Step 3: acc = AABBCC, byte = DD, step = 0, return `AABBCCDD`
            // Step 0: acc = ddAABB, byte = CC, step = 1
            // Step 1: acc = ccddAA, byte = BB, step = 2
            // Step 2: acc = bbccdd, byte = AA, step = 3
            // Step 3: acc = aabbcc, byte = dd, step = 0, return `aabbccdd`
            // The inner value (`phase.0`) is stored as `acc << 8 | step` as it's simple and easy
            // to mix into this operation. (Shifting the bytes by 8 also conveniently happens to
            // discard the step.)
            let prev = current;
            let next = prev >> 8 | zero_extend_u8_u32(byte) << 24;
            current = next & !0xFF | ((prev + 1) % 4);

            if prev & 0xFF == 0 {
                *phase = ReadPhase(current);
                return Some(next);
            }
        }

        *phase = ReadPhase(current);
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::bool_assert_comparison)]

    use super::*;

    #[test]
    fn read_iter_next_works_for_empty_buf() {
        let mut iter = ReadIter::new(&[]);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_next_works() {
        let buf = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut iter = ReadIter::new(&buf);

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), Some(7));
        assert_eq!(iter.next(), Some(8));
        assert_eq!(iter.next(), Some(9));
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_byte_acc_at_start_reads_to_end_on_incomplete() {
        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut target = ByteAccumulator::new(16);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), true);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_byte_acc_at_start_reads_to_end_on_exact() {
        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut target = ByteAccumulator::new(10);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), false);
        assert_eq!(&*target.finish().unwrap(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_byte_acc_at_start_reads_to_end_on_overflow() {
        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let mut target = ByteAccumulator::new(10);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), false);
        assert_eq!(&*target.finish().unwrap(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(
            Vec::from_iter(std::iter::from_fn(move || iter.next())),
            vec![11, 12, 13, 14, 15]
        );
    }

    #[test]
    fn read_iter_write_to_byte_acc_at_start_empty_returns_err() {
        let mut target = ByteAccumulator::new(10);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(&[])), false);
        assert_eq!(target.bytes_initialized(), 0);
        assert_eq!(target.has_remaining(), true);
    }

    #[test]
    fn read_iter_write_to_byte_acc_at_middle_reads_to_end_on_incomplete() {
        let mut target = ByteAccumulator::new(16);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(b"ABC")), true);

        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 13);
        assert_eq!(target.has_remaining(), true);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_byte_acc_at_middle_reads_to_end_on_exact() {
        let mut target = ByteAccumulator::new(10);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(b"ABC")), true);

        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), false);
        assert_eq!(
            &*target.finish().unwrap(),
            &[b'A', b'B', b'C', 1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_byte_acc_at_middle_reads_to_end_on_overflow() {
        let mut target = ByteAccumulator::new(10);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(b"ABC")), true);

        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), false);
        assert_eq!(
            &*target.finish().unwrap(),
            &[b'A', b'B', b'C', 1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(
            Vec::from_iter(std::iter::from_fn(move || iter.next())),
            vec![8, 9, 10, 11, 12, 13, 14, 15]
        );
    }

    #[test]
    fn read_iter_write_to_byte_acc_at_middle_empty_returns_err() {
        let mut target = ByteAccumulator::new(10);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(b"ABC")), true);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(&[])), false);
        assert_eq!(target.bytes_initialized(), 3);
        assert_eq!(target.has_remaining(), true);
    }
}
