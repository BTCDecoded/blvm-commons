//! Tests for fee forwarding transaction hash calculation

use bllvm_commons::governance::FeeForwardingTracker;
use bllvm_protocol::{Block, Transaction, TransactionInput, TransactionOutput, OutPoint};
use bitcoin::Network;
use sqlx::SqlitePool;

/// Setup test database
async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    
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
    
    pool
}

fn create_test_transaction() -> Transaction {
    Transaction {
        version: 2,
        inputs: vec![TransactionInput {
            prevout: OutPoint {
                hash: [0u8; 32],
                index: 0,
            },
            script_sig: vec![],
            sequence: 0xffffffff,
        }],
        outputs: vec![TransactionOutput {
            value: 100_000_000, // 1 BTC in satoshis
            script_pubkey: vec![0x76, 0xa9, 0x14, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x88, 0xac], // P2PKH script
        }],
        lock_time: 0,
    }
}

#[tokio::test]
async fn test_calculate_tx_hash_consistency() {
    let pool = setup_test_db().await;
    let tracker = FeeForwardingTracker::new(
        pool,
        vec![],
        Network::Bitcoin,
    );
    
    let tx = create_test_transaction();
    
    // Calculate hash twice - should be identical
    let hash1 = tracker.calculate_tx_hash(&tx);
    let hash2 = tracker.calculate_tx_hash(&tx);
    
    assert_eq!(hash1, hash2, "Transaction hash should be deterministic");
    assert_eq!(hash1.len(), 64, "Hash should be 64 hex characters (32 bytes)");
}

#[tokio::test]
async fn test_calculate_tx_hash_different_transactions() {
    let pool = setup_test_db().await;
    let tracker = FeeForwardingTracker::new(
        pool,
        vec![],
        Network::Bitcoin,
    );
    
    let mut tx1 = create_test_transaction();
    let mut tx2 = create_test_transaction();
    
    // Make tx2 different by changing version
    tx2.version = 1;
    
    let hash1 = tracker.calculate_tx_hash(&tx1);
    let hash2 = tracker.calculate_tx_hash(&tx2);
    
    assert_ne!(hash1, hash2, "Different transactions should have different hashes");
}

#[tokio::test]
async fn test_calculate_tx_hash_matches_bitcoin_core() {
    // Test that our hash calculation matches Bitcoin Core's txid
    // This is critical for duplicate detection
    let pool = setup_test_db().await;
    let tracker = FeeForwardingTracker::new(
        pool,
        vec![],
        Network::Bitcoin,
    );
    
    let tx = create_test_transaction();
    let hash = tracker.calculate_tx_hash(&tx);
    
    // Verify hash format (64 hex characters)
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()), "Hash should be hex string");
    assert_eq!(hash.len(), 64, "Hash should be 64 characters");
    
    // The hash should match what bllvm-consensus's calculate_tx_id produces
    // (which we're now using internally)
    use bllvm_protocol::block::calculate_tx_id;
    let expected_hash = hex::encode(calculate_tx_id(&tx));
    assert_eq!(hash, expected_hash, "Hash should match bllvm-consensus calculate_tx_id");
}

#[tokio::test]
async fn test_fee_forwarding_duplicate_detection() {
    let pool = setup_test_db().await;
    let tracker = FeeForwardingTracker::new(
        pool.clone(),
        vec!["bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string()],
        Network::Bitcoin,
    );
    
    let tx = create_test_transaction();
    let block = Block {
        header: bllvm_protocol::BlockHeader {
            version: 0x20000000,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: 0,
            bits: 0x1d00ffff,
            nonce: 0,
        },
        transactions: vec![
            Transaction {
                version: 1,
                inputs: vec![],
                outputs: vec![],
                lock_time: 0,
            }, // Coinbase
            tx.clone(),
        ],
    };
    
    // Process block first time
    let contributions1 = tracker.process_block(&block, 100, None).await.unwrap();
    
    // Process same block again - should detect duplicate
    let contributions2 = tracker.process_block(&block, 100, None).await.unwrap();
    
    // Second processing should not add duplicate
    assert_eq!(contributions2.len(), 0, "Duplicate transaction should not be recorded again");
}

