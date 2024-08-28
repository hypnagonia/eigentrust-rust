use wasm_bindgen::prelude::*;

use crate::basic::eigentrust::compute;
use crate::basic::localtrust::canonicalize_local_trust;
use crate::sparse::entry::Entry;
use crate::sparse::matrix::{CSMatrix, CSRMatrix};
use crate::sparse::vector::Vector;

use crate::basic::engine::calculate;

pub mod basic;
pub mod sparse;
use std::panic;
use web_sys::console;

#[wasm_bindgen]
pub fn run(left: u64, right: u64) -> String {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console::log_1(&"WASM Eigentrust connected".into());

    let result = calculate();
    let json = serde_json::to_string(&result).unwrap();

    json.to_string()
}
