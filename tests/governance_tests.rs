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

    // Test 5% cap
    let total_system_weight = 100.0;
    let calculated_weight = 10.0; // Would be 10% without cap
    let capped_weight = calculator.apply_weight_cap(calculated_weight, total_system_weight);

    assert_eq!(capped_weight, 5.0); // Capped at 5% of 100 = 5.0

    // Test weight below cap
    let small_weight = 2.0;
    let capped_small = calculator.apply_weight_cap(small_weight, total_system_weight);
    assert_eq!(capped_small, 2.0); // No cap applied
}

#[tokio::test]
async fn test_weight_calculator_cooling_off() {
    let pool = setup_test_db().await;
    let calculator = WeightCalculator::new(pool.clone());

    // Test cooling-off: contributions >= 0.1 BTC need 30 days
    assert!(!calculator.check_cooling_off(0.1, 29)); // 29 days, not eligible
    assert!(calculator.check_cooling_off(0.1, 30)); // 30 days, eligible
    assert!(calculator.check_cooling_off(0.1, 31)); // 31 days, eligible

    // Test small contributions: no cooling-off
    assert!(calculator.check_cooling_off(0.05, 0)); // Small, no cooling-off
    assert!(calculator.check_cooling_off(0.09, 1)); // Small, no cooling-off
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
