use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use tracing::{info, warn};

use crate::build::orchestrator::BuildOrchestrator;
use crate::config::{self, AppConfig};
use crate::database::Database;
use crate::github::client::GitHubClient;
use crate::webhooks::{comment, pull_request, release, review};

type HmacSha256 = Hmac<Sha256>;

/// Constant-time check of `X-Hub-Signature-256` over the raw request body.
/// An unusable secret never verifies, including the committed placeholders.
pub fn verify_github_signature(secret: &str, body: &[u8], signature_header: Option<&str>) -> bool {
    if config::require_webhook_secret(secret).is_err() {
        return false;
    }
    let Some(header) = signature_header else {
        return false;
    };
    let Some(encoded) = header.strip_prefix("sha256=") else {
        return false;
    };
    let Ok(presented) = hex::decode(encoded) else {
        return false;
    };
    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);
    mac.verify_slice(&presented).is_ok()
}

pub fn sign_github_body(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac-sha256 accepts any key");
    mac.update(body);
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

pub async fn handle_webhook(
    State((config, database)): State<(AppConfig, Database)>,
    headers: HeaderMap,
    body: Bytes,
) -> (StatusCode, Json<Value>) {
    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok());
    dispatch_github_webhook(config, database, event_type, signature, &body).await
}

/// Verify the signature, then parse and dispatch. Signature failure returns
/// before JSON parsing and before any event handler.
pub async fn dispatch_github_webhook(
    config: AppConfig,
    database: Database,
    event_type: &str,
    signature_header: Option<&str>,
    body: &[u8],
) -> (StatusCode, Json<Value>) {
    if !verify_github_signature(&config.github_webhook_secret, body, signature_header) {
        warn!("Rejected GitHub webhook: missing or invalid X-Hub-Signature-256");
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid signature"})),
        );
    }

    let payload: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid json"})),
            );
        }
    };

    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    info!(
        "Received webhook: event_type={}, action={}",
        event_type, action
    );

    match event_type {
        "pull_request" => {
            match action {
                "opened" | "synchronize" | "reopened" => {
                    match pull_request::handle_pull_request_event(&config, &database, &payload)
                        .await
                    {
                        Ok(response) => (StatusCode::OK, response),
                        Err(status) => (status, Json(serde_json::json!({"error": "failed"}))),
                    }
                }
                "closed" => {
                    // Check if PR was merged
                    let merged = payload
                        .get("pull_request")
                        .and_then(|pr| pr.get("merged"))
                        .and_then(|m| m.as_bool())
                        .unwrap_or(false);

                    if merged {
                        // PR was merged - publish to Nostr
                        if let Err(e) =
                            pull_request::handle_pr_merged(&config, &database, &payload).await
                        {
                            warn!("Failed to publish merge to Nostr: {}", e);
                        }
                    }

                    (
                        StatusCode::OK,
                        Json(serde_json::json!({"status": "processed"})),
                    )
                }
                _ => {
                    warn!("Unhandled pull_request action: {}", action);
                    (
                        StatusCode::OK,
                        Json(serde_json::json!({"status": "ignored"})),
                    )
                }
            }
        }
        "pull_request_review" => match review::handle_review_event(&database, &payload).await {
            Ok(response) => (StatusCode::OK, response),
            Err(status) => (status, Json(serde_json::json!({"error": "failed"}))),
        },
        "issue_comment" => match comment::handle_comment_event(&database, &payload).await {
            Ok(response) => (StatusCode::OK, response),
            Err(status) => (status, Json(serde_json::json!({"error": "failed"}))),
        },
        "release" => {
            // Initialize build orchestrator
            let github_client =
                match GitHubClient::new(config.github_app_id, &config.github_private_key_path) {
                    Ok(client) => client,
                    Err(e) => {
                        warn!("Failed to create GitHub client: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "failed to initialize"})),
                        );
                    }
                };

            // Extract organization from governance_repo (format: "org/repo")
            let organization = config
                .governance_repo
                .split('/')
                .next()
                .unwrap_or("BTCDecoded")
                .to_string();

            let orchestrator = BuildOrchestrator::new(github_client, database, organization);

            match release::handle_release_event(&payload, &orchestrator).await {
                Ok((status, response)) => (status, Json(response)),
                Err(e) => {
                    warn!("Failed to handle release event: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "failed"})),
                    )
                }
            }
        }
        "repository_dispatch" => {
            // Initialize build orchestrator
            let github_client =
                match GitHubClient::new(config.github_app_id, &config.github_private_key_path) {
                    Ok(client) => client,
                    Err(e) => {
                        warn!("Failed to create GitHub client: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "failed to initialize"})),
                        );
                    }
                };

            // Extract organization from governance_repo (format: "org/repo")
            let organization = config
                .governance_repo
                .split('/')
                .next()
                .unwrap_or("BTCDecoded")
                .to_string();

            // Clone database before moving it to orchestrator
            let database_clone = database.clone();
            let orchestrator =
                BuildOrchestrator::new(github_client, database_clone.clone(), organization);

            match release::handle_repository_dispatch(&payload, &orchestrator, &database_clone)
                .await
            {
                Ok((status, response)) => (status, Json(response)),
                Err(e) => {
                    warn!("Failed to handle repository_dispatch: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "failed"})),
                    )
                }
            }
        }
        _ => {
            warn!("Unhandled webhook event type: {}", event_type);
            (
                StatusCode::OK,
                Json(serde_json::json!({"status": "ignored", "event_type": event_type})),
            )
        }
    }
}
