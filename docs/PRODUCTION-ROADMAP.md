# Feuille de Route vers Production
## Monero Marketplace - Hidden Service .onion

**Version Actuelle:** 0.1.0-alpha
**Date de Début:** 2025-10-16
**Estimation Totale:** 8-11 mois (32-46 semaines)
**Objectif:** Production mainnet avec audit de sécurité complet

---

## 📊 État Actuel (Baseline)

### ✅ Composants Complétés
- [x] Architecture workspace Rust (common, wallet, cli)
- [x] Client RPC Monero avec isolation localhost strict
- [x] Workflow multisig complet (prepare, make, export, import)
- [x] CLI fonctionnel avec toutes commandes multisig
- [x] Système de Reality Checks Tor automatique
- [x] 6 Reality Checks Tor validés
- [x] Documentation extensive (34+ fichiers)
- [x] Tests d'intégration de base
- [x] Système anti-security-theatre (linting, pre-commit hooks)

### 🚧 Lacunes Identifiées
- [ ] Tests multisig end-to-end avec 3 wallets simultanés
- [ ] Hidden service .onion (Tor onion service v3)
- [ ] Backend web API REST
- [ ] Création/signature de transactions multisig
- [ ] Finalization & broadcast de transactions
- [ ] Frontend web (interface utilisateur)
- [ ] Système de dispute resolution (arbitrage)
- [ ] Monitoring & alerting production
- [ ] Audit de sécurité externe
- [ ] Infrastructure de déploiement

---

## Phase 1: Complétion du Multisig Core
**Durée:** 4-6 semaines
**Priorité:** CRITIQUE
**Objectif:** Workflow multisig 2-of-3 entièrement fonctionnel et testé

### 1.1 Tests End-to-End (Semaine 1-2)
**Délivrables:**
- [ ] Script de setup automatique pour 3 wallets testnet
- [ ] Test: 3 parties préparent multisig simultanément
- [ ] Test: Création wallet 2-of-3 avec échange d'infos
- [ ] Test: 2 rounds de sync (export/import)
- [ ] Test: Vérification état multisig final (is_multisig)
- [ ] Reality Check Tor pour chaque étape
- [ ] Documentation du flow complet avec diagrammes

**Acceptance Criteria:**
- ✅ 3 wallets testnet synchronisés sans erreur
- ✅ Tous les tests passent automatiquement
- ✅ Temps de setup < 5 minutes
- ✅ Zero unwrap/panic dans le code
- ✅ Reality Checks validés pour toutes fonctions

### 1.2 Transactions Multisig (Semaine 3-4)
**Délivrables:**
- [ ] Fonction `create_transaction()` - Créer TX non signée
- [ ] Fonction `sign_transaction()` - Signer avec clé locale
- [ ] Fonction `submit_transaction()` - Soumettre aux autres signataires
- [ ] Fonction `finalize_transaction()` - Finaliser avec 2/3 signatures
- [ ] Fonction `broadcast_transaction()` - Diffuser sur blockchain
- [ ] Tests avec montants réels sur testnet
- [ ] Monitoring des confirmations blockchain

**Spécifications Requises:**
- Spec pour chaque fonction (format standard projet)
- Reality Checks Tor pour tous les appels RPC
- Validation des montants (atomic units)
- Gestion des fees automatique
- Retry logic pour broadcast failures

### 1.3 Error Handling & Edge Cases (Semaine 5-6)
**Délivrables:**
- [ ] Test: Wallet déjà en mode multisig
- [ ] Test: Infos multisig invalides/corrompues
- [ ] Test: Timeout lors de l'échange d'infos
- [ ] Test: Participant déconnecté mid-flow
- [ ] Test: Insufficient funds pour transaction
- [ ] Test: Double-spend attempt detection
- [ ] Documentation des codes d'erreur

**Métriques de Succès:**
- Code coverage > 80% pour wallet/multisig.rs
- Zero panic possible (all paths return Result)
- Tous les edge cases documentés
- Temps de recovery < 30s après erreur

---

## Phase 2: Backend Web Service (.onion)
**Durée:** 6-8 semaines
**Priorité:** HAUTE
**Objectif:** Hidden service fonctionnel avec API REST

### 2.1 Tor Hidden Service Setup (Semaine 7-8)
**Délivrables:**
- [ ] Nouveau crate `server/` dans workspace
- [ ] Configuration Tor hidden service v3
- [ ] Génération des clés .onion automatique
- [ ] Health check endpoint `/api/health`
- [ ] Vérification isolation réseau (localhost only RPC)
- [ ] Reality Check Tor pour hidden service

**Stack Technique:**
```rust
server/
├── Actix-web 4.x (ou Rocket 0.5)
├── Hidden service config dans /etc/tor/torrc
├── HTTPS avec self-signed cert (Tor gère encryption)
└── Rate limiting middleware
```

**Configuration Tor:**
```
HiddenServiceDir /var/lib/tor/marketplace/
HiddenServicePort 80 127.0.0.1:8080
HiddenServiceVersion 3
```

### 2.2 API REST - Marketplace Core (Semaine 9-11)
**Endpoints Requis:**

#### Listings (Produits)
- `GET /api/listings` - Liste des produits
- `GET /api/listings/:id` - Détail produit
- `POST /api/listings` - Créer listing (vendor)
- `PUT /api/listings/:id` - Modifier listing
- `DELETE /api/listings/:id` - Supprimer listing

#### Orders (Commandes)
- `POST /api/orders` - Créer commande
- `GET /api/orders/:id` - Détail commande
- `GET /api/orders/user/:user_id` - Commandes utilisateur
- `PUT /api/orders/:id/status` - Update statut

#### Escrow (Multisig)
- `POST /api/escrow/init` - Initialiser escrow 2-of-3
- `POST /api/escrow/:id/prepare` - Prepare multisig
- `POST /api/escrow/:id/make` - Make multisig
- `POST /api/escrow/:id/sync` - Sync rounds
- `GET /api/escrow/:id/status` - État escrow
- `POST /api/escrow/:id/release` - Libérer fonds (buyer + vendor)
- `POST /api/escrow/:id/dispute` - Ouvrir dispute (arbitre)

#### Users (Authentification)
- `POST /api/auth/register` - Inscription
- `POST /api/auth/login` - Connexion
- `GET /api/auth/whoami` - Session info
- `POST /api/auth/logout` - Déconnexion

**Sécurité API:**
- Session tokens (pas JWT - trackable)
- Cookie HttpOnly + SameSite=Strict
- CSRF protection
- Rate limiting (10 req/min par IP Tor)
- Input validation stricte

### 2.3 Stockage Persistant (Semaine 12-14)
**Délivrables:**
- [ ] Schema PostgreSQL OU SQLite (chiffré avec sqlcipher)
- [ ] Migrations avec `diesel` ou `sqlx`
- [ ] ORM/Query builder
- [ ] Backup automatique chiffré
- [ ] Indexes pour performance

**Schema Database:**
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL, -- Argon2id
    role VARCHAR(20) NOT NULL, -- buyer, vendor, arbiter
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE listings (
    id UUID PRIMARY KEY,
    vendor_id UUID REFERENCES users(id),
    title VARCHAR(200) NOT NULL,
    description TEXT NOT NULL,
    price_xmr BIGINT NOT NULL, -- atomic units
    stock INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE orders (
    id UUID PRIMARY KEY,
    buyer_id UUID REFERENCES users(id),
    vendor_id UUID REFERENCES users(id),
    listing_id UUID REFERENCES listings(id),
    escrow_id UUID UNIQUE,
    status VARCHAR(50) NOT NULL, -- pending, escrowed, shipped, completed, disputed
    total_xmr BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE escrows (
    id UUID PRIMARY KEY,
    order_id UUID REFERENCES orders(id),
    buyer_wallet_info TEXT, -- encrypted multisig info
    vendor_wallet_info TEXT,
    arbiter_wallet_info TEXT,
    multisig_address VARCHAR(95), -- Monero address
    status VARCHAR(50) NOT NULL, -- init, syncing, ready, released, disputed
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    escrow_id UUID REFERENCES escrows(id),
    tx_hash VARCHAR(64) UNIQUE,
    amount_xmr BIGINT NOT NULL,
    confirmations INT NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

**Chiffrement:**
- Passwords: Argon2id (libsodium)
- Multisig info: AES-256-GCM avec clé dérivée du master key
- Master key: Dans variable d'environnement, jamais commitée
- At-rest encryption: Disk encryption (LUKS)

---

## Phase 3: Transactions & Escrow Flow
**Durée:** 4-6 semaines
**Priorité:** HAUTE
**Objectif:** Flow escrow complet buyer → vendor → release

### 3.1 Escrow Initialization (Semaine 15-16)
**Flow:**
```
1. Buyer crée order → POST /api/orders
2. Backend crée escrow → génère escrow_id
3. Backend assigne arbiter (round-robin ou random)
4. Backend notifie buyer + vendor + arbiter (WebSocket)
5. Buyer, Vendor, Arbiter préparent leurs wallets multisig
6. Backend collecte les 3 multisig_info via API
7. Backend orchestre make_multisig pour les 3 parties
8. Backend gère 2 rounds de sync (export/import)
9. Escrow status → "ready"
10. Buyer deposit funds à multisig_address
```

**Délivrables:**
- [ ] Orchestration endpoint pour étapes 1-9
- [ ] WebSocket pour notifications temps réel
- [ ] Timeout handling (si partie ne répond pas en 10min)
- [ ] Logs détaillés (sans données sensibles)

### 3.2 Release & Dispute (Semaine 17-18)
**Flow Release Normal:**
```
1. Vendor marque order "shipped"
2. Buyer confirme réception
3. Backend crée transaction de release
4. Backend demande signatures à Buyer + Vendor
5. Backend collecte 2/3 signatures
6. Backend finalize + broadcast transaction
7. Escrow status → "released"
8. Vendor reçoit fonds (minus fees)
```

**Flow Dispute:**
```
1. Buyer OU Vendor ouvre dispute
2. Escrow status → "disputed"
3. Arbiter review les preuves (messages, photos)
4. Arbiter décide: refund buyer OU release vendor
5. Backend crée transaction selon décision
6. Backend demande signatures (Arbiter + partie gagnante)
7. Backend finalize + broadcast
8. Escrow status → "resolved"
```

**Délivrables:**
- [ ] API endpoint `/api/escrow/:id/release`
- [ ] API endpoint `/api/escrow/:id/dispute`
- [ ] Système de messaging pour disputes
- [ ] Upload de preuves (images, textes)
- [ ] Signature collection avec retry logic
- [ ] Monitoring des transactions on-chain

### 3.3 Monitoring Blockchain (Semaine 19-20)
**Délivrables:**
- [ ] Background worker pour scanner blockchain
- [ ] Détection des confirmations (10 confirmations = finalized)
- [ ] Webhook notifications pour états escrow
- [ ] Dashboard admin pour monitorer tous les escrows
- [ ] Alertes si transaction stuck (>1h sans confirmation)

---

## Phase 4: Frontend Web UI
**Durée:** 6-8 semaines
**Priorité:** MOYENNE
**Objectif:** Interface utilisateur complète et anonyme

### 4.1 Tech Stack & Architecture (Semaine 21-22)
**Décision Stack:**

**Option A: HTML/CSS/JS Vanilla (RECOMMANDÉ pour OPSEC)**
- ✅ Pas de fingerprinting framework
- ✅ Taille minimale (pas de bloat)
- ✅ Contrôle total sur le code
- ❌ Plus lent à développer

**Option B: Framework Léger (Svelte/Alpine.js)**
- ✅ Développement rapide
- ✅ Réactivité native
- ❌ Possible fingerprinting
- ❌ Dépendances externes

**Décision:** HTML/CSS/JS Vanilla + Web Components

**Délivrables:**
- [ ] Setup projet frontend/
- [ ] Build system (esbuild ou parcel)
- [ ] Routing client-side (history API)
- [ ] State management basique
- [ ] API client avec retry logic

### 4.2 Pages Core (Semaine 23-26)
**Pages Requises:**

1. **Homepage** (`/`)
   - Featured listings
   - Search bar
   - Categories
   - Stats (# vendors, # orders completed)

2. **Listings** (`/listings`)
   - Grid de produits
   - Filters (price, category, vendor)
   - Pagination

3. **Product Detail** (`/listings/:id`)
   - Photos, description, price
   - Vendor info + rating
   - "Buy" button

4. **Checkout** (`/checkout/:listing_id`)
   - Order summary
   - Shipping address (PGP encrypted)
   - Escrow initialization
   - QR code pour payment

5. **My Orders** (`/orders`)
   - Liste des achats
   - Statut en temps réel
   - Messages avec vendor

6. **Vendor Dashboard** (`/vendor/dashboard`)
   - Mes listings
   - Orders en cours
   - Analytics

7. **Escrow Tracker** (`/escrow/:id`)
   - État multisig (init, syncing, ready, released)
   - Transactions blockchain
   - Bouton "Release" ou "Dispute"

8. **Admin Panel** (`/admin`) (Arbiter only)
   - Disputes à résoudre
   - Preuves uploadées
   - Bouton décision

**Délivrables:**
- [ ] Maquettes wireframes pour chaque page
- [ ] HTML/CSS responsive
- [ ] JavaScript pour interactivité
- [ ] Intégration API REST
- [ ] WebSocket pour real-time updates

### 4.3 OPSEC Frontend (Semaine 27-28)
**Hardening Requis:**
- [ ] Pas de CDN externe (tout self-hosted)
- [ ] Pas de Google Fonts (use system fonts)
- [ ] Pas d'analytics (pas de tracking)
- [ ] Pas de social media embeds
- [ ] CSP strict (Content Security Policy)
- [ ] SRI (Subresource Integrity) pour tous les assets
- [ ] Fingerprinting resistance (uniform canvas, WebGL disabled)
- [ ] No JavaScript errors exposées (catch all)

**Content Security Policy:**
```http
Content-Security-Policy:
  default-src 'self';
  script-src 'self';
  style-src 'self';
  img-src 'self' data:;
  connect-src 'self' ws://localhost:8080;
  font-src 'self';
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self';
```

---

## Phase 5: Sécurité & Audit
**Durée:** 8-12 semaines
**Priorité:** CRITIQUE
**Objectif:** Production-ready security posture

### 5.1 Internal Security Review (Semaine 29-32)
**Délivrables:**
- [ ] Code review complet (ligne par ligne)
- [ ] Threat modeling (STRIDE framework)
- [ ] Penetration testing interne
- [ ] Fuzzing (cargo-fuzz) sur wallet/RPC code
- [ ] Static analysis (cargo-audit, clippy pedantic)
- [ ] Dependency audit (cargo-deny)
- [ ] Secrets scanning (gitleaks, trufflehog)

**Checklist Sécurité:**
- [ ] Zero `.unwrap()` ou `.expect()` sans justification
- [ ] Tous les inputs validés (length, format, range)
- [ ] Rate limiting sur TOUS les endpoints
- [ ] CSRF tokens sur toutes les mutations
- [ ] SQL injection impossible (prepared statements only)
- [ ] XSS impossible (escaping strict)
- [ ] Pas de logs de données sensibles (.onion, keys, IPs)
- [ ] Tor isolation vérifiée (pas de leaks IP)
- [ ] Monero RPC localhost uniquement (bind 127.0.0.1)

### 5.2 External Security Audit (Semaine 33-40)
**CRITIQUE: Ne PAS lancer mainnet sans audit externe**

**Scope Audit:**
1. **Smart Contract Audit** (N/A - pas de smart contracts)
2. **Cryptographic Review** (2 semaines)
   - Multisig implementation
   - Key generation & storage
   - Encryption schemes (AES-GCM)
   - Password hashing (Argon2id)

3. **Network Security** (2 semaines)
   - Tor isolation
   - DNS leaks
   - Traffic analysis resistance
   - Fingerprinting vectors

4. **Application Security** (3 semaines)
   - Authentication/Authorization
   - Input validation
   - API security
   - Database security

5. **Infrastructure Security** (1 semaine)
   - Server hardening
   - Firewall rules
   - Backup security
   - Monitoring & alerting

**Auditors Recommandés:**
- Trail of Bits (top tier, $$$$)
- Kudelski Security (cryptography experts)
- NCC Group (application security)
- Cure53 (web/browser security)
- Budget: $50k-$150k USD

**Délivrables:**
- [ ] Audit report complet
- [ ] Liste des vulnérabilités (CVSS scores)
- [ ] Recommendations de remédiation
- [ ] Re-audit après fixes

### 5.3 Bug Bounty Program (Semaine 41-42)
**Délivrables:**
- [ ] Setup HackerOne ou Bugcrowd
- [ ] Scope definition (in-scope/out-of-scope)
- [ ] Reward structure ($100-$10k selon severity)
- [ ] Response SLA (24h pour critical, 7d pour low)
- [ ] Public disclosure policy (90 days)

**Reward Tiers:**
- Critical (RCE, key theft, fund theft): $5k-$10k
- High (auth bypass, XSS, CSRF): $1k-$5k
- Medium (DoS, info disclosure): $250-$1k
- Low (best practices): $100-$250

---

## Phase 6: Production Testnet
**Durée:** 4-6 semaines
**Priorité:** HAUTE
**Objectif:** Beta testing avec utilisateurs réels (testnet XMR)

### 6.1 Infrastructure Setup (Semaine 43-44)
**Délivrables:**
- [ ] VPS provider anonyme (Njalla, 1984 Hosting)
- [ ] OS hardening (Debian 12, minimal install)
- [ ] Firewall rules (ufw: allow 80/443, deny rest)
- [ ] Tor daemon configuration
- [ ] Monero daemon + wallet RPC (testnet)
- [ ] PostgreSQL setup (avec backups)
- [ ] Nginx reverse proxy
- [ ] SSL/TLS cert (Let's Encrypt OU self-signed)
- [ ] Monitoring (Prometheus + Grafana)
- [ ] Alerting (PagerDuty OU email)
- [ ] Log aggregation (sans données sensibles)

**Server Specs Minimum:**
- CPU: 4 vCPU
- RAM: 8GB
- Disk: 200GB SSD
- Bandwidth: Unlimited
- Location: Privacy-friendly jurisdiction (Iceland, Switzerland)

### 6.2 Deployment Pipeline (Semaine 45)
**Délivrables:**
- [ ] CI/CD avec GitHub Actions OU GitLab CI
- [ ] Automated tests dans pipeline
- [ ] Security checks (cargo-audit, clippy)
- [ ] Docker containerization (optional)
- [ ] Rollback procedure
- [ ] Blue-green deployment (zero downtime)

**Pipeline Steps:**
```yaml
1. git push to main
2. Run tests (cargo test)
3. Run clippy (strict mode)
4. Run security checks
5. Build release binary (cargo build --release)
6. Upload to server via SSH
7. Stop old service
8. Start new service
9. Health check (10s timeout)
10. If fail → rollback to previous version
```

### 6.3 Beta Testing (Semaine 46-48)
**Délivrables:**
- [ ] Invite 20-50 beta testers (Tor community, Monero community)
- [ ] Provide testnet XMR faucet
- [ ] Collect feedback (Google Forms OU Typeform via Tor)
- [ ] Monitor for bugs (Sentry error tracking)
- [ ] Fix critical bugs in <24h
- [ ] Iterate on UX issues

**Success Metrics:**
- 50+ completed escrows on testnet
- <5% error rate on transactions
- Average escrow completion time <30 minutes
- Zero security incidents
- User satisfaction score >4/5

---

## Phase 7: Mainnet Launch (SI AUDIT OK)
**Durée:** Variable (après audit OK)
**Priorité:** CRITIQUE
**Objectif:** Production mainnet avec monitoring 24/7

### 7.1 Pre-Launch Checklist
**TOUTES ces conditions doivent être remplies:**
- [ ] Audit externe complété avec tous les criticals fixés
- [ ] Bug bounty actif depuis 4+ semaines sans critical
- [ ] Testnet beta testing complété avec succès
- [ ] Backups testés et validés (restore OK)
- [ ] Monitoring & alerting fonctionnels
- [ ] Incident response plan documenté
- [ ] Legal compliance vérifiée (selon juridiction)
- [ ] Team disponible 24/7 (au moins 2 semaines post-launch)

### 7.2 Launch Strategy
**Phase de Lancement Progressif:**

**Week 1-2: Soft Launch**
- Invite-only (10-20 trusted users)
- Limits: Max 0.1 XMR par escrow
- Manual approval de nouveaux vendors
- 24/7 monitoring avec alertes

**Week 3-4: Limited Public**
- Open registration (avec captcha)
- Limits: Max 0.5 XMR par escrow
- Max 10 active escrows par user
- Review de tous les listings avant publication

**Week 5+: Full Launch**
- Remove invite requirement
- Increase limits: Max 5 XMR par escrow (configurable)
- Auto-approval vendors (avec reputation system)
- Publicité dans communautés Tor/Monero

### 7.3 Post-Launch Support
**Délivrables:**
- [ ] Support ticket system (email via PGP OU Tor form)
- [ ] FAQ / Knowledge base
- [ ] Video tutorials (hosted on onion)
- [ ] Community forum (optionnel)
- [ ] Regular security updates (monthly)
- [ ] Incident response team (24/7 on-call)

---

## Risques & Mitigations

### Risques Techniques
| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|------------|
| Vulnérabilité critique trouvée post-launch | Moyenne | Très Haut | Bug bounty, audits réguliers |
| Monero RPC instable/crash | Moyenne | Haut | Health checks, auto-restart, failover |
| Tor network down/censured | Faible | Haut | Bridges, fallback nodes |
| Database corruption | Faible | Très Haut | Backups quotidiens, réplication |
| DDoS sur hidden service | Moyenne | Moyen | Rate limiting, Tor PoW, backup .onion |

### Risques Légaux
| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|------------|
| Saisie des serveurs | Faible-Moyenne | Très Haut | Encryption at-rest, pas de KYC, logs minimaux |
| Responsabilité pour contenus illégaux | Moyenne | Haut | Terms of Service, modération, reporting |
| Contrainte juridique (backdoor) | Faible | Très Haut | Canary, open-source, multi-juridiction |

### Risques Business
| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|------------|
| Pas assez d'utilisateurs | Moyenne | Haut | Marketing Tor/Monero communities |
| Vendor scams (réputation) | Moyenne | Moyen | Reputation system, arbiters |
| Competitors (autres marketplaces) | Élevée | Moyen | Differentiation (meilleure OPSEC, meilleur UX) |

---

## Métriques de Succès

### Phase 1-3 (Testnet)
- [ ] 100% tests passing
- [ ] Code coverage >80%
- [ ] Zero security theatre détecté
- [ ] Tous les Reality Checks validés
- [ ] <5% error rate sur transactions

### Phase 4-6 (Beta)
- [ ] 50+ beta testers
- [ ] 100+ completed escrows on testnet
- [ ] User satisfaction >4/5
- [ ] Zero security incidents
- [ ] Audit report avec <5 mediums, 0 criticals

### Phase 7 (Mainnet)
- [ ] 500+ registered users (mois 1)
- [ ] 100+ completed escrows (mois 1)
- [ ] Uptime >99.5%
- [ ] Average response time <500ms
- [ ] Zero fund loss incidents
- [ ] Active bug bounty avec payouts

---

## Budget Estimation

| Phase | Description | Durée | Coût (si freelance) |
|-------|-------------|-------|---------------------|
| 1 | Multisig Core | 6 semaines | $15k-$25k |
| 2 | Backend API | 8 semaines | $25k-$40k |
| 3 | Escrow Flow | 6 semaines | $20k-$30k |
| 4 | Frontend UI | 8 semaines | $20k-$35k |
| 5 | Security Audit | 10 semaines | $50k-$150k (externe) |
| 6 | Testnet Beta | 6 semaines | $10k-$20k |
| 7 | Mainnet Infra | Ongoing | $200-$500/mois (VPS) |

**Total Développement:** $140k-$300k
**Infra Annuelle:** $2.4k-$6k
**Budget Total (An 1):** $142k-$306k

**Note:** Ces chiffres sont pour un développeur expérimenté à temps plein. Projet open-source bénévole = gratuit mais plus lent.

---

## Timeline Visuel

```
Mois 1-2:  [████████████] Phase 1: Multisig Core
Mois 3-4:  [████████████████████] Phase 2: Backend API
Mois 5-6:  [████████████] Phase 3: Escrow Flow
Mois 6-8:  [████████████████████] Phase 4: Frontend UI
Mois 8-11: [████████████████████████████] Phase 5: Security Audit
Mois 11:   [██████] Phase 6: Testnet Beta
Mois 12+:  [██] Phase 7: Mainnet Launch
```

---

## Prochaines Actions Immédiates

### Cette Semaine
1. ✅ Fixer problème de compilation Windows (PowerShell ou WSL2)
2. [ ] Setup 3 wallets testnet pour tests multisig
3. [ ] Créer test end-to-end pour workflow multisig complet
4. [ ] Valider Reality Checks pour prepare, make, export, import

### Semaine Prochaine
1. [ ] Implémenter `create_transaction()` pour multisig
2. [ ] Créer spec pour `sign_transaction()`
3. [ ] Setup CI/CD pipeline basique (GitHub Actions)
4. [ ] Commencer documentation architecture backend

### Mois 1 Goal
- [ ] Phase 1 complétée (Multisig Core fonctionnel)
- [ ] 10+ tests end-to-end passing
- [ ] Documentation à jour
- [ ] Reality Checks tous validés

---

## Ressources & Références

### Documentation Technique
- [Monero RPC Documentation](https://www.getmonero.org/resources/developer-guides/wallet-rpc.html)
- [Tor Hidden Service Guide](https://community.torproject.org/onion-services/)
- [Rust Security Best Practices](https://anssi-fr.github.io/rust-guide/)

### Outils Sécurité
- `cargo-audit` - Dependency vulnerability scanner
- `cargo-deny` - Dependency policy enforcement
- `cargo-fuzz` - Fuzzing framework
- `gitleaks` - Secrets scanning
- `semgrep` - Static analysis

### Communautés
- r/Monero (Reddit)
- r/onions (Reddit)
- Monero IRC (#monero-dev)
- Tor Project Mailing Lists

---

## Conclusion

Cette feuille de route représente **8-11 mois de développement intensif** pour atteindre production mainnet avec un niveau de sécurité acceptable.

**Les phases 1-3 (Multisig + Backend)** sont les fondations critiques et doivent être parfaites.

**La Phase 5 (Audit)** est NON-NÉGOCIABLE - ne jamais lancer mainnet sans audit externe par des experts reconnus.

**La prudence est clé:** Mieux vaut retarder le launch de 3 mois que de perdre des fonds utilisateur.

---

**Statut:** 📋 Planification approuvée
**Prochaine Révision:** Après Phase 1 completion
**Contact:** (À définir)
