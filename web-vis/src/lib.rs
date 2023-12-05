use pvec::core::RrbVec;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

// We need to keep state on the WASM side because we rely on the identity of underlying objects.
// If we serialize and send values over to JS we will lose identity of objects, which defeats the whole point of the demo.
thread_local! {
    static STATE: RefCell<Vec<RrbVec<usize>>> = RefCell::new(Vec::new());
}

#[wasm_bindgen]
pub fn push_vec() {
    STATE.with(|state| state.borrow_mut().push(RrbVec::new()));
}

#[wasm_bindgen]
pub fn set_vec_size(vec_idx: usize, size: usize) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let vec = state.get_mut(vec_idx).unwrap();

        if vec.len() < size {
            for i in vec.len()..size {
                vec.push(i);
            }
        } else {
            vec.split_off(size);
        }
    });
}

#[wasm_bindgen]
pub fn get_vec_size(vec_idx: usize) -> usize {
    STATE.with(|state| state.borrow().get(vec_idx).unwrap().len())
}

#[wasm_bindgen]
pub fn split_off_vec(vec_idx: usize, idx: usize) -> usize {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let other = state.get_mut(vec_idx).unwrap().split_off(idx);
        let new_vec_idx = vec_idx + 1;

        state.insert(new_vec_idx, other);

        new_vec_idx
    })
}

#[wasm_bindgen]
pub fn concatenate(vec_idx_self: usize, vec_idx_that: usize) {
    unsafe {
        let vec_self = STATE.get_mut(vec_idx_self).unwrap();
        let vec_that = STATE.get_mut(vec_idx_that).unwrap();

        vec_self.append(vec_that);

        // Ensure that empty vector is removed.
        STATE.remove(vec_idx_that);
    }
}

#[wasm_bindgen]
pub fn concatenat_all() {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.len() > 1 {
            let mut others: Vec<_> = state.drain(1..).collect();
            let first = state.first_mut().unwrap();
            for other in others.iter_mut() {
                first.append(other);
            }
        }
    });
}

#[wasm_bindgen]
pub fn clear() {
    STATE.with(|state| state.borrow_mut().clear());
}

#[wasm_bindgen]
pub fn get(index: usize) -> JsValue {
    STATE.with(|state| {
        let state = state.borrow();
        JsValue::from_str(serde_json::to_string(&state.get(index)).unwrap().as_str())
    })
}

#[wasm_bindgen]
pub fn len() -> usize {
    STATE.with(|state| state.borrow().len())
}
