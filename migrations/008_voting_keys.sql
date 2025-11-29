-- Migration 008: Privacy-Preserving Voting Keys
-- Adds voting key fields to veto_signals table for BIP32-derived privacy-preserving veto signals

-- Add voting key fields to veto_signals table
ALTER TABLE veto_signals ADD COLUMN voting_public_key TEXT;
ALTER TABLE veto_signals ADD COLUMN voting_key_path TEXT; -- Derivation path (e.g., "m/0'/123'/0'")
ALTER TABLE veto_signals ADD COLUMN signal_index INTEGER DEFAULT 0; -- Signal index for key derivation

-- Index for voting key lookups (for Merkle proof generation)
CREATE INDEX IF NOT EXISTS idx_veto_signals_voting_key ON veto_signals(voting_public_key);

-- Index for PR + signal_index lookups (for key derivation verification)
CREATE INDEX IF NOT EXISTS idx_veto_signals_pr_signal ON veto_signals(pr_id, signal_index);

