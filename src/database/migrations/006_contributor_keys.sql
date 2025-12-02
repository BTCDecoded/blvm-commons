-- Migration: Contributor Keys for Auto-Registration
-- Creates table to store public keys for Commons Contributors for automatic registration

CREATE TABLE IF NOT EXISTS contributor_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contributor_id TEXT NOT NULL UNIQUE,
    contributor_type TEXT NOT NULL,  -- 'merge_miner', 'fee_forwarder', 'zap_user'
    public_key TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_contributor_keys_id ON contributor_keys(contributor_id);
CREATE INDEX IF NOT EXISTS idx_contributor_keys_type ON contributor_keys(contributor_type);
CREATE INDEX IF NOT EXISTS idx_contributor_keys_pubkey ON contributor_keys(public_key);

