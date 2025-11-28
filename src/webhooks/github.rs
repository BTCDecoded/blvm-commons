use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde_json::Value;
use tracing::{info, warn};

use crate::build::orchestrator::BuildOrchestrator;
use crate::github::client::GitHubClient;
use crate::webhooks::{comment, pull_request, release, review};

pub async fn handle_webhook(
    State((config, database)): State<(crate::config::AppConfig, crate::database::Database)>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> (StatusCode, Json<Value>) {
    // Get event type from GitHub webhook header
    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

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
