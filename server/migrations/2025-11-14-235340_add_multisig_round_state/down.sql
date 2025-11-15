-- Rollback multisig round state tracking
DROP INDEX IF EXISTS idx_multisig_round_lookup;
DROP INDEX IF EXISTS idx_multisig_round_status;
DROP INDEX IF EXISTS idx_multisig_round_escrow;
DROP TABLE IF EXISTS multisig_round_state;
