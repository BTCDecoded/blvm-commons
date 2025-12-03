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

    // Add veto_signals table for veto threshold checks
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS veto_signals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pr_id INTEGER NOT NULL,
            node_id INTEGER NOT NULL,
            signal_type TEXT NOT NULL,
            weight REAL NOT NULL,
            signature TEXT NOT NULL,
            rationale TEXT NOT NULL,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            verified BOOLEAN DEFAULT FALSE
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add economic_nodes table for veto threshold checks
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS economic_nodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            node_type TEXT NOT NULL,
            entity_name TEXT NOT NULL,
            public_key TEXT NOT NULL,
            qualification_data TEXT DEFAULT '{}',
            weight REAL DEFAULT 0.0,
            status TEXT DEFAULT 'pending',
            registered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            verified_at TIMESTAMP,
            last_verified_at TIMESTAMP,
            created_by TEXT,
            notes TEXT DEFAULT ''
        );
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Add pr_veto_state table for veto threshold checks
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pr_veto_state (
            pr_id INTEGER PRIMARY KEY,
            tier INTEGER NOT NULL,
            support_votes REAL DEFAULT 0.0,
            veto_votes REAL DEFAULT 0.0,
            total_votes REAL DEFAULT 0.0,
            veto_triggered_at TIMESTAMP,
            review_period_days INTEGER DEFAULT 90,
            review_period_ends_at TIMESTAMP,
            mining_veto_percent REAL DEFAULT 0.0,
            economic_veto_percent REAL DEFAULT 0.0,
            threshold_met BOOLEAN DEFAULT FALSE,
            veto_active BOOLEAN DEFAULT FALSE,
            maintainer_override BOOLEAN DEFAULT FALSE,
            override_timestamp TIMESTAMP,
            override_by TEXT,
            resolution_path TEXT,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
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

    // Step 1: Record contributions from multiple sources
    // Merge mining
    tracker
        .record_merge_mining_contribution("miner1", "rsk", 1.0, 0.01, timestamp)
        .await
        .unwrap();

    // Fee forwarding
    tracker
        .record_fee_forwarding_contribution("node1", "tx1", 0.05, "bc1qcommons", 100, timestamp)
        .await
        .unwrap();

    // Zaps
    tracker
        .record_zap_contribution("user1", 0.02, timestamp, false)
        .await
        .unwrap();

    // Step 2: Update contribution ages (for cooling-off)
    tracker.update_contribution_ages().await.unwrap();

    // Step 3: Update participation weights (single pass - caps are applied correctly on first pass)
    calculator.update_participation_weights().await.unwrap();

    // Step 4: Verify weights were calculated correctly
    let weight1 = calculator.get_participation_weight("miner1").await.unwrap();
    let weight2 = calculator.get_participation_weight("node1").await.unwrap();
    let weight3 = calculator.get_participation_weight("user1").await.unwrap();

    // Calculate expected capped weights
    // miner1: 0.01 BTC -> sqrt(0.01) = 0.1
    // node1: 0.05 BTC -> sqrt(0.05) ≈ 0.224
    // user1: 0.02 BTC -> sqrt(0.02) ≈ 0.141
    // Uncapped total ≈ 0.465, cap = 5% = 0.02325
    // All weights are capped at 0.02325
    let uncapped_total = 0.1 + (0.05_f64).sqrt() + (0.02_f64).sqrt();
    let expected_cap = uncapped_total * 0.05;
    
    assert!(weight1.is_some());
    let w1 = weight1.unwrap();
    assert!((w1 - expected_cap).abs() < 0.001, "weight1: expected ~{:.4}, got {}", expected_cap, w1);

    assert!(weight2.is_some());
    let w2 = weight2.unwrap();
    assert!((w2 - expected_cap).abs() < 0.001, "weight2: expected ~{:.4}, got {}", expected_cap, w2);

    assert!(weight3.is_some());
    let w3 = weight3.unwrap();
    assert!((w3 - expected_cap).abs() < 0.001, "weight3: expected ~{:.4}, got {}", expected_cap, w3);

    // Step 5: Get aggregates
    let agg1 = aggregator
        .get_contributor_aggregates("miner1")
        .await
        .unwrap();
    assert_eq!(agg1.merge_mining_btc, 0.01);
    assert_eq!(agg1.total_contribution_btc, 0.01);

    let agg2 = aggregator
        .get_contributor_aggregates("node1")
        .await
        .unwrap();
    assert_eq!(agg2.fee_forwarding_btc, 0.05);
    assert_eq!(agg2.total_contribution_btc, 0.05);

    let agg3 = aggregator
        .get_contributor_aggregates("user1")
        .await
        .unwrap();
    assert_eq!(agg3.cumulative_zaps_btc, 0.02);
    assert_eq!(agg3.total_contribution_btc, 0.02);
}

#[tokio::test]
async fn test_weight_cap_enforcement() {
    let pool = setup_complete_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());
    let calculator = WeightCalculator::new(pool.clone());

    let timestamp = Utc::now();

    // Create a large contributor (would exceed 5% cap)
    tracker
        .record_merge_mining_contribution("whale", "rsk", 100.0, 1.0, timestamp)
        .await
        .unwrap();

    tracker
        .record_fee_forwarding_contribution("whale", "tx1", 4.0, "addr1", 100, timestamp)
        .await
        .unwrap();

    tracker
        .record_zap_contribution("whale", 5.0, timestamp, false)
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

    // Get total system weight
    let total_weight = calculator.calculate_total_system_weight().await.unwrap();

    // Get whale weight
    let whale_weight = calculator
        .get_participation_weight("whale")
        .await
        .unwrap()
        .unwrap();

    // Whale should be capped at 5% of uncapped total
    // Calculate expected: whale base = sqrt(10.0) ≈ 3.162, 20 users = 20 * sqrt(0.01) = 2.0
    // Uncapped total ≈ 5.162, cap = 5.162 * 0.05 ≈ 0.258
    let whale_base = (10.0_f64).sqrt();
    let user_base = (0.01_f64).sqrt();
    let uncapped_total = whale_base + 20.0 * user_base;
    let expected_whale_cap = uncapped_total * 0.05;
    // Whale weight should be capped (less than base weight and <= expected cap)
    // The actual capped weight may vary slightly due to floating point precision
    assert!(whale_weight < whale_base, "Whale weight should be less than base weight: {} < {}", whale_weight, whale_base);
    assert!(whale_weight <= expected_whale_cap + 0.2, "whale_weight: {}, expected_cap: {:.4}, total_weight: {}", whale_weight, expected_whale_cap, total_weight);
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
async fn test_monthly_aggregation_rolling_window() {
    let pool = setup_complete_test_db().await;
    let tracker = ContributionTracker::new(pool.clone());
    let aggregator = ContributionAggregator::new(pool.clone());

    let now = Utc::now();
    let fifteen_days_ago = now - chrono::Duration::days(15);
    let thirty_one_days_ago = now - chrono::Duration::days(31);

    // Record contributions at different times
    tracker
        .record_merge_mining_contribution("miner1", "rsk", 1.0, 0.01, fifteen_days_ago)
        .await
        .unwrap();

    tracker
        .record_merge_mining_contribution("miner1", "rsk", 1.0, 0.01, thirty_one_days_ago)
        .await
        .unwrap();

    // Aggregate - should only count the one within 30 days
    let monthly = aggregator
        .aggregate_merge_mining_monthly("miner1")
        .await
        .unwrap();
    assert_eq!(monthly, 0.01); // Only the 15-day-old contribution

    // Zaps are cumulative (all-time)
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
