use std::alloc::alloc;
use std::alloc::Layout;

// pub fn try_new_box<T>(value: T) -> Option<Box<T>> {
//     // SAFETY: The pointer's allocated in accordance with the contract for boxes.
//     unsafe {
//         let layout = Layout::new::<T>();
//         let ptr = alloc(layout);
//         if ptr.is_null() {
//             None
//         } else {
//             let ptr: *mut T = ptr.cast();
//             ptr.write(value);
//             Some(Box::from_raw(ptr))
//         }
//     }
// }

pub fn try_new_fixed_vec<T, const N: usize>() -> Option<Box<heapless::Vec<T, N>>> {
    // SAFETY: The pointer's allocated in accordance with the contract for boxes.
    unsafe {
        let layout = Layout::new::<heapless::Vec<T, N>>();
        let ptr = alloc(layout);
        if ptr.is_null() {
            None
        } else {
            let ptr: *mut heapless::Vec<T, N> = ptr.cast();
            // Most of this *should* get optimized out since it's all technically uninitialized.
            ptr.write(heapless::Vec::new());
            Some(Box::from_raw(ptr))
        }
    }
}

pub fn try_new_dynamic_vec<T>(capacity: usize) -> Option<Vec<T>> {
    // Nothing to allocate. Just return an empty vector.
    if capacity == 0 {
        return Some(Vec::new());
    }

    // SAFETY: The pointer's allocated in accordance with the contract for vecs.
    // See https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts for details -
    // there's a lot of considerations here.
    unsafe {
        let layout = match Layout::array::<T>(capacity) {
            Ok(layout) => layout,
            Err(_) => return None,
        };

        let ptr = alloc(layout);
        if ptr.is_null() {
            None
        } else {
            Some(Vec::from_raw_parts(ptr.cast(), 0, capacity))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_fixed_vec_returns_correct_dimensions() {
        let result = try_new_fixed_vec::<u32, 5>().unwrap();
        assert_eq!(result.capacity(), 5);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn try_new_dynamic_vec_returns_correct_dimensions() {
        let result = try_new_dynamic_vec::<u32>(5).unwrap();
        assert_eq!(result.capacity(), 5);
        assert_eq!(result.len(), 0);
    }
}
