#!/bin/sh
# Builds the WebAssembly packages from wasm/ and copies them into the demo.
#   pkg/           single-threaded, stable toolchain, works everywhere
#   pkg-parallel/  multithreaded (rayon on Web Workers), nightly + atomics,
#                  needs a cross-origin isolated page (COOP/COEP headers, see demo/vercel.json)
set -e
cd "$(dirname "$0")"

wasm-pack build wasm --target web --release --out-dir ../pkg --out-name eigentrust

RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals' \
    rustup run nightly wasm-pack build wasm --target web --release \
    --out-dir ../pkg-parallel --out-name eigentrust \
    -- --features threads -Z build-std=panic_abort,std

rm -rf demo/pkg demo/pkg-parallel
cp -r pkg demo/pkg
cp -r pkg-parallel demo/pkg-parallel
# wasm-bindgen-rayon imports its package as a directory ('../../..'), which only a bundler resolves
sed -i.bak "s|import('../../..')|import('../../../eigentrust.js')|" demo/pkg-parallel/snippets/*/src/workerHelpers.js
rm -f demo/pkg-parallel/snippets/*/src/workerHelpers.js.bak
rm -f demo/pkg/.gitignore demo/pkg-parallel/.gitignore demo/pkg/README.md demo/pkg-parallel/README.md
