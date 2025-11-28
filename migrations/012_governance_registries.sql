-- Migration 012: Governance Registries
-- Creates table to track monthly governance registry hashes for OTS anchoring

CREATE TABLE IF NOT EXISTS governance_registries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    registry_hash TEXT NOT NULL UNIQUE,
    registry_path TEXT NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    month_year TEXT NOT NULL, -- Format: "YYYY-MM" for easy querying
    ots_proof_path TEXT, -- Path to OTS proof file if anchored
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Index for querying latest registry
CREATE INDEX IF NOT EXISTS idx_registries_timestamp ON governance_registries(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_registries_month_year ON governance_registries(month_year DESC);

