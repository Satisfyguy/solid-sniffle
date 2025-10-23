---
name: Production-Ready Development
description: Enforce production-grade development practices for the Monero Marketplace. Use this skill when working on ANY code in the server/ directory, implementing features, or preparing for deployment. This skill ensures zero-tolerance for security theatre, complete error handling, and production-ready code quality.
---

# Production-Ready Development Skill

## 🎯 Mission Critical Context

You're developing a **production-grade Monero escrow marketplace** with multisig wallets handling real money. Every line of code you write will be used in production. There is NO separate "production version" - what you code today ships to mainnet.

**Zero-tolerance policy**: No placeholders, no TODOs in production paths, no `.unwrap()` without justification, no mocks in integration tests.

---

## 🔐 Security Hardening Checklist

### Authentication & Authorization
- Password hashing with Argon2id (time cost ≥ 2, memory cost ≥ 19MB)
- Session tokens: cryptographically random, 32+ bytes
- CSRF tokens on all state-changing operations
- Rate limiting: 5 failed logins per IP per hour
- Account lockout after 5 failed attempts (24h cooldown)
- JWT tokens with short expiration (2h) and refresh mechanism
- Role-based access control (RBAC) for all endpoints

### Input Validation
- Validate all inputs at API boundary (never trust client)
- Sanitize file uploads (magic number verification, size limits)
- SQL injection prevention (parameterized queries only)
- XSS prevention (escape all user content)
- Path traversal prevention (validate file paths)
- Email validation with DNS MX record check
- Amount validation (min/max limits, decimal precision)

### Cryptography
- TLS 1.3 only (disable TLS 1.2 and below)
- Perfect Forward Secrecy (PFS) enabled
- Strong cipher suites only (AEAD ciphers)
- HSTS header with long max-age (31536000)
- Certificate pinning for critical connections
- Encryption at rest (AES-256-GCM for DB, disk encryption)
- Secure key derivation (PBKDF2/Argon2 for passwords)

### Monero Security
- Wallet RPC behind firewall (no public exposure)
- Authentication on all RPC calls
- View-only wallets where possible
- Multisig address verification (all parties confirm)
- Transaction verification before broadcast
- Balance reconciliation (detect discrepancies)
- Automatic wallet backups (encrypted, off-site)

### Network Security
- Tor hidden service as primary interface
- DDoS protection (rate limiting + CloudFlare/similar)
- IP allowlisting for admin endpoints
- Fail2ban for repeated failures
- Network segmentation (DB, RPC, app in separate networks)
- Egress filtering (only allow necessary outbound)

### Application Security
- No secrets in code/logs/errors
- Secure headers (CSP, X-Frame-Options, etc.)
- CORS properly configured
- Dependency scanning (cargo audit, Dependabot)
- Container scanning (Trivy, Clair)
- Regular security updates
- Principle of least privilege (minimal permissions)

---

## 📊 Production-Ready Checklist

### Code Quality (Automated)
- All tests passing (`cargo test --workspace`)
- Zero clippy warnings (`cargo clippy -- -D warnings`)
- Code coverage ≥85% (`cargo tarpaulin`)
- Zero TODO/FIXME in `server/src/` (excluding tests)
- All functions documented (missing_docs lint)
- No `.unwrap()` in production paths
- Integration tests against real testnet

### Security Audit
- External security audit completed (professional firm)
- All critical/high vulnerabilities fixed
- Penetration testing performed
- Code review by 2+ senior developers
- Bug bounty program active (minimum 4 weeks)
- Security documentation complete
- Incident response plan tested

### Infrastructure
- Production environment provisioned
- Monero daemon fully synced (mainnet)
- 3 wallet RPC instances operational
- Database backups automated (hourly incremental, daily full)
- Backup restoration tested successfully
- Monitoring and alerting operational
- Tor .onion service configured
- SSL/TLS certificates valid and auto-renewing
- DDoS protection configured
- Firewall rules applied and tested

### Operational Readiness
- Runbook documented and reviewed:
  - Deployment procedures
  - Rollback procedures
  - Backup/restore procedures
  - Incident response playbook
  - On-call rotation schedule
- 24/7 on-call team defined (minimum 2 people)
- Escalation procedures documented
- Communication channels established (PagerDuty, Slack, etc.)
- Disaster recovery plan tested
- Capacity planning completed (expected load + 3x)

### Legal & Compliance
- Terms of Service reviewed by lawyer
- Privacy Policy compliant with GDPR/local laws
- KYC/AML requirements analyzed (consult lawyer)
- Data retention policy defined
- User data deletion procedures implemented
- Jurisdiction considerations documented

### Business Continuity
- Key personnel documented (bus factor > 1)
- Critical credentials in secure vault (1Password, Bitwarden)
- Succession plan for key roles
- Financial reserves (3-6 months operating costs)
- Insurance coverage evaluated

---

## 🚀 Go-Live Decision Matrix

### ✅ GO CRITERIA (All must be met)

| Category | Requirement | Status |
|----------|-------------|--------|
| **Security** | External audit: 0 critical, <5 high findings | ☐ |
| | Penetration testing passed | ☐ |
| | Bug bounty: 50+ researchers, 0 critical unfixed | ☐ |
| **Quality** | Code coverage ≥85% | ☐ |
| | Zero TODOs in production code | ☐ |
| | Integration tests: 100% pass rate | ☐ |
| **Infrastructure** | All services healthy (7-day uptime) | ☐ |
| | Monero daemon synced, <1 block behind | ☐ |
| | Backup/restore tested successfully | ☐ |
| | Monitoring: 0 false positives | ☐ |
| **Operations** | On-call team available (2+ people) | ☐ |
| | Runbook tested with 2+ drills | ☐ |
| | Incident response: <15 min MTTA | ☐ |
| **Beta Testing** | 50+ beta users | ☐ |
| | 100+ successful escrows | ☐ |
| | 0 fund losses | ☐ |
| | User satisfaction ≥4.0/5.0 | ☐ |

### 🛑 NO-GO CRITERIA (Any blocks launch)

- ❌ Any unresolved CRITICAL security finding
- ❌ Failed backup restoration test
- ❌ Unstable Monero RPC (<99% uptime in last 7 days)
- ❌ Database corruption detected
- ❌ Any fund loss in beta (even if reimbursed)
- ❌ On-call team unavailable
- ❌ Missing legal review (ToS/Privacy Policy)
- ❌ Key personnel unavailable for 2+ weeks post-launch

### ⚠️ CONSIDER DELAYING IF:

- Integration test pass rate <98%
- Beta testing <30 days
- Code coverage <80%
- On-call team has <2 people
- Documentation incomplete
- High-severity findings >10
- User-reported bugs >20 (unfixed)

---

## 🎓 Development Best Practices

### Error Handling Philosophy

```rust
// ❌ NEVER: Swallowing errors
let result = some_operation();  // Ignoring Result

// ❌ NEVER: Generic error messages
return Err(anyhow!("Failed"));

// ❌ NEVER: Panicking in production paths
let value = config.get("key").unwrap();

// ✅ ALWAYS: Context-rich errors
let value = config
    .get("key")
    .context("Missing required configuration key 'key'")?;

// ✅ ALWAYS: Structured logging with context
tracing::error!(
    escrow_id = %escrow_id,
    user_id = %user_id,
    error = %e,
    "Failed to release escrow funds"
);

// ✅ ALWAYS: Actionable error messages
return Err(WalletError::InsufficientFunds {
    required: amount,
    available: balance,
    suggestion: "Please deposit additional funds or reduce the escrow amount"
});
```

### Testing Strategy

```rust
// Unit tests: Fast, isolated, test business logic
#[test]
fn test_escrow_amount_validation() {
    let escrow = Escrow::new(50_000_000); // 0.05 XMR
    assert!(escrow.validate().is_ok());

    let escrow = Escrow::new(1_000_000_000_000); // 1000 XMR
    assert!(matches!(
        escrow.validate(),
        Err(ValidationError::AmountExceedsMaximum { .. })
    ));
}

// Integration tests: Real services, end-to-end flows
#[tokio::test]
async fn test_full_escrow_with_real_monero() {
    let env = TestEnvironment::with_real_testnet().await;
    // Test against actual Monero testnet
}

// Property-based tests: Fuzzing critical paths
#[quickcheck]
fn test_multisig_address_deterministic(seed: u64) -> bool {
    // Verify same inputs always produce same output
}
```

### Logging Levels

```rust
// ERROR: System is in degraded state, requires immediate attention
tracing::error!(
    escrow_id = %escrow_id,
    error = %e,
    "Failed to finalize multisig - manual intervention required"
);

// WARN: Unexpected but handled, may indicate future problem
tracing::warn!(
    user_id = %user_id,
    attempt_count = failed_attempts,
    "Repeated failed login attempts detected"
);

// INFO: Normal operational events (state changes)
tracing::info!(
    escrow_id = %escrow_id,
    from_status = ?old_status,
    to_status = ?new_status,
    "Escrow status changed"
);

// DEBUG: Detailed info for troubleshooting (dev/staging only)
tracing::debug!(
    rpc_url = %url,
    method = "get_balance",
    duration_ms = duration.as_millis(),
    "Monero RPC call completed"
);
```

### Performance Optimization

```rust
// Connection pooling
lazy_static! {
    static ref DB_POOL: PgPool = create_pool();
}

// Caching (with TTL)
#[cached(time = 300, result = true)]  // 5 min cache
async fn get_xmr_usd_rate() -> Result<f64> {
    // Expensive API call
}

// Batch operations
let escrows = get_active_escrows().await?;
let balances = fetch_balances_batch(&escrows).await?;

// Async where possible
tokio::spawn(async move {
    send_notification(user_id, event).await;
});

// Database indexes on foreign keys
// CREATE INDEX idx_escrows_order_id ON escrows(order_id);
// CREATE INDEX idx_orders_buyer_id ON orders(buyer_id);
// CREATE INDEX idx_orders_status ON orders(status) WHERE status = 'active';
```

---

## 🔄 Post-Launch Operations

### Daily Checklist
```bash
# Morning checks (automated)
./scripts/health-check-production.sh
./scripts/check-blockchain-sync.sh
./scripts/verify-backups.sh

# Review dashboards
# - Grafana: Error rates, latency, active escrows
# - Prometheus: Resource usage
# - Application logs: Errors/warnings
```

### Weekly Tasks
- Review security logs for anomalies
- Check disk space trends
- Verify backup retention policy
- Update dependencies (security patches)
- Review on-call incidents and improve runbook
- Capacity planning review

### Monthly Tasks
- Rotate encryption keys (if using file-based)
- Full disaster recovery drill
- Review and update incident response plan
- Security audit of access logs
- Performance optimization review
- Cost optimization review

### Quarterly Tasks
- External penetration testing
- Dependency major version updates
- Infrastructure capacity review
- Review and update documentation
- On-call team retrospective
- Business continuity plan review

---

## 🎯 Success Metrics

Track these KPIs post-launch:

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Uptime | 99.9% | <99% |
| API Response Time (P95) | <200ms | >500ms |
| Monero RPC Latency (P95) | <2s | >5s |
| Error Rate | <0.1% | >1% |
| Successful Escrows | >95% | <90% |
| Time to Multisig Setup | <5min | >15min |
| User Satisfaction | >4.5/5 | <4.0/5 |
| Security Incidents | 0 | Any |

---

## 🔄 Continuous Improvement

### Post-Mortem Process
After any incident:
1. **Document** within 24h (timeline, root cause, impact)
2. **Identify** action items (technical, process, documentation)
3. **Assign** owners with deadlines
4. **Track** completion
5. **Share** learnings with team
6. **Update** runbooks/monitoring

### Feedback Loop
- Weekly: Review user feedback, identify patterns
- Bi-weekly: Sprint retrospective (what went well, what to improve)
- Monthly: Technical debt review, prioritize improvements
- Quarterly: Architecture review, plan refactors

---

## 🎬 Final Words

**Remember**: You're building financial infrastructure. Lives and livelihoods depend on this system working correctly. When in doubt:

1. **Fail safe**: Better to reject a transaction than process it incorrectly
2. **Be transparent**: Log everything, hide nothing (except secrets)
3. **Trust no one**: Validate all inputs, verify all outputs
4. **Automate everything**: Humans make mistakes, especially under pressure
5. **Test in production**: (but carefully, with feature flags and monitoring)

**The goal isn't just working code - it's production-grade, secure, maintainable, observable, and scalable financial infrastructure.**

Good luck! 🚀
