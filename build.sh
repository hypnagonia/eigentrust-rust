#!/bin/sh

# set -ex

# cargo build --target wasm32-unknown-unknown --release

wasm-pack build --target web --release
  