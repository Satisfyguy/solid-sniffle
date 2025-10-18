//! Encryption utilities for sensitive data

use anyhow::{Context, Result};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use rand::RngCore;

const KEY_SIZE: usize = 32; // 256 bits
const NONCE_SIZE: usize = 12; // 96 bits

/// Generates a random 256-bit AES-GCM key.
pub fn generate_key() -> Vec<u8> {
    let mut key = vec![0u8; KEY_SIZE];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

/// Encrypts plaintext data using AES-256-GCM.
pub fn encrypt_field(plaintext: &str, key: &[u8]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).context("Failed to create cipher from key")?;
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext for storage
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypts ciphertext data using AES-256-GCM.
pub fn decrypt_field(ciphertext_with_nonce: &[u8], key: &[u8]) -> Result<String> {
    if ciphertext_with_nonce.len() < NONCE_SIZE {
        return Err(anyhow::anyhow!("Ciphertext too short to contain nonce"));
    }

    let cipher = Aes256Gcm::new_from_slice(key).context("Failed to create cipher from key")?;
    let mut nonce_array = [0u8; NONCE_SIZE];
    nonce_array.copy_from_slice(&ciphertext_with_nonce[..NONCE_SIZE]);
    let nonce = Nonce::from(&nonce_array);
    let ciphertext = &ciphertext_with_nonce[NONCE_SIZE..];

    let plaintext_bytes = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext_bytes).context("Failed to convert decrypted bytes to UTF-8")
}