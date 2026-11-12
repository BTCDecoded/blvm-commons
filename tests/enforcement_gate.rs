//! H3 and M5: signature intake, tier override, and release builds stay off
//! until merge checks are wired. Node registration does not record while
//! that gate is off.

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use blvm_commons::config::AppConfig;
use blvm_commons::database::Database;
use blvm_commons::node_registry::NodeRegistry;
use blvm_commons::node_registry::api::{RegisterNodeRequest, register_node};
use blvm_commons::webhooks::block::{BlockNotification, handle_block_notification};
use blvm_commons::webhooks::comment::handle_comment_event;
use blvm_commons::webhooks::github::handle_webhook;

fn config(wired: bool) -> AppConfig {
    let mut config = AppConfig::default();
    config.merge_checks_wired = wired;
    config.nostr.enabled = false;
    config
}

fn comment_payload(body: &str) -> serde_json::Value {
    serde_json::json!({
        "repository": {"full_name": "BTCDecoded/blvm-spec"},
        "issue": {"number": 9},
        "comment": {"user": {"login": "alice"}, "body": body}
    })
}

#[tokio::test]
async fn tier_override_does_not_record_when_gate_is_off() {
    let db = Database::new_in_memory().await.unwrap();
    db.insert_maintainer("alice", "02aa", 1).await.unwrap();
    db.create_pull_request("BTCDecoded/blvm-spec", 9, "abc", 1)
        .await
        .unwrap();

    let payload = comment_payload("/governance-tier-override 1 \"lower it\"");
    let off = handle_comment_event(&db, &payload, false).await.unwrap();
    assert_eq!(off.0["status"].as_str(), Some("enforcement_disabled"));
    assert!(
        db.get_tier_override("BTCDecoded/blvm-spec", 9)
            .await
            .unwrap()
            .is_none()
    );

    // Same maintainer and comment record when the gate is on, so the
    // negative assertion above is not an empty-database accident.
    let on = handle_comment_event(&db, &payload, true).await.unwrap();
    assert_eq!(on.0["status"].as_str(), Some("tier_override_set"));
    assert!(
        db.get_tier_override("BTCDecoded/blvm-spec", 9)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn signature_intake_does_not_record_when_gate_is_off() {
    let db = Database::new_in_memory().await.unwrap();
    db.insert_maintainer("alice", "02aa", 1).await.unwrap();
    db.create_pull_request("BTCDecoded/blvm-spec", 9, "abc", 1)
        .await
        .unwrap();

    let payload = comment_payload("/governance-sign deadbeef");
    let off = handle_comment_event(&db, &payload, false).await.unwrap();
    assert_eq!(off.0["status"].as_str(), Some("enforcement_disabled"));
    let stored = db
        .get_pull_request("BTCDecoded/blvm-spec", 9)
        .await
        .unwrap()
        .unwrap();
    assert!(stored.signatures.is_empty());
}

#[tokio::test]
async fn release_and_dispatch_do_not_build_when_gate_is_off() {
    let db = Database::new_in_memory().await.unwrap();
    let release = serde_json::json!({
        "action": "published",
        "release": {"tag_name": "v0.0.1", "prerelease": false}
    });
    let dispatch = serde_json::json!({
        "action": "build-complete",
        "client_payload": {"version": "v0.0.1"}
    });

    for (event, payload) in [("release", release), ("repository_dispatch", dispatch)] {
        let mut headers = HeaderMap::new();
        headers.insert("x-github-event", event.parse().unwrap());
        let (status, body) =
            handle_webhook(State((config(false), db.clone())), headers, Json(payload)).await;
        assert_eq!(status.as_u16(), 200, "{event}");
        assert_eq!(body.0["status"].as_str(), Some("enforcement_disabled"));
    }
    assert_eq!(db.count_build_runs().await.unwrap(), 0);
}

#[tokio::test]
async fn node_register_does_not_record_when_gate_is_off() {
    let db = Database::new_in_memory().await.unwrap();
    let request = RegisterNodeRequest {
        node_id: "node-1".to_string(),
        node_name: "one".to_string(),
        node_type: "full".to_string(),
        bitcoin_addresses: vec!["bc1qtest".to_string()],
        metadata: None,
    };
    let response = register_node(State((config(false), db.clone())), Json(request)).await;
    assert!(!response.0.success);
    let pool = db.get_sqlite_pool().unwrap().clone();
    let found = NodeRegistry::new(pool).get_node("node-1").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn block_webhook_does_not_accept_when_gate_is_off() {
    let db = Database::new_in_memory().await.unwrap();
    let response = handle_block_notification(
        State((config(false), db)),
        Json(BlockNotification {
            block_hash: "00".to_string(),
            block_height: 1,
            block: serde_json::json!({}),
            contributor_id: None,
        }),
    )
    .await;
    assert!(!response.0.success);
    assert_eq!(response.0.contributions_found, 0);
}
