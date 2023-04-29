pub use crate::state::ipc::parent::*;
pub use crate::state::ipc::VERSION_BYTES;

pub const fn index_hex(value: usize) -> u8 {
    b"0123456789ABCDEF"[value % 16]
}

pub fn gen_hex(len: usize) -> Box<[u8]> {
    let mut result = Vec::new();
    for i in 0..len {
        result.push(index_hex(i));
    }
    result.into()
}
