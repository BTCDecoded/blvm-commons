-- Governance Review Tracking Schema
-- Implements graduated sanctions, time limits, and protections from governance review policy

-- Governance review cases
CREATE TABLE IF NOT EXISTS governance_review_cases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_number TEXT UNIQUE NOT NULL, -- Format: GR-YYYY-MMDD-NNNN
    subject_maintainer_id INTEGER NOT NULL,
    reporter_maintainer_id INTEGER NOT NULL,
    case_type TEXT NOT NULL, -- 'abuse', 'harassment', 'malicious_code', 'collusion', 'conflict_of_interest', 'technical_errors', 'security_violation', 'false_report', 'retaliation'
    severity TEXT NOT NULL, -- 'minor', 'moderate', 'serious', 'gross_misconduct'
    status TEXT NOT NULL DEFAULT 'open', -- 'open', 'under_review', 'mediation', 'warning_issued', 'removal_pending', 'removed', 'resolved', 'dismissed', 'expired'
    description TEXT NOT NULL,
    evidence JSON NOT NULL DEFAULT '{}', -- Links, screenshots, etc.
    on_platform BOOLEAN NOT NULL DEFAULT true, -- Policy: only on-platform considered
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    response_deadline TIMESTAMP, -- 30 days from creation
    resolution_deadline TIMESTAMP, -- 180 days from creation
    resolved_at TIMESTAMP,
    resolution_reason TEXT,
    
    FOREIGN KEY (subject_maintainer_id) REFERENCES maintainers(id),
    FOREIGN KEY (reporter_maintainer_id) REFERENCES maintainers(id),
    CHECK (status IN ('open', 'under_review', 'mediation', 'warning_issued', 'removal_pending', 'removed', 'resolved', 'dismissed', 'expired')),
    CHECK (severity IN ('minor', 'moderate', 'serious', 'gross_misconduct')),
    CHECK (case_type IN ('abuse', 'harassment', 'malicious_code', 'collusion', 'conflict_of_interest', 'technical_errors', 'security_violation', 'false_report', 'retaliation'))
);

-- Subject responses
CREATE TABLE IF NOT EXISTS governance_review_responses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id INTEGER NOT NULL,
    maintainer_id INTEGER NOT NULL,
    response_text TEXT NOT NULL,
    counter_evidence JSON DEFAULT '{}',
    submitted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    FOREIGN KEY (case_id) REFERENCES governance_review_cases(id),
    FOREIGN KEY (maintainer_id) REFERENCES maintainers(id)
);

-- Graduated sanctions (warnings)
CREATE TABLE IF NOT EXISTS governance_review_warnings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id INTEGER NOT NULL,
    maintainer_id INTEGER NOT NULL,
    warning_level INTEGER NOT NULL, -- 1 = private, 2 = public
    warning_type TEXT NOT NULL, -- 'private_warning', 'public_warning'
    issued_by_team_approval INTEGER NOT NULL, -- Number of maintainers who approved (4-of-7 for private, 5-of-7 for public)
    issued_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    improvement_deadline TIMESTAMP, -- 90 days from issue
    improvement_extended BOOLEAN DEFAULT false,
    improvement_extended_until TIMESTAMP,
    resolved BOOLEAN DEFAULT false,
    resolved_at TIMESTAMP,
    warning_file_path TEXT, -- Path to governance/warnings/ file for public warnings
    
    FOREIGN KEY (case_id) REFERENCES governance_review_cases(id),
    FOREIGN KEY (maintainer_id) REFERENCES maintainers(id),
    CHECK (warning_level IN (1, 2)),
    CHECK (warning_type IN ('private_warning', 'public_warning'))
);

-- Sanction approvals (who approved the warning/removal)
CREATE TABLE IF NOT EXISTS governance_review_sanction_approvals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id INTEGER NOT NULL,
    maintainer_id INTEGER NOT NULL, -- Who approved
    sanction_type TEXT NOT NULL, -- 'private_warning', 'public_warning', 'removal'
    approved_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    signature TEXT, -- Cryptographic signature of approval
    
    FOREIGN KEY (case_id) REFERENCES governance_review_cases(id),
    FOREIGN KEY (maintainer_id) REFERENCES maintainers(id),
    CHECK (sanction_type IN ('private_warning', 'public_warning', 'removal'))
);

-- Mediation attempts
CREATE TABLE IF NOT EXISTS governance_review_mediation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id INTEGER NOT NULL,
    mediator_maintainer_id INTEGER, -- Optional neutral maintainer
    mediation_started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    mediation_deadline TIMESTAMP, -- 30 days from start
    status TEXT NOT NULL DEFAULT 'active', -- 'active', 'resolved', 'failed', 'skipped'
    resolution_notes TEXT,
    resolved_at TIMESTAMP,
    
    FOREIGN KEY (case_id) REFERENCES governance_review_cases(id),
    FOREIGN KEY (mediator_maintainer_id) REFERENCES maintainers(id),
    CHECK (status IN ('active', 'resolved', 'failed', 'skipped'))
);

-- Appeals
CREATE TABLE IF NOT EXISTS governance_review_appeals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id INTEGER NOT NULL,
    maintainer_id INTEGER NOT NULL,
    appeal_reason TEXT NOT NULL,
    new_evidence JSON DEFAULT '{}',
    submitted_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    appeal_deadline TIMESTAMP, -- 60 days from submission
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'under_review', 'granted', 'denied', 'expired'
    reviewed_at TIMESTAMP,
    review_decision TEXT,
    teams_approval_count INTEGER, -- Number of teams that approved (for overturning)
    
    FOREIGN KEY (case_id) REFERENCES governance_review_cases(id),
    FOREIGN KEY (maintainer_id) REFERENCES maintainers(id),
    CHECK (status IN ('pending', 'under_review', 'granted', 'denied', 'expired'))
);

-- Retaliation tracking (protection for reporters)
CREATE TABLE IF NOT EXISTS governance_review_retaliation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_case_id INTEGER NOT NULL,
    reporter_maintainer_id INTEGER NOT NULL,
    retaliator_maintainer_id INTEGER NOT NULL,
    retaliation_type TEXT NOT NULL, -- 'threat', 'exclusion', 'harassment', 'other'
    description TEXT NOT NULL,
    reported_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'open', -- 'open', 'under_review', 'confirmed', 'dismissed'
    confirmed_at TIMESTAMP,
    
    FOREIGN KEY (original_case_id) REFERENCES governance_review_cases(id),
    FOREIGN KEY (reporter_maintainer_id) REFERENCES maintainers(id),
    FOREIGN KEY (retaliator_maintainer_id) REFERENCES maintainers(id),
    CHECK (status IN ('open', 'under_review', 'confirmed', 'dismissed')),
    CHECK (retaliation_type IN ('threat', 'exclusion', 'harassment', 'other'))
);

-- False report tracking
CREATE TABLE IF NOT EXISTS governance_review_false_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_case_id INTEGER NOT NULL,
    false_reporter_maintainer_id INTEGER NOT NULL,
    confirmed_false_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    false_report_evidence TEXT NOT NULL,
    sanction_applied TEXT, -- 'warning', 'removal', 'none'
    sanction_case_id INTEGER, -- Case ID for the false reporter's sanction
    
    FOREIGN KEY (original_case_id) REFERENCES governance_review_cases(id),
    FOREIGN KEY (false_reporter_maintainer_id) REFERENCES maintainers(id),
    FOREIGN KEY (sanction_case_id) REFERENCES governance_review_cases(id),
    CHECK (sanction_applied IN ('warning', 'removal', 'none'))
);

-- Time limit tracking (ensures cases resolve within 180 days)
CREATE TABLE IF NOT EXISTS governance_review_time_limits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id INTEGER NOT NULL,
    limit_type TEXT NOT NULL, -- 'response', 'resolution', 'appeal', 'improvement'
    deadline TIMESTAMP NOT NULL,
    extended BOOLEAN DEFAULT false,
    extension_approved_by INTEGER, -- Maintainer ID who approved extension
    extension_reason TEXT,
    extension_until TIMESTAMP,
    
    FOREIGN KEY (case_id) REFERENCES governance_review_cases(id),
    FOREIGN KEY (extension_approved_by) REFERENCES maintainers(id),
    CHECK (limit_type IN ('response', 'resolution', 'appeal', 'improvement'))
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_subject ON governance_review_cases(subject_maintainer_id);
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_reporter ON governance_review_cases(reporter_maintainer_id);
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_status ON governance_review_cases(status);
CREATE INDEX IF NOT EXISTS idx_governance_review_cases_resolution_deadline ON governance_review_cases(resolution_deadline);
CREATE INDEX IF NOT EXISTS idx_governance_review_warnings_maintainer ON governance_review_warnings(maintainer_id);
CREATE INDEX IF NOT EXISTS idx_governance_review_warnings_case ON governance_review_warnings(case_id);
CREATE INDEX IF NOT EXISTS idx_governance_review_appeals_case ON governance_review_appeals(case_id);
CREATE INDEX IF NOT EXISTS idx_governance_review_appeals_status ON governance_review_appeals(status);
CREATE INDEX IF NOT EXISTS idx_governance_review_retaliation_reporter ON governance_review_retaliation(reporter_maintainer_id);
CREATE INDEX IF NOT EXISTS idx_governance_review_retaliation_retaliator ON governance_review_retaliation(retaliator_maintainer_id);
CREATE INDEX IF NOT EXISTS idx_governance_review_time_limits_deadline ON governance_review_time_limits(deadline);

