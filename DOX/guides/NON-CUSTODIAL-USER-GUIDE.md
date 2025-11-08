# Guide Utilisateur Non-Custodial
## Monero Marketplace - Phase 2 Migration

**Date:** 2025-11-08
**Status:** ✅ Phase 2 Implémentée
**Architecture:** 100% Non-Custodial (Haveno-style)

---

## Table des Matières

1. [Introduction](#introduction)
2. [Prérequis](#prérequis)
3. [Installation](#installation)
4. [Démarrage Rapide](#démarrage-rapide)
5. [Guide Détaillé](#guide-détaillé)
6. [Cas d'Usage](#cas-dusage)
7. [Dépannage](#dépannage)
8. [FAQ](#faq)

---

## Introduction

### Qu'est-ce que le Mode Non-Custodial?

Dans le mode **non-custodial**, vous gardez **100% du contrôle** sur vos fonds Monero:

- ✅ **Vous** créez votre wallet localement (pas le serveur)
- ✅ **Vous** exécutez votre propre `monero-wallet-rpc`
- ✅ **Vous** détenez vos clés privées
- ✅ Le serveur **coordonne uniquement** l'échange d'informations publiques

**Le serveur ne touche JAMAIS à vos clés privées.**

### Architecture Non-Custodiale

```
┌─────────────────────────────────────────────────────────────┐
│ VOUS (Client Local)                                         │
│                                                              │
│  ┌──────────────────────────────────────┐                  │
│  │ Votre monero-wallet-rpc (LOCAL)      │                  │
│  │ Port: 18083                           │                  │
│  │ Clés privées: UNIQUEMENT chez vous   │                  │
│  └──────────────────────────────────────┘                  │
│                     ↓                                        │
│  ┌──────────────────────────────────────┐                  │
│  │ CLI Non-Custodial                     │                  │
│  │ cargo run --bin monero-marketplace   │                  │
│  │ noncustodial init-escrow              │                  │
│  └──────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
                            ↓
                     Envoie uniquement
                    multisig_info (public)
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ SERVEUR (Coordinateur UNIQUEMENT)                          │
│                                                              │
│  ┌──────────────────────────────────────┐                  │
│  │ EscrowCoordinator                     │                  │
│  │ - Stocke RPC URLs                     │                  │
│  │ - Coordonne échange infos             │                  │
│  │ - Valide threshold=2, participants=3  │                  │
│  │ - NE TOUCHE JAMAIS aux wallets        │                  │
│  └──────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
                            ↓
                    Distribue infos aux
                     autres participants
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ AUTRES PARTICIPANTS (Seller, Arbiter)                      │
│                                                              │
│  Chacun avec son propre monero-wallet-rpc LOCAL            │
│  Chacun avec ses propres clés privées                      │
└─────────────────────────────────────────────────────────────┘
```

---

## Prérequis

### 1. Installer Monero CLI

Téléchargez Monero CLI depuis le site officiel:

```bash
# Option 1: Téléchargement manuel
wget https://downloads.getmonero.org/cli/monero-linux-x64-v0.18.3.1.tar.bz2
tar -xjf monero-linux-x64-v0.18.3.1.tar.bz2
cd monero-x86_64-linux-gnu-v0.18.3.1

# Option 2: Package manager (Ubuntu)
sudo apt update
sudo apt install monero
```

### 2. Vérifier l'Installation

```bash
monero-wallet-rpc --version
# Devrait afficher: Monero 'Fluorine Fermi' (v0.18.3.1-release)
```

### 3. Cloner le Projet Monero Marketplace

```bash
git clone https://github.com/Satisfyguy/solid-sniffle
cd solid-sniffle
```

### 4. Compiler le CLI

```bash
cargo build --release --package monero-marketplace-cli
```

---

## Installation

### Configuration de l'Environnement

1. **Créer les répertoires de wallets:**

```bash
mkdir -p ~/.monero/testnet/wallets/{buyer,seller,arbiter}
```

2. **Lancer le serveur coordinator:**

```bash
# Terminal 1: Serveur
cd solid-sniffle
cargo run --release --package server --bin server
```

Le serveur démarre sur `http://localhost:8080`.

---

## Démarrage Rapide

### Scénario: 3 Participants (Buyer, Seller, Arbiter)

Chaque participant doit:
1. Lancer son propre `monero-wallet-rpc`
2. Utiliser le CLI pour initialiser l'escrow

#### **Participant 1: Buyer**

```bash
# Terminal 2: Buyer wallet RPC
monero-wallet-rpc \
  --testnet \
  --rpc-bind-port 18083 \
  --disable-rpc-login \
  --wallet-dir ~/.monero/testnet/wallets/buyer \
  --offline

# Terminal 3: Buyer CLI
cargo run --release --bin monero-marketplace -- noncustodial init-escrow \
  --escrow-id "escrow_test_001" \
  --role buyer \
  --wallet-name "buyer_wallet" \
  --local-rpc-url "http://127.0.0.1:18083" \
  --server-url "http://localhost:8080"
```

#### **Participant 2: Seller**

```bash
# Terminal 4: Seller wallet RPC
monero-wallet-rpc \
  --testnet \
  --rpc-bind-port 18084 \
  --disable-rpc-login \
  --wallet-dir ~/.monero/testnet/wallets/seller \
  --offline

# Terminal 5: Seller CLI
cargo run --release --bin monero-marketplace -- noncustodial init-escrow \
  --escrow-id "escrow_test_001" \
  --role seller \
  --wallet-name "seller_wallet" \
  --local-rpc-url "http://127.0.0.1:18084" \
  --server-url "http://localhost:8080"
```

#### **Participant 3: Arbiter**

```bash
# Terminal 6: Arbiter wallet RPC
monero-wallet-rpc \
  --testnet \
  --rpc-bind-port 18085 \
  --disable-rpc-login \
  --wallet-dir ~/.monero/testnet/wallets/arbiter \
  --offline

# Terminal 7: Arbiter CLI
cargo run --release --bin monero-marketplace -- noncustodial init-escrow \
  --escrow-id "escrow_test_001" \
  --role arbiter \
  --wallet-name "arbiter_wallet" \
  --local-rpc-url "http://127.0.0.1:18085" \
  --server-url "http://localhost:8080"
```

### Résultat Attendu

Après que les 3 participants aient exécuté la commande:

```
✅ Non-custodial escrow initialized successfully!
Multisig address: 5AYxY... (adresse multisig 2-of-3)
```

**Tous les participants voient la MÊME adresse multisig**, mais:
- ✅ Chacun conserve ses clés privées localement
- ✅ Le serveur n'a JAMAIS vu les clés
- ✅ 2 signatures sur 3 sont requises pour toute transaction

---

## Guide Détaillé

### Étape 1: Lancer Votre Wallet RPC Local

#### Pourquoi?
Votre wallet RPC local est **votre coffre-fort personnel**. Le serveur ne peut pas y accéder directement.

#### Comment?

```bash
monero-wallet-rpc \
  --testnet \                           # Utiliser testnet pour tests
  --rpc-bind-port 18083 \               # Port de votre choix (18083-18099)
  --disable-rpc-login \                 # Pas de login pour local
  --wallet-dir ~/.monero/testnet/wallets/buyer \  # Votre répertoire
  --offline                             # Mode hors ligne (pas besoin de daemon)
```

**Options importantes:**
- `--testnet`: Utilisez testnet pour tests (mainnet pour production)
- `--rpc-bind-port`: Port unique pour chaque participant
- `--offline`: Permet de travailler sans daemon Monero

#### Vérification

Testez la connexion RPC:

```bash
curl -X POST http://127.0.0.1:18083/json_rpc -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}'
```

**Réponse attendue:**
```json
{"id":"0","jsonrpc":"2.0","result":{"version":196613}}
```

---

### Étape 2: Initialiser l'Escrow Non-Custodial

#### Commande Complète

```bash
cargo run --release --bin monero-marketplace -- \
  noncustodial init-escrow \
  --escrow-id "escrow_abc123" \
  --role buyer \
  --wallet-name "my_buyer_wallet" \
  --local-rpc-url "http://127.0.0.1:18083" \
  --server-url "http://localhost:8080"
```

#### Paramètres

| Paramètre | Description | Exemple |
|-----------|-------------|---------|
| `--escrow-id` | Identifiant unique de l'escrow | `escrow_abc123` |
| `--role` | Votre rôle (**buyer**, **seller**, ou **arbiter**) | `buyer` |
| `--wallet-name` | Nom du wallet à créer localement | `my_buyer_wallet` |
| `--local-rpc-url` | URL de votre wallet RPC local | `http://127.0.0.1:18083` |
| `--server-url` | URL du serveur coordinator | `http://localhost:8080` |

#### Flow Détaillé

Le CLI exécute automatiquement ces étapes:

1. **Création du wallet local**
   ```
   📁 Creating local wallet 'my_buyer_wallet'...
   ✅ Wallet 'my_buyer_wallet' created
   ```

2. **Préparation multisig**
   ```
   📝 Preparing multisig locally...
   ✅ Local multisig prepared
   Multisig info length: 327 chars
   ```

3. **Enregistrement avec le serveur**
   ```
   📡 Registering with server coordinator...
   ✅ Registered as buyer for escrow escrow_abc123
   State: AwaitingRegistrations
   Waiting for: ["seller", "arbiter"]
   ```

4. **Attente des autres participants**
   ```
   ⏳ Waiting for other participants to register...
   Waiting for participants: ["seller", "arbiter"] (attempt 1/60)
   Waiting for participants: ["arbiter"] (attempt 5/60)
   ✅ All participants registered!
   ```

5. **Coordination de l'échange**
   ```
   🔄 Coordinating multisig info exchange...
   ✅ Coordination successful
   Received 2 multisig infos from other participants
   ```

6. **Finalisation locale**
   ```
   🔧 Finalizing multisig locally (make_multisig with threshold=2)...
   ✅ Multisig wallet created locally!
   Multisig address: 5AYxY...
   ```

---

## Cas d'Usage

### Cas 1: Transaction d'Achat Simple

**Scénario:** Bob (buyer) achète un produit de Alice (seller) avec arbitrage de Charlie.

**Flow:**

1. **Tous lancent leur wallet RPC:**
   - Bob: port 18083
   - Alice: port 18084
   - Charlie: port 18085

2. **Tous initialisent l'escrow:**
   ```bash
   # Même escrow_id pour tous: "buy_laptop_001"
   # Bob:    --role buyer
   # Alice:  --role seller
   # Charlie: --role arbiter
   ```

3. **Multisig créé:**
   - Adresse multisig commune: `5AYxY...`
   - Bob peut voir l'adresse dans son wallet local
   - Alice et Charlie voient la même

4. **Bob envoie les fonds:**
   ```bash
   # Bob utilise son wallet local pour envoyer à l'adresse multisig
   ```

5. **Libération des fonds (2-of-3):**
   - Si transaction OK: Bob + Alice signent (libération à Alice)
   - Si dispute: Charlie + Bob ou Charlie + Alice signent

---

### Cas 2: Vérifier l'État de Votre Wallet

```bash
cargo run --release --bin monero-marketplace -- \
  noncustodial wallet-info \
  --local-rpc-url "http://127.0.0.1:18083" \
  --role buyer \
  --server-url "http://localhost:8080"
```

**Sortie:**
```
Getting wallet info for buyer at http://127.0.0.1:18083
📊 Wallet Information:
  Multisig: true
  Threshold: 2/3
  Balance: 0.0 XMR
  Block Height: 2500000
```

---

## Dépannage

### Erreur: "Failed to connect to local RPC"

**Cause:** Votre wallet RPC n'est pas démarré ou port incorrect.

**Solution:**
```bash
# Vérifier que le RPC tourne
ps aux | grep monero-wallet-rpc

# Relancer le RPC
monero-wallet-rpc --testnet --rpc-bind-port 18083 --disable-rpc-login --offline
```

---

### Erreur: "Timeout waiting for participants"

**Cause:** Les autres participants n'ont pas encore lancé leur commande `init-escrow`.

**Solution:**
- Vérifier que TOUS les participants utilisent le **même escrow_id**
- Vérifier que chaque participant a un **rôle unique** (buyer, seller, arbiter)
- Augmenter le timeout (2 minutes par défaut)

---

### Erreur: "Invalid number of multisig infos"

**Cause:** Coordination échouée, infos manquantes.

**Solution:**
```bash
# Relancer depuis le début
# 1. Fermer tous les wallets RPC
killall monero-wallet-rpc

# 2. Supprimer les wallets créés
rm -rf ~/.monero/testnet/wallets/*/{buyer,seller,arbiter}_wallet*

# 3. Relancer le flow complet
```

---

### Wallet Déjà Créé

**Message:**
```
Wallet 'my_wallet' already exists, will use existing
```

**Cause:** Le wallet existe déjà localement.

**Solution:**
- C'est **normal** si vous relancez avec le même `--wallet-name`
- Pour un nouveau wallet, utilisez un nom différent:
  ```bash
  --wallet-name "buyer_wallet_2"
  ```

---

## FAQ

### Q: Le serveur peut-il voler mes fonds?

**R:** **NON.** Le serveur:
- Ne crée JAMAIS de wallets
- Ne stocke JAMAIS de clés privées
- Ne peut JAMAIS signer de transactions
- Coordonne uniquement l'échange d'infos **publiques** (multisig_info)

---

### Q: Puis-je utiliser un wallet existant?

**R:** Oui, mais le wallet doit être **vide** ou **nouveau** car `prepare_multisig` convertit le wallet en mode multisig.

**ATTENTION:** Une fois converti en multisig, le wallet **ne peut pas revenir** en mode normal.

---

### Q: Combien de temps prend l'initialisation?

**R:**
- **Avec 3 participants prêts:** ~10-15 secondes
- **En attendant les autres:** jusqu'à 2 minutes (timeout)

---

### Q: Puis-je utiliser le mode non-custodial sur mainnet?

**R:** Oui, mais:
1. **Testez d'abord sur testnet** (utilisez `--testnet`)
2. Sur mainnet, retirez `--testnet` et utilisez `--mainnet`
3. Utilisez un daemon synchronisé (`--daemon-address`)

**Exemple mainnet:**
```bash
monero-wallet-rpc \
  --rpc-bind-port 18083 \
  --disable-rpc-login \
  --wallet-dir ~/.monero/mainnet/wallets/buyer \
  --daemon-address node.moneroworld.com:18089
```

---

### Q: Que faire si je perds mon wallet local?

**R:** **CRITIQUE:** Sauvegardez vos seeds!

Sans backup:
- ❌ Vous **perdez** l'accès à vos fonds multisig
- ❌ Les autres participants **ne peuvent pas** récupérer vos clés

**Best practice:**
```bash
# Sauvegarder le seed après création
monero-wallet-cli --wallet-file my_buyer_wallet --testnet
> seed
# Écrire le seed (25 mots) dans un endroit sûr!
```

---

### Q: Puis-je utiliser le CLI depuis un serveur distant?

**R:** **NON RECOMMANDÉ** pour des raisons de sécurité.

Le wallet RPC doit tourner sur **127.0.0.1** (localhost strict) pour éviter les expositions réseau.

Si vous devez absolument:
- Utilisez SSH tunneling
- Utilisez VPN
- N'exposez JAMAIS le port RPC publiquement

---

## Commandes Utiles

### Lister les Wallets Créés

```bash
ls -la ~/.monero/testnet/wallets/buyer/
```

### Vérifier les Logs du Serveur

```bash
# Voir les logs du coordinator
tail -f /var/log/monero-marketplace/server.log | grep "EscrowCoordinator"
```

### Tester la Connexion Serveur

```bash
curl http://localhost:8080/health
# Devrait retourner: {"status":"ok"}
```

---

## Prochaines Étapes

Après avoir initialisé votre escrow non-custodial:

1. **Synchronisation multisig (2 rounds):**
   - Export/import multisig_info entre participants
   - Requis avant de pouvoir effectuer des transactions

2. **Envoyer des fonds à l'adresse multisig**

3. **Créer et signer des transactions multisig**

Pour les étapes avancées, consultez:
- [Guide Multisig Complet](../architecture/MONERO-MULTISIG-2OF3-COMPLETE-GUIDE.md)
- [Implémentation Sync Multisig](../architecture/MULTISIG-SYNC-IMPLEMENTATION.md)

---

## Support

**Problèmes techniques:**
- GitHub Issues: https://github.com/Satisfyguy/solid-sniffle/issues
- Documentation: `/DOX/guides/`

**Sécurité:**
- Signalez les vulnérabilités via email privé (ne pas créer d'issue publique)

---

**Dernière mise à jour:** 2025-11-08
**Version du guide:** Phase 2 - v1.0
**Architecture:** 100% Non-Custodial ✅
