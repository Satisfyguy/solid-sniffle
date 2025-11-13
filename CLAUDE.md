# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Monero Marketplace** is a secure, privacy-focused marketplace platform that runs as a Tor hidden service with Monero-only escrow using 2-of-3 multisig. This is a high-security educational project with strict OPSEC requirements and automated enforcement against "security theatre" (code that appears secure but isn't).

**Current Status:** Alpha (v0.3.0) - Testnet only, NOT for production use.

**📋 Après chaque commit significatif:** Exécuter `/alpha-terminal` pour vérification anti-hallucination + mise à jour doc. Voir [PROTOCOLE-ALPHA-TERMINAL.md](DOX/protocols/PROTOCOLE-ALPHA-TERMINAL.md)

## Design Philosophy: Monero-Only Architecture

**Monero-only is not a limitation—it's a non-negotiable architectural requirement.**

This marketplace exclusively supports Monero (XMR) as the only payment rail. This is an intentional design decision, not a temporary constraint.

**Why Monero-only?**
- **Privacy guarantees:** Unlinkability and fungibility at protocol level (ring signatures, stealth addresses, RingCT)
- **No compromises:** Multi-crypto support would dilute security guarantees by introducing non-fungible chains (Bitcoin, Ethereum) or custodial bridges
- **OpSec integrity:** Single attack surface, single threat model, focused expertise
- **Technical honesty:** "Privacy-first" means rejecting all transparent blockchains

**Trade-off accepted:** Lower volume potential (Monero is ~0.5% of crypto market cap, ~50-100K daily active users) in exchange for cryptographic privacy guarantees without compromise.

**The question "Is Monero-only too niche?" is incorrectly framed.** The correct question is: "Does the architecture deliver on its privacy promise?" The niche size is a direct consequence of architectural integrity—this is success, not failure.

See [ADR-001: Monero-Only Rationale](DOX/architecture/ADR-001-MONERO-ONLY-RATIONALE.md) for complete technical justification.

## Professional Objectivity

**Truth over comfort. Effectiveness over appearance.**

This project demands unfiltered technical honesty. Your role is to deliver objective analysis without diplomatic softening or emotional consideration for user sensibilities.

### Core Principles

- **Prioritize technical accuracy** over user validation or comfort
- **Challenge assumptions directly** - identify flawed reasoning immediately
- **Focus on what actually works** vs what appears to work or sounds good
- **Disagree when necessary** - even if uncomfortable for the user
- **No unnecessary preambles** - no "I think maybe possibly..." hedging
- **No excessive praise** - no "great job!" unless truly exceptional
- **No emotional reassurance** - not a therapist, you're a technical advisor
- **State problem → State solution → Move forward**

### Anti-Theatre Evaluation

When evaluating any solution, deconstruct surface-level claims and focus on verifiable reality:

**Security:**
- Real protection mechanisms vs security theatre
- Actual threat mitigation vs security-sounding buzzwords
- Measurable risk reduction vs compliance checkbox

**Architecture:**
- Actual scalability (load tested) vs claimed scalability
- Real fault tolerance vs assumed resilience
- Proven patterns vs trendy but unvalidated approaches

**Performance:**
- Measured benchmarks vs assumed speed
- Profiler data vs gut feelings
- Real-world latency vs theoretical optimality

**Code Quality:**
- Production-ready vs "works on my machine"
- Proper error handling vs happy-path-only code
- Maintainable vs clever-but-obscure

### Communication Style

**DO:**
- Be direct and concise
- Use technical precision
- Point out mistakes immediately
- Provide actionable corrections
- Maintain professional respect

**DON'T:**
- Use euphemisms or softening language
- Add unnecessary apologies or disclaimers
- Provide false encouragement
- Hide hard truths behind politeness
- Waste time on social niceties

**Example - Bad Response:**
```
"I think there might possibly be a small issue here, and I'm not entirely sure,
but maybe we could consider perhaps looking at the error handling? Just a thought!"
```

**Example - Good Response:**
```
"This code has no error handling. The .unwrap() on line 47 will panic in production
when the RPC is unreachable. Replace with proper Result propagation using .context()."
```

### When To Be Especially Direct

- **Debugging loops** - After 4-5 failed attempts, stop and request detailed problem report
- **Security issues** - No softening, immediate correction required
- **Architecture flaws** - Call out fundamental design problems early
- **Performance myths** - Debunk assumptions with data
- **False progress** - Don't pretend broken code is "almost working"

Maintain professional respect while being brutally honest about technical realities. The user benefits more from hard truths than comfortable lies.

## 🚨 Non-Custodial Migration (Phase 3 - ACTIVE)

**Status:** Phase 3 Dépréciation - Mode custodial déprécié, suppression prévue v0.4.0

### Fonctions Dépréciées

⚠️ **Ces fonctions génèrent des warnings et seront supprimées dans v0.4.0 (2-3 semaines):**

1. **`WalletManager::create_temporary_wallet()`** - Crée wallets sur serveur (CUSTODIAL)
2. **`EscrowOrchestrator::init_escrow()`** - Utilise create_temporary_wallet en interne

### Migration Recommandée

**❌ Ancien (Déprécié):**
```rust
let wallet_id = wallet_manager.create_temporary_wallet(escrow_id, "buyer").await?;
```

**✅ Nouveau (Recommandé):**
```rust
coordinator.register_client_wallet(escrow_id, EscrowRole::Buyer, "http://127.0.0.1:18083").await?;
```

### Documentation Migration

- **Guide utilisateur:** `DOX/guides/NON-CUSTODIAL-USER-GUIDE.md`
- **Guide migration:** `DOX/guides/MIGRATION-TO-NONCUSTODIAL.md`
- **Plan complet:** `DOX/guides/MIGRATION-NON-CUSTODIAL-PLAN.md`

### CLI Non-Custodial

```bash
# Lancer wallet RPC local
monero-wallet-rpc --testnet --rpc-bind-port 18083 --disable-rpc-login --offline

# Initialiser escrow non-custodial
cargo run --bin monero-marketplace -- noncustodial init-escrow \
  --escrow-id "escrow_001" \
  --role buyer \
  --wallet-name "my_wallet" \
  --local-rpc-url "http://127.0.0.1:18083" \
  --server-url "http://localhost:8080"
```

## Critical Security Context

This project has **zero tolerance for security theatre**. All code is automatically scanned for:
- Unwrapped results without error handling
- Placeholder comments (TODO/FIXME)
- Magic numbers without constants
- Hardcoded credentials
- Logging of sensitive data (.onion addresses, keys, IPs)

**Before generating ANY code**, run: `./scripts/check-security-theatre.sh`

## Quick Audit - Pragmatic Check

**Before pushing code or starting work**, run the pragmatic audit to verify project health:

```bash
./scripts/audit-pragmatic.sh
```

This script (128 lines, <5 seconds) checks:
1. **Database**: schema.rs, marketplace.db, migrations
2. **Configuration**: .env secrets, git tracking
3. **Monero**: RPC localhost, multisig implementation
4. **Tor**: Daemon status, SOCKS port, no public exposure
5. **Tests**: Unit tests, E2E tests presence
6. **Security**: No private keys in logs, CSRF protection

**Exit codes:**
- `0` = All OK (score 100/100)
- `1` = Critical issues (must fix before continuing)
- `2` = High warnings (recommended fixes)

**Compared to other audit scripts:**
- `audit-pragmatic.sh`: **128 lines, <5s, 0 false positives** ✅
- `swissy.sh`: 9.5K, 2+ min, many false positives
- `suissemade.sh`: 82K (2164 lines), 5+ min, massive false positives

**Use this as your daily audit**, not the bloated alternatives.

## Build & Test Commands

### Development Environment
```bash
# Build all workspace members
cargo build --workspace

# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test --package wallet
cargo test --package common
cargo test --package cli

# Run single test
cargo test --package wallet test_prepare_multisig -- --nocapture

# Lint (STRICT enforcement)
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --workspace

# Check compilation without building
cargo check --workspace
```

### Pre-commit Validation
```bash
# Full pre-commit checks (runs automatically on git commit)
./scripts/pre-commit.sh

# Security theatre detection
./scripts/check-security-theatre.sh --verbose

# Security dashboard
./scripts/security-dashboard.sh

# Security alerts
./scripts/security-alerts.sh
```

### Audit & Validation
```bash
# Run the full audit script
./scripts/swissy.sh
```

### Monero Testnet Setup
```bash
# Install Tor daemon (if not already installed)
sudo apt update && sudo apt install -y tor
sudo systemctl start tor
sudo systemctl enable tor

# Verify Tor is running
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org

# Setup Monero testnet wallets
./scripts/setup-monero-testnet.sh

# Start testnet (if needed)
./scripts/start-testnet.sh

# Test RPC connectivity
./scripts/test-rpc.sh
```

## Architecture Overview

### Workspace Structure
This is a Cargo workspace with 3 crates:

1. **`common/`** - Shared types, errors, constants
   - `types.rs` - RPC types, Monero types, wallet structs
   - `error.rs` - Custom error types (Error, TorError, MoneroError)
   - `utils.rs` - Utility functions (ID generation, conversions)
   - Constants exported from `lib.rs` (MONERO_RPC_URL, XMR_TO_ATOMIC, etc.)

2. **`wallet/`** - Monero integration layer
   - `rpc.rs` - Low-level Monero RPC client (MoneroRpcClient)
   - `multisig.rs` - Multisig operations (MultisigManager)
   - `client.rs` - High-level Monero client (MoneroClient)

3. **`cli/`** - Command-line interface
   - `main.rs` - CLI entry point
   - `test_tool.rs` - Manual testing utilities

### Key Design Patterns

**Error Handling:**
- All functions return `Result<T, E>`
- Use `.context("clear message")?` from anyhow for error propagation
- Never use `.unwrap()` or `.expect()` without explicit justification
- Custom error types: `Error`, `TorError`, `MoneroError`

**RPC Client:**
- Thread-safe with Arc<Mutex<>> for serialization
- Semaphore-based rate limiting (max 5 concurrent requests)
- Retry logic with exponential backoff
- Strict localhost-only validation (blocks non-127.0.0.1 URLs)

**Monero Multisig Flow:**
The multisig setup is a strict 6-step process:
1. `prepare_multisig()` - Each party generates multisig info
2. `make_multisig(infos)` - Exchange info to create multisig wallet
3. `export_multisig_info()` - Export sync info
4. `import_multisig_info(infos)` - Import others' sync info
5. Repeat steps 3-4 (sync round 2)
6. `is_multisig()` - Verify wallet is ready

**State must be checked before each step** - wallet can already be in multisig mode.

## Strict Development Rules

### Absolute Prohibitions
1. **Never expose Monero RPC publicly** - Must bind to 127.0.0.1 only
2. **Never log sensitive data** - No .onion addresses, keys, passwords, IPs
3. **Never use .unwrap() or .expect()** - Use proper error handling
4. **Never use println!() in production** - Use `tracing::info!()` instead
5. **Never commit placeholder comments** - No TODO/FIXME without tracking
6. **Never hardcode credentials** - Use environment variables
7. **Never use magic numbers** - Define constants in common/src/lib.rs

### Required Patterns

**Network Code (Tor-aware):**
```rust
use reqwest::Proxy;

async fn fetch_via_tor(url: &str) -> Result<String> {
    let proxy = Proxy::all("socks5h://127.0.0.1:9050")
        .context("Failed to configure Tor proxy")?;

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; rv:102.0) Gecko/20100101 Firefox/102.0")
        .timeout(Duration::from_secs(30))  // Tor is slow
        .build()?;

    let response = client.get(url).send().await?;

    // OPSEC: Never log URLs (may contain .onion)
    tracing::debug!("Tor request completed");

    response.text().await.context("Failed to read response")
}
```

**Monero RPC Calls:**
```rust
pub async fn my_rpc_call(&self) -> Result<MyType, MoneroError> {
    let _permit = self.semaphore.acquire().await
        .map_err(|_| MoneroError::NetworkError("Semaphore closed".to_string()))?;

    let _guard = self.rpc_lock.lock().await;

    let request = RpcRequest::new("method_name");

    let response = self.client
        .post(&format!("{}/json_rpc", self.url))
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() {
                MoneroError::RpcUnreachable
            } else {
                MoneroError::NetworkError(e.to_string())
            }
        })?;

    let rpc_response: RpcResponse<MyResultType> = response
        .json()
        .await
        .map_err(|e| MoneroError::InvalidResponse(format!("JSON parse: {}", e)))?;

    if let Some(error) = rpc_response.error {
        return Err(MoneroError::RpcError(error.message));
    }

    let result = rpc_response.result
        .ok_or_else(|| MoneroError::InvalidResponse("Missing result".to_string()))?;

    Ok(result.into())
}
```

## Debugging Protocol

**CRITICAL: After 4-5 unsuccessful debugging attempts, ALWAYS request a detailed problem report.**

### When Debugging Stalls

If you've attempted to fix an issue **4-5 times** without success:

1. **STOP** further debugging attempts
2. **REQUEST** the user to create a detailed problem report with:
   - **Exact error messages** (full stack traces)
   - **Steps to reproduce** (command sequence)
   - **Expected vs actual behavior**
   - **Environment details** (OS, Rust version, dependencies)
   - **Recent changes** (commits, config modifications)
   - **Logs** (relevant excerpts with timestamps)

3. **DOCUMENT** the attempts made so far:
   - What was tried
   - Why each attempt failed
   - What was learned from each failure

**Rationale:** After multiple failed attempts, the problem is likely:
- Misdiagnosed (wrong root cause)
- Environmental (outside codebase)
- Requires deeper investigation
- Needs fresh perspective with complete context

**User Request Format:**
```
Please create a detailed problem report in DOX/debug-notes/YYYY-MM-DD-issue-name.md with:
1. Complete error output
2. Step-by-step reproduction
3. System environment (cargo --version, rustc --version, etc.)
4. Recent changes (git log --oneline -10)
5. Relevant logs (monero-wallet-rpc.log, server.log)

This will help us take a systematic approach to solving the issue.
```

**Benefits:**
- Prevents infinite debug loops
- Forces systematic problem analysis
- Creates documentation for future reference
- Often reveals the issue during documentation process
- Provides clean break to reassess approach

## Specification-Driven Development

**Every function must have a specification before implementation.**

### Workflow
```bash
# 1. Create spec
./scripts/new-spec.sh my_function

# 2. Edit spec at docs/specs/my_function.md
# 3. Implement function
# 4. If function does network calls, create Reality Check
./scripts/auto-reality-check-tor.sh my_function

# 5. Validate Reality Check
./scripts/validate-reality-check-tor.sh my_function
```

### Spec Template Structure
- **Objectif** - What the function does (1 line)
- **Préconditions** - What must be true before calling
- **Input** - Exact parameter types
- **Output** - Return type and Result
- **Erreurs Possibles** - All error variants and when they occur
- **Dépendances** - Required crates/versions
- **Test de Validation** - Bash commands to manually test
- **Estimation** - Time estimates
- **Status** - Checkboxes for completion tracking

## Tor Reality Checks

**For ANY function that makes network calls**, a Tor Reality Check is mandatory.

### What Gets Checked
1. **Tor daemon running** - Is Tor accessible on 127.0.0.1:9050?
2. **No IP leaks** - Does traffic go through Tor?
3. **RPC isolation** - Is Monero RPC on localhost only?
4. **No sensitive logs** - No .onion/keys/IPs in logs?
5. **Port exposure** - No public ports exposed?

### Reality Check Storage
- Auto-generated at: `docs/reality-checks/tor-{function_name}-{date}.md`
- Manual validation required before merge to production
- Validated with: `./scripts/validate-reality-check-tor.sh {function_name}`

## Testing Strategy

### Unit Tests
- Located in `mod tests` at bottom of each file
- Use `#[tokio::test]` for async tests
- Always return `Result<()>` with `?` operator
- Mock external dependencies when possible

### Integration Tests
- Located in `wallet/tests/integration.rs`
- Require running Monero RPC on localhost:18082
- Tests gracefully handle missing RPC (log warning, don't fail)

### E2E Tests (End-to-End)
- Located in `server/tests/escrow_e2e.rs`
- Test complete escrow flows with simulated blockchain operations
- Require database setup with migrations (`./scripts/setup-e2e-tests.sh`)
- Marked with `#[ignore]` - run explicitly with `-- --ignored`
- 5 comprehensive tests covering: normal flow, dispute, state transitions, concurrency
- See [`docs/TESTING.md`](docs/TESTING.md) and [`server/tests/README_E2E.md`](server/tests/README_E2E.md)

**Run E2E tests:**
```bash
# Setup first (creates DB and applies migrations)
./scripts/setup-e2e-tests.sh

# Run all E2E tests
cargo test --package server --test escrow_e2e -- --ignored

# Run specific test with output
cargo test --package server --test escrow_e2e test_complete_escrow_flow -- --ignored --nocapture
```

### Reality Check Tests (Manual)
- DNS leak test
- Fingerprinting test
- Hidden service test (if applicable)
- Traffic analysis check

## Common Constants

**Import from common crate:**
```rust
use monero_marketplace_common::{
    MONERO_RPC_URL,       // "http://127.0.0.1:18082/json_rpc"
    MONERO_RPC_PORT,      // 18082
    XMR_TO_ATOMIC,        // 1_000_000_000_000
    MAX_MULTISIG_INFO_LEN, // 5000
    MIN_MULTISIG_INFO_LEN, // 100
};
```

## Database Migrations & Diesel

**CRITICAL:** Database schema changes require strict adherence to migration workflow to avoid runtime errors.

### Migration Workflow (NEVER SKIP STEPS)

**Every time you modify database schema:**

```bash
# 1. Generate migration files
diesel migration generate add_column_name

# 2. Write SQL in both files
# - up.sql:   ALTER TABLE foo ADD COLUMN bar TEXT DEFAULT 'value';
# - down.sql: ALTER TABLE foo DROP COLUMN bar;

# 3. Apply migration to database (MOST CRITICAL STEP)
DATABASE_URL=marketplace.db diesel migration run

# 4. Verify migration was applied
DATABASE_URL=marketplace.db diesel migration list
# ALL migrations must show [X] (applied), not [ ] (pending)

# 5. Regenerate Rust schema
diesel print-schema > server/src/schema.rs

# 6. Update Rust structs to match new schema
# Add/remove fields from model structs (e.g., Listing, User, Order)

# 7. Recompile
cargo build --release --package server

# 8. Test before deploying
./target/release/server
```

### Common Migration Pitfalls

**❌ PROBLEM #1: Forgetting `diesel migration run`**
```
Error: "Failed to retrieve created listing"
Cause: Rust schema.rs has new column, but database doesn't
Solution: Run diesel migration run
```

**❌ PROBLEM #2: Column count mismatch**
```
Error: Column count mismatch between NewFoo and Foo
Cause: Forgot to add new field to insertion struct
Solution: Add field to both Queryable and Insertable structs
```

**❌ PROBLEM #3: Running server with old binary**
```
Error: Persistent errors after "fixing" code
Cause: Server process still running old binary
Solution: Kill ALL server processes, recompile, restart
```

### Pre-Deploy Checklist

Before starting the server after ANY migration:

```bash
# ✅ Verify all migrations applied
diesel migration list | grep -q "\[ \]" && echo "❌ PENDING MIGRATIONS!" || echo "✅ All applied"

# ✅ Check column count matches
sqlite3 marketplace.db "PRAGMA table_info(listings);" | wc -l
# Compare to number of fields in server/src/models/listing.rs Listing struct

# ✅ Verify binary is fresh
stat -c "%y" target/release/server  # Should be recent

# ✅ Kill old servers
killall -9 server; pkill -9 -f "target/release/server"

# ✅ Start fresh
./target/release/server > server.log 2>&1 &
```

### Migration Best Practices

**1. Always use DEFAULT for new columns:**
```sql
-- ✅ GOOD: Won't break existing rows
ALTER TABLE listings ADD COLUMN images_ipfs_cids TEXT DEFAULT '[]';

-- ❌ BAD: Will fail if rows exist
ALTER TABLE listings ADD COLUMN images_ipfs_cids TEXT NOT NULL;
```

**2. Test migrations on a copy first:**
```bash
cp marketplace.db marketplace_backup.db
DATABASE_URL=test.db diesel migration run
# If successful, apply to real DB
```

**3. Keep NewFoo and Foo structs in sync:**
```rust
// Queryable struct (for SELECT)
#[derive(Queryable, Identifiable)]
pub struct Listing {
    pub id: String,
    // ... all DB columns including new ones
    pub images_ipfs_cids: Option<String>,  // ✅ Added
}

// Insertable struct (for INSERT)
#[derive(Insertable)]
pub struct NewListing {
    pub id: String,
    // ... all required fields
    // ✅ Include new column if no DEFAULT, or if you want to set it explicitly
}
```

### Debugging Migration Issues

**If you see runtime errors after schema changes:**

```bash
# 1. Check migration status
DATABASE_URL=marketplace.db diesel migration list

# 2. Check actual DB schema
sqlite3 marketplace.db ".schema listings"

# 3. Compare to Rust schema
cat server/src/schema.rs | grep -A 15 "listings (id)"

# 4. Check running binary timestamp
ps aux | grep "[t]arget/release/server"
stat -c "%y" target/release/server

# 5. If timestamps don't match, kill and restart
```

### Emergency Rollback

If a migration breaks production:

```bash
# Rollback last migration
DATABASE_URL=marketplace.db diesel migration revert

# Regenerate schema
diesel print-schema > server/src/schema.rs

# Revert Rust code changes
git checkout HEAD -- server/src/models/

# Rebuild and restart
cargo build --release --package server
killall -9 server && ./target/release/server &
```

## Git Hooks

**Pre-commit hook runs automatically** at `.git/hooks/pre-commit` (or via `./scripts/pre-commit.sh`):
1. Cargo check
2. Cargo fmt --check
3. Cargo clippy -- -D warnings
4. Cargo test
5. Security theatre detection
6. Monero+Tor pattern validation
7. Test RPC connectivity

**Commits are blocked if any check fails.**

### Setup Git Hook (Ubuntu)
```bash
# Make pre-commit hook executable
chmod +x .git/hooks/pre-commit

# Or symlink to script
ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
```

## Exception Handling

If a security theatre pattern is legitimate (e.g., `println!` in CLI tools):

**Add to `.security-theatre-ignore`:**
```
# Format: path_pattern:regex_pattern
cli/src/test_tool.rs:println!
**/tests/*.rs:expect\(".*"\)
```

**Requires justification in commit message.**

## OPSEC Guidelines

### Never Log
- .onion addresses
- View/Spend keys
- Passwords
- Real IP addresses
- Circuit information

### Always Use
- SOCKS5 proxy (127.0.0.1:9050) for all external connections
- Generic User-Agent strings
- UTC timezone
- Rounded timestamps to prevent timing correlation

### Always Validate
- RPC URLs are localhost only
- No public port bindings
- Tor daemon is running before network operations
- No sensitive data in error messages

## Threat Model

**Adversaries considered:**
1. **ISP/Network Surveillance** - Mitigated by Tor for all traffic
2. **Exit Node Operators** - Mitigated by .onion services (no exit)
3. **Blockchain Analysis** - Mitigated by Monero's privacy features
4. **Global Passive Adversary** - Partially mitigated (timing analysis difficult but not impossible)

## Important Files

- `.cursorrules` - Comprehensive development rules (1100+ lines)
- `.cargo/config.toml` - Clippy configuration (200+ lints)
- `.security-theatre-ignore` - Legitimate exceptions
- `scripts/pre-commit.sh` - Pre-commit validation pipeline
- `docs/DEVELOPER-GUIDE.md` - Detailed development guide
- `docs/SECURITY-THEATRE-PREVENTION.md` - Security theatre documentation

## Protocole Alpha Terminal

**Après chaque commit significatif**, utiliser le **Protocole Alpha Terminal** pour:
- Vérifier toutes les affirmations (anti-hallucination)
- Évaluer production-readiness (scorecard)
- Mettre à jour PLAN-COMPLET.md
- Identifier actions immédiates

**Activation:**
```
Active le protocole Alpha Terminal
```
ou
```
/alpha-terminal
```

**Documentation complète:** [PROTOCOLE-ALPHA-TERMINAL.md](DOX/protocols/PROTOCOLE-ALPHA-TERMINAL.md)

**Quand utiliser:**
- Après commits avec nouvelles fonctionnalités
- Après résolution de bloqueurs critiques
- Avant reviews de milestone
- Sur demande pour audit du progrès

## When In Doubt

1. Run security dashboard: `./scripts/security-dashboard.sh`
2. Check for alerts: `./scripts/security-alerts.sh`
3. Review relevant spec in `docs/specs/`
4. Check Reality Check in `docs/reality-checks/`
5. Consult `.cursorrules` for specific patterns
6. **Execute Protocole Alpha Terminal:** `/alpha-terminal`
7. Ask before committing questionable code

## Project Philosophy

**"No security theatre. Real security or no security claims."**

This project prioritizes:
- Verifiable security over claimed security
- Automated enforcement over manual review
- Strict rules over flexible guidelines
- Production-ready code only (no placeholders)
- Privacy by default (Tor + Monero)
- Testnet experimentation before mainnet deployment
- always look at the official documentation of monero