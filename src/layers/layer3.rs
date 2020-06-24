use super::super::ascii85;
use anyhow::Result;
use std::convert::TryInto;

fn decrypt(bytes: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    Ok(key
        .iter()
        .cycle()
        .zip(bytes)
        .map(|(key, byte)| byte ^ key)
        .collect())
}

fn key(bytes: &[u8]) -> Result<[u8; 32]> {
    let cipher_bytes: Vec<u8> = bytes[0..32].to_vec();

    // found this by first grabbing the last 32 bytes of the first line and hoping they were all '='
    // unfortunately, this approach is 2 characters shy, but it was enough to guess the correct header
    let known_string: &[u8; 32] = b"==[ Layer 4/5: Network Traffic ]";

    let key_bytes: Vec<u8> = cipher_bytes
        .into_iter()
        .zip(known_string)
        .map(|(cipher, &known)| cipher ^ known)
        .collect();

    let key: [u8; 32] = key_bytes.as_slice().try_into()?;

    Ok(key)
}

pub fn run(bytes: &[u8]) -> Result<Vec<u8>> {
    let decoded = ascii85::decode(bytes)?;
    let key = key(&decoded)?;
    decrypt(&decoded, &key)
}
