use serde_json::Value;
use tracing::{info, warn};

use crate::crypto::signatures::SignatureManager;
use crate::database::Database;

pub async fn handle_comment_event(
    database: &Database,
    payload: &Value,
) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    let repo_name = payload
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    let pr_number = payload
        .get("issue")
        .and_then(|i| i.get("number"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);

    let commenter = payload
        .get("comment")
        .and_then(|c| c.get("user"))
        .and_then(|u| u.get("login"))
        .and_then(|l| l.as_str())
        .unwrap_or("unknown");

    let body = payload
        .get("comment")
        .and_then(|c| c.get("body"))
        .and_then(|b| b.as_str())
        .unwrap_or("");

    info!(
        "Comment by {} on PR #{} in {}",
        commenter, pr_number, repo_name
    );

    // Check for tier override command
    if body.starts_with("/governance-tier-override") {
        return handle_tier_override(database, repo_name, pr_number, commenter, body).await;
    }

    // Check for governance signature commands
    if body.starts_with("/governance-sign") {
        let remainder = body.strip_prefix("/governance-sign").unwrap_or("").trim();

        // Parse signature and optional reasoning
        // Format: /governance-sign <signature> "reasoning" or /governance-sign <signature>
        let (signature, reasoning) = if remainder.contains('"') {
            // Extract signature (before first quote) and reasoning (between quotes)
            if let Some(quote_start) = remainder.find('"') {
                let sig = remainder[..quote_start].trim();
                // Find the closing quote
                let after_quote = &remainder[quote_start + 1..];
                if let Some(quote_end) = after_quote.find('"') {
                    let reason = &after_quote[..quote_end];
                    (sig, Some(reason))
                } else {
                    // Unmatched quote - treat as signature only
                    (remainder.trim(), None)
                }
            } else {
                (remainder.trim(), None)
            }
        } else {
            (remainder.trim(), None)
        };

        if !signature.is_empty() {
            info!("Processing governance signature from {}", commenter);

            // Get maintainer public key from database
            let maintainer = match database.get_maintainer_by_username(commenter).await {
                Ok(Some(maintainer)) => maintainer,
                Ok(None) => {
                    warn!("User {} is not a registered maintainer", commenter);
                    return Ok(axum::response::Json(
                        serde_json::json!({"status": "not_maintainer", "error": "User is not a registered maintainer"}),
                    ));
                }
                Err(e) => {
                    warn!("Failed to get maintainer info: {}", e);
                    return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Verify signature using bllvm-sdk
            let signature_manager = SignatureManager::new();
            let message = format!("PR #{} in {}", pr_number, repo_name);

            match signature_manager.verify_governance_signature(
                &message,
                signature,
                &maintainer.public_key,
            ) {
                Ok(true) => {
                    info!("Valid signature from {} for PR #{}", commenter, pr_number);

                    // Store the verified signature with reasoning
                    match database
                        .add_signature(repo_name, pr_number as i32, commenter, signature, reasoning)
                        .await
                    {
                        Ok(_) => {
                            info!("Verified signature added for PR #{}", pr_number);

                            // Log governance event
                            let _ = database
                                .log_governance_event(
                                    "signature_collected",
                                    Some(repo_name),
                                    Some(pr_number as i32),
                                    Some(commenter),
                                    &serde_json::json!({
                                        "signature": signature,
                                        "message": message,
                                        "verified": true,
                                        "maintainer_layer": maintainer.layer,
                                        "reasoning": reasoning
                                    }),
                                )
                                .await;

                            Ok(axum::response::Json(
                                serde_json::json!({"status": "signature_verified", "verified": true}),
                            ))
                        }
                        Err(e) => {
                            warn!("Failed to add verified signature: {}", e);
                            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                }
                Ok(false) => {
                    warn!("Invalid signature from {} for PR #{}", commenter, pr_number);

                    // Log failed verification attempt
                    let _ = database
                        .log_governance_event(
                            "signature_verification_failed",
                            Some(repo_name),
                            Some(pr_number as i32),
                            Some(commenter),
                            &serde_json::json!({
                                "signature": signature,
                                "message": message,
                                "reason": "invalid_signature"
                            }),
                        )
                        .await;

                    Ok(axum::response::Json(
                        serde_json::json!({"status": "invalid_signature", "error": "Signature verification failed"}),
                    ))
                }
                Err(e) => {
                    warn!("Signature verification error: {}", e);
                    Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        } else {
            warn!("Empty signature provided by {}", commenter);
            Ok(axum::response::Json(
                serde_json::json!({"status": "empty_signature"}),
            ))
        }
    } else {
        info!("Non-governance comment, ignoring");
        Ok(axum::response::Json(
            serde_json::json!({"status": "ignored"}),
        ))
    }
}

/// Handle tier override command: /governance-tier-override <tier> "justification"
async fn handle_tier_override(
    database: &Database,
    repo_name: &str,
    pr_number: u64,
    commenter: &str,
    body: &str,
) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    use tracing::{info, warn};

    // Parse command: /governance-tier-override <tier> "justification"
    let remainder = body
        .strip_prefix("/governance-tier-override")
        .unwrap_or("")
        .trim();

    // Extract tier number and justification
    let parts: Vec<&str> = remainder.splitn(2, '"').collect();
    if parts.len() < 2 {
        warn!("Invalid tier override format. Expected: /governance-tier-override <tier> \"justification\"");
        return Ok(axum::response::Json(
            serde_json::json!({"status": "error", "error": "Invalid format. Use: /governance-tier-override <tier> \"justification\""}),
        ));
    }

    let tier_str = parts[0].trim();
    let justification = parts[1].trim_matches('"').trim();

    if justification.is_empty() {
        warn!("Empty justification provided for tier override");
        return Ok(axum::response::Json(
            serde_json::json!({"status": "error", "error": "Justification is required"}),
        ));
    }

    let override_tier: u32 = match tier_str.parse() {
        Ok(t) if (1..=5).contains(&t) => t,
        _ => {
            warn!("Invalid tier number: {}", tier_str);
            return Ok(axum::response::Json(
                serde_json::json!({"status": "error", "error": "Tier must be between 1 and 5"}),
            ));
        }
    };

    // Check if user is a maintainer
    let maintainer = match database.get_maintainer_by_username(commenter).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            warn!("User {} is not a registered maintainer", commenter);
            return Ok(axum::response::Json(
                serde_json::json!({"status": "not_maintainer", "error": "Only maintainers can override tiers"}),
            ));
        }
        Err(e) => {
            warn!("Failed to get maintainer info: {}", e);
            return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Store tier override
    match database
        .set_tier_override(
            repo_name,
            pr_number as i32,
            override_tier,
            justification,
            commenter,
        )
        .await
    {
        Ok(_) => {
            info!(
                "Tier override set to {} for PR #{} by {}",
                override_tier, pr_number, commenter
            );

            // Log governance event
            let _ = database
                .log_governance_event(
                    "tier_override",
                    Some(repo_name),
                    Some(pr_number as i32),
                    Some(commenter),
                    &serde_json::json!({
                        "override_tier": override_tier,
                        "justification": justification,
                        "maintainer_layer": maintainer.layer
                    }),
                )
                .await;

            Ok(axum::response::Json(serde_json::json!({
                "status": "tier_override_set",
                "override_tier": override_tier,
                "justification": justification
            })))
        }
        Err(e) => {
            warn!("Failed to set tier override: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
