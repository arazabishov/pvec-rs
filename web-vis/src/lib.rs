use pvec::core::RrbVec;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::prelude::*;

// web-vis only works with branching factor 4 (small_branch feature on pvec).
// Redefined here to avoid leaking pvec internals through a pub re-export.
const BRANCH_FACTOR: usize = 4;

#[wasm_bindgen]
pub fn branch_factor() -> usize {
    BRANCH_FACTOR
}

type VecId = String;

struct State {
    vectors: HashMap<VecId, RrbVec<usize>>,
    order: Vec<VecId>,
}

impl State {
    fn new() -> Self {
        State {
            vectors: HashMap::new(),
            order: Vec::new(),
        }
    }

    fn new_id() -> VecId {
        Uuid::new_v4().to_string()
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new());
}

#[wasm_bindgen]
pub fn push_vec() -> String {
    STATE.with(|state| {
        let mut s = state.borrow_mut();
        let id = State::new_id();
        s.vectors.insert(id.clone(), RrbVec::new());
        s.order.push(id.clone());
        id
    })
}

#[wasm_bindgen]
pub fn set_vec_size(vec_id: String, size: usize) {
    STATE.with(|state| {
        let mut s = state.borrow_mut();
        let vec = s.vectors.get_mut(&vec_id).expect("Vector not found");

        if vec.len() < size {
            for i in vec.len()..size {
                vec.push(i);
            }
        } else {
            vec.split_off(size);
        }
    })
}

#[wasm_bindgen]
pub fn get_vec_size(vec_id: String) -> usize {
    STATE.with(|state| {
        state
            .borrow()
            .vectors
            .get(&vec_id)
            .expect("Vector not found")
            .len()
    })
}

#[wasm_bindgen]
pub fn split_off_vec(vec_id: String, idx: usize) -> String {
    STATE.with(|state| {
        let mut s = state.borrow_mut();

        let vec = s.vectors.get_mut(&vec_id).expect("Vector not found");
        let other = vec.split_off(idx);

        let new_id = State::new_id();

        s.vectors.insert(new_id.clone(), other);

        let pos = s.order.iter().position(|id| *id == vec_id).unwrap();
        s.order.insert(pos + 1, new_id.clone());

        new_id
    })
}

#[wasm_bindgen]
pub fn concatenate(vec_id_self: String, vec_id_that: String) {
    STATE.with(|state| {
        let mut s = state.borrow_mut();

        // Remove the second vector
        let mut vec_that = s
            .vectors
            .remove(&vec_id_that)
            .expect("Second vector not found");

        // Remove from order
        let pos = s.order.iter().position(|id| *id == vec_id_that).unwrap();
        s.order.remove(pos);

        // Append to first vector
        let vec_self = s
            .vectors
            .get_mut(&vec_id_self)
            .expect("First vector not found");
        vec_self.append(&mut vec_that);
    })
}

#[wasm_bindgen]
pub fn concatenate_all() {
    STATE.with(|state| {
        let mut s = state.borrow_mut();

        if s.order.len() <= 1 {
            return;
        }

        let first_id = s.order[0].clone();
        let other_ids: Vec<VecId> = s.order.drain(1..).collect();

        let others: Vec<RrbVec<usize>> = other_ids
            .iter()
            .map(|id| s.vectors.remove(id).unwrap())
            .collect();

        let first = s.vectors.get_mut(&first_id).unwrap();
        for mut other in others {
            first.append(&mut other);
        }
    })
}

#[wasm_bindgen]
pub fn clear() {
    STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.vectors.clear();
        s.order.clear();
    })
}

#[wasm_bindgen]
pub fn get(vec_id: String) -> JsValue {
    STATE.with(|state| {
        let s = state.borrow();
        let vec = s.vectors.get(&vec_id);
        vec.serialize(&serde_wasm_bindgen::Serializer::json_compatible()).unwrap()
    })
}

#[wasm_bindgen]
pub fn len() -> usize {
    STATE.with(|state| state.borrow().vectors.len())
}

/// Returns the list of vector IDs in display order
#[wasm_bindgen]
pub fn get_ids() -> Vec<String> {
    STATE.with(|state| state.borrow().order.clone())
}
