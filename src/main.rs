use wasm_bindgen::prelude::*;

use crate::basic::localtrust::{ canonicalize_local_trust };
use crate::basic::eigentrust::{ compute };
use crate::sparse::entry::{ Entry };
use crate::sparse::vector::{ Vector };
use crate::sparse::matrix::{ CSRMatrix, CSMatrix };

use crate::basic::engine::calculate;

pub mod sparse;
pub mod basic;
use web_sys::console;
use std::panic;

fn main() {
    let result = calculate();
    println!("{:?}", result);
}
