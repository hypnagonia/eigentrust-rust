wasm-pack test --headless --chrome

wasm-pack build --target web

wasm-pack test --node

wasm-pack build --target web -- --features "threads"