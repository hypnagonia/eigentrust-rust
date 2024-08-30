use crate::basic::engine::calculate_from_csv;
use wasm_bindgen::prelude::*;

pub mod basic;
pub mod sparse;
use crate::basic::util::init_logger;
use std::panic;
use std::str;

#[wasm_bindgen]
pub fn prepare() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_logger();
    log::info!("WASM Eigentrust connected");
}

#[wasm_bindgen]
pub fn run(localtrust_csv: &[u8], pretrust_csv: &[u8], alpha: f64) -> String {
    let lt = str::from_utf8(localtrust_csv).unwrap();
    let pt = str::from_utf8(pretrust_csv).unwrap();

    let result = calculate_from_csv(lt, pt, Some(alpha));
    let json = serde_json::to_string(&result).unwrap();

    json.to_string()
}
