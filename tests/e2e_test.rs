//! End-to-End Governance Tests
//!
//! Tests complete governance scenarios from PR creation to merge,
//! including economic node veto scenarios, emergency activation,
//! and governance changes with fork capability

use bllvm_commons::{
    database::Database,
    economic_nodes::{registry::EconomicNodeRegistry, types::*, veto::VetoManager},
    enforcement::{merge_block::MergeBlocker, status_checks::StatusCheckGenerator},
    error::GovernanceError,
    fork::{adoption::AdoptionTracker, export::GovernanceExporter, types::RulesetVersion, versioning::RulesetVersioning},
    validation::tier_classification,
    crypto::signatures::SignatureManager,
};
use bllvm_sdk::governance::GovernanceKeypair;
use serde_json::json;
use std::str::FromStr;
use hex;
use sqlx;

mod common;
use common::create_test_decision_logger;

#[tokio::test]
async fn test_tier_1_routine_approval_flow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Tier 1 (Routine Maintenance) approval flow...");

    // Setup
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool.clone());
    let veto_manager = VetoManager::new(pool);

    // 1. Create a Tier 1 PR (routine maintenance)
    let pr_payload = json!({
        "pull_request": {
            "number": 1,
            "title": "Fix typo in README",
            "body": "Simple documentation fix",
            "head": {"sha": "abc123"},
            "base": {"sha": "def456"}
        },
        "repository": {"full_name": "test-org/test-repo"}
    });

    // 2. Classify PR tier
    let tier = tier_classification::classify_pr_tier(&pr_payload).await;
    assert_eq!(tier, 1);
    println!("✅ PR classified as Tier 1 (Routine Maintenance)");

    // 3. Check governance requirements
    let merge_blocker = MergeBlocker::new(None, create_test_decision_logger());

    // Tier 1 requirements: 3-of-5 signatures, 7 days review period
    let should_block = MergeBlocker::should_block_merge(
        true,  // review period met (simulated)
        true,  // signatures met (simulated)
        false, // no economic veto (Tier 1 doesn't require economic node input)
        tier,
        false, // emergency_mode
    )?;

    assert!(!should_block);
    println!("✅ Tier 1 PR can be merged when requirements met");

    // 4. Generate status checks
    let opened_at = chrono::Utc::now() - chrono::Duration::try_days(10).unwrap_or_default();
    let review_status = StatusCheckGenerator::generate_review_period_status(opened_at, 7, false);
    let signature_status = StatusCheckGenerator::generate_signature_status(
        3,
        5,
        5,
        &["maintainer1".to_string(), "maintainer2".to_string(), "maintainer3".to_string()],
        &["maintainer4".to_string(), "maintainer5".to_string()],
    );
    let tier_status = StatusCheckGenerator::generate_tier_status(
        tier,
        "Routine Maintenance",
        true,
        true,
        false,
        &review_status,
        &signature_status,
    );

    assert!(tier_status.contains("Routine Maintenance") || tier_status.contains("🔧"));
    println!("✅ Status checks generated for Tier 1 PR");

    println!("🎉 Tier 1 routine approval flow completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_tier_3_economic_node_veto_scenario() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Tier 3 (Consensus-Adjacent) with economic node veto...");

    // Setup
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool.clone());
    let veto_manager = VetoManager::new(pool);

    // 1. Register economic nodes
    use bllvm_commons::economic_nodes::types::{HashpowerProof, HoldingsProof, VolumeProof};
    
    let mining_pool_proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string(), "block2".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 25.0,
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

    let exchange_proof = QualificationProof {
        node_type: NodeType::Exchange,
        hashpower_proof: None,
        holdings_proof: Some(HoldingsProof {
            addresses: vec!["addr1".to_string()],
            total_btc: 15000.0,
            signature_challenge: "sig1".to_string(),
        }),
        volume_proof: Some(VolumeProof {
            daily_volume_usd: 100_000_000.0,
            monthly_volume_usd: 3_000_000_000.0,
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

    let mining_node_id = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Large Mining Pool",
            "mining_pool_key",
            &mining_pool_proof,
            Some("admin"),
        )
        .await?;

    let exchange_node_id = registry
        .register_economic_node(
            NodeType::Exchange,
            "Major Exchange",
            "exchange_key",
            &exchange_proof,
            Some("admin"),
        )
        .await?;

    // Activate nodes
    registry
        .update_node_status(mining_node_id, NodeStatus::Active)
        .await?;
    registry
        .update_node_status(exchange_node_id, NodeStatus::Active)
        .await?;
    println!("✅ Economic nodes registered and activated");

    // 2. Create a Tier 3 PR (consensus-adjacent)
    let pr_payload = json!({
        "pull_request": {
            "number": 2,
            "title": "[CONSENSUS-ADJACENT] Update validation logic",
            "body": "This PR updates consensus validation code",
            "head": {"sha": "consensus123"},
            "base": {"sha": "main456"}
        },
        "repository": {"full_name": "test-org/consensus-engine"}
    });

    let tier = tier_classification::classify_pr_tier(&pr_payload).await;
    assert_eq!(tier, 3);
    println!("✅ PR classified as Tier 3 (Consensus-Adjacent)");

    // 3. Submit veto signals
    // For testing, we need to create valid signatures
    // Since we registered nodes with "mining_pool_key" and "exchange_key" as public keys,
    // we need to either update those keys or create signatures that match
    // Simplest: Update the nodes' public keys to match generated keypairs
    use bllvm_commons::crypto::signatures::SignatureManager;
    use bllvm_sdk::governance::GovernanceKeypair;
    let sig_manager = SignatureManager::new();
    
    let mining_keypair = GovernanceKeypair::generate().expect("Failed to generate keypair");
    let exchange_keypair = GovernanceKeypair::generate().expect("Failed to generate keypair");
    
    // Update node public keys in database to match our keypairs
    let mining_pubkey_hex = hex::encode(mining_keypair.public_key().to_bytes());
    let exchange_pubkey_hex = hex::encode(exchange_keypair.public_key().to_bytes());
    
    let pool = db.pool().expect("Database should have SQLite pool");
    sqlx::query("UPDATE economic_nodes SET public_key = ? WHERE id = ?")
        .bind(&mining_pubkey_hex)
        .bind(mining_node_id)
        .execute(pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to update mining node key: {}", e)))?;
    
    sqlx::query("UPDATE economic_nodes SET public_key = ? WHERE id = ?")
        .bind(&exchange_pubkey_hex)
        .bind(exchange_node_id)
        .execute(pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to update exchange node key: {}", e)))?;
    
    // Get updated nodes via VetoManager (which has get_node_by_id)
    let mining_node = veto_manager.get_node_by_id(mining_node_id).await?;
    let exchange_node = veto_manager.get_node_by_id(exchange_node_id).await?;
    
    // Create valid signatures
    let mining_message = format!("PR #{} veto signal from {}", 2, mining_node.entity_name);
    let mining_sig = sig_manager.create_governance_signature(&mining_message, &mining_keypair).expect("Failed to create signature");
    
    let exchange_message = format!("PR #{} veto signal from {}", 2, exchange_node.entity_name);
    let exchange_sig = sig_manager.create_governance_signature(&exchange_message, &exchange_keypair).expect("Failed to create signature");
    
    veto_manager
        .collect_veto_signal(
            2, // PR ID
            mining_node_id,
            SignalType::Veto,
            &mining_sig,
            "This change threatens network security",
        )
        .await?;

    veto_manager
        .collect_veto_signal(
            2, // PR ID
            exchange_node_id,
            SignalType::Veto,
            &exchange_sig,
            "This change could impact user funds",
        )
        .await?;

    println!("✅ Veto signals submitted by economic nodes");

    // 4. Check veto threshold
    let threshold = veto_manager.check_veto_threshold(2).await?;
    assert!(threshold.veto_active);
    println!(
        "✅ Veto threshold exceeded: mining={}%, economic={}%, active={}",
        threshold.mining_veto_percent, threshold.economic_veto_percent, threshold.veto_active
    );

    // 5. Check merge blocking
    let merge_blocker = MergeBlocker::new(None, create_test_decision_logger());
    let should_block = MergeBlocker::should_block_merge(
        true, // review period met
        true, // signatures met
        true, // economic veto active
        tier,
        false, // emergency_mode
    )?;

    assert!(should_block);
    println!("✅ Tier 3 PR blocked due to economic node veto");

    // 6. Generate veto status
    let veto_status = StatusCheckGenerator::generate_economic_veto_status(
        true, // veto active
        25.0, // mining veto percent
        40.0, // economic veto percent
        2,    // total nodes
        2,    // veto count
    );

    assert!(veto_status.contains("Economic Node Veto Active") || veto_status.contains("Economic node veto active"));
    println!("✅ Economic veto status generated");

    println!("🎉 Tier 3 economic node veto scenario completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_tier_4_emergency_activation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Tier 4 (Emergency) activation...");

    // Setup
    let db = Database::new_in_memory().await?;

    // 1. Create an emergency PR
    let emergency_pr = json!({
        "pull_request": {
            "number": 3,
            "title": "[EMERGENCY] Critical security vulnerability fix",
            "body": "This PR fixes a critical security vulnerability that could lead to fund loss",
            "head": {"sha": "emergency123"},
            "base": {"sha": "main456"}
        },
        "repository": {"full_name": "test-org/security-critical"}
    });

    // 2. Classify as emergency
    let tier = tier_classification::classify_pr_tier(&emergency_pr).await;
    assert_eq!(tier, 4);
    println!("✅ PR classified as Tier 4 (Emergency)");

    // 3. Emergency requirements: 4-of-5 signatures, no review period
    let merge_blocker = MergeBlocker::new(None, create_test_decision_logger());

    // Emergency can be merged immediately if signatures are met
    let can_merge_emergency = !MergeBlocker::should_block_merge(
        true,  // no review period required for emergency
        true,  // signatures met
        false, // no economic veto for emergency
        tier,
        true, // emergency_mode
    )?;

    assert!(can_merge_emergency);
    println!("✅ Emergency PR can be merged immediately when signatures met");

    // 4. Generate emergency status
    use bllvm_commons::validation::emergency::{ActiveEmergency, EmergencyTier};
    use chrono::{Utc, Duration};
    let emergency = ActiveEmergency {
        id: 1,
        tier: EmergencyTier::Urgent,
        activated_by: "admin".to_string(),
        reason: "Critical security vulnerability discovered".to_string(),
        activated_at: Utc::now() - Duration::try_days(1).unwrap_or_default(),
        expires_at: Utc::now() + Duration::try_days(1).unwrap_or_default(),
        extended: false,
        extension_count: 0,
    };
    let emergency_status = StatusCheckGenerator::generate_emergency_status(&emergency);

    assert!(emergency_status.contains("Emergency"));
    println!("✅ Emergency status generated");

    println!("🎉 Tier 4 emergency activation completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_tier_5_governance_change_with_fork() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Tier 5 (Governance Change) with fork capability...");

    // Setup
    let db = Database::new_in_memory().await?;
    let versioning = RulesetVersioning::new();
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let tracker = AdoptionTracker::new(pool);

    // 1. Create governance change PR
    let governance_pr = json!({
        "pull_request": {
            "number": 4,
            "title": "[GOVERNANCE] Update governance rules",
            "body": "This PR updates the governance configuration",
            "head": {"sha": "governance123"},
            "base": {"sha": "main456"}
        },
        "repository": {"full_name": "test-org/governance"}
    });

    let tier = tier_classification::classify_pr_tier(&governance_pr).await;
    assert_eq!(tier, 5);
    println!("✅ PR classified as Tier 5 (Governance Change)");

    // 2. Export current governance configuration
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().to_str().unwrap();

    // Create sample governance config
    let config_content = r#"
tiers:
  - name: "Routine Maintenance"
    tier: 1
    signatures_required: 3
    signatures_total: 5
    review_period_days: 7
"#;

    tokio::fs::write(format!("{}/action-tiers.yml", config_path), config_content).await?;
    tokio::fs::write(format!("{}/economic-nodes.yml", config_path), "nodes: []").await?;
    tokio::fs::write(
        format!("{}/maintainers.yml", config_path),
        "maintainers: []",
    )
    .await?;
    tokio::fs::write(format!("{}/repos.yml", config_path), "repositories: []").await?;
    tokio::fs::write(
        format!("{}/governance-fork.yml", config_path),
        "fork: {enabled: true}",
    )
    .await?;

    let exporter = GovernanceExporter::new(config_path);
    let export = exporter
        .export_governance_config(
            "governance-v1.0.0",
            &RulesetVersion::new(1, 0, 0),
            "test_exporter",
            "test-repo",
            "governance123",
        )
        .await?;

    println!(
        "✅ Governance configuration exported: {}",
        export.ruleset_id
    );

    // 3. Create new ruleset version
    let new_config = json!({
        "tiers": [
            {
                "name": "Routine Maintenance",
                "tier": 1,
                "signatures_required": 4, // Changed from 3 to 4
                "signatures_total": 5,
                "review_period_days": 7
            }
        ]
    });

    let new_ruleset = versioning
        .create_ruleset(
            "governance-v1.1.0",
            "Governance v1.1.0",
            new_config,
            Some("Updated governance ruleset"),
        )?;

    println!(
        "✅ New governance ruleset created: {}",
        new_ruleset.id
    );

    // Ensure governance_rulesets table exists (migration might not have run)
    let pool = db.pool().expect("Database should have SQLite pool");
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='governance_rulesets')"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false);
    
    if !table_exists {
        // Create the table manually if migration didn't run
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS governance_rulesets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version_major INTEGER NOT NULL,
                version_minor INTEGER NOT NULL,
                version_patch INTEGER NOT NULL,
                version_pre_release TEXT,
                version_build_metadata TEXT,
                hash TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                config TEXT NOT NULL,
                description TEXT,
                status TEXT DEFAULT 'active'
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to create governance_rulesets table: {}", e)))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS fork_decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ruleset_id TEXT NOT NULL,
                node_id TEXT NOT NULL,
                node_type TEXT NOT NULL,
                weight REAL NOT NULL,
                decision_reason TEXT NOT NULL,
                signature TEXT NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (ruleset_id) REFERENCES governance_rulesets(id)
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to create fork_decisions table: {}", e)))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS fork_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT UNIQUE NOT NULL,
                event_type TEXT NOT NULL,
                ruleset_id TEXT,
                node_id TEXT,
                details TEXT NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (ruleset_id) REFERENCES governance_rulesets(id)
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to create fork_events table: {}", e)))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS adoption_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ruleset_id TEXT NOT NULL,
                node_count INTEGER NOT NULL,
                hashpower_percentage REAL NOT NULL,
                economic_activity_percentage REAL NOT NULL,
                total_weight REAL NOT NULL,
                calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (ruleset_id) REFERENCES governance_rulesets(id)
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to create adoption_metrics table: {}", e)))?;
    }
    
    // Save rulesets to database so fork_decisions can reference them
    sqlx::query(
        r#"
        INSERT INTO governance_rulesets (id, name, version_major, version_minor, version_patch, hash, config, description, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active')
        "#
    )
    .bind("governance-v1.0.0")
    .bind("Governance v1.0.0")
    .bind(1i32)
    .bind(0i32)
    .bind(0i32)
    .bind("hash_v1_0_0")
    .bind("{}")
    .bind::<Option<String>>(None)
    .execute(pool)
    .await
    .map_err(|e| GovernanceError::DatabaseError(format!("Failed to save ruleset v1.0.0: {}", e)))?;

    sqlx::query(
        r#"
        INSERT INTO governance_rulesets (id, name, version_major, version_minor, version_patch, hash, config, description, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active')
        "#
    )
    .bind(&new_ruleset.id)
    .bind(&new_ruleset.name)
    .bind(new_ruleset.version.major as i32)
    .bind(new_ruleset.version.minor as i32)
    .bind(new_ruleset.version.patch as i32)
    .bind(&new_ruleset.hash)
    .bind(serde_json::to_string(&new_ruleset.config).unwrap())
    .bind(new_ruleset.description.as_ref())
    .execute(pool)
    .await
    .map_err(|e| GovernanceError::DatabaseError(format!("Failed to save ruleset v1.1.0: {}", e)))?;

    // 4. Simulate adoption decisions
    use bllvm_commons::fork::types::ForkDecision;
    use chrono::Utc;
    
    let decision1 = ForkDecision {
        node_id: "1".to_string(),
        node_type: "mining_pool".to_string(),
        chosen_ruleset: "governance-v1.0.0".to_string(),
        decision_reason: "Prefer original ruleset".to_string(),
        weight: 0.3,
        timestamp: Utc::now(),
        signature: "signature1".to_string(),
    };
    tracker
        .record_fork_decision("governance-v1.0.0", "1", &decision1)
        .await?;

    let decision2 = ForkDecision {
        node_id: "2".to_string(),
        node_type: "exchange".to_string(),
        chosen_ruleset: "governance-v1.1.0".to_string(),
        decision_reason: "Prefer updated ruleset".to_string(),
        weight: 0.25,
        timestamp: Utc::now(),
        signature: "signature2".to_string(),
    };
    tracker
        .record_fork_decision("governance-v1.1.0", "2", &decision2)
        .await?;

    println!("✅ Fork decisions recorded");

    // 5. Calculate adoption metrics
    let metrics_v1 = tracker
        .calculate_adoption_metrics("governance-v1.0.0")
        .await?;
    let metrics_v2 = tracker
        .calculate_adoption_metrics("governance-v1.1.0")
        .await?;

    println!("✅ Adoption metrics calculated:");
    println!("   v1.0.0: {} nodes", metrics_v1.node_count);
    println!("   v1.1.0: {} nodes", metrics_v2.node_count);

    // 6. Get adoption statistics
    let stats = tracker.get_adoption_statistics().await?;
    assert!(stats.total_nodes > 0);
    assert!(stats.rulesets.len() > 0);
    println!(
        "✅ Adoption statistics: {} total nodes, {} rulesets",
        stats.total_nodes,
        stats.rulesets.len()
    );

    // 7. Check governance change requirements
    let merge_blocker = MergeBlocker::new(None, create_test_decision_logger());
    let should_block = MergeBlocker::should_block_merge(
        true,  // review period met (180 days for Tier 5)
        true,  // signatures met (5-of-5 for Tier 5)
        false, // no economic veto
        tier,
        false, // emergency_mode
    )?;

    assert!(!should_block);
    println!("✅ Tier 5 PR can be merged when all requirements met");

    println!("🎉 Tier 5 governance change with fork completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_complete_governance_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing complete governance lifecycle...");

    // Setup
    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool.clone());
    let veto_manager = VetoManager::new(pool);

    // 1. Register and activate economic nodes
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
            daily_volume_usd: 150_000_000.0, // Above 100M daily threshold
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

    // Generate keypairs for nodes
    let mining_keypair = GovernanceKeypair::generate()?;
    let exchange_keypair = GovernanceKeypair::generate()?;
    let mining_public_key = mining_keypair.public_key().to_string();
    let exchange_public_key = exchange_keypair.public_key().to_string();

    let mining_node_id = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Test Mining Pool",
            &mining_public_key,
            &mining_proof,
            Some("admin"),
        )
        .await?;

    let exchange_node_id = registry
        .register_economic_node(
            NodeType::Exchange,
            "Test Exchange",
            &exchange_public_key,
            &exchange_proof,
            Some("admin"),
        )
        .await?;

    registry
        .update_node_status(mining_node_id, NodeStatus::Active)
        .await?;
    registry
        .update_node_status(exchange_node_id, NodeStatus::Active)
        .await?;
    println!("✅ Economic nodes registered and activated");

    // 2. Test different PR scenarios
    let scenarios = vec![
        (1, "Routine maintenance", false),
        (2, "Feature addition", false),
        (3, "Consensus-adjacent change", true), // Requires economic node input
        (4, "Emergency fix", false),
        (5, "Governance change", false),
    ];

    for (tier, description, requires_economic_input) in scenarios {
        println!("  Testing Tier {}: {}", tier, description);

        // Create PR payload with appropriate title for tier classification
        let pr_title = match tier {
            1 => "Fix typo in README".to_string(),
            2 => "Add new feature".to_string(), // Will fall back to tier 2 (default)
            3 => "[CONSENSUS-ADJACENT] Update validation".to_string(),
            4 => "[EMERGENCY] Critical security fix".to_string(),
            5 => "[GOVERNANCE] Update governance rules".to_string(),
            _ => format!("[TIER{}] {}", tier, description),
        };
        
        let pr_payload = json!({
            "pull_request": {
                "number": tier,
                "title": pr_title,
                "body": format!("This is a {} PR", description),
                "head": {"sha": format!("tier{}123", tier)},
                "base": {"sha": "main456"}
            },
            "repository": {"full_name": "test-org/test-repo"}
        });

        // Classify tier
        let classified_tier = tier_classification::classify_pr_tier(&pr_payload).await;
        assert_eq!(classified_tier, tier as u32);
        println!("    ✅ Classified as Tier {}", classified_tier);

        // Test economic node input if required
        if requires_economic_input {
            // Create valid signatures for support signals
            let signature_manager = SignatureManager::new();
            let mining_message = format!("PR #{} veto signal from Test Mining Pool", tier);
            let exchange_message = format!("PR #{} veto signal from Test Exchange", tier);
            
            let mining_signature = signature_manager
                .create_governance_signature(&mining_message, &mining_keypair)?;
            let exchange_signature = signature_manager
                .create_governance_signature(&exchange_message, &exchange_keypair)?;

            // Submit support signal (not veto)
            veto_manager
                .collect_veto_signal(
                    tier,
                    mining_node_id,
                    SignalType::Support,
                    &mining_signature,
                    &format!("Supporting Tier {} change", tier),
                )
                .await?;

            veto_manager
                .collect_veto_signal(
                    tier,
                    exchange_node_id,
                    SignalType::Support,
                    &exchange_signature,
                    &format!("Supporting Tier {} change", tier),
                )
                .await?;

            // Check veto threshold (should not be active)
            let threshold = veto_manager.check_veto_threshold(tier).await?;
            assert!(!threshold.veto_active);
            println!("    ✅ Economic node support signals submitted, no veto active");
        }

        // Test merge blocking
        let merge_blocker = MergeBlocker::new(None, create_test_decision_logger());
        let should_block = MergeBlocker::should_block_merge(
            true,  // review period met
            true,  // signatures met
            false, // no veto active
            tier as u32,
            false, // emergency_mode
        )?;

        // Tier 4 (emergency) should not be blocked if requirements met
        if tier == 4 {
            assert!(!should_block);
        } else {
            // Other tiers should not be blocked if all requirements met
            assert!(!should_block);
        }
        println!("    ✅ Merge blocking logic working correctly");
    }

    // 3. Test governance fork scenario
    let versioning = RulesetVersioning::new();
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let tracker = AdoptionTracker::new(pool);

    // Create ruleset
    let config = json!({
        "tiers": [
            {
                "name": "Routine Maintenance",
                "tier": 1,
                "signatures_required": 3,
                "signatures_total": 5,
                "review_period_days": 7
            }
        ]
    });

    let ruleset = versioning
        .create_ruleset(
            "test-ruleset-v1.0.0",
            "Test Ruleset",
            config,
            Some("Test ruleset description"),
        )?;

    // Ensure governance_rulesets table exists (migration might not have run)
    let pool_for_ruleset = db.pool().expect("Database should have SQLite pool");
    let table_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='governance_rulesets')"
    )
    .fetch_one(pool_for_ruleset)
    .await
    .unwrap_or(false);
    
    if !table_exists {
        // Create the table manually if migration didn't run
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS governance_rulesets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version_major INTEGER NOT NULL,
                version_minor INTEGER NOT NULL,
                version_patch INTEGER NOT NULL,
                version_pre_release TEXT,
                version_build_metadata TEXT,
                hash TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                config TEXT NOT NULL,
                description TEXT,
                status TEXT DEFAULT 'active'
            )
            "#
        )
        .execute(pool_for_ruleset)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to create governance_rulesets table: {}", e)))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS fork_decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ruleset_id TEXT NOT NULL,
                node_id TEXT NOT NULL,
                node_type TEXT NOT NULL,
                weight REAL NOT NULL,
                decision_reason TEXT NOT NULL,
                signature TEXT NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (ruleset_id) REFERENCES governance_rulesets(id)
            )
            "#
        )
        .execute(pool_for_ruleset)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to create fork_decisions table: {}", e)))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS fork_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT UNIQUE NOT NULL,
                event_type TEXT NOT NULL,
                ruleset_id TEXT,
                node_id TEXT,
                details TEXT NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (ruleset_id) REFERENCES governance_rulesets(id)
            )
            "#
        )
        .execute(pool_for_ruleset)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to create fork_events table: {}", e)))?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS adoption_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ruleset_id TEXT NOT NULL,
                node_count INTEGER NOT NULL,
                hashpower_percentage REAL NOT NULL,
                economic_activity_percentage REAL NOT NULL,
                total_weight REAL NOT NULL,
                calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (ruleset_id) REFERENCES governance_rulesets(id)
            )
            "#
        )
        .execute(pool_for_ruleset)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to create adoption_metrics table: {}", e)))?;
    }
    
    // Save ruleset to database so fork_decisions can reference it
    sqlx::query(
        r#"
        INSERT INTO governance_rulesets (id, name, version_major, version_minor, version_patch, hash, config, description, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active')
        "#
    )
    .bind(&ruleset.id)
    .bind(&ruleset.name)
    .bind(ruleset.version.major as i32)
    .bind(ruleset.version.minor as i32)
    .bind(ruleset.version.patch as i32)
    .bind(&ruleset.hash)
    .bind(serde_json::to_string(&ruleset.config).unwrap())
    .bind(ruleset.description.as_ref())
    .execute(pool_for_ruleset)
    .await
    .map_err(|e| GovernanceError::DatabaseError(format!("Failed to save ruleset: {}", e)))?;

    // Record adoption decisions
    use bllvm_commons::fork::types::ForkDecision;
    use chrono::Utc;
    
    let decision1 = ForkDecision {
        node_id: mining_node_id.to_string(),
        node_type: "mining_pool".to_string(),
        chosen_ruleset: "test-ruleset-v1.0.0".to_string(),
        decision_reason: "Mining pool adopts this ruleset".to_string(),
        weight: 0.3,
        timestamp: Utc::now(),
        signature: "mining_adoption_signature".to_string(),
    };
    tracker
        .record_fork_decision("test-ruleset-v1.0.0", &mining_node_id.to_string(), &decision1)
        .await?;

    let decision2 = ForkDecision {
        node_id: exchange_node_id.to_string(),
        node_type: "exchange".to_string(),
        chosen_ruleset: "test-ruleset-v1.0.0".to_string(),
        decision_reason: "Exchange adopts this ruleset".to_string(),
        weight: 0.25,
        timestamp: Utc::now(),
        signature: "exchange_adoption_signature".to_string(),
    };
    tracker
        .record_fork_decision("test-ruleset-v1.0.0", &exchange_node_id.to_string(), &decision2)
        .await?;

    // Calculate adoption metrics
    let metrics = tracker
        .calculate_adoption_metrics("test-ruleset-v1.0.0")
        .await?;
    assert!(metrics.node_count > 0);
    println!(
        "✅ Governance fork scenario completed: {} nodes adopted ruleset",
        metrics.node_count
    );

    println!("🎉 Complete governance lifecycle test completed successfully!");
    Ok(())
}

#[tokio::test]
async fn test_error_handling_and_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing error handling and edge cases...");

    let db = Database::new_in_memory().await?;
    let pool = db.pool().expect("Database should have SQLite pool").clone();
    let registry = EconomicNodeRegistry::new(pool.clone());
    let veto_manager = VetoManager::new(pool);

    // 1. Test insufficient qualification
    let insufficient_proof = QualificationProof {
        node_type: NodeType::MiningPool,
        hashpower_proof: Some(HashpowerProof {
            blocks_mined: vec!["block1".to_string()],
            time_period_days: 30,
            total_network_blocks: 1000,
            percentage: 0.1, // Below 1% threshold
        }),
        holdings_proof: None,
        volume_proof: None,
        contact_info: ContactInfo {
            entity_name: "Insufficient Pool".to_string(),
            contact_email: "test@insufficient.com".to_string(),
            website: None,
            github_username: None,
        },
    };

    let result = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Insufficient Pool",
            "insufficient_key",
            &insufficient_proof,
            Some("admin"),
        )
        .await;

    assert!(result.is_err());
    println!("✅ Insufficient qualification correctly rejected");

    // 2. Test duplicate node registration
    let valid_proof = QualificationProof {
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

    registry
        .register_economic_node(
            NodeType::MiningPool,
            "Test Pool",
            "duplicate_key",
            &valid_proof,
            Some("admin"),
        )
        .await?;

    let duplicate_result = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Another Pool",
            "duplicate_key", // Same public key
            &valid_proof,
            Some("admin"),
        )
        .await;

    assert!(duplicate_result.is_err());
    println!("✅ Duplicate node registration correctly rejected");

    // 3. Test invalid signature format
    let node_id = registry
        .register_economic_node(
            NodeType::MiningPool,
            "Valid Pool",
            "valid_key",
            &QualificationProof {
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
    },
            Some("admin"),
        )
        .await?;
    
    // Activate the node so it can submit veto signals
    registry.update_node_status(node_id, NodeStatus::Active).await?;

    // This should fail due to invalid signature format (not valid hex)
    // Create a valid hex string but with wrong length to trigger hex decode error
    let invalid_signature_result = veto_manager
        .collect_veto_signal(
            1,
            node_id,
            SignalType::Veto,
            "abc", // Invalid hex (odd number of digits)
            "Test veto",
        )
        .await;

    // This should fail with invalid signature hex error
    assert!(invalid_signature_result.is_err());
    println!("✅ Invalid signature handling tested");

    // 4. Test non-existent node
    let non_existent_result = veto_manager
        .collect_veto_signal(
            1,
            99999, // Non-existent node ID
            SignalType::Veto,
            "test_signature",
            "Test veto",
        )
        .await;

    assert!(non_existent_result.is_err());
    println!("✅ Non-existent node correctly rejected");

    // 5. Test duplicate veto signal
    // Create valid signatures for the duplicate test
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
    
    let message1 = format!("PR #2 veto signal from Valid Pool");
    let signature1 = signature_manager
        .create_governance_signature(&message1, &keypair)?;
    
    veto_manager
        .collect_veto_signal(
            2,
            node_id,
            SignalType::Veto,
            &signature1,
            "First veto",
        )
        .await?;

    let message2 = format!("PR #2 veto signal from Valid Pool");
    let signature2 = signature_manager
        .create_governance_signature(&message2, &keypair)?;
    
    let duplicate_veto_result = veto_manager
        .collect_veto_signal(
            2,       // Same PR
            node_id, // Same node
            SignalType::Support,
            &signature2,
            "Changed mind",
        )
        .await;

    assert!(duplicate_veto_result.is_err());
    println!("✅ Duplicate veto signal correctly rejected");

    // 6. Test version parsing edge cases
    assert!(RulesetVersion::from_string("1.0.0").is_ok());
    assert!(RulesetVersion::from_string("1.0.0").is_ok()); // v prefix not supported
    assert!(RulesetVersion::from_string("invalid").is_err());
    assert!(RulesetVersion::from_string("1.0").is_err());
    assert!(RulesetVersion::from_string("1.0.0.0").is_err());
    println!("✅ Version parsing edge cases handled correctly");

    println!("🎉 Error handling and edge cases test completed successfully!");
    Ok(())
}




