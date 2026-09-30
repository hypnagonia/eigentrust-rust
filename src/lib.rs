#![cfg(target_arch = "wasm32")]
use crate::basic::engine::calculate_from_csv;
use crate::error::{Error, Input};
use wasm_bindgen::prelude::*;

pub mod basic;
pub mod error;
pub mod sparse;
use crate::basic::util::init_logger;
use std::panic;
use std::str;

#[cfg(feature = "parallel")]
pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen(start)]
fn start() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_logger();
    log::debug!("WASM Eigentrust connected");
}

#[wasm_bindgen]
// Returns JSON: {"Ok": [[peer, score], ...]} sorted by score, or {"Err": "message"}
pub fn run(localtrust_csv: &[u8], pretrust_csv: &[u8], alpha: f64) -> String {
    let result = str::from_utf8(localtrust_csv)
        .map_err(|_| Error::InvalidUtf8(Input::LocalTrust))
        .and_then(|lt| {
            let pt =
                str::from_utf8(pretrust_csv).map_err(|_| Error::InvalidUtf8(Input::PreTrust))?;
            calculate_from_csv(lt, pt, Some(alpha))
        })
        // the JS side gets the message: {"Err": "local trust CSV, line 3: ..."}
        .map_err(|e| e.to_string());

    serde_json::to_string(&result)
        .unwrap_or_else(|e| serde_json::json!({ "Err": e.to_string() }).to_string())
}

#[cfg(test)]
mod tests {
    use super::run;
    use wasm_bindgen_test::wasm_bindgen_test;

    fn scores(json: &str) -> Vec<(String, f64)> {
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        v["Ok"]
            .as_array()
            .expect("Ok result")
            .iter()
            .map(|e| (e[0].as_str().unwrap().to_string(), e[1].as_f64().unwrap()))
            .collect()
    }

    #[wasm_bindgen_test]
    fn run_returns_sorted_scores_summing_to_one() {
        let out = run(b"alice,bob,2\nbob,carol,1\n", b"alice\n", 0.5);
        let s = scores(&out);
        assert_eq!(
            s.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>(),
            ["alice", "bob", "carol"]
        );
        let total: f64 = s.iter().map(|(_, v)| v).sum();
        assert!((total - 1.0).abs() < 1e-9);
    }

    #[wasm_bindgen_test]
    fn run_reports_errors_as_err() {
        for (lt, pt) in [(&b"a,b,NaN"[..], &b"a"[..]), (b"a,b,-1", b"a"), (b"", b"")] {
            let out = run(lt, pt, 0.5);
            assert!(out.starts_with("{\"Err\":"), "{}", out);
        }
        assert!(run(&[0xff, 0xfe], b"a", 0.5).starts_with("{\"Err\":"));
    }

    #[wasm_bindgen_test]
    fn run_rejects_bad_alpha() {
        assert!(run(b"a,b", b"a", f64::NAN).starts_with("{\"Err\":"));
        assert!(run(b"a,b", b"a", 2.0).starts_with("{\"Err\":"));
    }
}
