#!/bin/sh
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir public --target web target/wasm32-unknown-unknown/release/flappy-bird.wasm
wasm-opt -Oz -o public/flappy-bird_bg.wasm public/flappy-bird_bg.wasm
ls -alh public/flappy-bird_bg.wasm 
