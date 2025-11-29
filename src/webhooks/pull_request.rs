use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::database::Database;
use crate::governance::ConfigRegistry;
use crate::nostr::publish_merge_action;
use crate::validation::threshold::ThresholdValidator;
use crate::validation::tier_classification;

pub async fn handle_pull_request_event(
    config: &AppConfig,
    database: &Database,
    payload: &Value,
) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    let repo_name = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    let pr_number = payload
        .get("pull_request")
        .and_then(|pr| pr.get("number"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);

    let head_sha = payload
        .get("pull_request")
        .and_then(|pr| pr.get("head").and_then(|h| h.get("sha")))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    info!("Processing PR #{} in {}", pr_number, repo_name);

    // Determine layer based on repository
    // Check more specific patterns first to avoid false matches
    let layer = match repo_name {
        repo if repo.contains("bllvm-spec") || repo.contains("orange-paper") => 1,
        repo if repo.contains("bllvm-consensus") || repo.contains("consensus-proof") => 2,
        repo if repo.contains("bllvm-protocol") || repo.contains("protocol-engine") => 3,
        repo if repo.contains("bllvm-sdk") || repo.contains("developer-sdk") => 5,
        repo if repo.contains("bllvm-commons") || repo.contains("governance-app") => 6,
        repo if repo.contains("bllvm-node")
            || repo.contains("reference-node")
            || repo.contains("/bllvm") =>
        {
            4
        }
        _ => {
            warn!("Unknown repository: {}", repo_name);
            return Ok(axum::response::Json(
                serde_json::json!({"status": "unknown_repo"}),
            ));
        }
    };

    // Classify PR tier based on file changes (check for override first)
    let tier = tier_classification::classify_pr_tier_with_db(
        database,
        payload,
        repo_name,
        pr_number as i32,
    )
    .await;
    info!("PR #{} classified as Tier {}", pr_number, tier);

    // Store PR in database
    match database
        .create_pull_request(repo_name, pr_number as i32, head_sha, layer)
        .await
    {
        Ok(_) => {
            info!("PR #{} stored in database", pr_number);

            // Log governance event
            let _ = database
                .log_governance_event(
                    "pr_opened",
                    Some(repo_name),
                    Some(pr_number as i32),
                    None,
                    &serde_json::json!({
                        "tier": tier,
                        "layer": layer,
                        "head_sha": head_sha
                    }),
                )
                .await;

            // Publish review period notification to Nostr (if enabled)
            if config.nostr.enabled {
                // Calculate review period end date
                // Note: Using static method for backward compatibility
                // TODO: Update to use ConfigReader-enabled validator when available
                let review_days = ThresholdValidator::get_combined_requirements_static(layer, tier).2;
                let review_period_ends =
                    chrono::Utc::now() + chrono::Duration::days(review_days as i64);

                // Publish review period notification
                if let Err(e) = crate::nostr::helpers::publish_review_period_notification(
                    config,
                    repo_name,
                    pr_number as i32,
                    layer,
                    tier,
                    review_period_ends,
                )
                .await
                {
                    warn!(
                        "Failed to publish review period notification to Nostr: {}",
                        e
                    );
                    // Don't fail the webhook if Nostr publishing fails
                }
            }

            Ok(axum::response::Json(serde_json::json!({
                "status": "stored",
                "tier": tier,
                "layer": layer
            })))
        }
        Err(e) => {
            warn!("Failed to store PR: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Handle PR merge event - publish to Nostr and activate config changes if Tier 5
pub async fn handle_pr_merged(
    config: &AppConfig,
    database: &Database,
    payload: &Value,
    config_registry: Option<Arc<ConfigRegistry>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_name = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    let pr_number = payload
        .get("pull_request")
        .and_then(|pr| pr.get("number"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0) as i32;

    let commit_hash = payload
        .get("pull_request")
        .and_then(|pr| pr.get("merge_commit_sha"))
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    info!(
        "PR #{} merged in {}, publishing to Nostr",
        pr_number, repo_name
    );

    // Get PR info to determine layer and tier
    let pr_info = database.get_pull_request(repo_name, pr_number).await?;

    if let Some(pr) = pr_info {
        let layer = pr.layer;

        // Get tier from database or re-classify
        // For now, we'll need to get it from the PR details or re-classify
        // This is a simplified version - in practice, tier should be stored with PR
        let tier =
            tier_classification::classify_pr_tier_with_db(database, payload, repo_name, pr_number)
                .await;

        // Publish merge action to Nostr
        publish_merge_action(
            config,
            database,
            repo_name,
            pr_number,
            commit_hash,
            layer,
            tier,
        )
        .await?;

        info!("Successfully published merge action to Nostr");

        // If this is a Tier 5 PR, activate any approved configuration changes
        if tier == 5 {
            if let Some(ref registry) = config_registry {
                let pr_body = payload
                    .get("pull_request")
                    .and_then(|pr| pr.get("body"))
                    .and_then(|b| b.as_str())
                    .unwrap_or("");

                match registry.process_pr_config_changes(pr_number, pr_body).await {
                    Ok(activated_keys) => {
                        if !activated_keys.is_empty() {
                            info!(
                                "Activated {} configuration changes from Tier 5 PR #{}: {:?}",
                                activated_keys.len(),
                                pr_number,
                                activated_keys
                            );
                            // Note: Cache invalidation should be handled by ConfigReader
                            // when it detects the new values on next read, or we could
                            // pass ConfigReader here to invalidate immediately
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to process config changes for PR #{}: {}",
                            pr_number, e
                        );
                    }
                }
            }
        }
    } else {
        warn!(
            "PR #{} not found in database, cannot publish to Nostr",
            pr_number
        );
    }

    Ok(())
}

/// Determine layer from repository name
pub fn determine_layer(repo_name: &str) -> Option<i32> {
    // Check more specific patterns first to avoid false matches
    if repo_name.contains("bllvm-spec") || repo_name.contains("orange-paper") {
        Some(1)
    } else if repo_name.contains("bllvm-consensus") || repo_name.contains("consensus-proof") {
        Some(2)
    } else if repo_name.contains("bllvm-protocol") || repo_name.contains("protocol-engine") {
        Some(3)
    } else if repo_name.contains("bllvm-sdk") || repo_name.contains("developer-sdk") {
        Some(5)
    } else if repo_name.contains("bllvm-commons") || repo_name.contains("governance-app") {
        Some(6)
    } else if repo_name.contains("bllvm-node")
        || repo_name.contains("reference-node")
        || repo_name.contains("/bllvm")
    {
        Some(4)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_layer_spec() {
        assert_eq!(determine_layer("BTCDecoded/bllvm-spec"), Some(1));
        assert_eq!(determine_layer("BTCDecoded/orange-paper"), Some(1));
    }

    #[test]
    fn test_determine_layer_consensus() {
        assert_eq!(determine_layer("BTCDecoded/bllvm-consensus"), Some(2));
        assert_eq!(determine_layer("BTCDecoded/consensus-proof"), Some(2));
    }

    #[test]
    fn test_determine_layer_protocol() {
        assert_eq!(determine_layer("BTCDecoded/bllvm-protocol"), Some(3));
        assert_eq!(determine_layer("BTCDecoded/protocol-engine"), Some(3));
    }

    #[test]
    fn test_determine_layer_node() {
        assert_eq!(determine_layer("BTCDecoded/bllvm-node"), Some(4));
        assert_eq!(determine_layer("BTCDecoded/reference-node"), Some(4));
        assert_eq!(determine_layer("BTCDecoded/bllvm"), Some(4));
    }

    #[test]
    fn test_determine_layer_sdk() {
        assert_eq!(determine_layer("BTCDecoded/bllvm-sdk"), Some(5));
        assert_eq!(determine_layer("BTCDecoded/developer-sdk"), Some(5));
    }

    #[test]
    fn test_determine_layer_commons() {
        assert_eq!(determine_layer("BTCDecoded/bllvm-commons"), Some(6));
        assert_eq!(determine_layer("BTCDecoded/governance-app"), Some(6));
    }

    #[test]
    fn test_determine_layer_unknown() {
        assert_eq!(determine_layer("Unknown/Repository"), None);
        assert_eq!(determine_layer(""), None);
    }
}
