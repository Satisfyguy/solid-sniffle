-- Create arbitration decisions table
-- Stores detailed arbitration analysis and decisions
CREATE TABLE IF NOT EXISTS arbitration_decisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    dispute_id TEXT NOT NULL UNIQUE,
    resolution_type TEXT NOT NULL CHECK (resolution_type IN (
        'release_to_vendor',
        'refund_to_buyer',
        'split',
        'manual_review'
    )),
    reasoning TEXT NOT NULL,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    evidence_quality REAL NOT NULL CHECK (evidence_quality >= 0.0 AND evidence_quality <= 1.0),
    manual_review_required INTEGER NOT NULL CHECK (manual_review_required IN (0, 1)),
    decided_at TEXT NOT NULL,
    decided_by TEXT NOT NULL, -- 'system' or admin username
    FOREIGN KEY (dispute_id) REFERENCES disputes(id)
);

-- Index for dispute lookups
CREATE INDEX IF NOT EXISTS idx_arbitration_decisions_dispute ON arbitration_decisions(dispute_id);

-- Index for manual review queue
CREATE INDEX IF NOT EXISTS idx_arbitration_decisions_manual ON arbitration_decisions(manual_review_required, decided_at);
