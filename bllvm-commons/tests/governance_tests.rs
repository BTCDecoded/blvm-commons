//! Governance System Integration Tests
//!
//! Tests for contribution tracking, weight calculation, and voting aggregation.

use bllvm_commons::governance::{
    ContributionAggregator, ContributionTracker, FeeForwardingTracker, VoteAggregator,
    WeightCalculator,
};
use bllvm_commons::nostr::{ZapTracker, ZapVotingProcessor};
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
async fn test_contribution_tracker_merge_mining() {
    let pool = setup_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());

    let timestamp = Utc::now();

    // Record merge mining contribution
    tracker
        .record_merge_mining_contribution(
            "miner1", "rsk", 1.0,  // 1 BTC reward
            0.01, // 0.01 BTC contribution (1%)
            timestamp,
        )
        .await
        .unwrap();

    // Verify it was recorded
    let total = tracker
        .get_contributor_total(
            "miner1",
            timestamp - chrono::Duration::days(1),
            timestamp + chrono::Duration::days(1),
        )
        .await
        .unwrap();

    assert_eq!(total.merge_mining_btc, 0.01);
    assert_eq!(total.total_btc, 0.01);
}

#[tokio::test]
async fn test_contribution_tracker_fee_forwarding() {
    let pool = setup_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());

    let timestamp = Utc::now();

    // Record fee forwarding contribution
    tracker
        .record_fee_forwarding_contribution(
            "node1",
            "tx_hash_123",
            0.05, // 0.05 BTC forwarded
            "bc1qcommons",
            100,
            timestamp,
        )
        .await
        .unwrap();

    // Verify it was recorded in both tables
    let total = tracker
        .get_contributor_total(
            "node1",
            timestamp - chrono::Duration::days(1),
            timestamp + chrono::Duration::days(1),
        )
        .await
        .unwrap();

    assert_eq!(total.fee_forwarding_btc, 0.05);
    assert_eq!(total.total_btc, 0.05);

    // Verify it's in fee_forwarding_contributions table
    let fee_forwarding: Option<(String, f64)> = sqlx::query_as(
        "SELECT tx_hash, amount_btc FROM fee_forwarding_contributions WHERE contributor_id = ?",
    )
    .bind("node1")
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(fee_forwarding.is_some());
    let (tx_hash, amount) = fee_forwarding.unwrap();
    assert_eq!(tx_hash, "tx_hash_123");
    assert_eq!(amount, 0.05);
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
async fn test_weight_calculator_quadratic() {
    let pool = setup_test_db().await;
    let calculator = WeightCalculator::new(pool.clone());

    // Test quadratic formula: sqrt(total_btc)
    let weight1 = calculator.calculate_participation_weight(1.0, 0.0, 0.0);
    assert!((weight1 - 1.0).abs() < 0.0001); // sqrt(1.0) = 1.0

    let weight4 = calculator.calculate_participation_weight(4.0, 0.0, 0.0);
    assert!((weight4 - 2.0).abs() < 0.0001); // sqrt(4.0) = 2.0

    let weight9 = calculator.calculate_participation_weight(9.0, 0.0, 0.0);
    assert!((weight9 - 3.0).abs() < 0.0001); // sqrt(9.0) = 3.0
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
async fn test_weight_calculator_update_weights() {
    let pool = setup_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());
    let calculator = WeightCalculator::new(pool.clone());

    let timestamp = Utc::now();

    // Record contributions
    tracker
        .record_merge_mining_contribution("contributor1", "rsk", 1.0, 0.01, timestamp)
        .await
        .unwrap();

    tracker
        .record_fee_forwarding_contribution("contributor1", "tx1", 0.04, "addr1", 100, timestamp)
        .await
        .unwrap();

    tracker
        .record_zap_contribution("contributor1", 0.05, timestamp, false)
        .await
        .unwrap();

    // Update weights
    calculator.update_participation_weights().await.unwrap();

    // Verify weight was calculated
    let weight = calculator
        .get_participation_weight("contributor1")
        .await
        .unwrap();
    assert!(weight.is_some());

    // Total contribution = 0.01 + 0.04 + 0.05 = 0.10 BTC
    // Weight = sqrt(0.10) ≈ 0.316
    let expected_weight = (0.10_f64).sqrt();
    let actual_weight = weight.unwrap();
    assert!((actual_weight - expected_weight).abs() < 0.01);
}

#[tokio::test]
async fn test_aggregator_monthly_aggregation() {
    let pool = setup_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());
    let aggregator = ContributionAggregator::new(pool.clone());

    let now = Utc::now();
    let thirty_days_ago = now - chrono::Duration::days(30);
    let thirty_one_days_ago = now - chrono::Duration::days(31);

    // Record merge mining (within 30 days)
    tracker
        .record_merge_mining_contribution(
            "miner1",
            "rsk",
            1.0,
            0.01,
            now - chrono::Duration::days(15),
        )
        .await
        .unwrap();

    // Record merge mining (outside 30 days - should not be counted)
    tracker
        .record_merge_mining_contribution("miner1", "rsk", 1.0, 0.01, thirty_one_days_ago)
        .await
        .unwrap();

    // Aggregate
    let monthly = aggregator
        .aggregate_merge_mining_monthly("miner1")
        .await
        .unwrap();

    // Should only count the one within 30 days
    assert_eq!(monthly, 0.01);
}

#[tokio::test]
async fn test_zap_voting_processor() {
    let pool = setup_test_db().await;
    let processor = ZapVotingProcessor::new(pool.clone());

    // This test would require actual zap events, so we'll test the vote type parsing
    use bllvm_commons::nostr::VoteType;

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

#[tokio::test]
async fn test_integration_full_flow() {
    let pool = setup_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());
    let aggregator = ContributionAggregator::new(pool.clone());
    let calculator = WeightCalculator::new(pool.clone());

    let timestamp = Utc::now();

    // 1. Record multiple contribution types
    tracker
        .record_merge_mining_contribution("contributor1", "rsk", 1.0, 0.01, timestamp)
        .await
        .unwrap();

    tracker
        .record_fee_forwarding_contribution("contributor1", "tx1", 0.04, "addr1", 100, timestamp)
        .await
        .unwrap();

    tracker
        .record_zap_contribution("contributor1", 0.05, timestamp, false)
        .await
        .unwrap();

    // 2. Update contribution ages
    tracker.update_contribution_ages().await.unwrap();

    // 3. Update weights
    calculator.update_participation_weights().await.unwrap();

    // 4. Get aggregates
    let aggregates = aggregator
        .get_contributor_aggregates("contributor1")
        .await
        .unwrap();

    assert_eq!(aggregates.merge_mining_btc, 0.01);
    assert_eq!(aggregates.fee_forwarding_btc, 0.04);
    assert_eq!(aggregates.cumulative_zaps_btc, 0.05);
    assert_eq!(aggregates.total_contribution_btc, 0.10);

    // Weight should be sqrt(0.10) ≈ 0.316
    let expected_weight = (0.10_f64).sqrt();
    assert!((aggregates.participation_weight - expected_weight).abs() < 0.01);
}
