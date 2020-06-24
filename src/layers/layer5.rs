use super::super::ascii85;
use anyhow::{anyhow, ensure, Result};
use openssl::aes::{unwrap_key, AesKey};
use openssl::symm::{decrypt, Cipher};
use std::convert::TryInto;

pub fn run(bytes: &[u8]) -> Result<Vec<u8>> {
    let bytes = ascii85::decode(bytes)?;

    ensure!(bytes.len() > 96 && bytes.len() % 8 == 0, "Invalid input");

    let key_encrypting_key_bytes: [u8; 32] = bytes[0..32].try_into()?;
    let key_encrypting_key = AesKey::new_decrypt(&key_encrypting_key_bytes)
        .map_err(|e| anyhow!("Key error: {:?}", e))?;
    let kek_iv: [u8; 8] = bytes[32..40].try_into()?;
    let mut decrypted_aes_key = [0u8; 32];
    let encrypted_aes_key = &bytes[40..80];

    unwrap_key(
        &key_encrypting_key,
        Some(kek_iv),
        &mut decrypted_aes_key,
        &encrypted_aes_key,
    )
    .map_err(|e| anyhow!("Key error: {:?}", e))?;

    let aes_iv: [u8; 16] = bytes[80..96].try_into()?;

    let encrypted_data = &bytes[96..];

    decrypt(
        Cipher::aes_256_cbc(),
        &decrypted_aes_key,
        Some(&aes_iv),
        encrypted_data,
    )
    .map_err(|e| anyhow!("Key error: {:?}", e))
}
