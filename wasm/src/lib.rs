//! WebAssembly bindings for the [`eigentrust`] crate.
//!
//! This crate only converts between JavaScript and the native API: CSV bytes in, JSON out.
//! Build it with `wasm-pack` (see `build.sh`); the multithreaded build enables the `threads`
//! feature and exports `initThreadPool`.

mod convert;

#[cfg(target_arch = "wasm32")]
mod bindings;

#[cfg(target_arch = "wasm32")]
pub use bindings::*;

pub use convert::rank_csv_json;
