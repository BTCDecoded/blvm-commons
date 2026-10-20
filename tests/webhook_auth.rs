//! GitHub webhook authentication (C1).
//!
//! A missing or wrong `X-Hub-Signature-256` must be rejected before a handler
//! records a pull request. A placeholder secret must not boot the server and
//! must not verify, even when the HMAC under that placeholder is correct.

use blvm_commons::config::{AppConfig, require_webhook_secret};
use blvm_commons::database::Database;
use blvm_commons::webhooks::github::{dispatch_github_webhook, sign_github_body};

const SECRET: &str = "test-webhook-secret-not-a-placeholder";

fn opened_pr_body() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "action": "opened",
        "repository": {"full_name": "BTCDecoded/blvm-spec"},
        "pull_request": {
            "number": 7,
            "merged": false,
            "head": {"sha": "abc123"}
        }
    }))
    .unwrap()
}

fn config_with_secret(secret: &str) -> AppConfig {
    let mut config = AppConfig::default();
    config.github_webhook_secret = secret.to_string();
    config.nostr.enabled = false;
    config
}

#[test]
fn placeholder_secret_refuses_boot() {
    assert!(require_webhook_secret("").is_err());
    assert!(require_webhook_secret("   ").is_err());
    assert!(require_webhook_secret("your_webhook_secret_here").is_err());
    assert!(require_webhook_secret("your-webhook-secret-here").is_err());
    assert!(require_webhook_secret("  your_webhook_secret_here  ").is_err());
    assert!(require_webhook_secret(SECRET).is_ok());

    let defaults = AppConfig::default();
    assert!(
        require_webhook_secret(&defaults.github_webhook_secret).is_err(),
        "the shipped default must not be usable to boot"
    );
}

#[tokio::test]
async fn missing_signature_does_not_record_pull_request() {
    let db = Database::new_in_memory().await.unwrap();
    let body = opened_pr_body();
    let (status, response) = dispatch_github_webhook(
        config_with_secret(SECRET),
        db.clone(),
        "pull_request",
        None,
        &body,
    )
    .await;

    assert_eq!(status.as_u16(), 401);
    assert_eq!(response.0["error"].as_str(), Some("invalid signature"));
    assert!(
        db.get_pull_request("BTCDecoded/blvm-spec", 7)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn wrong_signature_does_not_record_pull_request() {
    let db = Database::new_in_memory().await.unwrap();
    let body = opened_pr_body();
    let (status, _) = dispatch_github_webhook(
        config_with_secret(SECRET),
        db.clone(),
        "pull_request",
        Some("sha256=deadbeef"),
        &body,
    )
    .await;

    assert_eq!(status.as_u16(), 401);
    assert!(
        db.get_pull_request("BTCDecoded/blvm-spec", 7)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn placeholder_secret_does_not_verify_even_with_matching_hmac() {
    let db = Database::new_in_memory().await.unwrap();
    let body = opened_pr_body();
    let placeholder = "your_webhook_secret_here";
    let signature = sign_github_body(placeholder, &body);
    let (status, _) = dispatch_github_webhook(
        config_with_secret(placeholder),
        db.clone(),
        "pull_request",
        Some(&signature),
        &body,
    )
    .await;

    assert_eq!(status.as_u16(), 401);
    assert!(
        db.get_pull_request("BTCDecoded/blvm-spec", 7)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn valid_signature_reaches_the_pull_request_handler() {
    let db = Database::new_in_memory().await.unwrap();
    let body = opened_pr_body();
    let signature = sign_github_body(SECRET, &body);
    let (status, response) = dispatch_github_webhook(
        config_with_secret(SECRET),
        db.clone(),
        "pull_request",
        Some(&signature),
        &body,
    )
    .await;

    assert_eq!(status.as_u16(), 200);
    assert_eq!(response.0["status"].as_str(), Some("stored"));
    let stored = db
        .get_pull_request("BTCDecoded/blvm-spec", 7)
        .await
        .unwrap()
        .expect("a valid signature must reach the handler");
    assert_eq!(stored.head_sha, "abc123");
}
