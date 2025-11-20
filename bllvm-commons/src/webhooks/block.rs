//! Block webhook handler for fee forwarding integration
//!
//! Receives block notifications from bllvm-node and processes them for fee forwarding

use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

use crate::config::AppConfig;
use crate::database::Database;
use crate::governance::FeeForwardingTracker;

/// Block notification payload
/// Block should be provided as JSON object that can be deserialized to bllvm_protocol::Block
#[derive(Debug, Deserialize)]
pub struct BlockNotification {
    pub block_hash: String,
    pub block_height: i32,
    pub block: Value, // Block data as JSON - will be converted to bllvm_protocol::Block
    pub contributor_id: Option<String>, // Optional: node/miner identifier
}

/// Block notification response
#[derive(Debug, Serialize)]
pub struct BlockNotificationResponse {
    pub success: bool,
    pub message: String,
    pub contributions_found: usize,
}

/// Handle block notification webhook
pub async fn handle_block_notification(
    State((config, database)): State<(AppConfig, Database)>,
    Json(payload): Json<BlockNotification>,
) -> Json<BlockNotificationResponse> {
    // Check if fee forwarding is enabled
    if config.governance.commons_addresses.is_empty() {
        return Json(BlockNotificationResponse {
            success: false,
            message: "Fee forwarding not configured (no Commons addresses)".to_string(),
            contributions_found: 0,
        });
    }

    let pool = match database.get_sqlite_pool() {
        Some(pool) => pool,
        None => {
            return Json(BlockNotificationResponse {
                success: false,
                message: "Database pool not available".to_string(),
                contributions_found: 0,
            });
        }
    };

    // Parse block from JSON payload
    // The block field contains the block data as JSON
    let block: bllvm_protocol::Block = match serde_json::from_value(payload.block.clone()) {
        Ok(parsed) => parsed,
        Err(e) => {
            error!("Failed to parse block from JSON: {}", e);
            return Json(BlockNotificationResponse {
                success: false,
                message: format!("Failed to parse block: {}", e),
                contributions_found: 0,
            });
        }
    };

    // Initialize fee forwarding tracker
    let tracker = FeeForwardingTracker::from_network_string(
        pool.clone(),
        config.governance.commons_addresses.clone(),
        &config.governance.network,
    );

    // Process block
    match tracker
        .process_block(
            &block,
            payload.block_height,
            payload.contributor_id.as_deref(),
        )
        .await
    {
        Ok(contributions) => {
            info!(
                "Processed block {} at height {}: found {} fee forwarding contributions",
                payload.block_hash,
                payload.block_height,
                contributions.len()
            );

            Json(BlockNotificationResponse {
                success: true,
                message: "Block processed successfully".to_string(),
                contributions_found: contributions.len(),
            })
        }
        Err(e) => {
            error!("Failed to process block {}: {}", payload.block_hash, e);
            Json(BlockNotificationResponse {
                success: false,
                message: format!("Failed to process block: {}", e),
                contributions_found: 0,
            })
        }
    }
}
