# Multisig Flow – Implementation Plan (DangerZone)

Owner: Engineering (Server + Wallet + Frontend)
Date: 2025-11-14
Scope: Secure 2-of-3 multisig flow end-to-end with server-side binding, persistence, sync round endpoint, and UI phase alignment.

## Goals
- Enforce secure challenge-response binding for multisig submissions (addresses TM-003 finding).
- Persist validated multisig_info and transition state machine in DB (atomic, encrypted fields where needed).
- Provide an API endpoint to orchestrate a sync round (export → exchange → import) across participants.
- Align frontend UI phases with server MultisigPhase model.
- Prepare for multi-instance deployment by moving challenges storage out of process.

## Out-of-Scope
- Full PGP/Tor transport implementation (will assume existing secure transport).
- Full Monero multisig parsing variations beyond our targeted version (will document extension points).

## Architecture Summary
- Server handlers
  - POST /api/escrow/{id}/multisig/challenge: creates and stores challenge (to migrate to Redis).
  - POST /api/escrow/{id}/multisig/prepare: receives {multisig_info, signature}, verifies signature against challenge, then persists validated info and transitions MultisigPhase.
  - POST /api/escrow/{id}/multisig/sync-round: New endpoint; triggers one sync round step for the local participant (export → exchange hook → import). Idempotent per round.
- Repositories
  - save_multisig_state(escrow_id, phase, snapshot): encrypt sensitive payloads; record timestamps; transactional updates.
  - load_multisig_state(escrow_id): used to render dashboard.
- Frontend
  - Map server phases to UI: NotStarted → init, Preparing → prepared, Exchanging(round) → awaiting_signatures, Ready → finalized, Failed → failed.

## Tasks (Jira candidates)
1) Security: Robust multisig_info parsing and key extraction
   - Replace simplified key extraction with Monero-compatible parser (monero-rust), reject malformed inputs.
   - Unit tests with valid/invalid fixtures.
2) Security: Binding + Anti-replay hardening
   - Ensure challenge single-use, TTL enforced, escrow/user binding persisted.
   - Migrate challenge store to Redis; add cleanup job.
3) Persistence: Save validated multisig_info and transition MultisigPhase
   - Implement DB writes post-validation: snapshot + phase transition to Preparing/Exchanging.
   - Encrypt sensitive fields; add migrations if needed.
4) API: Sync round endpoint
   - POST /api/escrow/{id}/multisig/sync-round, accepts payload for round context and exchange hook.
   - Idempotency keys; rate limiting; audit logs.
5) UI: Phase alignment
   - Align template `multisig-dashboard.html` phases with server model.
   - Add round indicator and failure reason display.
6) Observability
   - Metrics: count of challenges issued/consumed, invalid signatures, phase transitions; alerts for stuck states.
7) Docs & Reality checks
   - Update specs and reality-checks; make one source of truth for prepare_multisig verdicts.

## Risks & Mitigations
- Parsing mismatches with wallet RPC versions → pin versions and add compatibility layer.
- Multi-instance race conditions on state transitions → use DB transactions + Redis locks.
- Operator misuse of sync round → role-based auth and audit trail.

## Acceptance Criteria
- All new endpoints covered by integration tests and OpenAPI doc.
- TM-003 finding marked as resolved with evidence.
- Dashboard reflects accurate phases end-to-end.

## Rollout Plan
- Feature flag the new sync-round endpoint.
- Staging validation with 3 wallets RPC and scripts.
- Progressive rollout; monitor metrics and logs.
