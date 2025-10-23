use wasm_bindgen::prelude::*;
use reputation_common::types::{SignedReview, VendorReputation};
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

// Error type for WASM
#[derive(Debug)]
pub enum WasmError {
    ParseError(String),
    VerificationError(String),
}

impl std::fmt::Display for WasmError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WasmError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            WasmError::VerificationError(msg) => write!(f, "Verification error: {}", msg),
        }
    }
}

type Result<T> = std::result::Result<T, WasmError>;

/// Initialize WASM module with panic hook for better debugging
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}

/// Result of reputation file verification
#[wasm_bindgen]
#[derive(Debug, Clone, serde::Serialize)]
pub struct VerificationResult {
    /// Overall verification status
    is_valid: bool,

    /// Total number of reviews in file
    total_reviews: u32,

    /// Number of reviews with valid signatures
    valid_signatures: u32,

    /// Number of reviews with invalid signatures
    invalid_signatures: u32,

    /// Whether statistics match calculated values
    stats_match: bool,

    /// Error message if verification failed
    error_message: Option<String>,
}

#[wasm_bindgen]
impl VerificationResult {
    #[wasm_bindgen(getter)]
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    #[wasm_bindgen(getter)]
    pub fn total_reviews(&self) -> u32 {
        self.total_reviews
    }

    #[wasm_bindgen(getter)]
    pub fn valid_signatures(&self) -> u32 {
        self.valid_signatures
    }

    #[wasm_bindgen(getter)]
    pub fn invalid_signatures(&self) -> u32 {
        self.invalid_signatures
    }

    #[wasm_bindgen(getter)]
    pub fn stats_match(&self) -> bool {
        self.stats_match
    }

    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }

    /// Convert to JavaScript object for detailed inspection
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self).unwrap_or(JsValue::NULL)
    }
}

/// Verify a complete vendor reputation file
///
/// This is the main entry point for client-side verification.
/// It verifies:
/// 1. All signatures are valid
/// 2. Statistics match calculated values
/// 3. No tampering has occurred
///
/// # Arguments
/// * `reputation_json` - JSON string of VendorReputation file
///
/// # Returns
/// * `VerificationResult` - Detailed verification results
///
/// # Example (JavaScript)
/// ```javascript
/// import init, { verify_reputation_file } from './reputation_wasm.js';
///
/// await init();
///
/// const reputationJson = await fetch('/api/reputation/vendor_id').then(r => r.text());
/// const result = verify_reputation_file(reputationJson);
///
/// if (result.is_valid) {
///   console.log(`✅ All ${result.total_reviews} reviews verified!`);
/// } else {
///   console.error(`❌ Verification failed: ${result.error_message}`);
/// }
/// ```
#[wasm_bindgen]
pub fn verify_reputation_file(reputation_json: &str) -> VerificationResult {
    match verify_reputation_file_internal(reputation_json) {
        Ok(result) => result,
        Err(e) => VerificationResult {
            is_valid: false,
            total_reviews: 0,
            valid_signatures: 0,
            invalid_signatures: 0,
            stats_match: false,
            error_message: Some(format!("Verification error: {}", e)),
        },
    }
}

/// Internal verification logic with proper error handling
fn verify_reputation_file_internal(reputation_json: &str) -> Result<VerificationResult> {
    // Parse JSON
    let reputation: VendorReputation = serde_json::from_str(reputation_json)
        .map_err(|e| WasmError::ParseError(format!("Failed to parse reputation JSON: {}", e)))?;

    let total_reviews = reputation.reviews.len() as u32;
    let mut valid_signatures = 0u32;
    let mut invalid_signatures = 0u32;

    // Verify each review signature
    for review in &reputation.reviews {
        match verify_review_signature(review) {
            Ok(true) => valid_signatures += 1,
            Ok(false) => invalid_signatures += 1,
            Err(_) => invalid_signatures += 1,
        }
    }

    // Verify statistics match
    let calculated_stats = calculate_stats(&reputation.reviews);
    let stats_match = verify_stats_match(&reputation.stats, &calculated_stats);

    // Overall validation
    let is_valid = invalid_signatures == 0 && stats_match;

    Ok(VerificationResult {
        is_valid,
        total_reviews,
        valid_signatures,
        invalid_signatures,
        stats_match,
        error_message: if !is_valid {
            Some(format!(
                "{} invalid signature(s), stats_match={}",
                invalid_signatures, stats_match
            ))
        } else {
            None
        },
    })
}

/// Verify a single review signature
///
/// # Arguments
/// * `review_json` - JSON string of SignedReview
///
/// # Returns
/// * `true` if signature is valid, `false` otherwise
///
/// # Example (JavaScript)
/// ```javascript
/// const reviewJson = JSON.stringify({
///   txid: "abc123",
///   rating: 5,
///   comment: "Great!",
///   timestamp: "2025-10-22T12:00:00Z",
///   buyer_pubkey: "base64_pubkey",
///   signature: "base64_signature"
/// });
///
/// const isValid = verify_single_review(reviewJson);
/// ```
#[wasm_bindgen]
pub fn verify_single_review(review_json: &str) -> bool {
    match verify_single_review_internal(review_json) {
        Ok(valid) => valid,
        Err(e) => {
            web_sys::console::error_1(&format!("Review verification error: {}", e).into());
            false
        }
    }
}

fn verify_single_review_internal(review_json: &str) -> Result<bool> {
    let review: SignedReview = serde_json::from_str(review_json)
        .map_err(|e| WasmError::ParseError(format!("Failed to parse review JSON: {}", e)))?;

    verify_review_signature(&review)
}

/// Core signature verification logic (same as crypto crate)
fn verify_review_signature(review: &SignedReview) -> Result<bool> {
    // 1. Decode public key
    let pubkey_bytes = base64::engine::general_purpose::STANDARD
        .decode(&review.buyer_pubkey)
        .map_err(|e| WasmError::VerificationError(format!("Invalid base64 in buyer_pubkey: {}", e)))?;

    if pubkey_bytes.len() != 32 {
        return Err(WasmError::VerificationError("Invalid public key length: expected 32 bytes".to_string()));
    }

    let mut pubkey_array = [0u8; 32];
    pubkey_array.copy_from_slice(&pubkey_bytes);

    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .map_err(|_| WasmError::VerificationError("Invalid ed25519 public key".to_string()))?;

    // 2. Decode signature
    let sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(&review.signature)
        .map_err(|e| WasmError::VerificationError(format!("Invalid base64 in signature: {}", e)))?;

    if sig_bytes.len() != 64 {
        return Err(WasmError::VerificationError("Invalid signature length: expected 64 bytes".to_string()));
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);

    let signature = Signature::from_bytes(&sig_array);

    // 3. Reconstruct message
    let message = format!(
        "{}|{}|{}|{}",
        review.txid,
        review.rating,
        review.comment.as_deref().unwrap_or(""),
        review.timestamp.to_rfc3339()
    );

    // 4. Hash message
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let message_hash = hasher.finalize();

    // 5. Verify signature
    Ok(verifying_key.verify(&message_hash, &signature).is_ok())
}

/// Calculate statistics from reviews
fn calculate_stats(reviews: &[SignedReview]) -> reputation_common::types::ReputationStats {
    use chrono::Utc;

    if reviews.is_empty() {
        let now = Utc::now();
        return reputation_common::types::ReputationStats {
            total_reviews: 0,
            average_rating: 0.0,
            rating_distribution: [0; 5],
            oldest_review: now,
            newest_review: now,
        };
    }

    let mut rating_dist = [0u32; 5];
    let mut total_rating = 0u32;

    let mut oldest = reviews[0].timestamp;
    let mut newest = reviews[0].timestamp;

    for review in reviews {
        let idx = (review.rating - 1) as usize;
        rating_dist[idx] += 1;
        total_rating += review.rating as u32;

        if review.timestamp < oldest {
            oldest = review.timestamp;
        }
        if review.timestamp > newest {
            newest = review.timestamp;
        }
    }

    let avg = total_rating as f32 / reviews.len() as f32;

    reputation_common::types::ReputationStats {
        total_reviews: reviews.len() as u32,
        average_rating: avg,
        rating_distribution: rating_dist,
        oldest_review: oldest,
        newest_review: newest,
    }
}

/// Verify statistics match between provided and calculated
fn verify_stats_match(
    provided: &reputation_common::types::ReputationStats,
    calculated: &reputation_common::types::ReputationStats,
) -> bool {
    provided.total_reviews == calculated.total_reviews
        && (provided.average_rating - calculated.average_rating).abs() < 0.01
        && provided.rating_distribution == calculated.rating_distribution
}

/// Get version information
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_verify_empty_reputation() {
        let empty_reputation = r#"{
            "format_version": "1.0",
            "vendor_pubkey": "test_pubkey",
            "generated_at": "2025-10-22T12:00:00Z",
            "reviews": [],
            "stats": {
                "total_reviews": 0,
                "average_rating": 0.0,
                "rating_distribution": [0, 0, 0, 0, 0],
                "oldest_review": "2025-10-22T12:00:00Z",
                "newest_review": "2025-10-22T12:00:00Z"
            }
        }"#;

        let result = verify_reputation_file(empty_reputation);
        assert!(result.is_valid());
        assert_eq!(result.total_reviews(), 0);
    }
}
