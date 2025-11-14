//! Cryptographic validation of multisig_info submissions (TM-003)
//!
//! This module prevents attacks where participants submit malicious or
//! backdoored multisig information by requiring proof-of-possession via
//! challenge-response signatures.
//!
//! # Security Model
//!
//! - Server generates random challenge (nonce + timestamp + escrow_id)
//! - Participant signs challenge with their multisig private key
//! - Server verifies signature matches submitted multisig_info public key
//! - Only valid signatures are accepted
//!
//! # Attack Prevention
//!
//! - ❌ Cannot submit someone else's multisig_info (no private key)
//! - ❌ Cannot submit backdoored keys without controlling them
//! - ❌ Cannot replay old signatures (nonce + timestamp binding)
//!
//! # Usage
//!
//! ```rust,no_run
//! use server::crypto::multisig_validation::{MultisigChallenge, verify_multisig_submission};
//!
//! // 1. Generate challenge for user
//! let challenge = MultisigChallenge::generate(escrow_id);
//! let challenge_message = challenge.message();
//!
//! // 2. User signs challenge offline (in their wallet)
//! // signature = monero_wallet.sign(challenge_message)
//!
//! // 3. Server verifies submission
//! verify_multisig_submission(&multisig_info, &signature, &challenge)?;
//! ```

use anyhow::{Context, Result};
use blake2::{Blake2b512, Digest};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Challenge expiry time (5 minutes)
const CHALLENGE_EXPIRY_SECS: u64 = 300;

/// Challenge nonce for proof-of-possession
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigChallenge {
    /// Random nonce (32 bytes)
    pub nonce: [u8; 32],

    /// Escrow ID this challenge is for
    pub escrow_id: Uuid,

    /// Unix timestamp when challenge was created
    pub created_at: u64,
}

impl MultisigChallenge {
    /// Generate new challenge for escrow
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let challenge = MultisigChallenge::generate(escrow_id);
    /// let message = challenge.message();
    /// // Send message to user for signing
    /// ```
    pub fn generate(escrow_id: Uuid) -> Self {
        use rand::RngCore;

        let mut nonce = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut nonce);

        Self {
            nonce,
            escrow_id,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Get challenge message to sign
    ///
    /// This message is what the user must sign with their multisig private key
    /// to prove they control the multisig_info they're submitting.
    ///
    /// # Format
    ///
    /// ```text
    /// BLAKE2b-512(
    ///     "MONERO_MARKETPLACE_MULTISIG_CHALLENGE" ||
    ///     nonce ||
    ///     escrow_id ||
    ///     timestamp
    /// )
    /// ```
    pub fn message(&self) -> Vec<u8> {
        let mut hasher = Blake2b512::new();

        // Domain separation
        hasher.update(b"MONERO_MARKETPLACE_MULTISIG_CHALLENGE");

        // Challenge components
        hasher.update(&self.nonce);
        hasher.update(self.escrow_id.as_bytes());
        hasher.update(&self.created_at.to_le_bytes());

        hasher.finalize().to_vec()
    }

    /// Check if challenge is still valid (not expired)
    ///
    /// Challenges expire after 5 minutes to prevent replay attacks
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.created_at < CHALLENGE_EXPIRY_SECS
    }

    /// Get remaining time before expiry (in seconds)
    pub fn time_remaining(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let elapsed = now - self.created_at;
        if elapsed >= CHALLENGE_EXPIRY_SECS {
            0
        } else {
            CHALLENGE_EXPIRY_SECS - elapsed
        }
    }
}

/// In-memory storage for active challenges
///
/// Maps (user_id, escrow_id) → Challenge
pub struct ChallengeStore {
    pub(crate) challenges: Arc<Mutex<HashMap<(Uuid, Uuid), MultisigChallenge>>>,
}

impl ChallengeStore {
    pub fn new() -> Self {
        Self {
            challenges: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store a new challenge for user/escrow
    pub fn store(&self, user_id: Uuid, escrow_id: Uuid, challenge: MultisigChallenge) {
        let mut challenges = self.challenges.lock().unwrap();
        challenges.insert((user_id, escrow_id), challenge);
    }

    /// Retrieve challenge for user/escrow
    pub fn get(&self, user_id: Uuid, escrow_id: Uuid) -> Option<MultisigChallenge> {
        let challenges = self.challenges.lock().unwrap();
        challenges.get(&(user_id, escrow_id)).cloned()
    }

    /// Remove challenge after use (one-time use)
    pub fn remove(&self, user_id: Uuid, escrow_id: Uuid) {
        let mut challenges = self.challenges.lock().unwrap();
        challenges.remove(&(user_id, escrow_id));
    }

    /// Clean up expired challenges (should be called periodically)
    pub fn cleanup_expired(&self) {
        let mut challenges = self.challenges.lock().unwrap();
        challenges.retain(|_, challenge| challenge.is_valid());
    }
}

impl Default for ChallengeStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify multisig_info submission with challenge-response signature
///
/// # Arguments
///
/// * `multisig_info` - The submitted multisig info string
/// * `signature` - Ed25519 signature over challenge message (64 bytes)
/// * `challenge` - Original challenge sent to participant
///
/// # Returns
///
/// Ok(()) if proof-of-possession is valid, Err otherwise
///
/// # Errors
///
/// Returns error if:
/// - Challenge expired (>5 minutes old)
/// - Cannot extract public key from multisig_info
/// - Signature verification fails
///
/// # Example
///
/// ```rust,no_run
/// let challenge = MultisigChallenge::generate(escrow_id);
/// // ... user signs challenge ...
/// verify_multisig_submission(&multisig_info, &signature, &challenge)?;
/// ```
pub fn verify_multisig_submission(
    multisig_info: &str,
    signature: &[u8],
    challenge: &MultisigChallenge,
) -> Result<()> {
    // 1. Check challenge hasn't expired
    if !challenge.is_valid() {
        anyhow::bail!(
            "Challenge expired ({} seconds old, max {})",
            challenge.created_at,
            CHALLENGE_EXPIRY_SECS
        );
    }

    // 2. Extract public key from multisig_info
    // NOTE: This is a SIMPLIFIED implementation for Monero's multisig format
    // In production, you need to properly parse Monero's actual multisig_info structure
    let public_key = extract_public_key_from_multisig_info(multisig_info)
        .context("Failed to extract public key from multisig_info")?;

    // 3. Parse signature
    if signature.len() != 64 {
        anyhow::bail!(
            "Invalid signature length: expected 64 bytes, got {}",
            signature.len()
        );
    }

    let sig_bytes: [u8; 64] = signature.try_into()
        .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;
    let sig = Signature::from_bytes(&sig_bytes);

    // 4. Verify signature
    let message = challenge.message();

    public_key
        .verify(&message, &sig)
        .map_err(|e| {
            anyhow::anyhow!(
                "Signature verification failed: {}. \
                 This means the submitter does not control the private key \
                 for the submitted multisig_info.",
                e
            )
        })?;

    tracing::info!(
        "✅ Multisig submission verified for escrow {}",
        challenge.escrow_id
    );

    Ok(())
}

/// Parsed representation of a Monero multisig_info (restricted to our supported formats)
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedMultisigInfo {
    version: String,
    pubkey_hex: String,
    m: Option<u8>,
    n: Option<u8>,
}

/// Parse Monero multisig_info into a structured form.
/// Supports two formats:
/// - Legacy minimal: "MultisigV1<hex_pubkey>" (64 hex chars)
/// - Structured: "MultisigV1:pk=<64hex>;m=<u8>;n=<u8>;chk=<hex>" (only pk mandatory)
fn parse_monero_multisig_info(multisig_info: &str) -> Result<ParsedMultisigInfo> {
    if !multisig_info.starts_with("MultisigV1") {
        anyhow::bail!(
            "Invalid multisig_info format: must start with 'MultisigV1', got: {}",
            &multisig_info[..20.min(multisig_info.len())]
        );
    }

    // Legacy minimal format: "MultisigV1<64hex>"
    if multisig_info.len() == "MultisigV1".len() + 64 {
        let pubkey_hex = multisig_info[10..].to_string();
        if !pubkey_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("multisig_info contains non-hex characters after prefix");
        }
        return Ok(ParsedMultisigInfo { version: "V1".into(), pubkey_hex, m: None, n: None });
    }

    // Structured variant: "MultisigV1:pk=<64hex>;m=<u8>;n=<u8>;chk=<hex>"
    if multisig_info.starts_with("MultisigV1:") {
        let rest = &multisig_info["MultisigV1:".len()..];
        let mut pk_hex: Option<String> = None;
        let mut m_opt: Option<u8> = None;
        let mut n_opt: Option<u8> = None;
        for seg in rest.split(';') {
            if seg.is_empty() { continue; }
            let mut it = seg.splitn(2, '=');
            let key = it.next().unwrap_or("").trim();
            let val = it.next().unwrap_or("").trim();
            match key {
                "pk" => pk_hex = Some(val.to_string()),
                "m" => {
                    let v = val.parse::<u8>().context("invalid m value")?; m_opt = Some(v);
                }
                "n" => {
                    let v = val.parse::<u8>().context("invalid n value")?; n_opt = Some(v);
                }
                "chk" | "checksum" => {
                    // Optional checksum; best-effort validation (hex)
                    if !val.chars().all(|c| c.is_ascii_hexdigit()) {
                        anyhow::bail!("checksum is not hex");
                    }
                }
                _ => anyhow::bail!("Unknown field '{}' in multisig_info", key),
            }
        }
        let pubkey_hex = pk_hex.ok_or_else(|| anyhow::anyhow!("multisig_info missing 'pk' field"))?;
        if pubkey_hex.len() != 64 || !pubkey_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("Invalid 'pk' field: expected 64 hex chars");
        }
        // Sanity: we only support 2-of-3
        if let (Some(m), Some(n)) = (m_opt, n_opt) {
            if !(m == 2 && n == 3) {
                anyhow::bail!("Unsupported multisig threshold: m={}, n={}", m, n);
            }
        }
        return Ok(ParsedMultisigInfo { version: "V1".into(), pubkey_hex, m: m_opt, n: n_opt });
    }

    anyhow::bail!("Unsupported multisig_info format. Expected 'MultisigV1<hex>' or 'MultisigV1:pk=...'");
}

/// Extract public key from Monero multisig_info string (using strict parser)
fn extract_public_key_from_multisig_info(multisig_info: &str) -> Result<VerifyingKey> {
    let parsed = parse_monero_multisig_info(multisig_info)?;
    let raw = hex::decode(&parsed.pubkey_hex).context("Failed to decode public key hex")?;
    if raw.len() != 32 { anyhow::bail!("Invalid public key length (expected 32 bytes)"); }
    let pk: [u8; 32] = raw.try_into().map_err(|_| anyhow::anyhow!("Invalid public key length"))?;
    let verifying_key = VerifyingKey::from_bytes(&pk).context("Invalid public key bytes")?;
    Ok(verifying_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn test_parse_multisig_info_legacy_ok() -> Result<()> {
        // Generate a random public key (simulate a 32-byte ed25519 pk)
        let sk = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let pk_hex = hex::encode(sk.verifying_key().as_bytes());
        let info = format!("MultisigV1{}", pk_hex);
        let parsed = parse_monero_multisig_info(&info)?;
        assert_eq!(parsed.version, "V1");
        assert_eq!(parsed.pubkey_hex, pk_hex);
        Ok(())
    }

    #[test]
    fn test_parse_multisig_info_structured_ok_2of3() -> Result<()> {
        let sk = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let pk_hex = hex::encode(sk.verifying_key().as_bytes());
        let info = format!("MultisigV1:pk={};m=2;n=3;chk=DEADBEEF", pk_hex);
        let parsed = parse_monero_multisig_info(&info)?;
        assert_eq!(parsed.version, "V1");
        assert_eq!(parsed.pubkey_hex, pk_hex);
        assert_eq!(parsed.m, Some(2));
        assert_eq!(parsed.n, Some(3));
        Ok(())
    }

    #[test]
    fn test_parse_multisig_info_structured_reject_unknown_field() {
        let info = "MultisigV1:pk=00112233445566778899AABBCCDDEEFF00112233445566778899AABBCCDDEEFF;x=1";
        let err = parse_monero_multisig_info(info).unwrap_err();
        assert!(err.to_string().contains("Unknown field"));
    }

    #[test]
    fn test_parse_multisig_info_wrong_threshold() {
        let info = "MultisigV1:pk=00112233445566778899AABBCCDDEEFF00112233445566778899AABBCCDDEEFF;m=1;n=2";
        let err = parse_monero_multisig_info(info).unwrap_err();
        assert!(err.to_string().contains("Unsupported multisig threshold"));
    }

    #[test]
    fn test_extract_public_key_from_multisig_info_ok() -> Result<()> {
        let sk = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let pk_hex = hex::encode(sk.verifying_key().as_bytes());
        let info = format!("MultisigV1{}", pk_hex);
        let vk = extract_public_key_from_multisig_info(&info)?;
        assert_eq!(vk.as_bytes(), sk.verifying_key().as_bytes());
        Ok(())
    }

    #[test]
    fn test_challenge_generation() {
        let escrow_id = Uuid::new_v4();
        let challenge = MultisigChallenge::generate(escrow_id);

        assert_eq!(challenge.escrow_id, escrow_id);
        assert!(challenge.is_valid());
        assert_eq!(challenge.nonce.len(), 32);
        assert!(challenge.time_remaining() > 0);
    }

    #[test]
    fn test_challenge_message() {
        let escrow_id = Uuid::new_v4();
        let challenge = MultisigChallenge::generate(escrow_id);

        let message1 = challenge.message();
        let message2 = challenge.message();

        // Message should be deterministic
        assert_eq!(message1, message2);
        assert_eq!(message1.len(), 64); // BLAKE2b-512 output
    }

    #[test]
    fn test_challenge_expiry() {
        let mut challenge = MultisigChallenge::generate(Uuid::new_v4());

        // Fresh challenge should be valid
        assert!(challenge.is_valid());

        // Backdate challenge by 6 minutes (past expiry)
        challenge.created_at -= 360;

        // Should now be expired
        assert!(!challenge.is_valid());
        assert_eq!(challenge.time_remaining(), 0);
    }

    #[test]
    fn test_challenge_store() {
        let store = ChallengeStore::new();
        let user_id = Uuid::new_v4();
        let escrow_id = Uuid::new_v4();
        let challenge = MultisigChallenge::generate(escrow_id);

        // Store challenge
        store.store(user_id, escrow_id, challenge.clone());

        // Retrieve challenge
        let retrieved = store.get(user_id, escrow_id).unwrap();
        assert_eq!(retrieved.escrow_id, challenge.escrow_id);

        // Remove challenge
        store.remove(user_id, escrow_id);
        assert!(store.get(user_id, escrow_id).is_none());
    }

    #[test]
    fn test_extract_public_key_valid() -> Result<()> {
        // Create a fake multisig_info with valid public key
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let fake_multisig_info = format!("MultisigV1{}", pubkey_hex);

        let extracted = extract_public_key_from_multisig_info(&fake_multisig_info)?;
        assert_eq!(extracted.as_bytes(), verifying_key.as_bytes());

        Ok(())
    }

    #[test]
    fn test_extract_public_key_invalid_prefix() {
        let result = extract_public_key_from_multisig_info("InvalidPrefix...");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MultisigV1"));
    }

    #[test]
    fn test_verify_multisig_submission_success() -> Result<()> {
        // Generate signing key
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();

        // Create fake multisig_info
        let pubkey_hex = hex::encode(verifying_key.as_bytes());
        let multisig_info = format!("MultisigV1{}", pubkey_hex);

        // Generate challenge
        let escrow_id = Uuid::new_v4();
        let challenge = MultisigChallenge::generate(escrow_id);

        // Sign challenge
        let message = challenge.message();
        let signature = signing_key.sign(&message);

        // Verify submission
        let result = verify_multisig_submission(
            &multisig_info,
            signature.to_bytes().as_ref(),
            &challenge,
        );

        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_verify_multisig_submission_wrong_signature() -> Result<()> {
        // Generate two different signing keys
        let signing_key1 = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let signing_key2 = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key1 = signing_key1.verifying_key();

        // Create multisig_info with signing_key1's public key
        let pubkey_hex = hex::encode(verifying_key1.as_bytes());
        let multisig_info = format!("MultisigV1{}", pubkey_hex);

        // Generate challenge
        let challenge = MultisigChallenge::generate(Uuid::new_v4());

        // Sign with signing_key2 (WRONG KEY)
        let message = challenge.message();
        let wrong_signature = signing_key2.sign(&message);

        // Verification should fail
        let result = verify_multisig_submission(
            &multisig_info,
            wrong_signature.to_bytes().as_ref(),
            &challenge,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Signature verification failed"));
        Ok(())
    }

    #[test]
    fn test_verify_multisig_submission_expired_challenge() -> Result<()> {
        // Generate signing key
        let signing_key = SigningKey::from_bytes(&rand::random::<[u8; 32]>());
        let verifying_key = signing_key.verifying_key();

        // Create multisig_info
        let pubkey_hex = hex::encode(verifying_key.as_bytes());
        let multisig_info = format!("MultisigV1{}", pubkey_hex);

        // Generate expired challenge
        let mut challenge = MultisigChallenge::generate(Uuid::new_v4());
        challenge.created_at -= 400; // Expire it

        // Sign challenge
        let message = challenge.message();
        let signature = signing_key.sign(&message);

        // Verification should fail due to expiry
        let result = verify_multisig_submission(
            &multisig_info,
            signature.to_bytes().as_ref(),
            &challenge,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expired"));
        Ok(())
    }
}
