-- Migration 007: Tier Overrides (PostgreSQL)
-- Allows maintainers to override automated tier classification with justification

CREATE TABLE tier_overrides (
  id SERIAL PRIMARY KEY,
  repo_name TEXT NOT NULL,
  pr_number INTEGER NOT NULL,
  override_tier INTEGER NOT NULL,
  justification TEXT NOT NULL,
  overridden_by TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(repo_name, pr_number)
);

CREATE INDEX idx_tier_overrides_pr ON tier_overrides(repo_name, pr_number);

