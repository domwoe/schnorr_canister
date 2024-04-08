extern crate schnorr_canister;

use bitcoin::{
    key::Secp256k1,
    secp256k1::{schnorr::Signature, Message, PublicKey},
};

use bitcoin_hashes::{sha256, Hash};

use candid::{decode_one, encode_one, CandidType, Principal};
use pocket_ic::{PocketIc, WasmResult};
use schnorr_canister::{
    SchnorrKeyIds, SchnorrPublicKeyArgs, SchnorrPublicKeyResult, SignWithSchnorrArgs,
    SignWithSchnorrResult,
};
use serde::Deserialize;
use serde_bytes::ByteBuf;
use std::path::Path;

#[test]
fn test_sign_with_schnorr_secp256k1() {
    let pic = PocketIc::new();

    let my_principal = Principal::anonymous();
    // Create an empty canister as the anonymous principal and add cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    let wasm_bytes = load_schnorr_canister_wasm();
    pic.install_canister(canister_id, wasm_bytes, vec![], None);

    // Make sure the canister is properly initialized
    fast_forward(&pic, 5);

    let derivation_path: Vec<ByteBuf> = [vec![1u8; 4]] // Example derivation path for signing
        .iter()
        .map(|v| ByteBuf::from(v.clone()))
        .collect();

    let key_id = SchnorrKeyIds::TestKey1.to_key_id();
    let message = b"Test message";

    let digest = sha256::Hash::hash(message).to_byte_array();

    let payload: SignWithSchnorrArgs = SignWithSchnorrArgs {
        message: ByteBuf::from(message.to_vec()),
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };

    let res: Result<SignWithSchnorrResult, String> = update(
        &pic,
        my_principal,
        canister_id,
        "sign_with_schnorr",
        encode_one(payload).unwrap(),
    );

    let sig = res.unwrap().signature;

    let payload = SchnorrPublicKeyArgs {
        canister_id: None,
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };

    let res: Result<SchnorrPublicKeyResult, String> = update(
        &pic,
        my_principal,
        canister_id,
        "schnorr_public_key",
        encode_one(payload).unwrap(),
    );

    let pub_key_sec1 = res.unwrap().public_key;

    let pub_key = PublicKey::from_slice(&pub_key_sec1).unwrap().into();

    let sig = Signature::from_slice(&sig).unwrap();

    let msg = Message::from_digest_slice(&digest).unwrap();

    let secp = Secp256k1::verification_only();
    assert!(secp.verify_schnorr(&sig, &msg, &pub_key).is_ok());
}

#[test]
fn test_sign_with_schnorr_ed25519() {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let pic = PocketIc::new();

    let my_principal = Principal::anonymous();
    // Create an empty canister as the anonymous principal and add cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    let wasm_bytes = load_schnorr_canister_wasm();
    pic.install_canister(canister_id, wasm_bytes, vec![], None);

    // Make sure the canister is properly initialized
    fast_forward(&pic, 5);

    let derivation_path: Vec<ByteBuf> = [vec![1u8; 4]] // Example derivation path for signing
        .iter()
        .map(|v| ByteBuf::from(v.clone()))
        .collect();

    let key_id = SchnorrKeyIds::TestKey1Ed25519.to_key_id();
    let message = b"Test message";

    let payload: SignWithSchnorrArgs = SignWithSchnorrArgs {
        message: ByteBuf::from(message.to_vec()),
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };

    let res: Result<SignWithSchnorrResult, String> = update(
        &pic,
        my_principal,
        canister_id,
        "sign_with_schnorr",
        encode_one(payload).unwrap(),
    );

    let sig = res.unwrap().signature;

    let payload = SchnorrPublicKeyArgs {
        canister_id: None,
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };

    let res: Result<SchnorrPublicKeyResult, String> = update(
        &pic,
        my_principal,
        canister_id,
        "schnorr_public_key",
        encode_one(payload).unwrap(),
    );

    let res_ = res.unwrap();
    let pub_key_ = res_.public_key.as_slice();
    let mut public_key = [0u8; 32];
    public_key.copy_from_slice(pub_key_);
    let pub_key = VerifyingKey::from_bytes(&public_key).unwrap();

    let sig = Signature::from_slice(&sig).unwrap();

    assert!(pub_key.verify(message, &sig).is_ok());
}

fn load_schnorr_canister_wasm() -> Vec<u8> {
    let wasm_path = Path::new("./target/wasm32-unknown-unknown/release/schnorr_canister.wasm");

    std::fs::read(wasm_path).unwrap()
}

pub fn update<T: CandidType + for<'de> Deserialize<'de>>(
    ic: &PocketIc,
    sender: Principal,
    receiver: Principal,
    method: &str,
    args: Vec<u8>,
) -> Result<T, String> {
    match ic.update_call(receiver, sender, method, args) {
        Ok(WasmResult::Reply(data)) => Ok(decode_one(&data).unwrap()),
        Ok(WasmResult::Reject(error_message)) => Err(error_message.to_string()),
        Err(user_error) => Err(user_error.to_string()),
    }
}

pub fn fast_forward(ic: &PocketIc, ticks: u64) {
    for _ in 0..ticks - 1 {
        ic.tick();
    }
}
