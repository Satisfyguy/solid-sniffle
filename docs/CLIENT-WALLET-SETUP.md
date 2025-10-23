# Client Wallet Setup - Non-Custodial Guide

**Version:** 1.0
**Date:** 23 October 2025
**Applies to:** Buyers & Vendors

---

## What is Non-Custodial?

In a **non-custodial** marketplace, **YOU control your private keys**.

- ✅ **You** generate your wallet's private keys on **your** machine
- ✅ **You** control the seed phrase (25 words)
- ✅ The marketplace server **NEVER** has access to your private keys
- ✅ Even if the server is hacked, your funds are **safe**

### Why This Matters

**Custodial (BAD):**
```
[Your Funds] → Server Controls Keys → ❌ Exit scam risk
                                       ❌ Hack = loss
                                       ❌ Trust required
```

**Non-Custodial (GOOD):**
```
[Your Funds] → YOU Control Keys → ✅ No exit scam
                                   ✅ Hack ≠ loss
                                   ✅ Trustless
```

---

## Quick Start

### Prerequisites

- Linux, macOS, or Windows WSL
- Tor daemon running (for privacy)
- ~20GB disk space (for testnet blockchain)

### Installation Steps

#### 1. Install Monero CLI

**Linux/Ubuntu:**
```bash
# Download latest Monero CLI
wget https://downloads.getmonero.org/cli/monero-linux-x64-v0.18.3.1.tar.bz2

# Extract
tar -xvf monero-linux-x64-v0.18.3.1.tar.bz2
cd monero-x86_64-linux-gnu-v0.18.3.1

# Optional: Add to PATH
sudo cp monero-wallet-cli monero-wallet-rpc /usr/local/bin/
```

**macOS:**
```bash
# Download latest Monero CLI
wget https://downloads.getmonero.org/cli/monero-mac-x64-v0.18.3.1.tar.bz2

# Extract
tar -xvf monero-mac-x64-v0.18.3.1.tar.bz2
cd monero-x86_64-apple-darwin11-v0.18.3.1

# Optional: Add to PATH
sudo cp monero-wallet-cli monero-wallet-rpc /usr/local/bin/
```

**Windows (WSL):**
```bash
# Same as Linux instructions above
```

#### 2. Create Your Wallet

**IMPORTANT:** This wallet is for **testnet** only (no real XMR).

```bash
# Create new wallet (interactive)
./monero-wallet-cli --testnet --generate-new-wallet ~/my_marketplace_wallet

# You will be prompted for:
# - Password (choose strong password)
# - Confirm password
```

**Output:**
```
Generated new wallet: 9uN... (testnet address)

View key: abc123...
********************************************************************
Your wallet has been generated!

** PLEASE NOTE DOWN YOUR MNEMONIC SEED **
********************************************************************
```

#### 3. BACKUP YOUR SEED PHRASE ⚠️

When you create the wallet, you'll see 25 words:

```
Example:
abbey abducts aching acquire across adapt addiction adjust admit adopt adrenalin adult
advance affirm afraid after against agenda aggravate agile aging agreed airport
```

**CRITICAL:**
- ✅ Write these words on paper (NOT digital)
- ✅ Store in a safe place (fireproof safe ideal)
- ❌ NEVER share with anyone (including marketplace admins)
- ❌ NEVER store in cloud/email/notes app

**This seed phrase = access to your funds forever!**

#### 4. Start Wallet RPC

```bash
# Start wallet RPC server (required for marketplace integration)
./monero-wallet-rpc --testnet \
    --rpc-bind-port 18082 \
    --wallet-file ~/my_marketplace_wallet \
    --password "your_wallet_password" \
    --disable-rpc-login \
    --rpc-bind-ip 127.0.0.1

# Keep this terminal window open!
```

**Expected output:**
```
2025-10-23 12:00:00 I Monero 'Oxygen Orion' (v0.18.3.1-release)
2025-10-23 12:00:00 I Wallet RPC server started, listening on port 18082
2025-10-23 12:00:00 I Run server in background: No
```

**Security Notes:**
- `--rpc-bind-ip 127.0.0.1` = Localhost only (can't be accessed remotely)
- `--disable-rpc-login` = No auth (OK for localhost testnet)
- For **mainnet** (real money): ALWAYS use `--rpc-login user:password`

#### 5. Register Wallet with Marketplace

**Via API (curl):**
```bash
curl -X POST http://marketplace.onion/api/escrow/register-wallet-rpc \
  -H "Content-Type: application/json" \
  -H "Cookie: session=YOUR_SESSION_COOKIE" \
  -d '{
    "rpc_url": "http://127.0.0.1:18082/json_rpc",
    "rpc_user": null,
    "rpc_password": null,
    "role": "buyer"
  }'
```

**Via Web Interface:**
1. Login to marketplace
2. Go to **Settings → Wallet**
3. Click **Register Non-Custodial Wallet**
4. Fill form:
   - RPC URL: `http://127.0.0.1:18082/json_rpc`
   - Role: `Buyer` or `Vendor`
   - Submit

**Expected response:**
```json
{
  "success": true,
  "message": "✅ Wallet RPC registered successfully. You control your private keys.",
  "wallet_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "wallet_address": "9uN...",
  "role": "buyer"
}
```

---

## Advanced Setup

### Production (Mainnet) Security

⚠️ **NEVER use testnet wallet for real money!**

For production with real XMR:

```bash
# Create mainnet wallet
./monero-wallet-cli --generate-new-wallet ~/mainnet_wallet

# Start RPC with authentication
./monero-wallet-rpc \
    --rpc-bind-port 18082 \
    --wallet-file ~/mainnet_wallet \
    --password "strong_wallet_password" \
    --rpc-login "rpc_user:rpc_strong_password" \
    --rpc-bind-ip 127.0.0.1 \
    --confirm-external-bind
```

**Register with marketplace (mainnet):**
```json
{
  "rpc_url": "http://127.0.0.1:18082/json_rpc",
  "rpc_user": "rpc_user",
  "rpc_password": "rpc_strong_password",
  "role": "buyer"
}
```

### Tor Hidden Service Setup (Advanced)

For ultimate privacy, expose wallet RPC via Tor:

**1. Create hidden service:**
Edit `/etc/tor/torrc`:
```
HiddenServiceDir /var/lib/tor/wallet_rpc/
HiddenServicePort 18082 127.0.0.1:18082
```

**2. Restart Tor:**
```bash
sudo systemctl restart tor

# Get your .onion address
sudo cat /var/lib/tor/wallet_rpc/hostname
```

**3. Register with marketplace:**
```json
{
  "rpc_url": "http://abc123xyz.onion:18082/json_rpc",
  "rpc_user": "rpc_user",
  "rpc_password": "rpc_password",
  "role": "buyer"
}
```

**Advantages:**
- Marketplace can connect to your wallet via Tor (no local network required)
- Your IP address never exposed
- Works across the internet (if both use Tor)

---

## Usage & Workflow

### Normal Purchase Flow (Buyer)

1. **Create wallet** (one-time setup)
2. **Start wallet RPC** (keep running)
3. **Register with marketplace**
4. **Browse & order** items
5. **Marketplace creates 2-of-3 multisig escrow**
6. **You participate in multisig setup** (automatic via RPC)
7. **Send XMR to multisig address**
8. **Vendor ships**
9. **You release funds** (requires 2 signatures: you + arbiter OR you + vendor)

### Selling Flow (Vendor)

Same as buyer, but:
- Role: `vendor`
- You receive XMR when buyer releases

### Dispute Flow

If something goes wrong:
1. **Initiate dispute** on marketplace
2. **Arbiter investigates**
3. **Arbiter + buyer** OR **arbiter + vendor** sign to release funds
4. **2-of-3 multisig = no single party can steal**

---

## Troubleshooting

### Error: "Wallet RPC not reachable"

**Cause:** Wallet RPC not running or wrong URL.

**Fix:**
```bash
# Check if wallet RPC is running
curl http://127.0.0.1:18082/json_rpc -d '{"jsonrpc":"2.0","id":"0","method":"get_version"}'

# Should return: {"id":"0","jsonrpc":"2.0","result":{...}}
# If not, start wallet RPC (see step 4)
```

### Error: "Non-custodial policy violation"

**Cause:** Trying to register as `arbiter` (not allowed).

**Fix:** Use `"role": "buyer"` or `"role": "vendor"` only.

### Error: "Role mismatch"

**Cause:** Your user account role doesn't match wallet role.

**Fix:**
- If registered as buyer on marketplace → Use `"role": "buyer"`
- If registered as vendor → Use `"role": "vendor"`

### Wallet sync taking forever

**Cause:** Monero wallet needs to sync with blockchain.

**Fix:**
```bash
# Check sync status in wallet CLI
./monero-wallet-cli --testnet --wallet-file ~/my_marketplace_wallet

# Inside wallet CLI:
> refresh
> status

# Testnet sync can take 2-4 hours first time
```

### Lost seed phrase

**Cause:** Seed phrase not backed up.

**Fix:** ⚠️ **NO FIX** - Funds are permanently lost if seed phrase is lost!

**Prevention:**
- Write seed on paper NOW
- Store in multiple safe locations
- Consider metal backup for fire protection

---

## Security Best Practices

### ✅ DO

- ✅ Write seed phrase on paper, store offline
- ✅ Use strong unique password for wallet
- ✅ Run wallet RPC on localhost only (`127.0.0.1`)
- ✅ Use RPC authentication for mainnet (`--rpc-login`)
- ✅ Keep wallet software updated
- ✅ Use Tor for all marketplace connections
- ✅ Verify multisig address before sending large amounts

### ❌ DON'T

- ❌ Share seed phrase with ANYONE (including support)
- ❌ Store seed phrase digitally (cloud, email, notes app)
- ❌ Reuse wallet password for other accounts
- ❌ Expose wallet RPC to public internet without auth
- ❌ Mix testnet and mainnet wallets
- ❌ Trust, verify! Always check multisig setup

---

## FAQ

**Q: Do I need to keep wallet RPC running all the time?**
A: Only when:
- Registering wallet with marketplace
- Participating in multisig setup
- Releasing/refunding escrow funds

You can stop it when not actively trading.

**Q: Can the marketplace steal my funds?**
A: **NO.** With 2-of-3 multisig:
- Marketplace (arbiter) has 1 key
- You (buyer/vendor) have 1 key
- Other party has 1 key

**Any 2 signatures required to move funds.** Marketplace alone cannot steal.

**Q: What if marketplace goes offline?**
A: Your funds are safe! The multisig address exists on the Monero blockchain, independent of the marketplace server.

**Q: Can I use hardware wallet (Ledger/Trezor)?**
A: Not yet supported for multisig escrow. Use software wallet for now.

**Q: How do I upgrade from testnet to mainnet?**
A:
1. Create NEW mainnet wallet (NEVER reuse testnet wallet)
2. Start mainnet wallet RPC with authentication
3. Register new wallet with marketplace
4. Transfer real XMR to new wallet

**Q: Is this complicated? Why not just let marketplace handle wallets?**
A: **Security vs Convenience tradeoff:**
- Custodial (marketplace controls keys) = Easy but risky (exit scam, hacks)
- Non-custodial (you control keys) = Slightly more setup but **your funds are safe**

We prioritize **your security** over convenience.

---

## Support & Help

**Issues with this guide?**
- GitHub: https://github.com/monero-marketplace/issues
- Community: [Tor Forum Link]

**Monero help:**
- Monero Docs: https://www.getmonero.org/resources/user-guides/
- r/Monero: https://reddit.com/r/Monero

**Security concerns:**
- Email: security@marketplace.onion (PGP: [key fingerprint])

---

## Checklist

Before starting trading:

- [ ] Monero CLI installed
- [ ] Testnet wallet created
- [ ] Seed phrase written on paper and stored safely
- [ ] Wallet RPC running on localhost:18082
- [ ] Wallet registered with marketplace
- [ ] Verified wallet address in marketplace profile
- [ ] Understand 2-of-3 multisig escrow flow
- [ ] Read security best practices

**Ready to trade? You're now part of the non-custodial revolution!** 🔒

---

**Last updated:** 23 October 2025
**Version:** 1.0 (Phase 2 Non-Custodial Migration)
