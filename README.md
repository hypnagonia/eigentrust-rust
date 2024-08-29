## Rust (WASM) port of Eigentrust

## Live Demo
[https://eigentrust.web.app](https://eigentrust.web.app)

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


## run OS native
```
cargo run
```



## test web??
```
wasm-pack test --headless --chrome

wasm-pack build --target web
```


### misc 
wasm-pack test --node

wasm-pack build --target web -- --features "threads"