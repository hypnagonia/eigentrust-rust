#![cfg(target_arch = "wasm32")]
use crate::basic::engine::calculate_from_csv;
use wasm_bindgen::prelude::*;

pub mod basic;
pub mod sparse;
use crate::basic::util::init_logger;
use std::panic;
use std::str;

#[cfg(feature = "parallel")]
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_logger();
    log::debug!("WASM Eigentrust connected");
}

#[wasm_bindgen]
// Returns JSON: {"Ok": [[peer, score], ...]} sorted by score, or {"Err": "message"}
pub fn run(localtrust_csv: &[u8], pretrust_csv: &[u8], alpha: f64) -> String {
    let result = match (str::from_utf8(localtrust_csv), str::from_utf8(pretrust_csv)) {
        (Ok(lt), Ok(pt)) => calculate_from_csv(lt, pt, Some(alpha)),
        _ => Err("CSV input is not valid UTF-8".to_string()),
    };

    serde_json::to_string(&result).unwrap_or_else(|e| format!("{{\"Err\":\"{}\"}}", e))
}
