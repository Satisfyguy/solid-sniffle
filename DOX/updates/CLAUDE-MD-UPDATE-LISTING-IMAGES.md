# CLAUDE.md Update - Listing Images Fix

**Date:** 2025-10-28

## New Section to Add to CLAUDE.md

Insert this section after "Database Migrations & Diesel" section:

---

### Critical Configuration - DATABASE_URL & IPFS

**IMPORTANT: Two configuration issues can break listing creation and image display.**

#### Issue 1: DATABASE_URL Format

**❌ WRONG - Creates file named "sqlite:marketplace.db":**
```bash
# .env
DATABASE_URL=sqlite:marketplace.db
```

**✅ CORRECT - Use absolute path:**
```bash
# .env
DATABASE_URL=/home/malix/Desktop/monero.marketplace/marketplace.db
```

**Why:** Diesel's `ConnectionManager<SqliteConnection>` treats the entire DATABASE_URL as a filename when it doesn't recognize the URL scheme. The string "sqlite:" is not parsed as a scheme and becomes part of the filename.

**Verification:**
```bash
# Check .env file
cat .env | grep DATABASE_URL

# Verify database exists at correct location
ls -lh marketplace.db  # Should show 188K+, not 0 bytes

# Check for wrong file
ls -lh "sqlite:marketplace.db" 2>/dev/null && echo "❌ Wrong DB file exists!"
```

**Starting Server:**
```bash
# Always unset env var to avoid conflicts
env -u DATABASE_URL ./target/release/server
```

#### Issue 2: IPFS Gateway Port

**Location:** `server/src/ipfs/client.rs:97-102`

**❌ WRONG - Port 8080 is marketplace server:**
```rust
pub fn new_local() -> Result<Self> {
    Self::new(
        "http://127.0.0.1:5001/api/v0".to_string(),
        "http://127.0.0.1:8080/ipfs".to_string(),  // Wrong!
    )
}
```

**✅ CORRECT - Port 8081 is IPFS gateway:**
```rust
pub fn new_local() -> Result<Self> {
    Self::new(
        "http://127.0.0.1:5001/api/v0".to_string(),
        "http://127.0.0.1:8081/ipfs".to_string(),  // Correct!
    )
}
```

**Verification:**
```bash
# Check IPFS gateway configuration
ipfs config Addresses.Gateway
# Should return: /ip4/127.0.0.1/tcp/8081

# Test IPFS gateway
curl -I http://127.0.0.1:8081/ipfs/<some-CID>
# Should return: HTTP/1.1 200 OK

# Verify code has correct port
grep "8081" server/src/ipfs/client.rs
# Should find: "http://127.0.0.1:8081/ipfs"
```

### Port Reference Table

| Service | Port | Purpose | Check Command |
|---------|------|---------|---------------|
| Marketplace | 8080 | Main HTTP server | `lsof -i :8080 \| grep server` |
| IPFS Gateway | 8081 | HTTP gateway for files | `lsof -i :8081 \| grep ipfs` |
| IPFS API | 5001 | RPC API | `lsof -i :5001 \| grep ipfs` |
| Tor SOCKS5 | 9050 | Proxy | `lsof -i :9050 \| grep tor` |

### Troubleshooting Listing Images

**Symptom:** "Failed to load active listings" (HTTP 500)
**Cause:** Wrong DATABASE_URL format or missing database
**Fix:**
```bash
# 1. Check DATABASE_URL in .env (must be absolute path)
# 2. Verify database exists: ls -lh marketplace.db
# 3. Recreate if needed: cargo run --release --bin init_db
```

**Symptom:** Images return HTTP 404
**Cause:** Wrong IPFS gateway port in code
**Fix:**
```bash
# 1. Edit server/src/ipfs/client.rs line 100
# 2. Change 8080 to 8081
# 3. Recompile: cargo build --release --package server
# 4. Restart server
```

**Symptom:** Multiple servers running, conflicting behavior
**Fix:**
```bash
# Kill all servers
killall -9 server
pkill -9 -f "target/release/server"

# Verify
ps aux | grep "[t]arget/release/server"
# Should return nothing
```

**Full debugging guide:** [DOX/debugging/LISTING-IMAGES-FIX-2025-10-28.md](../debugging/LISTING-IMAGES-FIX-2025-10-28.md)

**Quick checklist:** [DOX/guides/QUICK-FIX-CHECKLIST.md](../guides/QUICK-FIX-CHECKLIST.md)

---

## Testing Checklist for Listing Images

Before any deployment or major code change:

```bash
# ✅ 1. Kill all servers
killall -9 server; sleep 2

# ✅ 2. Verify IPFS daemon running
ps aux | grep "[i]pfs daemon" || ipfs daemon &

# ✅ 3. Check configuration
cat .env | grep DATABASE_URL  # Must be absolute path
grep "8081" server/src/ipfs/client.rs  # Must use port 8081

# ✅ 4. Check database
ls -lh marketplace.db  # Should be 188K+
cd server && DATABASE_URL=../marketplace.db diesel migration list
# All should show [X] applied

# ✅ 5. Recompile if code changed
cargo build --release --package server

# ✅ 6. Start server
env -u DATABASE_URL ./target/release/server > server.log 2>&1 &
sleep 5

# ✅ 7. Test endpoints
curl http://127.0.0.1:8080/api/health
curl http://127.0.0.1:8080/api/listings | jq

# ✅ 8. Create test listing with image
# Navigate to: http://127.0.0.1:8080/listings/create
# Upload an image and create listing

# ✅ 9. Verify image displays
# Navigate to the created listing page
# Images should display without 404 errors
```

---

## Integration with Existing Workflows

### Pre-commit Validation
Add to `.git/hooks/pre-commit` or `scripts/pre-commit.sh`:

```bash
# Verify DATABASE_URL format
if grep -q "^DATABASE_URL=sqlite:" .env; then
    echo "❌ ERROR: DATABASE_URL must use absolute path, not 'sqlite:' prefix"
    echo "Fix: Use DATABASE_URL=/absolute/path/to/marketplace.db"
    exit 1
fi

# Verify IPFS gateway port
if grep -q "8080/ipfs" server/src/ipfs/client.rs; then
    echo "❌ ERROR: IPFS gateway must use port 8081, not 8080"
    echo "Fix: Change to http://127.0.0.1:8081/ipfs in server/src/ipfs/client.rs"
    exit 1
fi
```

### Startup Script
Update `scripts/start-server.sh`:

```bash
#!/bin/bash
set -e

echo "🚀 Starting Monero Marketplace Server"

# Kill old servers
killall -9 server 2>/dev/null || true
pkill -9 -f "target/release/server" 2>/dev/null || true

# Verify IPFS
if ! ps aux | grep -q "[i]pfs daemon"; then
    echo "❌ IPFS daemon not running"
    echo "Start with: ipfs daemon &"
    exit 1
fi

# Verify database
if [ ! -f marketplace.db ]; then
    echo "❌ Database not found: marketplace.db"
    echo "Create with: cargo run --release --bin init_db"
    exit 1
fi

DB_SIZE=$(stat -c%s marketplace.db)
if [ "$DB_SIZE" -lt 100000 ]; then
    echo "⚠️  Database seems empty (${DB_SIZE} bytes)"
fi

# Verify configuration
if grep -q "^DATABASE_URL=sqlite:" .env; then
    echo "❌ DATABASE_URL has wrong format in .env"
    echo "Must be absolute path: /home/user/path/marketplace.db"
    exit 1
fi

# Start server
echo "✅ Starting server..."
env -u DATABASE_URL ./target/release/server > server.log 2>&1 &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Wait and verify
sleep 5
if ! ps -p $SERVER_PID > /dev/null; then
    echo "❌ Server failed to start"
    echo "Check logs: tail -50 server.log"
    exit 1
fi

# Health check
if curl -s http://127.0.0.1:8080/api/health | grep -q "ok"; then
    echo "✅ Server is healthy"
    echo "Access at: http://127.0.0.1:8080"
else
    echo "❌ Server health check failed"
    exit 1
fi
```

---

## Related Documentation

- [DOX/debugging/LISTING-IMAGES-FIX-2025-10-28.md](../debugging/LISTING-IMAGES-FIX-2025-10-28.md) - Full debugging report
- [DOX/guides/QUICK-FIX-CHECKLIST.md](../guides/QUICK-FIX-CHECKLIST.md) - Quick reference
- [CLAUDE.md](../../CLAUDE.md) - Main project guide (update this section)
- [server/README.md](../../server/README.md) - Server-specific docs

---

**Status:** Ready for integration into CLAUDE.md
**Priority:** HIGH - Prevents critical production issues
**Author:** Claude debugging session 2025-10-28
