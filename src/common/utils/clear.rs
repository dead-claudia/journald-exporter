/// Safety note: the caller must only call this if either all-zero is a valid value for `T` or if
/// it's being dropped and the contents are trivially droppable.
pub unsafe fn secure_clear<T>(values: &mut [T]) {
    secure_clear_bytes(std::slice::from_raw_parts_mut(
        values.as_mut_ptr().cast::<u8>(),
        values.len().wrapping_mul(std::mem::size_of::<T>()),
    ))
}

pub fn secure_clear_bytes(bytes: &mut [u8]) {
    // SAFETY: It's only writing to known-valid references. Ideally, I'd be sending it to
    // `libc::memset_s`, but `libc` doesn't expose that.
    unsafe {
        for i in bytes.iter_mut() {
            std::ptr::write_volatile(i, 0);
        }
    }
}
