pub fn split_req(req: &[u8]) -> [&[u8]; 4] {
    match req.len() {
        0 => panic!("Empty sequences cannot be split."),
        1 => panic!("Single-item sequences cannot be split."),
        2 => panic!("Sequences of length 2 are too short to split 4-way."),
        3 => panic!("Sequences of length 3 are too short to split 4-way."),
        _ => {
            let q1 = req.len() / 4;
            let q2 = req.len() / 2;
            let q3 = q1 + q2;
            [&req[..q1], &req[q1..q2], &req[q2..q3], &req[q3..]]
        }
    }
}
