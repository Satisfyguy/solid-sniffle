# Arbiter Setup Guide

Complete setup guide for the offline air-gap arbiter system.

## Overview

The arbiter is a **completely offline** wallet that holds the third key in 2-of-3 multisig escrows. It NEVER connects to the internet, ensuring maximum security even if the server is compromised.

## Architecture

```
┌──────────────────┐                  ┌──────────────────────┐
│  Online Server   │                  │  Offline Arbiter     │
│  (Marketplace)   │                  │  (Tails USB)         │
└────────┬─────────┘                  └──────────┬───────────┘
         │                                       │
         │  1. Export Dispute (QR Code)          │
         ├──────────────────────────────────────>│
         │                                       │
         │                          2. Review Evidence
         │                          3. Sign Decision
         │                                       │
         │  4. Import Decision (QR Code)         │
         │<──────────────────────────────────────┤
         │                                       │
```

## Requirements

### Hardware
- **Dedicated USB stick** (8GB+) with Tails OS
- **QR code scanner** (webcam or phone)
- **USB drive** (read-only) for evidence transfer

### Software (on Tails)
- Python 3 with PyNaCl
- qrencode (for QR generation)
- zbarcam (for QR scanning)

## Setup Steps

### Step 1: Install Tails OS

1. Download Tails from https://tails.boum.org
2. Verify signature
3. Flash to USB stick
4. Boot into Tails (verify network disabled)

### Step 2: Generate Arbiter Keypair

**⚠️ CRITICAL: Run this ONLY on the offline Tails machine**

```bash
# On Tails USB (offline)
./scripts/airgap/generate-arbiter-keypair.sh
```

This generates:
- **Public key** (64 hex chars) → Goes in server .env
- **Private key** (64 hex chars) → Stays on Tails ONLY

Output example:
```
Public Key: a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
Private Key: fedcba0987654321abcdef1234567890...  ← NEVER share this
```

### Step 3: Configure Server

Add to server's `.env` file:

```bash
# Arbiter Public Key (Ed25519, hex-encoded)
ARBITER_PUBKEY=a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456
```

Restart server:
```bash
killall server
./target/release/server
```

### Step 4: Configure Offline Script

Edit `arbiter-offline-review.sh` on Tails USB:

```bash
# Line ~50
ARBITER_PRIVATE_KEY="fedcba0987654321abcdef1234567890..."
```

Make executable:
```bash
chmod +x arbiter-offline-review.sh
```

### Step 5: Test Signature

On Tails, test signing:

```bash
python3 <<EOF
from nacl.signing import SigningKey
from nacl.encoding import HexEncoder

sk = SigningKey('YOUR_PRIVATE_KEY_HEX', encoder=HexEncoder)
message = b"ARBITER_DECISION:test-escrow-id:test-nonce"
signature = sk.sign(message).signature

print(f"Signature: {signature.hex()}")
EOF
```

On server, verify:

```bash
curl -X POST http://localhost:8080/api/test/verify-arbiter-signature \
  -H "Content-Type: application/json" \
  -d '{
    "message": "ARBITER_DECISION:test-escrow-id:test-nonce",
    "signature": "SIGNATURE_HEX_FROM_ABOVE"
  }'
```

Should return: `{"valid": true}`

## Operational Workflow

### When a Dispute Occurs

#### On Server (Online)

1. Marketplace admin exports dispute:
   ```bash
   curl http://localhost:8080/api/escrow/{escrow_id}/dispute/export
   ```

2. Server generates QR code with:
   - Escrow details
   - Buyer claim
   - Vendor response
   - Evidence file list
   - Nonce (anti-replay)

3. Admin scans QR code with phone/webcam

#### On Tails USB (Offline)

1. Boot Tails USB (ensure network disabled)

2. Run arbiter script:
   ```bash
   ./arbiter-offline-review.sh
   ```

3. Menu appears:
   ```
   ╔════════════════════════════════════════╗
   ║  Air-Gap Arbiter - Dispute Review     ║
   ╚════════════════════════════════════════╝

   1. Scan Dispute QR Code
   2. Review Evidence (USB)
   3. Make Decision
   4. Export Decision QR
   5. Exit
   ```

4. Select **1. Scan Dispute QR Code**
   - Uses webcam to scan QR from phone/screen
   - Parses dispute details
   - Displays:
     - Escrow ID
     - Amount (XMR)
     - Buyer claim
     - Vendor response
     - Evidence count

5. Select **2. Review Evidence (USB)**
   - Insert USB drive (read-only mode)
   - Script mounts USB and displays evidence files
   - Opens images, PDFs, etc. for review
   - Evidence files named: `evidence_{escrow_id}_*.{png,pdf,txt}`

6. Select **3. Make Decision**
   - Prompts: `Decision? (buyer/vendor):`
   - Enter: `buyer` or `vendor`
   - Prompts: `Reason:`
   - Enter explanation (will be logged on-chain)

7. Script signs decision with private key:
   ```
   Message: ARBITER_DECISION:{escrow_id}:{nonce}
   Signature: {64-byte Ed25519 signature}
   ```

8. Select **4. Export Decision QR**
   - Generates QR code containing:
     ```json
     {
       "escrow_id": "...",
       "nonce": "...",
       "decision": "buyer",
       "reason": "Product not as described, evidence supports buyer",
       "signed_tx_hex": "...",
       "decision_signature": "...",
       "decided_at": 1234567890
     }
     ```
   - Displays QR on screen

#### Back on Server

1. Admin scans decision QR code with phone

2. Admin imports decision:
   ```bash
   curl -X POST http://localhost:8080/api/escrow/{escrow_id}/dispute/import \
     -H "Content-Type: application/json" \
     -d '{"decision_json": "{...QR_PAYLOAD...}"}'
   ```

3. Server verifies:
   - ✅ Signature matches ARBITER_PUBKEY
   - ✅ Nonce matches original dispute
   - ✅ Escrow is in "disputed" state

4. Server updates escrow:
   - Status: `refunded` (if buyer won) or `completed` (if vendor won)
   - Stores signed transaction in database
   - Logs decision permanently

5. Transaction is broadcast (manual verification in alpha):
   ```bash
   # Admin manually broadcasts after verification
   monero-wallet-cli --testnet
   > relay_tx {signed_tx_hex}
   ```

## Security Properties

### ✅ What This Protects Against

1. **Server Compromise**
   - Attacker gains root access to marketplace server
   - Result: Cannot steal arbiter keys (they're offline)

2. **RCE Exploit**
   - Attacker exploits code execution vulnerability
   - Result: Cannot force arbiter decisions (offline signing required)

3. **State Actor Seizure**
   - Law enforcement seizes server hardware
   - Result: Cannot recover arbiter keys (on separate Tails USB)

4. **Insider Threat**
   - Malicious admin with database access
   - Result: Cannot forge arbiter signatures (private key unknown)

### ⚠️ What This Does NOT Protect Against

1. **Physical Theft of Tails USB**
   - If arbiter USB is stolen, private key is exposed
   - Mitigation: Store Tails USB in secure location (safe, lockbox)

2. **Coerced Arbiter**
   - Attacker forces arbiter to sign malicious decision
   - Mitigation: M-of-N arbiters (future), arbiter reputation system

3. **Evidence Tampering**
   - Attacker modifies evidence files before USB transfer
   - Mitigation: Cryptographic hashes in dispute export

## Troubleshooting

### QR Code Too Large

If dispute data exceeds QR capacity (1852 bytes):

```
ERROR: QR payload too large (2048 bytes > 1852 max)
```

**Solution**: Use USB transfer instead of QR:
```bash
# Export to JSON file
curl http://localhost:8080/api/escrow/{id}/dispute/export > dispute.json

# Transfer via USB to Tails
# Import manually in arbiter script
```

### Signature Verification Failed

```
ERROR: Signature verification failed
```

**Causes**:
1. Wrong ARBITER_PUBKEY in .env
2. Private key doesn't match public key
3. Message format mismatch

**Debug**:
```bash
# On Tails, print public key from private key
python3 <<EOF
from nacl.signing import SigningKey
from nacl.encoding import HexEncoder

sk = SigningKey('PRIVATE_KEY_HEX', encoder=HexEncoder)
print(sk.verify_key.encode(encoder=HexEncoder).decode())
EOF
```

Compare output with server's ARBITER_PUBKEY.

### Tails Persistence Issues

If arbiter script disappears after reboot:

**Solution**: Enable Tails persistence for /home directory
```bash
# In Tails
Applications → Tails → Configure persistent volume
# Enable: Personal Data
```

## Backup & Recovery

### Backup Arbiter Private Key

**Option 1: Paper Backup**
1. Print private key on paper
2. Store in fireproof safe
3. Optionally split using Shamir Secret Sharing (3-of-5)

**Option 2: Encrypted USB**
1. Create encrypted USB with VeraCrypt
2. Store private key in encrypted volume
3. Keep USB in separate secure location

**Option 3: BIP39 Mnemonic** (Advanced)
Convert private key to 24-word seed phrase for easier backup.

### Recovery Procedure

If Tails USB is lost:

1. Boot new Tails USB
2. Restore private key from backup
3. Verify public key matches server:
   ```bash
   python3 -c "
   from nacl.signing import SigningKey
   sk = SigningKey('BACKED_UP_PRIVATE_KEY', encoder=...)
   print(sk.verify_key.encode(...))
   "
   ```
4. Copy arbiter-offline-review.sh to new USB
5. Test signature with server

## Production Hardening

For mainnet deployment:

1. **Use dedicated hardware wallet**
   - Replace Tails USB with Ledger/Trezor
   - Sign decisions via hardware device

2. **Multi-arbiter system**
   - Use 2-of-3 arbiters instead of single arbiter
   - Requires consensus for disputes

3. **Tamper-evident packaging**
   - Seal Tails USB in evidence bag
   - Detect physical access

4. **Geographic distribution**
   - Store arbiter USB in bank safety deposit box
   - Require physical presence for disputes

5. **Audit logs**
   - Record all arbiter decisions
   - Publish monthly transparency reports

## References

- [TM-001 Audit Finding](../security-audit/TM-001-air-gap-arbiter.md)
- [Arbiter Offline Script](../scripts/airgap/arbiter-offline-review.sh)
- [Air-Gap Module](../server/src/services/airgap.rs)
- [Tails OS Documentation](https://tails.boum.org/doc/)
