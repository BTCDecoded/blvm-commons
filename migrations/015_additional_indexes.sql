-- Migration 015: Add additional indexes for governance review performance
-- Improves query performance for time-based queries and filtering

-- Index for time-based queries and sorting
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_created_at 
ON governance_review_cases(created_at DESC);

-- Indexes for filtering by case type and severity
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_type 
ON governance_review_cases(case_type);

CREATE INDEX IF NOT EXISTS idx_governance_review_cases_severity 
ON governance_review_cases(severity);

-- Composite index for expired case queries (optimization)
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_status_deadline 
ON governance_review_cases(status, resolution_deadline);

-- Explicit index for case number (redundant with UNIQUE but makes intent clear)
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_number 
ON governance_review_cases(case_number);

