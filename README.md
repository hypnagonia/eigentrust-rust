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
wasm-pack build --target web --release

python3 -m http.server

http://localhost:8000/index.html
```

## Call from browser environment
```js
const worker = new Worker('worker.js');
worker.onmessage = function (e) {
        console.log(e.data)
};

const localtrust = `alice,bob,2\nbob,charlie,2\nalice,charlie,1\ncharlie,bob,1\n`
const pretrust = 'alice,1\n'
const localtrustBytes = new TextEncoder().encode(localtrust)
const pretrustBytes = new TextEncoder().encode(pretrust)
const alpha = 0.5
worker.postMessage({ localtrustBytes, pretrustBytes, alpha });
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
