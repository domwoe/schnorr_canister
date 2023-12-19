#!/bin/bash
cargo build --release --target wasm32-unknown-unknown --package schnorr_canister_backend
candid-extractor target/wasm32-unknown-unknown/release/schnorr_canister_backend.wasm > src/schnorr_canister_backend/schnorr_canister_backend.did
dfx deploy