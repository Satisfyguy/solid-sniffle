# Security Checklist - Production Readiness
## Monero Marketplace Hidden Service

**Version:** 1.0
**Date:** 2025-10-16
**Statut:** Template pour audit pré-production

---

## ⚠️ CRITICAL: Ne PAS lancer mainnet sans 100% de cette checklist

Cette checklist DOIT être complétée et auditée avant tout lancement production mainnet.

---

## 1. Cryptographie & Key Management

### 1.1 Password Hashing
- [ ] Argon2id utilisé (pas bcrypt/scrypt)
- [ ] Memory cost ≥ 19 MiB
- [ ] Time cost ≥ 2 iterations
- [ ] Salt unique par password (via SaltString::generate)
- [ ] Hashes jamais loggés ou exposés
- [ ] Test: Vérifier timing attack resistance

### 1.2 Encryption At-Rest
- [ ] Database chiffrée (sqlcipher OU disk encryption)
- [ ] Master key stocké dans variable d'environnement
- [ ] Master key JAMAIS commitée dans git
- [ ] Sensitive fields chiffrés (AES-256-GCM):
  - [ ] `escrows.buyer_wallet_info`
  - [ ] `escrows.vendor_wallet_info`
  - [ ] `escrows.arbiter_wallet_info`
  - [ ] `messages.content`
- [ ] Nonces uniques par encryption
- [ ] Test: Dump database → vérifier données chiffrées

### 1.3 Monero Multisig
- [ ] Monero RPC bind 127.0.0.1 UNIQUEMENT
- [ ] View/Spend keys JAMAIS loggés
- [ ] Multisig info validé (format MultisigV1...)
- [ ] Timeout handling pour sync rounds
- [ ] Test: 3-party multisig setup end-to-end
- [ ] Test: Transaction 2-of-3 signatures
- [ ] Reality Check Tor validé pour chaque fonction RPC

### 1.4 Random Number Generation
- [ ] Utiliser `OsRng` (pas `rand::thread_rng()`)
- [ ] Session IDs = UUID v4 (cryptographically random)
- [ ] CSRF tokens = 32 bytes random
- [ ] Nonces = random per-operation

---

## 2. Authentication & Authorization

### 2.1 Authentication
- [ ] Sessions server-side (PAS JWT)
- [ ] Session ID = UUID v4 random
- [ ] Session timeout = 30 minutes d'inactivité
- [ ] Logout = session destruction immédiate
- [ ] Login rate limiting: 3 attempts/5min par session
- [ ] No username enumeration (même message erreur pour user/password invalide)
- [ ] Test: Brute-force login → doit être bloqué

### 2.2 Authorization
- [ ] Rôles implémentés: buyer, vendor, arbiter, admin
- [ ] Middleware vérifie rôle avant chaque endpoint protégé
- [ ] IDOR prevention: vérifier `user_id` dans DB
- [ ] Test: User A ne peut pas accéder orders de User B
- [ ] Test: Buyer ne peut pas créer listings (vendors only)
- [ ] Test: Arbiter ne peut résoudre QUE disputes assignées

### 2.3 Password Policy
- [ ] Minimum 8 caractères
- [ ] Au moins 1 uppercase, 1 lowercase, 1 digit
- [ ] Optionnel: 1 caractère spécial
- [ ] Vérifier contre common passwords (zxcvbn)
- [ ] Pas de maximum (permettre passphrases)

---

## 3. Input Validation & Sanitization

### 3.1 Validation
- [ ] TOUS les inputs validés (aucune exception)
- [ ] Validation côté serveur (pas juste frontend)
- [ ] Whitelist approach (définir ce qui est permis)
- [ ] Length limits sur TOUS les strings:
  - [ ] Username: 3-50 chars
  - [ ] Password: 8-128 chars
  - [ ] Listing title: 10-200 chars
  - [ ] Description: 50-5000 chars
- [ ] Numeric ranges validés:
  - [ ] XMR amounts: 0.001 to 10000 XMR
  - [ ] Stock: 0 to 10000
  - [ ] Priority: 0-3
- [ ] Monero addresses validées (regex + checksum)

### 3.2 Sanitization
- [ ] HTML escaped dans tous les templates
- [ ] SQL injection impossible (prepared statements ONLY)
- [ ] No eval() or equivalent
- [ ] File uploads: type validation (whitelist extensions)
- [ ] File uploads: size limit (max 5MB par fichier)
- [ ] File uploads: scan anti-malware (ClamAV)

### 3.3 CSRF Protection
- [ ] CSRF tokens sur TOUS les formulaires
- [ ] Token validation côté serveur
- [ ] Token rotation après utilisation
- [ ] SameSite=Strict sur cookies
- [ ] Test: Soumettre formulaire sans token → 403

---

## 4. Network Security & Tor

### 4.1 Tor Isolation
- [ ] Hidden service v3 (pas v2)
- [ ] Monero RPC accessible UNIQUEMENT depuis localhost
- [ ] Tous les appels externes via SOCKS5 proxy (127.0.0.1:9050)
- [ ] Pas de DNS leaks (utiliser socks5h://)
- [ ] Test: Vérifier IP via https://check.torproject.org/api/ip
- [ ] Test: Netstat → aucun port public exposé sauf Tor

### 4.2 Rate Limiting
- [ ] Global rate limit: 10 req/sec
- [ ] Per-session rate limit: 5 req/sec
- [ ] Per-endpoint rate limits:
  - [ ] `/api/auth/login`: 3 req/5min
  - [ ] `/api/orders`: 10 req/hour
  - [ ] `/api/escrow/init`: 5 req/hour
  - [ ] `/api/listings` (POST): 20 req/day
  - [ ] `/api/listings` (GET): 60 req/min
- [ ] 429 response avec Retry-After header
- [ ] Test: Spam endpoint → doit être bloqué

### 4.3 DDoS Protection
- [ ] Tor PoW (Proof-of-Work) activé si disponible
- [ ] Connection limits (max 100 concurrent)
- [ ] Request size limits (max 10MB)
- [ ] Timeout sur toutes les requêtes (30s)
- [ ] Graceful degradation sous charge

---

## 5. Data Protection & Privacy

### 5.1 Logging
- [ ] Zero sensitive data dans logs:
  - [ ] Pas de .onion addresses
  - [ ] Pas de Monero addresses
  - [ ] Pas de view/spend keys
  - [ ] Pas de passwords (même hashés)
  - [ ] Pas d'IPs réelles
  - [ ] Pas de session IDs complets
- [ ] Log level production = WARN ou ERROR
- [ ] Logs rotation automatique (max 7 jours)
- [ ] Test: Grep logs pour patterns sensibles → rien trouvé

### 5.2 Error Handling
- [ ] Pas de stack traces exposés aux users
- [ ] Messages d'erreur génériques (pas de détails techniques)
- [ ] Erreurs loggées côté serveur uniquement
- [ ] 500 errors → message générique "Internal Server Error"
- [ ] Test: Trigger erreur → user ne voit pas stack trace

### 5.3 Database Security
- [ ] No default credentials
- [ ] Database user = least privilege
- [ ] Backups chiffrés (AES-256)
- [ ] Backups stockés off-site
- [ ] Backup restoration testée (monthly)
- [ ] Connection pooling configuré (max 10 connections)

---

## 6. API Security

### 6.1 HTTP Headers
- [ ] Content-Security-Policy configuré:
  ```
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
- [ ] X-Content-Type-Options: nosniff
- [ ] X-Frame-Options: DENY
- [ ] X-XSS-Protection: 1; mode=block
- [ ] Referrer-Policy: no-referrer
- [ ] Permissions-Policy: geolocation=(), camera=(), microphone=()

### 6.2 CORS
- [ ] CORS désactivé OU strict whitelist
- [ ] Pas de Access-Control-Allow-Origin: *
- [ ] Credentials: true seulement si nécessaire

### 6.3 API Versioning
- [ ] API versionnée (v1, v2...)
- [ ] Breaking changes = nouvelle version
- [ ] Old versions dépréciées graduellement

---

## 7. Frontend Security

### 7.1 XSS Prevention
- [ ] Toutes les variables échappées dans templates
- [ ] Pas de innerHTML (utiliser textContent)
- [ ] Pas de eval() ou Function()
- [ ] DOMPurify pour user-generated HTML (si nécessaire)
- [ ] Test: Injecter `<script>alert(1)</script>` → ne s'exécute pas

### 7.2 OPSEC Frontend
- [ ] Pas de CDN externes (tout self-hosted)
- [ ] Pas de Google Fonts (system fonts only)
- [ ] Pas d'analytics (pas de Google Analytics, Matomo, etc.)
- [ ] Pas de social media embeds
- [ ] Pas de fingerprinting JS:
  - [ ] Canvas fingerprinting bloqué
  - [ ] WebGL désactivé ou randomisé
  - [ ] Battery API disabled
  - [ ] Geolocation disabled
- [ ] SRI (Subresource Integrity) pour tous les assets
- [ ] Test: Panopticlick → score élevé

---

## 8. Infrastructure Security

### 8.1 Server Hardening
- [ ] OS minimal (Debian 12 minimal OU Alpine Linux)
- [ ] SSH désactivé OU key-only (pas de password)
- [ ] Firewall activé (ufw OU iptables):
  ```bash
  ufw default deny incoming
  ufw default allow outgoing
  ufw allow 80/tcp
  ufw allow 443/tcp
  ufw enable
  ```
- [ ] Fail2ban configuré
- [ ] Automatic security updates activées
- [ ] Root login désactivé
- [ ] Services inutiles désactivés

### 8.2 Monitoring & Alerting
- [ ] Prometheus + Grafana configurés
- [ ] Alertes configurées:
  - [ ] CPU > 80% pendant 5min
  - [ ] RAM > 90%
  - [ ] Disk > 85%
  - [ ] Error rate > 5%
  - [ ] Response time > 2s
- [ ] Uptime monitoring (UptimeRobot OU self-hosted)
- [ ] Dead man's switch (canary)

### 8.3 Backup & Recovery
- [ ] Backups automatiques daily
- [ ] Backup retention: 7 daily, 4 weekly, 12 monthly
- [ ] Backups testés mensuellement (restore test)
- [ ] RTO (Recovery Time Objective) < 4 heures
- [ ] RPO (Recovery Point Objective) < 24 heures
- [ ] Disaster recovery plan documenté

---

## 9. Code Security

### 9.1 Dependencies
- [ ] `cargo audit` passe (zero vulnerabilities)
- [ ] `cargo deny` configuré
- [ ] Dépendances mises à jour régulièrement
- [ ] Pas de crates abandonnées
- [ ] Audit des nouvelles dépendances avant ajout

### 9.2 Static Analysis
- [ ] `cargo clippy -- -D warnings` passe
- [ ] Clippy pedantic mode activé
- [ ] Security theatre checks passent
- [ ] `semgrep` scan (si disponible)

### 9.3 Code Patterns
- [ ] Zero `.unwrap()` ou `.expect()` sans justification
- [ ] Tous les `Result` propagés avec `?`
- [ ] Pas de `panic!` dans production code
- [ ] Pas de `todo!()` ou `unimplemented!()`
- [ ] Pas de hardcoded secrets

---

## 10. Testing

### 10.1 Unit Tests
- [ ] Code coverage > 80%
- [ ] Tests passent: `cargo test --workspace`
- [ ] Tests isolés (pas de dépendances externes)
- [ ] Edge cases testés

### 10.2 Integration Tests
- [ ] Tests e2e multisig (3 wallets)
- [ ] Tests e2e transactions
- [ ] Tests API endpoints
- [ ] Tests WebSocket

### 10.3 Security Tests
- [ ] Penetration testing interne
- [ ] Fuzzing (cargo-fuzz) sur wallet code
- [ ] SQL injection tests
- [ ] XSS tests
- [ ] CSRF tests
- [ ] Authentication bypass tests
- [ ] Authorization tests (IDOR)

---

## 11. Compliance & Legal

### 11.1 Terms of Service
- [ ] ToS rédigés et affichés
- [ ] Users doivent accepter ToS avant utilisation
- [ ] Interdiction contenus illégaux clarifiée
- [ ] Disclaimer: "Educational purposes only"

### 11.2 Privacy Policy
- [ ] Privacy policy rédigée
- [ ] Déclaration: "No logs, no tracking, no KYC"
- [ ] Mention: "Tor usage recommended"

### 11.3 DMCA / Takedown
- [ ] Process de reporting abuse
- [ ] Contact email (PGP encrypted)
- [ ] Modération réactive (<24h)

---

## 12. Operational Security

### 12.1 Deployment
- [ ] CI/CD pipeline sécurisé
- [ ] Secrets stockés dans variables d'environnement
- [ ] Pas de secrets dans git history
- [ ] Rollback procedure testée
- [ ] Blue-green deployment (zero downtime)

### 12.2 Incident Response
- [ ] Incident response plan documenté
- [ ] Team on-call 24/7 (au moins 2 premières semaines)
- [ ] Escalation procedure
- [ ] Communication channels (Signal, PGP email)
- [ ] Post-mortem template

### 12.3 Canary & Transparency
- [ ] Canary warrant publié (mise à jour mensuelle)
- [ ] Pas de gag orders acceptés
- [ ] Open-source code (audit public)
- [ ] Bug bounty programme actif

---

## 13. External Audit

### 13.1 Pre-Audit
- [ ] Toutes les checklist items ci-dessus complétées
- [ ] Documentation complète pour auditeurs
- [ ] Scope d'audit défini
- [ ] Budget alloué ($50k-$150k)

### 13.2 Audit Findings
- [ ] Tous les CRITICAL fixés
- [ ] Tous les HIGH fixés
- [ ] >90% des MEDIUM fixés
- [ ] Re-audit après fixes

### 13.3 Post-Audit
- [ ] Rapport d'audit publié (après fixes)
- [ ] Recommendations implémentées
- [ ] Audit annuel planifié

---

## 14. Go/No-Go Decision

**TOUS ces critères doivent être TRUE pour lancer mainnet:**

- [ ] External security audit complété et tous les criticals fixés
- [ ] Bug bounty actif depuis 4+ semaines sans critical
- [ ] Testnet beta complété avec >50 users
- [ ] Cette checklist 100% complétée
- [ ] Team disponible 24/7 pendant 2 semaines post-launch
- [ ] Backups testés et fonctionnels
- [ ] Incident response plan documenté et testé
- [ ] Legal compliance vérifiée
- [ ] Monitoring & alerting opérationnels

**Si UNE SEULE case non cochée → NO-GO**

---

## Score de Sécurité

**Formule:**
```
Security Score = (Items Complétés / Items Totaux) × 100
```

**Niveaux:**
- 0-50%: 🔴 Dangereux - Ne PAS lancer
- 51-70%: 🟡 Insuffisant - Continuer développement
- 71-90%: 🟠 Acceptable - Testnet OK, mainnet NO
- 91-99%: 🟢 Bon - Mainnet possible après audit
- 100%: ✅ Excellent - Production ready

**Score Actuel:** `XX%` (à calculer)

---

## Prochaines Actions

### Cette Semaine
1. [ ] Implémenter Argon2id password hashing
2. [ ] Configurer rate limiting middleware
3. [ ] Setup sqlcipher pour database encryption

### Phase 1
1. [ ] Compléter sections 1 (Crypto), 9 (Code Security), 10 (Testing)

### Phase 5 (Audit)
1. [ ] Compléter 100% de cette checklist
2. [ ] Contracter auditeur externe
3. [ ] Fixer toutes les vulnérabilités trouvées

---

**Statut:** 🚧 En Cours
**Dernière Révision:** 2025-10-16
**Prochaine Révision:** Après Phase 1 completion
**Responsable Sécurité:** (À définir)
