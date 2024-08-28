## Rust (WASM) port of Eigentrust


## test
```
cargo test
```

## build WASM
```
wasm-pack build --target web

python3 -m http.server

http://localhost:8000/index.html
```


## run binary main.rs
```
./target/release/eigentrust-js
```

## test web??
```
wasm-pack test --headless --chrome

wasm-pack build --target web
```


### misc 
wasm-pack test --node

wasm-pack build --target web -- --features "threads"