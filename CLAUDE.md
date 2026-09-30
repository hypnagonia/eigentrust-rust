# CLAUDE.md

## Git: no Claude attribution, ever

- NEVER add `Co-Authored-By: Claude ...`, `noreply@anthropic.com`, "Generated with Claude Code"
  or any other Claude / Anthropic attribution to commit messages, PR titles or PR descriptions.
  This project was written by its author before Claude was involved. Plain commit messages only.
- This is enforced by `.githooks/commit-msg`. Enable it once per clone:
  `git config core.hooksPath .githooks`
- Never bypass it with `--no-verify`.

## Project

Rust port of EigenTrust (go-eigentrust), runs natively and as WASM in the browser.

- `src/basic/engine.rs` - `calculate_from_csv`: CSV in, sorted `(peer, score)` out
- `src/basic/eigentrust.rs` - power iteration (`compute`), runs on dense vectors
- `src/sparse/` - CSR matrix / sparse vector
- `src/lib.rs` - wasm-bindgen entry `run(localtrust, pretrust, alpha)` (wasm32 only)
- `src/main.rs` - native CLI
- `demo/` - static interactive web demo deployed to Vercel

## Commands

- Test: `cargo test --release`
- CLI: `cargo run --release -- ./example/localtrust.csv ./example/pretrust.csv [alpha]`
- WASM: `./build.sh` (builds `pkg/` and copies it into `demo/pkg/`)
- Demo locally: `python3 -m http.server -d demo` then open http://localhost:8000
- Deploy: `vercel deploy --prod` from `demo/`

## Notes

- Tests compare floats with exact equality. The iteration uses Kahan-Babuska-Neumaier summation
  in row order; keep the summation order if you touch `compute` / `mul_dense`.
- Duplicate `(i, j)` records in local trust: the last one wins.
