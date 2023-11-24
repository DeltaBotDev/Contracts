#!/bin/sh

echo ">> Building GridBotContract"

rustup target add wasm32-unknown-unknown
cargo build --all --target wasm32-unknown-unknown --release
