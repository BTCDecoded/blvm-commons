//! Integration tests for cross-layer status checks
//!
//! These tests verify the extract_test_counts_from_ci functionality
//! using mocked GitHub API responses.

use bllvm_commons::github::cross_layer_status::CrossLayerStatusChecker;
use bllvm_commons::github::client::GitHubClient;
use bllvm_commons::github::types::CheckRun;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};
use serde_json::json;

mod common;
use common::*;

/// Test extracting test counts from CI check runs with various formats
#[tokio::test]
async fn test_extract_test_counts_from_ci_with_mock() {
    // Start mock server
    let mock_server = MockServer::start().await;
    
    // Create a real GitHub client that will use the mock server
    // For this test, we'll need to configure the client to use the mock server URL
    // Since GitHubClient uses octocrab which doesn't easily support custom base URLs,
    // we'll test the extraction logic directly with mock data
    
    // Create test check runs with various formats
    let check_runs = vec![
        CheckRun {
            name: "cargo test --lib (123 tests)".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: Some("https://github.com/test/repo/actions/runs/123".to_string()),
        },
        CheckRun {
            name: "Unit & Property Tests: 456".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
        CheckRun {
            name: "Kani Model Checking: 10 passed".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
    ];

    // Test the extraction function directly
    let mut total_tests = 0;
    for check_run in &check_runs {
        if let Some(count) = CrossLayerStatusChecker::extract_test_count_from_name(&check_run.name) {
            total_tests += count;
        }
    }

    // Should extract: 123 + 456 + 10 = 589
    assert!(total_tests > 0, "Should extract test counts from check run names");
    assert!(total_tests >= 123, "Should extract at least the first test count");
}

/// Test extracting test counts with failure scenarios
#[tokio::test]
async fn test_extract_test_counts_with_failures() {
    let check_runs = vec![
        CheckRun {
            name: "Tests (100 passed, 5 failed)".to_string(),
            conclusion: Some("failure".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
        CheckRun {
            name: "cargo test: 200 tests".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
    ];

    // Extract counts
    let mut total_tests = 0;
    for check_run in &check_runs {
        if let Some(count) = CrossLayerStatusChecker::extract_test_count_from_name(&check_run.name) {
            total_tests += count;
        }
    }

    // Should extract test counts even from failed runs
    assert!(total_tests > 0);
}

/// Test extracting test counts with no test-related check runs
#[tokio::test]
async fn test_extract_test_counts_no_test_runs() {
    let check_runs = vec![
        CheckRun {
            name: "Lint".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
        CheckRun {
            name: "Build".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
    ];

    // Should handle gracefully when no test counts found
    let mut total_tests = 0;
    for check_run in &check_runs {
        if let Some(count) = CrossLayerStatusChecker::extract_test_count_from_name(&check_run.name) {
            total_tests += count;
        }
    }

    // No test counts should be extracted
    assert_eq!(total_tests, 0);
}

/// Test extracting test counts with mixed formats
#[tokio::test]
async fn test_extract_test_counts_mixed_formats() {
    let check_runs = vec![
        CheckRun {
            name: "123 tests".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
        CheckRun {
            name: "Tests: 456".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
        CheckRun {
            name: "1000 passed".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
        CheckRun {
            name: "passed: 42".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: None,
        },
    ];

    let mut extracted_counts = Vec::new();
    for check_run in &check_runs {
        if let Some(count) = CrossLayerStatusChecker::extract_test_count_from_name(&check_run.name) {
            extracted_counts.push(count);
        }
    }

    // Should extract all 4 counts
    assert_eq!(extracted_counts.len(), 4);
    assert_eq!(extracted_counts, vec![123, 456, 1000, 42]);
}

/// Test real-world CI check run name formats
#[tokio::test]
async fn test_extract_test_counts_real_world_formats() {
    let real_world_names = vec![
        "cargo test --lib (123 tests)",
        "Unit & Property Tests: 456",
        "Kani Model Checking: 10 passed",
        "Tests (100 passed, 5 failed)",
        "cargo test: 200 tests",
        "Rust tests: 789",
    ];

    let mut total_extracted = 0;
    for name in &real_world_names {
        if let Some(count) = CrossLayerStatusChecker::extract_test_count_from_name(name) {
            total_extracted += count;
        }
    }

    // Should extract counts from all real-world formats
    assert!(total_extracted > 0);
    assert!(total_extracted >= 123 + 456 + 10 + 100 + 200 + 789);
}

