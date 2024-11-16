#!/bin/sh

set -e
set -x

cargo build --target wasm32-unknown-unknown --release

ic-wasm target/wasm32-unknown-unknown/release/test_canister_b.wasm -o target/wasm32-unknown-unknown/release/test_canister_b.wasm shrink
gzip -k target/wasm32-unknown-unknown/release/test_canister_b.wasm --force

ic-wasm target/wasm32-unknown-unknown/release/test_canister_a.wasm -o target/wasm32-unknown-unknown/release/test_canister_a.wasm shrink
gzip -k target/wasm32-unknown-unknown/release/test_canister_a.wasm --force
