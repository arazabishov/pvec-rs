use pvec::core::RrbVec;
use wasm_bindgen::prelude::*;

// We need to keep state on the WASM side because we rely on the identity of underlying objects.
// If we serialize and send values over to JS we will lose identity of objects, which defeats the whole point of the demo.
static mut STATE: Vec<RrbVec<usize>> = Vec::new();

#[wasm_bindgen]
pub fn push_vec() {
    unsafe { STATE.push(RrbVec::new()) }
}

#[wasm_bindgen]
pub fn set_vec_size(vec_idx: usize, size: usize) {
    unsafe {
        let vec = STATE.get_mut(vec_idx).unwrap();

        if vec.len() < size {
            for i in vec.len()..size {
                vec.push(i);
            }
        } else {
            vec.split_off(size);
        }
    }
}

#[wasm_bindgen]
pub fn get_vec_size(vec_idx: usize) -> usize {
    unsafe { STATE.get(vec_idx).unwrap().len() }
}

#[wasm_bindgen]
pub fn split_off_vec(vec_idx: usize, idx: usize) -> usize {
    unsafe {
        let other = STATE.get_mut(vec_idx).unwrap().split_off(idx);
        let new_vec_idx = vec_idx + 1;

        STATE.insert(new_vec_idx, other);

        new_vec_idx
    }
}

#[wasm_bindgen]
pub fn concatenate(vec_idx_self: usize, vec_idx_that: usize) {
    unsafe {
        // Remove the second vector first to avoid mutable aliasing UB
        let mut vec_that = STATE.remove(vec_idx_that);

        // Adjust index if vec_idx_self was after vec_idx_that
        let adjusted_idx = if vec_idx_self > vec_idx_that {
            vec_idx_self - 1
        } else {
            vec_idx_self
        };

        let vec_self = STATE.get_mut(adjusted_idx).unwrap();
        vec_self.append(&mut vec_that);
    }
}

#[wasm_bindgen]
pub fn concatenate_all() {
    unsafe {
        if STATE.len() <= 1 {
            return;
        }

        // Drain all vectors except the first, then append them one by one
        let others: Vec<_> = STATE.drain(1..).collect();

        let first = STATE.get_mut(0).unwrap();
        for mut other in others {
            first.append(&mut other);
        }
    }
}

#[wasm_bindgen]
pub fn clear() {
    unsafe { STATE.clear() }
}

#[wasm_bindgen]
pub fn get(index: usize) -> JsValue {
    unsafe { JsValue::from_str(serde_json::to_string(&STATE.get(index)).unwrap().as_str()) }
}

#[wasm_bindgen]
pub fn len() -> usize {
    unsafe { STATE.len() }
}
