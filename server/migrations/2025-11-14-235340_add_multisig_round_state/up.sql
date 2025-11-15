-- Table pour persister l'état multisig par round
-- Permet recovery après crash: si Round 1 complete, skip to Round 2

CREATE TABLE multisig_round_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    escrow_id TEXT NOT NULL,
    round_number INTEGER NOT NULL, -- 1, 2, 3
    status TEXT NOT NULL, -- 'pending', 'in_progress', 'completed', 'failed'

    -- RPC tracking pour per-wallet locking
    rpc_url TEXT NOT NULL,
    wallet_filename TEXT NOT NULL,
    role TEXT NOT NULL, -- 'buyer', 'vendor', 'arbiter'

    -- Multisig info JSON (pour recovery)
    multisig_info TEXT, -- JSON array des prepare_multisig outputs

    -- Timestamps
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    last_error TEXT,

    -- Contrainte: un seul état par (escrow, round, role)
    UNIQUE(escrow_id, round_number, role),

    FOREIGN KEY (escrow_id) REFERENCES escrows(id) ON DELETE CASCADE
);

-- Index pour lookups rapides
CREATE INDEX idx_multisig_round_escrow ON multisig_round_state(escrow_id);
CREATE INDEX idx_multisig_round_status ON multisig_round_state(status);
CREATE INDEX idx_multisig_round_lookup ON multisig_round_state(escrow_id, round_number);
