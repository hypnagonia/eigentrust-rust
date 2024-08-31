#!/bin/sh

set -ex

cargo build --target wasm32-unknown-unknown --release

wasm-bindgen \
  target/wasm32-unknown-unknown/release/eigentrust.wasm \
  --out-dir ./pkg \
  --target no-modules
  