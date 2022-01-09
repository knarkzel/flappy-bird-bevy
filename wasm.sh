#!/bin/sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir docs --target web target/wasm32-unknown-unknown/release/flappy-bird.wasm
wasm-opt -Oz -o docs/flappy-bird_bg.wasm docs/flappy-bird_bg.wasm
ls -alh docs/flappy-bird_bg.wasm 
