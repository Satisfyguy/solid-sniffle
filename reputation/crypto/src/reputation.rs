use anyhow::{Context, Result};
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use reputation_common::types::{SignedReview, ReputationStats};
use chrono::Utc;

/// Génère une signature cryptographique pour un avis
///
/// # Arguments
/// * `txid` - Transaction hash Monero
/// * `rating` - Note 1-5
/// * `comment` - Commentaire optionnel
/// * `buyer_signing_key` - Clé de signature ed25519 de l'acheteur
///
/// # Returns
/// * `SignedReview` - Avis avec signature cryptographique
///
/// # Exemple
/// ```no_run
/// use ed25519_dalek::SigningKey;
/// use rand::{RngCore, rngs::OsRng};
/// use reputation_crypto::reputation::sign_review;
///
/// let mut csprng = OsRng;
/// let mut secret_bytes = [0u8; 32];
/// csprng.fill_bytes(&mut secret_bytes);
/// let signing_key = SigningKey::from_bytes(&secret_bytes);
/// let review = sign_review(
///     "abc123".to_string(),
///     5,
///     Some("Great!".to_string()),
///     &signing_key,
/// ).unwrap();
/// ```
pub fn sign_review(
    txid: String,
    rating: u8,
    comment: Option<String>,
    buyer_signing_key: &SigningKey,
) -> Result<SignedReview> {
    // Validate rating
    if !(1..=5).contains(&rating) {
        return Err(anyhow::anyhow!("Rating must be between 1 and 5"));
    }

    let timestamp = Utc::now();

    // 1. Construire le message à signer (format canonique)
    let message = format!(
        "{}|{}|{}|{}",
        txid,
        rating,
        comment.as_deref().unwrap_or(""),
        timestamp.to_rfc3339()
    );

    // 2. Hash du message (SHA-256)
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let message_hash = hasher.finalize();

    // 3. Signer avec clé privée acheteur
    let signature = buyer_signing_key.sign(&message_hash);

    // 4. Encoder en base64
    let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
    let verifying_key = buyer_signing_key.verifying_key();
    let buyer_pubkey_b64 = base64::engine::general_purpose::STANDARD.encode(verifying_key.to_bytes());

    Ok(SignedReview {
        txid,
        rating,
        comment,
        timestamp,
        buyer_pubkey: buyer_pubkey_b64,
        signature: signature_b64,
    })
}

/// Vérifie la signature cryptographique d'un avis
///
/// # Arguments
/// * `review` - Avis à vérifier
///
/// # Returns
/// * `bool` - true si signature valide, false sinon
///
/// # Exemple
/// ```ignore
/// // Example requires a SignedReview instance
/// use reputation_crypto::reputation::verify_review_signature;
///
/// let is_valid = verify_review_signature(&review).unwrap();
/// if is_valid {
///     println!("Signature valide!");
/// }
/// ```
pub fn verify_review_signature(review: &SignedReview) -> Result<bool> {
    // 1. Décoder la clé publique
    let pubkey_bytes = base64::engine::general_purpose::STANDARD
        .decode(&review.buyer_pubkey)
        .context("Invalid base64 in buyer_pubkey")?;

    if pubkey_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Invalid public key length: expected 32 bytes"));
    }

    let mut pubkey_array = [0u8; 32];
    pubkey_array.copy_from_slice(&pubkey_bytes);

    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .context("Invalid ed25519 public key")?;

    // 2. Décoder la signature
    let sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(&review.signature)
        .context("Invalid base64 in signature")?;

    if sig_bytes.len() != 64 {
        return Err(anyhow::anyhow!("Invalid signature length: expected 64 bytes"));
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);

    let signature = Signature::from_bytes(&sig_array);

    // 3. Reconstruire le message original
    let message = format!(
        "{}|{}|{}|{}",
        review.txid,
        review.rating,
        review.comment.as_deref().unwrap_or(""),
        review.timestamp.to_rfc3339()
    );

    // 4. Hash du message
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let message_hash = hasher.finalize();

    // 5. Vérifier la signature
    Ok(verifying_key.verify(&message_hash, &signature).is_ok())
}

/// Calcule les statistiques d'une liste d'avis
///
/// # Arguments
/// * `reviews` - Liste d'avis signés
///
/// # Returns
/// * `ReputationStats` - Statistiques calculées
pub fn calculate_stats(reviews: &[SignedReview]) -> ReputationStats {
    if reviews.is_empty() {
        let now = Utc::now();
        return ReputationStats {
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
        // Distribution
        let idx = (review.rating - 1) as usize;
        rating_dist[idx] += 1;
        total_rating += review.rating as u32;

        // Min/Max dates
        if review.timestamp < oldest {
            oldest = review.timestamp;
        }
        if review.timestamp > newest {
            newest = review.timestamp;
        }
    }

    let avg = total_rating as f32 / reviews.len() as f32;

    ReputationStats {
        total_reviews: reviews.len() as u32,
        average_rating: avg,
        rating_distribution: rating_dist,
        oldest_review: oldest,
        newest_review: newest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    #[test]
    fn test_sign_and_verify_review() {
        // Générer clé acheteur
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut csprng, &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);

        // Créer avis signé
        let review = sign_review(
            "abc123def456".to_string(),
            5,
            Some("Excellent product!".to_string()),
            &signing_key,
        )
        .unwrap();

        // Vérifier signature
        assert!(verify_review_signature(&review).unwrap());
    }

    #[test]
    fn test_tampered_review_fails_verification() {
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut csprng, &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);

        let review = sign_review(
            "abc123".to_string(),
            5,
            Some("Great!".to_string()),
            &signing_key,
        )
        .unwrap();

        // Modifier le rating (altération)
        let mut tampered = review.clone();
        tampered.rating = 1;

        // Vérification doit échouer
        assert!(!verify_review_signature(&tampered).unwrap());
    }

    #[test]
    fn test_invalid_rating_rejected() {
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut csprng, &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);

        let result = sign_review(
            "abc".to_string(),
            6,  // Invalid rating
            None,
            &signing_key,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_stats() {
        let mut csprng = OsRng;
        let mut secret_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut csprng, &mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);

        let review1 = sign_review("tx1".to_string(), 5, None, &signing_key).unwrap();
        let review2 = sign_review("tx2".to_string(), 4, None, &signing_key).unwrap();
        let review3 = sign_review("tx3".to_string(), 5, None, &signing_key).unwrap();

        let reviews = vec![review1, review2, review3];
        let stats = calculate_stats(&reviews);

        assert_eq!(stats.total_reviews, 3);
        assert!((stats.average_rating - 4.666667).abs() < 0.001);  // (5+4+5)/3
        assert_eq!(stats.rating_distribution[3], 1);  // 1x 4★
        assert_eq!(stats.rating_distribution[4], 2);  // 2x 5★
    }

    #[test]
    fn test_empty_reviews_stats() {
        let stats = calculate_stats(&[]);

        assert_eq!(stats.total_reviews, 0);
        assert_eq!(stats.average_rating, 0.0);
    }
}
