-- Migration 006: Sequential Veto Mechanism
-- Adds support for sequential two-phase veto mechanism with maintainer override

-- Add columns to veto_signals table for sequential mechanism
ALTER TABLE veto_signals ADD COLUMN review_period_start TIMESTAMP;
ALTER TABLE veto_signals ADD COLUMN review_period_days INTEGER DEFAULT 90;
ALTER TABLE veto_signals ADD COLUMN maintainer_override BOOLEAN DEFAULT FALSE;
ALTER TABLE veto_signals ADD COLUMN override_timestamp TIMESTAMP;
ALTER TABLE veto_signals ADD COLUMN override_by TEXT; -- GitHub username who overrode

-- Create table to track veto state per PR
CREATE TABLE IF NOT EXISTS pr_veto_state (
  pr_id INTEGER PRIMARY KEY,
  veto_triggered_at TIMESTAMP NOT NULL,
  review_period_days INTEGER DEFAULT 90,
  review_period_ends_at TIMESTAMP NOT NULL,
  mining_veto_percent REAL DEFAULT 0.0,
  economic_veto_percent REAL DEFAULT 0.0,
  threshold_met BOOLEAN DEFAULT FALSE,
  veto_active BOOLEAN DEFAULT FALSE,
  maintainer_override BOOLEAN DEFAULT FALSE,
  override_timestamp TIMESTAMP,
  override_by TEXT,
  resolution_path TEXT, -- 'consensus', 'override', 'dissolution'
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (pr_id) REFERENCES pull_requests(id)
);

CREATE INDEX IF NOT EXISTS idx_pr_veto_state_active ON pr_veto_state(veto_active, review_period_ends_at);
CREATE INDEX IF NOT EXISTS idx_pr_veto_state_override ON pr_veto_state(maintainer_override);

