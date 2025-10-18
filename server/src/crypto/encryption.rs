use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

const NONCE_SIZE: usize = 12; // AES-GCM standard nonce size
const KEY_SIZE: usize = 32; // AES-256-GCM key size

/// Generates a random 256-bit key.
pub fn generate_key() -> Key<Aes256Gcm> {
    Aes256Gcm::generate_key(&mut OsRng)
}

/// Encrypts sensitive data before storing in DB.
/// The nonce is prepended to the ciphertext for storage.
pub fn encrypt_field(plaintext: &str, key: &Key<Aes256Gcm>) -> Result<String> {
    let cipher = Aes256Gcm::new(key.clone()); // Clone key for cipher
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext_with_tag = cipher.encrypt(nonce, plaintext.as_bytes())?;

    // Prepend nonce to ciphertext for storage
    let mut encrypted_data = nonce_bytes.to_vec();
    encrypted_data.extend_from_slice(&ciphertext_with_tag);

    Ok(general_purpose::STANDARD.encode(encrypted_data))
}

/// Decrypts data read from DB.
/// Assumes the nonce is prepended to the ciphertext.
pub fn decrypt_field(encrypted_base64: &str, key: &Key<Aes256Gcm>) -> Result<String> {
    let encrypted_data = general_purpose::STANDARD.decode(encrypted_base64)?;

    if encrypted_data.len() < NONCE_SIZE {
        anyhow::bail!("Encrypted data too short to contain nonce");
    }

    let (nonce_bytes, ciphertext_with_tag) = encrypted_data.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(key.clone()); // Clone key for cipher
    let plaintext_bytes = cipher.decrypt(nonce, ciphertext_with_tag)?;

    Ok(String::from_utf8(plaintext_bytes)?)
}
