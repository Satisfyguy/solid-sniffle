# Specification: Honeypot Detection System

**Status:** DRAFT - Phase 4.6 Security Enhancement
**Created:** 2025-10-22
**Author:** Security Team
**Priority:** HIGH

---

## Objectif

Implémenter un système de honeypots multi-niveaux pour détecter, alerter et bloquer les attaques automatisées contre le Monero Marketplace, tout en maintenant une OPSEC stricte (pas de logs sensibles).

## Motivation

**Pourquoi des honeypots ?**
1. **Détection précoce** : Identifier les attaquants AVANT qu'ils atteignent les vrais endpoints
2. **Attribution** : Distinguer trafic légitime vs malveillant
3. **Dissuasion** : Augmenter le coût d'attaque (temps perdu sur faux endpoints)
4. **Intelligence** : Comprendre les TTPs (Tactics, Techniques, Procedures) des adversaires

**Threat Model :**
- Script kiddies avec scanners automatiques (Nikto, SQLMap, etc.)
- Bots de phishing cherchant des wallets Monero
- Attaquants cherchant des vulnérabilités SQL/XSS
- Reconnaissance pour attaques ciblées ultérieures

## Architecture

### Composants

```
┌─────────────────────────────────────────────────────────────┐
│                    Actix-Web Application                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Normal     │  │   Honeypot   │  │  Behavioral  │     │
│  │  Endpoints   │  │  Endpoints   │  │   Detector   │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                    ┌───────▼────────┐                       │
│                    │ Honeypot Guard │                       │
│                    │   Middleware   │                       │
│                    └───────┬────────┘                       │
│                            │                                 │
│         ┌──────────────────┼──────────────────┐             │
│         │                  │                  │             │
│    ┌────▼─────┐     ┌─────▼──────┐    ┌─────▼──────┐      │
│    │ Alerting │     │  Scoring   │    │  Blocking  │      │
│    │  System  │     │   Engine   │    │   Engine   │      │
│    └──────────┘     └────────────┘    └────────────┘      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Tier System

#### **Tier 1: Fake Endpoints (Honeypot Routes)**

**Endpoints créés :**
```rust
// server/src/handlers/honeypot.rs

/api/admin                  // Fake admin panel
/api/debug                  // Fake debug endpoint
/api/v1/users               // Fake user enumeration
/api/backup                 // Fake backup download
/.env                       // Fake environment file
/config.json                // Fake config leak
/api/internal/metrics       // Fake internal metrics
/phpMyAdmin                 // Legacy bait
/wp-admin                   // WordPress bait
/api/wallet/balance         // Fake wallet endpoint
```

**Comportement :**
- Retourner des réponses plausibles (200 OK avec fake data)
- Logger l'événement avec threat score
- Incrémenter le compteur d'attaque pour cette IP
- Délai artificiel de 3-5 secondes (ralentir les scanners)

#### **Tier 2: Canary Tokens (Hidden Form Fields)**

**Implémentation dans templates Tera :**
```html
<!-- templates/auth/register.html -->
<form>
  <!-- Champ visible -->
  <input type="text" name="username" required>
  <input type="password" name="password" required>

  <!-- HONEYPOT: Champ caché que seuls les bots remplissent -->
  <input type="text"
         name="email"
         class="hp-field"
         tabindex="-1"
         autocomplete="off">

  <!-- CSS dans static/css/main.css -->
  <!-- .hp-field { position: absolute; left: -9999px; } -->
</form>
```

**Validation côté serveur :**
```rust
// Si le champ honeypot est rempli -> bot détecté
if !req.email.is_empty() {
    return Err(ApiError::TooManyRequests("Rate limit exceeded".to_string()));
}
```

#### **Tier 3: Fake Monero Wallets (Canary Addresses)**

**Wallets factices dans HTML comments :**
```html
<!-- Debug mode: test wallet 888tNkZrPN6JsEgekjMnABU4TBzc2Dt29EPAvkRxbANsAnjyPbb3iQ1YBRk1UXcdRsiKc9dhwMVgN5S9cQUiyoogDavup3H -->
```

**Monitoring :**
- Script externe qui vérifie la blockchain Monero testnet
- Si des fonds arrivent sur ces wallets -> quelqu'un a scrapé le site
- Alerte immédiate (email/webhook)

#### **Tier 4: Behavioral Detection**

**Patterns détectés :**

| Pattern | Threshold | Action |
|---------|-----------|--------|
| Endpoint scanning | 5+ unknown endpoints in 30s | Log + Score +50 |
| SQL injection | `' OR 1=1`, `UNION SELECT` | Block IP + Alert |
| XSS attempts | `<script>`, `javascript:` | Block IP + Alert |
| Path traversal | `../`, `/etc/passwd` | Block IP + Alert |
| Brute-force | 10+ failed logins in 5min | Block IP 1 hour |
| Velocity anomaly | 100+ req/sec | Block IP + Alert |

**Implémentation :**
```rust
// server/src/middleware/behavioral_detector.rs

pub struct BehaviorAnalyzer {
    redis: RedisPool,  // Pour tracking temporel
}

impl BehaviorAnalyzer {
    pub fn analyze_request(&self, req: &HttpRequest) -> ThreatScore {
        let ip = extract_ip(req);
        let path = req.path();
        let query = req.query_string();

        let mut score = 0;

        // Check SQL injection patterns
        if contains_sql_injection(query) {
            score += 100;  // Immediate block
        }

        // Check XSS patterns
        if contains_xss(query) {
            score += 100;
        }

        // Check endpoint scanning
        if is_unknown_endpoint(path) {
            score += 10;
        }

        ThreatScore(score)
    }
}
```

#### **Tier 5: Database Honeypots**

**Fake tables :**
```sql
-- migrations/honeypot/000_create_honeypot_tables.sql

CREATE TABLE IF NOT EXISTS admin_sessions (
    id TEXT PRIMARY KEY,
    session_token TEXT NOT NULL,
    admin_username TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    ip_address TEXT
);

CREATE TABLE IF NOT EXISTS debug_logs (
    id TEXT PRIMARY KEY,
    log_level TEXT,
    message TEXT,
    wallet_address TEXT,  -- FAKE
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**Triggers pour détection :**
```sql
CREATE TRIGGER honeypot_access_detector
AFTER SELECT ON admin_sessions
BEGIN
    INSERT INTO security_events (event_type, severity, details)
    VALUES ('HONEYPOT_DB_ACCESS', 'CRITICAL', 'Unauthorized access to honeypot table');
END;
```

## Threat Scoring System

### Score Calculation

```rust
pub struct ThreatScore(pub u32);

impl ThreatScore {
    pub const SAFE: u32 = 0;
    pub const SUSPICIOUS: u32 = 50;
    pub const MALICIOUS: u32 = 100;
    pub const CRITICAL: u32 = 200;
}
```

### Scoring Rules

| Event | Points | Cumulative |
|-------|--------|------------|
| Unknown endpoint access | +10 | Yes |
| Honeypot endpoint hit | +50 | Yes |
| Canary token triggered | +75 | No (instant block) |
| SQL injection pattern | +100 | No (instant block) |
| XSS pattern | +100 | No (instant block) |
| Path traversal | +100 | No (instant block) |
| Fake wallet scraping | +200 | No (instant ban) |

### Actions by Score

```rust
match score.0 {
    0..=49 => Action::Allow,
    50..=99 => Action::LogAndWatch,
    100..=199 => Action::BlockTemporary(Duration::hours(1)),
    200.. => Action::BlockPermanent,
}
```

## Alerting System

### Alert Levels

```rust
pub enum AlertLevel {
    Info,       // Logged only
    Warning,    // Email to admin
    Critical,   // Email + Webhook
    Emergency,  // Email + Webhook + SMS
}
```

### Alert Channels

1. **Structured Logging (tracing)**
   ```rust
   tracing::warn!(
       ip = %attacker_ip,
       event = "honeypot_triggered",
       endpoint = %path,
       score = score.0,
       "Honeypot endpoint accessed"
   );
   ```

2. **Email Alerts** (for Critical+)
   - Via SMTP over Tor
   - Rate-limited (max 1 email/hour per IP)

3. **Webhook Alerts** (for Critical+)
   - POST to configured webhook URL
   - Includes: timestamp, IP (hashed), event type, score

4. **Metrics Export**
   - Prometheus-compatible endpoint `/metrics`
   - Honeypot hit counter
   - Blocked IP counter
   - Threat score distribution

## OPSEC Considerations

### ❌ NEVER LOG

- Real user IPs (hash with salt)
- Session tokens
- Monero wallet addresses (real ones)
- .onion addresses
- User-Agent strings (fingerprinting risk)

### ✅ ALWAYS LOG

```rust
// GOOD: OPSEC-safe logging
tracing::warn!(
    ip_hash = %hash_ip_with_salt(ip),
    event = "honeypot_hit",
    endpoint = %sanitize_path(path),
    score = score.0,
    timestamp = %Utc::now().timestamp(),
    "Security event detected"
);
```

### IP Hashing Strategy

```rust
use sha2::{Sha256, Digest};

pub fn hash_ip_with_salt(ip: &str) -> String {
    let salt = env::var("IP_HASH_SALT")
        .expect("IP_HASH_SALT must be set");

    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    hasher.update(salt.as_bytes());

    format!("ip_{:x}", hasher.finalize())
}
```

## Blocking Strategy

### Temporary Blocks (In-Memory)

```rust
// server/src/middleware/ip_blocker.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Instant;

pub struct IpBlocker {
    blocked: Arc<RwLock<HashMap<String, BlockEntry>>>,
}

struct BlockEntry {
    expires_at: Instant,
    reason: String,
    score: u32,
}

impl IpBlocker {
    pub async fn is_blocked(&self, ip: &str) -> bool {
        let blocks = self.blocked.read().await;

        if let Some(entry) = blocks.get(ip) {
            if Instant::now() < entry.expires_at {
                return true;
            }
        }

        false
    }

    pub async fn block_ip(&self, ip: String, duration: Duration, reason: String, score: u32) {
        let mut blocks = self.blocked.write().await;

        blocks.insert(ip.clone(), BlockEntry {
            expires_at: Instant::now() + duration,
            reason: reason.clone(),
            score,
        });

        tracing::warn!(
            ip_hash = %hash_ip_with_salt(&ip),
            duration_secs = duration.as_secs(),
            reason = %reason,
            score = score,
            "IP blocked temporarily"
        );
    }
}
```

### Permanent Bans (Database)

```sql
CREATE TABLE IF NOT EXISTS banned_ips (
    ip_hash TEXT PRIMARY KEY,
    banned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    reason TEXT NOT NULL,
    threat_score INTEGER NOT NULL
);
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_injection_detection() {
        let detector = BehaviorAnalyzer::new();

        let malicious_queries = vec![
            "' OR 1=1--",
            "UNION SELECT * FROM users",
            "'; DROP TABLE users--",
        ];

        for query in malicious_queries {
            let score = detector.detect_sql_injection(query);
            assert!(score >= ThreatScore::MALICIOUS);
        }
    }

    #[test]
    fn test_honeypot_endpoint_detection() {
        // Test que /api/admin retourne 200 mais log l'événement
    }

    #[test]
    fn test_canary_token_validation() {
        // Test que formulaire avec honeypot rempli = rejection
    }
}
```

### Integration Tests

```rust
#[actix_web::test]
async fn test_honeypot_triggers_alert() {
    let app = test::init_service(/* ... */).await;

    let req = test::TestRequest::get()
        .uri("/api/admin")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return 200 (looks legitimate)
    assert_eq!(resp.status(), 200);

    // But should have logged event
    // Check threat score increased
}
```

### Manual Validation (Reality Check)

```bash
# Test honeypot endpoints
curl -v http://127.0.0.1:8080/api/admin
curl -v http://127.0.0.1:8080/.env
curl -v http://127.0.0.1:8080/phpMyAdmin

# Test SQL injection detection
curl -v "http://127.0.0.1:8080/api/listings?search=' OR 1=1--"

# Test XSS detection
curl -v "http://127.0.0.1:8080/api/listings?search=<script>alert(1)</script>"

# Verify IP gets blocked after threshold
for i in {1..10}; do
  curl -v http://127.0.0.1:8080/api/nonexistent$i
done
```

## Deployment Checklist

- [ ] Generate IP hash salt: `openssl rand -hex 32 > .ip_hash_salt`
- [ ] Configure alert webhook URL in `.env`
- [ ] Setup email SMTP over Tor
- [ ] Run database migrations for honeypot tables
- [ ] Deploy honeypot CSS to hide canary fields
- [ ] Setup Prometheus metrics scraping
- [ ] Test all honeypot endpoints return plausible responses
- [ ] Verify OPSEC compliance (no sensitive data in logs)
- [ ] Load test: ensure honeypots don't impact performance

## Monitoring & Maintenance

### Metrics to Track

```prometheus
# Honeypot hits per endpoint
honeypot_hits_total{endpoint="/api/admin"} 42

# Blocked IPs (temporary)
blocked_ips_temporary 15

# Blocked IPs (permanent)
blocked_ips_permanent 3

# Threat score distribution
threat_score_bucket{le="50"} 1000
threat_score_bucket{le="100"} 50
threat_score_bucket{le="200"} 5
```

### Weekly Review

1. Review top 10 most triggered honeypots
2. Analyze new attack patterns
3. Update detection signatures
4. Tune threat score thresholds
5. Review false positives (legitimate users blocked)

## Future Enhancements (Phase 5+)

- [ ] Machine learning-based anomaly detection
- [ ] Distributed honeypot network (multiple nodes)
- [ ] Automated IP reputation scoring (integrate with blocklists)
- [ ] Honeypot response polymorphism (change responses weekly)
- [ ] Fake user accounts in database (active honeypots)
- [ ] Timing analysis resistance (add random delays to all responses)

## References

- [OWASP: Web Security Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
- [Honeypot Best Practices](https://www.sans.org/white-papers/37122/)
- [Tor Project: OPSEC Guidelines](https://2019.www.torproject.org/docs/faq.html.en#WhatAboutLogs)

## Changements

| Date | Version | Auteur | Description |
|------|---------|--------|-------------|
| 2025-10-22 | 0.1.0 | Security Team | Initial specification |

---

**Status:** ✅ Ready for Implementation
**Estimation:** 3-4 days développement + 1 day testing
**Dependencies:** Redis (pour rate limiting avancé), Prometheus (optionnel)
