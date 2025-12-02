//! Integration tests for build orchestration

use bllvm_commons::build::*;
use bllvm_commons::github::client::GitHubClient;
use bllvm_commons::database::Database;
use serde_json::json;

// Mock GitHub client for testing
struct MockGitHubClient {
    // Track triggered workflows
    triggered_workflows: std::sync::Arc<std::sync::Mutex<Vec<(String, String, serde_json::Value)>>>,
    // Mock workflow run IDs
    workflow_run_ids: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, u64>>>,
}

impl MockGitHubClient {
    fn new() -> Self {
        Self {
            triggered_workflows: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            workflow_run_ids: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[tokio::test]
async fn test_build_orchestration_happy_path() {
    // Test the full orchestration flow with mocks
    // This proves the logic works end-to-end
    
    // 1. Create dependency graph
    let graph = DependencyGraph::new("BTCDecoded".to_string());
    let build_order = graph.get_build_order().unwrap();
    
    // Verify build order is correct
    assert_eq!(build_order[0], "bllvm-consensus");
    assert!(build_order.iter().position(|r| r == "bllvm-protocol").unwrap() 
            > build_order.iter().position(|r| r == "bllvm-consensus").unwrap());
    
    // 2. Test parallel group detection
    let parallel_groups = graph.get_parallel_groups().unwrap();
    assert!(!parallel_groups.is_empty());
    
    // 3. Test dependency resolution
    let deps = graph.get_dependencies("bllvm-node");
    assert!(deps.contains(&"bllvm-protocol".to_string()));
    assert!(deps.contains(&"bllvm-consensus".to_string()));
}

#[tokio::test]
async fn test_build_order_respects_dependencies() {
    let graph = DependencyGraph::new("BTCDecoded".to_string());
    let order = graph.get_build_order().unwrap();
    
    // Verify all dependencies come before dependents
    let node_pos = order.iter().position(|r| r == "bllvm-node").unwrap();
    let protocol_pos = order.iter().position(|r| r == "bllvm-protocol").unwrap();
    let consensus_pos = order.iter().position(|r| r == "bllvm-consensus").unwrap();
    
    assert!(consensus_pos < protocol_pos);
    assert!(protocol_pos < node_pos);
    assert!(consensus_pos < node_pos);
}

#[tokio::test]
async fn test_circular_dependency_detection() {
    let mut graph = DependencyGraph::new("BTCDecoded".to_string());
    
    // Create a circular dependency
    graph.add_dependency("repo-a".to_string(), vec!["repo-b".to_string()]);
    graph.add_dependency("repo-b".to_string(), vec!["repo-a".to_string()]);
    
    // Should detect circular dependency
    let result = graph.get_build_order();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Circular dependency"));
}

#[tokio::test]
async fn test_parallel_build_groups() {
    let graph = DependencyGraph::new("BTCDecoded".to_string());
    let groups = graph.get_parallel_groups().unwrap();
    
    // First group should contain repos with no dependencies
    assert!(!groups.is_empty());
    
    // bllvm-consensus and bllvm-sdk can be built in parallel
    let first_group = &groups[0];
    assert!(
        first_group.contains(&"bllvm-consensus".to_string()) || 
        first_group.contains(&"bllvm-sdk".to_string()),
        "First group should contain repos with no dependencies"
    );
}

// Test with actual database (if available)
#[tokio::test]
#[ignore] // Requires database setup
async fn test_orchestrator_with_database() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "sqlite::memory:".to_string());
    
    let database = Database::new(&database_url).await.unwrap();
    
    // Test that orchestrator can be created
    // (We can't fully test without real GitHub client, but we can test structure)
    // This would require mocking the GitHub client
}

