extern crate pvec;
extern crate serde_json;

use pvec::core::RrbVec;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn gen_vec(size: usize) -> String {
    let mut vec = RrbVec::new();

    for i in 0..size {
        vec.push(i);
    }

    return serde_json::to_string(&vec).unwrap();
}
