use bip32::{Seed, XPrv};
use bitcoin::{
    key::{Secp256k1, UntweakedKeypair},
    secp256k1::Message,
};
use bitcoin_hashes::{sha256, Hash};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use getrandom::{register_custom_getrandom, Error};
use ic_crypto_extended_bip32::{DerivationIndex, DerivationPath};
use ic_stable_structures::{storable::Bound, StableBTreeMap, StableCell, Storable};
use serde::Serialize;
use serde_bytes::ByteBuf;
use std::{borrow::Cow, cell::RefCell, time::Duration};

mod ed25519;
mod memory;

use ed25519::derive_ed25519_private_key;
use memory::Memory;

const MAX_VALUE_SIZE: u32 = 100;

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct SchnorrPublicKeyArgs {
    pub canister_id: Option<Principal>,
    pub derivation_path: Vec<ByteBuf>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SchnorrPublicKeyResult {
    pub public_key: ByteBuf,
    pub chain_code: ByteBuf,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct SignWithSchnorrArgs {
    pub message: ByteBuf,
    pub derivation_path: Vec<ByteBuf>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SignWithSchnorrResult {
    pub signature: ByteBuf,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchnorrAlgorithm {
    Bip340Secp256k1,
    Ed25519,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchnorrKeyId {
    algorithm: SchnorrAlgorithm,
    name: String,
}

pub enum SchnorrKeyIds {
    DfxTestKey,
    TestKey1,
    DfxTestKeyEd25519,
    TestKey1Ed25519,
}

impl SchnorrKeyIds {
    pub fn to_key_id(&self) -> SchnorrKeyId {
        match self {
            Self::DfxTestKey => SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Bip340Secp256k1,
                name: "dfx_test_key".to_string(),
            },
            Self::TestKey1 => SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Bip340Secp256k1,
                name: "test_key_1".to_string(),
            },
            Self::DfxTestKeyEd25519 => SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: "dfx_test_key".to_string(),
            },
            Self::TestKey1Ed25519 => SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: "test_key_1".to_string(),
            },
        }
    }

    fn variants() -> Vec<SchnorrKeyIds> {
        vec![
            SchnorrKeyIds::DfxTestKey,
            SchnorrKeyIds::TestKey1,
            SchnorrKeyIds::DfxTestKeyEd25519,
            SchnorrKeyIds::TestKey1Ed25519,
        ]
    }
}

impl Storable for SchnorrKeyId {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: ByteBuf,
    pub certificate_version: Option<u16>,
}

type HeaderField = (String, String);

#[derive(Clone, Debug, CandidType, Deserialize)]
struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<HeaderField>,
    pub body: ByteBuf,
}

#[derive(Serialize, Deserialize)]
struct Metrics {
    pub balance: u128,
    pub sig_count: u128,
}

#[derive(Serialize, Deserialize)]
struct State {
    // The seeds for the keys are stored in a stable memory.
    #[serde(skip, default = "init_stable_data")]
    seeds: StableBTreeMap<SchnorrKeyId, [u8; 64], Memory>,

    #[serde(skip, default = "init_sig_count")]
    sig_count: StableCell<u128, Memory>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[ic_cdk::init]
fn init() {
    ic_cdk_timers::set_timer(Duration::ZERO, || {
        for key in SchnorrKeyIds::variants() {
            ic_cdk::spawn(async move {
                let seed = get_random_seed().await;
                STATE.with(|s| {
                    let seeds = &mut s.borrow_mut().seeds;
                    seeds
                        .get(&key.to_key_id())
                        .or_else(|| seeds.insert(key.to_key_id(), seed));
                });
            });
        }
    });
}
#[ic_cdk::update]
fn schnorr_public_key(arg: SchnorrPublicKeyArgs) -> SchnorrPublicKeyResult {
    let seed = Seed::new(STATE.with(|s| {
        s.borrow()
            .seeds
            .get(&arg.key_id)
            .unwrap_or_else(|| panic!("No key with name {:?}", &arg.key_id))
    }));

    let canister_id = match arg.canister_id {
        Some(canister_id) => canister_id,
        None => ic_cdk::caller(),
    };

    let indexes = to_derivation_indexes(&canister_id, &arg.derivation_path);
    match arg.key_id.algorithm {
        SchnorrAlgorithm::Bip340Secp256k1 => schnorr_public_key_secp256k1(seed, indexes),
        SchnorrAlgorithm::Ed25519 => schnorr_public_key_ed25519(seed, indexes),
    }
}

#[ic_cdk::update]
fn sign_with_schnorr(arg: SignWithSchnorrArgs) -> SignWithSchnorrResult {
    let seed = Seed::new(STATE.with(|s| {
        s.borrow()
            .seeds
            .get(&arg.key_id)
            .unwrap_or_else(|| panic!("No key with name {:?}", &arg.key_id))
    }));

    // Increment the signature count
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let current_count = *state.sig_count.get();
        let _ = state.sig_count.set(current_count + 1);
    });

    let canister_id = ic_cdk::caller();
    let indexes = to_derivation_indexes(&canister_id, &arg.derivation_path);

    match arg.key_id.algorithm {
        SchnorrAlgorithm::Bip340Secp256k1 => {
            sign_with_schnorr_secp256k1(seed, indexes, arg.message)
        }
        SchnorrAlgorithm::Ed25519 => sign_with_schnorr_ed25519(seed, indexes, arg.message),
    }
}

fn to_derivation_indexes(
    canister_id: &Principal,
    derivation_path: &Vec<ByteBuf>,
) -> Vec<DerivationIndex> {
    let mut path = vec![];
    let derivation_index = DerivationIndex(canister_id.as_slice().to_vec());
    path.push(derivation_index);

    for index in derivation_path {
        path.push(DerivationIndex(index.to_vec()));
    }
    path
}

fn schnorr_public_key_secp256k1(
    seed: Seed,
    indexes: Vec<DerivationIndex>,
) -> SchnorrPublicKeyResult {
    let secp256k1: Secp256k1<bitcoin::secp256k1::All> = Secp256k1::new();
    let root_xprv = XPrv::new(&seed).unwrap();
    let key_bytes = root_xprv.private_key().to_bytes();

    let key_pair = UntweakedKeypair::from_seckey_slice(&secp256k1, &key_bytes)
        .expect("Should generate key pair");

    let master_chain_code = [0u8; 32];
    let public_key_sec1 = key_pair.public_key().serialize();
    let res = DerivationPath::new(indexes)
        .public_key_derivation(&public_key_sec1, &master_chain_code)
        .expect("Should derive key");

    SchnorrPublicKeyResult {
        public_key: ByteBuf::from(res.derived_public_key),
        chain_code: ByteBuf::from(res.derived_chain_code),
    }
}

fn schnorr_public_key_ed25519(seed: Seed, indexes: Vec<DerivationIndex>) -> SchnorrPublicKeyResult {
    let (secret, chain_code) = derive_ed25519_private_key(seed.as_bytes(), indexes);
    let key = SigningKey::from_bytes(&secret);
    let public_key = VerifyingKey::from(&key);

    SchnorrPublicKeyResult {
        public_key: ByteBuf::from(public_key.to_bytes().to_vec()),
        chain_code: ByteBuf::from(chain_code.to_vec()),
    }
}

fn sign_with_schnorr_secp256k1(
    seed: Seed,
    indexes: Vec<DerivationIndex>,
    message: ByteBuf,
) -> SignWithSchnorrResult {
    let root_xprv = XPrv::new(&seed).unwrap();
    let private_key_bytes = root_xprv.private_key().to_bytes();

    let master_chain_code = [0u8; 32];
    let res = DerivationPath::new(indexes)
        .private_key_derivation(&private_key_bytes, &master_chain_code)
        .expect("Should derive key");

    let secp256k1: Secp256k1<bitcoin::secp256k1::All> = Secp256k1::new();
    let key_pair = UntweakedKeypair::from_seckey_slice(&secp256k1, &res.derived_private_key)
        .expect("Should generate key pair");

    let digest = sha256::Hash::hash(&message).to_byte_array();
    let sig = secp256k1.sign_schnorr_no_aux_rand(
        &Message::from_digest_slice(&digest).expect("should be cryptographically secure hash"),
        &key_pair,
    );

    SignWithSchnorrResult {
        signature: ByteBuf::from(sig.serialize().to_vec()),
    }
}

fn sign_with_schnorr_ed25519(
    seed: Seed,
    indexes: Vec<DerivationIndex>,
    message: ByteBuf,
) -> SignWithSchnorrResult {
    let (secret, _) = derive_ed25519_private_key(seed.as_bytes(), indexes);
    let key = SigningKey::from_bytes(&secret);
    SignWithSchnorrResult {
        signature: ByteBuf::from(key.sign(&message).to_vec()),
    }
}

#[ic_cdk::query]
fn http_request(_req: HttpRequest) -> HttpResponse {
    let sig_count = STATE.with(|s| *s.borrow().sig_count.get());
    let balance = ic_cdk::api::canister_balance128();
    let metrics = Metrics { balance, sig_count };

    HttpResponse {
        status_code: 200,
        headers: vec![("content-type".to_string(), "application/json".to_string())],
        body: ByteBuf::from(serde_json::to_string(&metrics).unwrap().as_bytes().to_vec()),
    }
}

fn init_sig_count() -> StableCell<u128, Memory> {
    StableCell::init(crate::memory::get_sig_count(), 0u128)
        .expect("Could not initialize sig count memory")
}

fn init_stable_data() -> StableBTreeMap<SchnorrKeyId, [u8; 64], Memory> {
    StableBTreeMap::init(crate::memory::get_seeds())
}

impl Default for State {
    fn default() -> Self {
        Self {
            sig_count: init_sig_count(),
            seeds: init_stable_data(),
        }
    }
}

async fn get_random_seed() -> [u8; 64] {
    match ic_cdk::api::management_canister::main::raw_rand().await {
        Ok(rand) => {
            let mut rand = rand.0;
            rand.extend(rand.clone());
            let rand: [u8; 64] = rand.try_into().expect("Expected a Vec of length 64");
            rand
        }
        Err(err) => {
            ic_cdk::trap(format!("Error getting random seed: {:?}", err).as_str());
        }
    }
}

pub fn my_custom_random(_buf: &mut [u8]) -> Result<(), Error> {
    ic_cdk::trap("Not implemented");
}

register_custom_getrandom!(my_custom_random);

ic_cdk::export_candid!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify_schnorr_secp256k1() {
        use bitcoin::{
            secp256k1::{schnorr::Signature, Secp256k1},
            PublicKey,
        };

        // Setup for signing
        let test_seed = [1u8; 64];
        // Example derivation path for signing
        let derivation_path = [vec![1u8; 4]]
            .iter()
            .map(|v| ByteBuf::from(v.clone()))
            .collect();
        let indexes = to_derivation_indexes(&Principal::anonymous(), &derivation_path);

        let message = b"Test message";
        let digest = sha256::Hash::hash(message).to_byte_array();

        // Call the sign function
        let sign_reply = sign_with_schnorr_secp256k1(
            Seed::new(test_seed),
            indexes.clone(),
            ByteBuf::from(message.to_vec()),
        );

        // Setup for verification
        let secp = Secp256k1::verification_only();
        let signature =
            Signature::from_slice(&sign_reply.signature).expect("Invalid signature format");

        let public_key_reply = schnorr_public_key_secp256k1(Seed::new(test_seed), indexes.clone());

        let raw_public_key = public_key_reply.public_key;

        let public_key = PublicKey::from_slice(&raw_public_key).unwrap().into();

        // Verify the signature
        assert!(secp
            .verify_schnorr(
                &signature,
                &Message::from_digest_slice(&digest).unwrap(),
                &public_key
            )
            .is_ok());
    }

    #[test]
    fn test_sign_and_verify_schnorr_ed25519() {
        use ed25519_dalek::{Signature, Verifier, VerifyingKey};

        // Setup for signing
        let test_seed = [1u8; 64];
        // Example derivation path for signing
        let derivation_path = [vec![1u8; 4]]
            .iter()
            .map(|v| ByteBuf::from(v.clone()))
            .collect();
        let indexes = to_derivation_indexes(&Principal::anonymous(), &derivation_path);

        let message = b"Test message";

        // Call the sign function
        let sign_reply = sign_with_schnorr_ed25519(
            Seed::new(test_seed),
            indexes.clone(),
            ByteBuf::from(message.to_vec()),
        );

        // Setup for verification
        let signature =
            Signature::from_slice(&sign_reply.signature).expect("Invalid signature format");

        let public_key_reply = schnorr_public_key_ed25519(Seed::new(test_seed), indexes.clone());

        let raw_public_key = public_key_reply.public_key.as_slice();
        assert_eq!(raw_public_key.len(), 32);
        let mut public_key = [0u8; 32];
        public_key.copy_from_slice(raw_public_key);

        let public_key = VerifyingKey::from_bytes(&public_key).unwrap();

        // Verify the signature
        assert!(public_key.verify(message, &signature).is_ok());
    }
}
