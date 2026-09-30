<h1 align="center">EigenTrust</h1>

<p align="center">
  Global trust scores from local trust. Fast, in Rust and WebAssembly.
</p>

<p align="center">
  <a href="https://github.com/hypnagonia/eigentrust-rust/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/hypnagonia/eigentrust-rust/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://eigentrust.jenyadoesapps.com"><img alt="Live demo" src="https://img.shields.io/badge/demo-live-3a3fd0"></a>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021-b7410e?logo=rust&logoColor=white">
  <img alt="WebAssembly" src="https://img.shields.io/badge/WebAssembly-multithreaded-654ff0?logo=webassembly&logoColor=white">
</p>

<p align="center">
  <a href="https://eigentrust.jenyadoesapps.com"><img alt="EigenTrust playground" src="docs/playground.png" width="820"></a>
</p>

## What it does

- **Input:** who trusts whom (`alice,bob,3`), plus a few seed peers you already trust.
- **Output:** a score for every peer. The scores add up to 1.
- **Why:** trust spreads from the seeds, so fake accounts that only vouch for each other get almost nothing.

Based on the [EigenTrust paper](https://nlp.stanford.edu/pubs/eigentrust.pdf) (Kamvar et al., 2003).

## Try it

**[eigentrust.jenyadoesapps.com](https://eigentrust.jenyadoesapps.com)**: edit a network, move the α slider, see the ranking update. You can also rank 250,000 peers in the browser.

## Command line

```sh
cargo run --release -- ./example/localtrust.csv ./example/pretrust.csv [alpha]
```

```
alice,0.6666666865348816
bob,0.3333333134651184
```

α defaults to `0.5`. A higher α keeps trust closer to the seeds.

## Browser

```js
import init, { run } from './pkg/eigentrust.js'

await init()
const enc = new TextEncoder()
const result = JSON.parse(run(enc.encode('alice,bob,2\nbob,carol,1\n'), enc.encode('alice\n'), 0.5))
// { Ok: [["alice", ...], ["bob", ...], ["carol", ...]] }  or  { Err: "..." }
```

Build with `./build.sh`. For a ready-made Web Worker, see [`demo/worker.js`](demo/worker.js).

## Input format

| File | Line | Example |
| --- | --- | --- |
| Local trust | `from,to[,weight]` | `alice,bob,2` |
| Seeds | `peer[,weight]` | `alice,1` |

- A header row is optional. Spaces, quoted fields, CRLF and a UTF-8 BOM are fine.
- Weights default to 1. Repeated `from,to` pairs: the last line wins.
- With no seeds, every peer starts equal (like PageRank).

## Performance

Random networks, 10 links per peer, CSV parsing included.

| Peers | Links | CLI | Browser |
| ---: | ---: | ---: | ---: |
| 20,000 | 200,000 | 0.06 s | 58 ms |
| 100,000 | 1,000,000 | 0.4 s | 0.2 s |
| 250,000 | 2,500,000 | | 0.6 s |

The browser build uses all CPU cores when the page is cross-origin isolated (see [`demo/vercel.json`](demo/vercel.json)).

## Development

```sh
cargo test --release              # tests
./build.sh                        # WASM builds, copied into demo/
python3 -m http.server -d demo    # run the playground locally
git config core.hooksPath .githooks   # once per clone
```
