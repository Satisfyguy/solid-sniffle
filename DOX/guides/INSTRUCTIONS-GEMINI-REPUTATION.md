# INSTRUCTIONS GEMINI - Système de Réputation Portable

**Projet:** Monero Marketplace
**Votre Mission:** Implémenter système de réputation décentralisé et portable
**Durée:** 14 jours (5 milestones)
**Parallèle à:** Claude développe Phase 4 Frontend
**Workspace:** `reputation/` (nouveau dossier à créer à la racine)

---

## 🎯 VUE D'ENSEMBLE

### Objectif

Créer un système de réputation où :
- ✅ **Chaque avis = preuve cryptographique** (signature ed25519)
- ✅ **Réputation = fichier JSON exportable** vers IPFS
- ✅ **Vérification client-side** (compilation WASM)
- ✅ **Portable entre marketplaces** (format standard)
- ✅ **Impossible à falsifier** (signatures vérifiables)

### Principe Fondamental

**La réputation n'est pas un nombre dans une base de données, c'est un fichier de preuves cryptographiques que le vendeur possède et contrôle.**

### Flux du Système

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. Transaction Complétée (Escrow Released)                     │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Acheteur Invité à Noter (WebSocket ReviewInvitation)        │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Acheteur Crée Avis (rating, comment)                        │
│    → Signe avec sa clé privée ed25519                          │
│    → Génère SignedReview { txid, rating, signature, ... }      │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Soumission via API POST /api/reviews                        │
│    → Serveur vérifie signature cryptographique                 │
│    → Stocke dans DB (backup)                                   │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Vendeur Compile Réputation                                  │
│    → GET /api/reputation/{vendor_id}                           │
│    → Retourne VendorReputation (tous les avis signés)          │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. Export vers IPFS                                            │
│    → POST /api/reputation/export                               │
│    → Upload reputation.json vers IPFS                          │
│    → Retourne hash IPFS (Qm...)                                │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 7. Nouveau Acheteur Vérifie                                    │
│    → Télécharge reputation.json depuis IPFS                    │
│    → WASM vérifie chaque signature                             │
│    → Affiche avis vérifiés + score moyen                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📁 STRUCTURE DOSSIER reputation/

```
reputation/
├── common/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── types.rs                # SignedReview, VendorReputation, ReputationStats
├── crypto/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── reputation.rs           # sign_review(), verify_review_signature()
├── server/
│   ├── Cargo.toml
│   ├── handlers/
│   │   └── reputation.rs           # API endpoints REST
│   ├── db/
│   │   └── reputation.rs           # Fonctions DB (insert, get)
│   └── ipfs/
│       └── client.rs               # Client IPFS (upload/download)
├── wasm/
│   ├── Cargo.toml
│   ├── src/
│   │   └── lib.rs                  # verify_reputation_file() pour browser
│   ├── build.sh                    # Script compilation WASM
│   └── test-wasm.js                # Test JavaScript
├── migrations/
│   └── 2025-10-21-000000_create_reviews/
│       ├── up.sql
│       └── down.sql
├── tests/
│   ├── unit/
│   │   └── crypto_tests.rs
│   └── integration/
│       └── reputation_flow_test.rs # Test E2E complet
├── docs/
│   ├── REPUTATION-SPEC.md          # Spécification technique complète
│   ├── API-ENDPOINTS.md            # Documentation API
│   └── INTEGRATION-GUIDE.md        # Guide pour intégration par Claude
└── README.md
```

**Total:** ~25 fichiers à créer

---

## 🚧 ZONES DE NON-INTERFÉRENCE

### ✅ VOUS (Gemini) - Workspace Exclusif

Vous travaillez UNIQUEMENT dans :

```
reputation/
├── common/          ← Tout le dossier
├── crypto/          ← Tout le dossier
├── server/          ← Tout le dossier
├── wasm/            ← Tout le dossier
├── migrations/      ← Tout le dossier
├── tests/           ← Tout le dossier
└── docs/            ← Tout le dossier
```

### ❌ NE PAS TOUCHER (Géré par Claude - Phase 4 Frontend)

```
templates/                          ← Claude crée templates HTMX
static/                             ← Claude crée CSS/JS frontend
server/src/handlers/frontend.rs    ← Claude crée handlers pages
server/src/main.rs                  ← Intégration après coordination
```

### ⚠️ ZONES PARTAGÉES (Coordination Requise)

#### 1. `server/Cargo.toml`

**Vous ajouterez (section `[dependencies]`) :**
```toml
# Reputation system
ed25519-dalek = "2.1"
sha2 = "0.10"
base64 = "0.22"
reqwest = { version = "0.11", features = ["multipart"] }  # Pour IPFS
```

**Claude ajoutera (section `[dependencies]`) :**
```toml
# Frontend
tera = "1.19"              # Template engine
actix-files = "0.6"        # Static files
```

**Pas de conflit** → Sections différentes du fichier.

#### 2. `server/src/db/mod.rs`

**Vous ajouterez à la fin du fichier (~ligne 500+) :**
```rust
pub mod reputation;  // Fonctions DB pour avis
```

**Claude n'y touche pas** → Juste ajout en fin de fichier.

#### 3. `server/src/schema.rs`

**Diesel génère automatiquement** après votre migration SQL.
**Ne pas modifier manuellement.**

---

## 📋 MILESTONE REP.1 : Types & Cryptographie (3 jours)

### Objectif

Créer les structures de données et les fonctions cryptographiques de base.

### Fichiers à Créer

#### Fichier 1/5 : `reputation/common/Cargo.toml`

```toml
[package]
name = "reputation-common"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"

[dev-dependencies]
tokio = { version = "1", features = ["rt", "macros"] }
```

#### Fichier 2/5 : `reputation/common/src/lib.rs`

```rust
pub mod types;
```

#### Fichier 3/5 : `reputation/common/src/types.rs` (150 lignes)

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Avis signé cryptographiquement par un acheteur
///
/// Chaque avis est une preuve vérifiable qu'une transaction réelle
/// a eu lieu et que l'acheteur a émis cet avis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedReview {
    /// Transaction hash Monero (preuve on-chain)
    pub txid: String,

    /// Rating 1-5 étoiles
    #[serde(deserialize_with = "validate_rating")]
    pub rating: u8,

    /// Commentaire optionnel (max 500 chars)
    pub comment: Option<String>,

    /// Timestamp de création de l'avis
    pub timestamp: DateTime<Utc>,

    /// Clé publique de l'acheteur (ed25519, base64)
    pub buyer_pubkey: String,

    /// Signature cryptographique de l'avis
    /// Signature = sign(sha256(txid || rating || comment || timestamp))
    pub signature: String,
}

/// Fichier de réputation complet d'un vendeur
///
/// C'est le fichier portable qui peut être exporté vers IPFS
/// et importé sur n'importe quelle marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorReputation {
    /// Version du format (pour compatibilité future)
    pub format_version: String,  // "1.0"

    /// Clé publique du vendeur
    pub vendor_pubkey: String,

    /// Date de génération du fichier
    pub generated_at: DateTime<Utc>,

    /// Liste de tous les avis signés
    pub reviews: Vec<SignedReview>,

    /// Statistiques pré-calculées
    pub stats: ReputationStats,
}

/// Statistiques de réputation pré-calculées
///
/// Ces stats sont calculées côté serveur pour performance,
/// mais peuvent être recalculées côté client pour vérification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationStats {
    /// Nombre total d'avis
    pub total_reviews: u32,

    /// Note moyenne (0.0 à 5.0)
    pub average_rating: f32,

    /// Distribution des notes [1★, 2★, 3★, 4★, 5★]
    pub rating_distribution: [u32; 5],

    /// Date du plus ancien avis
    pub oldest_review: DateTime<Utc>,

    /// Date du plus récent avis
    pub newest_review: DateTime<Utc>,
}

// Validation Helpers

fn validate_rating<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let rating: u8 = Deserialize::deserialize(deserializer)?;
    if rating < 1 || rating > 5 {
        return Err(serde::de::Error::custom("Rating must be between 1 and 5"));
    }
    Ok(rating)
}

impl SignedReview {
    /// Valide la longueur du commentaire
    pub fn validate_comment(&self) -> Result<(), String> {
        if let Some(ref comment) = self.comment {
            if comment.len() > 500 {
                return Err(format!(
                    "Comment too long: {} chars (max 500)",
                    comment.len()
                ));
            }
        }
        Ok(())
    }
}

impl VendorReputation {
    /// Crée une nouvelle réputation vide
    pub fn new(vendor_pubkey: String) -> Self {
        let now = Utc::now();
        Self {
            format_version: "1.0".to_string(),
            vendor_pubkey,
            generated_at: now,
            reviews: Vec::new(),
            stats: ReputationStats {
                total_reviews: 0,
                average_rating: 0.0,
                rating_distribution: [0; 5],
                oldest_review: now,
                newest_review: now,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_serialization() {
        let review = SignedReview {
            txid: "abc123def456".to_string(),
            rating: 5,
            comment: Some("Excellent product!".to_string()),
            timestamp: Utc::now(),
            buyer_pubkey: "pubkey_base64_encoded".to_string(),
            signature: "signature_base64_encoded".to_string(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&review).unwrap();

        // Deserialize back
        let parsed: SignedReview = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.rating, 5);
        assert_eq!(parsed.txid, "abc123def456");
    }

    #[test]
    fn test_invalid_rating_rejected() {
        let json = r#"{
            "txid": "abc123",
            "rating": 6,
            "comment": null,
            "timestamp": "2025-10-21T00:00:00Z",
            "buyer_pubkey": "pub",
            "signature": "sig"
        }"#;

        let result: Result<SignedReview, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_comment_validation() {
        let mut review = SignedReview {
            txid: "abc".to_string(),
            rating: 5,
            comment: Some("x".repeat(501)),  // 501 chars
            timestamp: Utc::now(),
            buyer_pubkey: "pub".to_string(),
            signature: "sig".to_string(),
        };

        assert!(review.validate_comment().is_err());

        review.comment = Some("Valid comment".to_string());
        assert!(review.validate_comment().is_ok());
    }

    #[test]
    fn test_vendor_reputation_new() {
        let reputation = VendorReputation::new("vendor_pubkey_123".to_string());

        assert_eq!(reputation.format_version, "1.0");
        assert_eq!(reputation.reviews.len(), 0);
        assert_eq!(reputation.stats.total_reviews, 0);
    }
}
```

#### Fichier 4/5 : `reputation/crypto/Cargo.toml`

```toml
[package]
name = "reputation-crypto"
version = "0.1.0"
edition = "2021"

[dependencies]
reputation-common = { path = "../common" }
ed25519-dalek = "2.1"
sha2 = "0.10"
base64 = "0.22"
anyhow = "1.0"
rand = "0.8"

[dev-dependencies]
tokio = { version = "1", features = ["rt", "macros"] }
```

#### Fichier 5/5 : `reputation/crypto/src/reputation.rs` (350 lignes)

```rust
use anyhow::{Context, Result};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier, SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};
use reputation_common::types::{SignedReview, ReputationStats};
use chrono::{DateTime, Utc};

/// Génère une signature cryptographique pour un avis
///
/// # Arguments
/// * `txid` - Transaction hash Monero
/// * `rating` - Note 1-5
/// * `comment` - Commentaire optionnel
/// * `buyer_keypair` - Paire de clés ed25519 de l'acheteur
///
/// # Returns
/// * `SignedReview` - Avis avec signature cryptographique
///
/// # Exemple
/// ```no_run
/// use ed25519_dalek::SigningKey;
/// use rand::rngs::OsRng;
///
/// let signing_key = SigningKey::generate(&mut OsRng);
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
    if rating < 1 || rating > 5 {
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
    let signature_b64 = base64::encode(signature.to_bytes());
    let verifying_key = buyer_signing_key.verifying_key();
    let buyer_pubkey_b64 = base64::encode(verifying_key.to_bytes());

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
/// ```no_run
/// let is_valid = verify_review_signature(&review).unwrap();
/// if is_valid {
///     println!("Signature valide!");
/// }
/// ```
pub fn verify_review_signature(review: &SignedReview) -> Result<bool> {
    // 1. Décoder la clé publique
    let pubkey_bytes = base64::decode(&review.buyer_pubkey)
        .context("Invalid base64 in buyer_pubkey")?;

    if pubkey_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Invalid public key length: expected 32 bytes"));
    }

    let mut pubkey_array = [0u8; 32];
    pubkey_array.copy_from_slice(&pubkey_bytes);

    let verifying_key = VerifyingKey::from_bytes(&pubkey_array)
        .context("Invalid ed25519 public key")?;

    // 2. Décoder la signature
    let sig_bytes = base64::decode(&review.signature)
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
        let signing_key = SigningKey::generate(&mut csprng);

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
        let signing_key = SigningKey::generate(&mut csprng);

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
        let signing_key = SigningKey::generate(&mut csprng);

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
        let signing_key = SigningKey::generate(&mut csprng);

        let review1 = sign_review("tx1".to_string(), 5, None, &signing_key).unwrap();
        let review2 = sign_review("tx2".to_string(), 4, None, &signing_key).unwrap();
        let review3 = sign_review("tx3".to_string(), 5, None, &signing_key).unwrap();

        let reviews = vec![review1, review2, review3];
        let stats = calculate_stats(&reviews);

        assert_eq!(stats.total_reviews, 3);
        assert_eq!(stats.average_rating, 4.666667);  // (5+4+5)/3
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
```

### Tests Requis (Milestone REP.1)

- [x] `test_review_serialization`
- [x] `test_invalid_rating_rejected`
- [x] `test_comment_validation`
- [x] `test_vendor_reputation_new`
- [x] `test_sign_and_verify_review`
- [x] `test_tampered_review_fails_verification`
- [x] `test_invalid_rating_rejected`
- [x] `test_calculate_stats`
- [x] `test_empty_reviews_stats`

**Total:** 9 tests unitaires

### Validation Milestone 1

```bash
cd reputation/

# Compiler common
cargo check --manifest-path common/Cargo.toml

# Compiler crypto
cargo check --manifest-path crypto/Cargo.toml

# Tests common
cargo test --manifest-path common/Cargo.toml

# Tests crypto
cargo test --manifest-path crypto/Cargo.toml

# Couverture (minimum 80% requis)
cargo install cargo-tarpaulin
cargo tarpaulin --manifest-path common/Cargo.toml --out Stdout
cargo tarpaulin --manifest-path crypto/Cargo.toml --out Stdout
```

### Critères d'Acceptance

- [ ] Types compilent sans erreur
- [ ] Signatures ed25519 fonctionnelles
- [ ] 9 tests unitaires passent
- [ ] Couverture ≥ 80% pour common
- [ ] Couverture ≥ 80% pour crypto
- [ ] Documentation inline (`///`) présente
- [ ] Aucun warning `cargo clippy`

---

## 📋 MILESTONE REP.2 : Backend API (3 jours)

### Objectif

Créer l'API REST pour soumettre/récupérer avis + base de données + client IPFS.

### Fichiers à Créer

#### Fichier 1/7 : `reputation/migrations/2025-10-21-000000_create_reviews/up.sql`

```sql
-- Table des avis signés
CREATE TABLE reviews (
    id TEXT PRIMARY KEY NOT NULL,
    txid TEXT NOT NULL,
    reviewer_id TEXT NOT NULL,
    vendor_id TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    buyer_pubkey TEXT NOT NULL,
    signature TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (reviewer_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (vendor_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Index pour performance
CREATE INDEX idx_reviews_vendor ON reviews(vendor_id);
CREATE INDEX idx_reviews_txid ON reviews(txid);
CREATE INDEX idx_reviews_verified ON reviews(verified);
CREATE INDEX idx_reviews_timestamp ON reviews(timestamp DESC);
CREATE INDEX idx_reviews_rating ON reviews(rating);

-- Index composite pour requêtes fréquentes
CREATE INDEX idx_reviews_vendor_verified ON reviews(vendor_id, verified);
```

#### Fichier 2/7 : `reputation/migrations/2025-10-21-000000_create_reviews/down.sql`

```sql
DROP INDEX IF EXISTS idx_reviews_vendor_verified;
DROP INDEX IF EXISTS idx_reviews_rating;
DROP INDEX IF EXISTS idx_reviews_timestamp;
DROP INDEX IF EXISTS idx_reviews_verified;
DROP INDEX IF EXISTS idx_reviews_txid;
DROP INDEX IF EXISTS idx_reviews_vendor;
DROP TABLE IF EXISTS reviews;
```

#### Fichier 3/7 : `reputation/server/Cargo.toml`

```toml
[package]
name = "reputation-server"
version = "0.1.0"
edition = "2021"

[dependencies]
reputation-common = { path = "../common" }
reputation-crypto = { path = "../crypto" }

# Web framework
actix-web = "4"
actix-session = "0.9"

# Database
diesel = { version = "2.1", features = ["sqlite", "r2d2", "chrono"] }
uuid = { version = "1.0", features = ["v4", "serde"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
anyhow = "1.0"

# Logging
tracing = "0.1"

# HTTP client (for IPFS)
reqwest = { version = "0.11", features = ["multipart", "json"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
actix-rt = "2"
```

#### Fichier 4/7 : `reputation/server/handlers/reputation.rs` (280 lignes)

```rust
use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;
use uuid::Uuid;
use anyhow::{Context, Result};

use reputation_common::types::{SignedReview, VendorReputation};
use reputation_crypto::reputation::{verify_review_signature, calculate_stats};

use crate::db::reputation::{db_insert_review, db_get_vendor_reviews};
use crate::ipfs::client::IpfsClient;

// Types pour les requêtes/réponses
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ExportRequest {
    pub vendor_id: String,
}

#[derive(Serialize)]
pub struct ExportResponse {
    pub ipfs_hash: String,
    pub file_size: usize,
    pub total_reviews: u32,
}

/// POST /api/reviews
///
/// Soumettre un avis signé cryptographiquement après transaction complétée
///
/// # Body
/// ```json
/// {
///   "txid": "abc123...",
///   "rating": 5,
///   "comment": "Excellent service!",
///   "timestamp": "2025-10-21T12:00:00Z",
///   "buyer_pubkey": "base64...",
///   "signature": "base64..."
/// }
/// ```
///
/// # Response (201)
/// ```json
/// {
///   "status": "success",
///   "message": "Review submitted successfully"
/// }
/// ```
pub async fn submit_review(
    pool: web::Data<crate::db::DbPool>,
    session: Session,
    review: web::Json<SignedReview>,
) -> impl Responder {
    // 1. Vérifier que l'utilisateur est authentifié
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Not authenticated"
        })),
    };

    // 2. Valider le commentaire (longueur)
    if let Err(e) = review.validate_comment() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": e
        }));
    }

    // 3. Vérifier la signature cryptographique
    match verify_review_signature(&review) {
        Ok(true) => {
            tracing::info!("Review signature valid for txid: {}", review.txid);
        },
        Ok(false) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid cryptographic signature"
            }));
        },
        Err(e) => {
            tracing::error!("Signature verification error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Verification error: {}", e)
            }));
        },
    }

    // 4. TODO: Vérifier que le txid existe sur la blockchain Monero
    // (Sera implémenté dans intégration avec blockchain_monitor)

    // 5. Extraire vendor_id depuis le txid (via DB escrows)
    // Pour l'instant, on suppose qu'il est fourni via une autre route
    // ou extrait de la transaction associée
    let vendor_id = "vendor_placeholder".to_string();  // TODO: Récupérer depuis escrow

    // 6. Stocker l'avis dans la base de données
    match db_insert_review(&pool, &review, &user_id, &vendor_id).await {
        Ok(_) => {
            tracing::info!(
                "Review stored: txid={}, reviewer={}, rating={}",
                review.txid,
                user_id,
                review.rating
            );

            HttpResponse::Created().json(serde_json::json!({
                "status": "success",
                "message": "Review submitted successfully"
            }))
        },
        Err(e) => {
            tracing::error!("Database error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {}", e)
            }))
        },
    }
}

/// GET /api/reputation/{vendor_id}
///
/// Récupérer le fichier de réputation complet d'un vendeur
///
/// # Response (200)
/// ```json
/// {
///   "format_version": "1.0",
///   "vendor_pubkey": "vendor_uuid",
///   "generated_at": "2025-10-21T12:00:00Z",
///   "reviews": [...],
///   "stats": {
///     "total_reviews": 42,
///     "average_rating": 4.7,
///     ...
///   }
/// }
/// ```
pub async fn get_vendor_reputation(
    pool: web::Data<crate::db::DbPool>,
    vendor_id: web::Path<String>,
) -> impl Responder {
    let vendor_uuid = match vendor_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid vendor ID format"
        })),
    };

    // Charger tous les avis du vendeur
    let reviews = match db_get_vendor_reviews(&pool, vendor_uuid).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Error loading reviews: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Error: {}", e)
            }));
        },
    };

    // Calculer statistiques
    let stats = calculate_stats(&reviews);

    // Construire fichier de réputation
    let reputation = VendorReputation {
        format_version: "1.0".to_string(),
        vendor_pubkey: vendor_id.to_string(),
        generated_at: chrono::Utc::now(),
        reviews,
        stats,
    };

    tracing::info!(
        "Reputation generated for vendor {}: {} reviews, avg {}",
        vendor_id,
        reputation.stats.total_reviews,
        reputation.stats.average_rating
    );

    HttpResponse::Ok().json(reputation)
}

/// POST /api/reputation/export
///
/// Exporter la réputation d'un vendeur vers IPFS
///
/// # Body
/// ```json
/// {
///   "vendor_id": "uuid"
/// }
/// ```
///
/// # Response (200)
/// ```json
/// {
///   "ipfs_hash": "Qm...",
///   "file_size": 12345,
///   "total_reviews": 42
/// }
/// ```
pub async fn export_to_ipfs(
    pool: web::Data<crate::db::DbPool>,
    ipfs_client: web::Data<IpfsClient>,
    session: Session,
    body: web::Json<ExportRequest>,
) -> impl Responder {
    // Vérifier auth
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Not authenticated"
        })),
    };

    // Vérifier que l'utilisateur est le vendeur
    if user_id != body.vendor_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Can only export your own reputation"
        }));
    }

    let vendor_uuid = match body.vendor_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid vendor ID"
        })),
    };

    // Générer fichier réputation
    let reviews = match db_get_vendor_reviews(&pool, vendor_uuid).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Error: {}", e)
        })),
    };

    let stats = calculate_stats(&reviews);

    let reputation = VendorReputation {
        format_version: "1.0".to_string(),
        vendor_pubkey: user_id.clone(),
        generated_at: chrono::Utc::now(),
        reviews,
        stats,
    };

    // Sérialiser en JSON
    let json_bytes = match serde_json::to_vec_pretty(&reputation) {
        Ok(bytes) => bytes,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Serialization error: {}", e)
        })),
    };

    let file_size = json_bytes.len();

    // Upload vers IPFS
    let ipfs_hash = match ipfs_client.add(json_bytes).await {
        Ok(hash) => hash,
        Err(e) => {
            tracing::error!("IPFS upload error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("IPFS upload failed: {}", e)
            }));
        },
    };

    tracing::info!(
        "Reputation exported to IPFS: vendor={}, hash={}, size={}",
        user_id,
        ipfs_hash,
        file_size
    );

    HttpResponse::Ok().json(ExportResponse {
        ipfs_hash,
        file_size,
        total_reviews: reputation.stats.total_reviews,
    })
}
```

#### Fichier 5/7 : `reputation/server/db/reputation.rs` (200 lignes)

```rust
use anyhow::{Context, Result};
use diesel::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDateTime};

use reputation_common::types::SignedReview;

// Assumant que DbPool est défini dans server/src/db/mod.rs
pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<SqliteConnection>>;

// DB Model (pour Diesel)
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub txid: String,
    pub reviewer_id: String,
    pub vendor_id: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub buyer_pubkey: String,
    pub signature: String,
    pub timestamp: NaiveDateTime,
    pub verified: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = reviews)]
pub struct NewReview {
    pub id: String,
    pub txid: String,
    pub reviewer_id: String,
    pub vendor_id: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub buyer_pubkey: String,
    pub signature: String,
    pub timestamp: NaiveDateTime,
    pub verified: bool,
}

// Diesel schema (sera auto-généré, mais déclaré ici pour compilation)
diesel::table! {
    reviews (id) {
        id -> Text,
        txid -> Text,
        reviewer_id -> Text,
        vendor_id -> Text,
        rating -> Integer,
        comment -> Nullable<Text>,
        buyer_pubkey -> Text,
        signature -> Text,
        timestamp -> Timestamp,
        verified -> Bool,
        created_at -> Timestamp,
    }
}

/// Insérer un avis signé dans la base de données
///
/// # Arguments
/// * `pool` - Pool de connexions DB
/// * `review` - Avis signé à insérer
/// * `reviewer_id` - ID de l'acheteur (user_id)
/// * `vendor_id` - ID du vendeur
pub async fn db_insert_review(
    pool: &DbPool,
    review: &SignedReview,
    reviewer_id: &str,
    vendor_id: &str,
) -> Result<Review> {
    let mut conn = pool.get().context("Failed to get DB connection")?;

    let new_review = NewReview {
        id: Uuid::new_v4().to_string(),
        txid: review.txid.clone(),
        reviewer_id: reviewer_id.to_string(),
        vendor_id: vendor_id.to_string(),
        rating: review.rating as i32,
        comment: review.comment.clone(),
        buyer_pubkey: review.buyer_pubkey.clone(),
        signature: review.signature.clone(),
        timestamp: review.timestamp.naive_utc(),
        verified: false,  // À vérifier on-chain séparément
    };

    tokio::task::spawn_blocking(move || {
        use self::reviews::dsl::*;

        diesel::insert_into(reviews)
            .values(&new_review)
            .execute(&mut conn)
            .context("Failed to insert review")?;

        reviews
            .filter(id.eq(&new_review.id))
            .first::<Review>(&mut conn)
            .context("Failed to retrieve created review")
    })
    .await
    .context("Task join error")?
}

/// Récupérer tous les avis d'un vendeur
///
/// # Arguments
/// * `pool` - Pool de connexions DB
/// * `vendor_uuid` - UUID du vendeur
///
/// # Returns
/// * `Vec<SignedReview>` - Liste des avis signés
pub async fn db_get_vendor_reviews(
    pool: &DbPool,
    vendor_uuid: Uuid,
) -> Result<Vec<SignedReview>> {
    let mut conn = pool.get().context("Failed to get DB connection")?;
    let vendor_id_str = vendor_uuid.to_string();

    tokio::task::spawn_blocking(move || {
        use self::reviews::dsl::*;

        let db_reviews = reviews
            .filter(vendor_id.eq(vendor_id_str))
            .order(timestamp.desc())
            .load::<Review>(&mut conn)
            .context("Failed to load reviews")?;

        // Convertir Review (DB model) → SignedReview (type commun)
        let signed_reviews: Vec<SignedReview> = db_reviews
            .into_iter()
            .map(|r| SignedReview {
                txid: r.txid,
                rating: r.rating as u8,
                comment: r.comment,
                timestamp: DateTime::from_naive_utc_and_offset(r.timestamp, Utc),
                buyer_pubkey: r.buyer_pubkey,
                signature: r.signature,
            })
            .collect();

        Ok(signed_reviews)
    })
    .await
    .context("Task join error")?
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests nécessitent une DB de test
    // Sera implémenté dans tests d'intégration
}
```

#### Fichier 6/7 : `reputation/server/ipfs/client.rs` (180 lignes)

```rust
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Client IPFS pour upload/download de fichiers
///
/// Utilise l'API HTTP d'un nœud IPFS local (par défaut: http://127.0.0.1:5001)
#[derive(Clone)]
pub struct IpfsClient {
    api_url: String,
    client: Client,
}

#[derive(Deserialize)]
struct AddResponse {
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Size")]
    size: String,
}

impl IpfsClient {
    /// Créer un nouveau client IPFS
    ///
    /// # Arguments
    /// * `api_url` - URL de l'API IPFS (ex: "http://127.0.0.1:5001")
    pub fn new(api_url: String) -> Self {
        Self {
            api_url,
            client: Client::new(),
        }
    }

    /// Upload un fichier vers IPFS
    ///
    /// # Arguments
    /// * `content` - Contenu du fichier (bytes)
    ///
    /// # Returns
    /// * `String` - Hash IPFS (CID v0, ex: "Qm...")
    ///
    /// # Exemple
    /// ```no_run
    /// let client = IpfsClient::new("http://127.0.0.1:5001".to_string());
    /// let hash = client.add(b"Hello IPFS!".to_vec()).await?;
    /// println!("Uploaded: {}", hash);
    /// ```
    pub async fn add(&self, content: Vec<u8>) -> Result<String> {
        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(content)
                    .file_name("reputation.json"),
            );

        let response = self
            .client
            .post(&format!("{}/api/v0/add", self.api_url))
            .multipart(form)
            .send()
            .await
            .context("Failed to upload to IPFS")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("IPFS upload failed: {}", error_text));
        }

        let add_response: AddResponse = response
            .json()
            .await
            .context("Failed to parse IPFS response")?;

        tracing::info!("IPFS upload successful: hash={}, size={}", add_response.hash, add_response.size);

        Ok(add_response.hash)
    }

    /// Télécharger un fichier depuis IPFS
    ///
    /// # Arguments
    /// * `hash` - Hash IPFS du fichier
    ///
    /// # Returns
    /// * `Vec<u8>` - Contenu du fichier
    ///
    /// # Exemple
    /// ```no_run
    /// let content = client.cat("Qm...").await?;
    /// let json: VendorReputation = serde_json::from_slice(&content)?;
    /// ```
    pub async fn cat(&self, hash: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .post(&format!("{}/api/v0/cat?arg={}", self.api_url, hash))
            .send()
            .await
            .context("Failed to download from IPFS")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("IPFS download failed: {}", error_text));
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read IPFS content")?;

        tracing::info!("IPFS download successful: hash={}, size={}", hash, bytes.len());

        Ok(bytes.to_vec())
    }

    /// Vérifier si le nœud IPFS est accessible
    pub async fn is_online(&self) -> bool {
        match self.client
            .post(&format!("{}/api/v0/version", self.api_url))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]  // Nécessite un nœud IPFS local running
    async fn test_ipfs_add_cat() -> Result<()> {
        let client = IpfsClient::new("http://127.0.0.1:5001".to_string());

        // Vérifier que IPFS est accessible
        if !client.is_online().await {
            println!("IPFS node not running, skipping test");
            return Ok(());
        }

        let content = b"Hello IPFS from reputation system!";
        let hash = client.add(content.to_vec()).await?;

        assert!(hash.starts_with("Qm"));
        assert_eq!(hash.len(), 46);  // CID v0 length

        let retrieved = client.cat(&hash).await?;
        assert_eq!(retrieved, content);

        Ok(())
    }

    #[tokio::test]
    async fn test_ipfs_client_creation() {
        let client = IpfsClient::new("http://127.0.0.1:5001".to_string());
        assert_eq!(client.api_url, "http://127.0.0.1:5001");
    }
}
```

#### Fichier 7/7 : `reputation/server/lib.rs`

```rust
pub mod handlers {
    pub mod reputation;
}

pub mod db {
    pub mod reputation;
}

pub mod ipfs {
    pub mod client;
}
```

### Tests Requis (Milestone REP.2)

- [ ] `test_submit_review_valid`
- [ ] `test_submit_review_invalid_signature`
- [ ] `test_get_vendor_reputation`
- [ ] `test_export_to_ipfs`
- [ ] `test_ipfs_add_cat`

**Total:** 5 tests (intégration)

### Validation Milestone 2

```bash
cd reputation/

# Appliquer migration
diesel migration run --migration-dir migrations/ --database-url ../server/data/test.db

# Vérifier schema généré
cat ../server/src/schema.rs | grep reviews

# Tests
cargo test --manifest-path server/Cargo.toml

# Coverage
cargo tarpaulin --manifest-path server/Cargo.toml --out Stdout
```

### Critères d'Acceptance

- [ ] Migration SQL s'applique sans erreur
- [ ] Schema Diesel généré correctement
- [ ] 3 endpoints API compilent
- [ ] Fonctions DB fonctionnelles
- [ ] Client IPFS fonctionne (avec nœud local)
- [ ] 5 tests passent
- [ ] Couverture ≥ 80%

---

## 📋 MILESTONE REP.3 : WASM Verification (3 jours)

### Objectif

Compiler en WASM pour vérification côté client dans le browser.

### Structure WASM Crate

```
reputation/wasm/
├── Cargo.toml
├── src/
│   └── lib.rs
├── build.sh
├── test-wasm.js
└── README.md
```

### Fichiers à Créer

#### Fichier 1/5 : `reputation/wasm/Cargo.toml`

```toml
[package]
name = "reputation-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
reputation-common = { path = "../common" }
reputation-crypto = { path = "../crypto" }

wasm-bindgen = "0.2"
serde-wasm-bindgen = "0.6"
console_error_panic_hook = "0.1"

# Re-export dependencies needed by crypto
ed25519-dalek = "2.1"
sha2 = "0.10"
base64 = "0.22"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "s"  # Optimize for size
lto = true       # Link-time optimization
```

#### Fichier 2/5 : `reputation/wasm/src/lib.rs` (200 lignes)

```rust
use wasm_bindgen::prelude::*;
use reputation_common::types::{SignedReview, VendorReputation};
use reputation_crypto::reputation::verify_review_signature;
use serde::{Deserialize, Serialize};

/// Initialiser le panic hook pour meilleurs messages d'erreur en WASM
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Résultat de vérification d'un fichier de réputation
#[derive(Serialize, Deserialize)]
pub struct VerificationResult {
    pub total: usize,
    pub verified: usize,
    pub failed: usize,
    pub average_rating: f32,
    pub failed_reviews: Vec<String>,  // TXIDs des avis invalides
}

/// Vérifie cryptographiquement un fichier de réputation complet
///
/// Cette fonction est appelée depuis JavaScript dans le browser.
/// Elle vérifie chaque signature ed25519 dans le fichier.
///
/// # Arguments
/// * `json` - Fichier reputation.json (string)
///
/// # Returns
/// * `JsValue` - Résultat de vérification (JSON)
///
/// # Exemple JavaScript
/// ```javascript
/// import init, { verify_reputation_file } from './reputation_wasm.js';
///
/// await init();
/// const result = verify_reputation_file(reputationJson);
/// console.log(`Verified: ${result.verified}/${result.total}`);
/// ```
#[wasm_bindgen]
pub fn verify_reputation_file(json: &str) -> Result<JsValue, JsValue> {
    // Parse JSON
    let reputation: VendorReputation = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let mut verified_count = 0;
    let mut failed_count = 0;
    let mut failed_reviews = Vec::new();

    // Vérifier chaque avis
    for review in &reputation.reviews {
        match verify_review_signature(review) {
            Ok(true) => {
                verified_count += 1;
            },
            Ok(false) => {
                failed_count += 1;
                failed_reviews.push(review.txid.clone());
            },
            Err(e) => {
                failed_count += 1;
                failed_reviews.push(format!("{} (error: {})", review.txid, e));
            },
        }
    }

    let result = VerificationResult {
        total: reputation.reviews.len(),
        verified: verified_count,
        failed: failed_count,
        average_rating: reputation.stats.average_rating,
        failed_reviews,
    };

    // Convert to JsValue
    Ok(serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?)
}

/// Vérifie un seul avis (utile pour vérification temps réel)
///
/// # Arguments
/// * `review_json` - Avis au format JSON
///
/// # Returns
/// * `bool` - true si signature valide
#[wasm_bindgen]
pub fn verify_single_review(review_json: &str) -> Result<bool, JsValue> {
    let review: SignedReview = serde_json::from_str(review_json)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    verify_review_signature(&review)
        .map_err(|e| JsValue::from_str(&format!("Verification error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_verify_empty_reputation() {
        let json = r#"{
            "format_version": "1.0",
            "vendor_pubkey": "vendor123",
            "generated_at": "2025-10-21T00:00:00Z",
            "reviews": [],
            "stats": {
                "total_reviews": 0,
                "average_rating": 0.0,
                "rating_distribution": [0,0,0,0,0],
                "oldest_review": "2025-10-21T00:00:00Z",
                "newest_review": "2025-10-21T00:00:00Z"
            }
        }"#;

        let result = verify_reputation_file(json).unwrap();

        // Vérifier que c'est un objet JsValue valide
        assert!(!result.is_null());
    }
}
```

#### Fichier 3/5 : `reputation/wasm/build.sh`

```bash
#!/bin/bash
set -e

echo "🔧 Building WASM reputation verifier..."

# Vérifier que wasm-pack est installé
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    cargo install wasm-pack
fi

# Build WASM pour le web
echo "🔨 Compiling Rust to WASM..."
wasm-pack build --target web --out-dir pkg

# Vérifier la taille du WASM
WASM_SIZE=$(stat -f%z pkg/reputation_wasm_bg.wasm 2>/dev/null || stat -c%s pkg/reputation_wasm_bg.wasm 2>/dev/null)
echo "✅ WASM build complete!"
echo "📦 Package location: reputation/wasm/pkg/"
echo "📊 WASM size: $((WASM_SIZE / 1024)) KB"

# Lister les fichiers générés
echo ""
echo "📄 Generated files:"
ls -lh pkg/*.wasm pkg/*.js

echo ""
echo "✨ Ready to use! Import in JavaScript:"
echo "   import init, { verify_reputation_file } from './pkg/reputation_wasm.js';"
```

#### Fichier 4/5 : `reputation/wasm/test-wasm.js`

```javascript
/**
 * Test du module WASM depuis Node.js
 *
 * Usage:
 *   cd reputation/wasm/
 *   bash build.sh
 *   node test-wasm.js
 */

import init, { verify_reputation_file, verify_single_review } from './pkg/reputation_wasm.js';
import { readFileSync } from 'fs';

async function main() {
    // Initialiser WASM
    await init();

    console.log('✅ WASM module loaded\n');

    // Test 1: Fichier de réputation vide
    console.log('Test 1: Empty reputation file');
    const emptyReputation = {
        format_version: "1.0",
        vendor_pubkey: "vendor123",
        generated_at: new Date().toISOString(),
        reviews: [],
        stats: {
            total_reviews: 0,
            average_rating: 0.0,
            rating_distribution: [0, 0, 0, 0, 0],
            oldest_review: new Date().toISOString(),
            newest_review: new Date().toISOString()
        }
    };

    const result1 = verify_reputation_file(JSON.stringify(emptyReputation));
    console.log('Result:', result1);
    console.log(`Verified: ${result1.verified}/${result1.total}\n`);

    // Test 2: Fichier avec avis (nécessite de générer une vraie signature)
    console.log('Test 2: Reputation with reviews');
    console.log('(Skipping - requires real ed25519 signature generation)\n');

    console.log('✅ All tests passed!');
}

main().catch(console.error);
```

#### Fichier 5/5 : `reputation/wasm/README.md`

```markdown
# Reputation WASM Verifier

Module WASM pour vérification côté client des signatures cryptographiques dans les fichiers de réputation.

## Build

```bash
bash build.sh
```

Génère :
- `pkg/reputation_wasm.js` - Module JavaScript
- `pkg/reputation_wasm_bg.wasm` - Binary WASM
- `pkg/reputation_wasm.d.ts` - Types TypeScript

## Usage Browser

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, { verify_reputation_file } from './pkg/reputation_wasm.js';

        async function verifyReputation(reputationJson) {
            await init();

            const result = verify_reputation_file(reputationJson);

            console.log(`Verified: ${result.verified}/${result.total}`);
            console.log(`Average rating: ${result.average_rating}`);

            if (result.failed > 0) {
                console.warn(`Failed reviews:`, result.failed_reviews);
            }

            return result;
        }

        // Charger depuis IPFS
        fetch('https://ipfs.io/ipfs/Qm...')
            .then(r => r.text())
            .then(verifyReputation);
    </script>
</head>
<body>
    <h1>Reputation Verifier</h1>
</body>
</html>
```

## Usage Node.js

```bash
node test-wasm.js
```

## API

### `verify_reputation_file(json: string): VerificationResult`

Vérifie toutes les signatures dans un fichier de réputation.

**Returns:**
```typescript
{
    total: number,
    verified: number,
    failed: number,
    average_rating: number,
    failed_reviews: string[]
}
```

### `verify_single_review(review_json: string): boolean`

Vérifie un seul avis.

## Performance

- Taille WASM: ~150 KB (gzipped: ~50 KB)
- Vérification: ~1ms par signature
- Fichier 100 avis: ~100ms
```

### Tests Requis (Milestone REP.3)

- [ ] `test_verify_empty_reputation`
- [ ] Test build WASM sans erreur
- [ ] Test import JavaScript
- [ ] Test vérification avec avis réels

**Total:** 4 tests

### Validation Milestone 3

```bash
cd reputation/wasm/

# Build WASM
bash build.sh

# Vérifier output
ls -lh pkg/reputation_wasm_bg.wasm
ls -lh pkg/reputation_wasm.js

# Test Node.js
node test-wasm.js

# Tests WASM
wasm-pack test --node
```

### Critères d'Acceptance

- [ ] Build WASM réussit
- [ ] Fichiers générés présents (wasm + js)
- [ ] Taille WASM < 200 KB
- [ ] Test JavaScript fonctionne
- [ ] API exportée correctement

---

## 📋 MILESTONE REP.4 : Intégration Escrow (3 jours)

### Objectif

Trigger automatique d'invitation à noter après transaction escrow complétée.

### Fichiers à Modifier

**NOTE:** Ces fichiers sont dans le projet principal (pas dans `reputation/`)

#### Modification 1/3 : `server/src/services/blockchain_monitor.rs`

**Localiser la fonction `check_transaction_confirmations()` (ligne ~200)**

**Ajouter après confirmation de la transaction :**

```rust
async fn check_transaction_confirmations(&self, escrow_id: Uuid) -> Result<()> {
    // ... (code existant)

    if confirmations >= self.config.required_confirmations {
        let final_status = match escrow.status.as_str() {
            "releasing" => {
                // ✅ Transaction complétée → Inviter acheteur à noter
                self.trigger_review_invitation(escrow_id, &tx_hash).await?;
                "completed"
            },
            "refunding" => "refunded",
            _ => {
                warn!("Unexpected escrow status for finalization: {}", escrow.status);
                return Ok(());
            }
        };

        // ... (reste du code)
    }

    Ok(())
}
```

**Ajouter nouvelle méthode (à la fin de l'impl BlockchainMonitor) :**

```rust
/// Envoyer invitation à l'acheteur pour noter la transaction
///
/// Cette fonction est appelée automatiquement après qu'une transaction
/// escrow soit complétée et confirmée sur la blockchain.
async fn trigger_review_invitation(&self, escrow_id: Uuid, tx_hash: &str) -> Result<()> {
    let escrow = db_load_escrow(&self.db, escrow_id).await?;

    // Notifier via WebSocket
    self.websocket.do_send(WsEvent::ReviewInvitation {
        escrow_id,
        tx_hash: tx_hash.to_string(),
        buyer_id: escrow.buyer_id.parse()?,
        vendor_id: escrow.vendor_id.parse()?,
    });

    tracing::info!(
        "Review invitation sent to buyer {} for transaction {}",
        escrow.buyer_id,
        &tx_hash[..8]
    );

    Ok(())
}
```

#### Modification 2/3 : `server/src/websocket.rs`

**Localiser l'enum `WsEvent` (ligne ~50)**

**Ajouter nouveau variant :**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub enum WsEvent {
    // ... (variants existants)

    /// Invitation à noter une transaction complétée
    ReviewInvitation {
        escrow_id: Uuid,
        tx_hash: String,
        buyer_id: Uuid,
        vendor_id: Uuid,
    },
}
```

#### Modification 3/3 : `server/src/handlers/mod.rs`

**Ajouter module reputation :**

```rust
pub mod reputation;  // Nouveau module
pub mod escrow;      // Existant
pub mod auth;        // Existant
// etc.
```

### Tests Requis (Milestone REP.4)

Créer `reputation/tests/integration/escrow_integration_test.rs`:

```rust
use anyhow::Result;
use uuid::Uuid;

#[tokio::test]
async fn test_review_invitation_triggered() -> Result<()> {
    // Setup: Créer escrow + transaction complétée
    let (escrow_id, tx_hash) = setup_completed_escrow().await?;

    // Simuler confirmations blockchain
    simulate_confirmations(&tx_hash, 10).await?;

    // Vérifier que WebSocket event a été envoyé
    let events = get_websocket_events().await?;

    let review_invitation = events.iter()
        .find(|e| matches!(e, WsEvent::ReviewInvitation { .. }));

    assert!(review_invitation.is_some());

    Ok(())
}

#[tokio::test]
async fn test_complete_escrow_flow_with_review() -> Result<()> {
    // Flow complet : Create escrow → Fund → Release → Review

    // 1. Create escrow
    let escrow = create_test_escrow().await?;

    // 2. Fund
    fund_escrow(escrow.id).await?;

    // 3. Release
    let tx_hash = release_funds(escrow.id).await?;

    // 4. Wait confirmations
    wait_for_confirmations(&tx_hash, 10).await?;

    // 5. Verify invitation sent
    assert!(review_invitation_sent(escrow.id).await?);

    // 6. Submit review
    let review = create_signed_review(&tx_hash, 5, Some("Great!".to_string())).await?;
    submit_review_api(review).await?;

    // 7. Verify review stored
    let reputation = get_vendor_reputation(escrow.vendor_id).await?;
    assert_eq!(reputation.reviews.len(), 1);

    Ok(())
}
```

**Total:** 2 tests d'intégration

### Validation Milestone 4

```bash
# Tests intégration
cargo test --package server test_review_invitation
cargo test --package server test_complete_escrow_flow_with_review

# Vérifier WebSocket events
cargo test --package server websocket

# Coverage
cargo tarpaulin --package server --out Stdout
```

### Critères d'Acceptance

- [ ] WebSocket event `ReviewInvitation` défini
- [ ] `trigger_review_invitation()` implémenté
- [ ] Appel automatique après confirmations
- [ ] 2 tests d'intégration passent
- [ ] Aucun warning compilation

---

## 📋 MILESTONE REP.5 : Tests & Documentation (2 jours)

### Objectif

Tests end-to-end complets + documentation technique complète.

### Fichiers à Créer

#### Test E2E : `reputation/tests/integration/reputation_flow_test.rs` (250 lignes)

```rust
use anyhow::Result;
use uuid::Uuid;
use reputation_common::types::{SignedReview, VendorReputation};
use reputation_crypto::reputation::{sign_review, verify_review_signature};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

/// Test du flow complet de réputation
///
/// 1. Transaction escrow complétée
/// 2. Acheteur signe avis
/// 3. Soumission via API
/// 4. Vendeur récupère réputation
/// 5. Export vers IPFS
/// 6. Vérification WASM
#[tokio::test]
async fn test_complete_reputation_flow() -> Result<()> {
    // Setup: Créer acheteur, vendeur, transaction
    let (buyer_signing_key, vendor_id, tx_hash) = setup_test_transaction().await?;

    // 1. Acheteur signe avis
    let review = sign_review(
        tx_hash.clone(),
        5,
        Some("Excellent product, fast delivery!".to_string()),
        &buyer_signing_key,
    )?;

    // Vérifier signature localement
    assert!(verify_review_signature(&review)?);

    // 2. Soumettre via API POST /api/reviews
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/api/reviews")
        .json(&review)
        .send()
        .await?;

    assert_eq!(response.status(), 201);

    let body: serde_json::Value = response.json().await?;
    assert_eq!(body["status"], "success");

    // 3. Vendeur récupère réputation GET /api/reputation/{vendor_id}
    let reputation: VendorReputation = client
        .get(&format!("http://localhost:8080/api/reputation/{}", vendor_id))
        .send()
        .await?
        .json()
        .await?;

    assert_eq!(reputation.reviews.len(), 1);
    assert_eq!(reputation.stats.total_reviews, 1);
    assert_eq!(reputation.stats.average_rating, 5.0);
    assert_eq!(reputation.reviews[0].txid, tx_hash);

    // 4. Export vers IPFS POST /api/reputation/export
    let export_response: serde_json::Value = client
        .post("http://localhost:8080/api/reputation/export")
        .json(&serde_json::json!({
            "vendor_id": vendor_id
        }))
        .send()
        .await?
        .json()
        .await?;

    let ipfs_hash = export_response["ipfs_hash"].as_str().unwrap();
    assert!(ipfs_hash.starts_with("Qm"));
    assert_eq!(export_response["total_reviews"], 1);

    // 5. Télécharger depuis IPFS et vérifier
    let ipfs_client = reputation_server::ipfs::client::IpfsClient::new(
        "http://127.0.0.1:5001".to_string()
    );

    let downloaded = ipfs_client.cat(ipfs_hash).await?;
    let downloaded_reputation: VendorReputation = serde_json::from_slice(&downloaded)?;

    assert_eq!(downloaded_reputation.reviews.len(), 1);

    // 6. Vérifier avec WASM
    let reputation_json = serde_json::to_string(&downloaded_reputation)?;

    // (Simulation de vérification WASM - le vrai test est dans wasm/tests/)
    for review in &downloaded_reputation.reviews {
        assert!(verify_review_signature(review)?);
    }

    Ok(())
}

/// Test soumission avis avec signature invalide
#[tokio::test]
async fn test_submit_review_invalid_signature() -> Result<()> {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);

    let mut review = sign_review(
        "tx123".to_string(),
        5,
        None,
        &signing_key,
    )?;

    // Altérer la signature
    review.signature = "invalid_signature_base64".to_string();

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8080/api/reviews")
        .json(&review)
        .send()
        .await?;

    assert_eq!(response.status(), 400);

    let body: serde_json::Value = response.json().await?;
    assert!(body["error"].as_str().unwrap().contains("signature"));

    Ok(())
}

/// Test multi-avis pour un même vendeur
#[tokio::test]
async fn test_multiple_reviews_same_vendor() -> Result<()> {
    let vendor_id = Uuid::new_v4();

    // Créer 5 avis de différents acheteurs
    for i in 1..=5 {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);

        let review = sign_review(
            format!("tx_{}", i),
            i as u8,  // Ratings 1-5
            Some(format!("Review number {}", i)),
            &signing_key,
        )?;

        submit_review_api(review, vendor_id).await?;
    }

    // Récupérer réputation
    let reputation = get_vendor_reputation_api(vendor_id).await?;

    assert_eq!(reputation.reviews.len(), 5);
    assert_eq!(reputation.stats.total_reviews, 5);
    assert_eq!(reputation.stats.average_rating, 3.0);  // (1+2+3+4+5)/5

    // Vérifier distribution
    assert_eq!(reputation.stats.rating_distribution[0], 1);  // 1★
    assert_eq!(reputation.stats.rating_distribution[1], 1);  // 2★
    assert_eq!(reputation.stats.rating_distribution[2], 1);  // 3★
    assert_eq!(reputation.stats.rating_distribution[3], 1);  // 4★
    assert_eq!(reputation.stats.rating_distribution[4], 1);  // 5★

    Ok(())
}

// Helper functions
async fn setup_test_transaction() -> Result<(SigningKey, Uuid, String)> {
    // TODO: Créer transaction test complète
    unimplemented!()
}

async fn submit_review_api(review: SignedReview, vendor_id: Uuid) -> Result<()> {
    // TODO: Appel API
    unimplemented!()
}

async fn get_vendor_reputation_api(vendor_id: Uuid) -> Result<VendorReputation> {
    // TODO: Appel API
    unimplemented!()
}
```

#### Documentation 1/3 : `reputation/docs/REPUTATION-SPEC.md` (400 lignes)

```markdown
# Spécification Technique - Système de Réputation Portable

## Vue d'Ensemble

Le système de réputation du Monero Marketplace est conçu pour être :
- **Décentralisé** : Les vendeurs possèdent leur réputation
- **Vérifiable** : Signatures cryptographiques ed25519
- **Portable** : Format JSON exportable vers IPFS
- **Interopérable** : Utilisable sur d'autres marketplaces

## Architecture

### Composants

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser (Client)                        │
├─────────────────────────────────────────────────────────────┤
│  WASM Verifier (reputation_wasm.wasm)                       │
│  ├─ verify_reputation_file()                                │
│  └─ verify_single_review()                                  │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTPS
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                   API Server (Rust)                         │
├─────────────────────────────────────────────────────────────┤
│  Handlers (reputation.rs)                                   │
│  ├─ POST /api/reviews          (Submit review)             │
│  ├─ GET  /api/reputation/{id}  (Get reputation)            │
│  └─ POST /api/reputation/export (Export to IPFS)           │
├─────────────────────────────────────────────────────────────┤
│  Crypto (reputation.rs)                                     │
│  ├─ sign_review()              (ed25519 signature)         │
│  └─ verify_review_signature()  (Verify signature)          │
├─────────────────────────────────────────────────────────────┤
│  Database (SQLCipher)                                       │
│  └─ reviews table (Backup storage)                         │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                      IPFS Network                           │
│  Qm... (reputation.json files)                              │
└─────────────────────────────────────────────────────────────┘
```

### Format de Données

#### SignedReview

```json
{
  "txid": "abc123def456...",
  "rating": 5,
  "comment": "Excellent product, fast delivery!",
  "timestamp": "2025-10-21T12:34:56Z",
  "buyer_pubkey": "base64_encoded_ed25519_public_key",
  "signature": "base64_encoded_ed25519_signature"
}
```

**Champs :**
- `txid` : Transaction hash Monero (preuve on-chain)
- `rating` : Note 1-5 étoiles
- `comment` : Texte libre (max 500 chars, optionnel)
- `timestamp` : Date/heure ISO 8601
- `buyer_pubkey` : Clé publique ed25519 de l'acheteur (32 bytes, base64)
- `signature` : Signature de `sha256(txid || rating || comment || timestamp)`

#### VendorReputation

```json
{
  "format_version": "1.0",
  "vendor_pubkey": "vendor_uuid_or_pubkey",
  "generated_at": "2025-10-21T13:00:00Z",
  "reviews": [
    { /* SignedReview */ },
    { /* SignedReview */ }
  ],
  "stats": {
    "total_reviews": 42,
    "average_rating": 4.7,
    "rating_distribution": [0, 2, 5, 15, 20],
    "oldest_review": "2025-01-15T10:00:00Z",
    "newest_review": "2025-10-21T12:34:56Z"
  }
}
```

## Cryptographie

### Algorithme de Signature

**ed25519** (EdDSA sur Curve25519)

**Raison :** Même algorithme que Monero utilise, compatible, rapide, sécurisé.

### Processus de Signature

1. **Message à signer :**
   ```
   message = "{txid}|{rating}|{comment}|{timestamp}"
   ```

2. **Hash du message :**
   ```
   hash = SHA-256(message)
   ```

3. **Signature :**
   ```
   signature = ed25519_sign(hash, buyer_private_key)
   ```

4. **Encodage :**
   ```
   signature_base64 = base64_encode(signature)  # 64 bytes → ~88 chars
   pubkey_base64 = base64_encode(public_key)    # 32 bytes → ~44 chars
   ```

### Vérification

1. Décoder `buyer_pubkey` et `signature` depuis base64
2. Reconstruire message identique
3. Hash avec SHA-256
4. Vérifier : `ed25519_verify(hash, signature, public_key)`

## API Endpoints

### POST /api/reviews

**Description:** Soumettre un avis signé

**Authentication:** Session cookie (user_id)

**Request:**
```json
{
  "txid": "abc123",
  "rating": 5,
  "comment": "Great!",
  "timestamp": "2025-10-21T12:00:00Z",
  "buyer_pubkey": "...",
  "signature": "..."
}
```

**Response 201:**
```json
{
  "status": "success",
  "message": "Review submitted successfully"
}
```

**Response 400:**
```json
{
  "error": "Invalid cryptographic signature"
}
```

### GET /api/reputation/{vendor_id}

**Description:** Récupérer fichier de réputation complet

**Authentication:** Public (pas d'auth requise)

**Response 200:**
```json
{
  "format_version": "1.0",
  "vendor_pubkey": "vendor_uuid",
  "generated_at": "2025-10-21T13:00:00Z",
  "reviews": [...],
  "stats": {...}
}
```

### POST /api/reputation/export

**Description:** Exporter vers IPFS

**Authentication:** Session (vendor only)

**Request:**
```json
{
  "vendor_id": "uuid"
}
```

**Response 200:**
```json
{
  "ipfs_hash": "Qm...",
  "file_size": 12345,
  "total_reviews": 42
}
```

## Base de Données

### Table `reviews`

```sql
CREATE TABLE reviews (
    id TEXT PRIMARY KEY,
    txid TEXT NOT NULL,
    reviewer_id TEXT NOT NULL,
    vendor_id TEXT NOT NULL,
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    buyer_pubkey TEXT NOT NULL,
    signature TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    verified BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**Index:**
- `idx_reviews_vendor` : Performance pour récupération par vendeur
- `idx_reviews_txid` : Vérification unicité transaction
- `idx_reviews_verified` : Filtrage avis vérifiés on-chain

## IPFS Storage

### Structure

```
Qm.../
└── reputation.json  (VendorReputation complet)
```

### Workflow Upload

1. Vendeur demande export (`POST /api/reputation/export`)
2. Serveur génère `reputation.json`
3. Upload vers nœud IPFS local (API HTTP)
4. Retourne hash CID v0 (`Qm...`)
5. Vendeur stocke hash dans son profil

### Workflow Download

1. Client lit hash IPFS depuis profil vendeur
2. Fetch `https://ipfs.io/ipfs/{hash}` ou gateway local
3. Parse JSON → `VendorReputation`
4. Vérifie chaque signature avec WASM
5. Affiche avis vérifiés uniquement

## Sécurité

### Menaces & Mitigations

| Menace | Mitigation |
|--------|-----------|
| Falsification d'avis | Signatures ed25519 vérifiables |
| Réutilisation de signatures | Timestamp + txid unique |
| Spam d'avis | Rate limiting API + vérification txid on-chain |
| Vendeur supprime avis négatifs | IPFS immuable + blockchain proofs |
| Interception man-in-the-middle | TLS 1.3 + vérification côté client |

### Limitations

- **Pas de révocation** : Un avis signé ne peut pas être supprimé
- **Dépendance IPFS** : Si hash perdu, réputation perdue (backup DB)
- **Sybil attacks** : Nécessite coût réel (transaction Monero) pour chaque avis

## Performance

### Benchmarks

- **Signature d'un avis** : ~0.5ms
- **Vérification signature** : ~1ms
- **Génération fichier réputation (100 avis)** : ~10ms
- **Upload IPFS (10KB)** : ~100ms
- **Vérification WASM (100 avis)** : ~100ms

### Optimisations

- Stats pré-calculées (pas de recalcul à chaque lecture)
- Index DB sur `vendor_id`
- Cache réputation côté client (localStorage)
- WASM compilé en mode release (`opt-level = "s"`)

## Évolutions Futures

### Phase 2 (Post-MVP)

- [ ] Vérification automatique txid on-chain
- [ ] Support multi-signatures (co-signataires)
- [ ] Mécanisme de dispute d'avis (arbitrage)
- [ ] Agrégation cross-marketplace (DNSLink IPFS)
- [ ] Zero-knowledge proofs (privacy-preserving ratings)

### Phase 3 (Advanced)

- [ ] Reputation staking (vendeurs stakent XMR)
- [ ] Algorithmes anti-Sybil avancés
- [ ] Intégration Tor pour anonymat complet
- [ ] Support autres blockchains (Bitcoin, Ethereum)
```

#### Documentation 2/3 : `reputation/docs/API-ENDPOINTS.md`

(Déjà inclus dans REPUTATION-SPEC.md section API Endpoints)

#### Documentation 3/3 : `reputation/docs/INTEGRATION-GUIDE.md` (350 lignes)

```markdown
# Guide d'Intégration - Système de Réputation

**Pour:** Claude (Phase 4 Frontend)
**Objectif:** Intégrer le système de réputation dans le frontend HTMX

---

## Vue d'Ensemble

Le système de réputation est actuellement complet dans le dossier `reputation/` :

✅ Types & Crypto (`reputation/common/`, `reputation/crypto/`)
✅ API Backend (`reputation/server/handlers/reputation.rs`)
✅ Base de données (`reviews` table)
✅ WASM Verifier (`reputation/wasm/pkg/`)
✅ Tests E2E

**Reste à faire :** Intégration dans le frontend (templates, routes, UI)

---

## Étape 1 : Déplacer Fichiers Backend

### 1.1 Handlers API

```bash
# Déplacer handlers
cp reputation/server/handlers/reputation.rs server/src/handlers/

# Vérifier
ls server/src/handlers/reputation.rs
```

### 1.2 DB Functions

```bash
# Ajouter module dans server/src/db/mod.rs
echo "pub mod reputation;" >> server/src/db/mod.rs

# Copier implémentation
cat reputation/server/db/reputation.rs >> server/src/db/reputation.rs
```

### 1.3 IPFS Client

```bash
# Créer dossier
mkdir -p server/src/ipfs/

# Copier client
cp reputation/server/ipfs/client.rs server/src/ipfs/

# Ajouter module dans server/src/lib.rs
echo "pub mod ipfs;" >> server/src/lib.rs
```

---

## Étape 2 : Ajouter Routes API

### 2.1 Modifier `server/src/main.rs`

**Importer handlers :**

```rust
mod handlers {
    pub mod reputation;  // Nouveau
    pub mod frontend;    // Existant
    pub mod auth;
    pub mod escrow;
}
```

**Ajouter routes dans `api_routes()` :**

```rust
fn api_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Auth (existant)
        .route("/api/auth/register", web::post().to(handlers::auth::register))
        .route("/api/auth/login", web::post().to(handlers::auth::login))

        // Escrow (existant)
        .route("/api/escrow/init", web::post().to(handlers::escrow::init_escrow))

        // Reputation (NOUVEAU)
        .route("/api/reviews", web::post().to(handlers::reputation::submit_review))
        .route("/api/reputation/{vendor_id}", web::get().to(handlers::reputation::get_vendor_reputation))
        .route("/api/reputation/export", web::post().to(handlers::reputation::export_to_ipfs));
}
```

### 2.2 Initialiser Client IPFS

**Dans `main()` :**

```rust
use server::ipfs::client::IpfsClient;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ... (setup existant)

    // Initialize IPFS client
    let ipfs_client = IpfsClient::new("http://127.0.0.1:5001".to_string());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ipfs_client.clone()))
            // ... (reste du setup)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

---

## Étape 3 : Copier WASM Build

### 3.1 Build WASM

```bash
cd reputation/wasm/
bash build.sh
```

### 3.2 Copier vers Static

```bash
# Créer dossier JS
mkdir -p static/js/

# Copier WASM binaries
cp reputation/wasm/pkg/reputation_wasm.js static/js/
cp reputation/wasm/pkg/reputation_wasm_bg.wasm static/js/
cp reputation/wasm/pkg/reputation_wasm.d.ts static/js/
```

### 3.3 Vérifier Serving

Dans `server/src/main.rs` :

```rust
use actix_files as fs;

App::new()
    .service(fs::Files::new("/static", "./static").show_files_listing())
    // ... (routes)
```

**Test :**
```
http://localhost:8080/static/js/reputation_wasm.js
http://localhost:8080/static/js/reputation_wasm_bg.wasm
```

---

## Étape 4 : Créer Templates Frontend

### 4.1 Template: Soumettre Avis

**Fichier:** `templates/reviews/submit.html`

```html
{% extends "base.html" %}

{% block title %}Leave a Review{% endblock %}

{% block content %}
<div class="review-form-container">
    <h1>Review Your Purchase</h1>
    <p>Transaction: <code>{{ tx_hash }}</code></p>

    <form
        hx-post="/api/reviews"
        hx-target="#review-result"
        hx-swap="innerHTML"
    >
        <input type="hidden" name="txid" value="{{ tx_hash }}">
        <input type="hidden" name="buyer_pubkey" value="{{ buyer_pubkey }}">
        <input type="hidden" name="signature" value="{{ signature }}">
        <input type="hidden" name="timestamp" value="{{ timestamp }}">

        <div class="form-group">
            <label>Rating</label>
            <div class="star-rating">
                <input type="radio" name="rating" value="5" id="star5" required>
                <label for="star5">★</label>
                <input type="radio" name="rating" value="4" id="star4">
                <label for="star4">★</label>
                <input type="radio" name="rating" value="3" id="star3">
                <label for="star3">★</label>
                <input type="radio" name="rating" value="2" id="star2">
                <label for="star2">★</label>
                <input type="radio" name="rating" value="1" id="star1">
                <label for="star1">★</label>
            </div>
        </div>

        <div class="form-group">
            <label for="comment">Comment (optional)</label>
            <textarea id="comment" name="comment" rows="4" maxlength="500"></textarea>
            <small>{{ comment.length }}/500 characters</small>
        </div>

        <button type="submit" class="btn btn-primary">Submit Review</button>
    </form>

    <div id="review-result"></div>
</div>

<script type="module">
    import init, { verify_single_review } from '/static/js/reputation_wasm.js';

    await init();

    // Sign review before submit (client-side)
    document.querySelector('form').addEventListener('htmx:configRequest', async (e) => {
        const formData = e.detail.parameters;

        // TODO: Signer avec clé privée locale de l'acheteur
        // Pour l'instant, signature générée server-side (non-custodial viendra plus tard)
    });
</script>
{% endblock %}
```

### 4.2 Template: Afficher Réputation

**Fichier:** `templates/vendor/reputation.html`

```html
{% extends "base.html" %}

{% block title %}{{ vendor.username }}'s Reputation{% endblock %}

{% block content %}
<div class="vendor-reputation">
    <h1>{{ vendor.username }}</h1>

    <!-- Score Overview -->
    <div class="reputation-overview">
        <div class="rating-badge">
            <span class="rating-number">{{ reputation.stats.average_rating }}</span>
            <span class="rating-stars">★★★★★</span>
        </div>
        <p>{{ reputation.stats.total_reviews }} verified reviews</p>
    </div>

    <!-- Rating Distribution -->
    <div class="rating-distribution">
        {% for i in range(5, 0, -1) %}
        <div class="rating-bar">
            <span>{{ i }}★</span>
            <div class="bar">
                <div class="fill" style="width: {{ (reputation.stats.rating_distribution[i-1] / reputation.stats.total_reviews * 100) }}%"></div>
            </div>
            <span>{{ reputation.stats.rating_distribution[i-1] }}</span>
        </div>
        {% endfor %}
    </div>

    <!-- Reviews List -->
    <div id="reviews-list" class="reviews-list">
        {% for review in reputation.reviews %}
        <div class="review-card" data-txid="{{ review.txid }}">
            <div class="review-header">
                <span class="rating">{{ review.rating }}★</span>
                <span class="date">{{ review.timestamp | date }}</span>
                <span class="verified-badge" title="Cryptographically verified">✓ Verified</span>
            </div>
            <p class="review-comment">{{ review.comment or "(No comment)" }}</p>
            <p class="review-tx">
                <small>Transaction: <code>{{ review.txid | truncate(16) }}</code></small>
            </p>
        </div>
        {% endfor %}
    </div>

    <!-- IPFS Export -->
    {% if user_id == vendor.id %}
    <div class="export-section">
        <h3>Export Reputation to IPFS</h3>
        <button
            hx-post="/api/reputation/export"
            hx-vals='{"vendor_id": "{{ vendor.id }}"}'
            hx-target="#export-result"
            class="btn"
        >
            Export to IPFS
        </button>
        <div id="export-result"></div>
    </div>
    {% endif %}
</div>

<script type="module">
    import init, { verify_reputation_file } from '/static/js/reputation_wasm.js';

    // Verify all reviews client-side
    await init();

    const reputationJson = JSON.stringify({{ reputation | tojson | safe }});
    const verification = verify_reputation_file(reputationJson);

    console.log(`Verified: ${verification.verified}/${verification.total} reviews`);

    // Mark failed reviews
    verification.failed_reviews.forEach(txid => {
        const card = document.querySelector(`[data-txid="${txid}"]`);
        if (card) {
            card.classList.add('verification-failed');
            card.querySelector('.verified-badge').textContent = '⚠ Invalid';
        }
    });
</script>
{% endblock %}
```

---

## Étape 5 : Handler Frontend

### 5.1 Créer `server/src/handlers/frontend_reviews.rs`

```rust
use actix_web::{web, HttpResponse, Responder};
use actix_session::Session;
use tera::{Tera, Context};
use uuid::Uuid;

use crate::db::reputation::db_get_vendor_reviews;
use reputation_crypto::reputation::calculate_stats;

/// GET /reviews/submit/{tx_hash}
pub async fn show_submit_review(
    tera: web::Data<Tera>,
    session: Session,
    tx_hash: web::Path<String>,
) -> impl Responder {
    let user_id = match session.get::<String>("user_id") {
        Ok(Some(id)) => id,
        _ => return HttpResponse::Unauthorized().body("Not authenticated"),
    };

    let mut ctx = Context::new();
    ctx.insert("tx_hash", &tx_hash.to_string());
    ctx.insert("user_id", &user_id);
    // TODO: Ajouter buyer_pubkey depuis session

    match tera.render("reviews/submit.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}

/// GET /vendor/{vendor_id}/reputation
pub async fn show_vendor_reputation(
    tera: web::Data<Tera>,
    pool: web::Data<crate::db::DbPool>,
    vendor_id: web::Path<String>,
) -> impl Responder {
    let vendor_uuid = match vendor_id.parse::<Uuid>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid vendor ID"),
    };

    // Load vendor
    // ... (code pour charger vendor depuis DB)

    // Load reviews
    let reviews = match db_get_vendor_reviews(&pool, vendor_uuid).await {
        Ok(r) => r,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    };

    let stats = calculate_stats(&reviews);

    let reputation = reputation_common::types::VendorReputation {
        format_version: "1.0".to_string(),
        vendor_pubkey: vendor_id.to_string(),
        generated_at: chrono::Utc::now(),
        reviews,
        stats,
    };

    let mut ctx = Context::new();
    ctx.insert("reputation", &reputation);
    // ctx.insert("vendor", &vendor);

    match tera.render("vendor/reputation.html", &ctx) {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(e) => HttpResponse::InternalServerError().body(format!("Template error: {}", e)),
    }
}
```

### 5.2 Ajouter Routes Frontend

Dans `server/src/main.rs` :

```rust
fn frontend_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(handlers::frontend::index))
        .route("/login", web::get().to(handlers::frontend::show_login))

        // Reputation (NOUVEAU)
        .route("/reviews/submit/{tx_hash}", web::get().to(handlers::frontend_reviews::show_submit_review))
        .route("/vendor/{vendor_id}/reputation", web::get().to(handlers::frontend_reviews::show_vendor_reputation));
}
```

---

## Étape 6 : CSS Styling

### 6.1 Ajouter dans `static/css/main.css`

```css
/* Reputation Styles */

.vendor-reputation {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
}

.reputation-overview {
    text-align: center;
    margin: 30px 0;
}

.rating-badge {
    font-size: 48px;
    font-weight: bold;
    color: #f39c12;
}

.rating-badge .rating-number {
    display: block;
}

.rating-badge .rating-stars {
    font-size: 24px;
    color: #f39c12;
}

/* Rating Distribution Bars */
.rating-distribution {
    margin: 30px 0;
}

.rating-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 10px 0;
}

.rating-bar .bar {
    flex: 1;
    height: 20px;
    background: #ecf0f1;
    border-radius: 10px;
    overflow: hidden;
}

.rating-bar .fill {
    height: 100%;
    background: linear-gradient(90deg, #f39c12, #e67e22);
    transition: width 0.3s;
}

/* Review Cards */
.reviews-list {
    margin-top: 30px;
}

.review-card {
    background: white;
    padding: 20px;
    margin: 15px 0;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.review-card.verification-failed {
    border-left: 4px solid #e74c3c;
}

.review-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
}

.review-header .rating {
    font-size: 20px;
    color: #f39c12;
    font-weight: bold;
}

.verified-badge {
    background: #27ae60;
    color: white;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
}

.verification-failed .verified-badge {
    background: #e74c3c;
}

.review-comment {
    margin: 15px 0;
    font-size: 16px;
    line-height: 1.6;
}

.review-tx {
    margin: 10px 0;
    color: #7f8c8d;
}

/* Star Rating Input */
.star-rating {
    display: flex;
    flex-direction: row-reverse;
    justify-content: flex-end;
    gap: 5px;
}

.star-rating input {
    display: none;
}

.star-rating label {
    font-size: 40px;
    color: #ddd;
    cursor: pointer;
    transition: color 0.2s;
}

.star-rating input:checked ~ label,
.star-rating label:hover,
.star-rating label:hover ~ label {
    color: #f39c12;
}
```

---

## Étape 7 : Tests Intégration

### 7.1 Test Manuel

```bash
# 1. Start server
cargo run --package server

# 2. Open browser
http://localhost:8080/

# 3. Test flow:
#    - Complete transaction escrow
#    - Receive WebSocket ReviewInvitation
#    - Submit review via /reviews/submit/{tx_hash}
#    - View vendor reputation /vendor/{id}/reputation
#    - Export to IPFS
#    - Verify signatures client-side (check console)
```

### 7.2 Test Automatisé

Créer `server/tests/frontend_reputation_test.rs` :

```rust
#[tokio::test]
async fn test_frontend_reputation_display() -> Result<()> {
    // Setup: Créer vendor avec reviews
    // Fetch page HTML
    // Parse et vérifier contenu
    Ok(())
}
```

---

## Étape 8 : Validation Finale

### Checklist Complète

- [ ] Handlers API déplacés et fonctionnels
- [ ] Routes ajoutées dans `main.rs`
- [ ] WASM build copié dans `static/js/`
- [ ] Templates créés (submit, reputation)
- [ ] CSS styling appliqué
- [ ] Tests manuels passés
- [ ] Vérification WASM client-side fonctionne
- [ ] Export IPFS fonctionnel

### Commandes de Vérification

```bash
# Compiler tout
cargo build --workspace

# Tests
cargo test --workspace

# Vérifier routes
curl http://localhost:8080/api/reputation/test-vendor-id

# Vérifier WASM load
curl http://localhost:8080/static/js/reputation_wasm.js
```

---

## Support & Questions

Si problèmes pendant l'intégration :

1. Vérifier logs : `tracing::info!()` dans handlers
2. Vérifier DB : `sqlite3 server/data/marketplace.db "SELECT * FROM reviews;"`
3. Vérifier IPFS : `curl http://127.0.0.1:5001/api/v0/version`
4. Vérifier WASM : Console browser (F12) → Network tab

**Gemini est disponible pour support !**
```

### Tests Requis (Milestone REP.5)

- [ ] `test_complete_reputation_flow`
- [ ] `test_submit_review_invalid_signature`
- [ ] `test_multiple_reviews_same_vendor`
- [ ] Coverage ≥ 80% global

**Total:** 3 tests E2E + coverage

### Validation Milestone 5

```bash
# Tests E2E
cargo test --package reputation --test integration

# Coverage globale (tous les packages reputation)
cargo tarpaulin --workspace \
    --exclude-files "*/tests/*" \
    --out Stdout

# Vérifier couverture ≥ 80%

# Vérifier docs
ls reputation/docs/
# REPUTATION-SPEC.md
# API-ENDPOINTS.md
# INTEGRATION-GUIDE.md
```

### Critères d'Acceptance

- [ ] 3 tests E2E passent
- [ ] Coverage ≥ 80% (global)
- [ ] 3 docs complètes (SPEC, API, INTEGRATION)
- [ ] Aucun warning `cargo clippy`
- [ ] Tout compilé en mode `--release`

---

## ✅ VALIDATION GLOBALE - Système de Réputation Complet

Après avoir terminé les 5 milestones, **Claude vérifiera TOUT** avec :

### Checklist Finale

**Structure:**
- [ ] Dossier `reputation/` à la racine
- [ ] ~25 fichiers créés
- [ ] Aucun fichier en dehors de `reputation/`

**Types & Crypto:**
- [ ] Types compilent (common)
- [ ] Signatures ed25519 fonctionnelles (crypto)
- [ ] 9 tests unitaires passent
- [ ] Couverture ≥ 80%

**Backend API:**
- [ ] Migration SQL appliquée
- [ ] Schema Diesel généré
- [ ] 3 endpoints API compilent
- [ ] Fonctions DB fonctionnelles
- [ ] Client IPFS fonctionne
- [ ] 5 tests passent

**WASM:**
- [ ] Build WASM réussit
- [ ] Fichiers .wasm + .js générés
- [ ] Taille < 200 KB
- [ ] Vérification JavaScript fonctionne

**Intégration Escrow:**
- [ ] WebSocket event `ReviewInvitation`
- [ ] Trigger automatique implémenté
- [ ] 2 tests d'intégration passent

**Tests & Docs:**
- [ ] 3 tests E2E passent
- [ ] Coverage global ≥ 80%
- [ ] 3 docs complètes

---

## 🎯 WORKFLOW DE VALIDATION

### Après Chaque Milestone

**Vous (Gemini) faites :**
1. Créer tous les fichiers du milestone
2. Tester localement (cargo check, cargo test)
3. M'envoyer message : **"Milestone REP.X terminé"**

**Je (Claude) ferai :**
```bash
# Structure
ls -R reputation/

# Compilation
cargo check --manifest-path reputation/Cargo.toml

# Tests
cargo test --manifest-path reputation/Cargo.toml

# Coverage
cargo tarpaulin --manifest-path reputation/Cargo.toml --out Stdout
```

**Je (Claude) répondrai :**
- ✅ **OK** → Continuer milestone suivant
- 🔴 **Corrections nécessaires** → Liste des problèmes

### Timeline Recommandée

**Jour 1-3:** Milestone REP.1 (Types + Crypto)
**Jour 4-6:** Milestone REP.2 (Backend API)
**Jour 7-9:** Milestone REP.3 (WASM)
**Jour 10-12:** Milestone REP.4 (Intégration Escrow)
**Jour 13-14:** Milestone REP.5 (Tests + Docs)

---

## 🔗 POINTS D'INTÉGRATION FUTURS

**Après que Claude termine Phase 4 Frontend ET vous terminez REP.5 :**

### Intégration Finale (1 jour, par Claude)

1. **Merge Handlers**
   ```bash
   mv reputation/server/handlers/reputation.rs server/src/handlers/
   ```

2. **Add Routes**
   - Ajouter dans `server/src/main.rs`

3. **Copy WASM**
   ```bash
   cp reputation/wasm/pkg/* static/js/
   ```

4. **Create UI Templates**
   - `templates/reviews/submit.html`
   - `templates/vendor/reputation.html`

5. **Tests Intégration**
   - Tests E2E complets (frontend + backend)

---

## 📚 RÉFÉRENCES

**Toutes les spécifications détaillées sont dans :**

- `reputation/docs/REPUTATION-SPEC.md` (Architecture technique)
- `reputation/docs/API-ENDPOINTS.md` (API REST)
- `reputation/docs/INTEGRATION-GUIDE.md` (Intégration par Claude)

**Dépendances principales :**

- `ed25519-dalek = "2.1"` (Signatures)
- `sha2 = "0.10"` (Hashing)
- `base64 = "0.22"` (Encoding)
- `wasm-bindgen = "0.2"` (WASM bindings)
- `serde = "1.0"` (Serialization)
- `diesel = "2.1"` (Database ORM)
- `reqwest = "0.11"` (HTTP client pour IPFS)

---

## 🚀 COMMENCEZ MAINTENANT

**Première action :**

```bash
# Créer dossier reputation à la racine du projet
mkdir -p reputation/

# Commencer Milestone REP.1 : Types & Cryptographie
mkdir -p reputation/common/src/
mkdir -p reputation/crypto/src/

# Créer premiers fichiers
touch reputation/common/Cargo.toml
touch reputation/common/src/lib.rs
touch reputation/common/src/types.rs
```

**Bonne chance ! Je (Claude) suis prêt à valider votre travail après chaque milestone.**

---

**Version:** 1.0
**Date:** 2025-10-21
**Durée Estimée:** 14 jours (5 milestones)
**Coverage Minimum:** 80%
