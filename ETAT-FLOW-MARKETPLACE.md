# État du Flow Marketplace - Audit Complet

**Date:** 2025-10-27
**Version:** v0.2.6 Alpha
**Environnement:** Testnet uniquement

---

## 📊 Résumé Exécutif

### ✅ Backend: 95% Complet
- **Base de données:** Toutes les tables nécessaires (users, listings, orders, escrows, transactions, wallet_rpc_configs, reviews)
- **API REST:** Endpoints complets pour auth, listings, orders, escrow
- **Services:** Orchestration d'escrow, monitoring blockchain, timeouts, airgap arbiter
- **Multisig:** Implémentation 2-of-3 avec Monero RPC

### ✅ Frontend: 90% Complet
- **Design System:** NEXUS appliqué sur 69 templates
- **Pages principales:** Listings, Orders, Escrow visualization, Auth
- **Composants:** Cards, badges, forms, breadcrumbs, timelines

### ⚠️ À Tester: Flow Complet End-to-End
- Flow de création de listing
- Flow d'achat/commande
- Flow d'escrow multisig (init, funding, completion)
- Flow de dispute et résolution

---

## 🗂️ Architecture Détaillée

### Base de Données (schema.rs)

#### Table: `users`
```
id, username, password_hash, role, wallet_address, wallet_id, created_at, updated_at
```
**Rôles:** buyer, vendor, arbiter

#### Table: `listings`
```
id, vendor_id, title, description, price_xmr, stock, status, images_ipfs_cids, created_at, updated_at
```
**Status:** active, inactive, sold_out

#### Table: `orders`
```
id, buyer_id, vendor_id, listing_id, escrow_id, status, total_xmr, created_at, updated_at
```
**Status:** pending, funded, shipped, completed, cancelled, disputed, refunded

#### Table: `escrows`
```
id, order_id, buyer_id, vendor_id, arbiter_id, amount, multisig_address, status,
buyer_wallet_info, vendor_wallet_info, arbiter_wallet_info, transaction_hash,
multisig_phase, multisig_state_json, recovery_mode, expires_at, last_activity_at, ...
```
**Status:** pending, awaiting_funding, funded, released, refunded, disputed
**Multisig Phases:** prepare, make, exchange_round1, exchange_round2, ready

#### Table: `transactions`
```
id, escrow_id, tx_hash, amount_xmr, confirmations, created_at
```

#### Table: `wallet_rpc_configs`
```
wallet_id, escrow_id, role, rpc_url_encrypted, rpc_user_encrypted, rpc_password_encrypted,
created_at, last_connected_at, connection_attempts, last_error
```
**Architecture Non-Custodiale:** Clients fournissent leurs propres URLs RPC

#### Table: `reviews`
```
id, txid, reviewer_id, vendor_id, rating, comment, buyer_pubkey, signature, timestamp, verified, created_at
```

---

## 🛠️ API Endpoints Disponibles

### Authentication (`/api/auth/`)
- `POST /register` - Création de compte
- `POST /login` - Authentification
- `POST /logout` - Déconnexion
- `GET /whoami` - Info utilisateur actuel

### Listings (`/api/listings/`)
- `POST /` - Créer un listing (vendor only)
- `POST /with-images` - Créer avec upload IPFS
- `GET /` - Lister tous les listings
- `GET /{id}` - Détails d'un listing
- `GET /vendor/{vendor_id}` - Listings d'un vendeur
- `GET /search?q=...` - Recherche
- `PUT /{id}` - Modifier un listing
- `DELETE /{id}` - Supprimer un listing
- `POST /{id}/images` - Upload images IPFS
- `GET /{id}/images/{cid}` - Récupérer image
- `DELETE /{id}/images/{cid}` - Supprimer image

### Orders (`/api/orders/`)
- `POST /` - Créer une commande
- `GET /` - Lister mes commandes
- `GET /{id}` - Détails d'une commande
- `GET /pending/count` - Nombre de commandes en attente
- `POST /{id}/init-escrow` - Initialiser l'escrow
- `POST /{id}/ship` - Marquer comme expédié (vendor)
- `POST /{id}/complete` - Compléter la commande
- `POST /{id}/cancel` - Annuler la commande
- `POST /{id}/dispute` - Ouvrir un litige
- `POST /{id}/dev-simulate-payment` - (Testnet only) Simuler paiement

### Escrow (`/api/escrow/`)
- `POST /register-wallet-rpc` - Enregistrer wallet RPC du client
- `GET /{id}` - Détails de l'escrow
- `POST /{id}/prepare` - Préparer multisig
- `POST /{id}/make` - Créer wallet multisig
- `POST /{id}/exchange` - Échanger infos multisig
- `POST /{id}/finalize` - Finaliser setup
- `POST /{id}/fund` - Funder l'escrow (buyer)
- `POST /{id}/release` - Libérer les fonds (2-of-3 signatures)
- `POST /{id}/refund` - Rembourser (2-of-3 signatures)

---

## 📁 Templates Frontend (69 fichiers)

### Pages Principales
- `templates/index.html` - Homepage avec NEXUS hero
- `templates/auth/login.html` - Login avec toast notifications
- `templates/auth/register.html` - Registration

### Listings
- `templates/listings/index.html` - Liste des produits (grid NEXUS)
- `templates/listings/show.html` - Détail produit avec images
- `templates/listings/create.html` - Formulaire création
- `templates/listings/edit.html` - Formulaire édition

### Orders
- `templates/orders/index.html` - Mes commandes
- `templates/orders/show.html` - Détail commande avec badges statut

### Escrow
- `templates/escrow/show.html` - Visualisation escrow avec timeline
- `templates/escrow/show-nexus.html` - Version NEXUS premium
- `templates/escrow/modals/` - Modals pour actions escrow

### Settings
- `templates/settings/index.html` - Paramètres utilisateur
- `templates/settings/wallet.html` - Configuration wallet RPC

### Composants NEXUS
- `templates/partials/nexus/atoms/` - 10 composants atomiques
- `templates/partials/nexus/molecules/` - 14 composants moléculaires
- `templates/partials/nexus/organisms/` - 7 composants organismes

---

## 🔄 Flow Théorique Complet

### 1️⃣ Création de Listing (Vendeur)
```
1. Vendeur s'inscrit et se connecte (/register → /login)
2. Vendeur va sur /listings/create
3. Remplit le formulaire (titre, description, prix XMR, stock)
4. (Optionnel) Upload images via IPFS
5. POST /api/listings/with-images
6. Listing créé avec status="active"
```

**Status Backend:** ✅ Implémenté
**Status Frontend:** ✅ Formulaire NEXUS complet
**À Tester:** Upload IPFS, validation, erreurs

---

### 2️⃣ Achat et Création de Commande (Acheteur)
```
1. Acheteur browse /listings
2. Clique sur un produit → /listings/{id}
3. Clique "Buy Now" ou "Add to Cart"
4. POST /api/orders { listing_id, quantity }
5. Order créé avec status="pending"
6. Redirection vers /orders/{order_id}
```

**Status Backend:** ✅ Implémenté
**Status Frontend:** ⚠️ À vérifier (bouton "Buy Now" existe?)
**À Tester:** Flow complet, validation stock, calcul total

---

### 3️⃣ Initialisation Escrow Multisig 2-of-3
```
1. Sur /orders/{id}, bouton "Initialize Escrow"
2. POST /api/orders/{id}/init-escrow { arbiter_id }
3. Backend crée escrow avec status="pending"
4. Backend sélectionne arbiter (ou acheteur choisit)
5. 3 wallets multisig créés (buyer, vendor, arbiter)
6. Escrow status → "awaiting_funding"
```

**Status Backend:** ✅ Implémenté (handlers + service)
**Status Frontend:** ⚠️ À vérifier (UI pour init?)
**À Tester:** Sélection arbiter, création wallets, erreurs

---

### 4️⃣ Configuration Wallet RPC (3 parties)
```
BUYER:
1. Va sur /settings/wallet
2. Entre son RPC URL: http://127.0.0.1:18082/json_rpc
3. POST /api/escrow/register-wallet-rpc { rpc_url, role: "buyer" }
4. Wallet enregistré de manière sécurisée (encrypted)

VENDOR:
1. Même processus avec role="vendor"

ARBITER:
1. Serveur gère le wallet arbiter (airgap ou local)
```

**Status Backend:** ✅ Implémenté
**Status Frontend:** ✅ Page settings/wallet existe
**À Tester:** Encryption, validation URL, connexion Tor

---

### 5️⃣ Setup Multisig (6 étapes strictes)
```
STEP 1: prepare_multisig()
- Chaque partie génère multisig_info
- POST /api/escrow/{id}/prepare

STEP 2: make_multisig()
- Échange des multisig_info
- POST /api/escrow/{id}/make { infos: [...] }
- Wallet multisig créé

STEP 3: export_multisig_info()
- Chaque partie exporte sync info (round 1)
- POST /api/escrow/{id}/exchange

STEP 4: import_multisig_info()
- Import des sync infos des autres
- Backend synchronise les wallets

STEP 5: export + import (round 2)
- Répétition pour sync complète

STEP 6: is_multisig() check
- Vérification que tout est prêt
- Escrow status → "awaiting_funding"
```

**Status Backend:** ✅ Implémenté dans `services/escrow.rs`
**Status Frontend:** ⚠️ UI pour suivre les étapes?
**À Tester:** Flow complet, gestion erreurs, timeouts

---

### 6️⃣ Funding de l'Escrow (Acheteur)
```
1. Acheteur sur /escrow/{id}
2. Voit l'adresse multisig
3. Transfert XMR vers cette adresse
4. POST /api/escrow/{id}/fund
5. Backend vérifie la transaction via RPC
6. Attente confirmations (10 confirmations recommandées)
7. Escrow status → "funded"
8. Order status → "funded"
```

**Status Backend:** ✅ Implémenté avec blockchain monitor
**Status Frontend:** ✅ Page escrow/show avec adresse
**À Tester:** Vérification TX, confirmations, timeouts

---

### 7️⃣ Expédition et Livraison (Vendeur)
```
1. Vendeur voit order status="funded"
2. Expédie le produit
3. POST /api/orders/{id}/ship
4. Order status → "shipped"
5. Acheteur reçoit notification
6. Acheteur confirme réception
7. POST /api/orders/{id}/complete
```

**Status Backend:** ✅ Endpoints implémentés
**Status Frontend:** ⚠️ Boutons sur /orders/{id}?
**À Tester:** Notifications, workflow

---

### 8️⃣ Release des Fonds (2-of-3 Signatures)
```
HAPPY PATH (Buyer + Vendor):
1. Acheteur satisfait, clique "Release Funds"
2. POST /api/escrow/{id}/release
3. Backend récupère 2 signatures (buyer + vendor)
4. Transaction multisig signée et broadcastée
5. Fonds transférés au vendeur
6. Escrow status → "released"
7. Order status → "completed"

DISPUTE PATH (Buyer + Arbiter OU Vendor + Arbiter):
1. Problème → POST /api/orders/{id}/dispute
2. Order status → "disputed"
3. Arbiter examine le cas
4. Arbiter décide: release to vendor OU refund to buyer
5. Arbiter + 1 partie signent
6. Transaction exécutée
```

**Status Backend:** ✅ Implémenté avec signing logic
**Status Frontend:** ⚠️ UI pour release/dispute?
**À Tester:** Signatures multisig, broadcast TX, erreurs

---

### 9️⃣ Reviews et Reputation
```
1. Order completed
2. Acheteur peut laisser un avis
3. POST /api/reviews { rating, comment, signature }
4. Review vérifié via cryptographic signature
5. Reputation du vendeur mise à jour
```

**Status Backend:** ✅ Table reviews + handlers reputation
**Status Frontend:** ⚠️ Formulaire review?
**À Tester:** Signature verification, display reputation

---

## 🧪 Plan de Test End-to-End

### Prérequis
```bash
# 1. Monero testnet RPC running
monero-wallet-rpc --testnet --daemon-address stagenet.xmr-tw.org:38081 \
  --rpc-bind-port 18082 --disable-rpc-login --wallet-dir ./wallets

# 2. Tor daemon running
sudo systemctl start tor

# 3. Database migrations applied
diesel migration run

# 4. Server running
cargo run --release --package server
```

### Scénario de Test Complet

#### Phase 1: Setup
```
1. Créer 3 utilisateurs:
   - alice (buyer)
   - bob (vendor)
   - charlie (arbiter)

2. Bob crée un listing:
   - Titre: "Premium VPN Subscription"
   - Prix: 0.5 XMR
   - Stock: 10
```

#### Phase 2: Purchase Flow
```
3. Alice browse /listings
4. Alice clique sur le listing de Bob
5. Alice clique "Buy Now"
6. Order créé (status=pending)
7. Alice clique "Initialize Escrow"
8. Escrow créé avec Charlie comme arbiter
```

#### Phase 3: Wallet Setup
```
9. Alice configure son wallet RPC (/settings/wallet)
10. Bob configure son wallet RPC
11. Charlie (arbiter) → server gère automatiquement
```

#### Phase 4: Multisig Setup
```
12. Alice: POST /api/escrow/{id}/prepare
13. Bob: POST /api/escrow/{id}/prepare
14. Charlie: POST /api/escrow/{id}/prepare (auto)

15. Backend: POST /api/escrow/{id}/make (échange infos)

16. Alice: POST /api/escrow/{id}/exchange (round 1)
17. Bob: POST /api/escrow/{id}/exchange (round 1)
18. Charlie: POST /api/escrow/{id}/exchange (round 1)

19. Backend: import_multisig_info (sync round 1)

20. Alice: POST /api/escrow/{id}/exchange (round 2)
21. Bob: POST /api/escrow/{id}/exchange (round 2)
22. Charlie: POST /api/escrow/{id}/exchange (round 2)

23. Backend: Finalize → escrow.status = "awaiting_funding"
```

#### Phase 5: Funding
```
24. Alice voit l'adresse multisig sur /escrow/{id}
25. Alice transfert 0.5 XMR vers cette adresse
26. Backend détecte la TX (blockchain monitor)
27. Attente 10 confirmations
28. Escrow status → "funded"
29. Order status → "funded"
```

#### Phase 6: Delivery
```
30. Bob voit order funded
31. Bob expédie le produit
32. Bob: POST /api/orders/{id}/ship
33. Alice reçoit notification
34. Alice confirme réception
35. Alice: POST /api/orders/{id}/complete
```

#### Phase 7: Release
```
36. Alice: POST /api/escrow/{id}/release
37. Backend collecte 2 signatures (Alice + Bob)
38. Transaction multisig signée
39. Broadcast sur testnet
40. Fonds transférés à Bob
41. Escrow status → "released"
42. Order status → "completed"
```

#### Phase 8: Review
```
43. Alice laisse un avis (5 étoiles)
44. Review vérifié et ajouté
45. Reputation de Bob mise à jour
```

---

## 🚨 Points Critiques à Vérifier

### 1. Non-Custodial Architecture
- [ ] Server ne stocke JAMAIS de private keys
- [ ] Clients fournissent leurs propres RPC URLs
- [ ] Encryption des RPC credentials en DB

### 2. Multisig Setup
- [ ] 6 étapes respectées dans l'ordre
- [ ] State checks avant chaque étape
- [ ] Gestion des erreurs de sync
- [ ] Timeouts appropriés

### 3. Security
- [ ] CSRF tokens sur toutes les actions
- [ ] Rate limiting (5 auth attempts / 15 min)
- [ ] No .onion addresses in logs
- [ ] Tor proxy pour tous les calls externes

### 4. Error Handling
- [ ] Pas de .unwrap() ou .expect() sans justification
- [ ] Context messages clairs sur toutes les erreurs
- [ ] Rollback en cas d'échec multisig
- [ ] Recovery mode pour escrows bloqués

### 5. Frontend UX
- [ ] Toast notifications pour tous les events
- [ ] Loading states pendant les opérations
- [ ] Désactivation des boutons pendant processing
- [ ] Messages d'erreur clairs pour l'utilisateur

---

## 📝 Prochaines Étapes

### Tests Prioritaires
1. ✅ Audit backend complet (FAIT)
2. ✅ Audit templates frontend (FAIT)
3. ⏳ Tester création de listing end-to-end
4. ⏳ Tester flow d'achat complet
5. ⏳ Tester multisig setup (6 étapes)
6. ⏳ Tester funding et confirmations
7. ⏳ Tester release avec 2-of-3 sigs
8. ⏳ Tester dispute flow

### Améliorations Possibles
- [ ] Ajouter panier d'achat (multi-listings)
- [ ] Améliorer sélection d'arbitre (liste, reputation)
- [ ] Ajouter chat encrypted buyer-vendor
- [ ] Améliorer monitoring blockchain (WebSocket updates)
- [ ] Ajouter export de receipts/invoices
- [ ] Dashboard vendor avec stats

---

## 📚 Documentation de Référence

- **CLAUDE.md** - Guide développement complet
- **docs/TESTING.md** - Guide de test
- **server/tests/escrow_e2e.rs** - Tests E2E existants
- **docs/specs/** - Spécifications des fonctions
- **docs/reality-checks/** - Vérifications Tor

---

**Status Général:** 🟡 Backend prêt, Frontend prêt, Tests E2E nécessaires

**Prochaine Action:** Lancer le serveur et tester le flow complet avec 3 utilisateurs.
