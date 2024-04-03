use ic_crypto_extended_bip32::DerivationIndex;
use std::convert::TryInto;
use std::io::Write;

// This is not a SLIP-0010 derivation path, but used for development purposes
pub fn derive_ed25519_private_key(seed: &[u8], path: Vec<DerivationIndex>) -> ([u8; 32], [u8; 32]) {
    let indexes: Vec<u32> = path
        .into_iter()
        .map(|mut v| {
            let bytes = [0u8; 4];
            let _ = v.0.write(&bytes);
            u32::from_be_bytes(bytes)
        })
        .collect();

    let mut x = hmac_sha512(b"ed25519 seed", seed);
    let mut data = [0u8; 37];

    for i in indexes {
        let hardened_index = 0x80000000 | i;
        let xl = &x[0..32];
        let xr = &x[32..64];

        data[1..33].copy_from_slice(xl);
        data[33..37].copy_from_slice(&hardened_index.to_be_bytes());

        x = hmac_sha512(xr, &data);
    }

    (x[0..32].try_into().unwrap(), x[32..].try_into().unwrap())
}

fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
    hmac_sha512::HMAC::mac(data, key)
}
