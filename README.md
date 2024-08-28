## Rust (WASM) port of Eigentrust


## test
```
cargo test
```

## build WASM
```
wasm-pack build --target web
```

## test web??
```
wasm-pack test --headless --chrome

wasm-pack build --target web
```


### misc 
wasm-pack test --node

wasm-pack build --target web -- --features "threads"