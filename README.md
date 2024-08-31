## Rust (WASM) port of Eigentrust
This is a Rust port of the [EigenTrust](https://nlp.stanford.edu/pubs/eigentrust.pdf) algorithm, originally implemented in [Go](https://github.com/Karma3Labs/go-eigentrust). The project includes both server and client implementations designed to run natively as well as in a **WebAssembly environment for browser compatibility**.

## Live Demo
[https://eigentrust.web.app](https://eigentrust.web.app)

## Test
```
cargo test
```

## Build WASM for web

```
wasm-pack build --target web

wasm-pack build --target web --release

python3 -m http.server

http://localhost:8000/index.html
```

## Call from browser environment
```js
import init, { prepare, run } from './pkg/eigentrust_js.js'
    async function main() {
        await init().then(prepare)

        console.time("eigentrust job")
        const result = run(localtrustBytes, pretrustBytes)
        console.timeEnd("eigentrust job")
        console.log({result})
    }

main()
```

## Run OS native
```
cargo run ./example/localtrust.csv ./example/pretrust2.csv
```

### Build 
```
cargo build --release
./target/release/eigentrust-rs ./tmp/trust-db.csv ./tmp/seed-db.csv
```

--------
## test web??
```
wasm-pack test --headless --chrome

wasm-pack build --target web
```


## Important


### misc 
wasm-pack test --node

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  rustup run nightly-2022-12-12 \
  wasm-pack build --target web [...] \
  -- -Z build-std=panic_abort,std

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' cargo build --target wasm32-unknown-unknown \ rustup run nightly-2022-12-12 \ wasm-pack build --target web

rustup component add rust-src --toolchain nightly-2024-06-13-x86_64-apple-darwin


web assembly threads
https://github.com/RReverser/wasm-bindgen-rayon
dis
Cross-Origin-Embedder-Policy: require-corp
Cross-Origin-Opener-Policy: same-origin

export RUSTFLAGS=\"-C target-feature=+atomics,+bulk-memory,+mutable-globals\" rustup run nightly-2022-12-12 wasm-pack build --target web --out-dir pkg-parallel -- --features parallel -Z build-std=panic_abort,std


rustup component add rust-src --toolchain nightly-x86_64-apple-darwin

rustup run nightly  wasm-pack build --target web 



rustup default nightly

```
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' 
  rustup run nightly 
  wasm-pack build --target web
  -- -Z build-std=panic_abort,std
```

rustc -vV

cargo build --release -- target


cargo build --target x86_64-apple-darwin --release
./target/x86_64-apple-darwin/release/eigentrust-rs ./tmp/trust-db.csv ./tmp/seed-db.csv

edit path in
workerHelpers.worker.js