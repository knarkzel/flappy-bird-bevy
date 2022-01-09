# flappy-bird

`flappy-bird` in Rust using genetic algorithms and neural networks for artificial intelligence.

## Native

```bash
cargo run --release
```

## Web

Install following:

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-server-runner
```

Then run with following:

```bash
cargo run --release --target wasm32-unknown-unknown
```
