use bip32::{Seed, XPrv};
use candid::{CandidType, Deserialize, Principal};
use k256::ecdsa::signature::Signer;
use serde::Serialize;

use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableCell};
use std::cell::RefCell;

use getrandom::{Error, register_custom_getrandom};


type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(CandidType, Deserialize, Serialize, Debug)]
struct SchnorrPublicKey {
    pub canister_id: Option<Principal>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
struct SchnorrPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
struct SignWithSchnorr {
    pub message_hash: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

// enum SchnorrKeyIds {
//     #[allow(unused)]
//     TestKey1,
// }

// impl SchnorrKeyIds {
//     fn to_key_id(&self) -> SchnorrKeyId {
//         SchnorrKeyId {
//             name: match self {
//                 Self::TestKey1 => "test_key_1",
//             }
//             .to_string(),
//         }
//     }
// }

#[derive(CandidType, Deserialize, Debug)]
struct SignWithSchnorrReply {
    pub signature: Vec<u8>,
}

#[derive(CandidType, Deserialize, Serialize, Debug, Clone)]
struct SchnorrKeyId {
    pub name: String,
}

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static LOCK: RefCell<bool> = RefCell::new(false);

    // Initialize a `StableCell` with `MemoryId(0)`.
    static SEED: RefCell<StableCell<[u8; 64], Memory>> = RefCell::new(
            StableCell::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
                [0; 64]
        ).unwrap()
    );
}

#[ic_cdk::update]
async fn init_key() -> () {
    SEED.with(|s| {
        let seed = s.borrow().get().clone();
        let is_initialized = seed != [0; 64];

        if is_initialized {
            ic_cdk::trap("Already initialized");
        }
    });

    LOCK.with_borrow_mut(|l| {
        if *l {
            ic_cdk::trap("Already initializing");
        }
        *l = true;
    });

    let mut rand = match ic_cdk::api::management_canister::main::raw_rand().await {
        Ok(rand) => {
            LOCK.with_borrow_mut(|l| {
                *l = false;
            });
            rand.0
        }
        Err(err) => {
            LOCK.with_borrow_mut(|l| {
                *l = false;
            });
            ic_cdk::trap(&format!("Error: {:?}", err));
        }
    };

    rand.extend(rand.clone());
    let rand: [u8; 64] = rand.try_into().expect("Expected a Vec of length 64");

    let seed = Seed::new(rand);

    SEED.with(|s| {
        s.borrow_mut().set(seed.as_bytes().to_owned()).unwrap();
    });
}

#[ic_cdk::update]
fn schnorr_public_key(_arg: SchnorrPublicKey) -> SchnorrPublicKeyReply {
    let seed = SEED.with(|s| s.borrow().get().clone());
    let seed = Seed::new(seed);

    let root_xprv = XPrv::new(&seed).unwrap();
    let key_bytes = root_xprv.private_key().to_bytes();

    // let canisterId = if arg.canister_id.is_none() {
    //     ic_cdk::caller()
    // } else {
    //     arg.canister_id.unwrap()
    // };

    // let derivation_path: Vec<Vec<u8>> = arg.derivation_path;

   
    let signing_key = k256::schnorr::SigningKey::from_bytes(key_bytes.as_slice()).unwrap();
    let verifying_key  = signing_key.verifying_key();

    let chain_code = Vec::new();


    SchnorrPublicKeyReply {
        public_key: verifying_key.to_bytes().to_vec(),
        chain_code,
    }
}

#[ic_cdk::update]
fn sign_with_schnorr(arg: SignWithSchnorr) -> SignWithSchnorrReply {

    let message_hash = arg.message_hash;

    if message_hash.len() != 32 {
        ic_cdk::trap("Message hash must be 32 bytes");
    }

    let seed = SEED.with(|s| s.borrow().get().clone());
    let seed = Seed::new(seed);

    let root_xprv = XPrv::new(&seed).unwrap();
    let key_bytes = root_xprv.private_key().to_bytes();

    let signing_key = k256::schnorr::SigningKey::from_bytes(key_bytes.as_slice()).unwrap();
    
    // let signature = match signing_key.sign_prehash(message_hash.as_slice()) {
    //     Ok(signature) => signature,
    //     Err(_err) => ic_cdk::trap("Error signing message"),
    // };

    let signature = signing_key.sign(message_hash.as_slice());

    SignWithSchnorrReply {
        signature: signature.to_bytes().to_vec(),
    }
}

pub fn my_custom_random(_buf: &mut [u8]) -> Result<(), Error> {
    Ok(())
}

register_custom_getrandom!(my_custom_random);

ic_cdk::export_candid!();
