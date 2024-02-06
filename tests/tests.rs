extern crate schnorr_canister;

use bitcoin_hashes::{Hash, sha256};

use candid::{Decode, Encode, Principal};
use ic_cdk::api::management_canister::provisional::CanisterId;
use pocket_ic::{PocketIc, WasmResult};
use schnorr_canister::{SignWithSchnorr, SchnorrKeyIds};
use std::path::Path;


#[test]
fn test_sign_with_schnorr() {
    let pic = PocketIc::new();
    // Create an empty canister as the anonymous principal and add cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    let wasm_bytes = load_schnorr_canister_wasm();
    pic.install_canister(canister_id, wasm_bytes, vec![], None);

    pic.tick();
    pic.tick();
    
    let derivation_path = vec![vec![1u8; 4]]; // Example derivation path for signing
    let key_id = SchnorrKeyIds::TestKey1.to_key_id();
    let message = b"Test message";

    let digest = sha256::Hash::hash(message).to_byte_array();

    let payload: SignWithSchnorr = SignWithSchnorr {
        message: digest.to_vec(),
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };

    let reply = call_schnorr_canister(&pic, canister_id, "sign_with_schnorr", Encode!(&payload).unwrap());

    let reply = match reply {
        WasmResult::Reply(reply) => {
            reply
        }
        WasmResult::Reject(msg) => panic!("Call failed: {}", msg),
    };

    println!("Reply: {:?}", Decode!(&reply.as_slice()));
    
}

fn load_schnorr_canister_wasm() -> Vec<u8> {
    let wasm_path = Path::new("./target/wasm32-unknown-unknown/release/schnorr_canister.wasm.gz");

    std::fs::read(wasm_path).unwrap()
}

fn call_schnorr_canister(
    ic: &PocketIc,
    can_id: CanisterId,
    method: &str,
    payload: Vec<u8>,
) -> WasmResult {
    ic.update_call(can_id, Principal::anonymous(), method, payload)
        .expect("Failed to call schnorr canister")
}
