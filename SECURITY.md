# Security Policy

## Overview

Monero Marketplace is a high-security, privacy-focused escrow platform built with **zero tolerance for security theatre**. This document describes our security posture, responsible disclosure process, and threat model.

**Security Grade:** A- (as of 2025-11-07)
**Audit Trail:** [DOX/security/AUDIT-IMPROVEMENTS-2025-11-07.md](DOX/security/AUDIT-IMPROVEMENTS-2025-11-07.md)

---

## Supported Versions

| Version | Supported          | Notes |
| ------- | ------------------ | ----- |
| 0.2.x   | :white_check_mark: | Alpha - Testnet only |
| < 0.2.0 | :x:                | Deprecated |

**Current Status:** Alpha (v0.2.6) - **NOT FOR PRODUCTION USE**

---

## Reporting a Vulnerability

### Quick Response

We take security vulnerabilities seriously and respond within **24 hours**.

**Report security issues to:** security@example.com (replace with actual)

**DO NOT** open public GitHub issues for security vulnerabilities.

---

### Reporting Process

1. **Email us** at security@example.com with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if applicable)

2. **Expect acknowledgment** within 24 hours

3. **Disclosure timeline:**
   - **Critical:** Fix within 48 hours, public disclosure after 7 days
   - **High:** Fix within 7 days, public disclosure after 30 days
   - **Medium/Low:** Fix within 30 days, public disclosure after 90 days

4. **Credit:** We acknowledge security researchers in our CHANGELOG and SECURITY.md

---

### Vulnerability Severity Classification

| Severity | Examples | Response Time |
|----------|----------|---------------|
| **Critical** | RCE, Auth bypass, Private key exposure | < 4 hours |
| **High** | XSS, SQL injection, CSRF bypass | < 24 hours |
| **Medium** | Information disclosure, DoS | < 7 days |
| **Low** | Minor info leak, rate limit bypass | < 30 days |

---

### Bug Bounty Program

**Status:** Not currently active (Alpha phase)

**Future:** Bug bounty program planned for Beta/Production releases

**Rewards:** Recognition in CHANGELOG + optional XMR bounty (TBD)

---

## Security Architecture

### Core Principles

1. **Non-Custodial** - Users control private keys
2. **2-of-3 Multisig** - Buyer + Vendor + Arbiter escrow
3. **Tor-Only** - Hidden service deployment
4. **End-to-End Encrypted** - SQLCipher + Monero privacy
5. **Shamir 3-of-5** - Database key protection (TM-002)

### Threat Model

Full threat model: [DOX/security/THREAT-MODEL.md](DOX/security/THREAT-MODEL.md)

**Adversaries considered:**
- 🔴 **Nation-State Actors** - Surveillance, traffic analysis
- 🟠 **Malicious Arbiters** - Collusion, bias
- 🟡 **Exit Scammers** - Vendor fraud, buyer fraud
- 🟢 **Script Kiddies** - Automated attacks, SQLi, XSS

**Out of scope:**
- Physical attacks (server seizure)
- Social engineering (phishing)
- Zero-day exploits in dependencies (mitigated via automated audits)

---

## Security Features

### Implemented Protections

#### 1. Placeholder Validation (2025-11-07)

**Risk:** Deployment with `.env.example` credentials
**Mitigation:** Automated detection + production panic
**Documentation:** [DOX/security/PLACEHOLDER-VALIDATION.md](DOX/security/PLACEHOLDER-VALIDATION.md)

```rust
// Blocks startup if placeholder detected
server::security::placeholder_validator::validate_all_critical_env_vars();
```

---

#### 2. Property-Based Testing (2025-11-07)

**Risk:** Edge cases in crypto operations
**Mitigation:** 10,000+ automated test scenarios
**Documentation:** [DOX/security/PROPERTY-BASED-TESTING.md](DOX/security/PROPERTY-BASED-TESTING.md)

```bash
# Run comprehensive crypto validation
./scripts/run-property-tests.sh thorough
```

---

#### 3. Automated Dependency Audits (2025-11-07)

**Risk:** Vulnerable dependencies (Log4Shell-style)
**Mitigation:** Daily CVE scanning + auto-alerts
**Documentation:** [DOX/security/DEPENDENCY-AUDIT.md](DOX/security/DEPENDENCY-AUDIT.md)

```bash
# Check for known vulnerabilities
./scripts/audit-dependencies.sh
```

---

#### 4. Prometheus Monitoring (2025-11-07)

**Risk:** Undetected incidents
**Mitigation:** Real-time alerting, < 1min MTTD
**Documentation:** [DOX/monitoring/PROMETHEUS-MONITORING.md](DOX/monitoring/PROMETHEUS-MONITORING.md)

```bash
# Setup monitoring stack
./scripts/setup-monitoring.sh
```

---

### Authentication & Authorization

- **Password Hashing:** Argon2id (OWASP recommended)
- **Session Management:** HttpOnly + SameSite cookies
- **CSRF Protection:** Token validation on all state-changing operations
- **Rate Limiting:** Actix-Governor (100 req/min global, 5 req/15min auth)
- **Role-Based Access:** Buyer, Vendor, Arbiter, Admin roles

---

### Cryptography

- **Escrow:** 2-of-3 multisig (Monero)
- **Database:** SQLCipher AES-256-GCM
- **Key Protection:** Shamir 3-of-5 secret sharing
- **Challenge-Response:** Ed25519 signatures (TM-003)
- **Hashing:** Blake2b for challenges, Argon2id for passwords

---

### Network Security

- **Tor Integration:** All traffic via SOCKS5h://127.0.0.1:9050
- **RPC Isolation:** Localhost-only (127.0.0.1) validation
- **TLS:** Required for clearnet (testing only)
- **HSTS:** Enabled (clearnet deployments)
- **CSP:** Strict Content Security Policy

---

## Security Checklist

### Pre-Production Deployment

- [ ] All `.env` variables set (no placeholders)
- [ ] `SESSION_SECRET_KEY` generated (64+ bytes)
- [ ] `DB_ENCRYPTION_KEY` generated (64 hex)
- [ ] Shamir shares distributed (3-of-5)
- [ ] Arbiter public key configured
- [ ] Tor daemon running and verified
- [ ] Monero testnet synced (or mainnet for prod)
- [ ] All tests passing: `cargo test --workspace`
- [ ] Property tests thorough: `./scripts/run-property-tests.sh thorough`
- [ ] Dependency audit clean: `./scripts/audit-dependencies.sh`
- [ ] Security theatre check: `./scripts/check-security-theatre.sh`
- [ ] Monitoring configured: `./scripts/setup-monitoring.sh`
- [ ] Backup strategy documented
- [ ] Incident response plan ready

### Post-Deployment Monitoring

- [ ] Prometheus scraping `/metrics`
- [ ] Alertmanager receiving alerts
- [ ] Grafana dashboard accessible
- [ ] Daily dependency audits enabled
- [ ] Log aggregation configured
- [ ] Backup cron jobs running
- [ ] Incident response team on-call

---

## Known Limitations (Alpha)

### Accepted Risks

1. **Testnet Only** - Uses Monero testnet, not real value
2. **Single Arbiter** - No arbiter redundancy yet
3. **Manual Dispute Resolution** - Arbiter decisions manual
4. **No Rate Limiting on Some Endpoints** - Temporarily disabled for testing
5. **No Horizontal Scaling** - Single server deployment

### Planned Improvements (Beta)

- [ ] Multi-arbiter support (3-of-5)
- [ ] Automated dispute resolution (evidence-based)
- [ ] Rate limiting re-enabled everywhere
- [ ] Horizontal scaling (load balancer + replicas)
- [ ] Disaster recovery automation
- [ ] Chaos engineering tests (network failures)

---

## Security Testing

### Automated Tests

```bash
# Full security test suite
./scripts/run-security-tests.sh

# Components:
# 1. Unit tests (cargo test)
# 2. Property-based tests (10,000+ scenarios)
# 3. Integration tests (E2E escrow flows)
# 4. Dependency audit (CVE scanning)
# 5. Security theatre detection
# 6. RPC validation tests
```

**Coverage:** 85%+ (target: 90%)

---

### Manual Security Testing

**Recommended tools:**
- **ZAP/Burp Suite:** Web vulnerability scanning
- **sqlmap:** SQL injection testing (should fail)
- **nmap:** Port scanning (only 8080 should be open)
- **Tor Browser:** Hidden service functionality
- **Wireshark:** Traffic analysis (verify Tor usage)

**Test scenarios:** See [DOX/security/PENETRATION-TESTING.md](DOX/security/PENETRATION-TESTING.md)

---

## Incident Response

### Severity Levels

| Level | Response Time | Actions |
|-------|---------------|---------|
| **P0 - Critical** | < 15 min | Page on-call, emergency patch |
| **P1 - High** | < 1 hour | Investigate immediately, schedule fix |
| **P2 - Medium** | < 24 hours | Create ticket, fix in next sprint |
| **P3 - Low** | < 1 week | Backlog, fix when convenient |

### Incident Response Team

- **Security Lead:** @username (Slack/Signal)
- **Backend Lead:** @username
- **DevOps Lead:** @username
- **Escalation:** CEO/CTO

**Runbooks:** [DOX/security/INCIDENT-RESPONSE.md](DOX/security/INCIDENT-RESPONSE.md)

---

## Compliance & Auditing

### Security Audits

| Date | Auditor | Findings | Status |
|------|---------|----------|--------|
| 2025-11-07 | External Audit | 3 critical, 2 high | ✅ Resolved |

**Next audit:** Planned for Beta release

---

### Audit Logs

**Logged events:**
- Authentication attempts (success/failure)
- Escrow state transitions
- Admin actions
- Configuration changes
- Dispute resolutions
- RPC failures

**Retention:** 90 days (encrypted at rest)

**Access:** Admin-only, logged

---

## Security Resources

### Documentation

- **Placeholder Validation:** [DOX/security/PLACEHOLDER-VALIDATION.md](DOX/security/PLACEHOLDER-VALIDATION.md)
- **Property-Based Testing:** [DOX/security/PROPERTY-BASED-TESTING.md](DOX/security/PROPERTY-BASED-TESTING.md)
- **Dependency Audits:** [DOX/security/DEPENDENCY-AUDIT.md](DOX/security/DEPENDENCY-AUDIT.md)
- **Monitoring:** [DOX/monitoring/PROMETHEUS-MONITORING.md](DOX/monitoring/PROMETHEUS-MONITORING.md)
- **Threat Model:** [DOX/security/THREAT-MODEL.md](DOX/security/THREAT-MODEL.md)
- **Incident Response:** [DOX/security/INCIDENT-RESPONSE.md](DOX/security/INCIDENT-RESPONSE.md)

### Development Guides

- **CLAUDE.md:** Development guidelines
- **DEVELOPER-GUIDE.md:** Comprehensive dev guide
- **SECURITY-THEATRE-PREVENTION.md:** Avoiding security theatre
- **OPSEC Guidelines:** Tor + Monero best practices

### External Resources

- **RustSec Advisory DB:** https://rustsec.org/
- **Monero Security:** https://www.getmonero.org/resources/developer-guides/
- **Tor Best Practices:** https://community.torproject.org/
- **OWASP Top 10:** https://owasp.org/Top10/

---

## Contact

**Security Team:** security@example.com
**General Questions:** info@example.com
**GitHub Issues:** (Non-security bugs only)

**PGP Key:** [security-team.asc](DOX/security/security-team.asc)

---

## Acknowledgments

### Security Researchers

*Hall of Fame - Security researchers who have responsibly disclosed vulnerabilities:*

- *Your name could be here!*

### Security Improvements Timeline

- **2025-11-07:** A- grade achieved (placeholder validation, property tests, dependency audits)
- **2025-11-03:** B+ grade (3 critical fixes applied)
- **2025-10-28:** Initial security audit

---

## License

This security policy is licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

The software is licensed under [MIT License](LICENSE).

---

**Last Updated:** 2025-11-07
**Version:** 1.0.0
**Next Review:** 2025-12-07
