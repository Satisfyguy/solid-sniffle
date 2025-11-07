# Threat Model: Monero Marketplace

## Overview

Comprehensive threat analysis for the Monero Marketplace escrow platform, covering adversaries, attack vectors, mitigations, and residual risks.

**Methodology:** STRIDE (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege)
**Last Updated:** 2025-11-07
**Next Review:** Quarterly

---

## System Overview

```
┌─────────────┐        Tor          ┌──────────────────┐
│   Buyer     │◄─────────────────────┤  Hidden Service  │
└─────────────┘                      │  (Marketplace)   │
                                     └────────┬─────────┘
┌─────────────┐        Tor                   │
│   Vendor    │◄─────────────────────────────┤
└─────────────┘                              │
                                             │
┌─────────────┐                              │
│   Arbiter   │◄─────────────────────────────┤
└─────────────┘                              │
                                             ▼
                                   ┌──────────────────┐
                                   │  Monero Testnet  │
                                   │  (Wallet RPC)    │
                                   └──────────────────┘
```

**Trust Boundaries:**
1. User → Hidden Service (Tor)
2. Hidden Service → Monero RPC (localhost)
3. Hidden Service → Database (SQLCipher)

---

## Adversary Profiles

### 1. Script Kiddie (Low Sophistication)

**Motivation:** Fame, curiosity, learning
**Capabilities:**
- ✅ Automated scanning tools (nmap, sqlmap, ZAP)
- ✅ Public exploits (Metasploit modules)
- ❌ Custom exploit development
- ❌ Traffic analysis

**Attack Vectors:**
- SQL injection attempts
- XSS via input fields
- Brute force authentication
- Directory traversal

**Mitigations:**
- ✅ Parameterized queries (Diesel ORM)
- ✅ Input validation (validator crate)
- ✅ Rate limiting (actix-governor)
- ✅ Secure file handling

**Residual Risk:** LOW - Automated defenses sufficient

---

### 2. Malicious User (Medium Sophistication)

**Motivation:** Financial gain, fraud
**Capabilities:**
- ✅ Account creation/abuse
- ✅ Social engineering
- ✅ Basic crypto knowledge
- ❌ Infrastructure attacks

**Attack Vectors:**

#### Buyer Fraud
- Create escrow, claim non-delivery, dispute
- Sybil attack (multiple accounts)
- Payment proof forgery

**Mitigations:**
- ✅ 2-of-3 multisig (requires arbiter)
- ✅ Reputation system (track disputes)
- ⚠️ KYC/AML (not implemented - Alpha)

#### Vendor Fraud
- Accept payment, never ship
- Ship empty package, claim delivered
- Selective scamming (high-value orders only)

**Mitigations:**
- ✅ Escrow holds funds until confirmation
- ✅ Dispute resolution by arbiter
- ✅ Reputation system flags bad vendors

**Residual Risk:** MEDIUM - Sophisticated fraudsters may succeed initially

---

### 3. Malicious Arbiter (High Sophistication)

**Motivation:** Financial gain, sabotage
**Capabilities:**
- ✅ Access to all dispute details
- ✅ Authority to move escrowed funds
- ✅ Pattern analysis of users
- ❌ Cannot steal funds alone (requires 2-of-3)

**Attack Vectors:**

#### Collusion
- Arbiter + Buyer: Always rule against vendor
- Arbiter + Vendor: Always rule against buyer
- Selective bias for bribes

**Mitigations:**
- ✅ Dispute resolution tracked/logged
- ✅ Bias detection alerts (Prometheus)
- ⚠️ Multi-arbiter rotation (not implemented)
- ⚠️ Reputation system for arbiters (planned)

#### Information Abuse
- Sell dispute data (addresses, amounts)
- Deanonymize users via timing analysis
- Blackmail users with sensitive info

**Mitigations:**
- ✅ Encryption at rest (SQLCipher)
- ✅ Access logs for audit
- ⚠️ Air-gapped arbiter key (TM-006)
- ❌ Multi-party computation (future)

**Residual Risk:** MEDIUM-HIGH - Single arbiter is single point of failure

---

### 4. Nation-State Actor (Very High Sophistication)

**Motivation:** Surveillance, enforcement, disruption
**Capabilities:**
- ✅ Traffic analysis at ISP level
- ✅ Tor node operation
- ✅ Zero-day exploits
- ✅ Server seizure
- ✅ Lawful intercept orders

**Attack Vectors:**

#### Traffic Analysis
- Correlation attacks (entry/exit timing)
- Website fingerprinting
- Volume analysis

**Mitigations:**
- ✅ Tor hidden service (no exit node)
- ✅ Constant-time operations where possible
- ⚠️ Dummy traffic padding (not implemented)

#### Server Compromise
- Physical seizure
- Remote exploit (0-day)
- Supply chain attack (dependencies)

**Mitigations:**
- ✅ Full disk encryption
- ✅ Database encryption (SQLCipher)
- ✅ Shamir 3-of-5 key splitting
- ✅ Automated dependency audits
- ⚠️ Dead man's switch (not implemented)

#### Deanonymization
- Blockchain analysis (Monero)
- Timing correlation
- Operational security failures

**Mitigations:**
- ✅ Monero privacy features (RingCT, stealth addresses)
- ✅ No on-chain metadata
- ⚠️ User OPSEC education required

**Residual Risk:** HIGH - NSA-level adversaries may succeed

---

## STRIDE Analysis

### Spoofing (Identity)

| Threat | Impact | Likelihood | Mitigation | Residual Risk |
|--------|--------|------------|------------|---------------|
| Stolen session cookie | Account takeover | Medium | HttpOnly + SameSite + short TTL | Low |
| Fake multisig info | Escrow theft | High | Challenge-response validation (TM-003) | Low |
| Arbiter impersonation | Unauthorized resolutions | Critical | Ed25519 signatures | Low |
| Sybil attack | Reputation manipulation | Medium | Rate limiting + cost of entry | Medium |

---

### Tampering (Data Integrity)

| Threat | Impact | Likelihood | Mitigation | Residual Risk |
|--------|--------|------------|------------|---------------|
| SQL injection | Data corruption/theft | High | Parameterized queries (Diesel) | Low |
| CSRF | Unauthorized actions | Medium | CSRF tokens + SameSite cookies | Low |
| Man-in-the-Middle | Traffic modification | Low | Tor end-to-end encryption | Low |
| Database tampering | Escrow manipulation | Critical | SQLCipher + checksums | Low |

---

### Repudiation (Non-repudiation)

| Threat | Impact | Likelihood | Mitigation | Residual Risk |
|--------|--------|------------|------------|---------------|
| "I didn't place that order" | Dispute complexity | Medium | Signed transactions + logs | Low |
| "Arbiter changed decision" | Trust erosion | Low | Immutable audit logs | Low |
| "Never received payment" | Payment disputes | High | On-chain verification | Low |

---

### Information Disclosure

| Threat | Impact | Likelihood | Mitigation | Residual Risk |
|--------|--------|------------|------------|---------------|
| Database breach | User data leak | High | SQLCipher encryption | Medium |
| Memory dump | Keys exposed | Medium | Memory scrubbing (zeroize) | Medium |
| Log leakage | .onion addresses leaked | Medium | Sanitized logging | Low |
| Timing attacks | Amount estimation | Low | Constant-time crypto | Low |

---

### Denial of Service

| Threat | Impact | Likelihood | Mitigation | Residual Risk |
|--------|--------|------------|------------|---------------|
| DDoS (Layer 7) | Service unavailable | High | Rate limiting | Medium |
| Resource exhaustion | Server crash | Medium | Connection limits + timeouts | Low |
| Database locking | Queries blocked | Low | Connection pooling | Low |
| Tor DoS | Hidden service down | Medium | Multiple .onion addresses | High |

---

### Elevation of Privilege

| Threat | Impact | Likelihood | Mitigation | Residual Risk |
|--------|--------|------------|------------|---------------|
| Admin panel access | Platform takeover | Critical | Role-based auth + MFA | Low |
| Arbiter privilege escalation | Unauthorized resolutions | High | Strict RBAC | Low |
| SQL injection to admin | Database control | High | Parameterized queries | Low |
| Container escape | Host compromise | Low | Sandboxing + AppArmor | Low |

---

## Attack Scenarios

### Scenario 1: Vendor Exit Scam

**Attacker:** Malicious vendor
**Goal:** Steal funds and disappear

**Attack Steps:**
1. Build reputation (complete 10+ legit orders)
2. Accept large order (5 XMR)
3. Receive escrow funding
4. Never ship product
5. Buyer disputes
6. Arbiter rules in buyer favor
7. **Result:** Funds returned to buyer ✅

**Why Attack Fails:**
- Escrow prevents vendor access until completion
- Arbiter has no incentive to collude (reputation)
- Reputation system flags suspicious behavior

---

### Scenario 2: Arbiter Collusion

**Attacker:** Malicious arbiter + Vendor cartel
**Goal:** Steal from buyers via biased disputes

**Attack Steps:**
1. Vendor cartel coordinates with arbiter
2. Buyers place orders
3. Vendors never ship
4. Buyers dispute
5. Arbiter always rules in vendor favor
6. **Detection:** Bias alert fires after 10 disputes

**Why Attack Detected:**
```promql
# Prometheus alert
(disputes_vendor_won_total / (disputes_buyer_won_total + disputes_vendor_won_total)) > 0.8
```

**Mitigation:**
- Automated bias detection
- Arbiter rotation (planned)
- Multi-arbiter consensus (future)

---

### Scenario 3: Database Seizure

**Attacker:** Law enforcement (nation-state)
**Goal:** Deanonymize users and transactions

**Attack Steps:**
1. Seize server (physical access)
2. Extract marketplace.db file
3. Attempt decryption
4. **Blocked:** SQLCipher + Shamir 3-of-5

**Why Attack Fails:**
- Database encrypted with AES-256-GCM
- Encryption key split via Shamir (need 3-of-5 shares)
- Shares held by 5 different entities (not on server)
- Even with database, Monero transactions are private

---

### Scenario 4: Supply Chain Attack

**Attacker:** Malicious dependency maintainer
**Goal:** Inject backdoor via compromised crate

**Attack Steps:**
1. Compromise popular Rust crate (e.g., `serde_json`)
2. Add malicious code to new version
3. Marketplace auto-updates dependency
4. **Blocked:** cargo-audit detects anomaly

**Why Attack Detected:**
```bash
# Daily audit (CI/CD)
cargo audit
# → RUSTSEC-2025-XXXX: Backdoor in serde_json
# → CI blocks deployment
# → GitHub issue created automatically
```

**Mitigation:**
- Automated dependency audits (daily)
- Cargo.lock pinning
- Manual review of major updates

---

## Mitigations Summary

### Implemented (A- Grade)

| Control | Effectiveness | Cost |
|---------|---------------|------|
| Placeholder validation | High | None |
| Property-based tests | High | Low |
| Dependency audits | High | Low |
| Prometheus monitoring | Medium | Medium |
| SQLCipher encryption | High | Low |
| 2-of-3 multisig | High | Medium |
| Tor integration | High | Medium |
| Rate limiting | Medium | None |

### Planned (A → A+)

| Control | Effectiveness | Cost | Priority |
|---------|---------------|------|----------|
| Multi-arbiter (3-of-5) | High | High | P0 |
| Chaos engineering tests | Medium | Medium | P1 |
| Air-gapped arbiter | High | Medium | P1 |
| Horizontal scaling | Medium | High | P2 |
| MFA for admin | High | Low | P0 |

---

## Residual Risks

### Accepted Risks (Alpha)

1. **Single arbiter** - Accept risk for Alpha, mitigate in Beta
2. **No horizontal scaling** - Single point of failure accepted
3. **Manual dispute resolution** - Arbiter workload manageable for Alpha
4. **Testnet only** - No real value at risk

### Risk Acceptance Criteria

**For Production:**
- [ ] Multi-arbiter implemented
- [ ] Horizontal scaling deployed
- [ ] Chaos tests passing
- [ ] External security audit (Grade A+)
- [ ] Bug bounty program active

---

## Review Schedule

**Frequency:** Quarterly or after major releases
**Next Review:** 2025-12-07
**Responsible:** Security Team

**Triggers for emergency review:**
- Critical vulnerability disclosed
- Successful attack reported
- New adversary capability identified
- Major architecture change

---

## Related Documentation

- **Security Policy:** [SECURITY.md](../../SECURITY.md)
- **Incident Response:** [INCIDENT-RESPONSE.md](INCIDENT-RESPONSE.md)
- **Placeholder Validation:** [PLACEHOLDER-VALIDATION.md](PLACEHOLDER-VALIDATION.md)
- **Dependency Audits:** [DEPENDENCY-AUDIT.md](DEPENDENCY-AUDIT.md)
- **Monitoring:** [../monitoring/PROMETHEUS-MONITORING.md](../monitoring/PROMETHEUS-MONITORING.md)

---

**Version:** 1.0.0
**Classification:** Public
**Maintainer:** Security Team
