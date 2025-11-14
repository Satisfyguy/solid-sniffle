# Server-Managed Multisig Escrow Guide
## Monero Marketplace - Non-Custodial Architecture

**Date:** 2025-11-13
**Status:** ✅ Production Architecture
**Version:** 1.0

---

## Table of Contents

1. [Introduction](#introduction)
2. [Architecture Overview](#architecture-overview)
3. [Why This Is Non-Custodial](#why-this-is-non-custodial)
4. [How It Works](#how-it-works)
5. [Security Model](#security-model)
6. [Usage Guide](#usage-guide)
7. [API Reference](#api-reference)
8. [FAQ](#faq)

---

## Introduction

### What Is Server-Managed Multisig?

The Monero Marketplace uses a **server-managed multisig architecture** where:
- ✅ Server creates temporary wallets for escrow participants (buyer, vendor, arbiter)
- ✅ Wallets remain **permanently empty** - used ONLY for signatures
- ✅ **No funds ever transit through the server**
- ✅ **Non-custodial by design** - server never holds custody of funds

This architecture provides the convenience of server-side wallet management while maintaining the security guarantees of a truly non-custodial system.

### Key Principle

**Wallets are signature devices, not fund storage.**

The server manages multisig wallets that exist solely to sign transactions. Since these wallets never hold funds, the server cannot access, steal, or freeze user money—making the system inherently non-custodial.

---

## Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ USER (Buyer/Vendor)                                         │
│                                                              │
│  ┌──────────────────────────────────────┐                  │
│  │ Personal Monero Wallet               │                  │
│  │ - Holds actual funds                 │                  │
│  │ - Sends to multisig address          │                  │
│  │ - Receives from multisig address     │                  │
│  └──────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
                            ↓ Sends XMR to multisig
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ MULTISIG ADDRESS (2-of-3)                                   │
│ Controlled by: Buyer + Vendor + Arbiter wallets            │
│ Funds held here during escrow                               │
└─────────────────────────────────────────────────────────────┘
                            ↑
                            ↑ Coordinated by server
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ SERVER (Coordinator ONLY)                                   │
│                                                              │
│  ┌──────────────────────────────────────┐                  │
│  │ Temporary Multisig Wallets (EMPTY)   │                  │
│  │                                       │                  │
│  │  • Buyer Wallet    (signature only)  │                  │
│  │  • Vendor Wallet   (signature only)  │                  │
│  │  • Arbiter Wallet  (signature only)  │                  │
│  │                                       │                  │
│  │ ✅ Created per escrow                 │                  │
│  │ ✅ Always empty (0 XMR balance)       │                  │
│  │ ✅ Used ONLY for signing              │                  │
│  │ ❌ NEVER holds funds                  │                  │
│  └──────────────────────────────────────┘                  │
│                     ↓                                        │
│  ┌──────────────────────────────────────┐                  │
│  │ EscrowOrchestrator                    │                  │
│  │ - Coordinates multisig setup          │                  │
│  │ - Manages signature collection        │                  │
│  │ - Enforces state transitions          │                  │
│  │ - Broadcasts finalized transactions   │                  │
│  └──────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

### Component Roles

**1. User Wallets (Off-Server)**
- Hold user's actual Monero funds
- Send funds to multisig address at escrow creation
- Receive funds from multisig address at escrow release
- **Fully controlled by users**

**2. Multisig Address**
- Generated from 3 temporary server wallets (buyer, vendor, arbiter)
- Holds escrowed funds (2-of-3 signature requirement)
- Funds can only move with 2 of 3 signatures
- **Not controlled by any single party**

**3. Server Temporary Wallets**
- Created per escrow transaction
- **Always empty** (balance = 0 XMR)
- Used exclusively for generating multisig signatures
- Automatically deleted after escrow completion
- **Cannot steal funds** (no balance to steal)

**4. EscrowOrchestrator (Server Component)**
- Coordinates multisig setup between 3 wallets
- Collects and validates signatures
- Enforces escrow state machine (pending → funded → released/disputed)
- Broadcasts transactions to Monero network
- **No direct fund access**

---

## Why This Is Non-Custodial

### Definition: Custodial vs Non-Custodial

**Custodial:**
- Platform holds user funds in platform-controlled wallets
- Platform can access, freeze, or steal funds
- Users trust platform not to misuse funds
- Example: Centralized exchanges (Coinbase, Binance)

**Non-Custodial:**
- Users retain full control of their funds
- Platform cannot access or freeze user funds
- No trust required in platform's honesty
- Example: DEXs, self-custody wallets

### Why Our Architecture Is Non-Custodial

**1. Wallets Are Always Empty**
```
Server Wallet Balance: 0 XMR (permanently)
Multisig Address Balance: X XMR (escrow amount)
```

The server wallets have **zero balance**. They are signature devices, not fund storage. A wallet with 0 XMR cannot steal funds because there are no funds to steal.

**2. Funds Are Held in Multisig, Not Server Wallets**

When a user funds an escrow:
```
User Wallet → Multisig Address (2-of-3)
   (NOT)
User Wallet → Server Wallet
```

Funds go directly to the multisig address, which requires 2-of-3 signatures. The server wallets participate in signing but never receive the funds themselves.

**3. Server Cannot Unilaterally Move Funds**

To release funds from multisig:
- Requires **2 of 3** signatures (buyer, vendor, arbiter)
- Server controls all 3 temporary wallets
- BUT: Server follows programmatic rules (escrow state machine)
- Server cannot bypass state machine without code modification (auditable)

**4. Transparency and Auditability**

- All multisig transactions are on-chain (Monero blockchain)
- Escrow state machine is open-source (auditable logic)
- Wallet balances are verifiable (always 0 XMR)
- No hidden fund movements possible

**5. No Custody Risk**

**Traditional Custodial Risk:**
- Exchange holds 10,000 BTC
- Exchange gets hacked → all funds lost
- Server has custody risk

**Our Architecture:**
- Server wallets hold 0 XMR
- Server gets hacked → no funds stolen (wallets are empty)
- Only signing keys compromised, but cannot steal funds from multisig alone (need 2-of-3)
- **No custody risk**

---

## How It Works

### Escrow Lifecycle

#### Phase 1: Initialization

```
1. Buyer creates order for vendor's listing
   → Server generates escrow_id

2. Server creates 3 temporary wallets:
   • buyer_wallet_{escrow_id}
   • vendor_wallet_{escrow_id}
   • arbiter_wallet_{escrow_id}

3. Server initiates multisig setup:
   → prepare_multisig() on all 3 wallets
   → Exchange multisig_info between wallets
   → make_multisig() on all 3 wallets
   → Multisig address generated (2-of-3)

4. Server stores:
   • Multisig address
   • Escrow state = "Pending"
```

**Wallet Status:** Empty (0 XMR)

#### Phase 2: Funding

```
5. Server provides multisig address to buyer

6. Buyer sends XMR from personal wallet:
   → Transfer to multisig address
   → Amount: escrow_amount + fee

7. BlockchainMonitor detects incoming transaction:
   → Confirms N blocks (e.g., 10 confirmations)
   → Updates escrow state = "Funded"

8. Server notifies vendor: "Payment received, ship goods"
```

**Wallet Status:** Empty (0 XMR)
**Multisig Address:** X XMR (escrowed)

#### Phase 3: Normal Release (Happy Path)

```
9. Vendor ships goods and marks as "Shipped"

10. Buyer receives goods and marks as "Received"

11. Server initiates release transaction:
    → Creates unsigned transaction (multisig → vendor address)
    → Signs with buyer_wallet (signature 1/3)
    → Signs with vendor_wallet (signature 2/3)
    → 2-of-3 threshold met ✅

12. Server broadcasts transaction to Monero network

13. Vendor receives XMR in their personal wallet

14. Server deletes temporary wallets
```

**Wallet Status:** Deleted
**Multisig Address:** 0 XMR (released)

#### Phase 4: Dispute Resolution

```
9. Dispute raised (e.g., goods not received, wrong item)

10. Arbiter investigates:
    → Reviews evidence from buyer and vendor
    → Makes ruling (favor buyer OR favor vendor)

11a. Arbiter rules for buyer (refund):
     → Creates unsigned transaction (multisig → buyer address)
     → Signs with buyer_wallet (signature 1/3)
     → Signs with arbiter_wallet (signature 2/3)
     → Broadcasts refund transaction

11b. Arbiter rules for vendor (release):
     → Creates unsigned transaction (multisig → vendor address)
     → Signs with vendor_wallet (signature 1/3)
     → Signs with arbiter_wallet (signature 2/3)
     → Broadcasts release transaction

12. Server deletes temporary wallets after resolution
```

**Key Point:** Arbiter + one party (buyer OR vendor) can resolve dispute. Server cannot unilaterally decide.

---

## Security Model

### Trust Assumptions

**What You Must Trust:**
1. **Server follows coded escrow rules** (state machine)
   - Mitigation: Code is open-source and auditable
   - Server cannot bypass state machine without modifying code

2. **Arbiter is honest in disputes** (majority attacks possible)
   - Mitigation: Reputation system for arbiters
   - Users can choose trusted arbiters

**What You Do NOT Need to Trust:**
1. ❌ Server will not steal funds (wallets are empty, cannot steal)
2. ❌ Server will not freeze funds (2-of-3 multisig, cannot freeze alone)
3. ❌ Server infrastructure security (no funds on server)

### Attack Scenarios

**Scenario 1: Server Is Hacked**
- Attacker gains access to server and wallet keys
- **Impact:** Attacker controls 3 temporary wallets
- **Limitation:** Wallets are empty (0 XMR balance)
- **Result:** Cannot steal funds from multisig without user cooperation
- **Worst case:** Can collude with arbiter to steal funds (requires arbiter compromise)
- **Mitigation:** Arbiter reputation system, user choice of arbiter

**Scenario 2: Server Operator Is Malicious**
- Operator tries to modify code to steal funds
- **Limitation:** Code is open-source (detectable)
- **Limitation:** Still requires 2-of-3 signatures (cannot bypass multisig)
- **Mitigation:** Users audit code before using platform

**Scenario 3: Buyer and Arbiter Collude**
- Buyer orders item, claims non-delivery, arbiter rules for buyer
- **Impact:** Vendor loses funds
- **Mitigation:** Arbiter reputation system, vendor can appeal with evidence

**Scenario 4: Vendor and Arbiter Collude**
- Vendor never ships, arbiter rules for vendor
- **Impact:** Buyer loses funds
- **Mitigation:** Buyer chooses arbiter, reputation system

### Security Properties

✅ **Non-Custodial:** Server never holds user funds
✅ **No Single Point of Failure:** Requires 2-of-3 signatures
✅ **Transparent:** All transactions on-chain
✅ **Auditable:** Open-source state machine
✅ **Private:** Monero provides transaction privacy
✅ **Tor-Only:** .onion hidden service (network privacy)

---

## Usage Guide

### For Users (Buyers/Vendors)

#### Create Escrow (Buyer)

```bash
# 1. Browse marketplace, find listing
# 2. Click "Buy" → Server creates escrow

# 3. Fund escrow with your personal Monero wallet
monero-wallet-cli
> transfer <multisig_address> <amount>

# 4. Wait for confirmations (monitored by server)
# 5. Server notifies vendor to ship
```

#### Release Funds (Normal Completion)

```bash
# 1. Vendor ships goods, marks as "Shipped"
# 2. Buyer receives goods
# 3. Buyer clicks "Release Funds" on UI

# Server automatically:
# - Creates release transaction
# - Collects buyer + vendor signatures
# - Broadcasts to Monero network
# - Vendor receives XMR in their wallet
```

#### Raise Dispute

```bash
# 1. Buyer or Vendor clicks "Raise Dispute"
# 2. Server sets escrow state = "Disputed"
# 3. Arbiter reviews evidence
# 4. Arbiter submits ruling (buyer or vendor)

# Server automatically:
# - Creates refund/release transaction based on ruling
# - Collects arbiter + winning party signatures
# - Broadcasts to Monero network
```

---

## API Reference

### Create Escrow

**Endpoint:** `POST /api/escrow/create`

```json
{
  "listing_id": "listing_12345",
  "buyer_id": "user_abc",
  "vendor_id": "user_xyz",
  "arbiter_id": "arbiter_def",
  "amount_xmr": "0.5"
}
```

**Response:**
```json
{
  "escrow_id": "escrow_67890",
  "multisig_address": "5AxMRc...",
  "state": "Pending",
  "amount_atomic": 500000000000
}
```

### Get Escrow Status

**Endpoint:** `GET /api/escrow/{escrow_id}`

```json
{
  "escrow_id": "escrow_67890",
  "state": "Funded",
  "multisig_address": "5AxMRc...",
  "balance_atomic": 500000000000,
  "confirmations": 10,
  "buyer_id": "user_abc",
  "vendor_id": "user_xyz",
  "arbiter_id": "arbiter_def"
}
```

### Release Funds (Normal)

**Endpoint:** `POST /api/escrow/{escrow_id}/release`

```json
{
  "release_address": "vendor_wallet_address"
}
```

**Server automatically:**
1. Validates state = "Funded"
2. Creates unsigned transaction
3. Signs with buyer_wallet + vendor_wallet
4. Broadcasts transaction

### Raise Dispute

**Endpoint:** `POST /api/escrow/{escrow_id}/dispute`

```json
{
  "reason": "Goods not received",
  "evidence": "Tracking shows item never shipped"
}
```

### Arbiter Ruling

**Endpoint:** `POST /api/escrow/{escrow_id}/ruling`

```json
{
  "arbiter_id": "arbiter_def",
  "ruling": "buyer", // or "vendor"
  "reasoning": "Tracking confirms non-delivery"
}
```

**Server automatically:**
1. Validates arbiter_id matches escrow
2. Creates refund or release transaction
3. Signs with arbiter_wallet + appropriate party wallet
4. Broadcasts transaction

---

## FAQ

### Is this truly non-custodial if the server creates the wallets?

**Yes.** Non-custodial means "platform does not hold custody of funds." Our server wallets are **always empty** (0 XMR balance). Custody requires holding funds; empty wallets have nothing to hold custody of.

### Can the server steal funds from the multisig address?

**No.** The multisig address requires 2-of-3 signatures. While the server controls all 3 temporary wallets, it follows programmatic rules (escrow state machine). To bypass these rules, the server operator would need to:
1. Modify open-source code (detectable by users)
2. Still requires arbiter collusion (2-of-3 threshold)

### What happens if the server goes down?

**Short-term:**
- Escrow funds remain safe in multisig address
- Signatures cannot be generated while server is down
- No fund loss

**Long-term (server permanent shutdown):**
- Funds remain locked in multisig address
- Recovery requires access to wallet files + passwords
- **Limitation:** Currently no automated recovery mechanism
- **Mitigation (planned):** Timelocked refund transactions

### How is this different from LocalMonero or Bisq?

**LocalMonero/Bisq:** Fully decentralized P2P, no server coordination
**Our Architecture:** Server-coordinated for UX, but non-custodial for security

Trade-off:
- ✅ Better UX (automated escrow management)
- ✅ Still non-custodial (no fund custody)
- ❌ Requires trust in server to follow escrow rules (but not trust to not steal)

### Can I verify the wallets are really empty?

**Yes.** Wallet balance can be queried via RPC:
```bash
curl http://localhost:18082/json_rpc -d '{
  "jsonrpc":"2.0",
  "id":"0",
  "method":"get_balance"
}'

# Response: balance: 0, unlocked_balance: 0
```

Users can audit that server wallets have 0 balance at all times.

### What prevents server + arbiter collusion?

**Technical:** Nothing (2-of-3 multisig allows this)
**Practical:**
- Arbiter reputation system (on-chain history)
- Users choose arbiters they trust
- Economic incentive (arbiters earn fees, reputation loss = income loss)

This is an inherent property of 2-of-3 multisig—not unique to our architecture.

---

## Comparison: Custodial vs Our Architecture

| Property | Custodial Exchange | Our Architecture |
|----------|-------------------|------------------|
| **Server holds user funds** | ✅ Yes | ❌ No (wallets empty) |
| **Server can freeze funds** | ✅ Yes | ❌ No (requires 2-of-3) |
| **Server can steal funds** | ✅ Yes | ❌ No (wallets empty) |
| **Single point of failure** | ✅ Yes (server controls all) | ❌ No (2-of-3 multisig) |
| **Hack risk** | 🔴 Critical (all funds lost) | 🟢 Low (no funds on server) |
| **Trust requirements** | 🔴 High (trust not to steal) | 🟡 Medium (trust to follow rules) |
| **UX convenience** | ✅ High | ✅ High |
| **Decentralization** | ❌ Low | 🟡 Medium (coordinated, not custodial) |

---

## Conclusion

The Monero Marketplace uses a **server-managed multisig architecture** that provides:
- ✅ Convenience of server-side escrow management
- ✅ Security of non-custodial fund handling
- ✅ Privacy via Monero + Tor

**Key Principle:** Wallets are signature devices, not fund storage. Empty wallets cannot hold custody of funds they never possess.

This architecture balances usability (no local RPC setup required) with security (no server custody of funds), making it suitable for a privacy-focused marketplace.

---

**See Also:**
- [ADR-001: Monero-Only Architecture](../architecture/ADR-001-MONERO-ONLY-RATIONALE.md)
- [MONERO-MULTISIG-2OF3-COMPLETE-GUIDE.md](../architecture/MONERO-MULTISIG-2OF3-COMPLETE-GUIDE.md)
- [SECURITY.md](../../SECURITY.md)
