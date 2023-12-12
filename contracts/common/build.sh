#!/bin/sh

echo ">> Building CommonContract"

rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release
rm ../grid/res/token.wasm
cp target/wasm32-unknown-unknown/release/common.wasm ../grid/res/token.wasm
