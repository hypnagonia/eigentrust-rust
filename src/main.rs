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

fn main() {
    let result = calculate();
    println!("{:?}", result);
}
