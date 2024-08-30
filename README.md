## Rust (WASM) port of Eigentrust
This is a Rust port of the [EigenTrust](https://nlp.stanford.edu/pubs/eigentrust.pdf) algorithm, originally implemented in [Go](https://github.com/Karma3Labs/go-eigentrust). The project includes both server and client implementations designed to run natively as well as in a **WebAssembly environment for browser compatibility**.

## Live Demo
[https://eigentrust2.web.app](https://eigentrust.web.app)

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

## run OS native
```
cargo run ./example/localtrust.csv ./example/pretrust2.csv
```

## test web??
```
wasm-pack test --headless --chrome

wasm-pack build --target web
```


## Important
* wasm-pack must be not above 0.12.1 to support multithreading

### misc 
wasm-pack test --node

wasm-pack build --target web -- --features "threads"

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
  rustup run nightly-2022-12-12 \
  wasm-pack build --target web [...] \
  -- -Z build-std=panic_abort,std

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' cargo build --target wasm32-unknown-unknown \ rustup run nightly-2022-12-12 \ wasm-pack build --target web


rustup component add rust-src --toolchain nightly-2024-06-13-x86_64-apple-darwin

wasm-bindgen-futures = "0.4" 
console_error_panic_hook = "0.1"
console_log = "0.2"