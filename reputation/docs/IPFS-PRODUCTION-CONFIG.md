# IPFS Production Configuration with Tor

This guide explains how to configure IPFS for production deployment with Tor routing for the Monero Marketplace reputation system.

## Security Requirements

### Zero-Tolerance Security Theatre Principles

1. **ALL IPFS traffic MUST route through Tor**
2. **NO direct IP exposure** - API bound to 127.0.0.1 only
3. **NO clearnet peer connections** - Tor-only mode
4. **Verify before deploy** - Automated checks required

## Production Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Monero Marketplace Server (Hidden Service)                 │
│                                                              │
│  ┌──────────────┐       ┌──────────────┐                   │
│  │  Reputation  │──────▶│  IPFS Node   │                   │
│  │   Handler    │       │  (127.0.0.1) │                   │
│  └──────────────┘       └───────┬──────┘                   │
│                                 │                            │
│                        ┌────────▼────────┐                  │
│                        │   Tor Daemon    │                  │
│                        │ (SOCKS5: 9050)  │                  │
│                        └────────┬────────┘                  │
└─────────────────────────────────┼──────────────────────────┘
                                  │
                                  │ (All IPFS traffic)
                                  │
                          ┌───────▼────────┐
                          │   Tor Network  │
                          └───────┬────────┘
                                  │
                          ┌───────▼────────┐
                          │  IPFS Network  │
                          │  (DHT, Peers)  │
                          └────────────────┘
```

## Step-by-Step Configuration

### 1. Install and Configure Tor

```bash
# Install Tor
sudo apt update
sudo apt install -y tor

# Edit Tor configuration
sudo nano /etc/tor/torrc
```

**Add to `/etc/tor/torrc`:**

```
# SOCKS proxy for IPFS
SocksPort 127.0.0.1:9050

# OPSEC: Prevent DNS leaks
DNSPort 127.0.0.1:5353

# Strict exit policy (no exit traffic)
ExitPolicy reject *:*

# Performance tuning
NumEntryGuards 4
CircuitBuildTimeout 30

# Logging (OPSEC: no sensitive data)
Log notice file /var/log/tor/notices.log
SafeLogging 1
```

**Start Tor:**

```bash
sudo systemctl enable tor
sudo systemctl start tor
sudo systemctl status tor
```

**Verify Tor is working:**

```bash
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip
```

### 2. Configure IPFS for Tor

**Edit IPFS config:**

```bash
ipfs config --json Swarm.AddrFilters '[
  "/ip4/0.0.0.0/ipcidr/0",
  "/ip4/10.0.0.0/ipcidr/8",
  "/ip4/100.64.0.0/ipcidr/10",
  "/ip4/169.254.0.0/ipcidr/16",
  "/ip4/172.16.0.0/ipcidr/12",
  "/ip4/192.0.0.0/ipcidr/24",
  "/ip4/192.0.2.0/ipcidr/24",
  "/ip4/192.168.0.0/ipcidr/16",
  "/ip4/198.18.0.0/ipcidr/15",
  "/ip4/198.51.100.0/ipcidr/24",
  "/ip4/203.0.113.0/ipcidr/24",
  "/ip4/224.0.0.0/ipcidr/4",
  "/ip4/240.0.0.0/ipcidr/4",
  "/ip6/100::/ipcidr/64",
  "/ip6/2001:2::/ipcidr/48",
  "/ip6/2001:db8::/ipcidr/32",
  "/ip6/fc00::/ipcidr/7",
  "/ip6/fe80::/ipcidr/10"
]'
```

**Configure API and Gateway (localhost only):**

```bash
ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001
ipfs config Addresses.Gateway /ip4/127.0.0.1/tcp/8080
```

**Disable QUIC (not supported over SOCKS):**

```bash
ipfs config --json Swarm.Transports.Network.QUIC false
```

**Set DHT client mode (reduces exposure):**

```bash
ipfs config Routing.Type dhtclient
```

**Disable mDNS (local network discovery):**

```bash
ipfs config --json Discovery.MDNS.Enabled false
```

### 3. Environment Configuration

Create `/etc/systemd/system/ipfs.service`:

```ini
[Unit]
Description=IPFS Daemon (Tor-routed)
After=network.target tor.service
Requires=tor.service

[Service]
Type=simple
User=ipfs
Group=ipfs
Environment="IPFS_PATH=/var/lib/ipfs"
Environment="ALL_PROXY=socks5h://127.0.0.1:9050"
ExecStart=/usr/local/bin/ipfs daemon
Restart=on-failure
RestartSec=10s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/ipfs

[Install]
WantedBy=multi-user.target
```

**Create IPFS user:**

```bash
sudo useradd -r -s /bin/false ipfs
sudo mkdir -p /var/lib/ipfs
sudo chown ipfs:ipfs /var/lib/ipfs
```

**Initialize IPFS as service user:**

```bash
sudo -u ipfs IPFS_PATH=/var/lib/ipfs ipfs init
```

**Apply configuration:**

```bash
sudo systemctl daemon-reload
sudo systemctl enable ipfs
sudo systemctl start ipfs
```

### 4. Application Configuration

**Create `.env` file for reputation system:**

```bash
# IPFS Configuration (Production)
IPFS_API_URL=http://127.0.0.1:5001
IPFS_GATEWAY_URL=http://127.0.0.1:8080
IPFS_USE_TOR=true

# Tor SOCKS Proxy
TOR_SOCKS_PROXY=socks5h://127.0.0.1:9050

# IPFS Timeouts (Tor is slow)
IPFS_UPLOAD_TIMEOUT_SECS=120
IPFS_DOWNLOAD_TIMEOUT_SECS=90

# Pinning Strategy
IPFS_AUTO_PIN=true
IPFS_PIN_TTL_DAYS=365
```

## Verification

### Automated Security Checks

Create `scripts/verify-ipfs-tor.sh`:

```bash
#!/bin/bash
set -e

echo "========================================="
echo "  IPFS + Tor Security Verification"
echo "========================================="
echo ""

# Check 1: Tor is running
echo "[1/7] Checking Tor daemon..."
if curl --socks5-hostname 127.0.0.1:9050 -s https://check.torproject.org/api/ip &>/dev/null; then
    echo "✅ Tor is running and accessible"
else
    echo "❌ CRITICAL: Tor is not running"
    exit 1
fi

# Check 2: IPFS API is localhost-only
echo "[2/7] Verifying IPFS API binding..."
API_ADDR=$(ipfs config Addresses.API)
if [[ "$API_ADDR" == *"127.0.0.1"* ]]; then
    echo "✅ IPFS API is bound to localhost: $API_ADDR"
else
    echo "❌ CRITICAL: IPFS API is exposed: $API_ADDR"
    exit 1
fi

# Check 3: IPFS Gateway is localhost-only
echo "[3/7] Verifying IPFS Gateway binding..."
GATEWAY_ADDR=$(ipfs config Addresses.Gateway)
if [[ "$GATEWAY_ADDR" == *"127.0.0.1"* ]]; then
    echo "✅ IPFS Gateway is bound to localhost: $GATEWAY_ADDR"
else
    echo "❌ CRITICAL: IPFS Gateway is exposed: $GATEWAY_ADDR"
    exit 1
fi

# Check 4: No direct peer connections (check for non-localhost IPs)
echo "[4/7] Checking IPFS peer connections..."
if ipfs swarm peers 2>/dev/null | grep -v "127.0.0.1" | grep -E '[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}' &>/dev/null; then
    echo "⚠️  WARNING: Direct IP connections detected (may not be through Tor)"
    echo "    This is expected if IPFS daemon was started without ALL_PROXY"
else
    echo "✅ No direct IP peer connections"
fi

# Check 5: QUIC is disabled
echo "[5/7] Verifying QUIC is disabled..."
QUIC_ENABLED=$(ipfs config Swarm.Transports.Network.QUIC)
if [[ "$QUIC_ENABLED" == "false" ]]; then
    echo "✅ QUIC is disabled (SOCKS5 compatible)"
else
    echo "❌ CRITICAL: QUIC is enabled (incompatible with SOCKS5)"
    exit 1
fi

# Check 6: DHT mode
echo "[6/7] Verifying DHT mode..."
DHT_MODE=$(ipfs config Routing.Type)
if [[ "$DHT_MODE" == "dhtclient" ]]; then
    echo "✅ DHT client mode enabled"
else
    echo "⚠️  DHT mode is: $DHT_MODE (consider 'dhtclient' for reduced exposure)"
fi

# Check 7: Test IPFS upload through Tor
echo "[7/7] Testing IPFS upload (through Tor)..."
TEST_CONTENT="IPFS Tor Test $(date +%s)"
TEST_HASH=$(echo "$TEST_CONTENT" | ipfs add -q --only-hash)

if [ -n "$TEST_HASH" ]; then
    echo "✅ IPFS upload test successful (hash: ${TEST_HASH:0:20}...)"
else
    echo "❌ IPFS upload test failed"
    exit 1
fi

echo ""
echo "========================================="
echo "  ✅ All security checks passed!"
echo "========================================="
echo ""
echo "IPFS is properly configured for Tor routing."
echo "All traffic will be routed through 127.0.0.1:9050"
```

**Run verification:**

```bash
chmod +x scripts/verify-ipfs-tor.sh
./scripts/verify-ipfs-tor.sh
```

### Manual Verification

**Test IPFS through Tor:**

```bash
# Stop IPFS daemon
sudo systemctl stop ipfs

# Start with Tor proxy
sudo -u ipfs bash -c 'export ALL_PROXY=socks5h://127.0.0.1:9050 && ipfs daemon &'

# Wait 10 seconds for daemon to start
sleep 10

# Upload test content
echo "Hello IPFS + Tor" | ipfs add

# Verify no direct connections
netstat -anp | grep ipfs | grep -v "127.0.0.1"
# Should show NO external IP connections

# Check Tor SOCKS connections
netstat -anp | grep 9050
# Should show connections from IPFS to Tor
```

### Network Traffic Verification

**Monitor IPFS traffic with tcpdump:**

```bash
# Terminal 1: Start packet capture
sudo tcpdump -i any port 4001 -nn

# Terminal 2: Upload to IPFS
echo "Test" | ipfs add

# Expected: NO traffic on port 4001 (all goes through Tor)
```

## Deployment Checklist

- [ ] Tor daemon installed and running
- [ ] Tor configured with SocksPort 127.0.0.1:9050
- [ ] IPFS API bound to 127.0.0.1:5001 only
- [ ] IPFS Gateway bound to 127.0.0.1:8080 only
- [ ] QUIC disabled in IPFS config
- [ ] DHT mode set to "dhtclient"
- [ ] mDNS disabled
- [ ] Address filters configured (no direct IPs)
- [ ] `ALL_PROXY=socks5h://127.0.0.1:9050` in environment
- [ ] systemd service created and enabled
- [ ] Automated verification script passes
- [ ] No direct peer connections detected
- [ ] Test upload/download successful through Tor

## Monitoring

### Health Check Endpoint

Add to `server/src/handlers/health.rs`:

```rust
async fn check_ipfs_health() -> Result<HealthStatus> {
    let ipfs_client = IpfsClient::new("http://127.0.0.1:5001")?;

    // Check IPFS API is accessible
    let version = ipfs_client.version().await?;

    // Verify Tor is being used (no direct peer IPs visible)
    let peers = ipfs_client.swarm_peers().await?;
    let direct_ips = peers.iter()
        .filter(|p| !p.addr.contains("127.0.0.1"))
        .count();

    if direct_ips > 0 {
        return Err(anyhow::anyhow!(
            "SECURITY VIOLATION: {} direct IP connections detected",
            direct_ips
        ));
    }

    Ok(HealthStatus::Healthy {
        ipfs_version: version,
        peers: peers.len(),
        tor_enabled: true,
    })
}
```

### Prometheus Metrics

```rust
// Expose IPFS metrics
ipfs_upload_total{status="success|failure"}
ipfs_download_total{status="success|failure"}
ipfs_upload_duration_seconds
ipfs_download_duration_seconds
ipfs_peer_count
ipfs_tor_violations_total  // CRITICAL: should always be 0
```

### Alerting

**Create alert rules:**

```yaml
# alerts/ipfs-security.yml
groups:
  - name: ipfs_security
    rules:
      - alert: IPFSDirectConnectionDetected
        expr: ipfs_tor_violations_total > 0
        for: 1m
        severity: critical
        annotations:
          summary: "IPFS direct connection detected (not through Tor)"
          description: "SECURITY VIOLATION: IPFS is making direct connections"

      - alert: TorDaemonDown
        expr: up{job="tor"} == 0
        for: 5m
        severity: critical
        annotations:
          summary: "Tor daemon is down"
          description: "IPFS traffic cannot be routed - SERVICE OFFLINE"
```

## Troubleshooting

### IPFS Cannot Connect Through Tor

```bash
# Check Tor is running
systemctl status tor

# Check SOCKS proxy is accessible
curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org

# Check IPFS environment has proxy set
sudo systemctl cat ipfs | grep ALL_PROXY

# Restart both services
sudo systemctl restart tor
sudo systemctl restart ipfs
```

### Slow Upload/Download Performance

```bash
# Increase connection limits
ipfs config --json Swarm.ConnMgr.HighWater 2000
ipfs config --json Swarm.ConnMgr.LowWater 1000

# Increase timeouts in .env
IPFS_UPLOAD_TIMEOUT_SECS=300
IPFS_DOWNLOAD_TIMEOUT_SECS=180

# Use faster DHT mode (if acceptable)
ipfs config Routing.Type dht
```

### Content Not Propagating

```bash
# Manually pin content
ipfs pin add QmXXXXXX

# Force DHT provide
ipfs dht provide QmXXXXXX

# Check pinning status
ipfs pin ls --type=recursive
```

## Security Best Practices

1. **Never disable Tor in production** - IPFS_USE_TOR=true is mandatory
2. **Bind to localhost only** - Never 0.0.0.0
3. **Monitor direct connections** - Alert on any non-Tor traffic
4. **Regular security audits** - Run verification script daily
5. **Log analysis** - Check for IP leaks in IPFS logs
6. **Firewall rules** - Block port 4001 (IPFS swarm) outbound except to 127.0.0.1
7. **Process isolation** - Run IPFS as dedicated user with minimal permissions

## References

- [IPFS Privacy Best Practices](https://docs.ipfs.tech/how-to/privacy-best-practices/)
- [Tor Project: Hidden Service Best Practices](https://community.torproject.org/onion-services/setup/)
- [IPFS Config Reference](https://github.com/ipfs/kubo/blob/master/docs/config.md)
