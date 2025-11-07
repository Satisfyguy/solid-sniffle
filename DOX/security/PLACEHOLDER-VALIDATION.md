# Placeholder Validation System

## Overview

The placeholder validation system prevents accidental deployment with insecure example/placeholder configuration values (e.g., copying `.env.example` directly without changing credentials).

**Criticality:** 🔴 **CRITICAL** - Prevents deployment with default/example credentials
**Status:** ✅ **IMPLEMENTED** - `server/src/security/placeholder_validator.rs`

---

## Security Rationale

A common vulnerability in production deployments is using example configuration values:
- Developer copies `.env.example` to `.env`
- Forgets to replace `your-64-char-hex-key-here` with actual secure value
- Deploys to production with insecure placeholders
- Attacker can guess credentials from public `.env.example`

**Real-world impact:**
- **MongoDB incident 2017:** Thousands of databases exposed due to default credentials
- **Elasticsearch 2020:** 25 billion records leaked from deployments with default config
- **Redis 2021:** Mass exploitation of instances with no password set

This validator **panics at startup** in production if placeholder patterns are detected.

---

## How It Works

### Placeholder Patterns Detected

The system detects common placeholder patterns (case-insensitive):

```rust
const PLACEHOLDER_PATTERNS: &[&str] = &[
    "your-", "your_",       // your-xxx-here
    "xxx",                  // xxx-secret-key
    "example",              // example-value
    "changeme", "change_me",// changeme123
    "placeholder",          // placeholder-key
    "todo", "fixme",        // todo-update-this
    "dummy",                // dummy-data
    "test123", "password123", "secret123", "key123",
    "-here", "_here",       // key-here
    "default",              // default-value
    "sample",               // sample-config
];
```

### Validation Flow

```
┌─────────────────────────────────────────────┐
│  Application Startup                        │
│  1. Load .env variables                     │
│  2. Initialize logging                      │
│  3. ▶ VALIDATE PLACEHOLDERS ◀               │
│     └─ Check DB_ENCRYPTION_KEY              │
│     └─ Check SESSION_SECRET_KEY             │
│     └─ Check JWT_SECRET                     │
│     └─ Check ARBITER_PUBKEY                 │
│  4. Continue initialization...              │
└─────────────────────────────────────────────┘
              │
              ├─ DEV MODE (debug build)
              │  └─ Log WARNING + Continue
              │
              └─ PROD MODE (release build)
                 └─ PANIC if placeholder found
```

### Critical Environment Variables Validated

| Variable | Purpose | Example Placeholder | Example Secure Value |
|----------|---------|-------------------|-------------------|
| `DB_ENCRYPTION_KEY` | SQLCipher encryption | `your-64-char-hex-key-here` | `8dca8a38790f...` |
| `SESSION_SECRET_KEY` | Cookie signing | `your_secret_key_here` | `dK3mN8pQvL2x...` |
| `JWT_SECRET` | JWT token signing | `changeme` | `7fR9kP3mL6x...` |
| `ARBITER_PUBKEY` | Arbiter public key | `example-key` | `a1b2c3d4e5f6...` |

---

## Usage

### In Application Code

The validation is automatically called during server startup in `server/src/main.rs`:

```rust
#[actix_web::main]
async fn main() -> Result<()> {
    // 1. Load environment variables
    dotenvy::dotenv().ok();

    // 2. Initialize logging
    tracing_subscriber::registry()...;

    // 3. CRITICAL SECURITY: Validate placeholders
    server::security::placeholder_validator::validate_all_critical_env_vars();

    // 4. Continue with database, wallet setup, etc.
    // ...
}
```

### Manual Validation (Optional)

You can also validate specific variables manually:

```rust
use server::security::placeholder_validator::validate_no_placeholders;

// Validate a single variable
validate_no_placeholders("API_KEY", &api_key);
```

---

## Testing

### Unit Tests

Run built-in Rust tests:

```bash
cargo test --package server placeholder_validator
```

**Tests included:**
- ✅ Placeholder pattern detection
- ✅ Legitimate value acceptance
- ✅ Case-insensitive detection
- ✅ Edge cases (empty, short values)

### Integration Tests

Run the comprehensive test script:

```bash
./SPECIALSEC/tests/test_placeholder_validation.sh
```

**Test scenarios:**
1. **Placeholder detection** - Reject `your-xxx-here`, `changeme`, etc.
2. **Valid values** - Accept legitimate hex/base64 keys
3. **Case insensitivity** - Detect `YOUR-SECRET`, `ChangeMe`
4. **Edge cases** - Handle empty/short values gracefully

---

## Production Behavior

### Scenario 1: Valid Configuration ✅

```bash
# .env
DB_ENCRYPTION_KEY=8dca8a38790f2ce50422553309fa4f756dfd50d7c67a0aba2009d688b64ea811
SESSION_SECRET_KEY=dK3mN8pQvL2xYwZ6tB9jR4sA7fH1gE5nC0uI2oM3kP8vT6qX9rW1lJ4hD7yS0bF=
```

**Result:**
```
🔍 Validating critical environment variables for placeholder patterns...
✅ All critical environment variables validated successfully
Starting Monero Marketplace Server
```

**Server starts successfully** ✅

---

### Scenario 2: Placeholder Detected ❌

```bash
# .env (copied from .env.example)
DB_ENCRYPTION_KEY=your-64-char-hex-key-here
SESSION_SECRET_KEY=changeme
```

**Result (DEVELOPMENT MODE):**
```
⚠️  WARNING: DB_ENCRYPTION_KEY contains placeholder pattern 'your-'
Value: your-64-char-hex-key-here
This indicates you copied .env.example without changing the values.
Generate secure credentials before deploying to production.
⚠️  This would PANIC in production!
```

**Server continues in dev mode** (with warning)

**Result (PRODUCTION MODE):**
```
🚨 SECURITY ERROR: DB_ENCRYPTION_KEY contains placeholder pattern 'your-'
Value: your-64-char-hex-key-here
This indicates you copied .env.example without changing the values.
Generate secure credentials before deploying to production.
See CLAUDE.md for credential generation instructions.

thread 'main' panicked at 'SECURITY ERROR: DB_ENCRYPTION_KEY...'
```

**Server PANICS and refuses to start** ❌

---

## Generating Secure Credentials

### DB_ENCRYPTION_KEY (64-char hex)

```bash
# Linux/macOS
openssl rand -hex 32

# Windows PowerShell
-join ((48..57) + (65..70) | Get-Random -Count 64 | % {[char]$_})

# Rust
use rand::Rng;
let key: String = (0..64)
    .map(|_| format!("{:x}", rand::thread_rng().gen::<u8>() % 16))
    .collect();
```

**Example output:** `8dca8a38790f2ce50422553309fa4f756dfd50d7c67a0aba2009d688b64ea811`

### SESSION_SECRET_KEY (base64, 64+ bytes)

```bash
# Linux/macOS
openssl rand -base64 48

# Windows PowerShell
[Convert]::ToBase64String((1..48 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))

# Online (use with caution)
# https://www.random.org/bytes/ (48 bytes, format: base64)
```

**Example output:** `dK3mN8pQvL2xYwZ6tB9jR4sA7fH1gE5nC0uI2oM3kP8vT6qX9rW1lJ4hD7yS0bF=`

### JWT_SECRET (hex or base64)

Same as `SESSION_SECRET_KEY` generation.

### ARBITER_PUBKEY (Ed25519 public key)

```bash
# Generate with airgap script
./scripts/airgap/generate-arbiter-keypair.sh

# Or manually with ssh-keygen
ssh-keygen -t ed25519 -f arbiter_key
# Extract public key hex from arbiter_key.pub
```

---

## CI/CD Integration

### Pre-deployment Validation

Add to your deployment pipeline:

```yaml
# .github/workflows/deploy.yml
jobs:
  validate-config:
    runs-on: ubuntu-latest
    steps:
      - name: Check for placeholder values
        run: |
          if grep -r "your-.*-here" .env* ; then
            echo "❌ Found placeholder values in .env files!"
            exit 1
          fi

      - name: Validate environment variables
        run: |
          # Export production secrets from GitHub Secrets
          export DB_ENCRYPTION_KEY="${{ secrets.DB_ENCRYPTION_KEY }}"
          export SESSION_SECRET_KEY="${{ secrets.SESSION_SECRET_KEY }}"

          # Dry-run server startup (validates but doesn't run)
          cargo run --release --package server -- --validate-config
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 AS builder

# Build the application
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Copy binary
COPY --from=builder /target/release/server /usr/local/bin/

# IMPORTANT: Never bake secrets into the image
# Secrets must be provided at runtime via environment variables

# Entrypoint validates configuration before starting
ENTRYPOINT ["/usr/local/bin/server"]
```

---

## Troubleshooting

### Issue: "SECURITY ERROR" on startup

**Cause:** Environment variable contains a placeholder pattern.

**Solution:**
1. Check which variable failed: Look at the panic message
2. Generate a secure value: Use `openssl rand -hex 32` or similar
3. Update `.env` file with the new value
4. Restart the server

**Example:**
```bash
# Find the problematic variable
cat .env | grep "your-"

# Generate new secure value
NEW_KEY=$(openssl rand -hex 32)

# Update .env
sed -i "s/your-64-char-hex-key-here/$NEW_KEY/" .env

# Restart
./start-server.sh
```

---

### Issue: False positive detection

**Cause:** Legitimate value contains a pattern like "example" or "your".

**Solution:**
Legitimate values should NOT contain these patterns. If you have a legitimate reason:

1. **Preferred:** Change the value to not contain the pattern
2. **Alternative:** Modify `PLACEHOLDER_PATTERNS` in `placeholder_validator.rs`

**Example:**
```rust
// If you need to allow "example.com" in a URL:
const PLACEHOLDER_PATTERNS: &[&str] = &[
    // "example",  // Commented out
    "example-key",  // More specific pattern
    // ...
];
```

⚠️ **Warning:** Disabling patterns reduces security. Only do this if absolutely necessary.

---

## Security Best Practices

### ✅ DO

- **Generate unique credentials** for each environment (dev/staging/prod)
- **Use a password manager** to store production secrets securely
- **Rotate credentials** regularly (every 90 days recommended)
- **Test in staging** with production-like (but different) credentials first
- **Use CI/CD secrets management** (GitHub Secrets, AWS Secrets Manager, Vault)

### ❌ DON'T

- **Never commit** `.env` files to version control (use `.gitignore`)
- **Never share** production credentials via email/Slack/Discord
- **Never reuse** the same credentials across environments
- **Never bake** secrets into Docker images or binaries
- **Never disable** placeholder validation in production

---

## Related Documentation

- **Credential Generation:** [CLAUDE.md](../../CLAUDE.md#generating-secure-credentials)
- **Deployment Guide:** [PRODUCTION-ROADMAP.md](../../docs/PRODUCTION-ROADMAP.md)
- **Security Checklist:** [SECURITY-CHECKLIST-PRODUCTION.md](../../docs/SECURITY-CHECKLIST-PRODUCTION.md)
- **Audit Results:** [SPECIALSEC/README.md](../../SPECIALSEC/README.md)

---

## Implementation Details

**Location:** `server/src/security/placeholder_validator.rs`
**Tests:** `server/src/security/placeholder_validator.rs` (unit tests)
**Integration Tests:** `SPECIALSEC/tests/test_placeholder_validation.sh`
**Called From:** `server/src/main.rs:89`

**Key Functions:**
- `validate_no_placeholders(var_name, value)` - Validate single variable
- `validate_all_critical_env_vars()` - Validate all critical variables

**Behavior:**
- **Debug builds:** Log warning, continue startup
- **Release builds:** Panic immediately, refuse startup

---

## Audit Trail

**Initial Audit Finding:** B+ grade - "Risk of deployment with default credentials"
**Implementation Date:** 2025-11-07
**Implemented By:** Claude (via GitHub Issue)
**Status:** ✅ **RESOLVED** - Production deployment now impossible with placeholders

**Audit Score Impact:**
- **Before:** B+ (Placeholder credentials risk)
- **After:** A- (Critical security gap resolved)

---

## Future Enhancements

### Planned Improvements

1. **Key strength validation**
   - Detect weak keys (e.g., "12345678...")
   - Enforce minimum entropy requirements

2. **Duplicate detection**
   - Detect if multiple environments use the same credentials
   - Warn if dev/prod keys are identical

3. **External validation**
   - Check if keys have been leaked (Have I Been Pwned API)
   - Validate against known weak key databases

4. **Automated rotation**
   - Generate new credentials automatically
   - Blue-green credential rotation during deployment

### Contributing

To add new placeholder patterns:

1. Edit `PLACEHOLDER_PATTERNS` in `server/src/security/placeholder_validator.rs`
2. Add test cases in the `#[cfg(test)]` section
3. Run tests: `cargo test placeholder_validator`
4. Submit pull request with rationale

---

**Last Updated:** 2025-11-07
**Version:** 1.0.0
**Maintainer:** Security Team
