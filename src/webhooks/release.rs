//! Release event webhook handler

use serde_json::Value;
use tracing::{info, error, warn};
use axum::http::StatusCode;

use crate::error::GovernanceError;
use crate::build::orchestrator::BuildOrchestrator;
use crate::github::client::GitHubClient;
use crate::database::Database;
use crate::config::AppConfig;

/// Handle release webhook events
pub async fn handle_release_event(
    payload: &Value,
    orchestrator: &BuildOrchestrator,
) -> Result<(StatusCode, Value), GovernanceError> {
    info!("Handling release event");
    
    // Extract release information
    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    let release = payload
        .get("release")
        .ok_or_else(|| GovernanceError::GitHubError("Missing release in payload".to_string()))?;
    
    let tag_name = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| GovernanceError::GitHubError("Missing tag_name in release".to_string()))?;
    
    let prerelease = release
        .get("prerelease")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    info!("Release event: action={}, tag={}, prerelease={}", action, tag_name, prerelease);
    
    // Only handle published releases
    if action != "published" {
        info!("Ignoring release event with action: {}", action);
        return Ok((
            StatusCode::OK,
            serde_json::json!({"status": "ignored", "reason": format!("Action {} not handled", action)}),
        ));
    }
    
    // Trigger build orchestration
    match orchestrator.handle_release_event(tag_name, prerelease).await {
        Ok(_) => {
            info!("Successfully orchestrated builds for release {}", tag_name);
            Ok((
                StatusCode::OK,
                serde_json::json!({
                    "status": "success",
                    "message": format!("Build orchestration started for {}", tag_name),
                }),
            ))
        }
        Err(e) => {
            error!("Failed to orchestrate builds for release {}: {}", tag_name, e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                serde_json::json!({
                    "status": "error",
                    "message": format!("Failed to orchestrate builds: {}", e),
                }),
            ))
        }
    }
}

/// Handle repository_dispatch events (build completion notifications)
pub async fn handle_repository_dispatch(
    payload: &Value,
    _orchestrator: &BuildOrchestrator,
) -> Result<(StatusCode, Value), GovernanceError> {
    info!("Handling repository_dispatch event");
    
    let action = payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    let client_payload = payload
        .get("client_payload")
        .and_then(|v| v.as_object())
        .ok_or_else(|| GovernanceError::GitHubError("Missing client_payload".to_string()))?;
    
    info!("Repository dispatch: action={}, payload={:?}", action, client_payload);
    
    // Handle different dispatch event types
    match action {
        "build-complete" => {
            let repo = client_payload
                .get("repo")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let status = client_payload
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            
            info!("Build completed for {}: {}", repo, status);
            
            // TODO: Update build state in database
            // TODO: Check if all builds are complete
            // TODO: Proceed to next step (artifact collection, etc.)
        }
        "build-request" => {
            // This is handled by the workflow, not the governance app
            info!("Build request received (handled by workflow)");
        }
        _ => {
            warn!("Unknown repository_dispatch action: {}", action);
        }
    }
    
    Ok((
        StatusCode::OK,
        serde_json::json!({"status": "received"}),
    ))
}

