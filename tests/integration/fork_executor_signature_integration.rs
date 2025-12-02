//! Integration tests for Fork Executor with Cryptographic Signatures
//!
//! Tests the complete flow of fork execution including:
//! - Fork decision creation and signing
//! - Signature verification
//! - Unsigned decisions (backward compatibility)
//! - Tampering detection

use governance_app::fork::{ForkExecutor, ForkStatus, verify_fork_decision_signature};
use governance_app::fork::types::{Ruleset, ForkDecision};
use governance_app::error::GovernanceError;
use bllvm_sdk::governance::GovernanceKeypair;
use secp256k1::SecretKey;
use chrono::Utc;
use tempfile::tempdir;
use sqlx::SqlitePool;

#[tokio::test]
async fn test_fork_executor_with_signing() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let export_path = temp_dir.path().join("exports");
    std::fs::create_dir_all(&export_path)?;
    
    // Create in-memory database
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Create keypair for signing
    let keypair = GovernanceKeypair::generate()?;
    let secret_key = keypair.secret_key();
    
    // Create executor with secret key
    let mut executor = ForkExecutor::new_with_key(
        export_path.to_str().unwrap(),
        pool,
        None,
        Some(secret_key),
    )?;
    
    // Create a test ruleset
    let ruleset = Ruleset {
        id: "test-ruleset-1".to_string(),
        version: "1.0.0".to_string(),
        config_hash: "test-hash".to_string(),
        created_at: Utc::now(),
        description: Some("Test ruleset".to_string()),
    };
    
    // Add ruleset to executor
    executor.available_rulesets.insert("test-ruleset-1".to_string(), ruleset.clone());
    
    // Execute fork (this should sign the decision)
    executor.execute_fork("test-ruleset-1").await?;
    
    // Verify current ruleset is set
    assert!(executor.get_current_ruleset().is_some());
    assert_eq!(executor.get_current_ruleset().unwrap().id, "test-ruleset-1");
    
    Ok(())
}

#[tokio::test]
async fn test_fork_decision_signature_verification() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = GovernanceKeypair::generate()?;
    let public_key = keypair.public_key();
    let secret_key = keypair.secret_key();
    
    // Create a fork decision
    let mut decision = ForkDecision {
        node_id: "test-node".to_string(),
        node_type: "governance-app".to_string(),
        chosen_ruleset: "ruleset-1".to_string(),
        decision_reason: "Test fork execution".to_string(),
        weight: 1.0,
        timestamp: Utc::now(),
        signature: String::new(),
    };
    
    // Sign the decision
    use bllvm_sdk::governance::signatures::sign_message;
    use hex;
    use serde_json;
    
    let message = serde_json::json!({
        "node_id": decision.node_id,
        "node_type": decision.node_type,
        "chosen_ruleset": decision.chosen_ruleset,
        "decision_reason": decision.decision_reason,
        "weight": decision.weight,
        "timestamp": decision.timestamp.to_rfc3339(),
    });
    let message_bytes = serde_json::to_vec(&message)?;
    
    let signature = sign_message(&secret_key, &message_bytes)?;
    decision.signature = hex::encode(signature.to_bytes());
    
    // Verify signature
    let verified = verify_fork_decision_signature(&decision, &public_key)?;
    assert!(verified, "Signature should verify correctly");
    
    Ok(())
}

#[tokio::test]
async fn test_fork_decision_tampering_detection() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = GovernanceKeypair::generate()?;
    let public_key = keypair.public_key();
    let secret_key = keypair.secret_key();
    
    use bllvm_sdk::governance::signatures::sign_message;
    use hex;
    use serde_json;
    
    // Create and sign decision
    let mut decision = ForkDecision {
        node_id: "test-node".to_string(),
        node_type: "governance-app".to_string(),
        chosen_ruleset: "ruleset-1".to_string(),
        decision_reason: "Test fork".to_string(),
        weight: 1.0,
        timestamp: Utc::now(),
        signature: String::new(),
    };
    
    let message = serde_json::json!({
        "node_id": decision.node_id,
        "node_type": decision.node_type,
        "chosen_ruleset": decision.chosen_ruleset,
        "decision_reason": decision.decision_reason,
        "weight": decision.weight,
        "timestamp": decision.timestamp.to_rfc3339(),
    });
    let message_bytes = serde_json::to_vec(&message)?;
    
    let signature = sign_message(&secret_key, &message_bytes)?;
    decision.signature = hex::encode(signature.to_bytes());
    
    // Verify original - should pass
    let verified = verify_fork_decision_signature(&decision, &public_key)?;
    assert!(verified, "Original decision should verify");
    
    // Tamper with decision
    decision.chosen_ruleset = "ruleset-2".to_string();
    
    // Verification should fail
    let verified = verify_fork_decision_signature(&decision, &public_key)?;
    assert!(!verified, "Tampered decision should fail verification");
    
    Ok(())
}

#[tokio::test]
async fn test_fork_executor_without_key() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let export_path = temp_dir.path().join("exports");
    std::fs::create_dir_all(&export_path)?;
    
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Create executor without secret key (backward compatibility)
    let mut executor = ForkExecutor::new(
        export_path.to_str().unwrap(),
        pool,
        None,
    )?;
    
    // Create test ruleset
    let ruleset = Ruleset {
        id: "test-ruleset-2".to_string(),
        version: "1.0.0".to_string(),
        config_hash: "test-hash-2".to_string(),
        created_at: Utc::now(),
        description: Some("Test ruleset 2".to_string()),
    };
    
    executor.available_rulesets.insert("test-ruleset-2".to_string(), ruleset.clone());
    
    // Execute fork - should work but decision will be unsigned
    executor.execute_fork("test-ruleset-2").await?;
    
    // Should still work
    assert!(executor.get_current_ruleset().is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_fork_executor_set_key_later() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let export_path = temp_dir.path().join("exports");
    std::fs::create_dir_all(&export_path)?;
    
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Create executor without key
    let mut executor = ForkExecutor::new(
        export_path.to_str().unwrap(),
        pool,
        None,
    )?;
    
    // Set key later
    let keypair = GovernanceKeypair::generate()?;
    executor.set_secret_key(keypair.secret_key());
    
    // Create and execute fork
    let ruleset = Ruleset {
        id: "test-ruleset-3".to_string(),
        version: "1.0.0".to_string(),
        config_hash: "test-hash-3".to_string(),
        created_at: Utc::now(),
        description: Some("Test ruleset 3".to_string()),
    };
    
    executor.available_rulesets.insert("test-ruleset-3".to_string(), ruleset.clone());
    executor.execute_fork("test-ruleset-3").await?;
    
    assert!(executor.get_current_ruleset().is_some());
    
    Ok(())
}

#[tokio::test]
async fn test_fork_decision_wrong_public_key() -> Result<(), Box<dyn std::error::Error>> {
    let keypair1 = GovernanceKeypair::generate()?;
    let keypair2 = GovernanceKeypair::generate()?;
    
    let public_key1 = keypair1.public_key();
    let secret_key1 = keypair1.secret_key();
    let public_key2 = keypair2.public_key();
    
    use bllvm_sdk::governance::signatures::sign_message;
    use hex;
    use serde_json;
    
    // Create and sign decision with keypair1
    let mut decision = ForkDecision {
        node_id: "test-node".to_string(),
        node_type: "governance-app".to_string(),
        chosen_ruleset: "ruleset-1".to_string(),
        decision_reason: "Test".to_string(),
        weight: 1.0,
        timestamp: Utc::now(),
        signature: String::new(),
    };
    
    let message = serde_json::json!({
        "node_id": decision.node_id,
        "node_type": decision.node_type,
        "chosen_ruleset": decision.chosen_ruleset,
        "decision_reason": decision.decision_reason,
        "weight": decision.weight,
        "timestamp": decision.timestamp.to_rfc3339(),
    });
    let message_bytes = serde_json::to_vec(&message)?;
    
    let signature = sign_message(&secret_key1, &message_bytes)?;
    decision.signature = hex::encode(signature.to_bytes());
    
    // Verify with correct key - should pass
    let verified = verify_fork_decision_signature(&decision, &public_key1)?;
    assert!(verified, "Correct key should verify");
    
    // Verify with wrong key - should fail
    let verified = verify_fork_decision_signature(&decision, &public_key2)?;
    assert!(!verified, "Wrong key should fail verification");
    
    Ok(())
}

