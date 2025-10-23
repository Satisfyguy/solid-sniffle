-- Create disputes table for tracking escrow disputes
CREATE TABLE IF NOT EXISTS disputes (
    id TEXT PRIMARY KEY NOT NULL,
    escrow_id TEXT NOT NULL,
    buyer_username TEXT NOT NULL,
    vendor_username TEXT NOT NULL,
    opened_by TEXT NOT NULL CHECK (opened_by IN ('buyer', 'vendor')),
    reason TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('open', 'under_review', 'resolved', 'manual_review', 'closed')),
    evidence TEXT NOT NULL DEFAULT '[]', -- JSON array
    resolution TEXT, -- Resolution decision (JSON)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Index for fast lookups by escrow
CREATE INDEX IF NOT EXISTS idx_disputes_escrow_id ON disputes(escrow_id);

-- Index for status queries
CREATE INDEX IF NOT EXISTS idx_disputes_status ON disputes(status);

-- Index for finding disputes by user
CREATE INDEX IF NOT EXISTS idx_disputes_buyer ON disputes(buyer_username);
CREATE INDEX IF NOT EXISTS idx_disputes_vendor ON disputes(vendor_username);
