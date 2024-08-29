## Rust (WASM) port of Eigentrust
This is a Rust port of the [EigenTrust](https://nlp.stanford.edu/pubs/eigentrust.pdf) algorithm, originally implemented in [Go](https://github.com/Karma3Labs/go-eigentrust). The project includes both server and client implementations designed to run natively as well as in a **WebAssembly (Wasm) environment for browser compatibility**.

## Live Demo
[https://eigentrust.web.app](https://eigentrust.web.app)

## Test
```
cargo test
```

## Build WASM for web
```
wasm-pack build --target web

python3 -m http.server

http://localhost:8000/index.html
```

## Call from browser environment
```js
import init, { prepare, run } from './pkg/eigentrust_js.js'
init().then(prepare)

    async function main() {
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


### misc 
wasm-pack test --node

wasm-pack build --target web -- --features "threads"