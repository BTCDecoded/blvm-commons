//! Nostr Integration Tests
//!
//! Tests for zap tracking and voting functionality.

use bllvm_commons::nostr::{VoteType, ZapVotingProcessor};
use chrono::Utc;
use sqlx::SqlitePool;

/// Setup test database for Nostr tests
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

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
async fn test_vote_type_parsing() {
    assert_eq!(VoteType::from_str("support"), VoteType::Support);
    assert_eq!(VoteType::from_str("Support"), VoteType::Support);
    assert_eq!(VoteType::from_str("SUPPORT"), VoteType::Support);

    assert_eq!(VoteType::from_str("veto"), VoteType::Veto);
    assert_eq!(VoteType::from_str("Veto"), VoteType::Veto);
    assert_eq!(VoteType::from_str("oppose"), VoteType::Veto);
    assert_eq!(VoteType::from_str("against"), VoteType::Veto);

    assert_eq!(VoteType::from_str("abstain"), VoteType::Abstain);
    assert_eq!(VoteType::from_str("neutral"), VoteType::Abstain);

    // Default to support
    assert_eq!(VoteType::from_str("unknown"), VoteType::Support);
    assert_eq!(VoteType::from_str(""), VoteType::Support);
}

#[tokio::test]
async fn test_vote_type_to_string() {
    assert_eq!(VoteType::Support.as_str(), "support");
    assert_eq!(VoteType::Veto.as_str(), "veto");
    assert_eq!(VoteType::Abstain.as_str(), "abstain");
}

#[tokio::test]
async fn test_zap_voting_processor_vote_type_parsing() {
    use bllvm_commons::nostr::zap_voting::ZapVotingProcessor;

    // Test message parsing
    let processor = ZapVotingProcessor::new(setup_test_db().await);

    // These are private methods, but we can test the public interface
    // by checking vote type strings
    assert_eq!(VoteType::from_str("veto"), VoteType::Veto);
    assert_eq!(VoteType::from_str("oppose"), VoteType::Veto);
    assert_eq!(VoteType::from_str("against"), VoteType::Veto);
}

#[tokio::test]
async fn test_zap_vote_weight_calculation() {
    let pool = setup_test_db().await;
    let processor = ZapVotingProcessor::new(pool.clone());

    // Insert a test zap vote
    sqlx::query(
        r#"
        INSERT INTO proposal_zap_votes
        (pr_id, governance_event_id, sender_pubkey, amount_msat, amount_btc, vote_weight, vote_type, timestamp, verified)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(1)
    .bind("event_123")
    .bind("sender_pubkey")
    .bind(100_000_000_000i64) // 1 BTC in msat
    .bind(1.0)
    .bind(1.0) // sqrt(1.0) = 1.0
    .bind("support")
    .bind(Utc::now())
    .bind(true)
    .execute(&pool)
    .await
    .unwrap();

    // Get votes
    let votes = processor.get_proposal_votes(1).await.unwrap();
    assert_eq!(votes.len(), 1);
    assert_eq!(votes[0].amount_btc, 1.0);
    assert_eq!(votes[0].vote_weight, 1.0);
    assert_eq!(votes[0].vote_type, VoteType::Support);
}

#[tokio::test]
async fn test_zap_vote_totals() {
    let pool = setup_test_db().await;
    let processor = ZapVotingProcessor::new(pool.clone());

    let now = Utc::now();

    // Insert multiple votes
    sqlx::query(
        r#"
        INSERT INTO proposal_zap_votes
        (pr_id, governance_event_id, sender_pubkey, amount_msat, amount_btc, vote_weight, vote_type, timestamp, verified)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(1)
    .bind("event_123")
    .bind("sender1")
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
    .bind(1)
    .bind("event_123")
    .bind("sender2")
    .bind(400_000_000_000i64) // 4 BTC
    .bind(4.0)
    .bind(2.0) // sqrt(4.0) = 2.0
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
    .bind(1)
    .bind("event_123")
    .bind("sender3")
    .bind(100_000_000_000i64) // 1 BTC
    .bind(1.0)
    .bind(1.0) // sqrt(1.0) = 1.0
    .bind("veto")
    .bind(now)
    .bind(true)
    .execute(&pool)
    .await
    .unwrap();

    // Get totals
    let totals = processor.get_proposal_vote_totals(1).await.unwrap();

    assert_eq!(totals.support_weight, 3.0); // 1.0 + 2.0
    assert_eq!(totals.veto_weight, 1.0);
    assert_eq!(totals.total_weight, 4.0); // 3.0 + 1.0
    assert_eq!(totals.support_count, 2);
    assert_eq!(totals.veto_count, 1);
    assert_eq!(totals.total_count, 3);
}
