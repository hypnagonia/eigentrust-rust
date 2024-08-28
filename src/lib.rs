use wasm_bindgen::prelude::*;

use crate::basic::localtrust::{ canonicalize_local_trust };
use crate::basic::eigentrust::{ compute };
use crate::sparse::entry::{ Entry };
use crate::sparse::vector::{ Vector };
use crate::sparse::matrix::{ CSRMatrix, CSMatrix };

use crate::basic::compute::run_compute;

pub mod sparse;
pub mod basic;
use web_sys::console;
use std::panic;

#[wasm_bindgen]
pub fn run(left: u64, right: u64) -> u64 {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    
    let result = run_compute();
    //let json = serde_json::to_string(&result).unwrap();
    console::log_1(&"Hello from Rust!".into());

    666
}
