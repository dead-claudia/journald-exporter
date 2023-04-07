pub use crate::state::ipc::parent::*;
pub use crate::state::ipc::VERSION_BYTES;

pub const fn index_hex(value: usize) -> u8 {
    b"0123456789ABCDEF"[value % 16]
}

pub fn gen_hex(len: usize) -> Box<[u8]> {
    let mut result = vec![0; len];
    for (i, target) in result.iter_mut().enumerate() {
        *target = index_hex(i);
    }
    result.into()
}

pub fn split_req(req: &[u8]) -> [&[u8]; 4] {
    match req.len() {
        0..=3 => panic!("Did you forget to push the version?"),
        _ => {
            let q1 = req.len() / 4;
            let q2 = req.len() / 2;
            let q3 = q1 + q2;
            [&req[..q1], &req[q1..q2], &req[q2..q3], &req[q3..]]
        }
    }
}
