# Quick Fix Checklist - Listing Images

## Before Starting Server

```bash
# 1. Kill all old servers
killall -9 server
pkill -9 -f "target/release/server"

# 2. Verify IPFS daemon running
ps aux | grep "[i]pfs daemon" || ipfs daemon &

# 3. Check database exists
ls -lh marketplace.db  # Should be 188K+

# 4. Check .env configuration
cat .env | grep DATABASE_URL
# Should be: DATABASE_URL=/home/malix/Desktop/monero.marketplace/marketplace.db
# NOT: DATABASE_URL=sqlite:marketplace.db
```

## Start Server

```bash
# Always unset DATABASE_URL env var before starting
env -u DATABASE_URL ./target/release/server > server.log 2>&1 &

# Wait 5 seconds
sleep 5

# Verify server started
curl http://127.0.0.1:8080/api/health
# Expected: {"status":"ok"}
```

## Quick Test

```bash
# Test listings API
curl http://127.0.0.1:8080/api/listings | jq length

# Test image endpoint (replace with real IDs)
curl -I http://127.0.0.1:8080/api/listings/<LISTING_ID>/images/<CID>
# Expected: HTTP/1.1 200 OK
```

## Common Fixes

### Problem: "Failed to load active listings"
**Fix:**
```bash
# Check database path
cat .env | grep DATABASE_URL
# Must be absolute path: /home/malix/Desktop/monero.marketplace/marketplace.db
```

### Problem: Images return 404
**Fix:**
```bash
# Check IPFS gateway port in code
grep "8081" server/src/ipfs/client.rs
# Should find: "http://127.0.0.1:8081/ipfs"

# If not, edit and recompile:
cargo build --release --package server
```

### Problem: Multiple servers running
**Fix:**
```bash
ps aux | grep "[t]arget/release/server" | awk '{print $2}' | xargs kill -9
```

## Port Reference

- **8080** - Marketplace Server
- **8081** - IPFS Gateway (HTTP)
- **5001** - IPFS API
- **9050** - Tor SOCKS5 Proxy

## Full Details

See: [DOX/debugging/LISTING-IMAGES-FIX-2025-10-28.md](../debugging/LISTING-IMAGES-FIX-2025-10-28.md)
