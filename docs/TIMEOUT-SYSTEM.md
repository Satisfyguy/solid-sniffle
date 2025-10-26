# Timeout System Documentation

## Overview

The Timeout System prevents escrows from becoming stuck indefinitely by automatically detecting and handling escrows that exceed configured time limits. It provides real-time monitoring, automatic actions, and WebSocket notifications to all affected parties.

## Architecture

### Components

1. **TimeoutConfig** (`server/src/config/timeout.rs`)
   - Configurable timeout policies per escrow status
   - Environment variable support
   - Default values for production readiness

2. **TimeoutMonitor** (`server/src/services/timeout_monitor.rs`)
   - Background service polling database every 60s (configurable)
   - Detects expired and expiring escrows
   - Triggers automatic actions based on status
   - Sends WebSocket notifications

3. **Escrow Model Extensions** (`server/src/models/escrow.rs`)
   - `expires_at`: Deadline timestamp for current status
   - `last_activity_at`: Track last significant action
   - Methods: `is_expired()`, `is_expiring_soon()`, `seconds_until_expiration()`

4. **WebSocket Events** (`server/src/websocket.rs`)
   - `EscrowExpiring`: Warning before expiration
   - `EscrowExpired`: Notification when deadline passed
   - `EscrowAutoCancelled`: Automatic cancellation notification
   - `DisputeEscalated`: Arbiter timeout escalation
   - `TransactionStuck`: Blockchain confirmation timeout

5. **Monitoring API** (`server/src/handlers/monitoring.rs`)
   - `GET /admin/escrows/health`: System-wide health check
   - `GET /admin/escrows/{id}/status`: Detailed escrow status

## Database Schema

### New Columns in `escrows` Table

```sql
ALTER TABLE escrows ADD COLUMN expires_at TIMESTAMP;
ALTER TABLE escrows ADD COLUMN last_activity_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP;

CREATE INDEX idx_escrows_timeout ON escrows(status, expires_at) WHERE expires_at IS NOT NULL;
```

**Migration:** `server/migrations/2025-10-26-175351-0000_add_timeout_fields_to_escrows/`

## Timeout Policies

### Default Timeouts

| Status | Timeout | Action on Expiration |
|--------|---------|----------------------|
| `created` | 1 hour | Auto-cancel (multisig setup incomplete) |
| `funded` | 24 hours | Auto-cancel (buyer never deposited) |
| `releasing` | 6 hours | Alert admin (transaction stuck) |
| `refunding` | 6 hours | Alert admin (transaction stuck) |
| `disputed` | 7 days | Escalate to admin |

### Terminal States (No Timeout)

- `completed`
- `refunded`
- `cancelled`
- `expired`

## Configuration

### Environment Variables

Set in `.env` file:

```bash
# Timeout durations (in seconds)
TIMEOUT_MULTISIG_SETUP_SECS=3600         # 1 hour
TIMEOUT_FUNDING_SECS=86400                # 24 hours
TIMEOUT_TX_CONFIRMATION_SECS=21600        # 6 hours
TIMEOUT_DISPUTE_RESOLUTION_SECS=604800    # 7 days

# Monitoring intervals
TIMEOUT_POLL_INTERVAL_SECS=60             # 1 minute
TIMEOUT_WARNING_THRESHOLD_SECS=3600       # 1 hour warning before expiration
```

### Production Recommendations

**Short timeouts (aggressive):**
```bash
TIMEOUT_MULTISIG_SETUP_SECS=1800    # 30 minutes
TIMEOUT_FUNDING_SECS=43200          # 12 hours
```

**Long timeouts (permissive):**
```bash
TIMEOUT_MULTISIG_SETUP_SECS=7200    # 2 hours
TIMEOUT_FUNDING_SECS=172800         # 48 hours
```

## Workflow

### 1. Escrow Creation

```
Escrow created with status="created"
  ↓
last_activity_at = NOW()
  ↓
expires_at = NOW() + TIMEOUT_MULTISIG_SETUP_SECS
```

### 2. Status Transitions

Every time an escrow status changes, the system:

1. Updates `last_activity_at` to current time
2. Calculates new `expires_at` based on new status:
   - Terminal states → `expires_at = NULL`
   - Active states → `expires_at = NOW() + timeout_for_status(new_status)`

### 3. Timeout Detection

TimeoutMonitor runs every `TIMEOUT_POLL_INTERVAL_SECS` (default 60s):

```sql
-- Find expired escrows
SELECT * FROM escrows
WHERE expires_at IS NOT NULL
  AND expires_at < NOW()
  AND status NOT IN ('completed', 'refunded', 'cancelled', 'expired');

-- Find expiring soon (warning threshold)
SELECT * FROM escrows
WHERE expires_at IS NOT NULL
  AND expires_at > NOW()
  AND expires_at <= NOW() + INTERVAL '1 hour'
  AND status NOT IN ('completed', 'refunded', 'cancelled', 'expired');
```

### 4. Automatic Actions

#### `created` (Multisig Setup Timeout)
```
Expired escrow detected
  ↓
Update status to "cancelled"
  ↓
Send WebSocket notification: EscrowAutoCancelled
  ↓
Log: "Escrow {id} auto-cancelled due to setup timeout"
```

**Reason:** Multisig was never completed, no funds at risk.

#### `funded` (Funding Timeout)
```
Expired escrow detected
  ↓
Update status to "cancelled"
  ↓
Send WebSocket notification: EscrowAutoCancelled
  ↓
Log: "Escrow {id} auto-cancelled due to funding timeout"
```

**Reason:** Buyer never deposited funds to multisig address.

#### `releasing`/`refunding` (Transaction Stuck)
```
Expired escrow detected
  ↓
Calculate hours_pending
  ↓
Send WebSocket notification: TransactionStuck
  ↓
Log: "Transaction {tx_hash} stuck for {hours}h - check blockchain"
```

**Reason:** Transaction may be stuck in mempool or have low fee. **No auto-action** since funds are already on blockchain.

#### `disputed` (Arbiter Timeout)
```
Expired escrow detected
  ↓
Calculate days_in_dispute
  ↓
Send WebSocket notification: DisputeEscalated
  ↓
Log: "Dispute escalated for escrow {id}: admin intervention required"
```

**Reason:** Arbiter failed to resolve within 7 days. **Manual intervention required.**

## WebSocket Notifications

### Client-Side Integration

Clients should listen for timeout events:

```javascript
// WebSocket connection
const ws = new WebSocket('ws://127.0.0.1:8080/ws/');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  switch (data.type) {
    case 'EscrowExpiring':
      console.warn(`Escrow ${data.escrow_id} expires in ${data.expires_in_secs}s`);
      console.warn(`Action required: ${data.action_required}`);
      showWarningBanner(data);
      break;

    case 'EscrowExpired':
      console.error(`Escrow ${data.escrow_id} has expired`);
      console.error(`Reason: ${data.reason}`);
      showErrorModal(data);
      break;

    case 'EscrowAutoCancelled':
      console.log(`Escrow ${data.escrow_id} was auto-cancelled: ${data.reason}`);
      redirectToOrdersPage();
      break;

    case 'DisputeEscalated':
      console.log(`Dispute escalated for escrow ${data.escrow_id}`);
      console.log(`Action: ${data.action_taken}`);
      break;

    case 'TransactionStuck':
      console.warn(`Transaction ${data.tx_hash} stuck for ${data.hours_pending}h`);
      console.warn(`Suggestion: ${data.suggested_action}`);
      break;
  }
};
```

### Event Recipients

| Event | Sent To |
|-------|---------|
| `EscrowExpiring` | Buyer, Vendor, Arbiter |
| `EscrowExpired` | Buyer, Vendor, Arbiter |
| `EscrowAutoCancelled` | Buyer, Vendor, Arbiter |
| `DisputeEscalated` | Buyer, Vendor, Arbiter, Admin |
| `TransactionStuck` | Buyer, Vendor, Arbiter, Admin |

## Monitoring API

### Get Escrow Health

**Endpoint:** `GET /admin/escrows/health`

**Response:**
```json
{
  "total_active_escrows": 15,
  "escrows_by_status": {
    "created": 3,
    "funded": 8,
    "releasing": 2,
    "disputed": 2
  },
  "expired_escrows": [
    {
      "escrow_id": "123e4567-e89b-12d3-a456-426614174000",
      "status": "created",
      "created_at": "2025-10-26T10:00:00",
      "expires_at": "2025-10-26T11:00:00",
      "hours_overdue": 2
    }
  ],
  "expiring_soon": [
    {
      "escrow_id": "789e0123-e89b-12d3-a456-426614174001",
      "status": "funded",
      "expires_at": "2025-10-27T09:30:00",
      "seconds_remaining": 1800,
      "action_required": "Buyer: deposit funds to escrow address"
    }
  ]
}
```

### Get Specific Escrow Status

**Endpoint:** `GET /admin/escrows/{id}/status`

**Response:**
```json
{
  "escrow_id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "funded",
  "amount": 1000000000000,
  "created_at": "2025-10-26T10:00:00",
  "last_activity_at": "2025-10-26T10:05:00",
  "expires_at": "2025-10-27T10:00:00",
  "seconds_until_expiration": 82800,
  "is_expired": false,
  "is_expiring_soon": false,
  "buyer_id": "...",
  "vendor_id": "...",
  "arbiter_id": "...",
  "multisig_address": "4...",
  "transaction_hash": null
}
```

## Troubleshooting

### Escrow Stuck in "created" for >1 Hour

**Symptom:** Escrow not auto-cancelled despite timeout.

**Diagnosis:**
1. Check `expires_at` field in database:
   ```sql
   SELECT id, status, created_at, expires_at, last_activity_at
   FROM escrows WHERE id = '{escrow_id}';
   ```
2. Check TimeoutMonitor logs:
   ```bash
   grep "TimeoutMonitor" server.log
   ```

**Possible Causes:**
- TimeoutMonitor not running (check logs for "TimeoutMonitor background service started")
- Migration not applied (check `expires_at` column exists)
- Database clock mismatch (verify `NOW()` returns correct time)

**Fix:**
- Restart server to ensure TimeoutMonitor starts
- Apply migration: `DATABASE_URL=../marketplace.db diesel migration run`
- Manually trigger timeout:
  ```sql
  UPDATE escrows SET expires_at = NOW() - INTERVAL '1 hour' WHERE id = '{escrow_id}';
  ```

### Warning Notifications Not Received

**Symptom:** No `EscrowExpiring` events before deadline.

**Diagnosis:**
1. Check WebSocket connection active
2. Verify warning threshold:
   ```bash
   echo $TIMEOUT_WARNING_THRESHOLD_SECS  # Should be 3600
   ```
3. Check escrow is within threshold:
   ```sql
   SELECT id, expires_at,
          EXTRACT(EPOCH FROM (expires_at - NOW())) as seconds_remaining
   FROM escrows WHERE id = '{escrow_id}';
   ```

**Fix:**
- Reconnect WebSocket client
- Lower warning threshold temporarily for testing:
  ```bash
  export TIMEOUT_WARNING_THRESHOLD_SECS=7200  # 2 hours
  ```

### Transaction Stuck Alert Spam

**Symptom:** Continuous `TransactionStuck` notifications for same escrow.

**Diagnosis:**
- Transaction genuinely stuck in mempool due to low fee or blockchain congestion

**Fix:**
1. Check blockchain explorer for transaction status
2. Manually update escrow status once confirmed:
   ```sql
   UPDATE escrows SET status = 'completed', expires_at = NULL
   WHERE id = '{escrow_id}';
   ```
3. Consider increasing `TIMEOUT_TX_CONFIRMATION_SECS` during network congestion

## Security Considerations

### Non-Custodial Architecture

Timeouts respect the non-custodial design:
- ✅ Auto-cancel for `created`/`funded`: No funds at risk
- ✅ Alert for `releasing`/`refunding`: Funds already on blockchain
- ✅ Escalate for `disputed`: Manual admin review required

### No Forced Refunds

TimeoutMonitor **does NOT** automatically execute refunds for expired disputes. This prevents:
- Malicious buyer creating fake timeouts to force refunds
- Race conditions between arbiter resolution and auto-refund
- Loss of funds due to premature refunds

**Manual review required** for all dispute escalations.

## Testing

### Unit Tests

```bash
cargo test --package server timeout
```

### Manual Testing

**Test 1: Create Expired Escrow**
```sql
-- Create test escrow with past expiration
INSERT INTO escrows (id, order_id, buyer_id, vendor_id, arbiter_id, amount, status, created_at, last_activity_at, expires_at)
VALUES ('test-timeout-id', 'order-id', 'buyer-id', 'vendor-id', 'arbiter-id', 1000, 'created', NOW(), NOW(), NOW() - INTERVAL '1 hour');
```

**Expected:** TimeoutMonitor cancels within 60s and sends notification.

**Test 2: Expiring Soon Warning**
```sql
-- Create escrow expiring in 30 minutes
UPDATE escrows SET expires_at = NOW() + INTERVAL '30 minutes' WHERE id = '{escrow_id}';
```

**Expected:** `EscrowExpiring` notification received.

## Performance

### Database Load

- Poll interval: 60s
- Queries per poll: 2 (expired + expiring)
- Index used: `idx_escrows_timeout`

**Estimated load:**
- 10,000 active escrows → ~5ms per poll
- 100,000 active escrows → ~50ms per poll

### Scaling Recommendations

- Poll interval can be increased to 120s for large deployments
- Partition `escrows` table by `created_at` if >1M rows
- Add read replica for monitoring queries

## Future Enhancements

1. **Configurable Auto-Refund for Disputes**
   - After escalation, optionally trigger auto-refund after additional timeout
   - Requires multi-signature review process

2. **Timeout History Logging**
   - Track all timeout events in separate `timeout_events` table
   - Enable analytics and pattern detection

3. **Dynamic Timeout Adjustment**
   - Adjust timeouts based on network congestion (Monero mempool size)
   - Longer timeouts during high blockchain load

4. **Admin Dashboard**
   - Real-time visualization of escrow health
   - Manual timeout override interface
   - Bulk operations for stuck escrows

## References

- **Code:** `server/src/services/timeout_monitor.rs`
- **Config:** `server/src/config/timeout.rs`
- **Migration:** `server/migrations/2025-10-26-175351-0000_add_timeout_fields_to_escrows/`
- **WebSocket Events:** `server/src/websocket.rs`
- **Monitoring API:** `server/src/handlers/monitoring.rs`
