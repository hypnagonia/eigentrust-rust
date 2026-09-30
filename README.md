<h1 align="center">EigenTrust in Rust and WebAssembly</h1>

<p align="center">
  A fast implementation of the EigenTrust reputation algorithm.<br>
  Global trust scores for peer-to-peer networks, social graphs and web-of-trust systems, with Sybil attack resistance built in.
</p>

<p align="center">
  <a href="https://crates.io/crates/eigentrust"><img alt="crates.io" src="https://img.shields.io/crates/v/eigentrust"></a>
  <a href="https://docs.rs/eigentrust"><img alt="docs.rs" src="https://img.shields.io/docsrs/eigentrust"></a>
  <a href="https://github.com/hypnagonia/eigentrust-rust/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/hypnagonia/eigentrust-rust/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://eigentrust.jenyadoesapps.com"><img alt="Live demo" src="https://img.shields.io/badge/demo-live-3a3fd0"></a>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021-b7410e?logo=rust&logoColor=white">
  <img alt="WebAssembly" src="https://img.shields.io/badge/WebAssembly-multithreaded-654ff0?logo=webassembly&logoColor=white">
  <a href="#license"><img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue"></a>
</p>

<p align="center">
  <a href="https://eigentrust.jenyadoesapps.com"><img alt="EigenTrust playground: an interactive trust graph ranked live in the browser with WebAssembly" src="https://raw.githubusercontent.com/hypnagonia/eigentrust-rust/main/docs/playground.png" width="820"></a>
</p>

## What is EigenTrust?

EigenTrust turns local trust ("alice trusts bob") into a global reputation score for every peer. Trust spreads from a few seed peers you already trust, so fake accounts that only vouch for each other get almost nothing. With no seeds it behaves like PageRank.

It is the algorithm from the [EigenTrust paper](https://nlp.stanford.edu/pubs/eigentrust.pdf) (Kamvar, Schlosser, Garcia-Molina, 2003), used for reputation in peer-to-peer and decentralized networks, Sybil and spam resistance, and ranking accounts or contributors by who vouches for them.

**Try it: [eigentrust.jenyadoesapps.com](https://eigentrust.jenyadoesapps.com)**, a live playground in 10 languages that can also rank 250,000 peers in the browser.

## Rust library

```toml
[dependencies]
eigentrust = "0.2"
```

```rust
use eigentrust::{eigentrust, PreTrust, TrustEdge};

fn main() -> Result<(), eigentrust::EigenTrustError> {
    let local_trust = [
        TrustEdge::new(0, 1, 2.0), // peer 0 trusts peer 1 with weight 2
        TrustEdge::new(0, 2, 1.0),
        TrustEdge::new(1, 2, 1.0),
        TrustEdge::new(2, 0, 1.0),
    ];
    let pre_trust = [PreTrust::new(0, 1.0)]; // peer 0 is the seed

    let result = eigentrust(local_trust, pre_trust)?;
    for (peer, score) in result.ranking() {
        println!("{peer}: {score:.4}");
    }
    Ok(())
}
```

- `eigentrust_with_options` takes `EigenTrustOptions` for `alpha` (default 0.5), the convergence threshold and the iteration limit.
- Results come back as `TrustScores`: one score per peer, summing to 1, plus the iteration count and residual.
- Errors are a typed `EigenTrustError`.
- Optional features:
  - `csv` adds `eigentrust::csv::Network` for named peers from CSV.
  - `parallel` runs the iteration on all cores with rayon.
- There are no required dependencies beyond `log`, and the same code builds for `wasm32`.

The [documentation on docs.rs](https://docs.rs/eigentrust) covers the exact input rules and convergence behavior. See also [`examples/`](examples).

## Command line

```sh
cargo install eigentrust-cli
eigentrust localtrust.csv pretrust.csv [alpha]
```

```text
alice,0.6666666865348816
bob,0.3333333134651184
```

It prints `peer,score` for every peer with a non-zero score, highest first. Errors go to stderr with exit code 1.

## Browser (WebAssembly)

```js
import init, { run } from './pkg/eigentrust.js'

await init()
const enc = new TextEncoder()
const result = JSON.parse(run(enc.encode('alice,bob,2\nbob,carol,1\n'), enc.encode('alice\n'), 0.5))
// { Ok: [["alice", ...], ["bob", ...], ["carol", ...]] }  or  { Err: "..." }
```

- `./build.sh` builds `pkg/` and a multithreaded `pkg-parallel/` from the [`wasm`](wasm) crate.
- The multithreaded build needs a cross-origin isolated page; see [`demo/vercel.json`](demo/vercel.json).
- [`demo/worker.js`](demo/worker.js) runs the engine in a Web Worker and picks the right build.

## Input rules

| Input | CSV line | Rust type |
| --- | --- | --- |
| Local trust | `from,to[,weight]` | `TrustEdge { from, to, weight }` |
| Pre-trust (seeds) | `peer[,weight]` | `PreTrust { peer, weight }` |

- **Weights:** finite and non-negative, default 1. Zero means no trust. Negative trust (distrust) is rejected.
- **Normalization:** each truster's weights are scaled to sum to 1, and so is pre-trust. With no pre-trust, every peer starts equal.
- **Duplicates:** a repeated edge or seed keeps its last weight.
- **Convergence:** at α = 0 some networks oscillate instead of converging. The engine stops after 10,000 iterations with an error.
- **CSV:** standard CSV, so quoted fields (`"Smith, J"`), a header row, spaces, CRLF and a UTF-8 BOM are fine. Errors name the file and line.

## Performance

Random trust graphs, 10 links per peer, CSV parsing included.

| Peers | Links | CLI | Browser |
| ---: | ---: | ---: | ---: |
| 20,000 | 200,000 | 0.06 s | 58 ms |
| 100,000 | 1,000,000 | 0.3 s | 0.2 s |
| 250,000 | 2,500,000 | | 0.6 s |

Results are reproducible bit for bit, with or without threads.

## Repository layout

| Path | Crate | Role |
| --- | --- | --- |
| [`src/`](src) | `eigentrust` | the library: one implementation of the algorithm |
| [`cli/`](cli) | `eigentrust-cli` | the `eigentrust` command, built on the library's public API |
| [`wasm/`](wasm) | `eigentrust-wasm` | JavaScript bindings, built on the same API |
| [`demo/`](demo) | | the web playground |

## Development

```sh
cargo test --workspace --all-features   # tests, including doc tests
cargo run --example basic               # library example
./build.sh                              # WASM builds, copied into demo/
python3 -m http.server -d demo          # run the playground locally
git config core.hooksPath .githooks     # once per clone
```

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
