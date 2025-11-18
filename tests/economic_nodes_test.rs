//! Economic Node Infrastructure Tests
//!
//! Tests for economic node registration, qualification verification,
//! weight calculation, veto signal collection, and threshold calculation

use chrono::Utc;
use bllvm_commons::database::Database;
use bllvm_commons::economic_nodes::{registry::EconomicNodeRegistry, types::*, veto::VetoManager};
use bllvm_commons::error::GovernanceError;
use bllvm_commons::crypto::signatures::SignatureManager;
use bllvm_sdk::governance::GovernanceKeypair;
use sqlx::SqlitePool;

#[tokio::test]
async fn test_economic_node_registration() -> Result<(), Box<dyn std::error::Error>> {
    // Setup in-memory database
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool);

    // Test mining pool registration
    let mining_pool_proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string(), "block2".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 5.0,
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Test Mining Pool".to_string(),
            contact_email: "test@mining.com".to_string(),
            website: Some("https://mining.com".to_string()),
            github_username: None,
        },
    };

    let node_id = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Test Mining Pool",
            "test_public_key_1",
            &mining_pool_proof,
            Some("test_admin"),
        )
        .await?;

    assert!(node_id > 0);
    println!("✅ Mining pool registered with ID: {}", node_id);

    // Test exchange registration
    use bllvm_commons::economic_nodes::types::{HashpowerProof, HoldingsProof, VolumeProof};
    let exchange_proof = QualificationProof {
        node_type: NodeType::Exchange,
        hashpower_proof: None,
        holdings_proof: Some(HoldingsProof {
            addresses: vec!["addr1".to_string()],
            total_btc: 15_000.0, // Above 10,000 BTC threshold
            signature_challenge: "sig1".to_string(),
        }),
        volume_proof: Some(VolumeProof {
            daily_volume_usd: 150_000_000.0, // Above $100M daily threshold
            monthly_volume_usd: 4_500_000_000.0,
            data_source: "test".to_string(),
            verification_url: None,
        }),
        contact_info: ContactInfo {
            entity_name: "Test Exchange".to_string(),
            contact_email: "test@exchange.com".to_string(),
            website: Some("https://exchange.com".to_string()),
            github_username: None,
        },
    };

    let exchange_id = registry
        .register_economic_node(
            NodeType::Exchange,
            "Test Exchange",
            "test_public_key_2",
            &exchange_proof,
            Some("test_admin"),
        )
        .await?;

    assert!(exchange_id > 0);
    println!("✅ Exchange registered with ID: {}", exchange_id);

    // Test custodian registration
    let custodian_proof = QualificationProof {
        node_type: NodeType::Custodian,
        hashpower_proof: None,
        holdings_proof: Some(HoldingsProof {
            addresses: vec!["addr2".to_string()],
            total_btc: 15_000.0, // Above 10,000 BTC threshold
            signature_challenge: "sig2".to_string(),
        }),
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Test Custodian".to_string(),
            contact_email: "test@custodian.com".to_string(),
            website: Some("https://custodian.com".to_string()),
            github_username: None,
        },
    };

    let custodian_id = registry
        .register_economic_node(
            NodeType::Custodian,
            "Test Custodian",
            "test_public_key_3",
            &custodian_proof,
            Some("test_admin"),
        )
        .await?;

    assert!(custodian_id > 0);
    println!("✅ Custodian registered with ID: {}", custodian_id);

    Ok(())
}

#[tokio::test]
async fn test_qualification_verification() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool);

    // Test insufficient mining pool qualification
    let insufficient_mining_proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 0.5, // Below 1% threshold
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Insufficient Mining Pool".to_string(),
            contact_email: "test@insufficient.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let result = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Insufficient Mining Pool",
            "test_public_key_4",
            &insufficient_mining_proof,
            Some("test_admin"),
        )
        .await;

    assert!(result.is_err());
    println!("✅ Insufficient mining pool correctly rejected");

    // Test insufficient exchange qualification
    let insufficient_exchange_proof = QualificationProof {
        node_type: NodeType::Exchange,
        hashpower_proof: None,
        holdings_proof: Some(HoldingsProof {
            addresses: vec!["addr3".to_string()],
            total_btc: 500.0, // Below 1000 BTC threshold
            signature_challenge: "sig3".to_string(),
        }),
        volume_proof: Some(VolumeProof {
            daily_volume_usd: 5_000_000.0, // Below $10M threshold
            monthly_volume_usd: 150_000_000.0,
            data_source: "test".to_string(),
            verification_url: None,
        }),
        contact_info: ContactInfo {
            entity_name: "Insufficient Exchange".to_string(),
            contact_email: "test@insufficient.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let result = registry
        .register_economic_node(
            NodeType::Exchange,
            "Insufficient Exchange",
            "test_public_key_5",
            &insufficient_exchange_proof,
            Some("test_admin"),
        )
        .await;

    assert!(result.is_err());
    println!("✅ Insufficient exchange correctly rejected");

    Ok(())
}

#[tokio::test]
async fn test_weight_calculation() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool);

    // Test mining pool weight calculation
    let mining_proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 10.0,
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Test Mining Pool".to_string(),
            contact_email: "test@mining.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let weight = registry
        .calculate_weight(NodeType::MiningPool, &mining_proof)
        .await?;
    assert!(weight > 0.0 && weight <= 1.0); // Weight is hashpower percentage / 100.0
    assert_eq!(weight, 0.1); // 10% hashpower = 0.1 weight
    println!("✅ Mining pool weight calculated: {}", weight);

    // Test exchange weight calculation
    let exchange_proof = QualificationProof {
        node_type: NodeType::Exchange,
        hashpower_proof: None,
        holdings_proof: Some(HoldingsProof {
            addresses: vec!["addr4".to_string()],
            total_btc: 15_000.0, // Above 10,000 BTC threshold
            signature_challenge: "sig4".to_string(),
        }),
        volume_proof: Some(VolumeProof {
            daily_volume_usd: 150_000_000.0, // Above $100M daily threshold
            monthly_volume_usd: 4_500_000_000.0,
            data_source: "test".to_string(),
            verification_url: None,
        }),
        contact_info: ContactInfo {
            entity_name: "Test Exchange".to_string(),
            contact_email: "test@exchange.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let weight = registry
        .calculate_weight(NodeType::Exchange, &exchange_proof)
        .await?;
    assert!(weight > 0.0 && weight <= 1.0); // Weight is normalized 0-1 (70% holdings + 30% volume)
    assert!(weight >= 0.9); // With 15K BTC and $150M daily, should be close to 1.0
    println!("✅ Exchange weight calculated: {}", weight);

    Ok(())
}

#[tokio::test]
async fn test_veto_signal_collection() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool.clone());
    let veto_manager = VetoManager::new(pool);

    // Register a mining pool
    let mining_proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 5.0,
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Test Node".to_string(),
            contact_email: "test@test.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let node_id = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Test Mining Pool",
            "test_public_key_1",
            &mining_proof,
            Some("test_admin"),
        )
        .await?;

    // Activate the node so it can submit veto signals
    registry.update_node_status(node_id, NodeStatus::Active).await?;

    // Create valid signature for veto signal
    let signature_manager = SignatureManager::new();
    let keypair = GovernanceKeypair::generate()?;
    let public_key = keypair.public_key().to_string();
    
    // Update the node's public key to match the generated keypair
    let pool = db.pool().expect("Database should have SQLite pool");
    sqlx::query("UPDATE economic_nodes SET public_key = ? WHERE id = ?")
        .bind(&public_key)
        .bind(node_id)
        .execute(pool)
        .await?;
    
    let message = format!("PR #1 veto signal from Test Mining Pool");
    let signature = signature_manager
        .create_governance_signature(&message, &keypair)?;

    // Submit a veto signal
    let signal_id = veto_manager
        .collect_veto_signal(
            1, // PR ID
            node_id,
            SignalType::Veto,
            &signature,
            "This change threatens network security",
        )
        .await?;

    assert!(signal_id > 0);
    println!("✅ Veto signal submitted with ID: {}", signal_id);

    // Test duplicate signal rejection
    let message2 = format!("PR #1 veto signal from Test Mining Pool");
    let signature2 = signature_manager
        .create_governance_signature(&message2, &keypair)?;
    
    let result = veto_manager
        .collect_veto_signal(
            1, // Same PR ID
            node_id,
            SignalType::Support,
            &signature2,
            "Changed my mind",
        )
        .await;

    assert!(result.is_err());
    println!("✅ Duplicate signal correctly rejected");

    Ok(())
}

#[tokio::test]
async fn test_veto_threshold_calculation() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool.clone());
    let veto_manager = VetoManager::new(pool);

    // Register multiple nodes with different weights
    let mining_proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 20.0,
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Large Mining Pool".to_string(),
            contact_email: "test@large.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let mining_node_id = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Large Mining Pool",
            "test_public_key_1",
            &mining_proof,
            Some("test_admin"),
        )
        .await?;

    let exchange_proof = QualificationProof {
        node_type: NodeType::Exchange,
        hashpower_proof: None,
        holdings_proof: Some(HoldingsProof {
            addresses: vec!["addr6".to_string()],
            total_btc: 15_000.0, // Above 10,000 BTC threshold
            signature_challenge: "sig6".to_string(),
        }),
        volume_proof: Some(VolumeProof {
            daily_volume_usd: 150_000_000.0, // Above $100M daily threshold
            monthly_volume_usd: 4_500_000_000.0,
            data_source: "test".to_string(),
            verification_url: None,
        }),
        contact_info: ContactInfo {
            entity_name: "Large Exchange".to_string(),
            contact_email: "test@large.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let exchange_node_id = registry
        .register_economic_node(
            NodeType::Exchange,
            "Large Exchange",
            "test_public_key_2",
            &exchange_proof,
            Some("test_admin"),
        )
        .await?;

    // Activate nodes so they can submit veto signals
    registry.update_node_status(mining_node_id, NodeStatus::Active).await?;
    registry.update_node_status(exchange_node_id, NodeStatus::Active).await?;

    // Create valid signatures for veto signals
    let signature_manager = SignatureManager::new();
    let mining_keypair = GovernanceKeypair::generate()?;
    let exchange_keypair = GovernanceKeypair::generate()?;
    let mining_public_key = mining_keypair.public_key().to_string();
    let exchange_public_key = exchange_keypair.public_key().to_string();
    
    // Update nodes' public keys to match generated keypairs
    let pool = db.pool().expect("Database should have SQLite pool");
    sqlx::query("UPDATE economic_nodes SET public_key = ? WHERE id = ?")
        .bind(&mining_public_key)
        .bind(mining_node_id)
        .execute(pool)
        .await?;
    sqlx::query("UPDATE economic_nodes SET public_key = ? WHERE id = ?")
        .bind(&exchange_public_key)
        .bind(exchange_node_id)
        .execute(pool)
        .await?;
    
    let mining_message = format!("PR #1 veto signal from Large Mining Pool");
    let exchange_message = format!("PR #1 veto signal from Large Exchange");
    let mining_signature = signature_manager
        .create_governance_signature(&mining_message, &mining_keypair)?;
    let exchange_signature = signature_manager
        .create_governance_signature(&exchange_message, &exchange_keypair)?;

    // Submit veto signals
    veto_manager
        .collect_veto_signal(
            1, // PR ID
            mining_node_id,
            SignalType::Veto,
            &mining_signature,
            "Mining pool veto",
        )
        .await?;

    veto_manager
        .collect_veto_signal(
            1, // PR ID
            exchange_node_id,
            SignalType::Veto,
            &exchange_signature,
            "Exchange veto",
        )
        .await?;

    // Check veto threshold
    let threshold = veto_manager.check_veto_threshold(1).await?;

    // Should have veto active due to high weights
    assert!(threshold.veto_active);
    println!(
        "✅ Veto threshold correctly calculated: mining={}%, economic={}%, active={}",
        threshold.mining_veto_percent, threshold.economic_veto_percent, threshold.veto_active
    );

    Ok(())
}

#[tokio::test]
async fn test_node_status_management() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool);

    // Register a node
    let proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 5.0,
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Test Node".to_string(),
            contact_email: "test@test.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let node_id = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Test Node",
            "test_public_key",
            &proof,
            Some("test_admin"),
        )
        .await?;

    // Update node status to active
    registry
        .update_node_status(node_id, NodeStatus::Active)
        .await?;
    println!("✅ Node status updated to active");

    // Get active nodes
    let active_nodes = registry.get_active_nodes().await?;
    assert_eq!(active_nodes.len(), 1);
    assert_eq!(active_nodes[0].entity_name, "Test Node");
    println!("✅ Active nodes retrieved: {}", active_nodes.len());

    // Update to inactive
    registry
        .update_node_status(node_id, NodeStatus::Suspended)
        .await?;
    let active_nodes = registry.get_active_nodes().await?;
    assert_eq!(active_nodes.len(), 0);
    println!("✅ Node status updated to inactive");

    Ok(())
}

#[tokio::test]
async fn test_weight_recalculation() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool);

    // Register multiple nodes
    let proof1 = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 5.0,
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Test Node".to_string(),
            contact_email: "test@test.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let proof2 = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 10.0,
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Test Node".to_string(),
            contact_email: "test@test.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    registry
        .register_economic_node(
            NodeType::MiningPool,
            "Small Pool",
            "test_public_key_1",
            &proof1,
            Some("test_admin"),
        )
        .await?;

    registry
        .register_economic_node(
            NodeType::MiningPool,
            "Large Pool",
            "test_public_key_2",
            &proof2,
            Some("test_admin"),
        )
        .await?;

    // Recalculate all weights
    registry.recalculate_all_weights().await?;
    println!("✅ All node weights recalculated");

    Ok(())
}

#[tokio::test]
async fn test_veto_statistics() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool.clone());
    let veto_manager = VetoManager::new(pool);

    // Register nodes
    let mining_proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 15.0,
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Test Node".to_string(),
            contact_email: "test@test.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let exchange_proof = QualificationProof {
        node_type: NodeType::Exchange,
        hashpower_proof: None,
        holdings_proof: Some(HoldingsProof {
            addresses: vec!["addr1".to_string()],
            total_btc: 15_000.0, // Above 10,000 BTC threshold
            signature_challenge: "sig1".to_string(),
        }),
        volume_proof: Some(VolumeProof {
            daily_volume_usd: 150_000_000.0, // Above $100M daily threshold
            monthly_volume_usd: 4_500_000_000.0,
            data_source: "test".to_string(),
            verification_url: None,
        }),
        contact_info: ContactInfo {
            entity_name: "Test Exchange".to_string(),
            contact_email: "test@exchange.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let mining_node_id = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Mining Pool",
            "test_public_key_1",
            &mining_proof,
            Some("test_admin"),
        )
        .await?;

    let exchange_node_id = registry
        .register_economic_node(
            NodeType::Exchange,
            "Exchange",
            "test_public_key_2",
            &exchange_proof,
            Some("test_admin"),
        )
        .await?;

    // Activate nodes so they can submit veto signals
    registry.update_node_status(mining_node_id, NodeStatus::Active).await?;
    registry.update_node_status(exchange_node_id, NodeStatus::Active).await?;

    // Create valid signatures for veto signals
    let signature_manager = SignatureManager::new();
    let mining_keypair = GovernanceKeypair::generate()?;
    let exchange_keypair = GovernanceKeypair::generate()?;
    let mining_public_key = mining_keypair.public_key().to_string();
    let exchange_public_key = exchange_keypair.public_key().to_string();
    
    // Update nodes' public keys to match generated keypairs
    let pool = db.pool().expect("Database should have SQLite pool");
    sqlx::query("UPDATE economic_nodes SET public_key = ? WHERE id = ?")
        .bind(&mining_public_key)
        .bind(mining_node_id)
        .execute(pool)
        .await?;
    sqlx::query("UPDATE economic_nodes SET public_key = ? WHERE id = ?")
        .bind(&exchange_public_key)
        .bind(exchange_node_id)
        .execute(pool)
        .await?;
    
    let mining_message = format!("PR #1 veto signal from Mining Pool");
    let exchange_message = format!("PR #1 veto signal from Exchange");
    let mining_signature = signature_manager
        .create_governance_signature(&mining_message, &mining_keypair)?;
    let exchange_signature = signature_manager
        .create_governance_signature(&exchange_message, &exchange_keypair)?;

    // Submit different types of signals
    veto_manager
        .collect_veto_signal(
            1,
            mining_node_id,
            SignalType::Veto,
            &mining_signature,
            "Mining veto",
        )
        .await?;

    veto_manager
        .collect_veto_signal(
            1,
            exchange_node_id,
            SignalType::Support,
            &exchange_signature,
            "Exchange support",
        )
        .await?;

    // Get veto statistics
    let statistics = veto_manager.get_veto_statistics(1).await?;

    let signals = statistics.get("signals").unwrap();
    assert!(signals.get("total").unwrap().as_u64().unwrap() > 0);
    assert!(signals.get("veto").unwrap().as_u64().unwrap() > 0);
    assert!(signals.get("support").unwrap().as_u64().unwrap() > 0);

    println!("✅ Veto statistics retrieved: {:?}", statistics);

    Ok(())
}
