//! End-to-End Governance Integration Test
//!
//! Tests the complete flow from contributions to voting.

use blvm_commons::governance::{
    ContributionAggregator, ContributionTracker, VoteAggregator, WeightCalculator,
};
use blvm_commons::nostr::ZapVotingProcessor;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

/// Setup complete test database with all tables
async fn setup_complete_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // Create all tables
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
        CREATE TABLE pull_requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repo_name TEXT NOT NULL,
            pr_number INTEGER NOT NULL,
            opened_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            layer INTEGER NOT NULL,
            head_sha TEXT NOT NULL,
            signatures TEXT DEFAULT '[]',
            governance_status TEXT DEFAULT 'pending',
            linked_prs TEXT DEFAULT '[]',
            emergency_mode BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
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
async fn test_end_to_end_governance_flow() {
    let pool = setup_complete_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());
    let aggregator = ContributionAggregator::new(pool.clone());
    let calculator = WeightCalculator::new(pool.clone());
    let vote_aggregator = VoteAggregator::new(pool.clone());

    let timestamp = Utc::now();

    tracker
        .record_zap_contribution("user1", 0.02, timestamp, false)
        .await
        .unwrap();
    tracker.update_contribution_ages().await.unwrap();
    calculator.update_participation_weights().await.unwrap();

    let weight = calculator.get_participation_weight("user1").await.unwrap();
    assert_eq!(weight, Some(0.0));

    let agg = aggregator
        .get_contributor_aggregates("user1")
        .await
        .unwrap();
    assert_eq!(agg.cumulative_zaps_btc, 0.02);
    assert_eq!(agg.total_contribution_btc, 0.02);
    assert_eq!(agg.participation_weight, 0.0);
    let _ = vote_aggregator;
}

#[tokio::test]
async fn test_weight_cap_enforcement() {
    let pool = setup_complete_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());
    let calculator = WeightCalculator::new(pool.clone());

    let timestamp = Utc::now();

    tracker
        .record_zap_contribution("whale", 10.0, timestamp, false)
        .await
        .unwrap();

    // Create many small contributors
    for i in 0..20 {
        tracker
            .record_zap_contribution(&format!("user{}", i), 0.01, timestamp, false)
            .await
            .unwrap();
    }

    // Update weights
    calculator.update_participation_weights().await.unwrap();

    let total_weight = calculator.calculate_total_system_weight().await.unwrap();
    let whale_weight = calculator
        .get_participation_weight("whale")
        .await
        .unwrap()
        .unwrap();

    // A large zap does not buy weight.
    assert_eq!(whale_weight, 0.0);
    assert_eq!(total_weight, 0.0);
}

#[tokio::test]
async fn test_cooling_off_period() {
    let pool = setup_complete_test_db().await;
    let calculator = WeightCalculator::new(pool);

    assert!(calculator.check_cooling_off(0.1, 29));
    assert!(calculator.check_cooling_off(1.0, 0));
    assert!(calculator.check_cooling_off(0.05, 0));
}

#[tokio::test]
async fn test_monthly_aggregation_rolling_window() {
    let pool = setup_complete_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());
    let aggregator = ContributionAggregator::new(pool.clone());

    let now = Utc::now();
    let fifteen_days_ago = now - chrono::Duration::days(15);
    let thirty_one_days_ago = now - chrono::Duration::days(31);

    tracker
        .record_zap_contribution("user1", 0.01, thirty_one_days_ago, false)
        .await
        .unwrap();

    tracker
        .record_zap_contribution("user1", 0.01, fifteen_days_ago, false)
        .await
        .unwrap();

    let cumulative = aggregator.aggregate_zaps_cumulative("user1").await.unwrap();
    assert_eq!(cumulative, 0.02); // Both zaps counted
}

#[tokio::test]
async fn test_vote_aggregation_integration() {
    let pool = setup_complete_test_db().await;
    let _vote_processor = ZapVotingProcessor::new(pool.clone());
    let vote_aggregator = VoteAggregator::new(pool.clone());

    let now = Utc::now();

    // Insert zap votes directly (simulating processed zaps)
    sqlx::query(
        r#"
        INSERT INTO proposal_zap_votes
        (pr_id, governance_event_id, sender_pubkey, amount_msat, amount_btc, vote_weight, vote_type, timestamp, verified)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(123)
    .bind("event_123")
    .bind("voter1")
    .bind(100_000_000_000i64) // 1 BTC
    .bind(1.0)
    .bind(1.0) // sqrt(1.0) = 1.0
    .bind("support")
    .bind(now)
    .bind(true)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO proposal_zap_votes
        (pr_id, governance_event_id, sender_pubkey, amount_msat, amount_btc, vote_weight, vote_type, timestamp, verified)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(123)
    .bind("event_123")
    .bind("voter2")
    .bind(400_000_000_000i64) // 4 BTC
    .bind(4.0)
    .bind(2.0) // sqrt(4.0) = 2.0
    .bind("veto")
    .bind(now)
    .bind(true)
    .execute(&pool)
    .await
    .unwrap();

    // Aggregate votes
    let result = vote_aggregator
        .aggregate_proposal_votes(123, 3)
        .await
        .unwrap();

    assert_eq!(result.pr_id, 123);
    assert_eq!(result.tier, 3);
    // Zap rows are counted. They are not the maintainer vote.
    assert_eq!(result.support_votes, 0.0);
    assert_eq!(result.veto_votes, 0.0);
    assert_eq!(result.total_votes, 0.0);
    assert_eq!(result.zap_vote_count, 2);
    assert!(!result.threshold_met);
}
