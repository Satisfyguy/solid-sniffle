# Plan d'Implémentation - Fonctionnalités Compétitives 2025 (RÉVISÉ)

**Version:** 2.0.0 (Révisé après audit codebase)
**Date:** 2025-10-28
**Status:** Planification révisée
**Objectif:** Implémenter 3 différenciateurs majeurs + améliorer 1 feature existante face aux concurrents (TorZon, Abacus, STYX, etc.)

---

## 🔍 ANALYSE PRÉALABLE - Features Existantes vs. Proposées

**Découverte importante:** Après audit du codebase, plusieurs features proposées **EXISTENT DÉJÀ** et sont même **SUPÉRIEURES** aux implémentations concurrentes!

### Tableau Récapitulatif

| Feature Proposée | Status | Localisation Code | Action |
|-----------------|--------|-------------------|---------|
| **Auto-finalization Timer** | ✅ **EXISTE** | `server/src/services/timeout_monitor.rs` | ❌ SKIP (déjà complet) |
| **Système Réputation** | ✅ **EXISTE (meilleur!)** | `reputation/` (workspace complet) | ❌ SKIP (supérieur à PGP) |
| **Arbitrage Disputes** | 🟡 **PARTIEL** | `custodial/src/arbitration.rs` | 🔧 AMÉLIORER (ajouter DAO) |
| **Multi-language Support** | ❌ **ABSENT** | N/A | ✅ IMPLÉMENTER |
| **Phishing Protection** | ❌ **ABSENT** | N/A | ✅ IMPLÉMENTER |
| **Vendor Bond Multisig** | ❌ **ABSENT** | N/A | ✅ IMPLÉMENTER |

---

## ✅ CE QUI EXISTE DÉJÀ (Audit Technique)

### 1. ⏰ Auto-Finalization Timer - COMPLET ET OPÉRATIONNEL

**Implémentation actuelle:**
- **Fichier:** `server/src/services/timeout_monitor.rs` (150+ lignes)
- **Configuration:** `server/src/config/timeout.rs` (207 lignes)
- **Database:** Champs `expires_at`, `last_activity_at`, `multisig_phase` dans table `escrows`

**Fonctionnalités:**
- ✅ Background worker avec polling toutes les 60s
- ✅ Timeouts configurables par status:
  - Multisig setup: 1 heure
  - Funding: 24 heures
  - Transaction confirmation: 6 heures
  - Dispute resolution: 7 jours
- ✅ Warning notifications 1h avant expiration
- ✅ Auto-cancellation des escrows expirés
- ✅ WebSocket notifications en temps réel
- ✅ Méthodes: `check_expired_escrows()`, `check_expiring_escrows()`

**Verdict:** ✅ **100% FONCTIONNEL - RIEN À AJOUTER**

---

### 2. 📜 Système de Réputation - SUPÉRIEUR AU PGP IMPORT

**Ma proposition originale:** Importer réputation de marketplaces legacy avec signatures PGP.

**Ce qui existe DÉJÀ (et c'est MEILLEUR):**

#### Architecture Actuelle

**Composants:**
- **Workspace Rust complet:** `reputation/` (2500+ lignes)
  - `reputation/common/` - Types (`SignedReview`, `VendorReputation`)
  - `reputation/crypto/` - Cryptographie ed25519
  - `reputation/wasm/` - Module WASM pour vérification client-side
  - `reputation/tests/` - 15 tests (100% coverage)

- **Backend Integration:**
  - `server/src/db/reputation.rs` (296 lignes) - DB operations
  - `server/src/handlers/reputation.rs` (542 lignes) - API REST
  - `server/src/handlers/reputation_ipfs.rs` (198 lignes) - Export IPFS

- **Database:** Table `reviews` complète avec champs:
  ```sql
  reviews (
      id, txid, reviewer_id, vendor_id,
      rating, comment, buyer_pubkey, signature,
      timestamp, verified, created_at
  )
  ```

#### Fonctionnalités Implémentées

✅ **Signatures cryptographiques ed25519** (plus modernes que PGP RSA)
✅ **Reviews liés aux transactions Monero** (txid = preuve on-chain)
✅ **Export IPFS** pour portabilité inter-marketplaces
✅ **Vérification client-side WASM** (226 KB, zero-trust)
✅ **Impossible à falsifier** (signature + SHA-256 hash)
✅ **Statistiques auto-calculées** (average, distribution, oldest/newest)
✅ **API REST complète:**
  - `POST /api/reviews` - Submit signed review
  - `GET /api/reputation/{vendor_id}` - Get reputation file
  - `GET /api/reputation/{vendor_id}/stats` - Quick stats
  - `POST /api/reputation/export` - Export to IPFS

✅ **Frontend intégré:**
  - Templates Tera (`vendor_profile.html`, `submit_review.html`)
  - JavaScript WASM bindings (`static/js/reputation-verify.js`)
  - CSS glassmorphism + HTMX

✅ **Tests complets:** 15/15 tests passent (100% coverage)

#### Comparaison: PGP Import vs. Système Actuel

| Critère | Mon PGP Import Proposé | Système Actuel (ed25519 + IPFS) |
|---------|------------------------|----------------------------------|
| **Proof de transaction** | ❌ Non (juste signature PGP) | ✅ Oui (txid Monero on-chain) |
| **Portabilité** | 🟡 Via PGP keys | ✅ Export IPFS (JSON standard) |
| **Vérification** | 🟡 Manuelle par admin | ✅ Automatique (crypto + WASM) |
| **Faux positifs** | ⚠️ Possible (PGP key volée) | ❌ Impossible (lié au txid blockchain) |
| **Complexité** | 🔴 Haute (validation admin) | 🟢 Faible (automatisé) |
| **Client-side verification** | ❌ Non | ✅ Oui (WASM 226 KB) |
| **Résistance Sybil** | 🟡 Moyenne | ✅ Haute (coût txid = proof of work) |

**Verdict:** ✅ **SYSTÈME ACTUEL ARCHITECTURALEMENT SUPÉRIEUR - GARDER TEL QUEL**

**Documentation:** Voir `reputation/README.md` pour guide complet.

---

### 3. ⚖️ Système d'Arbitrage - PARTIELLEMENT IMPLÉMENTÉ

**Ce qui existe:**
- **Fichier:** `custodial/src/arbitration.rs` (150+ lignes)
- **Handlers:** `server/src/handlers/airgap_dispute.rs` (100+ lignes)
- **Script:** `create_arbiter.rs` (racine projet)

**Fonctionnalités actuelles:**
- ✅ Air-gap dispute system (arbiter wallet offline)
- ✅ Export QR code pour transfert offline
- ✅ `ArbitrationEngine` avec règles automatiques
- ✅ `EvidenceAnalysis` (photos, tracking, chat logs)
- ✅ Confidence scoring (0.0-1.0)
- ✅ Manual review escalation si confidence < 0.8

**Ce qui MANQUE (ma proposition DAO):**
- ❌ Pool d'arbitres multiples (actuellement: 1 seul arbiter système)
- ❌ Arbitres élus par communauté (voting)
- ❌ Voting weighted par stake + réputation
- ❌ Slashing pour mauvaises décisions
- ❌ Appeal system avec nouveaux arbitres
- ❌ Term rotation (3 mois)

**Action:** 🔧 **AMÉLIORER** (ajouter couche DAO sur base existante)

---

## ❌ CE QUI N'EXISTE PAS (À Implémenter)

### Audit Complet

```bash
# Recherche multi-language support
grep -r "i18n\|locale\|translation" server/
# Résultat: 0 fichiers ❌

# Recherche phishing protection
grep -r "mirror\|phishing\|canary" server/
# Résultat: 0 fichiers ❌

# Recherche vendor bonds
grep -r "vendor.*bond\|bond.*vendor" server/
# Résultat: 0 fichiers ❌
```

**Conclusion:** 3 features critiques absentes du codebase.

---

## 📋 Plan Révisé - Vue d'Ensemble

### Fonctionnalités Cibles (RÉVISÉ)

| # | Feature | Status | Priorité | Complexité | Estimation | Valeur Business |
|---|---------|--------|----------|------------|------------|-----------------|
| ~~1~~ | ~~Auto-finalization Timer~~ | ✅ **EXISTE** | N/A | N/A | 0h | N/A |
| 1 | **Multi-language Support** | ❌ À faire | **HAUTE** | Faible | 6-10h | ⭐⭐⭐⭐ |
| ~~2~~ | ~~Vendor Reputation Import~~ | ✅ **EXISTE (meilleur)** | N/A | N/A | 0h | N/A |
| 2 | **Phishing Protection System** | ❌ À faire | **HAUTE** | Moyenne | 10-16h | ⭐⭐⭐⭐ |
| 3 | **Vendor Bond Multisig** | ❌ À faire | Moyenne | Très Haute | 24-40h | ⭐⭐⭐⭐⭐ |
| 4 | **Dispute Resolution DAO** | 🟡 Améliorer | Basse | Haute | 20-30h | ⭐⭐⭐⭐⭐ |

**Total estimé RÉVISÉ:** 60-96 heures (8-12 jours de dev)
**Total original:** 104-162 heures (13-20 jours)
**Économie:** 44-66 heures grâce aux features existantes! 🎉

---

## 🎯 Phase 1: Foundation (Quick Win)

**Objectif:** Implémenter multi-language support pour audience internationale
**Durée estimée:** 6-10 heures (1 jour)
**Dépendances:** Aucune

### 1.1 Multi-Language Support 🌍

**Problème résolu:** Marketplace 100% anglophone limite audience
**Inspiration:** WeTheNorth (EN/FR bilingual = succès canadien)

**Impact Business:** TorZon/Abacus supportent EN/FR/RU/DE → vous perdez marchés non-anglophones (Russie = 40% du dark web).

#### Spécification Technique

**Approche: i18n avec Fluent (Mozilla)**

Pourquoi Fluent?
- Supporte pluralization complexe (1 item vs 2 items en français)
- Formatage natif des dates/nombres
- Fallback automatique vers langue par défaut
- Utilisé par Firefox, Discord, Dropbox

**Architecture:**

```
server/
├── locales/
│   ├── en/         # English (default)
│   │   ├── common.ftl
│   │   ├── orders.ftl
│   │   ├── listings.ftl
│   │   └── errors.ftl
│   ├── fr/         # Français
│   ├── ru/         # Русский
│   ├── de/         # Deutsch
│   └── es/         # Español
```

**Cargo.toml additions:**
```toml
[dependencies]
fluent = "0.16"
fluent-bundle = "0.15"
unic-langid = "0.9"
```

**Exemple de fichier de traduction (locales/en/common.ftl):**
```fluent
# Navigation
nav-home = Home
nav-listings = Listings
nav-orders = Orders
nav-profile = Profile

# Auth
login-title = Login to Marketplace
login-username = Username
login-password = Password
login-submit = Sign In
login-error-invalid = Invalid credentials

# Pluralization
item-count =
    { $count ->
        [one] 1 item
       *[other] { $count } items
    }

# Dates
order-created = Order created { DATETIME($date, dateStyle: "short") }
```

**Exemple français (locales/fr/common.ftl):**
```fluent
nav-home = Accueil
nav-listings = Annonces
nav-orders = Commandes
nav-profile = Profil

login-title = Connexion au Marketplace
login-username = Nom d'utilisateur
login-password = Mot de passe
login-submit = Se connecter
login-error-invalid = Identifiants invalides

item-count =
    { $count ->
        [one] 1 article
       *[other] { $count } articles
    }
```

**Middleware (server/src/middleware/i18n.rs):**
```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use fluent::{FluentBundle, FluentResource};

pub struct I18nLayer {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
}

impl I18nLayer {
    pub fn new() -> Result<Self> {
        let mut bundles = HashMap::new();

        for lang in &["en", "fr", "ru", "de", "es"] {
            let bundle = load_bundle(lang)?;
            bundles.insert(lang.to_string(), bundle);
        }

        Ok(Self { bundles })
    }

    fn get_user_locale(&self, req: &Request) -> String {
        // 1. Check cookie
        if let Some(cookie) = req.headers().get("cookie") {
            if let Some(lang) = extract_lang_from_cookie(cookie) {
                return lang;
            }
        }

        // 2. Check Accept-Language header
        if let Some(accept) = req.headers().get("accept-language") {
            if let Some(lang) = parse_accept_language(accept) {
                return lang;
            }
        }

        // 3. Default to English
        "en".to_string()
    }
}

pub async fn i18n_middleware(
    State(i18n): State<I18nLayer>,
    mut req: Request,
    next: Next,
) -> Response {
    let locale = i18n.get_user_locale(&req);
    req.extensions_mut().insert(Locale(locale));
    next.run(req).await
}
```

**Tera Filter (server/src/templates/filters.rs):**
```rust
use tera::{Value, Result as TeraResult};

pub fn translate(value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
    let key = value.as_str().ok_or("Key must be string")?;
    let locale = args.get("locale")
        .and_then(|v| v.as_str())
        .unwrap_or("en");

    let translation = I18N
        .get_bundle(locale)
        .and_then(|bundle| bundle.get_message(key))
        .map(|msg| msg.value().unwrap())
        .unwrap_or(key);  // Fallback to key if not found

    Ok(Value::String(translation.to_string()))
}
```

**Template Usage (templates/listings/index.html):**
```html
<h1>{{ "listings-title" | translate(locale=user.locale) }}</h1>

<p>
  {{ "item-count" | translate(locale=user.locale, count=listings.len()) }}
</p>

<!-- Language Switcher -->
<div class="language-switcher">
    <select hx-post="/user/set-locale" hx-trigger="change">
        <option value="en" {% if user.locale == "en" %}selected{% endif %}>🇬🇧 English</option>
        <option value="fr" {% if user.locale == "fr" %}selected{% endif %}>🇫🇷 Français</option>
        <option value="ru" {% if user.locale == "ru" %}selected{% endif %}>🇷🇺 Русский</option>
        <option value="de" {% if user.locale == "de" %}selected{% endif %}>🇩🇪 Deutsch</option>
        <option value="es" {% if user.locale == "es" %}selected{% endif %}>🇪🇸 Español</option>
    </select>
</div>
```

**Database Migration (users table):**
```sql
-- Migration: add_locale_to_users
ALTER TABLE users ADD COLUMN locale VARCHAR(5) DEFAULT 'en';
CREATE INDEX idx_users_locale ON users(locale);
```

**Handler (server/src/handlers/user.rs):**
```rust
pub async fn set_locale(
    State(state): State<AppState>,
    session: Session,
    Json(locale): Json<LocaleChange>,
) -> Result<StatusCode, AppError> {
    // Validate locale
    if !SUPPORTED_LOCALES.contains(&locale.locale.as_str()) {
        return Err(AppError::BadRequest("Unsupported locale".into()));
    }

    // Save to database
    let user_id = session.get_user_id()?;
    update_user_locale(&state.pool, &user_id, &locale.locale).await?;

    // Set cookie
    let cookie = Cookie::build("locale", locale.locale)
        .path("/")
        .max_age(Duration::days(365))
        .http_only(true)
        .secure(true)
        .finish();

    Ok(StatusCode::OK)
}
```

**Checklist d'implémentation:**
- [ ] Ajouter dépendances Fluent au Cargo.toml
- [ ] Créer structure `server/locales/en/`
- [ ] Extraire toutes les strings hardcodées en anglais
- [ ] Créer middleware i18n
- [ ] Ajouter colonne `locale` aux users (migration)
- [ ] Créer handler `/user/set-locale`
- [ ] Ajouter filtre Tera `translate`
- [ ] Créer language switcher component
- [ ] Traduire en français (priorité pour test)
- [ ] Recruter traducteurs natifs pour RU/DE/ES
- [ ] Tester avec différentes locales
- [ ] Documenter dans `docs/specs/i18n.md`

**Estimation:** 6-10 heures (base), +4h par langue additionnelle

---

## 🔐 Phase 2: Security Layer

**Objectif:** Sécurité anti-phishing
**Durée estimée:** 10-16 heures (2 jours)
**Dépendances:** Phase 1 (multi-language pour UI)

### 2.1 Phishing Protection System 🛡️

**Problème résolu:** Fake mirrors volent credentials
**Inspiration:** TorZon distribue PGP-signed mirror links
**Impact:** BidenCash avait 145 domaines clones avant seizure

#### Spécification Technique

**Concept:** Signer cryptographiquement les URLs officielles + vérification automatique

**Components:**

1. **Official Mirror Registry (hardcoded)**
```rust
// server/src/config/mirrors.rs
pub const OFFICIAL_MIRRORS: &[&str] = &[
    "http://marketxyz...onion",  // Primary
    "http://marketabc...onion",  // Mirror 1
    "http://marketdef...onion",  // Mirror 2
];

pub const MARKETPLACE_PGP_KEY: &str = r#"
-----BEGIN PGP PUBLIC KEY BLOCK-----
...
-----END PGP PUBLIC KEY BLOCK-----
"#;
```

2. **PGP-Signed Mirror List (généré périodiquement)**
```rust
// server/src/tasks/sign_mirrors.rs
pub async fn generate_signed_mirror_list() -> Result<String> {
    let message = format!(
        "Official Monero Marketplace Mirrors (Valid until: {})\n\n{}\n\nVerify at: /verify-mirror",
        Utc::now() + Duration::days(30),
        OFFICIAL_MIRRORS.join("\n")
    );

    // Sign with marketplace private key
    let signed = sign_message(&message, &MARKETPLACE_PRIVATE_KEY)?;

    Ok(signed)
}
```

3. **Mirror Verification Endpoint**
```rust
// server/src/handlers/security.rs
pub async fn verify_mirror(
    State(state): State<AppState>,
    Query(params): Query<VerifyMirrorParams>,
) -> Result<Json<MirrorVerification>, AppError> {
    let url = params.url;

    let is_official = OFFICIAL_MIRRORS.iter()
        .any(|mirror| url.starts_with(mirror));

    let verification = MirrorVerification {
        url: url.clone(),
        is_official,
        checked_at: Utc::now(),
        message: if is_official {
            "✅ This is an official Monero Marketplace mirror"
        } else {
            "⚠️ WARNING: This URL is NOT an official mirror - do not enter credentials!"
        }.to_string(),
        official_mirrors: OFFICIAL_MIRRORS.iter().map(|s| s.to_string()).collect(),
    };

    Ok(Json(verification))
}
```

4. **Browser Extension (optionnel mais haute valeur)**
```javascript
// browser-extension/manifest.json
{
  "manifest_version": 3,
  "name": "Monero Marketplace - Phishing Protection",
  "version": "1.0.0",
  "permissions": ["tabs", "storage"],
  "background": {
    "service_worker": "background.js"
  },
  "content_scripts": [{
    "matches": ["*://*.onion/*"],
    "js": ["content.js"]
  }]
}

// browser-extension/background.js
const OFFICIAL_MIRRORS = [
  'http://marketxyz...onion',
  'http://marketabc...onion',
];

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.url) {
    const isOfficial = OFFICIAL_MIRRORS.some(mirror =>
      tab.url.startsWith(mirror)
    );

    if (!isOfficial && tab.url.includes('marketplace')) {
      chrome.tabs.executeScript(tabId, {
        code: `
          if (document.querySelector('input[type="password"]')) {
            document.body.innerHTML = '<div style="background: red; color: white; padding: 50px; text-align: center; font-size: 24px;">⚠️ PHISHING SITE DETECTED ⚠️<br><br>This is NOT an official Monero Marketplace mirror.<br><br>Do NOT enter your credentials!</div>';
          }
        `
      });
    }
  }
});
```

5. **Canary System (dead man's switch)**
```rust
// server/src/tasks/canary.rs
pub const CANARY_MESSAGE: &str = r#"
Monero Marketplace Warrant Canary
Last Updated: {date}

The Monero Marketplace operators have NOT received:
- National Security Letters
- FISA court orders
- Gag orders preventing disclosure
- Requests to install backdoors or monitoring
- Seizure warrants for servers or domains

PGP Signature: {signature}

If this canary is not updated within 30 days, assume compromise.
"#;

pub async fn publish_canary() -> Result<()> {
    let message = CANARY_MESSAGE.replace("{date}", &Utc::now().to_rfc3339());
    let signed = sign_message(&message, &MARKETPLACE_PRIVATE_KEY)?;

    // Publish to multiple channels
    publish_to_dread(&signed).await?;
    publish_to_pastebin(&signed).await?;
    publish_to_github(&signed).await?;

    Ok(())
}
```

6. **Frontend Warning System**
```html
<!-- templates/_phishing_warning.html -->
<div class="phishing-warning-banner" id="phishing-check">
    <div class="warning-content">
        <span class="warning-icon">🛡️</span>
        <span>Checking if this is an official mirror...</span>
    </div>
</div>

<script>
// Check mirror on page load
document.addEventListener('DOMContentLoaded', async () => {
    const currentUrl = window.location.href;

    try {
        const response = await fetch(`/api/verify-mirror?url=${encodeURIComponent(currentUrl)}`);
        const result = await response.json();

        const banner = document.getElementById('phishing-check');

        if (result.is_official) {
            banner.className = 'phishing-warning-banner success';
            banner.innerHTML = '<span class="warning-icon">✅</span><span>Official mirror verified</span>';
            setTimeout(() => banner.style.display = 'none', 3000);
        } else {
            banner.className = 'phishing-warning-banner danger';
            banner.innerHTML = `
                <div class="danger-content">
                    <h2>⚠️ PHISHING SITE DETECTED ⚠️</h2>
                    <p>This URL is NOT an official Monero Marketplace mirror.</p>
                    <p><strong>DO NOT enter your credentials!</strong></p>
                    <h3>Official Mirrors:</h3>
                    <ul>${result.official_mirrors.map(m => `<li><code>${m}</code></li>`).join('')}</ul>
                </div>
            `;

            // Disable all forms
            document.querySelectorAll('form').forEach(form => {
                form.addEventListener('submit', (e) => {
                    e.preventDefault();
                    alert('Forms disabled for your protection');
                });
            });
        }
    } catch (e) {
        console.error('Mirror verification failed:', e);
    }
});
</script>
```

**Checklist d'implémentation:**
- [ ] Générer PGP keypair pour marketplace
- [ ] Créer `config/mirrors.rs` avec OFFICIAL_MIRRORS
- [ ] Implémenter `/api/verify-mirror` endpoint
- [ ] Créer signed mirror list generator
- [ ] Ajouter banner JavaScript sur toutes les pages
- [ ] Créer page `/security` avec PGP key + mirrors
- [ ] Implémenter canary system
- [ ] Créer browser extension (Firefox + Chrome)
- [ ] Publier extension sur AMO/Chrome Store
- [ ] Documenter dans `docs/PHISHING-PROTECTION.md`

**Estimation:** 10-16 heures

---

## 💎 Phase 3: Advanced Security Features

**Objectif:** Features complexes nécessitant infrastructure multisig + DAO
**Durée estimée:** 44-70 heures (6-9 jours)
**Dépendances:** Phase 1 + 2, infrastructure wallet mature

### 3.1 Vendor Bond Multisig 🔒

**Problème résolu:** Admins peuvent voler vendor bonds, vendors perdent bonds sur exit scam
**Innovation:** Bonds verrouillés en 2-of-3 multisig (vendor + marketplace + timelock arbiter)

#### Spécification Technique

**Concept:** Le vendor bond n'est PAS contrôlé par le marketplace, mais par un multisig où le vendor peut récupérer son bond après X mois d'inactivité.

**Architecture du Bond Multisig:**
```
Participants:
1. Vendor (clé 1)
2. Marketplace (clé 2)
3. Timelock Arbiter (clé 3, activé après 180 jours d'inactivité)

Scénarios:
- Vendor veut retirer bond après bon comportement: Vendor + Marketplace
- Vendor exit scam: Marketplace + Timelock Arbiter (après investigation)
- Marketplace exit scam: Vendor + Timelock Arbiter (après 180 jours)
```

**Database Schema:**
```sql
-- Migration: add_vendor_bonds_multisig
CREATE TABLE vendor_bonds (
    id TEXT PRIMARY KEY,
    vendor_id TEXT NOT NULL REFERENCES users(id),
    amount_atomic BIGINT NOT NULL,  -- XMR in atomic units
    status TEXT DEFAULT 'pending',  -- pending, active, released, slashed

    -- Multisig wallet details
    multisig_address TEXT,
    vendor_multisig_info TEXT,
    marketplace_multisig_info TEXT,
    timelock_multisig_info TEXT,
    multisig_wallet_state TEXT,  -- JSON state

    -- Lifecycle
    created_at TIMESTAMP DEFAULT NOW(),
    activated_at TIMESTAMP,
    released_at TIMESTAMP,
    last_activity_at TIMESTAMP,

    -- Terms
    bond_period_days INTEGER DEFAULT 180,  -- Timelock period
    slash_conditions JSONB,  -- Rules for slashing

    UNIQUE(vendor_id)
);

CREATE INDEX idx_vendor_bonds_status ON vendor_bonds(status);
CREATE INDEX idx_vendor_bonds_activity ON vendor_bonds(last_activity_at);
```

**Pour spécifications complètes:** Voir section 3.1 du plan original (architecture détaillée conservée).

**Checklist d'implémentation:**
- [ ] Créer migration `add_vendor_bonds_multisig`
- [ ] Créer modèle `VendorBond`
- [ ] Implémenter handlers create/setup/activate/release
- [ ] Créer timelock monitoring worker
- [ ] Implémenter slash logic (avec voting?)
- [ ] Créer templates vendor bond UI
- [ ] Ajouter tests E2E pour bond lifecycle
- [ ] Tester avec testnet XMR
- [ ] Documenter dans `docs/specs/vendor_bonds_multisig.md`
- [ ] Créer guide vendor complet

**Estimation:** 24-40 heures

---

### 3.2 Dispute Resolution DAO (Amélioration) 🏛️

**Status actuel:** Base arbitrage existe dans `custodial/src/arbitration.rs`

**Ce qui EXISTE:**
- ✅ `ArbitrationEngine` avec règles
- ✅ `EvidenceAnalysis` (photos, tracking, chat)
- ✅ Confidence scoring (0.0-1.0)
- ✅ Air-gap dispute export/import (QR codes)
- ✅ Manual review escalation

**Ce qui MANQUE (à ajouter):**
- ❌ Pool d'arbitres multiples (actuellement: 1 seul)
- ❌ Arbitres élus par communauté
- ❌ Voting weighted par stake + réputation
- ❌ Slashing pour mauvaises décisions
- ❌ Appeal system avec nouveaux arbitres
- ❌ Term rotation (3 mois)

**Approche:** Améliorer l'`ArbitrationEngine` existant avec couche DAO.

#### Spécification Technique

**Database Schema:**
```sql
-- Migration: add_dispute_resolution_dao

CREATE TABLE arbiters (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    stake_atomic BIGINT NOT NULL,
    status TEXT DEFAULT 'pending',  -- pending, active, suspended, slashed

    -- Stats
    disputes_resolved INTEGER DEFAULT 0,
    consensus_rate REAL DEFAULT 0.0,  -- % of times agreed with majority
    appeal_overturn_rate REAL DEFAULT 0.0,  -- % of decisions overturned
    reputation_score REAL DEFAULT 0.0,

    -- Lifecycle
    elected_at TIMESTAMP,
    term_ends_at TIMESTAMP,
    last_assigned_at TIMESTAMP,

    created_at TIMESTAMP DEFAULT NOW(),

    UNIQUE(user_id)
);

CREATE TABLE arbiter_votes (
    id TEXT PRIMARY KEY,
    dispute_id TEXT NOT NULL REFERENCES disputes(id),
    arbiter_id TEXT NOT NULL REFERENCES arbiters(id),
    vote TEXT NOT NULL,  -- 'buyer', 'seller', 'split'
    reasoning TEXT,
    voted_at TIMESTAMP DEFAULT NOW(),

    UNIQUE(dispute_id, arbiter_id)
);

-- Update existing disputes table
ALTER TABLE disputes ADD COLUMN arbiter_1_id TEXT REFERENCES arbiters(id);
ALTER TABLE disputes ADD COLUMN arbiter_2_id TEXT REFERENCES arbiters(id);
ALTER TABLE disputes ADD COLUMN arbiter_3_id TEXT REFERENCES arbiters(id);
ALTER TABLE disputes ADD COLUMN final_decision TEXT;
ALTER TABLE disputes ADD COLUMN appeal_status TEXT;
```

**Integration avec ArbitrationEngine existant:**
```rust
// Amélioration de custodial/src/arbitration.rs
impl ArbitrationEngine {
    /// NOUVELLE MÉTHODE: Resolve with DAO voting
    pub async fn resolve_with_dao(&self, dispute: &Dispute, pool: &DbPool) -> Result<DisputeResolution> {
        // 1. Assign 3 arbiters (weighted random by reputation)
        let arbiters = assign_arbiters_to_dispute(pool, &dispute.id).await?;

        // 2. Notify arbiters via WebSocket
        for arbiter in &arbiters {
            notify_arbiter_assignment(arbiter.user_id, &dispute.id).await?;
        }

        // 3. Wait for votes (with timeout)
        let votes = wait_for_arbiter_votes(pool, &dispute.id, Duration::days(3)).await?;

        // 4. Calculate majority
        let decision = calculate_majority_vote(&votes)?;

        // 5. Update arbiter stats
        update_arbiter_consensus_rates(pool, &votes, &decision).await?;

        Ok(decision)
    }

    // GARDER méthode existante pour compatibilité
    pub async fn resolve(&self, dispute: &Dispute) -> Result<DisputeResolution> {
        // Méthode originale (règles automatiques)
        // ...
    }
}
```

**Checklist d'implémentation:**
- [ ] Créer migrations pour arbiters/votes/elections
- [ ] Implémenter weighted random selection
- [ ] Créer arbiter application flow
- [ ] Implémenter voting system (election + disputes)
- [ ] Créer appeal system avec new arbiter assignment
- [ ] Implémenter slashing logic
- [ ] Améliorer `ArbitrationEngine` avec méthode `resolve_with_dao()`
- [ ] Créer DAO governance UI
- [ ] Ajouter tests E2E pour complet flow
- [ ] Tester avec simulated elections
- [ ] Documenter dans `docs/specs/dispute_resolution_dao.md`
- [ ] Créer guide pour arbiters

**Estimation:** 20-30 heures (réduit car base existe)

---

## 📊 Récapitulatif & Roadmap

### Timeline Proposée (RÉVISÉE)

```
Semaine 1:    Phase 1 (Multi-language Support)
Semaine 2:    Phase 2 (Phishing Protection)
Semaine 3-4:  Phase 3 (Vendor Bond Multisig)
Semaine 5-6:  Phase 3 (DAO Amélioration)
```

**Comparaison:**
- **Plan original:** 13-20 jours (104-162h)
- **Plan révisé:** 8-12 jours (60-96h)
- **Économie:** 5-8 jours grâce aux features existantes! ✅

### Ordre d'Implémentation Recommandé

1. **Multi-language Support** (infrastructure)
2. **Phishing Protection** (sécurité critique)
3. **Vendor Bond Multisig** (nécessite wallet infrastructure stable)
4. **Dispute Resolution DAO** (feature la plus complexe, dernière)

### Métriques de Succès

**Phase 1:**
- [ ] Support multi-langue augmente signups internationaux de >50%

**Phase 2:**
- [ ] 0 phishing incidents reportés
- [ ] Browser extension installée par >30% utilisateurs

**Phase 3:**
- [ ] 100% vendor bonds en multisig (0% custodial)
- [ ] DAO gère >80% des disputes sans escalation

### Post-Implementation

**Marketing Push:**
- Créer comparaison table (vous vs. concurrents)
- Annoncer sur Dread, forums dark web
- Publier blog posts techniques
- Créer demo videos

**Documentation Utilisateur:**
- Guide: "Becoming a Vendor with Bond"
- Guide: "Becoming an Arbiter"
- FAQ: "Why Multisig Bonds are Better"

**Monitoring:**
- Métriques d'adoption par feature
- A/B testing messaging
- User feedback loops

---

## 🎯 Next Steps Immédiats

1. **Review ce plan révisé avec l'équipe**
2. **Prioriser 1-2 features pour MVP**
3. **Créer specs détaillées** (`./scripts/new-spec.sh feature_name`)
4. **Setup branch feature** (`git checkout -b feature/multi-language`)
5. **Commencer Phase 1**

---

## 📚 Références au Code Existant

### Auto-Finalization Timer (✅ COMPLET)
- `server/src/services/timeout_monitor.rs`
- `server/src/config/timeout.rs`
- Migration: `2025-10-26-175351-0000_add_timeout_fields_to_escrows`

### Système Réputation (✅ SUPÉRIEUR)
- `reputation/` (workspace complet)
- `server/src/db/reputation.rs`
- `server/src/handlers/reputation.rs`
- `server/src/handlers/reputation_ipfs.rs`
- Migration: `2025-10-22-000000-0000_create_reviews`
- **Documentation:** `reputation/README.md` (guide complet)

### Arbitrage Base (🟡 À AMÉLIORER)
- `custodial/src/arbitration.rs`
- `server/src/handlers/airgap_dispute.rs`
- `create_arbiter.rs`

---

**Questions? Besoin de clarifications sur une feature spécifique?**

**Version Control:**
- v1.0.0: Plan original (6 features, 104-162h)
- v2.0.0: Plan révisé après audit (4 features, 60-96h) ← ACTUEL
