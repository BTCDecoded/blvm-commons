-- Migration: Governance Contributions
-- Creates tables for tracking zap contributions, merge mining, fee forwarding, and participation weights

-- Zap Contributions Table
CREATE TABLE IF NOT EXISTS zap_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient_pubkey TEXT NOT NULL,
    sender_pubkey TEXT,
    amount_msat INTEGER NOT NULL,
    amount_btc REAL NOT NULL,
    timestamp DATETIME NOT NULL,
    invoice_hash TEXT,
    message TEXT,
    zapped_event_id TEXT,  -- Event being zapped (for proposal zaps)
    is_proposal_zap BOOLEAN DEFAULT FALSE,
    governance_event_id TEXT,  -- If zapping a governance proposal
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_zap_recipient ON zap_contributions(recipient_pubkey);
CREATE INDEX IF NOT EXISTS idx_zap_sender ON zap_contributions(sender_pubkey);
CREATE INDEX IF NOT EXISTS idx_zap_timestamp ON zap_contributions(timestamp);
CREATE INDEX IF NOT EXISTS idx_zap_governance ON zap_contributions(governance_event_id);
CREATE INDEX IF NOT EXISTS idx_zap_recipient_time ON zap_contributions(recipient_pubkey, timestamp);

-- Unified Contributions Table
CREATE TABLE IF NOT EXISTS unified_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contributor_id TEXT NOT NULL,
    contributor_type TEXT NOT NULL,  -- 'merge_miner', 'fee_forwarder', 'zap_user'
    contribution_type TEXT NOT NULL,  -- 'merge_mining', 'fee_forwarding', 'zap'
    amount_btc REAL NOT NULL,
    timestamp DATETIME NOT NULL,
    contribution_age_days INTEGER DEFAULT 0,  -- For cooling-off check
    period_type TEXT NOT NULL,  -- 'monthly', 'cumulative'
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_contributor ON unified_contributions(contributor_id);
CREATE INDEX IF NOT EXISTS idx_contributor_type ON unified_contributions(contributor_type);
CREATE INDEX IF NOT EXISTS idx_timestamp ON unified_contributions(timestamp);
CREATE INDEX IF NOT EXISTS idx_contributor_time ON unified_contributions(contributor_id, timestamp);

-- Fee Forwarding Contributions Table
CREATE TABLE IF NOT EXISTS fee_forwarding_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contributor_id TEXT NOT NULL,  -- Miner/node identifier
    tx_hash TEXT NOT NULL,  -- Transaction hash
    block_height INTEGER NOT NULL,
    amount_btc REAL NOT NULL,  -- Amount forwarded to Commons address
    commons_address TEXT NOT NULL,  -- Commons address that received funds
    timestamp DATETIME NOT NULL,
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tx_hash)  -- Prevent duplicate tracking
);

CREATE INDEX IF NOT EXISTS idx_fee_forwarding_contributor ON fee_forwarding_contributions(contributor_id);
CREATE INDEX IF NOT EXISTS idx_fee_forwarding_timestamp ON fee_forwarding_contributions(timestamp);
CREATE INDEX IF NOT EXISTS idx_fee_forwarding_contributor_time ON fee_forwarding_contributions(contributor_id, timestamp);

-- Participation Weights Table
CREATE TABLE IF NOT EXISTS participation_weights (
    contributor_id TEXT PRIMARY KEY,
    contributor_type TEXT NOT NULL,
    merge_mining_btc REAL DEFAULT 0.0,
    fee_forwarding_btc REAL DEFAULT 0.0,
    cumulative_zaps_btc REAL DEFAULT 0.0,
    total_contribution_btc REAL NOT NULL,
    base_weight REAL NOT NULL,  -- sqrt(total_contribution_btc)
    capped_weight REAL NOT NULL,  -- After 5% cap applied
    total_system_weight REAL NOT NULL,  -- For cap calculation
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Proposal Zap Votes Table
CREATE TABLE IF NOT EXISTS proposal_zap_votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pr_id INTEGER NOT NULL,
    governance_event_id TEXT NOT NULL,
    sender_pubkey TEXT NOT NULL,
    amount_msat INTEGER NOT NULL,
    amount_btc REAL NOT NULL,
    vote_weight REAL NOT NULL,  -- sqrt(amount_btc)
    vote_type TEXT NOT NULL,  -- 'support', 'veto', 'abstain'
    timestamp DATETIME NOT NULL,
    verified BOOLEAN DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_proposal_zap_pr ON proposal_zap_votes(pr_id);
CREATE INDEX IF NOT EXISTS idx_proposal_zap_event ON proposal_zap_votes(governance_event_id);
CREATE INDEX IF NOT EXISTS idx_proposal_zap_sender ON proposal_zap_votes(sender_pubkey);

