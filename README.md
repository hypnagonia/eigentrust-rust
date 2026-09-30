# EigenTrust

Fast [EigenTrust](https://nlp.stanford.edu/pubs/eigentrust.pdf) in Rust. Runs on the command line and in the browser through WebAssembly.

**Try it: [eigentrust.jenyadoesapps.com](https://eigentrust.jenyadoesapps.com)**. Build a trust network, pick the peers you already trust, and watch every peer get ranked as you edit. The playground can also rank a generated network of 250,000 peers right in the browser.

## What it does

You give it two things:

- **Local trust**: who trusts whom, and how much. `alice,bob,3` means alice trusts bob with weight 3.
- **Seeds (pre-trust)**: a few peers you already trust, such as your own accounts or a vetted core team.

It returns a global trust score for every peer. The scores add up to 1. Trust starts at the seeds and flows along the network. At every step a share **α** of it goes back to the seeds, so a cluster of fake accounts that only vouch for each other can't create trust out of nothing. Raising α tightens that further.

Typical uses are reputation in peer-to-peer and social networks, spam and Sybil resistance, and ranking contributors or accounts by who vouches for them.

## Quick start

```sh
cargo run --release -- ./example/localtrust.csv ./example/pretrust.csv
```

The output is one `peer,score` line per peer, highest first:

```
alice,0.6666666865348816
bob,0.3333333134651184
```

An optional third argument sets α (default `0.5`):

```sh
cargo run --release -- ./example/localtrust2.csv ./example/pretrust2.csv 0.2
```

Set `RUST_LOG=debug` or `RUST_LOG=trace` to follow the iterations.

## Input format

Both files are plain CSV. A header row is optional. Spaces, quoted fields, Windows line endings and a UTF-8 BOM are all handled.

Local trust has one statement per line: `from,to[,weight]`. The weight defaults to 1.

```csv
from,to,weight
alice,bob,2
bob,charlie,2
alice,charlie,1
charlie,bob
```

Seeds have one peer per line: `peer[,weight]`. Weights are normalized.

```csv
peer,weight
alice,1
```

Details worth knowing:

- If the same `from,to` pair appears more than once, the last line wins.
- A peer who trusts nobody passes their trust on to the seeds.
- With no seeds, every peer starts with equal trust, which behaves like PageRank.
- Peer ids are any strings without commas: names, numbers, wallet addresses.

## In the browser

Build the WebAssembly package:

```sh
wasm-pack build --target web --release
```

Then call it, ideally from a Web Worker so the page stays responsive:

```js
import init, { run } from './pkg/eigentrust.js'

await init()

const localtrust = new TextEncoder().encode('alice,bob,2\nbob,charlie,2\nalice,charlie,1\ncharlie,bob,1\n')
const pretrust = new TextEncoder().encode('alice,1\n')

const result = JSON.parse(run(localtrust, pretrust, 0.5))
// { Ok: [["alice", 0.5], ["bob", 0.278], ["charlie", 0.222]] }
// or { Err: "Cannot parse local trust CSV record #3: ..." }
```

[`demo/worker.js`](demo/worker.js) is a ready-made worker. It also uses the multithreaded build when the page allows it.

### Multithreading

There are two WebAssembly builds:

| Build | Toolchain | Where it runs |
| --- | --- | --- |
| `pkg/` | stable | everywhere |
| `pkg-parallel/` | nightly, `+atomics` | pages served with `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` |

The parallel build spreads the matrix-vector product over Web Workers using [wasm-bindgen-rayon](https://github.com/RReverser/wasm-bindgen-rayon). Call `initThreadPool(navigator.hardwareConcurrency)` once before `run`. `./build.sh` builds both.

In practice the iteration is already fast. Most of the time on big inputs goes to reading the CSV, which runs on one thread, so threads give about 5–15% end to end.

## Performance

Measured on a laptop with random networks where each peer trusts 10 others. Times include parsing the CSV.

| Network | Native CLI | Browser, 1 thread | Browser, all threads |
| --- | --- | --- | --- |
| 20,000 peers, 200,000 links | 0.06 s | 58 ms | |
| 100,000 peers, 1,000,000 links | 0.3–0.4 s | 212 ms | 200 ms |
| 250,000 peers, 2,500,000 links | | 709 ms | 614 ms |

Each iteration of the power method costs O(links): it multiplies a sparse matrix by a dense vector, using compensated (Kahan–Babuška) summation for stable results.

## Development

```sh
cargo test --release      # unit tests
./build.sh                # both WASM builds, copied into demo/
python3 -m http.server -d demo 8000   # playground, single-threaded
```

The playground in [`demo/`](demo) is static HTML and JavaScript with no build step. It's deployed to Vercel from that folder, and [`demo/vercel.json`](demo/vercel.json) sets the headers the multithreaded build needs.

After cloning, enable the repository's git hooks once:

```sh
git config core.hooksPath .githooks
```

## Reference

Sepandar D. Kamvar, Mario T. Schlosser, Hector Garcia-Molina. [The EigenTrust Algorithm for Reputation Management in P2P Networks](https://nlp.stanford.edu/pubs/eigentrust.pdf). WWW 2003.
