//! Integration tests for Consensus Modification Verification
//!
//! Tests the complete flow of consensus modification detection including:
//! - File path pattern matching
//! - Consensus-critical file detection
//! - GitHub PR file checking (mocked)
//! - Rule-based validation

use governance_app::validation::cross_layer::CrossLayerValidator;
use governance_app::error::GovernanceError;
use serde_json::json;

#[tokio::test]
async fn test_consensus_pattern_matching() {
    let validator = CrossLayerValidator;
    
    // Test consensus-critical file patterns
    let consensus_files = vec![
        "src/block.rs".to_string(),
        "src/transaction.rs".to_string(),
        "src/script.rs".to_string(),
        "src/economic.rs".to_string(),
        "src/pow.rs".to_string(),
        "src/validation/block.rs".to_string(),
        "src/consensus/rules.rs".to_string(),
    ];
    
    let non_consensus_files = vec![
        "src/utils.rs".to_string(),
        "tests/test_block.rs".to_string(),
        "docs/README.md".to_string(),
        "Cargo.toml".to_string(),
    ];
    
    // Create a rule that checks for consensus modifications
    let rule = json!({
        "target_repo": "BTCDecoded/bllvm-consensus",
        "validation_type": "no_consensus_modifications",
        "allowed_imports_only": false,
        "check_files": consensus_files.clone(),
    });
    
    // Should fail because consensus files are in the list
    let result = validator.validate_dependency(
        "BTCDecoded/bllvm-consensus",
        "no_consensus_modifications",
        &rule,
        None,
    ).await;
    
    assert!(result.is_err(), "Consensus files should trigger validation error");
    
    // Test with non-consensus files
    let rule2 = json!({
        "target_repo": "BTCDecoded/bllvm-consensus",
        "validation_type": "no_consensus_modifications",
        "allowed_imports_only": false,
        "check_files": non_consensus_files,
    });
    
    let result2 = validator.validate_dependency(
        "BTCDecoded/bllvm-consensus",
        "no_consensus_modifications",
        &rule2,
        None,
    ).await;
    
    assert!(result2.is_ok(), "Non-consensus files should pass validation");
}

#[tokio::test]
async fn test_consensus_verification_with_file_patterns() {
    let validator = CrossLayerValidator;
    
    // Test pattern matching with different file paths
    let test_cases = vec![
        // Should fail (consensus files)
        (vec!["src/block.rs".to_string()], true),
        (vec!["src/validation/block.rs".to_string()], true),
        (vec!["src/consensus/rules.rs".to_string()], true),
        // Should pass (non-consensus files)
        (vec!["src/utils.rs".to_string()], false),
        (vec!["tests/test_block.rs".to_string()], false),
        (vec!["docs/README.md".to_string()], false),
    ];
    
    for (files, should_fail) in test_cases {
        let rule = json!({
            "target_repo": "BTCDecoded/bllvm-consensus",
            "validation_type": "no_consensus_modifications",
            "check_files": files,
        });
        
        let result = validator.validate_dependency(
            "BTCDecoded/bllvm-consensus",
            "no_consensus_modifications",
            &rule,
            None,
        ).await;
        
        if should_fail {
            assert!(result.is_err(), "Consensus file should trigger error: {:?}", files);
        } else {
            assert!(result.is_ok(), "Non-consensus file should pass: {:?}", files);
        }
    }
}

#[tokio::test]
async fn test_consensus_verification_backward_compatibility() {
    let validator = CrossLayerValidator;
    
    // Test that validation passes when no file info is provided (backward compatibility)
    let rule = json!({
        "target_repo": "BTCDecoded/bllvm-consensus",
        "validation_type": "no_consensus_modifications",
        "allowed_imports_only": false,
    });
    
    let result = validator.validate_dependency(
        "BTCDecoded/bllvm-consensus",
        "no_consensus_modifications",
        &rule,
        None,
    ).await;
    
    // Should pass (backward compatibility - no file info means we can't check)
    assert!(result.is_ok(), "Should pass when no file info provided (backward compatibility)");
}

#[tokio::test]
async fn test_consensus_verification_allowed_imports_only() {
    let validator = CrossLayerValidator;
    
    // Test with allowed_imports_only flag
    let rule = json!({
        "target_repo": "BTCDecoded/bllvm-consensus",
        "validation_type": "no_consensus_modifications",
        "allowed_imports_only": true,
        "check_files": vec!["src/block.rs"],
    });
    
    let result = validator.validate_dependency(
        "BTCDecoded/bllvm-consensus",
        "no_consensus_modifications",
        &rule,
        None,
    ).await;
    
    // Should still fail because we're checking file paths, not diff content
    // (Full diff analysis is Phase 2)
    assert!(result.is_err(), "File path check should still fail even with allowed_imports_only");
}

#[tokio::test]
async fn test_consensus_verification_multiple_files() {
    let validator = CrossLayerValidator;
    
    // Test with mix of consensus and non-consensus files
    let mixed_files = vec![
        "src/utils.rs".to_string(), // OK
        "src/block.rs".to_string(), // Consensus - should fail
        "tests/test.rs".to_string(), // OK
    ];
    
    let rule = json!({
        "target_repo": "BTCDecoded/bllvm-consensus",
        "validation_type": "no_consensus_modifications",
        "check_files": mixed_files,
    });
    
    let result = validator.validate_dependency(
        "BTCDecoded/bllvm-consensus",
        "no_consensus_modifications",
        &rule,
        None,
    ).await;
    
    // Should fail because at least one consensus file is present
    assert!(result.is_err(), "Should fail when any consensus file is present");
    
    // Check error message contains the consensus file
    if let Err(GovernanceError::ValidationError(msg)) = result {
        assert!(msg.contains("block.rs"), "Error should mention the consensus file");
        assert!(msg.contains("Tier 3+"), "Error should mention Tier 3+ requirement");
    } else {
        panic!("Expected ValidationError");
    }
}

#[tokio::test]
async fn test_consensus_pattern_matching_edge_cases() {
    let validator = CrossLayerValidator;
    
    // Test edge cases
    let test_cases = vec![
        // Exact matches
        ("src/block.rs", true),
        ("src/transaction.rs", true),
        // Subdirectory matches
        ("src/validation/block.rs", true),
        ("src/consensus/rules.rs", true),
        // Non-matches
        ("src/block_utils.rs", false), // Different file
        ("block.rs", false), // No src/ prefix
        ("src/blocks/block.rs", false), // Different directory
    ];
    
    for (file, should_match) in test_cases {
        let rule = json!({
            "target_repo": "BTCDecoded/bllvm-consensus",
            "validation_type": "no_consensus_modifications",
            "check_files": vec![file],
        });
        
        let result = validator.validate_dependency(
            "BTCDecoded/bllvm-consensus",
            "no_consensus_modifications",
            &rule,
            None,
        ).await;
        
        if should_match {
            assert!(result.is_err(), "Should fail for consensus file: {}", file);
        } else {
            assert!(result.is_ok(), "Should pass for non-consensus file: {}", file);
        }
    }
}

