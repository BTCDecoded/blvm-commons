use bllvm_commons::crypto::multisig::MultisigManager;
use bllvm_commons::crypto::signatures::SignatureManager;
use bllvm_commons::database::Database;
use bllvm_commons::enforcement::decision_log::DecisionLogger;
use bllvm_sdk::governance::GovernanceKeypair;
use chrono::{DateTime, Duration, Utc};
use rand::rngs::OsRng;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

/// Setup an in-memory SQLite database for testing
pub async fn setup_test_db() -> Database {
    Database::new_in_memory()
        .await
        .expect("Failed to create test database")
}

/// Create a test signature manager
pub fn create_test_signature_manager() -> SignatureManager {
    SignatureManager::new()
}

/// Create a test multisig manager
pub fn create_test_multisig_manager() -> MultisigManager {
    MultisigManager::new()
}

/// Create a test decision logger
pub fn create_test_decision_logger() -> DecisionLogger {
    DecisionLogger::new(true, false, None)
}

/// Generate a valid governance signature for testing
pub fn create_test_governance_signature(message: &str) -> (String, String) {
    let keypair = GovernanceKeypair::generate().expect("Failed to generate keypair");
    let signature_manager = SignatureManager::new();
    let signature = signature_manager
        .create_governance_signature(message, &keypair)
        .expect("Failed to create signature");
    let public_key = keypair.public_key().to_string();
    (signature, public_key)
}

/// Generate test keypairs for testing
pub fn generate_test_keypairs(count: usize) -> Vec<(String, SecretKey, PublicKey)> {
    let secp = Secp256k1::new();
    let mut keypairs = Vec::new();

    for i in 0..count {
        let secret_key = SecretKey::new(&mut OsRng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let username = format!("testuser{}", i);

        keypairs.push((username, secret_key, public_key));
    }

    keypairs
}

/// Create test maintainers data
pub fn create_test_maintainers() -> Vec<(String, String, i32)> {
    vec![
        ("alice".to_string(), "pubkey_alice".to_string(), 1),
        ("bob".to_string(), "pubkey_bob".to_string(), 1),
        ("charlie".to_string(), "pubkey_charlie".to_string(), 2),
        ("dave".to_string(), "pubkey_dave".to_string(), 2),
        ("eve".to_string(), "pubkey_eve".to_string(), 3),
    ]
}

/// Create test emergency keyholders
pub fn create_test_emergency_keyholders() -> Vec<(String, String)> {
    vec![
        (
            "emergency_alice".to_string(),
            "emergency_pubkey_alice".to_string(),
        ),
        (
            "emergency_bob".to_string(),
            "emergency_pubkey_bob".to_string(),
        ),
        (
            "emergency_charlie".to_string(),
            "emergency_pubkey_charlie".to_string(),
        ),
        (
            "emergency_dave".to_string(),
            "emergency_pubkey_dave".to_string(),
        ),
        (
            "emergency_eve".to_string(),
            "emergency_pubkey_eve".to_string(),
        ),
        (
            "emergency_frank".to_string(),
            "emergency_pubkey_frank".to_string(),
        ),
        (
            "emergency_grace".to_string(),
            "emergency_pubkey_grace".to_string(),
        ),
    ]
}

/// Create test pull request data
pub fn create_test_pull_request(
    repo_name: &str,
    pr_number: i32,
    layer: i32,
    opened_days_ago: i64,
) -> (String, i32, String, i32, DateTime<Utc>) {
    let opened_at = Utc::now() - Duration::days(opened_days_ago);
    let head_sha = format!("abc123def456{}", pr_number);

    (repo_name.to_string(), pr_number, head_sha, layer, opened_at)
}

/// Create test cross-layer rules
pub fn create_test_cross_layer_rules() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "source_repo": "BTCDecoded/bllvm-consensus",
            "source_pattern": "src/consensus/**",
            "target_repo": "BTCDecoded/bllvm-protocol",
            "target_pattern": "src/validation/**",
            "validation_type": "corresponding_file_exists"
        }),
        serde_json::json!({
            "source_repo": "BTCDecoded/bllvm-protocol",
            "source_pattern": "src/network/**",
            "target_repo": "BTCDecoded/bllvm-node",
            "target_pattern": "src/network/**",
            "validation_type": "references_latest_version"
        }),
    ]
}

/// Create test signatures for a pull request
pub fn create_test_signatures(signers: &[String]) -> Vec<serde_json::Value> {
    signers
        .iter()
        .map(|signer| {
            serde_json::json!({
                "signer": signer,
                "signature": format!("signature_{}", signer),
                "timestamp": Utc::now()
            })
        })
        .collect()
}

/// Mock GitHub webhook payloads
pub mod github_mocks {
    use serde_json::Value;

    pub fn pull_request_opened_payload(repo: &str, pr_number: u64) -> Value {
        serde_json::json!({
            "action": "opened",
            "repository": {
                "full_name": repo
            },
            "pull_request": {
                "number": pr_number,
                "head": {
                    "sha": "abc123def456"
                }
            }
        })
    }

    pub fn pull_request_synchronize_payload(repo: &str, pr_number: u64) -> Value {
        serde_json::json!({
            "action": "synchronize",
            "repository": {
                "full_name": repo
            },
            "pull_request": {
                "number": pr_number,
                "head": {
                    "sha": "def456ghi789"
                }
            }
        })
    }

    pub fn review_submitted_payload(
        repo: &str,
        pr_number: u64,
        reviewer: &str,
        state: &str,
    ) -> Value {
        serde_json::json!({
            "action": "submitted",
            "repository": {
                "full_name": repo
            },
            "pull_request": {
                "number": pr_number
            },
            "review": {
                "user": {
                    "login": reviewer
                },
                "state": state
            }
        })
    }

    pub fn comment_created_payload(
        repo: &str,
        pr_number: u64,
        commenter: &str,
        body: &str,
    ) -> Value {
        serde_json::json!({
            "action": "created",
            "repository": {
                "full_name": repo
            },
            "issue": {
                "number": pr_number
            },
            "comment": {
                "user": {
                    "login": commenter
                },
                "body": body
            }
        })
    }

    pub fn push_payload(repo: &str, pusher: &str, ref_name: &str) -> Value {
        serde_json::json!({
            "repository": {
                "full_name": repo
            },
            "pusher": {
                "name": pusher
            },
            "ref": ref_name
        })
    }
}

/// Mock GitHub client for testing
pub mod mock_github {
    use bllvm_commons::error::GovernanceError;
    use bllvm_commons::github::types::CheckRun;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// In-memory mock GitHub client for testing
    #[derive(Clone)]
    pub struct MockGitHubClient {
        pull_requests: Arc<Mutex<HashMap<String, Value>>>,
        check_runs: Arc<Mutex<HashMap<String, Vec<CheckRun>>>>,
    }

    impl MockGitHubClient {
        pub fn new() -> Self {
            Self {
                pull_requests: Arc::new(Mutex::new(HashMap::new())),
                check_runs: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        /// Set a mock pull request response
        pub async fn set_pull_request(
            &self,
            owner: &str,
            repo: &str,
            pr_number: u64,
            pr_data: Value,
        ) {
            let key = format!("{}/{}/{}", owner, repo, pr_number);
            self.pull_requests.lock().await.insert(key, pr_data);
        }

        /// Set mock check runs for a commit SHA
        pub async fn set_check_runs(
            &self,
            owner: &str,
            repo: &str,
            sha: &str,
            runs: Vec<CheckRun>,
        ) {
            let key = format!("{}/{}/{}", owner, repo, sha);
            self.check_runs.lock().await.insert(key, runs);
        }

        /// Get mock pull request
        pub async fn get_pull_request(
            &self,
            owner: &str,
            repo: &str,
            pr_number: u64,
        ) -> Result<Value, GovernanceError> {
            let key = format!("{}/{}/{}", owner, repo, pr_number);
            self.pull_requests
                .lock()
                .await
                .get(&key)
                .cloned()
                .ok_or_else(|| {
                    GovernanceError::GitHubError(format!(
                        "Mock PR not found: {}/{}#{}",
                        owner, repo, pr_number
                    ))
                })
        }

        /// Get mock check runs
        pub async fn get_check_runs(
            &self,
            owner: &str,
            repo: &str,
            sha: &str,
        ) -> Result<Vec<CheckRun>, GovernanceError> {
            let key = format!("{}/{}/{}", owner, repo, sha);
            self.check_runs
                .lock()
                .await
                .get(&key)
                .cloned()
                .ok_or_else(|| {
                    GovernanceError::GitHubError(format!(
                        "Mock check runs not found: {}/{}@{}",
                        owner, repo, sha
                    ))
                })
        }
    }

    impl Default for MockGitHubClient {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Helper to create a mock PR response
    pub fn create_mock_pr_response(pr_number: u64, head_sha: &str) -> Value {
        serde_json::json!({
            "id": pr_number,
            "number": pr_number,
            "title": "Test PR",
            "body": "Test PR body",
            "state": "open",
            "head": {
                "sha": head_sha,
                "ref": "feature/test"
            },
            "base": {
                "sha": "base123",
                "ref": "main"
            }
        })
    }

    /// Helper to create mock check runs with test counts
    pub fn create_mock_check_runs_with_tests(
        test_count: usize,
        passed: usize,
        failed: usize,
    ) -> Vec<CheckRun> {
        let mut runs = Vec::new();

        // Add a test check run
        if test_count > 0 {
            runs.push(CheckRun {
                name: format!("Tests ({} passed, {} failed)", passed, failed),
                conclusion: if failed == 0 {
                    Some("success".to_string())
                } else {
                    Some("failure".to_string())
                },
                status: "completed".to_string(),
                html_url: Some("https://github.com/test/repo/actions/runs/123".to_string()),
            });
        }

        // Add other common check runs
        runs.push(CheckRun {
            name: "Lint".to_string(),
            conclusion: Some("success".to_string()),
            status: "completed".to_string(),
            html_url: Some("https://github.com/test/repo/actions/runs/124".to_string()),
        });

        runs
    }

    /// Helper to create mock check runs with various formats
    pub fn create_mock_check_runs_various_formats() -> Vec<CheckRun> {
        vec![
            CheckRun {
                name: "cargo test --lib (123 tests)".to_string(),
                conclusion: Some("success".to_string()),
                status: "completed".to_string(),
                html_url: None,
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
        ]
    }
}

/// Test data fixtures
pub mod fixtures {
    use super::*;

    pub async fn setup_test_database_with_data() -> Database {
        let db = setup_test_db().await;

        // Insert test maintainers
        let maintainers = create_test_maintainers();
        for (username, public_key, layer) in maintainers {
            // This would use actual database insertion in a real implementation
            // For now, we'll just return the database
        }

        // Insert test emergency keyholders
        let keyholders = create_test_emergency_keyholders();
        for (username, public_key) in keyholders {
            // This would use actual database insertion in a real implementation
        }

        db
    }
}
