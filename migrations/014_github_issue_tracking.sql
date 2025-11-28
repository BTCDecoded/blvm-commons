-- Migration 014: Add GitHub issue number tracking for governance review cases
-- Enables linking cases to GitHub issues for notifications and updates

ALTER TABLE governance_review_cases
ADD COLUMN github_issue_number INTEGER;

-- Index for quick lookups by issue number
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_github_issue 
ON governance_review_cases(github_issue_number);

