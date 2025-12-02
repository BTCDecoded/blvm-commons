//! Snapshot tests for GitHub-related functions
//!
//! These tests snapshot GitHub API responses and status checks.

use bllvm_commons::github::cross_layer_status::CrossLayerStatusChecker;
use insta::assert_snapshot;

#[test]
fn test_status_description_snapshot() {
    // Test various status combinations
    let test_cases = vec![
        (vec!["success", "success", "success"], "all_success"),
        (vec!["failure", "success", "success"], "one_failure"),
        (vec!["pending", "pending", "pending"], "all_pending"),
        (vec!["success", "failure", "pending"], "mixed_statuses"),
    ];
    
    for (statuses, name) in test_cases {
        // Create a simple status description
        let description = statuses.iter()
            .map(|s| match *s {
                "success" => "✅",
                "failure" => "❌",
                "pending" => "⏳",
                _ => "❓",
            })
            .collect::<Vec<_>>()
            .join(" ");
        
        assert_snapshot!(format!("status_description_{}", name), description);
    }
}

#[test]
fn test_test_count_extraction_snapshot() {
    let test_cases = vec![
        "123 tests",
        "Tests: 456",
        "cargo test: 789",
        "1000 passed",
        "passed: 42",
    ];
    
    for test_case in test_cases {
        let result = CrossLayerStatusChecker::extract_test_count_from_name(test_case);
        assert_snapshot!(
            format!("test_count_extraction_{}", test_case.replace(" ", "_").replace(":", "_")),
            format!("input: '{}', output: {:?}", test_case, result)
        );
    }
}

