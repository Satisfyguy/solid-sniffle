# Module Custodial - Monero Marketplace

## Vue d'Ensemble

Ce dossier contient toute la documentation et les audits liés au **module custodial** (gardien/arbitre) du système d'escrow 2-of-3 multisig.

## Statut du Module

**⚠️ EN DÉVELOPPEMENT** - Le module custodial n'est PAS encore implémenté dans le code.

L'architecture actuelle est **NON-CUSTODIAL** avec escrow 2-of-3 multisig où:
- Acheteur possède 1 clé
- Vendeur possède 1 clé
- Marketplace possède 1 clé (arbitrage uniquement en cas de dispute)

## Contenu du Dossier

### 📄 Documents Principaux

1. **[CUSTODIAL-AUDIT-2025-10-22.md](CUSTODIAL-AUDIT-2025-10-22.md)** (64 KB)
   - Audit complet du concept de module custodial
   - Analyse de sécurité détaillée
   - Risques et mitigations
   - Recommandations d'implémentation
   - Date: 22 octobre 2025

2. **[non_custodial_migration.md](non_custodial_migration.md)** (28 KB)
   - Spécification de migration vers architecture non-custodiale
   - Plan de transition
   - Impact sur l'escrow existant
   - Stratégies de déploiement

### 📁 Structure

```
custodial/
├── README.md                           # Ce fichier
├── CUSTODIAL-AUDIT-2025-10-22.md      # Audit principal
├── non_custodial_migration.md         # Spec migration
├── docs/                               # Documentation supplémentaire
└── audits/                             # Audits de sécurité
```

## Qu'est-ce qu'un Module Custodial?

Un **module custodial** (ou "gardien") est un composant de sécurité qui:

### Rôle Principal
- **Arbitrage en cas de dispute** entre acheteur et vendeur
- **Protection contre la fraude** des deux côtés
- **Résolution de conflits** selon des règles prédéfinies

### Fonctionnalités Clés

1. **Gestion des Clés Multisig**
   - Détient 1 des 3 clés du portefeuille escrow
   - Ne peut pas libérer les fonds seul (nécessite 2/3 signatures)
   - Clé stockée de manière sécurisée (HSM recommandé)

2. **Arbitrage**
   - Analyse des preuves soumises par acheteur/vendeur
   - Décision basée sur les termes du contrat
   - Signature de transaction pour débloquer escrow

3. **Audit Trail**
   - Journalisation de toutes les actions
   - Traçabilité des décisions d'arbitrage
   - Conformité réglementaire

## Architecture Actuelle vs. Custodial

### Architecture Actuelle (Implémentée)

```
┌─────────────────────────────────────────────┐
│         ESCROW 2-OF-3 MULTISIG              │
├─────────────────────────────────────────────┤
│                                             │
│  Acheteur (1 clé) ──┐                      │
│                     │                       │
│  Vendeur (1 clé) ───┼──> Portefeuille XMR  │
│                     │                       │
│  Marketplace (1 clé)─┘   (2/3 requis)      │
│                                             │
└─────────────────────────────────────────────┘

Flux normal:
1. Acheteur + Vendeur signent → Fonds libérés
2. Si dispute: Marketplace + (Acheteur OU Vendeur) → Arbitrage
```

### Architecture avec Module Custodial (Planifiée)

```
┌─────────────────────────────────────────────┐
│     MODULE CUSTODIAL (Marketplace)          │
├─────────────────────────────────────────────┤
│                                             │
│  ┌─────────────────────────────┐            │
│  │   Gestionnaire de Clés      │            │
│  │   - HSM (Hardware Security) │            │
│  │   - Backup encrypted        │            │
│  └─────────────────────────────┘            │
│                                             │
│  ┌─────────────────────────────┐            │
│  │   Moteur d'Arbitrage        │            │
│  │   - Analyse preuves         │            │
│  │   - Règles automatiques     │            │
│  │   - ML fraud detection      │            │
│  └─────────────────────────────┘            │
│                                             │
│  ┌─────────────────────────────┐            │
│  │   Audit & Compliance        │            │
│  │   - Logs immuables          │            │
│  │   - Reporting               │            │
│  └─────────────────────────────┘            │
│                                             │
└─────────────────────────────────────────────┘
```

## Risques de Sécurité (Identifiés dans l'Audit)

### 🔴 Risques Critiques

1. **Single Point of Failure**
   - Si module custodial compromis → Tous les escrows à risque
   - Mitigation: HSM, cold storage, multi-sig interne

2. **Centralisation**
   - Contrôle par marketplace = risque de censure
   - Mitigation: Gouvernance décentralisée, DAO

3. **Responsabilité Légale**
   - Détention de clés = statut de "custodian" réglementé
   - Mitigation: Analyse juridique, compliance KYC/AML

### 🟡 Risques Modérés

4. **Attaques par Collusion**
   - Marketplace + Acheteur/Vendeur malveillant
   - Mitigation: Time-locks, proof of dispute

5. **Disponibilité**
   - Si module offline → Escrows bloqués
   - Mitigation: Haute disponibilité, fallback keys

## État d'Implémentation

### ✅ Composants Implémentés

- [x] Escrow 2-of-3 multisig (wallet/multisig.rs)
- [x] Gestion des transactions Monero (wallet/client.rs)
- [x] Base de données escrow (server/src/db/escrow.rs)
- [x] API REST pour escrow (server/src/handlers/escrow.rs)
- [x] Tests E2E escrow (server/tests/escrow_e2e.rs)

### ❌ Composants NON Implémentés (Custodial)

- [ ] Module custodial dédié
- [ ] HSM pour stockage clés
- [ ] Moteur d'arbitrage automatique
- [ ] Interface de dispute resolution
- [ ] Système de preuves cryptographiques
- [ ] Audit trail immuable
- [ ] Compliance & reporting

## Prochaines Étapes

Si implémentation du module custodial:

### Phase 1: Architecture (1-2 semaines)
1. Finaliser spec technique
2. Choix HSM (Ledger, Trezor, AWS CloudHSM)
3. Design API custodial
4. Threat modeling

### Phase 2: Sécurité (2-3 semaines)
5. Implémentation HSM integration
6. Multi-sig interne pour clés marketplace
7. Cold storage backup
8. Disaster recovery plan

### Phase 3: Arbitrage (3-4 semaines)
9. Moteur de règles d'arbitrage
10. Interface admin pour disputes
11. Système de preuves (photos, messages, tracking)
12. ML fraud detection (optionnel)

### Phase 4: Compliance (2-3 semaines)
13. Audit logging immuable (blockchain)
14. Reporting automatique
15. KYC/AML si requis légalement
16. Audit externe de sécurité

### Phase 5: Tests & Déploiement (3-4 semaines)
17. Tests unitaires + intégration
18. Testnet deployment
19. Bug bounty program
20. Mainnet gradual rollout

**Estimation totale:** 11-16 semaines (3-4 mois)

## Dépendances Techniques

### Crates Rust Requis

```toml
# Cargo.toml additions pour module custodial
[dependencies]
# HSM/Signing
ledger-transport-hid = "0.10"
trezor-client = "0.1"
aws-sdk-cloudhsm = "1.0"  # Si AWS HSM

# Encryption
age = "0.10"              # Backup encryption
sodiumoxide = "0.2"       # Key derivation

# Arbitrage
serde_json = "1.0"
chrono = "0.4"

# Audit logging
merkle-tree = "0.2"       # Merkle proofs
sha3 = "0.10"

# Compliance
reqwest = "0.11"          # API calls (KYC providers)
```

### Infrastructure

- **HSM**: Ledger Nano X, Trezor Model T, ou AWS CloudHSM
- **Database**: PostgreSQL avec audit trail (ou blockchain append-only)
- **Monitoring**: Prometheus + Grafana pour alertes
- **Backup**: S3-compatible storage avec encryption

## Références

### Documents Internes
- [CUSTODIAL-AUDIT-2025-10-22.md](CUSTODIAL-AUDIT-2025-10-22.md) - Audit complet
- [non_custodial_migration.md](non_custodial_migration.md) - Migration vers non-custodial
- [../docs/ESCROW-SPEC.md](../docs/ESCROW-SPEC.md) - Spec escrow actuelle

### Standards & Best Practices
- **Monero Multisig**: https://www.getmonero.org/resources/user-guides/multisig-messaging-system.html
- **HSM Best Practices**: NIST SP 800-57
- **Custodial Wallet Security**: https://github.com/bitcoin-core/secp256k1

### Réglementation (à vérifier selon juridiction)
- **USA**: FinCEN guidance on custodial wallets
- **EU**: MiCA (Markets in Crypto-Assets Regulation)
- **Général**: FATF Travel Rule for crypto custodians

## Auteurs & Contributions

- **Audit Initial**: Gemini/Claude (22 Oct 2025)
- **Spec Migration**: Équipe technique
- **Maintenance**: À définir

## License

Ce module fait partie du projet Monero Marketplace.
Tous les documents sont fournis à des fins éducatives uniquement.

**⚠️ AVERTISSEMENT**: L'implémentation d'un module custodial a des implications légales.
Consulter un avocat spécialisé en crypto avant déploiement en production.

---

**Dernière mise à jour:** 23 octobre 2025
**Statut:** Documentation uniquement - Pas de code implémenté
