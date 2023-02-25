use crate::prelude::*;

pub fn unknown_byte(byte: u8) -> ! {
    let mut result = *b"Unknown IPC byte '  '";
    let (hi, lo) = to_hex_pair(byte);
    result[18] = hi;
    result[19] = lo;
    // It's guaranteed pure ASCII.
    match std::str::from_utf8(&result) {
        Ok(result) => panic!("{}", result),
        Err(_) => unreachable!(),
    }
}

pub fn unknown_version(version: u32) -> ! {
    panic!("Bad version ID: {}", version)
}

#[derive(Clone, Copy)]
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
pub struct SliceAccumulator<T> {
    remaining: usize,
    // This points to the current byte. To get the actual initialized slice, subtract
    // `len - remaining` from the pointer.
    data: Vec<T>,
}

impl<T> SliceAccumulator<T> {
    pub fn new(len: u32) -> Self {
        Self {
            remaining: zero_extend_u32_usize(len),
            data: Vec::with_capacity(zero_extend_u32_usize(len)),
        }
    }

    #[cfg(test)]
    fn bytes_initialized(&self) -> usize {
        self.data.len()
    }

    pub fn has_remaining(&self) -> bool {
        self.remaining != 0
    }

    pub fn initialized(&self) -> &[T] {
        &self.data
    }

    pub fn finish(self) -> Box<[T]> {
        debug_assert_eq!(self.remaining, 0);
        self.data.into()
    }

    pub fn push(&mut self, value: T) {
        match self.remaining.checked_sub(1) {
            None => panic!("Overflowed slice buffer!"),
            Some(remaining) => self.remaining = remaining,
        }

        self.data.push(value)
    }
}

impl SliceAccumulator<u8> {
    pub fn push_from_iter(&mut self, iter: &mut ReadIter) -> bool {
        if iter.buf.is_empty() {
            return false;
        }

        let remaining_len = self.remaining;

        // Nothing to consume, so just return an empty boxed slice.
        if remaining_len > 0 {
            let extend_len = iter.buf.len().min(remaining_len);
            let (head, tail) = iter.buf.split_at(extend_len);

            self.data.extend_from_slice(head);

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
mod test {
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

    #[test]
    fn read_iter_write_to_acc_at_start_reads_to_end_on_incomplete() {
        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut target = SliceAccumulator::new(16);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), true);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_acc_at_start_reads_to_end_on_exact() {
        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut target = SliceAccumulator::new(10);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), false);
        assert_eq!(target.finish(), Box::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_acc_at_start_reads_to_end_on_overflow() {
        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let mut target = SliceAccumulator::new(10);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), false);
        assert_eq!(target.finish(), Box::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
        assert_eq!(
            Vec::from_iter(std::iter::from_fn(move || iter.next())),
            vec![11, 12, 13, 14, 15]
        );
    }

    #[test]
    fn read_iter_write_to_acc_at_start_empty_returns_err() {
        let mut target = SliceAccumulator::new(10);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(&[])), false);
        assert_eq!(target.bytes_initialized(), 0);
        assert_eq!(target.has_remaining(), true);
    }

    #[test]
    fn read_iter_write_to_acc_at_middle_reads_to_end_on_incomplete() {
        let mut target = SliceAccumulator::new(16);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(b"ABC")), true);

        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 13);
        assert_eq!(target.has_remaining(), true);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_acc_at_middle_reads_to_end_on_exact() {
        let mut target = SliceAccumulator::new(10);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(b"ABC")), true);

        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), false);
        assert_eq!(
            target.finish(),
            Box::from([b'A', b'B', b'C', 1, 2, 3, 4, 5, 6, 7])
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn read_iter_write_to_acc_at_middle_reads_to_end_on_overflow() {
        let mut target = SliceAccumulator::new(10);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(b"ABC")), true);

        let mut iter = ReadIter::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);

        assert_eq!(target.push_from_iter(&mut iter), true);
        assert_eq!(target.bytes_initialized(), 10);
        assert_eq!(target.has_remaining(), false);
        assert_eq!(
            target.finish(),
            Box::from([b'A', b'B', b'C', 1, 2, 3, 4, 5, 6, 7])
        );
        assert_eq!(
            Vec::from_iter(std::iter::from_fn(move || iter.next())),
            vec![8, 9, 10, 11, 12, 13, 14, 15]
        );
    }

    #[test]
    fn read_iter_write_to_acc_at_middle_empty_returns_err() {
        let mut target = SliceAccumulator::new(10);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(b"ABC")), true);
        assert_eq!(target.push_from_iter(&mut ReadIter::new(&[])), false);
        assert_eq!(target.bytes_initialized(), 3);
        assert_eq!(target.has_remaining(), true);
    }
}
