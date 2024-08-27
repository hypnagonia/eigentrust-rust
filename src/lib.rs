use wasm_bindgen::prelude::*;
use crate::sparse::entry::{Entry};

pub mod sparse;

// This function will be exposed to JavaScript
#[wasm_bindgen]
pub fn add(left: u64, right: u64) -> u64 {
    let e = Entry{index: 1, value: 1.0};
    left + right
}