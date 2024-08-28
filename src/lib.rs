use wasm_bindgen::prelude::*;

use crate::basic::localtrust::{ canonicalize_local_trust };
use crate::basic::eigentrust::{ compute };
use crate::sparse::entry::{ Entry };
use crate::sparse::vector::{ Vector };
use crate::sparse::matrix::{ CSRMatrix, CSMatrix };

use crate::basic::compute::run_compute;

pub mod sparse;
pub mod basic;

#[wasm_bindgen]
pub fn run(left: u64, right: u64) -> u64 {
    // let result = run_compute();

    1
}
