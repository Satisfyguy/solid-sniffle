-- Create immutable audit log table
-- This table implements a hash chain for tamper detection
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL CHECK (event_type IN (
        'arbitration_attempt',
        'resolution',
        'signing',
        'key_rotation',
        'manual_review'
    )),
    entity_id TEXT NOT NULL, -- Dispute ID, escrow ID, or other entity
    data TEXT NOT NULL, -- Event data (JSON)
    timestamp TEXT NOT NULL,
    entry_hash TEXT NOT NULL UNIQUE, -- SHA3-256 hash of this entry
    previous_hash TEXT, -- Hash of previous entry (NULL for first entry)
    actor TEXT NOT NULL, -- Username or 'system'
    FOREIGN KEY (previous_hash) REFERENCES audit_log(entry_hash)
);

-- Index for entity lookups
CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_id);

-- Index for timeline queries
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp);

-- Index for event type filtering
CREATE INDEX IF NOT EXISTS idx_audit_log_event_type ON audit_log(event_type);

-- Index for hash chain verification
CREATE INDEX IF NOT EXISTS idx_audit_log_hash ON audit_log(entry_hash);
