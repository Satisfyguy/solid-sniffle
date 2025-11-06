# Complete Guide: 2-of-3 Multisig System - Monero Marketplace

**For:** Integration wizard, user documentation, technical support
**Audience:** Buyers, Vendors, Arbiters, Developers integrating the API
**Version:** 1.0 - 2025-11-05
**Level:** 🟢 Beginner to Advanced

---

## Table of Contents

1. [What is a 2-of-3 Multisig Escrow?](#1-what-is-a-2-of-3-multisig-escrow)
2. [Why It's Secure](#2-why-its-secure)
3. [The 3 Roles Explained](#3-the-3-roles-explained)
4. [Complete Flow: From Order to Payment](#4-complete-flow-from-order-to-payment)
5. [Step-by-Step Guide for Each Role](#5-step-by-step-guide-for-each-role)
6. [Real-World Scenarios](#6-real-world-scenarios)
7. [Frequently Asked Questions (FAQ)](#7-frequently-asked-questions-faq)
8. [Troubleshooting](#8-troubleshooting)
9. [Technical Glossary](#9-technical-glossary)

---

## 1. What is a 2-of-3 Multisig Escrow?

### 🎯 Simple Definition

A **2-of-3 multisig escrow** is like **a safe with 3 keys** where you need **at least 2 keys** to open it.

```
┌────────────────────────────────────────────┐
│         💰 DIGITAL SAFE                    │
│                                            │
│   🔑 Key 1: BUYER                         │
│   🔑 Key 2: VENDOR                        │
│   🔑 Key 3: ARBITER (neutral)             │
│                                            │
│   ✅ Any 2 keys = OPEN                    │
│   ❌ Only 1 key = LOCKED                  │
└────────────────────────────────────────────┘
```

### 💡 Why "2 out of 3"?

**Principle**: Nobody can steal the money alone, but 2 honest parties can always unlock the funds.

| Combination | Scenario | Result |
|------------|----------|--------|
| 🔑 Buyer + 🔑 Vendor | Normal transaction | ✅ Funds released to vendor |
| 🔑 Buyer + 🔑 Arbiter | Defective product | ✅ Refund to buyer |
| 🔑 Vendor + 🔑 Arbiter | Buyer unresponsive | ✅ Payment to vendor (if legitimate) |
| 🔑 Buyer alone | Theft attempt | ❌ IMPOSSIBLE |
| 🔑 Vendor alone | Theft attempt | ❌ IMPOSSIBLE |
| 🔑 Arbiter alone | Theft attempt | ❌ IMPOSSIBLE |

---

## 2. Why It's Secure

### 🔒 5 Security Guarantees

#### ✅ 1. Non-Custodial (No Fund Custody)

**Problem with traditional platforms:**
```
You → [PLATFORM controls money] → Vendor
       ⚠️ Platform can:
       - Freeze your funds
       - Close your account
       - Go bankrupt with your money
```

**Our solution:**
```
You → [MULTISIG SAFE] → Vendor
       ✅ NOBODY controls alone
       ✅ Platform NEVER has access to funds
       ✅ Cryptographically impossible to steal
```

#### ✅ 2. Monero Blockchain (Privacy + Immutability)

- **Privacy**: Amounts and addresses hidden (RingCT + Stealth Addresses)
- **Immutable**: Once confirmed, impossible to reverse
- **Decentralized**: No central bank, no censorship

#### ✅ 3. Neutral Arbiter

- Arbiter has **NO financial interest** in the transaction
- Selected **randomly** by the platform
- **CANNOT decide alone** (needs a 2nd signature)
- **Reputation system** to ensure fairness

#### ✅ 4. Temporary EMPTY Wallets

**Non-Custodial Architecture:**

```
┌──────────────────────────────────────────────────────┐
│  IMPORTANT: Server creates 3 EMPTY wallets          │
│  These wallets NEVER CONTAIN money!                 │
│                                                      │
│  Purpose: Generate shared multisig address          │
│  After generation: Wallets CLOSED (frees RPC)       │
│  Money goes DIRECTLY into multisig                  │
└──────────────────────────────────────────────────────┘
```

**Flow:**
1. 🏗️ Server creates 3 temporary wallets (balance: 0 XMR)
2. 🔗 Wallets exchange cryptographic info (multisig setup)
3. 🎯 Generate shared multisig address (95 characters)
4. 🔒 Close temporary wallets (resource economy)
5. 💸 Buyer pays from their OWN wallet → multisig address
6. ✅ Funds secured, nobody can steal them

#### ✅ 5. Cryptographic Signatures

Each "signature" is a **mathematical proof** that you approve the transaction.

```
Unsigned Transaction:
  "Send 1 XMR from [escrow] to [vendor]"

+ Buyer Signature (buyer private key)
  = "I approve this payment to vendor"

+ Arbiter Signature (arbiter private key)
  = "Transaction legitimate, I approve"

= Complete Transaction (2/3 signatures)
  → Broadcast to Monero network ✅
```

---

## 3. The 3 Roles Explained

### 👤 BUYER

**Role:** Pays for a product/service

**Powers:**
- ✅ Release funds to vendor (if satisfied)
- ✅ Open a dispute (if problem)
- ✅ Sign a refund (with arbiter)

**Responsibilities:**
- 💰 Fund the escrow (send Monero to multisig address)
- 📦 Confirm product receipt
- ⏱️ Respond within deadlines (escrow expires otherwise)

**Cannot:**
- ❌ Steal money alone (need 2 signatures)
- ❌ Cancel a confirmed transaction
- ❌ Recover money without consensus (buyer+arbiter or vendor+arbiter)

---

### 🏪 VENDOR

**Role:** Provides a product/service

**Powers:**
- ✅ Receive payment (with buyer OR arbiter signature)
- ✅ Request refund (if customer unresponsive)
- ✅ Sign payment (with arbiter)

**Responsibilities:**
- 📦 Deliver product/service as promised
- 💬 Communicate with buyer
- 📸 Provide delivery proof if dispute

**Cannot:**
- ❌ Take money before buyer approves
- ❌ Steal money alone
- ❌ Manipulate arbiter (reputation system)

---

### ⚖️ ARBITER

**Role:** Neutral judge in case of conflict

**Powers:**
- ✅ Decide who receives money in case of dispute
- ✅ Sign transactions (release OR refund)
- ✅ Request evidence from both parties

**Responsibilities:**
- 🔍 Examine evidence impartially
- ⚖️ Decide according to platform rules
- 💬 Communicate decision clearly

**Cannot:**
- ❌ Decide without seeing evidence
- ❌ Steal money (needs a 2nd signature)
- ❌ Systematically favor buyers or vendors (loses reputation)

**Arbiter Selection:**
- 🎲 **Random assignment** (round-robin on active arbiter pool)
- ⭐ **Reputation system** (arbiters rated by parties)
- 💼 **Verified arbiters** (selected by platform)

---

## 4. Complete Flow: From Order to Payment

### 🗺️ Overview (4 Phases)

```
┌────────────────────────────────────────────────────────────────┐
│                     PHASE 1: ESCROW CREATION                    │
│  Buyer clicks "Buy" → Server creates 3 empty wallets           │
│  Duration: 5-10 seconds (automatic)                            │
└────────────────────────────────────────────────────────────────┘
                              ↓
┌────────────────────────────────────────────────────────────────┐
│                   PHASE 2: MULTISIG SETUP                       │
│  3 wallets exchange info → Generate multisig address           │
│  Duration: 10-15 seconds (automatic)                           │
└────────────────────────────────────────────────────────────────┘
                              ↓
┌────────────────────────────────────────────────────────────────┐
│                   PHASE 3: FUNDING                              │
│  Buyer sends XMR from external wallet → multisig               │
│  Duration: 2-4 minutes (depending on Monero network)           │
│  Required confirmations: 10 blocks (~20 min)                   │
└────────────────────────────────────────────────────────────────┘
                              ↓
┌────────────────────────────────────────────────────────────────┐
│                   PHASE 4: RESOLUTION                           │
│  Option A: Buyer + Vendor → Release (happy path)               │
│  Option B: Buyer + Arbiter → Refund (dispute → buyer)          │
│  Option C: Vendor + Arbiter → Release (dispute → vendor)       │
│  Duration: Variable (depending on communications)              │
└────────────────────────────────────────────────────────────────┘
```

### 📋 Phase Technical Details

#### PHASE 1: Escrow Creation (Backend)

**Steps:**
1. 🛒 Buyer clicks "Order" on a listing
2. 🔍 Server validates: stock available, price valid
3. 📝 Create order record in DB
4. 🎲 Random arbiter assignment (round-robin)
5. 🏗️ Create 3 EMPTY temporary wallets:
   - `buyer_temp_escrow_{escrow_id}`
   - `vendor_temp_escrow_{escrow_id}`
   - `arbiter_temp_escrow_{escrow_id}`
6. 💾 Store wallet IDs in DB (columns: `buyer_temp_wallet_id`, etc.)
7. ✅ Escrow status: `created`

**Duration:** 5-10 seconds
**Automatic:** Yes
**User intervention:** None

---

#### PHASE 2: Multisig Setup (Cryptography)

**Steps:**

```
┌─────────────────────────────────────────────────────────────┐
│  STEP 1/3: prepare_multisig()                                │
│  ─────────────────────────────────────────────────────────── │
│  Each wallet generates its "multisig_info" (public key)     │
│                                                              │
│  Wallet Buyer:   "MultisigxV2ABC123..." (2000 chars)        │
│  Wallet Vendor:  "MultisigxV2DEF456..." (2000 chars)        │
│  Wallet Arbiter: "MultisigxV2GHI789..." (2000 chars)        │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│  STEP 2/3: make_multisig(threshold=2, infos=[...])          │
│  ─────────────────────────────────────────────────────────── │
│  Each wallet imports info from the other 2                  │
│  Create local multisig wallet with 2/3 threshold            │
│                                                              │
│  Result: Each wallet can now sign                           │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│  STEP 3/3: finalize_multisig()                               │
│  ─────────────────────────────────────────────────────────── │
│  Generate shared multisig address                            │
│  Validation: address.len() == 95 (Monero testnet standard)  │
│                                                              │
│  Address: 9wq792k9sxVZiLn66S3Qzv8QfmtcwkdXgM5cW...           │
│                                                              │
│  ✅ All 3 wallets now have the SAME multisig address        │
└─────────────────────────────────────────────────────────────┘
```

**Duration:** 10-15 seconds
**Automatic:** Yes
**User intervention:** None

**Security:**
- ✅ Strict validation of `multisig_info` (prefix, length, chars)
- ✅ Automatic retry if failure (3 attempts, exponential backoff)
- ✅ Close temporary wallets after setup (frees RPC slots)

---

#### PHASE 3: Funding (Buyer Action)

**Steps:**

```
┌─────────────────────────────────────────────────────────────┐
│  👤 BUYER                                                    │
│  ─────────────────────────────────────────────────────────── │
│  1. Copy multisig address displayed on platform             │
│     Example: 9wq792k9sxVZiLn66S3Qzv8QfmtcwkdXgM...           │
│                                                              │
│  2. Open your external Monero wallet (Feather, CLI, GUI)    │
│                                                              │
│  3. Send EXACTLY the requested amount                       │
│     Amount: 1.000000000000 XMR (12 decimals)                │
│     Destination: [Copied multisig address]                  │
│                                                              │
│  4. Wait for 10 confirmations (~20 minutes)                 │
│     Block 1-10: "Pending..."                                │
│     Block 10: ✅ "Funds confirmed!"                         │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│  🔍 BLOCKCHAIN MONITOR (Automatic)                           │
│  ─────────────────────────────────────────────────────────── │
│  Server queries Monero network every 30 seconds             │
│                                                              │
│  Query: get_transfer_by_txid(tx_hash)                       │
│  → Confirmations: 0, 1, 2, ..., 10 ✅                       │
│                                                              │
│  When confirmations >= 10:                                  │
│    - Escrow status: funded → active                         │
│    - WebSocket notification → Buyer + Vendor                │
│    - Vendor can now ship                                    │
└─────────────────────────────────────────────────────────────┘
```

**Duration:** 2-4 min (transaction) + 20 min (confirmations)
**Automatic:** Blockchain monitoring yes, buyer payment NO
**User intervention:** Buyer must send Monero manually

**⚠️ Important Points:**

1. **Exact Amount:**
   ```
   ✅ CORRECT: 1.000000000000 XMR (12 decimals)
   ❌ ERROR:  1.0 XMR (partial payment)
   ❌ ERROR:  1.5 XMR (overpaid, money lost)
   ```

2. **Correct Address:**
   - Verify first 5 + last 5 characters
   - Monero testnet: starts with `9` or `B`
   - Monero mainnet: starts with `4`

3. **External Wallet:**
   - Use your OWN Monero wallet
   - Server NEVER has access to your private keys
   - You control your funds until payment

---

#### PHASE 4: Resolution (3 Scenarios)

##### 📦 SCENARIO A: Happy Path (Normal)

```
┌─────────────────────────────────────────────────────────────┐
│  1. Vendor ships product                                     │
│  2. Buyer receives product                                   │
│  3. Buyer clicks "Release Funds"                             │
│  4. Signature 1/2: Buyer signs transaction                   │
│  5. Signature 2/2: Arbiter signs automatically               │
│     (or Vendor signs if implemented)                         │
│  6. Transaction broadcast to Monero network                  │
│  7. Vendor receives payment (after 10 confirmations)         │
│  8. Escrow status: releasing → completed                     │
└─────────────────────────────────────────────────────────────┘
```

**Participants:** Buyer + Arbiter (2/3)
**Duration:** 1 minute (signatures) + 20 min (confirmations)
**Probability:** ~95% of transactions

---

##### 🚨 SCENARIO B: Dispute → Buyer Refund

```
┌─────────────────────────────────────────────────────────────┐
│  1. Buyer receives defective product / never received        │
│  2. Buyer clicks "Open Dispute"                              │
│  3. Arbiter receives notification + evidence                 │
│  4. Arbiter examines:                                        │
│     - Photos of defective product                           │
│     - Tracking number (if applicable)                       │
│     - Buyer/vendor messages                                 │
│  5. Arbiter decides: "Buyer refund justified"                │
│  6. Signature 1/2: Buyer signs refund                        │
│  7. Signature 2/2: Arbiter signs refund                      │
│  8. Refund transaction broadcast                             │
│  9. Buyer recovers funds                                     │
│ 10. Escrow status: refunding → refunded                      │
└─────────────────────────────────────────────────────────────┘
```

**Participants:** Buyer + Arbiter (2/3)
**Duration:** Variable (arbiter investigation: 1-7 days)
**Probability:** ~3-4% of transactions

**Required Evidence:**
- 📸 Photos/videos of product
- 📦 Proof of delivery (tracking)
- 💬 Communication history
- 📋 Detailed problem description

---

##### 🔄 SCENARIO C: Dispute → Vendor Payment

```
┌─────────────────────────────────────────────────────────────┐
│  1. Vendor ships, buyer unresponsive (30 days)              │
│  2. Vendor opens dispute                                     │
│  3. Arbiter examines:                                        │
│     - Proof of delivery (signature, tracking)               │
│     - Vendor contact attempts → buyer                       │
│     - Buyer history (known scammer?)                        │
│  4. Arbiter decides: "Vendor payment justified"              │
│  5. Signature 1/2: Vendor signs release                      │
│  6. Signature 2/2: Arbiter signs release                     │
│  7. Release transaction broadcast                            │
│  8. Vendor receives payment                                  │
│  9. Escrow status: releasing → completed                     │
└─────────────────────────────────────────────────────────────┘
```

**Participants:** Vendor + Arbiter (2/3)
**Duration:** Variable (arbiter investigation: 1-7 days)
**Probability:** ~1-2% of transactions

**Triggers:**
- ⏱️ Buyer doesn't confirm receipt (30-day timeout)
- 📴 Buyer unreachable (no response 14 days)
- 🚩 Known fraudulent buyer (scam pattern)

---

## 5. Step-by-Step Guide for Each Role

### 👤 BUYER GUIDE

#### Step 1: Order a Product

```
┌────────────────────────────────────────────────────────┐
│  🛍️ LISTING PAGE                                       │
│  ──────────────────────────────────────────────────────│
│  Product: "Dell XPS 15 Laptop"                         │
│  Price: 2.5 XMR                                        │
│  Vendor: TechSeller ⭐⭐⭐⭐⭐ (156 sales)              │
│  Stock: 3 units                                        │
│                                                        │
│  [📸 View Photos] [💬 Contact Vendor]                 │
│                                                        │
│  [🛒 Add to Cart] [⚡ Buy Now]                        │
└────────────────────────────────────────────────────────┘
                        ↓ CLICK
┌────────────────────────────────────────────────────────┐
│  ✅ Order created!                                     │
│  Escrow #a7f3e2b1 initialized                         │
│  Status: Awaiting payment                             │
│                                                        │
│  → Redirecting to escrow page...                      │
└────────────────────────────────────────────────────────┘
```

---

#### Step 2: View Escrow Details

```
┌────────────────────────────────────────────────────────────┐
│  💼 ESCROW #a7f3e2b1                                       │
│  ────────────────────────────────────────────────────────  │
│  Status: 🟡 Awaiting Funding                               │
│  Amount: 2.500000000000 XMR                                │
│                                                            │
│  TIMELINE:                                                 │
│  ✅ 1. Escrow Initiated (2025-11-05 14:32)                │
│  ✅ 2. Multisig Setup Complete (2025-11-05 14:32)         │
│  🟡 3. Awaiting Payment from Buyer                        │
│  ⏳ 4. Pending Resolution                                 │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  📋 MULTISIG ADDRESS (Copy this address)            │ │
│  │  9wq792k9sxVZiLn66S3Qzv8QfmtcwkdXgM5cWGsXAPxo...    │ │
│  │  [📋 Copy] [📱 QR Code]                             │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
│  ⚠️ IMPORTANT:                                             │
│  • Send EXACTLY 2.500000000000 XMR                        │
│  • Verify address (first/last chars)                      │
│  • Use your own Monero wallet                             │
│  • Wait for 10 confirmations (~20 min)                    │
└────────────────────────────────────────────────────────────┘
```

---

#### Step 3: Send Payment (External Wallet)

**Example with Feather Wallet:**

```
┌────────────────────────────────────────────────────────┐
│  FEATHER WALLET - Send                                 │
│  ──────────────────────────────────────────────────────│
│  Recipient:                                            │
│  [9wq792k9sxVZiLn66S3Qzv8QfmtcwkdXgM5cWGsXAPxo...]    │
│                                                        │
│  Amount:                                               │
│  [2.500000000000] XMR                                  │
│                                                        │
│  Priority:                                             │
│  ⚫ Normal (2 min)  ⚪ Fast (30 sec)                   │
│                                                        │
│  Estimated fee: 0.00015 XMR                           │
│  Total: 2.50015 XMR                                    │
│                                                        │
│  [🚀 Send]                                             │
└────────────────────────────────────────────────────────┘
                        ↓ CLICK
┌────────────────────────────────────────────────────────┐
│  ✅ Transaction sent!                                  │
│  TX ID: a3f8b2c1d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0...   │
│                                                        │
│  Confirmations: 0/10                                   │
│  Estimated time: ~20 minutes                          │
└────────────────────────────────────────────────────────┘
```

---

#### Step 4: Wait for Confirmations

```
┌────────────────────────────────────────────────────────┐
│  💼 ESCROW #a7f3e2b1                                   │
│  ──────────────────────────────────────────────────────│
│  Status: 🟡 Confirming Payment                         │
│                                                        │
│  🔄 Confirmations: 7/10                                │
│  ████████████░░░░  (~5 min remaining)                 │
│                                                        │
│  Transaction detected:                                 │
│  a3f8b2c1d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0...          │
│                                                        │
│  ⏱️ Auto-updating every 30 seconds                     │
└────────────────────────────────────────────────────────┘
```

**After 10 confirmations:**

```
┌────────────────────────────────────────────────────────┐
│  ✅ Payment Confirmed!                                 │
│  ──────────────────────────────────────────────────────│
│  Status: 🟢 Active (Funds Secured)                     │
│                                                        │
│  Vendor has been notified.                            │
│  You will receive notification when:                  │
│  • Vendor ships the product                           │
│  • Package is delivered                               │
│                                                        │
│  📱 Notifications enabled                              │
└────────────────────────────────────────────────────────┘
```

---

#### Step 5: Product Receipt & Fund Release

**5.1 Shipping Notification:**

```
┌────────────────────────────────────────────────────────┐
│  📦 Product Shipped!                                   │
│  ──────────────────────────────────────────────────────│
│  Vendor: TechSeller                                    │
│  Product: Dell XPS 15 Laptop                           │
│                                                        │
│  Tracking: 1Z999AA10123456784                          │
│  Carrier: DHL Express                                  │
│  Estimated delivery: 2025-11-08                        │
│                                                        │
│  [📍 Track Package]                                    │
└────────────────────────────────────────────────────────┘
```

**5.2 Receipt & Verification:**

```
┌────────────────────────────────────────────────────────┐
│  ✅ Package Delivered! (2025-11-08 10:42)              │
│  ──────────────────────────────────────────────────────│
│  Did you receive the product in good condition?       │
│                                                        │
│  [ ] Yes, product as described                        │
│  [ ] No, defective product                            │
│  [ ] No, wrong product                                │
│  [ ] No, empty package                                │
│  [ ] Not received yet                                 │
│                                                        │
│  [✅ Confirm Receipt] [🚨 Report Problem]             │
└────────────────────────────────────────────────────────┘
```

**5.3 Fund Release:**

```
┌────────────────────────────────────────────────────────┐
│  🎉 Thank you for confirmation!                        │
│  ──────────────────────────────────────────────────────│
│  You are about to release:                            │
│  💰 2.5 XMR → TechSeller                               │
│                                                        │
│  ⚠️ This action is IRREVERSIBLE                        │
│  Funds will be sent to vendor.                        │
│                                                        │
│  [⬅️ Cancel] [✅ Confirm Release]                     │
└────────────────────────────────────────────────────────┘
                        ↓ CLICK
┌────────────────────────────────────────────────────────┐
│  ✅ Transaction in progress...                         │
│  ──────────────────────────────────────────────────────│
│  Signature 1/2: You ✅                                 │
│  Signature 2/2: Arbiter ⏳ (automatic)                 │
│                                                        │
│  Broadcasting to Monero network...                    │
└────────────────────────────────────────────────────────┘
                        ↓ 20 seconds
┌────────────────────────────────────────────────────────┐
│  🎊 Transaction Complete!                              │
│  ──────────────────────────────────────────────────────│
│  Status: ✅ Completed                                  │
│  TX ID: f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1...     │
│                                                        │
│  Vendor will receive funds after 10 confirmations.    │
│                                                        │
│  Thank you for using Monero Marketplace!              │
│  [⭐ Rate Vendor] [🏠 Home]                            │
└────────────────────────────────────────────────────────┘
```

---

### 🏪 VENDOR GUIDE

#### Step 1: Receive Order Notification

```
┌────────────────────────────────────────────────────────┐
│  🔔 New Order!                                         │
│  ──────────────────────────────────────────────────────│
│  Product: Dell XPS 15 Laptop                           │
│  Buyer: CryptoUser42                                   │
│  Amount: 2.5 XMR                                       │
│  Escrow: #a7f3e2b1                                     │
│                                                        │
│  Status: 🟡 Awaiting Buyer Payment                     │
│                                                        │
│  [📋 View Details]                                     │
└────────────────────────────────────────────────────────┘
```

---

#### Step 2: Wait for Buyer Payment

```
┌────────────────────────────────────────────────────────┐
│  📋 ORDER #a7f3e2b1                                    │
│  ──────────────────────────────────────────────────────│
│  Status: 🟡 Awaiting Buyer Payment                     │
│                                                        │
│  ⏳ Waiting for buyer payment...                       │
│                                                        │
│  • Escrow created: 2025-11-05 14:32                   │
│  • Expiration deadline: 2025-11-12 14:32 (7 days)     │
│                                                        │
│  💡 Tip: Prepare product while waiting                │
│                                                        │
│  [💬 Contact Buyer]                                    │
└────────────────────────────────────────────────────────┘
```

---

#### Step 3: Payment Confirmed → Shipping

```
┌────────────────────────────────────────────────────────┐
│  ✅ Payment Confirmed! (2025-11-05 15:02)              │
│  ──────────────────────────────────────────────────────│
│  Status: 🟢 Active - Ready to Ship                     │
│                                                        │
│  2.5 XMR now secured in escrow.                       │
│  You can safely ship the product.                     │
│                                                        │
│  ┌──────────────────────────────────────────────────┐ │
│  │  📦 DELIVERY INFORMATION                         │ │
│  │  Name: [Encrypted - revealed after shipping]    │ │
│  │  Address: [Encrypted - revealed after shipping] │ │
│  │  Phone: [Encrypted - revealed after shipping]   │ │
│  └──────────────────────────────────────────────────┘ │
│                                                        │
│  [📦 Mark as Shipped]                                  │
└────────────────────────────────────────────────────────┘
```

---

#### Step 4: Confirm Shipping

```
┌────────────────────────────────────────────────────────┐
│  📦 CONFIRM SHIPPING                                   │
│  ──────────────────────────────────────────────────────│
│  Order: #a7f3e2b1                                      │
│                                                        │
│  Carrier:                                              │
│  [▼ Select]                                            │
│  • DHL Express                                         │
│  • FedEx                                               │
│  • USPS                                                │
│  • UPS                                                 │
│  • Other                                               │
│                                                        │
│  Tracking number:                                      │
│  [____________________]                                │
│                                                        │
│  Shipping date:                                        │
│  [2025-11-06] [12:00]                                  │
│                                                        │
│  Notes (optional):                                     │
│  [Fragile package, handle with care]                  │
│                                                        │
│  [⬅️ Cancel] [✅ Confirm Shipping]                    │
└────────────────────────────────────────────────────────┘
                        ↓ CLICK
┌────────────────────────────────────────────────────────┐
│  ✅ Shipping Confirmed!                                │
│  ──────────────────────────────────────────────────────│
│  Buyer has been notified.                             │
│  Tracking: 1Z999AA10123456784                          │
│                                                        │
│  You will be paid once buyer confirms                 │
│  product receipt.                                      │
│                                                        │
│  [📍 Track Delivery] [💬 Contact Buyer]               │
└────────────────────────────────────────────────────────┘
```

---

#### Step 5: Receive Payment

```
┌────────────────────────────────────────────────────────┐
│  🎊 Payment Received! (2025-11-08 11:15)               │
│  ──────────────────────────────────────────────────────│
│  Status: ✅ Completed                                  │
│  Amount: 2.5 XMR                                       │
│                                                        │
│  Buyer confirmed product receipt.                     │
│  Funds have been released.                            │
│                                                        │
│  Transaction:                                          │
│  f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1...            │
│                                                        │
│  Confirmations: 10/10 ✅                               │
│  Funds available in your wallet!                      │
│                                                        │
│  [⭐ Rate Buyer] [💰 View Wallet]                      │
└────────────────────────────────────────────────────────┘
```

---

### ⚖️ ARBITER GUIDE

#### Role & Responsibilities

```
┌────────────────────────────────────────────────────────┐
│  ⚖️ ARBITER DASHBOARD                                  │
│  ──────────────────────────────────────────────────────│
│  Statistics:                                           │
│  • Assigned disputes: 3                               │
│  • Resolved disputes: 127                             │
│  • Satisfaction rate: 94.1%                           │
│  • Reputation: ⭐⭐⭐⭐⭐ (4.8/5.0)                      │
│                                                        │
│  ┌──────────────────────────────────────────────────┐ │
│  │  🚨 PENDING DISPUTES (3)                         │ │
│  ├──────────────────────────────────────────────────┤ │
│  │  #f7a2b3 - Defective product (2 days)           │ │
│  │  #c1d4e5 - Non-delivery (4 days) 🔥             │ │
│  │  #a9b8c7 - Wrong product (1 day)                │ │
│  └──────────────────────────────────────────────────┘ │
│                                                        │
│  [📋 Process Next Dispute]                             │
└────────────────────────────────────────────────────────┘
```

---

#### Processing a Dispute

**Step 1: Case Review**

```
┌────────────────────────────────────────────────────────┐
│  🔍 DISPUTE #f7a2b3                                    │
│  ──────────────────────────────────────────────────────│
│  Type: Defective product                              │
│  Amount: 2.5 XMR                                       │
│  Opened by: Buyer (CryptoUser42)                      │
│  Opened on: 2025-11-09 18:45                          │
│  Decision deadline: 2025-11-16 (7 days)               │
│                                                        │
│  PARTIES:                                              │
│  👤 Buyer: CryptoUser42 (18 purchases, 100% positive)│
│  🏪 Vendor: TechSeller (156 sales, 97% positive)     │
│                                                        │
│  PRODUCT:                                              │
│  Dell XPS 15 Laptop - 2.5 XMR                         │
│  Listing: #xyz789                                      │
│                                                        │
│  [📋 View Evidence] [💬 Chat History]                 │
└────────────────────────────────────────────────────────┘
```

**Step 2: Evidence Analysis**

```
┌────────────────────────────────────────────────────────┐
│  📸 BUYER EVIDENCE                                     │
│  ──────────────────────────────────────────────────────│
│  Description:                                          │
│  "Received laptop with broken screen. Packaging       │
│   intact, so damage before shipping."                 │
│                                                        │
│  Photos (3):                                           │
│  🖼️ [photo1.jpg] - Broken screen (general view)      │
│  🖼️ [photo2.jpg] - Screen crack zoom                 │
│  🖼️ [photo3.jpg] - Original packaging intact         │
│                                                        │
│  Unboxing video (2 min):                              │
│  🎬 [unboxing_video.mp4]                               │
│                                                        │
│  [▶️ View Photos] [▶️ View Video]                      │
└────────────────────────────────────────────────────────┘
        ↓
┌────────────────────────────────────────────────────────┐
│  📸 VENDOR EVIDENCE                                    │
│  ──────────────────────────────────────────────────────│
│  Description:                                          │
│  "Laptop tested before shipping. Photos show          │
│   functional screen. Damage probably during           │
│   transport."                                          │
│                                                        │
│  Photos (2):                                           │
│  🖼️ [test_before_ship1.jpg] - Screen on, working     │
│  🖼️ [test_before_ship2.jpg] - No crack               │
│                                                        │
│  Tracking:                                             │
│  1Z999AA10123456784 (DHL Express)                      │
│  Delivered: 2025-11-08 10:42 - Signed by recipient   │
│                                                        │
│  [▶️ View Photos] [📍 Tracking History]               │
└────────────────────────────────────────────────────────┘
```

**Step 3: Decision**

```
┌────────────────────────────────────────────────────────┐
│  ⚖️ ARBITRATION DECISION - Dispute #f7a2b3            │
│  ──────────────────────────────────────────────────────│
│  Analysis:                                             │
│  ✅ Buyer photos clearly show broken screen           │
│  ✅ Unboxing video confirms damage on receipt         │
│  ✅ Intact packaging = damage before shipping         │
│  ⚠️ Vendor tested before (photo evidence)             │
│  ⚠️ Possibly damaged during transport                 │
│                                                        │
│  Conclusion:                                           │
│  Product defective on receipt. Vendor responsible     │
│  (transport insurance mandatory).                     │
│                                                        │
│  Decision: 🔴 BUYER REFUND                            │
│                                                        │
│  Justification (visible to both parties):             │
│  [Product arrived defective. Although vendor tested   │
│   before shipping, lack of transport insurance makes  │
│   vendor responsible. Full refund granted to buyer.]  │
│                                                        │
│  [⬅️ Revise] [✅ Confirm Decision]                    │
└────────────────────────────────────────────────────────┘
                        ↓ CLICK
┌────────────────────────────────────────────────────────┐
│  ✅ Decision Recorded                                  │
│  ──────────────────────────────────────────────────────│
│  Refund transaction in progress...                    │
│                                                        │
│  Signature 1/2: Buyer ⏳ (automatic)                   │
│  Signature 2/2: You (Arbiter) ⏳                       │
│                                                        │
│  Both parties have been notified.                     │
└────────────────────────────────────────────────────────┘
                        ↓ 30 seconds
┌────────────────────────────────────────────────────────┐
│  🎊 Dispute Resolved!                                  │
│  ──────────────────────────────────────────────────────│
│  Decision: Buyer Refund                               │
│  TX ID: r3f4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1...     │
│                                                        │
│  Reputation updated:                                   │
│  • Buyer: +5 points (honesty)                         │
│  • Vendor: -2 points (defective product)              │
│  • You: +1 point (fair resolution)                    │
│                                                        │
│  [📋 Next Dispute] [📊 View Statistics]               │
└────────────────────────────────────────────────────────┘
```

---

## 6. Real-World Scenarios

### ✅ Scenario 1: Perfect Transaction

**Context:**
Alice buys a rare book (0.05 XMR) from Bob.

**Timeline:**
```
14:00 → Alice orders → Escrow created
14:01 → Multisig setup → Address generated
14:05 → Alice sends 0.05 XMR → Transaction broadcast
14:25 → 10 confirmations → Escrow active
14:30 → Bob ships (2-day shipping)
16:30 → Alice receives book (2 days later)
16:35 → Alice confirms receipt
16:36 → Signatures (Alice + Arbiter automatic)
16:37 → Bob receives 0.05 XMR
17:00 → 10 confirmations → Transaction completed ✅
```

**Total duration:** 2 days 3h
**Satisfied parties:** Alice ⭐⭐⭐⭐⭐, Bob ⭐⭐⭐⭐⭐

---

### ⚠️ Scenario 2: Defective Product

**Context:**
Charlie buys headphones (0.15 XMR) from Dave. Headphones broken on receipt.

**Timeline:**
```
09:00 → Charlie orders
09:25 → Escrow active (payment confirmed)
10:00 → Dave ships
12:00 → Charlie receives (2 days later)
12:05 → Charlie tests → Left earphone doesn't work
12:10 → Charlie opens dispute + uploads photos/video
12:15 → Arbiter Emma assigned automatically
14:00 → Emma examines evidence (2h)
        ✅ Unboxing video → broken earphone
        ✅ Multiple photos → confirmed defective
        ❌ Dave denies responsibility (no proof)
14:30 → Emma decides: Charlie refund
14:31 → Signatures (Charlie + Emma)
14:32 → Charlie receives 0.15 XMR
15:00 → Transaction confirmed ✅

Reputation:
  Charlie: +5 (proven honesty)
  Dave: -10 (defective product, poor communication)
  Emma: +2 (fast and fair resolution)
```

**Total duration:** 2 days 6h (including 2h30 dispute resolution)
**Result:** Charlie refunded, Dave penalized

---

### 🚨 Scenario 3: Ghost Buyer

**Context:**
Eve buys a GPU (3.5 XMR) from Frank. Eve disappears after delivery.

**Timeline:**
```
10:00 → Eve orders GPU
10:25 → Escrow active
11:00 → Frank ships (FedEx 24h)
14:00 → GPU delivered + Eve signature (tracking confirmed)
14:05 → Frank messages Eve: "Please confirm receipt"
        → No response
16:00 → Frank reminder
        → No response
D+1 → Frank daily reminders (7 days)
      → Still no response
D+7 → Timeout approaching (escrow expires in 7 days)
D+8 → Frank opens dispute
D+8 → Arbiter George examines:
        ✅ Tracking: Delivered + signed by Eve
        ✅ Frank messages → Eve (7 contact attempts)
        ✅ Eve history: 2 similar disputes (pattern)
        ❌ Eve still unresponsive
D+8 → George decides: Frank payment (Eve = scammer)
D+8 → Signatures (Frank + George)
D+8 → Frank receives 3.5 XMR ✅

Reputation:
  Frank: +5 (legitimate victim, solid evidence)
  Eve: -50 (confirmed scammer) + AUTO-BAN
  George: +3 (thorough investigation)
```

**Total duration:** 8 days (7 days waiting + 1 day resolution)
**Result:** Frank paid, Eve banned from platform

---

### 🔄 Scenario 4: Honest Mistake (Wrong Product)

**Context:**
Grace orders RED phone (1.2 XMR). Henry ships BLUE phone by mistake.

**Timeline:**
```
08:00 → Grace orders (color: RED)
08:25 → Escrow active
09:00 → Henry prepares order
        ⚠️ ERROR: Takes BLUE phone instead of RED
09:30 → Henry ships
11:00 → Grace receives (next day)
11:05 → Grace opens package → BLUE phone
11:10 → Grace messages Henry: "Wrong color"
11:15 → Henry responds: "Sorry! Honest mistake.
        2 options:
        1. I refund immediately
        2. I send correct product + you keep wrong one as compensation"
11:20 → Grace: "Option 2 please, thanks for honesty"
11:25 → Henry ships RED phone
13:00 → Grace receives RED phone (next day)
13:05 → Grace confirms receipt
13:06 → Grace releases funds → Henry paid ✅

Reputation:
  Grace: +3 (patient, understanding)
  Henry: +0 (mistake compensated, no penalty)
```

**Total duration:** 2 days
**Result:** Problem solved without arbiter, trust reinforced

---

## 7. Frequently Asked Questions (FAQ)

### 🔒 Security & Trust

#### Q1: Who controls the escrow money?

**A:** NOBODY controls the money alone.

- ❌ Platform CANNOT take the money
- ❌ Buyer alone CANNOT take back the money
- ❌ Vendor alone CANNOT take the money
- ❌ Arbiter alone CANNOT take the money

✅ **Only 2 of 3 parties together** can move funds.

#### Q2: What happens if platform shuts down?

**A:** Your funds are **ALWAYS RECOVERABLE**.

Multisig exists **on Monero blockchain**, not on our servers.

**Recovery procedure:**
1. Download your multisig private keys (automatic backup)
2. Contact other party (buyer/vendor) off-platform
3. Sign transaction together (2/3 without platform)
4. Recover your funds ✅

**We provide:**
- Emergency recovery guide
- Automatic multisig key export (encrypted)
- Emergency arbiter contact (Tor messaging)

#### Q3: Can arbiter steal my money?

**A:** NO. Arbiter needs a 2nd signature.

**Impossible scenario:**
```
Malicious arbiter tries to steal:
  1. Arbiter signs transaction to their address
  2. ❌ BLOCKED: Need buyer OR vendor signature
  3. Buyer/Vendor see the transaction
  4. ⚠️ Refuse to sign (destination = arbiter address)
  5. ❌ Transaction NEVER passes
  6. 🚨 Arbiter reported, banned, reputation destroyed
```

**Additional guarantees:**
- Selected arbiters (off-platform identity verification)
- Reputation system (history visibility)
- Arbiter rotation (not always the same one)
- Algorithm surveillance (suspicious pattern detection)

---

### 💰 Payments & Transactions

#### Q4: How long does a transaction take?

**Typical timeline:**

| Phase | Duration | Explanation |
|-------|----------|-------------|
| **Escrow Creation** | 5-10 sec | Wallet creation + DB |
| **Multisig Setup** | 10-15 sec | Crypto info exchange |
| **Buyer Payment** | 2-4 min | Monero tx broadcast |
| **10 Confirmations** | ~20 min | Blockchain validation |
| **Vendor Ships** | 1-7 days | Real logistics |
| **Buyer Confirms** | Instant | 1 click |
| **Release Signatures** | 30-60 sec | 2 crypto signatures |
| **Final Confirmations** | ~20 min | Blockchain validation |

**Normal total:** 1-7 days (depending on shipping)
**Dispute total:** +2-7 days (arbiter investigation)

#### Q5: What if I send wrong amount?

**Case 1: Insufficient amount (e.g., 0.9 XMR instead of 1.0)**

```
❌ Escrow will NOT be activated
⚠️ Blockchain monitor detects: received 0.9, expected 1.0
📧 Buyer notification: "Partial payment detected"
💡 Solutions:
  1. Send complement (0.1 additional XMR)
  2. Request refund from support
```

**Case 2: Excessive amount (e.g., 1.5 XMR instead of 1.0)**

```
✅ Escrow activated (only 1.0 XMR counts)
⚠️ 0.5 XMR surplus = LOST (impossible to refund)
📧 Notification: "Overpayment - surplus lost"
💡 Prevention: VERIFY 3 TIMES before sending
```

**⚠️ IMPORTANT:** Monero transactions are **IRREVERSIBLE**.
Always **verify** amount AND address before sending.

#### Q6: Can I cancel my order?

**Before payment (status: created):**
```
✅ YES - Click "Cancel Order"
   Escrow closed, no fees
```

**After payment (status: active):**
```
⚠️ NO - Funds locked in multisig
💡 Solutions:
  1. Negotiate cancellation with vendor
     → Vendor signs refund with arbiter
  2. Wait for expiration (7-30 days depending on config)
     → Automatic refund if vendor doesn't ship
```

---

### 🚨 Disputes & Problems

#### Q7: How to open a dispute?

**Step 1: Document the problem**

Prepare BEFORE opening dispute:
- 📸 **Photos/videos** (minimum 3 different angles)
- 📦 **Packaging** (show condition on receipt)
- 💬 **Message history** with vendor (screenshots)
- 📋 **Detailed description** (what, when, why)

**Step 2: Attempt friendly resolution**

```
Before official dispute, try:
  1. Message vendor (explain problem)
  2. Wait for response (48h max)
  3. Negotiate solution (refund, exchange, discount)
  4. If agreement → No dispute needed!
  5. If disagreement → Open official dispute
```

**Step 3: Open dispute (if necessary)**

```
Escrow Page → Button [🚨 Open Dispute]
  ↓
Form:
  • Reason: [Defective product ▼]
  • Description: [Text 200-1000 chars]
  • Evidence: [Upload 10 files max]
  • Vendor contact attempts: [Yes ✓] [No ✗]
  ↓
[Submit Dispute]
  ↓
✅ Dispute created #abc123
   Arbiter assigned within 24h
   Resolution deadline: 7 days max
```

#### Q8: How long does dispute resolution take?

**Standard timeline:**

```
Day 0: Dispute opened
       ↓
Day 0-1: Arbiter assignment (automatic)
         ↓
Day 1-2: Arbiter examines evidence
         • Reads descriptions
         • Views photos/videos
         • Checks party history
         ↓
Day 2-3: Arbiter requests additional info (if needed)
         • Questions to parties
         • Additional evidence
         ↓
Day 3-5: Arbiter deliberation
         • Impartial analysis
         • Platform rules verification
         ↓
Day 5-7: Decision + Execution
         • Decision publication
         • Transaction signatures
         • Payment/Refund
         ↓
Day 7: ✅ RESOLVED
```

**Average duration:** 3-5 days
**Maximum:** 7 days (after = support escalation)

#### Q9: Can arbiter make mistakes?

**YES, it's possible** (arbiters = humans).

**Protection mechanisms:**

1. **Appeal (once):**
   ```
   If decision unfair:
     → Button [⚖️ Appeal]
     → New senior arbiter reviews case
     → Final decision (no appeal after)
   ```

2. **Reputation system:**
   ```
   After resolution:
     → Rate arbiter 1-5 ⭐
     → Optional comment
     → Affects arbiter reputation
     → Bad-rated arbiters = suspended
   ```

3. **Quality monitoring:**
   ```
   Platform analyzes:
     • Arbiter satisfaction rates
     • Decision consistency
     • Resolution time
     → Problem arbiters = removed
   ```

---

### ⏱️ Timeouts & Expirations

#### Q10: What happens if escrow expires?

**Standard timeouts:**

| Phase | Timeout | Action if expired |
|-------|---------|-------------------|
| **Buyer payment** | 7 days | Escrow automatically cancelled |
| **Vendor shipping** | 14 days | Automatic buyer refund |
| **Buyer confirmation** | 30 days | Automatic vendor release |
| **Dispute resolution** | 7 days | Support escalation |

**Example: Vendor doesn't ship**

```
Day 0: Escrow active (buyer paid)
Day 1-14: Waiting for vendor shipment
          ⏳ Visible countdown: "13 days remaining"
Day 14: ⚠️ Shipping deadline reached
        🚨 Vendor notification: "Ship now or refund"
Day 15: ❌ Vendor still hasn't shipped
        ✅ AUTOMATIC REFUND activated
           • Signatures: Buyer + Arbiter (auto)
           • Refund transaction broadcast
           • Buyer recovers funds
           • Vendor: -20 reputation (heavy penalty)
```

---

## 8. Troubleshooting

### ❌ Problem 1: "Multisig address not generated"

**Symptoms:**
```
Escrow status: created (stuck)
Error: "Multisig setup failed"
No multisig address displayed
```

**Possible causes:**
1. Monero RPC offline
2. Corrupted temporary wallets
3. Multisig setup timeout

**Solutions:**

```
Solution A: Refresh page (30 sec)
  → Multisig setup retries automatically
  → If success: address appears

Solution B: Recreate escrow
  → Cancel current order
  → Place order again
  → New multisig setup

Solution C: Contact support
  → Button [💬 Support]
  → Provide escrow ID
  → Support forces backend retry
```

---

### ❌ Problem 2: "Payment not detected"

**Symptoms:**
```
Buyer paid, but:
  Status: "Awaiting Payment"
  Confirmations: 0/10 (stuck)
```

**Verifications:**

```
1. TX in buyer wallet?
   → Open Feather/GUI
   → "Transactions" tab
   → TX appears = ✅ Sent
   → TX doesn't appear = ❌ Not sent (redo)

2. Correct destination address?
   → Copy multisig address from platform
   → Compare with TX in wallet
   → If different = ❌ WRONG ADDRESS (money lost)
   → If identical = ✅ Correct address

3. Correct amount?
   → Verify amount in TX
   → Compare with requested amount
   → If < amount = ❌ Insufficient (complete)
   → If = amount = ✅ Correct

4. Confirmations?
   → Wait for 10 confirmations (~20 min)
   → If stuck after 30 min = Monero network issue
   → Solution: Wait, blockchain synchronizes
```

**If everything verified and still stuck:**
```
→ Button [🔍 Verify Payment Manually]
→ Enter TX ID
→ Server checks blockchain directly
→ If found: Escrow activated manually
```

---

### ❌ Problem 3: "Cannot release funds"

**Symptoms:**
```
Buyer clicks "Release Funds"
Error: "Failed to sign transaction"
```

**Possible causes:**
1. Temporary wallets closed (normal)
2. Monero RPC offline
3. Corrupted multisig keys

**Solutions:**

```
Automatic (90% cases):
  → Server reopens wallets automatically
  → Retry transaction
  → Success: Funds released

Manual (10% cases):
  → Escrow goes to "manual_resolution"
  → Support creates transaction manually
  → Buyer + Arbiter sign off-platform
  → Funds released (delay: 1-2 days)
```

---

### ❌ Problem 4: "Escrow stuck in limbo"

**Symptoms:**
```
Status: active (for 60 days)
Buyer + Vendor disappeared
No action possible
```

**Automatic protection:**

```
Timeout monitor system:
  ↓
Day 30: Buyer notification "Confirm receipt"
Day 35: Vendor notification "Buyer unresponsive"
Day 40: Arbiter notification "Intervention required"
        ↓
Day 41-45: Arbiter attempts party contact
           • Email (if provided)
           • In-app messages
           • Push notifications
        ↓
Day 45: If still no response:
        → Automatic analysis:
           • If tracking = delivered → Vendor payment
           • If tracking = not delivered → Buyer refund
           • If no tracking → 50/50 split (rare)
        ↓
Day 46: ✅ RESOLVED automatically
```

---

## 9. Technical Glossary

### 🔐 Cryptographic Terms

#### Multisig (Multi-Signature)
Technology allowing creation of a wallet requiring multiple signatures to send funds.

**Example:**
```
Normal Wallet:
  1 private key = Total control

Multisig Wallet 2/3:
  3 private keys exist
  Minimum 2 signatures required
  1 signature alone = Blocked
```

#### Threshold
Minimum number of signatures required in a multisig.

**Our case:** Threshold = 2 (hence "2-of-3")

#### Atomic Units
Smallest Monero unit (picomonero).

**Conversion:**
```
1 XMR = 1,000,000,000,000 atomic units (12 zeros)
0.5 XMR = 500,000,000,000 atomic units
0.001 XMR = 1,000,000,000 atomic units (1 milliXMR)
```

#### Confirmations
Number of blocks added after your transaction on blockchain.

**Security:**
```
0 conf = Transaction sent (not yet validated)
1 conf = 1 block added (~2 min)
10 conf = 10 blocks added (~20 min) ✅ Secure
```

---

### 💼 Escrow Terms

#### Escrow (Security Deposit)
Mechanism where a neutral third party holds money until conditions are met.

**Centralized version (classic):**
```
Buyer → [Platform holds money] → Vendor
        ⚠️ Platform = Single point of failure
```

**Decentralized version (our system):**
```
Buyer → [Blockchain Multisig 2/3] → Vendor
        ✅ No single point of failure
```

#### Non-Custodial
Principle where platform NEVER holds user funds.

**Advantages:**
- ✅ No platform hack risk
- ✅ No bankruptcy with your funds
- ✅ You always control (with 1 other party)

#### Release
Action to unlock escrow funds to send to vendor.

**Required signatures:** Buyer + Arbiter (or Buyer + Vendor if implemented)

#### Refund
Action to return escrow funds to buyer.

**Required signatures:** Buyer + Arbiter (or Vendor + Arbiter)

---

### 📊 Monero Terms

#### RingCT (Ring Confidential Transactions)
Monero technology that hides transaction amounts.

**Result:** Nobody can see how much you're sending.

#### Stealth Addresses
Monero technology that hides recipient address.

**Result:** Nobody can see who you're sending to.

#### View Key
Key allowing to see wallet transactions without being able to spend.

**Usage:** Audit, accounting (without theft risk)

#### Spend Key
Key allowing to spend wallet funds.

**Security:** NEVER share this key.

---

### 🛠️ Technical Terms

#### RPC (Remote Procedure Call)
Interface allowing communication with Monero wallet via code.

**Our usage:**
```
Server → [RPC Command] → monero-wallet-rpc
         prepare_multisig()
         make_multisig()
         transfer_multisig()
         etc.
```

#### WebSocket
Real-time communication technology between server and browser.

**Our usage:**
```
Server detects: Payment confirmed
  ↓
WebSocket broadcast
  ↓
Buyer browser: Instant notification ✅
```

#### Wallet Pool
System allowing reuse of Monero RPCs for multiple escrows.

**Advantage:** Scalability (1000 escrows with 9 RPCs instead of 3000)

---

## 📚 Additional Resources

### Official Documentation
- [Monero Multisig Documentation](https://github.com/monero-project/monero/blob/master/docs/multisig.md)
- [Monero RPC Documentation](https://www.getmonero.org/resources/developer-guides/wallet-rpc.html)

### Video Tutorials
- 🎬 "How to create your first Monero wallet" (10 min)
- 🎬 "Make your first purchase on Marketplace" (15 min)
- 🎬 "Resolve a dispute as arbiter" (20 min)

### Support
- 💬 Support Chat: Available 24/7
- 📧 Email: support@moneromarketplace.onion
- 🔐 PGP Key: [Download]

---

**End of Multisig 2/3 Guide**
*Updated: 2025-11-05 | Version 1.0*
