# CLAUDE.md

## Git: no Claude attribution, ever

- NEVER add `Co-Authored-By: Claude ...`, `noreply@anthropic.com`, "Generated with Claude Code"
  or any other Claude / Anthropic attribution to commit messages, PR titles or PR descriptions.
  This project was written by its author before Claude was involved. Plain commit messages only.
- This is enforced by `.githooks/commit-msg` and by CI. Enable the hook once per clone:
  `git config core.hooksPath .githooks`
- Never bypass it with `--no-verify`.
- PRs get squash-merged quickly: start follow-up work from a fresh branch off `main`.

## Project

EigenTrust in Rust. Cargo workspace:

- `eigentrust` (root, `src/`) - the library. The only implementation of the algorithm.
  - `lib.rs` - crate docs, the public API (`eigentrust`, `eigentrust_with_options`, re-exports)
  - `types.rs` (`TrustEdge`, `PreTrust`), `options.rs`, `scores.rs` (`TrustScores`), `error.rs`
  - `algorithm/` - private: input validation and normalization (`mod.rs`), CSR matrix,
    power iteration (`power.rs`)
  - `csv.rs` - `csv` feature: named peers from CSV (`csv::Network`), its own error type
  - features: `csv`, `parallel` (rayon); default none
- `eigentrust-cli` (`cli/`) - binary `eigentrust`, uses only the public API
- `eigentrust-wasm` (`wasm/`) - JS bindings; `convert.rs` is plain Rust, `bindings.rs` wasm32 only;
  `threads` feature for the multithreaded build; `publish = false`
- `demo/` - static web playground deployed to Vercel

Keep the public surface small: modules private by default, export through `lib.rs`,
`#![warn(missing_docs)]` is on.

## Commands

- Test: `cargo test --workspace --all-features` (unit, integration, doc tests incl. README)
- Library alone: `cargo test -p eigentrust`
- WASM tests: `wasm-pack test --node --release wasm`
- Lint: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  and `cargo clippy -p eigentrust -p eigentrust-wasm --target wasm32-unknown-unknown --all-targets -- -D warnings`
- Docs: `RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo +nightly doc -p eigentrust --all-features --no-deps`
- Package: `cargo package -p eigentrust -p eigentrust-cli`, check `cargo package -p eigentrust --list`
- CLI: `cargo run --release -p eigentrust-cli -- ./example/localtrust.csv ./example/pretrust.csv [alpha]`
- WASM: `./build.sh` (builds `pkg/` and `pkg-parallel/` from `wasm/` and copies them into `demo/`)
- Demo locally: `python3 -m http.server -d demo` then open http://localhost:8000
- Deploy: `vercel deploy --prod` from `demo/`

## Notes

- Results must stay bit-identical: compensated (KBN) sums in a fixed order in `power.rs`, plain
  sums for normalization, stable sorts. Tests compare floats exactly.
- Duplicate edges and duplicate pre-trust peers: the last one wins.
- Weights must be finite and non-negative; distrust (negative weights) is rejected.
- The iteration stops after `max_iterations` (default 10,000) with `EigenTrustError::NotConverged`.
- Version 0.2.0 is not on crates.io yet; the CI semver job is informational until it is.
