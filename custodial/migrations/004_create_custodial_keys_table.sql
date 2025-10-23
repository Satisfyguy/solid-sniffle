-- Create custodial keys table
-- Tracks key rotation history
CREATE TABLE IF NOT EXISTS custodial_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key TEXT NOT NULL UNIQUE,
    key_type TEXT NOT NULL DEFAULT 'ed25519',
    status TEXT NOT NULL CHECK (status IN ('active', 'rotated', 'deprecated')),
    created_at TEXT NOT NULL,
    rotated_at TEXT,
    backup_location TEXT, -- Encrypted backup location (for disaster recovery)
    notes TEXT -- Admin notes about key rotation
);

-- Index for active key lookup
CREATE INDEX IF NOT EXISTS idx_custodial_keys_status ON custodial_keys(status);

-- Index for key rotation history
CREATE INDEX IF NOT EXISTS idx_custodial_keys_created ON custodial_keys(created_at);
