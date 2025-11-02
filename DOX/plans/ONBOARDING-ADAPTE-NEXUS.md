# Onboarding Adapté au Monero Marketplace (Nexus)

**Document de Référence:** Adaptation du playbook stratégique d'onboarding marketplace au contexte privacy-first (Tor + Monero)
**Version:** 1.0
**Date:** 2025-11-02
**Statut:** 🟢 Référence Active

---

## Table des Matières

1. [Introduction & Contexte](#1-introduction--contexte)
2. [Analyse des Incompatibilités](#2-analyse-des-incompatibilités)
3. [Partie 1: Onboarding Vendeur (Privacy-Preserving)](#3-partie-1-onboarding-vendeur-privacy-preserving)
4. [Partie 2: Onboarding Acheteur (Speed & Privacy)](#4-partie-2-onboarding-acheteur-speed--privacy)
5. [Partie 3: Stack Technique Adaptée](#5-partie-3-stack-technique-adaptée)
6. [Partie 4: KPIs & Mesure (Anonymisés)](#6-partie-4-kpis--mesure-anonymisés)
7. [Roadmap d'Implémentation](#7-roadmap-dimplémentation)
8. [Actions Immédiates](#8-actions-immédiates)
9. [Références](#9-références)

---

## 1. Introduction & Contexte

### 1.1 Philosophie du Projet

Le **Monero Marketplace (Nexus)** est une marketplace **privacy-first** qui opère sur les principes suivants:

- **Anonymat par défaut**: Tor hidden service + pseudonymes
- **Trustless transactions**: Escrow multisig 2-of-3 (pas de tiers de confiance)
- **Zéro KYC**: Pas de vérification d'identité, jamais
- **OPSEC strict**: Pas de logs sensibles, pas de tracking nominatif

### 1.2 Le Défi de l'Onboarding Privacy-First

Le playbook traditionnel ([ONBOARDING.md](../../ONBOARDING.md)) repose sur des principes incompatibles avec notre philosophie:

| Principe Traditionnel | Incompatibilité Nexus |
|----------------------|----------------------|
| **Confiance par vérification** (KYC/KYB) | ❌ Détruit l'anonymat |
| **PSP tiers** (Stripe, Lemonway) | ❌ Nécessitent identité légale |
| **Tracking utilisateur** (IP, cookies, analytics) | ❌ Compromet OPSEC |
| **Email marketing** | ❌ Crée un lien identitaire |

**Notre défi:** Construire un système d'onboarding qui génère de la liquidité (connexion offre/demande) **sans sacrifier la privacy**.

### 1.3 Objectifs de ce Document

1. **Identifier** les concepts applicables du playbook traditionnel
2. **Adapter** les stratégies d'onboarding au contexte privacy-first
3. **Proposer** une roadmap d'implémentation concrète
4. **Définir** des KPIs mesurables respectueux de la privacy

---

## 2. Analyse des Incompatibilités

### 2.1 Matrice d'Applicabilité

| Concept Playbook | Statut | Adaptation Requise |
|-----------------|--------|-------------------|
| **Double flux Vendeur/Acheteur** | ✅ Compatible | Aucune (principe universel) |
| **Segmentation utilisateurs** | ✅ Compatible | Auto-déclaration (pas de vérification) |
| **KYC/KYB (Know Your Customer)** | ❌ Incompatible | Remplacer par wallet setup + réputation |
| **PSP API (Stripe Connect)** | ❌ Incompatible | Remplacer par monero-wallet-rpc |
| **Time-to-Listing (TTL)** | ✅ Compatible | Aucune (métrique universelle) |
| **Progressive Onboarding** | ✅ Compatible | Aucune (UX pattern universel) |
| **"Moment Aha!"** | ✅ Compatible | Adapter: focus sur sécurité escrow |
| **Gamification** | ✅ Compatible | Badges anonymes (pas de leaderboards nominatifs) |
| **Email sequences** | ❌ Incompatible | Remplacer par notifications in-app |
| **A/B Testing** | ⚠️ Partiellement | Analytics anonymes uniquement |
| **Dashboard vendeur** | ✅ Compatible | Métriques agrégées (pas de données individuelles clients) |

### 2.2 Substitutions Majeures

#### KYC/KYB → Wallet Setup + Réputation Cryptographique

```
┌─────────────────────────────────────────────────────────────┐
│              MARKETPLACE TRADITIONNELLE                      │
├─────────────────────────────────────────────────────────────┤
│ 1. Inscription (email, nom, adresse)                        │
│ 2. Upload documents (ID, Kbis)                              │
│ 3. Attente validation manuelle (2-5 jours)                  │
│ 4. Compte "vérifié" → Peut vendre                           │
│                                                              │
│ Confiance = Vérification d'Identité Légale                  │
└─────────────────────────────────────────────────────────────┘

                          VS

┌─────────────────────────────────────────────────────────────┐
│                    NEXUS (PRIVACY-FIRST)                     │
├─────────────────────────────────────────────────────────────┤
│ 1. Inscription (username unique, password)                   │
│ 2. Setup wallet multisig (5 min, automatisé)                │
│ 3. Validation cryptographique (is_multisig() = true)        │
│ 4. Compte "prêt" → Peut vendre                              │
│ 5. Réputation accumulée via transactions (proof-of-trade)   │
│                                                              │
│ Confiance = Cryptographie + Historique Anonyme              │
└─────────────────────────────────────────────────────────────┘
```

**Avantage Nexus:**
- ✅ Instantané (5 min vs 2-5 jours)
- ✅ Pas de friction légale
- ✅ Anonymat préservé
- ✅ Trustless (pas de tiers validateur)

**Désavantage:**
- ⚠️ Barrière technique (comprendre multisig)
- ⚠️ Nouveaux vendeurs = 0 réputation (cold start)

**Solution:** Wizard éducatif + système de bond optionnel (vendeur dépose XMR comme garantie pour booster réputation initiale)

---

## 3. Partie 1: Onboarding Vendeur (Privacy-Preserving)

### 3.1 Vue d'Ensemble du Flux

```
┌─────────────────────────────────────────────────────────────┐
│                  FLUX ONBOARDING VENDEUR NEXUS               │
└─────────────────────────────────────────────────────────────┘

[Inscription] → [Segmentation] → [Wallet Setup] → [1ère Listing] → [Activation]
    ↓              ↓                  ↓                ↓              ↓
  30 sec      Auto-déclaré      5 min (wizard)    Variable      Services+

KPI Clés:
- Time-to-Account: < 1 min
- Time-to-Multisig-Ready: < 10 min (médiane)
- Time-to-First-Listing: < 30 min (médiane)
- Taux Activation Vendeur: (Vendeurs avec ≥1 listing actif) / (Total inscrits en tant que vendor)
```

### 3.2 Étape 1: Inscription & Segmentation Anonyme

#### 3.2.1 Interface d'Inscription

**Formulaire minimal (3 champs):**
```html
<!-- templates/auth/register.html -->
<form action="/auth/register" method="POST">
    <input type="text" name="username"
           placeholder="Choose anonymous username"
           pattern="[a-zA-Z0-9_]{3,20}"
           required>

    <input type="password" name="password"
           minlength="12"
           required>

    <select name="role">
        <option value="buyer">I want to buy</option>
        <option value="vendor">I want to sell</option>
    </select>

    <button type="submit">Create Anonymous Account</button>
</form>
```

**❌ NE PAS demander:**
- Email
- Téléphone
- Nom réel
- Adresse
- Date de naissance
- CAPTCHA (crée friction + peut tracker)

**✅ Sécurité alternative:**
- Rate limiting par IP (via Tor, limite efficacité mais bloque spam basique)
- Proof-of-Work client-side (calculer hash avant submit)
- Honeypot fields (champs cachés pour bots)

#### 3.2.2 Segmentation Auto-Déclarée

**Après inscription, si role = "vendor":**

```
┌──────────────────────────────────────────────────────────┐
│  Welcome, vendor! Help us personalize your experience:  │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  How many products do you plan to sell?                 │
│                                                          │
│  ○ 1-10 products (Casual Seller)                        │
│     → Wizard simple, 1 produit à la fois               │
│                                                          │
│  ○ 11-100 products (Professional)                       │
│     → Bulk upload CSV                                   │
│                                                          │
│  ○ 100+ products (Power Seller)                         │
│     → API REST documentation                            │
│                                                          │
│  [Continue →]                                            │
└──────────────────────────────────────────────────────────┘
```

**Backend:**
```rust
// server/src/models/user.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VendorType {
    Casual,        // 1-10 produits
    Professional,  // 11-100 produits
    PowerSeller,   // 100+ produits
}

pub struct User {
    pub id: String,
    pub username: String,
    pub role: UserRole,
    pub vendor_type: Option<VendorType>,  // Nouveau champ
    pub created_at: NaiveDateTime,
    // ...
}
```

**Impact:** Router automatiquement vers différents flows de création listing

### 3.3 Étape 2: Wallet Setup (Remplace KYC/KYB)

#### 3.3.1 Le "KYC" du Darknet

Dans une marketplace traditionnelle, le KYC/KYB sert à:
1. **Établir l'identité** (qui es-tu?)
2. **Établir la confiance** (es-tu légitime?)
3. **Permettre les paiements** (lien compte bancaire)

Dans Nexus, le **Wallet Setup** remplit ces fonctions sans identité:
1. **Établir la pseudonymité** (wallet address = identité cryptographique)
2. **Établir la confiance** (multisig = protection cryptographique, pas besoin de "trust")
3. **Permettre les paiements** (wallet opérationnel)

#### 3.3.2 Wizard Wallet Setup (5 Étapes)

**Route:** `/vendor/wallet-setup`

**Template existant:** [templates/docs/wallet-setup.html](../../templates/docs/wallet-setup.html) (à transformer en wizard interactif)

**Flux proposé:**

```
┌─────────────────────────────────────────────────────────────┐
│ Step 1/5: Why Multisig? (Education - 30 sec)                │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  🔒 Your funds are protected by cryptography, not trust     │
│                                                              │
│  In a 2-of-3 multisig escrow:                               │
│  • You control 1 key                                        │
│  • Buyer controls 1 key                                     │
│  • Arbiter controls 1 key                                   │
│                                                              │
│  ✅ Release funds: 2 of 3 signatures required               │
│  ✅ No single party can steal                               │
│  ✅ Dispute resolution built-in                             │
│                                                              │
│  [Watch 30s video] [Skip, I understand →]                   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Step 2/5: Generate Your Wallet                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  [Generate Wallet] ← Click to create                        │
│                                                              │
│  ⚠️ CRITICAL: Save your seed phrase                         │
│                                                              │
│  [ word1 ] [ word2 ] [ word3 ] ... [ word25 ]              │
│                                                              │
│  ☐ I have written down my seed phrase                       │
│                                                              │
│  [Continue →]                                                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Step 3/5: Setup Multisig (Automated)                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ⏳ Preparing multisig wallet...                            │
│                                                              │
│  [████████████░░░░] 75%                                      │
│                                                              │
│  This takes ~2 minutes. Do not close this page.             │
│                                                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Step 4/5: Verification                                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ✅ Wallet created successfully                             │
│  ✅ Multisig enabled                                         │
│  ✅ Ready to receive payments                               │
│                                                              │
│  Your wallet address:                                        │
│  47vZ... [Copy] [Show QR]                                   │
│                                                              │
│  [Continue →]                                                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Step 5/5: Optional - Vendor Bond (Boost Reputation)         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  New vendors start with 0 reputation.                       │
│  Deposit a bond to signal trustworthiness:                  │
│                                                              │
│  Bond Tiers:                                                │
│  ○ No bond (0 XMR) - Standard                               │
│  ○ Bronze (0.5 XMR) - +1 trust badge                        │
│  ○ Silver (1 XMR) - +2 trust badges                         │
│  ○ Gold (2 XMR) - +3 trust badges + Priority arbitrage     │
│                                                              │
│  Bond is refundable after 90 days or 10 successful trades.  │
│                                                              │
│  [Deposit Bond] [Skip, maybe later]                         │
└─────────────────────────────────────────────────────────────┘
```

#### 3.3.3 Implémentation Backend

**Appel à wallet/src/client.rs:**

```rust
// server/src/handlers/vendor.rs
use monero_marketplace_wallet::MoneroClient;

pub async fn setup_wallet_step3(
    user_id: String,
    monero_client: Arc<MoneroClient>,
) -> Result<WalletSetupResponse, Error> {
    // 1. Créer wallet pour ce user
    let wallet_name = format!("vendor_{}", user_id);

    // 2. prepare_multisig() - Étape 1/6 du flow multisig
    let multisig_info = monero_client
        .prepare_multisig()
        .await
        .context("Failed to prepare multisig")?;

    // 3. Sauvegarder multisig_info en DB (pour étapes futures)
    diesel::update(users::table.find(&user_id))
        .set((
            users::wallet_name.eq(&wallet_name),
            users::multisig_info.eq(&multisig_info),
            users::wallet_setup_completed.eq(true),
        ))
        .execute(&conn)?;

    // 4. Retourner confirmation
    Ok(WalletSetupResponse {
        success: true,
        wallet_address: "47vZ...", // Obtenir via get_address()
        next_step: "create_listing",
    })
}
```

**KPI:** `Time-to-Multisig-Ready` = Date(wallet_setup_completed=true) - Date(inscription)

**Objectif:** Médiane < 10 minutes (vs. KYC traditionnel = 2-5 jours)

### 3.4 Étape 3: Time-to-Listing (Activation Critique)

#### 3.4.1 Le Goulot d'Étranglement Principal

**Constat:** Un vendeur avec wallet configuré mais 0 listing n'est pas "activé". Il ne contribue pas à l'offre.

**KPI critique:** `Time-to-First-Listing` (TTL)

**Formule:**
```
TTL = Date(première listing active) - Date(inscription)
```

**Objectif:**
- 🎯 Casual Seller: < 30 min
- 🎯 Professional: < 2 heures (temps d'upload CSV)
- 🎯 Power Seller: < 1 jour (temps d'intégration API)

#### 3.4.2 Méthodes d'Intégration (Segmentées)

**1. Wizard Manuel (Casual Seller)**

**Template:** [templates/listings/create.html](../../templates/listings/create.html)

**Optimisations à implémenter:**

```html
<!-- Version optimisée - 5 champs essentiels -->
<form action="/listings/create" method="POST" enctype="multipart/form-data">
    <!-- Step 1: Essentials (required) -->
    <input type="text" name="title" placeholder="Product title" required>

    <textarea name="description" placeholder="Description" required></textarea>

    <input type="number" name="price_xmr"
           step="0.000000000001"
           placeholder="Price in XMR"
           required>

    <input type="file" name="images[]"
           accept="image/*"
           multiple
           max="5"
           required>

    <select name="category" required>
        <option>Electronics</option>
        <option>Books</option>
        <option>Services</option>
        <!-- ... -->
    </select>

    <!-- Step 2: Optional (collapsible) -->
    <details>
        <summary>Additional info (optional)</summary>
        <input type="text" name="shipping_countries" placeholder="Ships to...">
        <input type="number" name="stock_quantity" placeholder="Stock">
        <textarea name="terms" placeholder="Terms & conditions"></textarea>
    </details>

    <button type="submit">Publish Listing</button>
    <button type="button" onclick="saveDraft()">Save Draft</button>
</form>
```

**Nouvelles fonctionnalités:**
- ✅ **Draft system**: Sauvegarder brouillon (table `listing_drafts`)
- ✅ **Live preview**: HTMX pour preview en temps réel
- ✅ **Image upload via IPFS**: Upload vers node IPFS via Tor

**2. Bulk Upload CSV (Professional)**

**Route:** `POST /listings/bulk-import`

**Format CSV:**
```csv
title,description,price_xmr,category,image_urls,stock,shipping
"Product 1","Description 1",0.05,"Electronics","http://ipfs/img1.jpg",10,"Worldwide"
"Product 2","Description 2",0.10,"Books","http://ipfs/img2.jpg",5,"EU only"
```

**Handler:**
```rust
// server/src/handlers/listings.rs
pub async fn bulk_import(
    user_id: String,
    csv_file: Multipart,
) -> Result<BulkImportResponse, Error> {
    // 1. Parse CSV
    let records = csv::Reader::from_reader(csv_file)
        .deserialize()
        .collect::<Result<Vec<NewListing>, _>>()?;

    // 2. Validate (max 100 listings par batch)
    if records.len() > 100 {
        return Err(Error::BadRequest("Max 100 listings per batch".into()));
    }

    // 3. Insert en transaction
    let inserted = diesel::insert_into(listings::table)
        .values(&records)
        .execute(&conn)?;

    Ok(BulkImportResponse {
        success: true,
        imported: inserted,
        errors: vec![],
    })
}
```

**3. API REST (Power Seller)**

**Endpoint:** `POST /api/v1/listings`

**Authentication:** JWT token (généré dans `/settings/api-keys`)

**Documentation:** Créer `/docs/api.html` avec exemples curl:

```bash
curl -X POST https://nexus.onion/api/v1/listings \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Product Title",
    "description": "Description",
    "price_xmr": "0.05",
    "category": "Electronics",
    "images_ipfs_cids": ["QmHash1", "QmHash2"],
    "stock_quantity": 10
  }'
```

**4. IPFS Integration (Toutes Catégories)**

**Problème:** Stocker images on-chain = impossible. Stocker sur serveur centralisé = point de défaillance.

**Solution:** IPFS (InterPlanetary File System)

**Flow:**
```
[User uploads image]
    ↓
[Server uploads to IPFS node via Tor]
    ↓
[IPFS returns CID: QmHash...]
    ↓
[Store CID in DB, not the image]
    ↓
[Render: <img src="/ipfs/QmHash...">]
```

**Backend:**
```rust
// server/src/services/ipfs.rs
use reqwest::Proxy;

pub struct IpfsClient {
    client: reqwest::Client,
    gateway_url: String,  // "http://127.0.0.1:5001" (local node)
}

impl IpfsClient {
    pub async fn upload_image(&self, image_bytes: Vec<u8>) -> Result<String, Error> {
        // Upload via Tor
        let response = self.client
            .post(&format!("{}/api/v0/add", self.gateway_url))
            .body(image_bytes)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        let cid = json["Hash"].as_str()
            .ok_or(Error::IpfsError("No hash returned".into()))?;

        Ok(cid.to_string())
    }
}
```

**Priority:** P1 (Beta) - Essential pour scalabilité

### 3.5 Étape 4: Activation & Services à Valeur

#### 3.5.1 Dashboard Vendeur

**Route:** `/vendor/dashboard`

**Métriques affichées (anonymisées):**

```
┌─────────────────────────────────────────────────────────────┐
│                    VENDOR DASHBOARD                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Active Listings: 12                                        │
│  Total Views (7d): 347                                      │
│  Click-Through Rate: 8.2%                                   │
│  Orders (30d): 23                                           │
│  Revenue (30d): 2.45 XMR                                    │
│                                                              │
│  ⭐ Reputation: Trusted Vendor (15 positive reviews)        │
│                                                              │
│  [Create New Listing] [Manage Orders] [Boost Listings]     │
└─────────────────────────────────────────────────────────────┘
```

**Implémentation:**
```rust
// server/src/handlers/vendor.rs
pub async fn dashboard(user_id: String) -> Result<VendorDashboard, Error> {
    let metrics = VendorMetrics {
        active_listings: count_active_listings(&user_id)?,
        total_views_7d: sum_listing_views(&user_id, 7)?,
        ctr: calculate_ctr(&user_id)?,
        orders_30d: count_orders(&user_id, 30)?,
        revenue_30d: sum_order_totals(&user_id, 30)?,
        reputation_score: calculate_reputation(&user_id)?,
    };

    Ok(VendorDashboard { metrics })
}
```

**❌ Ne PAS afficher:**
- Données individuelles des acheteurs (noms, adresses)
- IPs, user agents
- Données de navigation détaillées

**✅ Afficher uniquement:**
- Métriques agrégées
- Statistiques anonymisées
- Données du vendeur lui-même

#### 3.5.2 Services Différenciants

**1. Listing Boost (Featured Placement)**

**Concept:** Vendeur paie XMR pour placer listing en featured position (homepage, catégorie)

**Pricing:**
```
┌──────────────────────────────────────────────────────┐
│  Boost Your Listing Visibility                      │
├──────────────────────────────────────────────────────┤
│                                                      │
│  ○ Homepage Featured (24h) - 0.01 XMR               │
│     → Top 3 slots, 10x more views                   │
│                                                      │
│  ○ Category Featured (7d) - 0.005 XMR               │
│     → Category page top slot                        │
│                                                      │
│  ○ Search Priority (30d) - 0.02 XMR                 │
│     → Appear higher in search results               │
│                                                      │
│  [Boost Now →]                                       │
└──────────────────────────────────────────────────────┘
```

**Implémentation:**
```rust
// server/src/models/listing.rs
pub struct Listing {
    // ... champs existants
    pub boosted_until: Option<NaiveDateTime>,
    pub boost_type: Option<BoostType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoostType {
    HomepageFeatured,
    CategoryFeatured,
    SearchPriority,
}
```

**Query handler:**
```rust
// Listings homepage: prioriser les boosted
let listings = listings::table
    .order_by(
        sql::<Bool>("CASE WHEN boosted_until > NOW() THEN 0 ELSE 1 END")
            .then(listings::created_at.desc())
    )
    .limit(50)
    .load::<Listing>(&conn)?;
```

**2. Priority Arbitrage (Pour Vendors avec Bond)**

**Concept:** En cas de dispute, vendors qui ont déposé un bond ont priorité dans la file d'arbitrage.

**File d'attente:**
```
┌──────────────────────────────────────────────────────┐
│         ARBITRAGE QUEUE (Admin View)                 │
├──────────────────────────────────────────────────────┤
│                                                      │
│  🥇 Priority (Bond deposited):                      │
│    • Dispute #1234 - Gold Vendor (2 XMR bond)       │
│    • Dispute #1256 - Silver Vendor (1 XMR bond)     │
│                                                      │
│  🥈 Standard:                                        │
│    • Dispute #1212 - No bond                        │
│    • Dispute #1223 - No bond                        │
│                                                      │
└──────────────────────────────────────────────────────┘
```

**3. Vendor Analytics Report (Weekly)**

**Notification in-app chaque lundi:**

```
📊 Your Weekly Report is Ready

Highlights:
• Your listings received 234 views (+12% vs. last week)
• 5 new orders (Revenue: 0.45 XMR)
• Top performing product: "Product X" (87 views)

[View Full Report →]
```

---

## 4. Partie 2: Onboarding Acheteur (Speed & Privacy)

### 4.1 Philosophie: "Ghost Mode" First

**Principe fondamental:** L'acheteur doit pouvoir explorer **toute la marketplace** sans révéler **aucune information**.

```
┌─────────────────────────────────────────────────────────────┐
│                   GHOST MODE (No Account)                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ✅ Browse all listings                                      │
│  ✅ Search by keyword/category                               │
│  ✅ Filter by price, seller reputation                       │
│  ✅ View product details (full description, images)          │
│  ✅ Add to cart (session-based)                              │
│                                                              │
│  ❌ Checkout (BLOCKER)                                       │
│     → "Create account to complete purchase (30 sec)"        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Implémentation Technique

#### 4.2.1 Routes Publiques (Sans Auth)

**Audit actuel:**
```bash
# Vérifier quelles routes nécessitent auth
grep -r "require_auth\|auth_middleware" server/src/handlers/
```

**Routes qui DOIVENT être publiques:**
- `GET /` (homepage)
- `GET /listings` (browse all)
- `GET /listings/:id` (product detail)
- `GET /search?q=...` (search)
- `GET /categories/:slug` (category pages)

**Routes qui DOIVENT nécessiter auth:**
- `POST /cart/checkout`
- `GET /orders`
- `GET /orders/:id`
- `POST /listings/create`

**Middleware configuration:**
```rust
// server/src/main.rs
App::new()
    // Public routes (no middleware)
    .service(
        web::scope("")
            .route("/", web::get().to(handlers::index))
            .route("/listings", web::get().to(handlers::listings::index))
            .route("/listings/{id}", web::get().to(handlers::listings::show))
            .route("/search", web::get().to(handlers::search))
    )
    // Protected routes (require auth)
    .service(
        web::scope("")
            .wrap(AuthMiddleware)
            .route("/cart/checkout", web::post().to(handlers::cart::checkout))
            .route("/orders", web::get().to(handlers::orders::index))
            .route("/listings/create", web::get().to(handlers::listings::create))
    )
```

#### 4.2.2 Session Cart (Crypté)

**Problème:** Comment gérer un panier sans compte?

**Solution:** Cookie de session crypté (pas de base de données)

```rust
// server/src/models/cart.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SessionCart {
    pub items: Vec<CartItem>,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct CartItem {
    pub listing_id: String,
    pub quantity: u32,
    pub price_xmr: String,
}

// Handler: Add to cart
pub async fn add_to_cart(
    session: Session,  // actix-session
    listing_id: String,
) -> Result<HttpResponse, Error> {
    // 1. Récupérer cart existant ou créer nouveau
    let mut cart = session
        .get::<SessionCart>("cart")?
        .unwrap_or_default();

    // 2. Ajouter item
    cart.items.push(CartItem {
        listing_id,
        quantity: 1,
        price_xmr: "0.05".to_string(),  // Fetch from DB
    });

    // 3. Sauvegarder dans session (cookie crypté)
    session.insert("cart", cart)?;

    Ok(HttpResponse::Ok().json(json!({"success": true})))
}
```

**Configuration cookie:**
```rust
// server/src/main.rs
use actix_session::{SessionMiddleware, config::PersistentSession};
use actix_web::cookie::{Key, SameSite};

let secret_key = Key::from(&config.session_secret);  // 32 bytes from .env

SessionMiddleware::builder(
    CookieSessionStore::default(),
    secret_key
)
.cookie_name("nexus_session")
.cookie_secure(true)  // HTTPS only (via Tor)
.cookie_same_site(SameSite::Strict)
.cookie_http_only(true)  // Pas accessible en JS (XSS protection)
.session_lifecycle(
    PersistentSession::default()
        .session_ttl(time::Duration::days(7))  // Expire après 7 jours
)
.build()
```

**Privacy:** Session cookie ne contient AUCUNE info personnelle, juste le cart. IP non loggée (Tor).

### 4.3 Le "Moment Aha!" Adapté

#### 4.3.1 Définition

**Playbook traditionnel:** "J'ai trouvé le produit que je cherchais"

**Nexus:** "J'ai trouvé le produit que je cherchais **ET mes fonds sont protégés cryptographiquement**"

**Pourquoi c'est différent:**
- Sur Amazon: confiance = marque Amazon
- Sur eBay: confiance = PayPal buyer protection
- Sur Nexus: confiance = **comprendre le multisig escrow**

**Challenge:** Éduquer sans friction

#### 4.3.2 Implémentation: Trust Badge

**Sur chaque page produit, afficher bloc explicatif:**

```html
<!-- templates/listings/show.html -->
<div class="trust-badge">
    <div class="badge-icon">🔒</div>
    <div class="badge-content">
        <h4>Protected by 2-of-3 Multisig Escrow</h4>
        <p>Your XMR is cryptographically locked until:</p>
        <ul>
            <li>✓ You confirm receipt (release funds)</li>
            <li>✓ Seller releases after delivery</li>
            <li>✓ Arbiter resolves dispute (if needed)</li>
        </ul>
        <a href="/docs/escrow-explained" class="learn-more">
            Learn how it works →
        </a>
    </div>
</div>
```

**Styling (glassmorphism):**
```css
.trust-badge {
    background: rgba(16, 185, 129, 0.1);  /* Vert transparent */
    border: 1px solid rgba(16, 185, 129, 0.3);
    border-radius: 12px;
    padding: 20px;
    margin: 20px 0;
    backdrop-filter: blur(10px);
}
```

**Variante interactive (HTMX tooltip):**
```html
<span class="tooltip-trigger"
      hx-get="/api/escrow-explainer"
      hx-trigger="mouseenter once"
      hx-target="#tooltip-container">
    🔒 Escrow Protected
</span>
```

#### 4.3.3 Page Dédiée: `/docs/escrow-explained`

**Contenu:**
1. **Vidéo 60 sec** (animation): "How Multisig Escrow Works"
2. **Diagram interactif**: Flow de transaction avec states
3. **FAQ**: "What if seller doesn't deliver?", "Who is the arbiter?", etc.
4. **Real examples**: "99.2% of transactions complete without dispute"

**Template:** [templates/docs/wallet-setup.html](../../templates/docs/wallet-setup.html) (à adapter)

### 4.4 Progressive Onboarding (3 Écrans Max)

#### 4.4.1 Trigger: Au Checkout

**User clique "Checkout" → Redirect vers `/auth/register` avec context:**

```
┌─────────────────────────────────────────────────────────────┐
│ You have 3 items in your cart (Total: 0.45 XMR)             │
│                                                              │
│ Create your account to complete purchase                     │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Step 1/3: Create Account                                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Choose Username:                                            │
│  [________________]                                          │
│                                                              │
│  Password:                                                   │
│  [________________]                                          │
│                                                              │
│  ☐ I agree to Terms of Service                              │
│                                                              │
│  [Create Account & Continue to Payment →]                   │
│                                                              │
│  Estimated time: 30 seconds                                  │
└─────────────────────────────────────────────────────────────┘
```

**Après création compte:**

```
┌─────────────────────────────────────────────────────────────┐
│ Step 2/3: Setup Payment Wallet (Optional)                   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Option A: Use temporary wallet (Recommended for first buy) │
│  ○ We'll create a wallet for you                            │
│  ○ You can export it after purchase                         │
│  [Use Temporary Wallet →]                                    │
│                                                              │
│  Option B: Connect my existing Monero wallet                │
│  ○ For advanced users                                       │
│  [Connect Existing Wallet →]                                 │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Enfin, checkout:**

```
┌─────────────────────────────────────────────────────────────┐
│ Step 3/3: Complete Purchase                                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Order Summary:                                              │
│  • Product 1 (0.15 XMR)                                      │
│  • Product 2 (0.30 XMR)                                      │
│                                                              │
│  Total: 0.45 XMR                                             │
│                                                              │
│  Send exactly 0.45 XMR to:                                   │
│  [Wallet Address]  [Copy] [Show QR]                         │
│                                                              │
│  ⏳ Waiting for payment confirmation...                      │
│     (This takes ~20 minutes on Monero network)              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**KPI:** `Time-to-First-Purchase` = Date(first order created) - Date(account creation)

**Objectif:** < 5 minutes (médiane)

### 4.5 Engagement Post-Achat

#### 4.5.1 Notifications In-App (Remplace Email)

**Problème:** Email marketing = lien identitaire (email ↔ pseudonyme)

**Solution:** Système de notifications in-app uniquement

**Table:**
```sql
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    type VARCHAR(50) NOT NULL,  -- 'order_update', 'message', 'review_request'
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    link TEXT,  -- URL interne (e.g., "/orders/123")
    read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_unread
    ON notifications(user_id, read)
    WHERE read = FALSE;
```

**Triggers backend:**
```rust
// server/src/services/notifications.rs
pub async fn notify_order_shipped(
    user_id: &str,
    order_id: &str,
) -> Result<(), Error> {
    diesel::insert_into(notifications::table)
        .values(&NewNotification {
            user_id: user_id.to_string(),
            type_: "order_update".to_string(),
            title: "Your order has shipped!".to_string(),
            message: format!("Order #{} is on its way.", order_id),
            link: Some(format!("/orders/{}", order_id)),
        })
        .execute(&conn)?;

    Ok(())
}
```

**Frontend (HTMX polling):**
```html
<!-- templates/partials/header.html -->
<div class="notifications-bell"
     hx-get="/api/notifications/count"
     hx-trigger="every 30s"
     hx-swap="innerHTML">
    🔔 <span class="badge">0</span>
</div>
```

**Dropdown:**
```html
<div class="notifications-dropdown"
     hx-get="/api/notifications"
     hx-trigger="click"
     hx-target="#notifications-list">

    <div id="notifications-list">
        <!-- Populated by HTMX -->
    </div>
</div>
```

#### 4.5.2 Événements Déclencheurs (Trigger-Based)

**Contrairement au playbook (email drip = basé sur temps), nous utilisons des triggers comportementaux:**

| Event | Trigger Notification |
|-------|---------------------|
| Order created | "Your order #X is confirmed. Track it here." |
| Order shipped | "Your order #X has shipped." |
| Order delivered (auto) | "Did you receive your order? [Confirm Receipt]" |
| Dispute opened | "Dispute #X opened. Arbiter will review within 48h." |
| Seller sent message | "You have a new message from @seller_name" |
| Product back in stock | "Product X you favorited is back in stock!" |

**Implémentation:**
```rust
// server/src/handlers/orders.rs
pub async fn update_order_status(
    order_id: String,
    new_status: OrderStatus,
) -> Result<HttpResponse, Error> {
    // 1. Update order
    diesel::update(orders::table.find(&order_id))
        .set(orders::status.eq(&new_status))
        .execute(&conn)?;

    // 2. Trigger notification based on status
    match new_status {
        OrderStatus::Shipped => {
            let order = orders::table.find(&order_id).first::<Order>(&conn)?;
            notifications::notify_order_shipped(&order.buyer_id, &order_id).await?;
        },
        OrderStatus::Delivered => {
            let order = orders::table.find(&order_id).first::<Order>(&conn)?;
            notifications::notify_confirm_receipt(&order.buyer_id, &order_id).await?;
        },
        // ... autres statuts
    }

    Ok(HttpResponse::Ok().finish())
}
```

---

## 5. Partie 3: Stack Technique Adaptée

### 5.1 Couche UX (Guidage & Tours)

#### 5.1.1 Choix: Shepherd.js (Self-Hosted)

**Pourquoi PAS UserGuiding/Appcues (SaaS du playbook):**
- ❌ Tracking externe (compromet privacy)
- ❌ Dépendance à un tiers (single point of failure)
- ❌ Coût récurrent ($200-500/mois)

**Pourquoi Shepherd.js:**
- ✅ Open-source (MIT license)
- ✅ Self-hosted (aucun appel externe)
- ✅ Léger (15KB gzipped)
- ✅ Framework-agnostic (vanilla JS)
- ✅ Accessible (keyboard navigation, ARIA)

**Installation:**
```bash
# Télécharger en local (pas de CDN)
cd static/vendor/shepherd
curl -L https://github.com/shepherd-pro/shepherd/releases/download/v11.2.0/shepherd.js \
     -o shepherd.min.js

curl -L https://github.com/shepherd-pro/shepherd/releases/download/v11.2.0/shepherd.css \
     -o shepherd.min.css
```

**Usage:**
```html
<!-- templates/base-nexus.html -->
<script src="/static/vendor/shepherd/shepherd.min.js"></script>
<link rel="stylesheet" href="/static/vendor/shepherd/shepherd.min.css">

<script>
// Tour d'accueil homepage
const tour = new Shepherd.Tour({
    useModalOverlay: true,
    defaultStepOptions: {
        classes: 'nexus-tour',
        scrollTo: true,
        cancelIcon: {
            enabled: true
        }
    }
});

tour.addStep({
    id: 'welcome',
    text: 'Welcome to Nexus, the anonymous marketplace powered by Monero and Tor.',
    buttons: [
        {
            text: 'Next',
            action: tour.next
        }
    ]
});

tour.addStep({
    id: 'escrow',
    text: 'All transactions are protected by 2-of-3 multisig escrow. Your funds are safe.',
    attachTo: {
        element: '.trust-badge',
        on: 'bottom'
    },
    buttons: [
        {
            text: 'Back',
            action: tour.back
        },
        {
            text: 'Next',
            action: tour.next
        }
    ]
});

tour.addStep({
    id: 'browse',
    text: 'Start browsing anonymously. No account required.',
    attachTo: {
        element: '.header-nav',
        on: 'bottom'
    },
    buttons: [
        {
            text: 'Got it!',
            action: tour.complete
        }
    ]
});

// Start tour si first visit
if (!localStorage.getItem('tour_completed')) {
    tour.start();
    tour.on('complete', () => {
        localStorage.setItem('tour_completed', 'true');
    });
}
</script>
```

**Tours à créer:**
1. **Homepage tour** (3 steps): Welcome → Escrow → Browse
2. **Vendor onboarding tour** (5 steps): Dashboard → Create listing → Wallet setup → Boost → Analytics
3. **First purchase tour** (4 steps): Add to cart → Checkout → Payment → Track order

#### 5.1.2 Progressive Disclosure Pattern

**Implémentation avec `<details>` natif HTML:**

```html
<!-- Formulaire création listing avec progressive disclosure -->
<form action="/listings/create" method="POST">
    <!-- Required fields (toujours visibles) -->
    <fieldset>
        <legend>Essential Information</legend>
        <input type="text" name="title" required>
        <textarea name="description" required></textarea>
        <input type="number" name="price_xmr" required>
    </fieldset>

    <!-- Optional fields (collapsible) -->
    <details>
        <summary>⚙️ Additional Options (optional)</summary>
        <fieldset>
            <input type="text" name="shipping_countries">
            <input type="number" name="stock_quantity">
            <textarea name="terms"></textarea>
        </fieldset>
    </details>

    <!-- Advanced fields (collapsible) -->
    <details>
        <summary>🔧 Advanced Settings (for power users)</summary>
        <fieldset>
            <input type="text" name="custom_escrow_terms">
            <input type="number" name="auto_finalize_days">
        </fieldset>
    </details>

    <button type="submit">Create Listing</button>
</form>
```

**Styling:**
```css
details {
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 12px;
    margin: 16px 0;
}

summary {
    cursor: pointer;
    font-weight: 600;
    user-select: none;
}

summary:hover {
    color: var(--primary-color);
}

details[open] summary {
    margin-bottom: 12px;
}
```

### 5.2 Couche Paiement & Conformité

**Stack actuelle (déjà implémentée):**

```
┌─────────────────────────────────────────────────────────────┐
│                  PAYMENT & ESCROW STACK                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ✅ monero-wallet-rpc (wallet/src/rpc.rs)                   │
│      • Low-level RPC client                                 │
│      • Rate limiting (semaphore)                            │
│      • Retry logic                                          │
│                                                              │
│  ✅ MultisigManager (wallet/src/multisig.rs)                │
│      • prepare_multisig()                                   │
│      • make_multisig() [TODO]                               │
│      • export/import_multisig_info() [TODO]                 │
│                                                              │
│  ✅ MoneroClient (wallet/src/client.rs)                     │
│      • High-level operations                                │
│      • Error handling                                       │
│                                                              │
│  ❌ Escrow State Machine [TODO]                             │
│      • server/src/services/escrow.rs                        │
│      • State transitions                                    │
│      • Dispute resolution                                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Pas d'adaptation nécessaire** - notre stack remplace déjà Stripe/Lemonway.

**Next steps (Phase 3 du projet):**
1. Compléter multisig flow (make_multisig, export/import)
2. Implémenter escrow state machine
3. Tester E2E avec testnet

### 5.3 Couche Engagement (Notifications)

**Architecture:**

```
┌─────────────────────────────────────────────────────────────┐
│                   ENGAGEMENT ARCHITECTURE                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Backend Events                                              │
│  ============                                                │
│  • Order created/updated                                     │
│  • Message received                                          │
│  • Dispute opened                                            │
│  • Review requested                                          │
│           ↓                                                  │
│  Notification Service                                        │
│  ====================                                        │
│  • Create notification record                                │
│  • Store in PostgreSQL                                       │
│  • No external calls                                         │
│           ↓                                                  │
│  Frontend Polling (HTMX)                                     │
│  =======================                                     │
│  • Poll /api/notifications/count every 30s                   │
│  • Fetch /api/notifications on click                         │
│  • Mark as read                                              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Implémentation complète:**

**1. Table DB:**
```sql
-- migrations/YYYY-MM-DD-create-notifications/up.sql
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    type VARCHAR(50) NOT NULL,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    link TEXT,
    read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_notifications_user_unread
    ON notifications(user_id)
    WHERE read = FALSE;

CREATE INDEX idx_notifications_created
    ON notifications(created_at DESC);
```

**2. Model:**
```rust
// server/src/models/notification.rs
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Queryable, Identifiable, Serialize)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub type_: String,
    pub title: String,
    pub message: String,
    pub link: Option<String>,
    pub read: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = notifications)]
pub struct NewNotification {
    pub user_id: String,
    pub type_: String,
    pub title: String,
    pub message: String,
    pub link: Option<String>,
}
```

**3. Service:**
```rust
// server/src/services/notifications.rs
use diesel::prelude::*;

pub struct NotificationService;

impl NotificationService {
    pub async fn create(
        conn: &PgConnection,
        user_id: &str,
        type_: &str,
        title: &str,
        message: &str,
        link: Option<String>,
    ) -> Result<Notification, Error> {
        diesel::insert_into(notifications::table)
            .values(&NewNotification {
                user_id: user_id.to_string(),
                type_: type_.to_string(),
                title: title.to_string(),
                message: message.to_string(),
                link,
            })
            .get_result(conn)
            .map_err(Into::into)
    }

    pub async fn get_unread_count(
        conn: &PgConnection,
        user_id: &str,
    ) -> Result<i64, Error> {
        notifications::table
            .filter(notifications::user_id.eq(user_id))
            .filter(notifications::read.eq(false))
            .count()
            .get_result(conn)
            .map_err(Into::into)
    }

    pub async fn get_recent(
        conn: &PgConnection,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<Notification>, Error> {
        notifications::table
            .filter(notifications::user_id.eq(user_id))
            .order_by(notifications::created_at.desc())
            .limit(limit)
            .load(conn)
            .map_err(Into::into)
    }

    pub async fn mark_as_read(
        conn: &PgConnection,
        notification_id: &str,
        user_id: &str,
    ) -> Result<(), Error> {
        diesel::update(
            notifications::table
                .filter(notifications::id.eq(notification_id))
                .filter(notifications::user_id.eq(user_id))
        )
        .set(notifications::read.eq(true))
        .execute(conn)?;

        Ok(())
    }
}
```

**4. Handlers:**
```rust
// server/src/handlers/notifications.rs
use actix_web::{web, HttpResponse};

pub async fn get_unread_count(
    user_id: web::ReqData<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get()?;
    let count = NotificationService::get_unread_count(&conn, &user_id).await?;

    Ok(HttpResponse::Ok().json(json!({
        "count": count
    })))
}

pub async fn get_notifications(
    user_id: web::ReqData<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get()?;
    let notifications = NotificationService::get_recent(&conn, &user_id, 20).await?;

    Ok(HttpResponse::Ok().json(notifications))
}

pub async fn mark_read(
    user_id: web::ReqData<String>,
    notification_id: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get()?;
    NotificationService::mark_as_read(&conn, &notification_id, &user_id).await?;

    Ok(HttpResponse::Ok().finish())
}
```

**5. Frontend (HTMX):**
```html
<!-- templates/partials/header.html -->
<div class="notifications-widget">
    <!-- Badge avec count -->
    <button class="notifications-bell"
            hx-get="/api/notifications/unread-count"
            hx-trigger="load, every 30s"
            hx-swap="innerHTML"
            onclick="toggleNotifications()">
        🔔 <span id="notification-count" class="badge">0</span>
    </button>

    <!-- Dropdown (caché par défaut) -->
    <div id="notifications-dropdown" class="dropdown" style="display:none;">
        <div class="dropdown-header">
            <h4>Notifications</h4>
            <button onclick="markAllAsRead()">Mark all as read</button>
        </div>

        <div class="notifications-list"
             hx-get="/api/notifications"
             hx-trigger="load"
             hx-swap="innerHTML">
            <!-- Populated by HTMX -->
        </div>
    </div>
</div>

<script>
function toggleNotifications() {
    const dropdown = document.getElementById('notifications-dropdown');
    dropdown.style.display = dropdown.style.display === 'none' ? 'block' : 'none';
}

function markAllAsRead() {
    // Implementation
}
</script>
```

### 5.4 Couche Analytics (Privacy-Preserving)

#### 5.4.1 Option 1: Analytics DB Interne

**Table:**
```sql
CREATE TABLE analytics_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,  -- Nullable pour événements anonymes
    event_type VARCHAR(100) NOT NULL,
    properties JSONB,  -- Flexible metadata
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_analytics_type_date
    ON analytics_events(event_type, created_at DESC);

-- Auto-hash user_id après 30 jours (GDPR-style)
CREATE OR REPLACE FUNCTION hash_old_analytics() RETURNS void AS $$
BEGIN
    UPDATE analytics_events
    SET user_id = NULL
    WHERE created_at < NOW() - INTERVAL '30 days'
      AND user_id IS NOT NULL;
END;
$$ LANGUAGE plpgsql;

-- Run daily via cron
-- 0 2 * * * psql -c "SELECT hash_old_analytics();"
```

**Events à tracker:**
```rust
// server/src/services/analytics.rs
pub enum AnalyticsEvent {
    // User events
    UserRegistered { role: String },
    UserLogin,

    // Buyer events
    ListingViewed { listing_id: String, category: String },
    AddedToCart { listing_id: String, price: String },
    CheckoutStarted { cart_value: String },
    OrderCreated { order_id: String, total: String },

    // Vendor events
    WalletSetupStarted,
    WalletSetupCompleted { time_taken_seconds: i64 },
    ListingCreated { category: String },
    ListingBoosted { boost_type: String },

    // Escrow events
    EscrowFunded,
    EscrowReleased,
    DisputeOpened,
}

pub async fn track_event(
    conn: &PgConnection,
    user_id: Option<&str>,
    event: AnalyticsEvent,
) -> Result<(), Error> {
    let (event_type, properties) = match event {
        AnalyticsEvent::UserRegistered { role } => {
            ("user_registered", json!({ "role": role }))
        },
        AnalyticsEvent::ListingViewed { listing_id, category } => {
            ("listing_viewed", json!({
                "listing_id": listing_id,
                "category": category
            }))
        },
        // ... autres events
    };

    diesel::insert_into(analytics_events::table)
        .values(&NewAnalyticsEvent {
            user_id: user_id.map(|s| s.to_string()),
            event_type: event_type.to_string(),
            properties: Some(properties),
        })
        .execute(conn)?;

    Ok(())
}
```

**Queries pour KPIs:**
```rust
// server/src/services/analytics.rs
pub async fn calculate_ttfv_buyer(
    conn: &PgConnection,
) -> Result<i64, Error> {
    // Time-to-First-Value: time between registration and first add-to-cart
    let query = r#"
        SELECT
            EXTRACT(EPOCH FROM (
                MIN(cart.created_at) - reg.created_at
            ))::BIGINT AS ttfv_seconds
        FROM
            (SELECT user_id, created_at
             FROM analytics_events
             WHERE event_type = 'user_registered') AS reg
        JOIN
            (SELECT user_id, created_at
             FROM analytics_events
             WHERE event_type = 'added_to_cart') AS cart
        ON reg.user_id = cart.user_id
        GROUP BY reg.user_id
    "#;

    // Median TTFV
    diesel::sql_query(query)
        .load::<TtfvResult>(conn)?
        .into_iter()
        .map(|r| r.ttfv_seconds)
        .collect::<Vec<_>>()
        .median()
}
```

#### 5.4.2 Option 2: Plausible Analytics (Self-Hosted)

**Si besoin d'analytics frontend (pageviews, etc.):**

**Setup:**
```bash
# Docker compose
services:
  plausible:
    image: plausible/analytics:latest
    ports:
      - "127.0.0.1:8000:8000"  # Localhost only
    environment:
      - BASE_URL=http://nexus.onion
      - SECRET_KEY_BASE=<generated>
    volumes:
      - plausible-db:/var/lib/postgresql/data
```

**Intégration:**
```html
<!-- templates/base-nexus.html -->
<script defer data-domain="nexus.onion"
        src="http://127.0.0.1:8000/js/script.js"></script>
```

**Privacy:**
- ✅ Self-hosted (aucune donnée externe)
- ✅ Pas de cookies
- ✅ Pas d'IP tracking (masked by Tor anyway)
- ✅ Agrégation uniquement

**Priority:** P2 (Nice-to-have, pas critique)

---

## 6. Partie 4: KPIs & Mesure (Anonymisés)

### 6.1 Tableau de Bord Dual (Vendeur vs Acheteur)

**Dashboard admin:** `/admin/metrics` (protected route)

**Vue d'ensemble:**

```
┌──────────────────────────────────────────────────────────────┐
│                   NEXUS MARKETPLACE METRICS                   │
│                   Last Updated: 2025-11-02 14:32 UTC         │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  MARKETPLACE HEALTH                                           │
│  ==================                                           │
│  Active Listings: 1,234                                       │
│  Total Vendors: 456                                           │
│  Total Buyers: 2,890                                          │
│  Liquidity Ratio: 2.7 (buyers per active vendor)             │
│                                                               │
├──────────────────────────────────────────────────────────────┤
│  VENDOR METRICS (Supply Side)                                │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  Registration Funnel:                                         │
│  • Total Signups (30d): 89                                    │
│  • Wallet Setup Completed: 67 (75.3%)                         │
│  • First Listing Created: 45 (50.6%)                          │
│  → Activation Rate: 50.6%                                     │
│                                                               │
│  Time Metrics:                                                │
│  • Median Time-to-Multisig-Ready: 8 min                       │
│  • Median Time-to-First-Listing: 24 min                       │
│                                                               │
│  Retention:                                                   │
│  • D7 Retention: 62%                                          │
│  • D30 Retention: 45%                                         │
│                                                               │
├──────────────────────────────────────────────────────────────┤
│  BUYER METRICS (Demand Side)                                 │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  Activation Funnel:                                           │
│  • Anonymous Sessions (30d): 8,934                            │
│  • Signups: 234 (2.6% conversion)                             │
│  • Added to Cart: 178 (76.1%)                                 │
│  • First Purchase: 89 (38.0%)                                 │
│  → Activation Rate: 38.0%                                     │
│                                                               │
│  Time Metrics:                                                │
│  • Median Time-to-First-Add-to-Cart: 3 min                    │
│  • Median Time-to-First-Purchase: 12 min                      │
│                                                               │
│  Retention:                                                   │
│  • D7 Retention: 58%                                          │
│  • D30 Retention: 41%                                         │
│                                                               │
├──────────────────────────────────────────────────────────────┤
│  TRANSACTION METRICS                                          │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  Volume (30d):                                                │
│  • Total Orders: 567                                          │
│  • Total GMV: 45.6 XMR                                        │
│  • Avg Order Value: 0.08 XMR                                  │
│                                                               │
│  Escrow Health:                                               │
│  • Normal Completion: 96.3%                                   │
│  • Dispute Rate: 3.7%                                         │
│  • Avg Resolution Time: 2.3 days                              │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

### 6.2 Définition des KPIs Critiques

#### 6.2.1 Time-to-First-Value (TTFV)

**Pour Acheteur:**
```sql
-- Calcul TTFV Acheteur (time to first add-to-cart)
WITH user_registrations AS (
    SELECT user_id, MIN(created_at) AS reg_date
    FROM analytics_events
    WHERE event_type = 'user_registered'
    GROUP BY user_id
),
first_cart_adds AS (
    SELECT user_id, MIN(created_at) AS first_cart_date
    FROM analytics_events
    WHERE event_type = 'added_to_cart'
    GROUP BY user_id
)
SELECT
    PERCENTILE_CONT(0.5) WITHIN GROUP (
        ORDER BY EXTRACT(EPOCH FROM (fca.first_cart_date - ur.reg_date))
    ) AS median_ttfv_seconds
FROM user_registrations ur
JOIN first_cart_adds fca ON ur.user_id = fca.user_id;
```

**Pour Vendeur:**
```sql
-- Calcul TTFV Vendeur (time to first listing)
WITH vendor_registrations AS (
    SELECT user_id, MIN(created_at) AS reg_date
    FROM analytics_events
    WHERE event_type = 'user_registered'
      AND (properties->>'role')::text = 'vendor'
    GROUP BY user_id
),
first_listings AS (
    SELECT user_id, MIN(created_at) AS first_listing_date
    FROM analytics_events
    WHERE event_type = 'listing_created'
    GROUP BY user_id
)
SELECT
    PERCENTILE_CONT(0.5) WITHIN GROUP (
        ORDER BY EXTRACT(EPOCH FROM (fl.first_listing_date - vr.reg_date))
    ) AS median_ttl_seconds
FROM vendor_registrations vr
JOIN first_listings fl ON vr.user_id = fl.user_id;
```

#### 6.2.2 Taux d'Activation

**Formule:**
```
Taux Activation = (Utilisateurs ayant complété l'événement d'activation) / (Total nouveaux utilisateurs) × 100
```

**Événement d'activation:**
- **Acheteur:** Premier achat complété (OrderStatus::Funded)
- **Vendeur:** Première listing active + wallet multisig ready

**SQL:**
```sql
-- Taux d'activation Acheteur
SELECT
    (COUNT(DISTINCT CASE
        WHEN event_type = 'order_created' THEN user_id
    END)::FLOAT /
    COUNT(DISTINCT CASE
        WHEN event_type = 'user_registered'
            AND (properties->>'role')::text = 'buyer'
        THEN user_id
    END)) * 100 AS buyer_activation_rate
FROM analytics_events
WHERE created_at >= NOW() - INTERVAL '30 days';

-- Taux d'activation Vendeur
SELECT
    (COUNT(DISTINCT CASE
        WHEN event_type = 'listing_created' THEN user_id
    END)::FLOAT /
    COUNT(DISTINCT CASE
        WHEN event_type = 'user_registered'
            AND (properties->>'role')::text = 'vendor'
        THEN user_id
    END)) * 100 AS vendor_activation_rate
FROM analytics_events
WHERE created_at >= NOW() - INTERVAL '30 days';
```

#### 6.2.3 Taux de Rétention (D1, D7, D30)

**Définition:** % d'utilisateurs qui reviennent après N jours

**SQL:**
```sql
-- Rétention D7 (cohort analysis)
WITH cohorts AS (
    SELECT
        user_id,
        DATE_TRUNC('day', MIN(created_at)) AS cohort_date
    FROM analytics_events
    WHERE event_type = 'user_registered'
    GROUP BY user_id
),
returning_users AS (
    SELECT DISTINCT
        c.user_id,
        c.cohort_date,
        DATE_TRUNC('day', ae.created_at) AS return_date
    FROM cohorts c
    JOIN analytics_events ae ON c.user_id = ae.user_id
    WHERE ae.created_at > c.cohort_date + INTERVAL '6 days'
      AND ae.created_at <= c.cohort_date + INTERVAL '8 days'
)
SELECT
    c.cohort_date,
    COUNT(DISTINCT c.user_id) AS cohort_size,
    COUNT(DISTINCT ru.user_id) AS returned_d7,
    (COUNT(DISTINCT ru.user_id)::FLOAT / COUNT(DISTINCT c.user_id)) * 100 AS d7_retention
FROM cohorts c
LEFT JOIN returning_users ru ON c.user_id = ru.user_id AND c.cohort_date = ru.cohort_date
WHERE c.cohort_date >= NOW() - INTERVAL '60 days'
GROUP BY c.cohort_date
ORDER BY c.cohort_date DESC;
```

#### 6.2.4 Liquidity Ratio

**Définition:** Ratio acheteurs actifs / vendeurs actifs

**Formule:**
```
Liquidity Ratio = Active Buyers (30d) / Active Vendors (30d)
```

**Interprétation:**
- < 1.0: Pas assez de demande (risque de churn vendeurs)
- 1.0-3.0: Zone saine
- > 5.0: Pas assez d'offre (risque de churn acheteurs)

**SQL:**
```sql
SELECT
    (COUNT(DISTINCT CASE
        WHEN event_type IN ('added_to_cart', 'order_created')
        THEN user_id
    END)::FLOAT /
    NULLIF(COUNT(DISTINCT CASE
        WHEN event_type IN ('listing_created', 'listing_updated')
        THEN user_id
    END), 0)) AS liquidity_ratio
FROM analytics_events
WHERE created_at >= NOW() - INTERVAL '30 days';
```

### 6.3 Dashboard d'Implémentation

**Outil recommandé:** Grafana (self-hosted) + PostgreSQL datasource

**Setup:**
```yaml
# docker-compose.yml
services:
  grafana:
    image: grafana/grafana:latest
    ports:
      - "127.0.0.1:3000:3000"
    environment:
      - GF_AUTH_ANONYMOUS_ENABLED=false
      - GF_SECURITY_ADMIN_PASSWORD=<secure_password>
    volumes:
      - grafana-data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
```

**Dashboards à créer:**
1. **Executive Dashboard**: Liquidity, GMV, User Growth
2. **Onboarding Funnel**: Registration → Activation (Vendor/Buyer split)
3. **Retention Cohorts**: D1/D7/D30 retention charts
4. **Escrow Health**: Dispute rate, resolution time, completion rate

---

## 7. Roadmap d'Implémentation

### 7.1 Priorisation

**Framework: MoSCoW (Must have, Should have, Could have, Won't have)**

| Feature | Priority | Sprint | Effort | Impact |
|---------|----------|--------|--------|--------|
| Routes publiques (no auth) | **Must** | Sprint 1 | 2h | 🔥 High |
| Session cart (cookie) | **Must** | Sprint 1 | 4h | 🔥 High |
| Message "create account" au checkout | **Must** | Sprint 1 | 1h | 🔥 High |
| Trust badge escrow (product pages) | **Must** | Sprint 1 | 3h | 🔥 High |
| Optimiser formulaire listing (5 champs) | **Must** | Sprint 1 | 2h | 🔥 High |
| Shepherd.js integration | **Should** | Sprint 2 | 4h | 🟠 Medium |
| Homepage tour (3 steps) | **Should** | Sprint 2 | 3h | 🟠 Medium |
| Vendor wallet setup wizard | **Should** | Sprint 2 | 8h | 🔥 High |
| Buyer registration wizard (3 screens) | **Should** | Sprint 2 | 6h | 🔥 High |
| Draft system (listings) | **Should** | Sprint 2 | 4h | 🟠 Medium |
| Notifications table + service | **Should** | Sprint 3 | 6h | 🔥 High |
| HTMX notification polling | **Should** | Sprint 3 | 4h | 🔥 High |
| Vendor dashboard | **Should** | Sprint 3 | 8h | 🟠 Medium |
| Reputation system (badges) | **Should** | Sprint 3 | 6h | 🟠 Medium |
| Listing boost (featured) | **Could** | Sprint 3 | 6h | 🟡 Low |
| Analytics events table | **Should** | Sprint 4 | 4h | 🟠 Medium |
| Admin metrics dashboard | **Should** | Sprint 4 | 8h | 🟠 Medium |
| Auto-hash user_id (privacy) | **Should** | Sprint 4 | 2h | 🟠 Medium |
| Bulk CSV upload | **Could** | Sprint 5 | 8h | 🟡 Low |
| API REST (power sellers) | **Could** | Sprint 6 | 12h | 🟡 Low |
| IPFS integration | **Should** | Sprint 5 | 12h | 🟠 Medium |
| Plausible analytics | **Won't** | - | - | 🔵 Nice-to-have |

### 7.2 Sprint Détaillés

#### **Sprint 1: Quick Wins (Semaine 1) - 12h total**

**Objectif:** Réduire friction onboarding acheteur immédiatement

**Tasks:**

1. **Rendre routes publiques** (2h)
   ```bash
   # Fichier: server/src/main.rs
   - Retirer AuthMiddleware de: /, /listings, /listings/:id, /search
   - Garder AuthMiddleware sur: /cart/checkout, /orders, /listings/create
   ```

2. **Implémenter session cart** (4h)
   ```bash
   # Fichiers:
   - server/src/models/cart.rs (struct SessionCart)
   - server/src/handlers/cart.rs (add_to_cart, remove, update_quantity)
   - Configurer actix-session middleware
   ```

3. **Message au checkout** (1h)
   ```bash
   # Fichier: templates/cart/index.html
   - Ajouter condition: if !logged_in
   - Afficher: "Create account to checkout (30 sec)"
   - Bouton: [Create Account & Checkout →] → /auth/register?redirect=/cart/checkout
   ```

4. **Trust badge escrow** (3h)
   ```bash
   # Fichier: templates/listings/show.html
   - Créer partial: templates/partials/trust-badge.html
   - Styling glassmorphism
   - Link vers /docs/escrow-explained
   ```

5. **Optimiser formulaire listing** (2h)
   ```bash
   # Fichier: templates/listings/create.html
   - Réduire à 5 champs required
   - Reste dans <details> (optional)
   ```

**Acceptance Criteria:**
- ✅ Visiteur peut browser sans compte
- ✅ Panier fonctionne sans compte
- ✅ Message clair au checkout
- ✅ Badge escrow visible sur product pages
- ✅ Formulaire listing = max 5 champs visibles

**Deploy:** Testnet staging

---

#### **Sprint 2: Wizards & Progressive Disclosure (Semaine 2-3) - 25h total**

**Objectif:** Créer expérience d'onboarding guidée

**Tasks:**

1. **Shepherd.js setup** (4h)
   ```bash
   - Télécharger Shepherd.js en local (static/vendor/shepherd/)
   - Créer script: static/js/tours.js
   - Intégrer dans base-nexus.html
   ```

2. **Homepage tour** (3h)
   ```bash
   # Fichier: static/js/tours.js
   - Tour 3 steps: Welcome → Escrow → Browse
   - localStorage check (show once)
   ```

3. **Vendor wallet wizard** (8h)
   ```bash
   # Route: /vendor/wallet-setup
   # Template: templates/vendor/wallet-setup-wizard.html
   - Step 1: Education (why multisig)
   - Step 2: Generate wallet (call backend API)
   - Step 3: Automated setup (progress bar)
   - Step 4: Verification
   - Step 5: Optional bond

   # Backend:
   - server/src/handlers/vendor.rs::wallet_setup_step2()
   - Appeler wallet/src/client.rs::prepare_multisig()
   ```

4. **Buyer registration wizard** (6h)
   ```bash
   # Route: /auth/register
   # Template: templates/auth/register-wizard.html
   - Screen 1: Username + Password
   - Screen 2: Wallet setup (optional/temp)
   - Screen 3: Welcome + CTA

   # Redirect context:
   - Si vient de /cart/checkout → redirect vers checkout après
   ```

5. **Draft system** (4h)
   ```bash
   # Migration: create_listing_drafts table
   # Model: server/src/models/listing_draft.rs
   # Handler: POST /listings/save-draft (HTMX)
   # Button: "Save Draft" sur formulaire
   ```

**Acceptance Criteria:**
- ✅ Tour homepage s'affiche au first visit
- ✅ Vendor wizard guide setup wallet (5 steps)
- ✅ Buyer registration = 3 écrans max
- ✅ Drafts sauvegardables

**Deploy:** Testnet staging

---

#### **Sprint 3: Engagement & Retention (Semaine 4-5) - 24h total**

**Objectif:** Activer utilisateurs et les faire revenir

**Tasks:**

1. **Table notifications** (2h)
   ```bash
   # Migration: create_notifications
   # Schema: id, user_id, type, title, message, link, read, created_at
   ```

2. **Service notifications** (4h)
   ```bash
   # Fichier: server/src/services/notifications.rs
   - create()
   - get_unread_count()
   - get_recent()
   - mark_as_read()
   ```

3. **Handlers API** (2h)
   ```bash
   # Fichier: server/src/handlers/notifications.rs
   - GET /api/notifications/unread-count
   - GET /api/notifications
   - POST /api/notifications/:id/read
   ```

4. **Frontend HTMX** (4h)
   ```bash
   # Fichier: templates/partials/header.html
   - Bell icon avec badge
   - Polling HTMX every 30s
   - Dropdown avec liste
   ```

5. **Triggers événements** (4h)
   ```bash
   # Ajouter appels notifications::create() dans:
   - handlers/orders.rs (order shipped, delivered)
   - handlers/escrow.rs (dispute opened)
   - handlers/messages.rs (new message)
   ```

6. **Vendor dashboard** (8h)
   ```bash
   # Route: /vendor/dashboard
   # Template: templates/vendor/dashboard.html
   - Métriques: listings, views, CTR, orders, revenue
   - Query helpers dans services/analytics.rs
   ```

**Acceptance Criteria:**
- ✅ Notifications in-app fonctionnelles
- ✅ Polling HTMX toutes les 30s
- ✅ Triggers sur événements clés
- ✅ Dashboard vendeur avec métriques

**Deploy:** Testnet staging

---

#### **Sprint 4: Mesure & Optimisation (Semaine 6) - 14h total**

**Objectif:** Instrumenter pour mesurer KPIs

**Tasks:**

1. **Table analytics_events** (2h)
   ```bash
   # Migration: create_analytics_events
   # Schema: id, user_id, event_type, properties (JSONB), created_at
   # Index: (event_type, created_at)
   ```

2. **Service analytics** (4h)
   ```bash
   # Fichier: server/src/services/analytics.rs
   - track_event() enum-based
   - calculate_ttfv_buyer()
   - calculate_ttfv_vendor()
   - calculate_activation_rate()
   - calculate_retention()
   ```

3. **Instrumentation** (4h)
   ```bash
   # Ajouter track_event() dans:
   - handlers/auth.rs::register() → UserRegistered
   - handlers/listings.rs::show() → ListingViewed
   - handlers/cart.rs::add() → AddedToCart
   - handlers/orders.rs::create() → OrderCreated
   # ... etc
   ```

4. **Dashboard admin** (4h)
   ```bash
   # Route: /admin/metrics (protected)
   # Template: templates/admin/metrics.html
   - Liquidity ratio
   - TTFV (buyer/vendor)
   - Activation rates
   - Retention D7/D30
   - Query data via services/analytics.rs
   ```

**Acceptance Criteria:**
- ✅ Events trackés en DB
- ✅ Queries KPIs fonctionnelles
- ✅ Dashboard admin accessible
- ✅ Privacy: auto-hash après 30j (cron job)

**Deploy:** Testnet staging

---

### 7.3 Estimation Totale

**Total effort:** ~75 heures (1.5-2 mois pour 1 dev full-time)

**Répartition:**
- Sprint 1 (Quick Wins): 12h
- Sprint 2 (Wizards): 25h
- Sprint 3 (Engagement): 24h
- Sprint 4 (Analytics): 14h

**Sprints 5-6 (Optionnel - features "Could have"):**
- IPFS integration: 12h
- Bulk CSV: 8h
- API REST: 12h

---

## 8. Actions Immédiates

### 8.1 Checklist Next Steps

**Aujourd'hui (2h):**
- [ ] Créer branch `feature/onboarding-adapte`
- [ ] Commit ce document: `DOX/plans/ONBOARDING-ADAPTE-NEXUS.md`
- [ ] Audit routes (quelles nécessitent auth?)
  ```bash
  grep -r "AuthMiddleware\|require_auth" server/src/
  ```
- [ ] Tester navigation sans compte (identifier blockers)

**Cette semaine (Sprint 1 - 12h):**
- [ ] Retirer auth middleware des routes publiques
- [ ] Implémenter session cart (cookie crypté)
- [ ] Ajouter message "create account" au checkout
- [ ] Créer partial trust-badge.html
- [ ] Optimiser formulaire création listing

**Semaine 2-3 (Sprint 2 - 25h):**
- [ ] Télécharger Shepherd.js (local, pas CDN)
- [ ] Créer homepage tour (3 steps)
- [ ] Développer wizard wallet setup (5 steps)
- [ ] Développer wizard registration buyer (3 screens)
- [ ] Implémenter draft system

### 8.2 Questions à Résoudre

**Décisions techniques:**

1. **Session storage:**
   - ✅ Cookie crypté (actix-session) → Recommandé
   - ⚠️ Redis (meilleure scalabilité mais complexité)

2. **Temp wallet pour buyers:**
   - Option A: Backend crée wallet, user peut exporter seed
   - Option B: Forcer user à créer wallet (friction)
   - **Recommandation:** Option A (minimal friction)

3. **Vendor bond storage:**
   - Option A: On-chain (wallet balance)
   - Option B: Off-chain (DB + proof)
   - **Recommandation:** Option A (plus trustless)

4. **Analytics privacy:**
   - Auto-hash après combien de jours? (30j recommandé)
   - Store IP hashs? (Non, Tor les masque anyway)

**Décisions UX:**

1. **Tour homepage:**
   - Show on every visit ou localStorage once?
   - **Recommandation:** Once (localStorage check)

2. **Activation definition:**
   - Buyer: First add-to-cart OU first purchase?
   - **Recommandation:** First purchase (plus significatif)

3. **Notification frequency:**
   - Polling every 30s ou WebSockets?
   - **Recommandation:** Polling (simpler, pas de WS overhead)

### 8.3 Risques & Mitigations

| Risque | Impact | Probabilité | Mitigation |
|--------|--------|-------------|------------|
| **Friction crypto trop élevée** | 🔥 High | Medium | Wizard éducatif + option temp wallet |
| **Spam sans CAPTCHA** | 🟠 Medium | High | Rate limiting + Proof-of-Work client |
| **Abandon au wallet setup** | 🔥 High | Medium | Progressive disclosure (setup après premier achat OK) |
| **Liquidity imbalance** (trop buyers, pas assez vendors) | 🔥 High | Medium | Incitations vendors (boost, priority arbitrage) |
| **Privacy leaks via analytics** | 🔥 High | Low | Auto-hash, pas d'IP tracking, audits réguliers |

---

## 9. Références

### 9.1 Documents Projet

- [ONBOARDING.md](../../ONBOARDING.md) - Playbook source
- [CLAUDE.md](../../CLAUDE.md) - Guidelines projet
- [README.md](../../README.md) - Vue d'ensemble
- [.cursorrules](../../.cursorrules) - Règles développement

### 9.2 Spécifications Techniques

- [wallet/src/](../../wallet/src/) - Monero RPC client
- [server/src/handlers/](../../server/src/handlers/) - API handlers
- [templates/](../../templates/) - Tera templates

### 9.3 Outils & Bibliothèques

**UX:**
- [Shepherd.js](https://shepherdjs.dev/) - Guided tours
- [HTMX](https://htmx.org/) - Interactivité sans JS framework

**Backend:**
- [Actix-web](https://actix.rs/) - Web framework
- [Diesel](https://diesel.rs/) - ORM
- [Tera](https://tera.netlify.app/) - Template engine

**Analytics:**
- [Plausible](https://plausible.io/) - Privacy-friendly analytics (optionnel)
- [Grafana](https://grafana.com/) - Dashboard (optionnel)

### 9.4 Lectures Recommandées

**Onboarding UX:**
- [UserOnboarding.com](https://useronboarding.com/) - Teardowns de SaaS
- [Laws of UX](https://lawsofux.com/) - Principes psychologiques

**Privacy:**
- [OWASP Privacy Risks](https://owasp.org/www-project-top-10-privacy-risks/)
- [Tor Project Best Practices](https://2019.www.torproject.org/docs/documentation.html.en)

**Monero:**
- [Monero Documentation](https://www.getmonero.org/resources/developer-guides/)
- [Multisig Guide](https://www.getmonero.org/resources/user-guides/multisig-messaging-system.html)

---

## Conclusion

Ce document transforme le playbook d'onboarding traditionnel en une stratégie **privacy-first** adaptée au Monero Marketplace. Les principes universels (double flux, progressive disclosure, gamification) restent valides, mais l'implémentation remplace KYC/PSP/email par **wallet setup/multisig/notifications in-app**.

**La formule du succès:**
```
Liquidité Nexus = (Vitesse Wallet Setup × Qualité Catalogue) / (Friction Crypto × FUD Sécurité)
```

**Prochaines étapes:**
1. Valider les décisions techniques (section 8.2)
2. Lancer Sprint 1 (quick wins, 12h)
3. Itérer basé sur métriques (Time-to-First-Value, Activation Rate)

---

**Dernière mise à jour:** 2025-11-02
**Auteur:** Claude Code
**Statut:** 🟢 Document de Référence Actif