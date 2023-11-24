#!/bin/sh

./build.sh

echo ">> Deploying GridBotContract"

near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/grid.wasm
