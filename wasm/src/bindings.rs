// JavaScript-facing exports. Only compiled for wasm32.

use wasm_bindgen::prelude::*;

#[cfg(feature = "threads")]
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen(start)]
fn start() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    // the demo recomputes on every slider move, keep the browser console quiet
    let _ = console_log::init_with_level(log::Level::Warn);
}

/// Ranks peers from CSV bytes.
///
/// Returns JSON: `{"Ok": [[peer, score], ...]}` with non-zero scores, highest first,
/// or `{"Err": "message"}`.
#[wasm_bindgen]
pub fn run(local_trust: &[u8], pre_trust: &[u8], alpha: f64) -> String {
    crate::convert::rank_csv_json(local_trust, pre_trust, alpha)
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn run_returns_json() {
        let out = super::run(b"alice,bob,2\nbob,carol,1\n", b"alice\n", 0.5);
        assert!(out.starts_with("{\"Ok\":[[\"alice\","), "{}", out);
        assert!(super::run(b"a,b,NaN", b"a", 0.5).starts_with("{\"Err\":"));
    }
}
