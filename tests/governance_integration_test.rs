//! End-to-End Governance Integration Test
//!
//! Tests the complete flow from contributions to voting.

use blvm_commons::governance::{VoteAggregator, WeightCalculator};
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
async fn test_cooling_off_period() {
    let pool = setup_complete_test_db().await;
    let calculator = WeightCalculator::new(pool);

    // Test cooling-off logic
    // Large contribution (>= 0.1 BTC) needs 30 days
    assert!(!calculator.check_cooling_off(0.1, 29));
    assert!(calculator.check_cooling_off(0.1, 30));
    assert!(calculator.check_cooling_off(0.1, 31));
    assert!(calculator.check_cooling_off(1.0, 30));

    // Small contribution (< 0.1 BTC) has no cooling-off
    assert!(calculator.check_cooling_off(0.05, 0));
    assert!(calculator.check_cooling_off(0.09, 1));
    assert!(calculator.check_cooling_off(0.099, 29));
}

#[tokio::test]
async fn test_vote_aggregation_integration() {
    let pool = setup_complete_test_db().await;
    let vote_processor = ZapVotingProcessor::new(pool.clone());
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
    assert_eq!(result.support_votes, 1.0);
    assert_eq!(result.veto_votes, 2.0);
    assert_eq!(result.total_votes, 3.0);
    assert_eq!(result.zap_vote_count, 2);
}
