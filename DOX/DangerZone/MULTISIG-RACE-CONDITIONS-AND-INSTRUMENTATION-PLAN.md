# Multisig Race Conditions and Instrumentation Plan

Status: Proposed remediation plan (ready to implement)
Date: 2025-11-14
Scope: server/src/wallet_manager.rs, server/src/services/wallet_session_manager.rs, server/src/instrumentation/*, server/src/models/multisig_state.rs, server/src/wallet_pool.rs

---

## Executive Summary

The current multisig setup implements a robust orchestration with WalletManager, WalletSessionManager, WalletPool, and a complete Instrumentation framework. However, we confirmed several concurrency hazards and a major throughput bottleneck:

- Global wallet creation lock serializes concurrent escrows
- 10-second fixed waits per round (RPC cache purge) severely limit throughput
- Non-atomic copy operations between rounds expose a TOCTOU risk
- Public, unprotected `WalletManager.wallets` HashMap risks concurrent mutation
- Instrumentation macro exists but is not applied to all critical RPC calls

Notably, WalletSessionManager already implements a correct double-checked locking pattern and cleans up duplicate wallets on race, which resolves the previously identified session-creation race.

---

## Verified Findings (Code Reality)

1) Global wallet creation lock (WALLET_CREATION_LOCK)
- File: `server/src/wallet_manager.rs`
- Behavior: Global TokioMutex held during entire wallet creation in `create_temporary_wallet()`.
- Risks: Serializes all wallet creations across escrows; no timeout or contention metrics.

2) 10-second RPC cache purge waits
- File: `server/src/wallet_manager.rs`
- Occurrences: found at lines around 1538 and 1789 via grep
- Impact: Adds ~20–30s per escrow, drastically reducing throughput.

3) Non-atomic file copies between rounds
- File: `server/src/wallet_manager.rs`
- Operations: `std::fs::copy` for wallet + keys into round_2, round_3, and final paths
- Risk: `exists()` + `copy()` is non-atomic; potential TOCTOU under concurrency; no duration/bytes metrics

4) Public `WalletManager.wallets` without synchronization
- File: `server/src/wallet_manager.rs`
- Type: `pub wallets: HashMap<Uuid, WalletInstance>`
- Risk: Concurrent readers/writers can corrupt state or cause panics

5) Instrumentation macro unused on critical RPCs
- File: `server/src/instrumentation/mod.rs` defines `instrument_rpc_call!`
- Observation: Not applied in `wallet_manager.rs` for prepare/make/export/import/close/open

6) Session creation race already addressed
- File: `server/src/services/wallet_session_manager.rs`
- Pattern: Double-checked locking + duplicate wallet cleanup (background close)
- Status: Good; keep and instrument

---

## Remediation Plan

### Phase A — Observability and Safety (quick wins)

A1) Apply `instrument_rpc_call!` in `wallet_manager.rs`
- Wrap all RPCs in multisig phases: `prepare_multisig`, `make_multisig`, `export_multisig_info`, `import_multisig_info`, plus `open_wallet`, `close_wallet`, `set_attribute`, `get_address` as needed.
- Outcome: Per-call duration, success, and role/port tagging for timeline analysis.

A2) Instrument WALLET_CREATION_LOCK
- Measure `lock_wait_ms` and `lock_held_ms` when calling `create_temporary_wallet()`.
- Emit `EventType::Custom` with `{ operation, escrow_id, role }` for lock contention visibility.

A3) Atomic copy operations with metrics
- Replace `exists()` + `copy()` with: copy to `*.tmp` → `rename()` to final (atomic on same FS).
- Emit `EventType::FileOperation` with `{ src, dst, bytes, duration_ms }`.

A4) Replace fixed 10s sleeps with conditional waits
- Implement short retries (e.g., 3–5 attempts, 250–500ms backoff) checking minimal signal (e.g., `is_multisig`, `get_wallet_info`) instead of sleeping 10s unconditionally.
- Record actual wait duration; fallback to capped total wait (e.g., 4s) and emit a warning.

A5) Complete snapshot coverage
- Use `WalletSnapshot` at: pre-round1, post-make_multisig, pre-round2, post-export, pre-round3, post-import, final.
- Add `diff()` comparisons to detect unexpected state jumps (cache pollution).

### Phase B — Concurrency hardening

B1) Per-escrow wallet creation locks
- Replace global `WALLET_CREATION_LOCK` with per-escrow locks (e.g., `DashMap<Uuid, Arc<TokioMutex<()>>>`).
- Apply a timeout (e.g., 5s) and log contention with instrumentation.

B2) Protect `WalletManager.wallets`
- Make field private; wrap with `Arc<RwLock<HashMap<Uuid, WalletInstance>>>`.
- Provide accessor methods (`get`, `insert`, `remove`) using read/write guards.

B3) DB persistence races (if applicable)
- Wrap state transitions in DB transactions or optimistic concurrency control (version/CAS).
- Emit `EventType::StateChange` with `{ from, to, escrow_id }`.

### Phase C — Throughput improvements

C1) WalletPool parallelism
- Ensure role-based RPC separation is respected during rounds to minimize cache interference.
- Where safe, parallelize sub-steps across roles if WalletPool isolates RPC instances cleanly.

C2) Session-first architecture
- Prefer `WalletSessionManager` to keep wallets open across operations and reduce reopen/close churn.
- Add session-level instrumentation: creation time, per-wallet open port, lifetime, eviction.

---

## Concrete Implementation Notes

- Use `instrument_rpc_call!` like:
  ```rust
  let res = instrument_rpc_call!(collector, "make_multisig", role_str, rpc_port, {
      rpc_client.multisig().make_multisig(threshold, peers).await
  });
  ```
- For lock timing:
  ```rust
  let wait_start = Instant::now();
  let _guard = WALLET_CREATION_LOCK.lock().await;
  let wait_ms = wait_start.elapsed().as_millis() as u64;
  collector.record_event(EventType::Custom, role, json!({"event":"lock_acquired","wait_ms":wait_ms})).await;
  let hold_start = Instant::now();
  // ... wallet creation ...
  let hold_ms = hold_start.elapsed().as_millis() as u64;
  collector.record_event(EventType::Custom, role, json!({"event":"lock_released","hold_ms":hold_ms})).await;
  ```
- Atomic copy pattern:
  ```rust
  let tmp = dst.with_extension("tmp");
  std::fs::copy(&src, &tmp)?;
  std::fs::rename(&tmp, &dst)?; // atomic on same fs
  ```
- Conditional wait replacing fixed 10s:
  ```rust
  let start = Instant::now();
  let mut ok = false;
  for (i, delay_ms) in [250, 500, 750, 1000].into_iter().enumerate() {
      if rpc_sanity_check().await.is_ok() { ok = true; break; }
      tokio::time::sleep(Duration::from_millis(delay_ms)).await;
  }
  collector.record_event(EventType::Custom, role, json!({"event":"cache_purge_wait","elapsed_ms": start.elapsed().as_millis() as u64, "ok": ok})).await;
  ```

---

## Metrics to Track (Dashboards)
- Lock contention: wait_ms histograms (creation lock and per-escrow locks)
- RPC call durations: success/error rates by method and role
- File copy throughput: bytes/sec and error rates
- Session metrics: creation time, active sessions, TTL evictions
- End-to-end escrow setup time per phase and total

---

## Risks and Mitigations
- Changing sleep behavior could expose latent cache issues → keep fallback sleep path and emit warnings
- Introducing RwLock may require refactors where `wallets` was directly accessed → stage changes and add helper methods
- Per-escrow locks require lifecycle management → use DashMap and weak cleanup upon escrow completion

---

## Rollout Strategy
1) Implement Phase A with feature flag `ENABLE_INSTRUMENTATION` (already present) and safeguard toggles for conditional waits
2) Deploy to testnet; capture JSON dumps via collector; compare successful vs failing runs
3) Roll in Phase B (locks + RwLock) with minimal API changes
4) Roll in Phase C parallelization once stability confirmed

---

## Open TODOs
- Apply instrument_rpc_call! across wallet_manager.rs RPC calls
- Replace fixed 10s sleeps with conditional wait loop
- Make file copies atomic (tmp + rename) and instrument
- Add per-escrow locks and timeouts
- Protect WalletManager.wallets with RwLock and accessors
- Transactionalize MultisigStateRepository transitions (if needed)

---

## Links / References
- Instrumentation framework: `server/src/instrumentation/*`
- Wallet pool/rotation: `server/src/wallet_pool.rs`
- Session manager (race fix done): `server/src/services/wallet_session_manager.rs`
- State model: `server/src/models/multisig_state.rs`
- Prior related docs: `DOX/DangerZone/RPC-CACHE-POLLUTION-SOLUTION.md`, `DOX/DangerZone/ROUND-ROBIN-FIX-FINAL.md`, `DOX/DangerZone/WALLET-LIFECYCLE-COMPLETE-ANALYSIS.md`
