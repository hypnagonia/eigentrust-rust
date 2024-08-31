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

