# TM-004: Weak Network Hardening - Insufficient Tor Enforcement

**Severity:** 🟠 HIGH (not CRITICAL because some protections exist)
**CVSS Score:** 7.5 (High)
**CVSS Vector:** CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:N/A:N
**Date Identified:** 2025-10-26
**Status:** ⚠️ VULNERABLE (requires hardening)
**Threat Model:** State Actor + Sophisticated Hacker

---

## Executive Summary

The marketplace has **partial** Tor enforcement with multiple bypasses and weak validation. While some network hardening exists (localhost-only Monero RPC, IPFS conditional Tor), the implementation has gaps that could leak real IP addresses to adversaries.

**Current Protections (Partial):**
1. ✅ Monero RPC localhost validation (lines 43-47 of wallet/src/rpc.rs)
2. ✅ IPFS Tor proxy for remote gateways (lines 127-128 of server/src/ipfs/client.rs)
3. ✅ Server binds to 127.0.0.1 only (line 390 of server/src/main.rs)

**Vulnerabilities:**
1. ❌ RPC localhost validation uses weak `contains()` check (bypassable)
2. ❌ IPFS Tor is **disabled** for localhost connections (IP leak if misconfigured)
3. ❌ No validation that external HTTP clients use Tor proxy
4. ❌ No runtime check that Tor daemon is actually running
5. ❌ No prevention of accidental clearnet DNS leaks

**Attack Impact:**
- **Deanonymization:** Real IP exposed to network adversary
- **Location Disclosure:** Physical location revealed via GeoIP
- **Traffic Analysis:** Marketplace usage patterns observable

---

## Vulnerability Details

### Vulnerability 1: Weak Monero RPC Localhost Validation

**File:** [`wallet/src/rpc.rs:43-47`](../../wallet/src/rpc.rs#L43-L47)

```rust
// OPSEC: Vérifier que URL est localhost
if !url.contains("127.0.0.1") && !url.contains("localhost") {
    return Err(MoneroError::InvalidResponse(
        "RPC URL must be localhost only (OPSEC)".to_string(),
    ));
}
```

**Problem:** `contains()` is substring match, not IP validation

**Bypass Examples:**

```rust
// ✅ Currently ACCEPTED (but should be REJECTED):
"http://malicious-127.0.0.1.attacker.com:18082"  // Contains "127.0.0.1"
"http://localhost.attacker.com:18082"            // Contains "localhost"
"http://evil.com/proxy?url=127.0.0.1:18082"      // Contains "127.0.0.1"

// ❌ Correctly REJECTED:
"http://192.168.1.10:18082"  // No "127.0.0.1" or "localhost"
"http://0.0.0.0:18082"       // No "127.0.0.1" or "localhost"
```

**Impact:**
- Attacker tricks server into connecting to malicious RPC endpoint
- Malicious endpoint logs real IP address of marketplace server
- Attacker gains deanonymization data

---

### Vulnerability 2: IPFS Tor Bypass for Local Connections

**File:** [`server/src/ipfs/client.rs:74-81`](../../server/src/ipfs/client.rs#L74-L81)

```rust
// SECURITY: Route all IPFS traffic through Tor in production
// For development, set IPFS_USE_TOR=false to disable Tor proxy
if api_url.starts_with("http://127.0.0.1") || api_url.starts_with("http://localhost") {
    tracing::info!("IPFS: Connecting directly to local IPFS node (Tor proxy bypassed for local connection)");
} else {
    tracing::info!("IPFS: Configuring Tor SOCKS5 proxy (127.0.0.1:9050)");
    let proxy = Proxy::all("socks5://127.0.0.1:9050")
        .context("Failed to configure Tor SOCKS5 proxy for IPFS")?;
    client_builder = client_builder.proxy(proxy);
}
```

**Problem:** Assumes localhost = safe, but IPFS daemon might be misconfigured

**Attack Scenario:**

1. **Admin Misconfigures IPFS Daemon:**
   ```bash
   # IPFS config accidentally exposes API publicly
   ipfs config Addresses.API /ip4/0.0.0.0/tcp/5001
   ```

2. **Marketplace Connects "Locally":**
   ```rust
   // Server code
   IpfsClient::new("http://127.0.0.1:5001/api/v0")  // Bypasses Tor
   ```

3. **IPFS Daemon Forwards to Public Gateway:**
   - IPFS daemon running on VPS with public IP
   - Daemon configured to use clearnet gateway for missing content
   - Upload/download requests leak server's real IP to IPFS network

**Impact:**
- Real IP disclosed to IPFS network swarm
- DHT queries observable by network adversaries
- Content CIDs linkable to specific marketplace instance

---

### Vulnerability 3: No Tor Daemon Runtime Validation

**Current Implementation:** Code **assumes** Tor daemon is running at 127.0.0.1:9050

**Missing Check:**
```rust
// ❌ NO validation like this exists:
async fn check_tor_available() -> Result<()> {
    let proxy = Proxy::all("socks5://127.0.0.1:9050")?;
    let client = Client::builder().proxy(proxy).build()?;

    // Test Tor connectivity
    let response = client
        .get("https://check.torproject.org/api/ip")
        .send()
        .await?;

    let ip_check: TorCheckResponse = response.json().await?;

    if !ip_check.IsTor {
        anyhow::bail!("Tor is NOT active - traffic will leak to clearnet!");
    }

    Ok(())
}
```

**Attack Scenario:**

1. **Tor Daemon Crashes/Stops:**
   ```bash
   sudo systemctl stop tor
   ```

2. **Server Continues Running:**
   - No health check detects Tor is down
   - Subsequent HTTP requests **fail** (good)
   - BUT: Error messages may leak in logs

3. **Fallback to Clearnet (Worst Case):**
   - If reqwest has fallback logic → clearnet connection
   - Real IP exposed to external services

**Impact:**
- Silent deanonymization if Tor fails
- No alerts/monitoring for Tor downtime
- Operational risk (marketplace offline until admin notices)

---

### Vulnerability 4: DNS Leak Potential

**Current Implementation:** Uses `socks5://` proxy

**Problem:** Should use `socks5h://` to prevent DNS leaks

**Vulnerable Code Pattern:**
```rust
// ⚠️ VULNERABLE (DNS resolved on client side)
Proxy::all("socks5://127.0.0.1:9050")

// ✅ CORRECT (DNS resolved through Tor)
Proxy::all("socks5h://127.0.0.1:9050")
```

**Difference:**
- `socks5://` → DNS query happens **before** Tor (clearnet DNS leak)
- `socks5h://` → DNS query happens **through** Tor (no leak)

**Attack Impact:**
- ISP/network observer sees DNS queries for .onion addresses
- Correlation: "This IP is accessing dark web marketplaces"
- Deanonymization even if HTTP traffic is through Tor

**Current Status in Codebase:**
```bash
# Correct usage (server/src/ipfs/client.rs:127):
Proxy::all("socks5h://127.0.0.1:9050")  # ✅ GOOD (with 'h')

# Vulnerable usage (server/src/ipfs/client.rs:78):
Proxy::all("socks5://127.0.0.1:9050")   # ⚠️ VULNERABLE (no 'h')
```

**Inconsistency:** Some code uses `socks5h://`, some uses `socks5://`

---

## Threat Model Analysis

### Attack Scenario 1: Malicious Monero RPC Server

**Adversary:** Sophisticated hacker with web server
**Capability:** Social engineering or config file tampering

**Attack Steps:**

1. **Attacker Registers Lookalike Domain:**
   ```bash
   # Register domain that tricks contains() check
   malicious-127.0.0.1-rpc.onion
   ```

2. **Attacker Modifies .env:**
   ```bash
   # Via RCE, insider threat, or stolen backup
   MONERO_RPC_URL=http://malicious-127.0.0.1-rpc.onion:18082
   ```

3. **Server Accepts Malicious URL:**
   ```rust
   // Validation passes (contains "127.0.0.1")
   MoneroRpcClient::new(config)  // ✅ Passes validation
   ```

4. **Marketplace Connects to Attacker's RPC:**
   - Attacker's server logs real IP: `<MARKETPLACE_IP>`
   - Attacker responds with fake wallet data
   - Escrow transactions fail silently

**Impact:**
- **Deanonymization:** Real IP disclosed
- **DoS:** Escrow system non-functional
- **Financial:** Potential for double-spend if RPC lies

---

### Attack Scenario 2: IPFS Gateway IP Leak

**Adversary:** State actor monitoring IPFS network
**Capability:** DHT crawling, swarm analysis

**Attack Steps:**

1. **Admin Uses Public IPFS Gateway:**
   ```bash
   # .env configuration
   IPFS_API_URL=https://ipfs.infura.io:5001/api/v0
   ```

2. **Marketplace Uploads Reputation File:**
   ```rust
   // Code uses Tor for Infura (GOOD)
   ipfs_client.add(reputation_data, "reputation.json", "application/json")
   ```

3. **State Actor Monitors Infura API Logs:**
   - Infura logs see Tor exit node IP (good)
   - BUT: If Tor fails → real IP logged

4. **Correlation Attack:**
   - State actor identifies all CIDs uploaded from specific Tor circuit
   - Cross-reference with marketplace listing timestamps
   - Deanonymize marketplace operator

**Impact:**
- **Weak anonymity:** Depends on Tor never failing
- **Metadata leakage:** Upload patterns observable
- **Vendor deanonymization:** CIDs linkable to specific sellers

---

### Attack Scenario 3: DNS Leak During .onion Access

**Adversary:** ISP or network-level observer
**Capability:** Passive DNS monitoring

**Attack Steps:**

1. **Marketplace Configured for Tor Hidden Service:**
   ```bash
   # Server runs as .onion hidden service
   HiddenServiceDir /var/lib/tor/marketplace
   HiddenServicePort 80 127.0.0.1:8080
   ```

2. **Code Makes External HTTP Request (Hypothetical):**
   ```rust
   // If marketplace needs to fetch external data
   let client = Client::builder()
       .proxy(Proxy::all("socks5://127.0.0.1:9050"))  // ⚠️ No 'h'
       .build()?;

   client.get("http://external-api.onion/data").send().await?;
   ```

3. **DNS Leak Occurs:**
   ```
   Client → DNS resolver: "What is IP of external-api.onion?"
   DNS resolver → Root servers: "Unknown TLD .onion"
   ISP logs: "User at <IP> queried external-api.onion"
   ```

4. **ISP Flags Activity:**
   - Alert: "Darknet marketplace access detected"
   - Cross-reference with other surveillance data
   - Deanonymization

**Impact:**
- **ISP-level surveillance:** .onion access revealed
- **Timing attacks:** Correlate DNS queries with Tor traffic
- **Legal risk:** Jurisdiction may criminalize marketplace operation

---

## Current Security Posture

### What DOES Work

**✅ Server Localhost Binding:**
```rust
// server/src/main.rs:390
.bind(("127.0.0.1", 8080))
```
- Prevents accidental public exposure
- Forces Tor hidden service usage

**✅ IPFS Tor for Remote Gateways:**
```rust
// server/src/ipfs/client.rs:127-128
let proxy = Proxy::all("socks5h://127.0.0.1:9050")
```
- Correct `socks5h://` usage
- Prevents DNS leaks for Infura

**✅ Monero RPC Basic Validation:**
```rust
// wallet/src/rpc.rs:43-47
if !url.contains("127.0.0.1") && !url.contains("localhost") {
    return Err(...)
}
```
- Catches most accidental misconfigurations
- Better than no validation

### What DOES NOT Work

**❌ Weak RPC URL Validation:**
- `contains()` is substring match, not IP parsing
- Trivially bypassable with lookalike domains

**❌ No Tor Health Checks:**
- Assumes Tor is always running
- No alerts if Tor daemon fails

**❌ Inconsistent `socks5://` vs `socks5h://`:**
- Some code has DNS leak protection
- Some code vulnerable to DNS leaks

**❌ IPFS Localhost Bypass:**
- Assumes local IPFS is safe
- No validation of IPFS daemon configuration

---

## Recommended Solution

### Zero-Budget Solution: Strict Network Validation Module

**File:** `server/src/network/tor_validator.rs` (NEW)

```rust
//! Tor connectivity validation and enforcement
//!
//! Ensures all external network traffic goes through Tor,
//! with runtime health checks and DNS leak prevention.

use anyhow::{Context, Result};
use reqwest::{Client, Proxy};
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

/// Tor SOCKS5 proxy address
const TOR_PROXY: &str = "socks5h://127.0.0.1:9050";

/// Tor check endpoint
const TOR_CHECK_URL: &str = "https://check.torproject.org/api/ip";

/// Response from Tor check API
#[derive(Debug, Deserialize)]
struct TorCheckResponse {
    #[serde(rename = "IsTor")]
    is_tor: bool,
    #[serde(rename = "IP")]
    ip: String,
}

/// Validate that Tor daemon is running and functional
pub async fn validate_tor_running() -> Result<()> {
    tracing::info!("Validating Tor daemon connectivity...");

    let proxy = Proxy::all(TOR_PROXY)
        .context("Failed to configure Tor proxy")?;

    let client = Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client with Tor proxy")?;

    let response = client
        .get(TOR_CHECK_URL)
        .send()
        .await
        .context("Failed to connect to Tor check endpoint - is Tor running?")?;

    let check: TorCheckResponse = response
        .json()
        .await
        .context("Failed to parse Tor check response")?;

    if !check.is_tor {
        anyhow::bail!(
            "Tor validation FAILED - traffic is NOT going through Tor! Exit IP: {}",
            check.ip
        );
    }

    tracing::info!("✅ Tor validation PASSED - Exit IP: {}", check.ip);
    Ok(())
}

/// Validate that URL is strictly localhost (127.0.0.1 or ::1)
///
/// # Arguments
/// * `url` - URL to validate
///
/// # Returns
/// Ok(()) if URL is localhost, Err otherwise
///
/// # Examples
/// ```
/// validate_localhost_url("http://127.0.0.1:18082")?;  // ✅ OK
/// validate_localhost_url("http://localhost:18082")?;   // ✅ OK (resolved to 127.0.0.1)
/// validate_localhost_url("http://192.168.1.10:18082")?;  // ❌ Error
/// validate_localhost_url("http://malicious-127.0.0.1.com:18082")?;  // ❌ Error
/// ```
pub fn validate_localhost_url(url: &str) -> Result<()> {
    use url::Url;

    let parsed = Url::parse(url)
        .context("Invalid URL format")?;

    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("URL missing host"))?;

    // Try to parse as IP address
    if let Ok(ip) = host.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(ipv4) => {
                if ipv4 == Ipv4Addr::LOCALHOST {
                    return Ok(());  // 127.0.0.1
                }
            }
            IpAddr::V6(ipv6) => {
                if ipv6 == std::net::Ipv6Addr::LOCALHOST {
                    return Ok(());  // ::1
                }
            }
        }
        anyhow::bail!(
            "RPC URL must be localhost (127.0.0.1 or ::1), got: {}",
            ip
        );
    }

    // If not an IP, check hostname
    if host == "localhost" {
        return Ok(());
    }

    anyhow::bail!(
        "RPC URL must be localhost, got hostname: {}. Use 127.0.0.1 instead.",
        host
    );
}

/// Create HTTP client with mandatory Tor proxy
///
/// # Arguments
/// * `timeout_secs` - Request timeout in seconds
///
/// # Returns
/// reqwest::Client configured with Tor SOCKS5h proxy
///
/// # Security
/// - Uses `socks5h://` to prevent DNS leaks
/// - Enforces reasonable timeout to prevent hanging
pub fn create_tor_client(timeout_secs: u64) -> Result<Client> {
    let proxy = Proxy::all(TOR_PROXY)
        .context("Failed to configure Tor SOCKS5h proxy")?;

    Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .context("Failed to build Tor HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_localhost_valid() {
        assert!(validate_localhost_url("http://127.0.0.1:18082").is_ok());
        assert!(validate_localhost_url("http://localhost:18082").is_ok());
        assert!(validate_localhost_url("http://[::1]:18082").is_ok());
    }

    #[test]
    fn test_validate_localhost_invalid() {
        assert!(validate_localhost_url("http://192.168.1.10:18082").is_err());
        assert!(validate_localhost_url("http://0.0.0.0:18082").is_err());
        assert!(validate_localhost_url("http://malicious-127.0.0.1.com:18082").is_err());
        assert!(validate_localhost_url("http://localhost.attacker.com:18082").is_err());
    }

    #[tokio::test]
    #[ignore]  // Requires Tor daemon running
    async fn test_tor_validation() {
        let result = validate_tor_running().await;
        assert!(result.is_ok());
    }
}
```

---

### Integration into Startup

**File:** `server/src/main.rs` (MODIFIED)

```rust
// Add at top of main():
mod network;
use network::tor_validator;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // ... logging setup ...

    // ✅ NEW: Validate Tor before starting server
    tor_validator::validate_tor_running()
        .await
        .context("Tor validation failed - cannot start server without Tor")?;

    info!("✅ Tor connectivity validated");

    // ... rest of startup ...
}
```

---

### Updated Monero RPC Validation

**File:** `wallet/src/rpc.rs` (MODIFIED)

```rust
use server::network::tor_validator;

impl MoneroRpcClient {
    pub fn new(config: MoneroConfig) -> Result<Self, MoneroError> {
        let url = config.rpc_url;

        // ✅ NEW: Strict localhost validation (replaces weak contains() check)
        tor_validator::validate_localhost_url(&url)
            .map_err(|e| MoneroError::ValidationError(e.to_string()))?;

        // ... rest of implementation ...
    }
}
```

---

### Updated IPFS Client

**File:** `server/src/ipfs/client.rs` (MODIFIED)

```rust
use crate::network::tor_validator;

impl IpfsClient {
    pub fn new(api_url: String, gateway_url: String) -> Result<Self> {
        let mut client_builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .pool_max_idle_per_host(10);

        // ✅ NEW: Always validate localhost OR use Tor
        if api_url.starts_with("http://127.0.0.1") || api_url.starts_with("http://localhost") {
            // Validate it's REALLY localhost
            tor_validator::validate_localhost_url(&api_url)
                .context("IPFS API URL claims localhost but validation failed")?;

            tracing::info!("IPFS: Connecting to validated local node (no Tor)");
        } else {
            // External IPFS - MUST use Tor
            tracing::info!("IPFS: External gateway detected - enforcing Tor proxy");

            // ✅ Changed: socks5:// → socks5h:// (prevent DNS leak)
            let proxy = Proxy::all("socks5h://127.0.0.1:9050")
                .context("Failed to configure Tor SOCKS5h proxy for IPFS")?;
            client_builder = client_builder.proxy(proxy);
        }

        let client = client_builder.build()?;
        Ok(Self { client, api_base_url: api_url, gateway_url })
    }
}
```

---

## Validation & Testing

### Test 1: Tor Daemon Running

```bash
# Start Tor
sudo systemctl start tor

# Verify Tor check passes
cargo run --bin server

# Expected output:
# INFO Validating Tor daemon connectivity...
# INFO ✅ Tor validation PASSED - Exit IP: 185.220.101.45
# INFO Starting HTTP server on http://127.0.0.1:8080
```

---

### Test 2: Tor Daemon Stopped (Should Fail)

```bash
# Stop Tor
sudo systemctl stop tor

# Try to start server
cargo run --bin server

# Expected output:
# INFO Validating Tor daemon connectivity...
# ERROR Tor validation FAILED - connection refused
# Error: Tor validation failed - cannot start server without Tor
```

---

### Test 3: Malicious RPC URL Rejected

```bash
# Set malicious RPC URL
export MONERO_RPC_URL="http://malicious-127.0.0.1.attacker.com:18082"

cargo run --bin server

# Expected output:
# ERROR RPC URL must be localhost, got hostname: malicious-127.0.0.1.attacker.com
# Error: Failed to create Monero RPC client
```

---

### Test 4: DNS Leak Test

```bash
# Monitor DNS queries
sudo tcpdump -i any port 53 &

# Start server with external IPFS
export IPFS_API_URL="https://ipfs.infura.io:5001/api/v0"
cargo run --bin server

# Check tcpdump output
# ✅ Should see NO DNS queries (Tor handles DNS)
# ❌ If you see DNS queries → DNS leak vulnerability confirmed
```

---

## Temporary Mitigations

Until full validation is implemented:

### Mitigation 1: Manual Tor Check Before Startup

```bash
#!/bin/bash
# Pre-flight Tor check script

echo "Checking Tor connectivity..."

tor_ip=$(curl --socks5-hostname 127.0.0.1:9050 https://check.torproject.org/api/ip 2>/dev/null | jq -r '.IP')

if [ -z "$tor_ip" ]; then
    echo "❌ FATAL: Tor is NOT running or not accessible"
    echo "Start Tor: sudo systemctl start tor"
    exit 1
fi

echo "✅ Tor is running - Exit IP: $tor_ip"
echo "Starting marketplace server..."
./target/release/server
```

---

### Mitigation 2: Hardcoded RPC URLs (No .env)

```rust
// server/src/main.rs
let monero_config = MoneroConfig {
    rpc_url: "http://127.0.0.1:18082".to_string(),  // Hardcoded
    timeout_seconds: 30,
};

// ❌ Do NOT read from env var (prevents tampering)
// let rpc_url = env::var("MONERO_RPC_URL")?;
```

**Limitations:**
- Less flexible (can't change without recompiling)
- ✅ More secure (no config file tampering)

---

## Historical Precedents

### Case Study: Silk Road IP Leak (2013)

**Incident:** Silk Road server's real IP exposed via CAPTCHA misconfiguration

**Root Cause:** CAPTCHA service connected via clearnet instead of Tor

**Parallel to TM-004:**
- Silk Road: External service bypassed Tor
- TM-004: IPFS/RPC could bypass Tor if misconfigured

**Lesson:** **ONE clearnet connection = total deanonymization**

---

## References

1. **Tor Project - DNS Leaks**
   https://support.torproject.org/tbb/dns-leak/

2. **OWASP - Network Segmentation**
   https://cheatsheetseries.owasp.org/cheatsheets/Network_Segmentation_Cheat_Sheet.html

3. **Whonix - Stream Isolation**
   https://www.whonix.org/wiki/Stream_Isolation

---

## Appendices

### Appendix A: Network Security Checklist

**Pre-Production Validation:**
- [ ] Tor daemon running and validated at startup
- [ ] All external HTTP clients use `socks5h://` (not `socks5://`)
- [ ] Monero RPC URL validated with strict IP parsing
- [ ] IPFS client validates localhost OR enforces Tor
- [ ] No clearnet DNS queries (verify with tcpdump)
- [ ] Server binds to 127.0.0.1 only (no 0.0.0.0)

---

### Appendix B: DNS Leak Detection

```bash
#!/bin/bash
# Detect DNS leaks during server operation

echo "Starting DNS leak detection..."
echo "Press Ctrl+C to stop"

sudo tcpdump -i any -n port 53 | while read line; do
    # Ignore Tor's own DNS (to 127.0.0.1:9053)
    if ! echo "$line" | grep -q "127.0.0.1.9053"; then
        echo "⚠️ POTENTIAL DNS LEAK: $line"
    fi
done
```

---

## End of Report

**Next Steps:**

1. **Immediate:** Implement `tor_validator.rs` module (2 hours)
2. **Short-term:** Add startup Tor validation (30 minutes)
3. **Long-term:** Continuous Tor health monitoring (4 hours)

**Status:** Awaiting approval to proceed with implementation

---

**Report prepared by:** Claude (Anthropic)
**Review required by:** Project security lead
**Classification:** INTERNAL - Security Audit
**Version:** 1.0
**Last updated:** 2025-10-26
