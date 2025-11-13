# ADR-001: Monero-Only Architecture

**Date:** 2025-11-13
**Status:** Accepted
**Version:** 1.0
**Auteur:** Project Team

---

## TL;DR

**Monero-only is not a limitation—it's a non-negotiable architectural requirement.**

This marketplace exclusively supports Monero (XMR). Multi-cryptocurrency support would fundamentally undermine the privacy guarantees that define this project's purpose. The reduced market size is an intentional trade-off for cryptographic integrity.

---

## Context

### The Question

"Isn't a Monero-only marketplace too niche?"

This question surfaces periodically and reflects a fundamental misunderstanding: **niche size is being evaluated as a weakness when it's actually a direct consequence of architectural rigor.**

### Project Mandate

From the beginning, this project has had a clear mandate:
- **Privacy-first marketplace** with Tor hidden service
- **Non-custodial escrow** using 2-of-3 multisig
- **Zero compromise on OpSec** (no security theatre)
- **Educational platform** for high-security system design

---

## Decision

**The marketplace supports ONLY Monero (XMR). No other cryptocurrencies will be integrated.**

This includes:
- ❌ No Bitcoin (BTC)
- ❌ No Ethereum (ETH)
- ❌ No other altcoins
- ❌ No fiat payment rails
- ❌ No multi-crypto architecture

---

## Rationale

### Technical Reality Matrix

| Dimension | Multi-Crypto Approach | Monero-Only Approach |
|-----------|----------------------|----------------------|
| **Volume Potential** | High (appeals to broader market) | Structurally limited (~0.5% crypto market cap) |
| **Privacy Guarantees** | Degraded (transparent chains leak data) | Maximized (protocol-level unlinkability) |
| **Fungibility** | Compromised (tainted coins, blacklists) | Guaranteed (all XMR identical at protocol) |
| **OpSec Integrity** | Multiple attack surfaces | Single, well-defined threat model |
| **Technical Complexity** | 5-10x (multiple RPC clients, sync flows) | Baseline (single implementation) |
| **Maintenance** | Nightmare (chain-specific bugs) | Manageable (focused expertise) |

### Why Other Cryptocurrencies Fail

**Bitcoin:**
- Fully transparent blockchain (all transactions public)
- Chain analysis companies (Chainalysis, Elliptic) can trace flows
- CoinJoin/Lightning add complexity without guarantees
- "Privacy by adoption" is not a technical property
- **Incompatible with "privacy-first" mandate**

**Ethereum & Smart Contract Chains:**
- Even worse for privacy (MEV, public mempool)
- Unpredictable gas fees
- Complex attack surface
- Zero privacy by design

**"Privacy Coins" (Zcash, etc.):**
- Optional privacy = weak privacy (shielded pool too small)
- Trusted setup concerns (Zcash)
- Smaller ecosystems than Monero
- No proven track record in adversarial environments

**Multi-Crypto Architecture:**
- Complexity explosion (5-10x codebase size)
- Every chain = new RPC implementation, new multisig flow, new attack surface
- Dilutes value proposition: "privacy when convenient" vs "privacy by default"
- Security vulnerabilities compound

### What Monero Provides

**Protocol-Level Privacy:**
- Ring signatures (sender anonymity)
- Stealth addresses (receiver anonymity)
- RingCT (amount obfuscation)
- **Unlinkability:** Cannot determine if two transactions involve same parties
- **Fungibility:** All coins are equal (no "tainted" XMR)

**Proven Track Record:**
- Adopted by all modern darknet markets (8/10 as of 2025)
- Survives in adversarial environments (nation-state surveillance)
- ~50-100K daily active users despite regulatory pressure
- LocalMonero: ~$2M monthly P2P volume

**Architectural Simplicity:**
- Single RPC implementation
- One multisig protocol to secure
- Focused threat model
- Deep expertise in one domain > shallow knowledge of many

---

## Trade-Offs Accepted

### ✅ Benefits (Why Monero-Only Wins)

1. **Privacy guarantees are verifiable and measurable**
   - No blockchain analysis possible
   - No transaction graph leaks
   - No "mixing" required—private by default

2. **OpSec integrity maintained**
   - Single attack surface to harden
   - Clear threat model
   - No "weakest link" chain

3. **Technical honesty**
   - "Privacy-first" is backed by cryptography, not marketing
   - No false claims about "private" Bitcoin transactions

4. **Maintainability**
   - Focused codebase
   - Single RPC client to maintain
   - Deep Monero expertise > shallow multi-chain knowledge

### ❌ Costs (What We Accept)

1. **Market size is structurally limited**
   - Monero is ~0.5% of total crypto market cap
   - ~50-100K daily active users (vs millions for BTC/ETH)
   - Higher user acquisition friction (requires KYC exchanges or P2P)

2. **Steeper learning curve**
   - Users must acquire XMR first
   - More complex for mainstream adoption

3. **Regulatory pressure**
   - Some exchanges delisting privacy coins
   - Increased scrutiny from authorities

4. **Perception challenge**
   - "Too niche" questions persist
   - Must educate users on why privacy matters

---

## Alternatives Considered and Rejected

### Alternative 1: Start with BTC, Add XMR Later

**Rejected because:**
- Bitcoin first = privacy debt from day 1
- Users with BTC history are already deanonymized
- Impossible to "add privacy later" (blockchain is permanent)
- Dilutes brand: "marketplace with optional privacy" vs "privacy marketplace"

### Alternative 2: Multi-Crypto with "Privacy Mode"

**Rejected because:**
- Complexity explosion (5-10x maintenance burden)
- "Privacy mode" = security theatre (optional privacy = weak privacy)
- Attack surface increases with every chain
- Users will choose convenience over privacy (default matters)

### Alternative 3: BTC + CoinJoin/Lightning

**Rejected because:**
- CoinJoin: Not protocol-level, requires coordination, identifiable on-chain
- Lightning: Routing nodes see amounts, privacy research still early
- Both add massive complexity without guaranteed privacy
- Chainalysis already analyzing both techniques

### Alternative 4: Layer-2 "Privacy Wrapper"

**Rejected because:**
- Bridges introduce custodial risk
- Wrapped tokens = counterparty risk
- Defeats purpose of non-custodial escrow
- Privacy at L2 doesn't hide L1 entry/exit

---

## Consequences

### Positive

1. **Clear value proposition**
   - "Privacy-first marketplace. No compromises."
   - Attracts users who actually value privacy
   - Filters out users seeking convenience over security

2. **Technical credibility**
   - Architecture matches claims
   - No security theatre
   - Demonstrates understanding of threat models

3. **Focused development**
   - Resources go to quality, not breadth
   - Deep Monero expertise
   - Manageable codebase

4. **Long-term sustainability**
   - As surveillance increases, privacy demand grows
   - Monero adoption trending up since 2020
   - Market is small but real and expanding

### Negative

1. **Volume is inherently limited**
   - Will never compete with mainstream marketplaces on transaction count
   - Harder to attract VC funding (if that were a goal)
   - Growth ceiling exists

2. **User acquisition friction**
   - Each user must acquire XMR first
   - Steeper onboarding than BTC/ETH
   - Requires user education

3. **Regulatory scrutiny**
   - Privacy coins face regulatory pressure
   - Some exchanges delisting XMR
   - Must be prepared for legal challenges

---

## Implementation Notes

### What This Means for Code

1. **No multi-crypto abstraction layer needed**
   - `MoneroRpcClient` is the only payment backend
   - No `trait CryptoBackend` architecture required
   - Simplifies wallet management

2. **Escrow is Monero-specific**
   - 2-of-3 multisig uses Monero's protocol
   - No need to support other multisig schemes
   - Deep integration with monero-wallet-rpc

3. **Documentation assumes XMR**
   - No "if using BTC..." conditional logic
   - Clear, focused guides
   - Single threat model to document

### What This Means for Users

1. **Prerequisites**
   - User must have XMR wallet
   - User must run monero-wallet-rpc locally (non-custodial)
   - User must understand basic Monero concepts

2. **Benefits**
   - No blockchain surveillance
   - True financial privacy
   - Fungible money (no blacklists)

3. **Trade-offs**
   - Must acquire XMR (P2P or exchange)
   - Slightly longer transaction times (~20 min confirmations)
   - Must trust Monero's cryptography (which is peer-reviewed)

---

## Related Documentation

- [MONERO-MULTISIG-2OF3-COMPLETE-GUIDE.md](MONERO-MULTISIG-2OF3-COMPLETE-GUIDE.md) - Technical implementation
- [NON-CUSTODIAL-USER-GUIDE.md](../guides/NON-CUSTODIAL-USER-GUIDE.md) - User setup instructions
- [CLAUDE.md](../../CLAUDE.md) - Design Philosophy section

---

## Evaluation Criteria

**The question "Is Monero-only too niche?" is incorrectly framed.**

The correct question is: **"Does the architecture deliver on its privacy promise?"**

| Evaluation | Result |
|------------|--------|
| Privacy guaranteed at protocol level? | ✅ Yes (unlinkability, fungibility) |
| Non-custodial escrow working? | ✅ Yes (2-of-3 multisig) |
| Tor integration secure? | ✅ Yes (hidden service only) |
| No security theatre? | ✅ Yes (automated enforcement) |
| OpSec integrity maintained? | ✅ Yes (single threat model) |

**Verdict:** The niche size is a direct consequence of architectural integrity. This is success, not failure.

---

## Revision History

- **v1.0 (2025-11-13):** Initial ADR documenting Monero-only decision and rationale
