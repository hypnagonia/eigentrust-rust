# CLAUDE.md

## Git: no Claude attribution, ever

- NEVER add `Co-Authored-By: Claude ...`, `noreply@anthropic.com`, "Generated with Claude Code"
  or any other Claude / Anthropic attribution to commit messages, PR titles or PR descriptions.
  This project was written by its author before Claude was involved. Plain commit messages only.
- This is enforced by `.githooks/commit-msg` and by CI. Enable the hook once per clone:
  `git config core.hooksPath .githooks`
- Never bypass it with `--no-verify`.

## Project

EigenTrust in Rust, runs natively and as WASM in the browser.

- `src/basic/engine.rs` - `calculate_from_csv`: CSV in, sorted `(peer, score)` out
- `src/basic/input.rs` - CSV reading (csv crate), header detection, weight validation
- `src/basic/eigentrust.rs` - power iteration (`compute`), runs on dense vectors
- `src/error.rs` - `Error` enum used everywhere; `Display` gives user-facing messages
- `src/sparse/` - CSR matrix / sparse vector
- `src/lib.rs` - wasm-bindgen entry `run(localtrust, pretrust, alpha)` (wasm32 only)
- `src/main.rs` - native CLI, reports errors as `error: ...` with exit code 1
- `demo/` - static interactive web demo deployed to Vercel

## Commands

- Test: `cargo test --release`
- WASM tests: `wasm-pack test --node --release`
- Lint: `cargo fmt --check`, `cargo clippy --release --all-targets -- -D warnings`
  (also with `--target wasm32-unknown-unknown`)
- CLI: `cargo run --release -- ./example/localtrust.csv ./example/pretrust.csv [alpha]`
- WASM: `./build.sh` (builds `pkg/` and `pkg-parallel/` and copies them into `demo/`)
- Demo locally: `python3 -m http.server -d demo` then open http://localhost:8000
- Deploy: `vercel deploy --prod` from `demo/`

## Notes

- Tests compare floats with exact equality. The iteration uses Kahan-Babuska-Neumaier summation
  in row order; keep the summation order if you touch `compute` / `mul_dense`.
- Duplicate `(i, j)` local trust records and duplicate pre-trust peers: the last one wins.
- Weights must be finite and non-negative. Distrust (negative weights) is rejected until its
  effect on the scores is defined; `extract_distrust` / `discount_trust_vector` are kept for that.
- `compute` stops after `DEFAULT_MAX_ITERATIONS` (10,000) with `Error::NotConverged`.
