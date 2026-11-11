//! Governance System Integration Tests
//!
//! Tests for contribution tracking, weight calculation, and voting aggregation.

use blvm_commons::governance::{
    ContributionAggregator, ContributionTracker, VoteAggregator, WeightCalculator,
};
use blvm_commons::nostr::{ZapTracker, ZapVotingProcessor};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::time::Duration;

/// Setup test database
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // Run migrations
    sqlx::query(
        r#"
        CREATE TABLE unified_contributions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contributor_id TEXT NOT NULL,
            contributor_type TEXT NOT NULL,
            contribution_type TEXT NOT NULL,
            amount_btc REAL NOT NULL,
            timestamp DATETIME NOT NULL,
            contribution_age_days INTEGER DEFAULT 0,
            period_type TEXT NOT NULL,
            verified BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE zap_contributions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient_pubkey TEXT NOT NULL,
            sender_pubkey TEXT,
            amount_msat INTEGER NOT NULL,
            amount_btc REAL NOT NULL,
            timestamp DATETIME NOT NULL,
            invoice_hash TEXT,
            message TEXT,
            zapped_event_id TEXT,
            is_proposal_zap BOOLEAN DEFAULT FALSE,
            governance_event_id TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE fee_forwarding_contributions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contributor_id TEXT NOT NULL,
            tx_hash TEXT NOT NULL,
            block_height INTEGER NOT NULL,
            amount_btc REAL NOT NULL,
            commons_address TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            verified BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(tx_hash)
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE participation_weights (
            contributor_id TEXT PRIMARY KEY,
            contributor_type TEXT NOT NULL,
            merge_mining_btc REAL DEFAULT 0.0,
            fee_forwarding_btc REAL DEFAULT 0.0,
            cumulative_zaps_btc REAL DEFAULT 0.0,
            total_contribution_btc REAL NOT NULL,
            base_weight REAL NOT NULL,
            capped_weight REAL NOT NULL,
            total_system_weight REAL NOT NULL,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE proposal_zap_votes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pr_id INTEGER NOT NULL,
            governance_event_id TEXT NOT NULL,
            sender_pubkey TEXT NOT NULL,
            amount_msat INTEGER NOT NULL,
            amount_btc REAL NOT NULL,
            vote_weight REAL NOT NULL,
            vote_type TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            verified BOOLEAN DEFAULT FALSE
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

#[tokio::test]
async fn test_contribution_tracker_zap() {
    let pool = setup_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());

    let timestamp = Utc::now();

    // Record zap contribution
    tracker
        .record_zap_contribution(
            "user_pubkey_123",
            0.001, // 0.001 BTC zapped
            timestamp,
            false, // Not a proposal zap
        )
        .await
        .unwrap();

    // Verify it was recorded
    let total = tracker
        .get_contributor_total(
            "user_pubkey_123",
            timestamp - chrono::Duration::days(1),
            timestamp + chrono::Duration::days(1),
        )
        .await
        .unwrap();

    assert_eq!(total.zaps_btc, 0.001);
    assert_eq!(total.total_btc, 0.001);
}

#[tokio::test]
async fn test_weight_calculator_cap() {
    let pool = setup_test_db().await;
    let calculator = WeightCalculator::new(pool.clone());

    // Since dabfdbf governance is maintainer-only and apply_weight_cap is a
    // reporting stub: the weight is returned unchanged (no 5% cap).
    let total_system_weight = 100.0;
    assert_eq!(calculator.apply_weight_cap(10.0, total_system_weight), 10.0);
    assert_eq!(calculator.apply_weight_cap(2.0, total_system_weight), 2.0);
}

#[tokio::test]
async fn test_weight_calculator_cooling_off() {
    let pool = setup_test_db().await;
    let calculator = WeightCalculator::new(pool.clone());

    // Since dabfdbf governance is maintainer-only and check_cooling_off is a
    // reporting stub: no cooling-off period for any contribution.
    assert!(calculator.check_cooling_off(0.1, 29));
    assert!(calculator.check_cooling_off(0.1, 30));
    assert!(calculator.check_cooling_off(0.1, 31));
    assert!(calculator.check_cooling_off(0.05, 0));
    assert!(calculator.check_cooling_off(0.09, 1));
}

#[tokio::test]
async fn test_zap_voting_processor() {
    let pool = setup_test_db().await;
    let processor = ZapVotingProcessor::new(pool.clone());

    // This test would require actual zap events, so we'll test the vote type parsing
    use blvm_commons::nostr::VoteType;

    assert_eq!(VoteType::from_str("support"), VoteType::Support);
    assert_eq!(VoteType::from_str("veto"), VoteType::Veto);
    assert_eq!(VoteType::from_str("abstain"), VoteType::Abstain);
    assert_eq!(VoteType::from_str("unknown"), VoteType::Support); // Default
}

#[tokio::test]
async fn test_vote_aggregator_thresholds() {
    let pool = setup_test_db().await;
    let aggregator = VoteAggregator::new(pool.clone());

    // Test tier thresholds
    assert_eq!(aggregator.get_threshold_for_tier(1).unwrap(), 100);
    assert_eq!(aggregator.get_threshold_for_tier(2).unwrap(), 500);
    assert_eq!(aggregator.get_threshold_for_tier(3).unwrap(), 1_000);
    assert_eq!(aggregator.get_threshold_for_tier(4).unwrap(), 2_500);
    assert_eq!(aggregator.get_threshold_for_tier(5).unwrap(), 5_000);

    // Invalid tier
    assert!(aggregator.get_threshold_for_tier(6).is_err());
}
