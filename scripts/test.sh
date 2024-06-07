#!/bin/bash
cargo build --release --target wasm32-unknown-unknown --package schnorr_canister
cargo test