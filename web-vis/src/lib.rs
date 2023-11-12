extern crate pvec;
extern crate serde_json;

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
pub fn concatenat_all() {
    unsafe {
        let mut first = STATE.first_mut();
        if let Some(ref mut accumulator) = first {
            for i in 1..STATE.len() {
                let other = STATE.get_mut(i).unwrap();
                accumulator.append(other);
            }
        }
        STATE.truncate(1);
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
