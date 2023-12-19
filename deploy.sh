#!/bin/bash
cargo build --release --target wasm32-unknown-unknown --package schnorr_canister
candid-extractor target/wasm32-unknown-unknown/release/schnorr_canister.wasm > schnorr_canister.did
dfx deploy