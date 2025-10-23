# Build & Test Guide - Reputation System

**Production-Grade Build and Testing Instructions**

---

## 🔧 Prerequisites

### 1. Install Rust

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 2. Install wasm-pack (for WASM module)

```bash
cargo install wasm-pack

# Verify
wasm-pack --version
```

### 3. Install IPFS (for REP.2 backend)

```bash
# Download IPFS
wget https://dist.ipfs.io/go-ipfs/v0.20.0/go-ipfs_v0.20.0_linux-amd64.tar.gz
tar -xvzf go-ipfs_v0.20.0_linux-amd64.tar.gz
cd go-ipfs
sudo bash install.sh

# Initialize
ipfs init

# Start daemon (run in background)
ipfs daemon &

# Verify
curl http://127.0.0.1:5001/api/v0/version
```

---

## 📦 Build Instructions

### Step 1: Build Common Types (REP.1)

```bash
cd reputation/common/

# Check compilation
cargo check

# Run tests
cargo test

# Expected output:
# running 4 tests
# test tests::test_review_serialization ... ok
# test tests::test_invalid_rating_rejected ... ok
# test tests::test_comment_validation ... ok
# test tests::test_vendor_reputation_new ... ok
```

### Step 2: Build Crypto Module (REP.1)

```bash
cd ../crypto/

# Check compilation
cargo check

# Run tests
cargo test

# Expected output:
# running 5 tests
# test tests::test_sign_and_verify_review ... ok
# test tests::test_tampered_review_fails_verification ... ok
# test tests::test_invalid_rating_rejected ... ok
# test tests::test_calculate_stats ... ok
# test tests::test_empty_reviews_stats ... ok
```

### Step 3: Build WASM Module (REP.3)

```bash
cd ../wasm/

# Build for web target
./build.sh

# OR manually:
wasm-pack build \
    --target web \
    --release \
    --out-dir pkg \
    --out-name reputation_wasm

# Verify output
ls -lh pkg/
# Should see:
# reputation_wasm_bg.wasm (~150KB)
# reputation_wasm.js
# reputation_wasm.d.ts
# package.json

# Run WASM tests
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome
```

### Step 4: Build Complete Workspace

```bash
cd ../

# Build entire workspace
cargo build --workspace --release

# Run all tests
cargo test --workspace

# Check for warnings
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --workspace --check
```

---

## 🧪 Testing Strategy

### Unit Tests

**Common types:**
```bash
cd reputation/common/
cargo test -- --nocapture

# Coverage (requires tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Stdout --output-dir coverage/
# Target: ≥80% coverage
```

**Crypto module:**
```bash
cd reputation/crypto/
cargo test -- --nocapture

# Test specific function
cargo test test_sign_and_verify_review -- --nocapture
```

**WASM module:**
```bash
cd reputation/wasm/

# Browser tests
wasm-pack test --headless --firefox
wasm-pack test --headless --chrome

# Node.js tests
wasm-pack test --node
```

### Integration Tests (Manual)

#### Test 1: WASM Verification in Browser

1. **Build and serve WASM:**
   ```bash
   cd reputation/wasm/
   ./build.sh

   # Verify files copied to static/
   ls -lh ../../static/wasm/
   ```

2. **Start simple HTTP server:**
   ```bash
   cd ../../static/
   python3 -m http.server 8000
   ```

3. **Create test HTML:**
   ```html
   <!-- test-wasm.html -->
   <!DOCTYPE html>
   <html>
   <head>
       <title>WASM Test</title>
   </head>
   <body>
       <h1>Reputation WASM Test</h1>
       <div id="result"></div>

       <script type="module">
           import init, { verify_reputation_file, get_version } from './wasm/reputation_wasm.js';

           async function test() {
               await init();

               const version = get_version();
               console.log('WASM Version:', version);

               // Test empty reputation
               const emptyRep = {
                   format_version: "1.0",
                   vendor_pubkey: "test",
                   generated_at: new Date().toISOString(),
                   reviews: [],
                   stats: {
                       total_reviews: 0,
                       average_rating: 0.0,
                       rating_distribution: [0,0,0,0,0],
                       oldest_review: new Date().toISOString(),
                       newest_review: new Date().toISOString()
                   }
               };

               const result = verify_reputation_file(JSON.stringify(emptyRep));
               console.log('Verification result:', result);

               document.getElementById('result').innerHTML = `
                   <p>Version: ${version}</p>
                   <p>Valid: ${result.is_valid}</p>
                   <p>Total: ${result.total_reviews}</p>
               `;
           }

           test().catch(console.error);
       </script>
   </body>
   </html>
   ```

4. **Open in browser:**
   ```
   http://localhost:8000/test-wasm.html
   ```

5. **Expected console output:**
   ```
   WASM Version: 0.1.0
   Verification result: { is_valid: true, total_reviews: 0, ... }
   ```

#### Test 2: End-to-End Review Flow

**Prerequisites:**
- Server running (`cargo run --release` in `server/`)
- IPFS daemon running (`ipfs daemon`)
- Database migrated (`diesel migration run`)

**Test steps:**

1. **Create test review:**
   ```bash
   # Generate signing key
   cd reputation/crypto/

   # Create test program
   cat > test_sign.rs << 'EOF'
   use ed25519_dalek::SigningKey;
   use rand::rngs::OsRng;
   use reputation_crypto::reputation::sign_review;

   fn main() {
       let mut csprng = OsRng;
       let signing_key = SigningKey::generate(&mut csprng);

       let review = sign_review(
           "test_tx_abc123".to_string(),
           5,
           Some("Excellent!".to_string()),
           &signing_key,
       ).unwrap();

       println!("{}", serde_json::to_string_pretty(&review).unwrap());
   }
   EOF

   # Run
   rustc test_sign.rs --edition 2021 -L target/release/deps
   ./test_sign
   ```

2. **Submit via API:**
   ```bash
   # Get CSRF token from session
   CSRF_TOKEN="your_csrf_token"
   VENDOR_ID="vendor_uuid"

   curl -X POST http://localhost:8080/api/reviews \
     -H "Content-Type: application/json" \
     -b "session_cookie" \
     -d '{
       "review": {
         "txid": "test_tx_abc123",
         "rating": 5,
         "comment": "Excellent!",
         "timestamp": "2025-10-22T12:00:00Z",
         "buyer_pubkey": "base64_pubkey_here",
         "signature": "base64_signature_here"
       },
       "csrf_token": "'$CSRF_TOKEN'",
       "vendor_id": "'$VENDOR_ID'"
     }'
   ```

3. **Verify in database:**
   ```bash
   sqlite3 server/marketplace.db "SELECT * FROM reviews;"
   ```

4. **Fetch reputation:**
   ```bash
   curl http://localhost:8080/api/reputation/$VENDOR_ID
   ```

5. **Export to IPFS:**
   ```bash
   curl -X POST http://localhost:8080/api/reputation/export \
     -H "Content-Type: application/json" \
     -b "session_cookie" \
     -d '{
       "vendor_id": "'$VENDOR_ID'",
       "csrf_token": "'$CSRF_TOKEN'"
     }'

   # Expected response:
   # {
   #   "status": "success",
   #   "ipfs_hash": "QmXxxx...",
   #   "gateway_url": "http://127.0.0.1:8080/ipfs/QmXxxx..."
   # }
   ```

6. **Retrieve from IPFS:**
   ```bash
   # Get IPFS hash from previous response
   IPFS_HASH="QmXxxx..."

   curl http://127.0.0.1:8080/ipfs/$IPFS_HASH
   ```

7. **Verify client-side (browser):**
   - Open vendor profile: `http://localhost:8080/vendor/$VENDOR_ID`
   - Check browser console for: "✅ Reputation WASM v0.1.0 initialized"
   - Verification badge should show: "✅ Verified - X reviews"

---

## 🔍 Quality Checks

### 1. Security Theatre Detection

```bash
cd reputation/

# Run security check script (from parent project)
../scripts/check-security-theatre.sh .

# Expected: No violations found
```

### 2. Clippy Lints

```bash
cargo clippy --workspace -- -D warnings

# Should return: 0 warnings
```

### 3. Code Formatting

```bash
cargo fmt --workspace --check

# Should return: no diffs
```

### 4. Test Coverage

```bash
cargo install cargo-tarpaulin

cd reputation/
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Open coverage/index.html
# Target: ≥80% for common, crypto
# Target: ≥70% for wasm (harder to test)
```

---

## 📊 Performance Benchmarks

### WASM Module Size

```bash
cd reputation/wasm/pkg/

# Check WASM file size
ls -lh reputation_wasm_bg.wasm

# Target: <200KB uncompressed
# Target: <60KB gzipped

# Compress and check
gzip -c reputation_wasm_bg.wasm | wc -c
```

### Signature Verification Speed

**Test in browser console:**
```javascript
// Load 1000 reviews
const reviews = Array(1000).fill({
    txid: "test",
    rating: 5,
    comment: "Test",
    timestamp: new Date().toISOString(),
    buyer_pubkey: "valid_pubkey_base64",
    signature: "valid_signature_base64"
});

const reputation = {
    format_version: "1.0",
    vendor_pubkey: "vendor",
    generated_at: new Date().toISOString(),
    reviews: reviews,
    stats: { /* ... */ }
};

// Benchmark
console.time('verify_1000_reviews');
const result = verify_reputation_file(JSON.stringify(reputation));
console.timeEnd('verify_1000_reviews');

// Target: <1000ms for 1000 reviews
```

---

## 🐛 Troubleshooting

### Error: "wasm-pack not found"

```bash
cargo install wasm-pack
```

### Error: "failed to load WASM module"

**Check browser console:**
- CORS errors? → Ensure server serves `/static/wasm/` correctly
- 404 errors? → Run `./build.sh` to copy files to `static/`

**Verify files exist:**
```bash
ls -lh static/wasm/
# Should show:
# reputation_wasm_bg.wasm
# reputation_wasm.js
```

### Error: "IPFS daemon not running"

```bash
# Start IPFS daemon
ipfs daemon &

# Verify
curl http://127.0.0.1:5001/api/v0/version
```

### Error: "Database locked"

```bash
# Stop any running server instances
pkill -f "cargo run"

# Restart server
cd server/
cargo run --release
```

### Build fails with "linker error"

```bash
# Install build dependencies
sudo apt update
sudo apt install build-essential pkg-config libssl-dev

# Retry build
cargo clean
cargo build --workspace
```

---

## ✅ Success Criteria

### REP.1 (Types & Crypto)
- [x] `cargo test --package reputation-common` → All tests pass
- [x] `cargo test --package reputation-crypto` → All tests pass
- [x] Coverage ≥80% for both crates

### REP.3 (WASM)
- [x] `wasm-pack build` → Successful build
- [x] WASM file size <200KB
- [x] Browser tests pass (`wasm-pack test --headless`)
- [x] Manual verification in browser works

### REP.4 (Frontend)
- [x] Templates render correctly
- [x] HTMX interactions work (no page reload)
- [x] WASM verification badge updates automatically
- [x] Review submission flow complete
- [x] IPFS export functional

### Overall
- [x] Zero `.unwrap()` or `.expect()` (except tests)
- [x] Zero `TODO` comments in production code
- [x] All Clippy warnings resolved
- [x] Code formatted with `cargo fmt`
- [x] No security theatre violations

---

## 📝 Next Steps

After successful build and tests:

1. **Commit changes:**
   ```bash
   git add reputation/
   git commit -m "feat: REP.3 & REP.4 - WASM verification + frontend integration"
   ```

2. **Run security audit:**
   ```bash
   ../scripts/check-security-theatre.sh reputation/
   ../scripts/security-dashboard.sh
   ```

3. **Integration with main server:**
   - Copy templates to `templates/reputation/`
   - Copy static files to `static/`
   - Update server routes

4. **Deploy to staging:**
   - Build WASM in release mode
   - Start IPFS daemon
   - Run E2E tests

---

**Ready for production deployment after passing all checks! ✅**
