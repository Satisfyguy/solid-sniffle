# IPFS Setup Guide

This guide explains how to install and configure IPFS for the Monero Marketplace reputation system.

## Overview

The reputation system uses IPFS (InterPlanetary File System) to store vendor reputation files in a decentralized manner. This provides:
- Censorship resistance
- Data persistence
- Decentralized access
- Content-addressed storage

## Installation Options

### Option 1: Install IPFS Desktop (Recommended for Local Testing)

1. **Download IPFS Desktop**:
   ```bash
   # For Ubuntu/Debian
   wget https://github.com/ipfs/ipfs-desktop/releases/download/v0.35.0/ipfs-desktop-0.35.0-linux-x86_64.AppImage
   chmod +x ipfs-desktop-0.35.0-linux-x86_64.AppImage
   ./ipfs-desktop-0.35.0-linux-x86_64.AppImage
   ```

2. **Or use package manager**:
   ```bash
   # Ubuntu/Debian
   sudo snap install ipfs-desktop
   ```

### Option 2: Install IPFS CLI (Kubo)

1. **Download and install**:
   ```bash
   wget https://dist.ipfs.tech/kubo/v0.25.0/kubo_v0.25.0_linux-amd64.tar.gz
   tar -xvzf kubo_v0.25.0_linux-amd64.tar.gz
   cd kubo
   sudo bash install.sh
   ```

2. **Initialize IPFS**:
   ```bash
   ipfs init
   ```

3. **Start the daemon**:
   ```bash
   ipfs daemon
   ```

### Option 3: Docker (Easiest)

```bash
docker run -d \
  --name ipfs_host \
  -v /path/to/ipfs/export:/export \
  -v /path/to/ipfs/data:/data/ipfs \
  -p 4001:4001 \
  -p 4001:4001/udp \
  -p 127.0.0.1:5001:5001 \
  -p 127.0.0.1:8080:8080 \
  ipfs/kubo:latest
```

## Configuration

### Default Configuration (Local Testing)

For local testing without Tor, the default configuration works:

```bash
# IPFS API endpoint
IPFS_API_URL=http://127.0.0.1:5001

# Gateway endpoint
IPFS_GATEWAY_URL=http://127.0.0.1:8080

# Tor disabled for local testing
IPFS_USE_TOR=false
```

### Production Configuration (with Tor)

For production deployment, IPFS should route through Tor:

```bash
# IPFS API endpoint (localhost only)
IPFS_API_URL=http://127.0.0.1:5001

# Gateway endpoint (over Tor)
IPFS_GATEWAY_URL=http://127.0.0.1:8080

# Enable Tor routing
IPFS_USE_TOR=true

# Tor SOCKS proxy
TOR_SOCKS_PROXY=socks5h://127.0.0.1:9050
```

### Configure IPFS to Use Tor

1. **Edit IPFS config**:
   ```bash
   ipfs config --json Swarm.Addrs.NoAnnounce '["/ip4/127.0.0.1", "/ip4/0.0.0.0"]'
   ```

2. **Configure SOCKS proxy** (add to `~/.ipfs/config`):
   ```json
   {
     "Swarm": {
       "AddrFilters": null,
       "ConnMgr": {},
       "Transports": {
         "Network": {
           "QUIC": false
         }
       }
     },
     "Routing": {
       "Type": "dhtclient"
     }
   }
   ```

3. **Use Tor as SOCKS proxy**:
   ```bash
   export ALL_PROXY=socks5h://127.0.0.1:9050
   ipfs daemon
   ```

## Verification

### Test IPFS is Running

```bash
# Check daemon is running
ipfs swarm peers

# Test API endpoint
curl http://127.0.0.1:5001/api/v0/version

# Expected output:
# {"Version":"0.25.0","Commit":"","Repo":"15","System":"amd64/linux","Golang":"go1.21.5"}
```

### Test IPFS Upload/Download

```bash
# Upload a test file
echo "Hello IPFS!" > test.txt
ipfs add test.txt

# Output: QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

# Download the file
ipfs cat QmXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

# Expected: Hello IPFS!
```

### Test Tor Integration (if enabled)

```bash
# Verify IPFS traffic goes through Tor
sudo tcpdump -i any port 4001 -c 10

# Should show no direct peer connections (only localhost)

# Check Tor SOCKS connections
netstat -an | grep 9050
```

## Integration with Reputation System

### Environment Variables

Create a `.env` file in `reputation/` directory:

```bash
# For local testing (no Tor)
IPFS_API_URL=http://127.0.0.1:5001
IPFS_GATEWAY_URL=http://127.0.0.1:8080
IPFS_USE_TOR=false

# For production (with Tor)
# IPFS_API_URL=http://127.0.0.1:5001
# IPFS_GATEWAY_URL=http://127.0.0.1:8080
# IPFS_USE_TOR=true
# TOR_SOCKS_PROXY=socks5h://127.0.0.1:9050
```

### Test Reputation Upload

```bash
# Run reputation system with IPFS
cd reputation
cargo test --test integration reputation_flow_test -- --nocapture

# Expected:
# ✅ Reputation file uploaded to IPFS: QmXXXXXX
# ✅ Retrieved via IPFS gateway successfully
```

## Troubleshooting

### IPFS Daemon Not Starting

```bash
# Check if port 5001 is already in use
lsof -i :5001

# Kill existing process if needed
kill -9 <PID>

# Restart daemon
ipfs daemon
```

### API Connection Refused

```bash
# Check IPFS config
ipfs config Addresses.API

# Should be: /ip4/127.0.0.1/tcp/5001

# If not, set it:
ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001
```

### Tor Connection Issues

```bash
# Verify Tor is running
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org

# Check IPFS is using proxy
ps aux | grep ipfs | grep -i proxy

# Restart IPFS with Tor
pkill ipfs
export ALL_PROXY=socks5h://127.0.0.1:9050
ipfs daemon
```

### Content Not Accessible

```bash
# Pin content to ensure persistence
ipfs pin add QmXXXXXXX

# Check pinned items
ipfs pin ls

# Re-publish to DHT
ipfs dht provide QmXXXXXXX
```

## Security Considerations

### Local Testing
- IPFS API should only bind to `127.0.0.1` (never `0.0.0.0`)
- Gateway can be public for read-only access
- No Tor required for local testing

### Production Deployment
- **MUST use Tor** for all IPFS traffic
- API endpoint must be `127.0.0.1` only
- Gateway access must be Tor-only
- No direct IP exposure
- All reputation files are public (by design)

### Privacy Notes
- Reputation files are **intentionally public**
- All reviews are cryptographically signed (non-repudiable)
- Buyer pseudonyms (public keys) are visible
- No transaction amounts or addresses are stored
- Content is immutable (content-addressed)

## Performance Tuning

### Faster DHT Lookups

```bash
# Increase connection limits
ipfs config --json Swarm.ConnMgr.HighWater 900
ipfs config --json Swarm.ConnMgr.LowWater 600

# Enable faster routing
ipfs config Routing.Type dhtclient
```

### Reduce Storage Usage

```bash
# Set garbage collection policy
ipfs config Datastore.GCPeriod "1h"

# Limit repo size
ipfs config --json Datastore.StorageMax '"10GB"'
```

## Monitoring

### Check IPFS Stats

```bash
# Bandwidth stats
ipfs stats bw

# Repository stats
ipfs repo stat

# Peer count
ipfs swarm peers | wc -l
```

### Health Check Script

Create `scripts/check-ipfs-health.sh`:

```bash
#!/bin/bash
set -e

echo "Checking IPFS health..."

# Check daemon is running
if ! ipfs swarm peers &>/dev/null; then
    echo "❌ IPFS daemon is not running"
    exit 1
fi

# Check API is accessible
if ! curl -s http://127.0.0.1:5001/api/v0/version &>/dev/null; then
    echo "❌ IPFS API is not accessible"
    exit 1
fi

# Check peer count
PEERS=$(ipfs swarm peers | wc -l)
if [ "$PEERS" -lt 5 ]; then
    echo "⚠️  Low peer count: $PEERS"
else
    echo "✅ Connected to $PEERS peers"
fi

# Check repo size
REPO_SIZE=$(ipfs repo stat -H | grep "RepoSize" | awk '{print $2}')
echo "✅ Repository size: $REPO_SIZE"

# Check pinned items
PINNED=$(ipfs pin ls --type=recursive | wc -l)
echo "✅ Pinned items: $PINNED"

echo "✅ IPFS is healthy"
```

## References

- [IPFS Documentation](https://docs.ipfs.tech/)
- [Kubo (go-ipfs) GitHub](https://github.com/ipfs/kubo)
- [IPFS over Tor Guide](https://docs.ipfs.tech/how-to/privacy-best-practices/)
- [Content Addressing Spec](https://docs.ipfs.tech/concepts/content-addressing/)
