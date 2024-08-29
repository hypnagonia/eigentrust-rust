use wasm_bindgen::prelude::*;

use crate::basic::eigentrust::compute;
use crate::basic::localtrust::canonicalize_local_trust;
use crate::sparse::entry::Entry;
use crate::sparse::matrix::{CSMatrix, CSRMatrix};
use crate::sparse::vector::Vector;

use crate::basic::engine::calculate_from_csv;

pub mod basic;
pub mod sparse;
use std::panic;
use std::str;
use web_sys::console;

#[wasm_bindgen]
pub fn run(localtrust_csv: &[u8], pretrust_csv: &[u8]) -> String {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console::log_1(&"WASM Eigentrust connected".into());

    let lt = str::from_utf8(localtrust_csv).unwrap();
    let pt = str::from_utf8(pretrust_csv).unwrap();

    let result = calculate_from_csv(lt, pt);
    let json = serde_json::to_string(&result).unwrap();

    json.to_string()
}
