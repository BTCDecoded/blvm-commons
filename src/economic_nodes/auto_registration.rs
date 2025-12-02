//! Automatic Registration for Commons Contributors
//!
//! Automatically registers Commons Contributors as economic nodes when their
//! contributions meet qualification thresholds. Uses tracked contribution data
//! to generate proofs and register without manual intervention.

use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use std::sync::Arc;
use tracing::{info, warn};

use super::registry::EconomicNodeRegistry;
use super::types::*;
use crate::error::GovernanceError;
use crate::governance::contributions::{ContributionTracker, ContributorTotal};
use crate::nostr::zap_tracker::ZapTracker;

/// Automatic registration service for Commons Contributors
pub struct CommonsContributorAutoRegistrar {
    pool: SqlitePool,
    registry: Arc<EconomicNodeRegistry>,
    contribution_tracker: ContributionTracker,
    zap_tracker: Option<Arc<ZapTracker>>,
}

impl CommonsContributorAutoRegistrar {
    /// Create a new auto-registrar
    pub fn new(
        pool: SqlitePool,
        registry: Arc<EconomicNodeRegistry>,
        zap_tracker: Option<Arc<ZapTracker>>,
    ) -> Self {
        Self {
            pool: pool.clone(),
            registry,
            contribution_tracker: ContributionTracker::new(pool),
            zap_tracker,
        }
    }

    /// Check if contributor qualifies and auto-register
    /// Called automatically when contributions are recorded
    pub async fn check_and_register(
        &self,
        contributor_id: &str,
        contributor_type: &str, // "merge_miner", "fee_forwarder", "zap_user"
    ) -> Result<Option<i32>, GovernanceError> {
        // 1. Get contributions in 90-day window
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(90);
        let total = self
            .contribution_tracker
            .get_contributor_total(contributor_id, start_time, end_time)
            .await
            .map_err(|e| GovernanceError::DatabaseError(format!("Failed to get contributions: {}", e)))?;

        // 2. Check if already registered
        if let Some(existing) = self
            .get_node_by_contributor_id(contributor_id)
            .await?
        {
            info!(
                "Contributor {} already registered as economic node {}",
                contributor_id, existing
            );
            return Ok(Some(existing));
        }

        // 3. Check thresholds
        let qualified = self.check_qualification(&total).await?;
        if !qualified {
            return Ok(None); // Doesn't meet thresholds yet
        }

        info!(
            "Contributor {} meets qualification thresholds, auto-registering...",
            contributor_id
        );

        // 4. Auto-generate proof from tracked contributions
        let proof = self
            .generate_proof_from_contributions(contributor_id, contributor_type, &total, start_time, end_time)
            .await?;

        // 5. Get or generate public key
        let public_key = self
            .get_or_generate_public_key(contributor_id, contributor_type)
            .await?;

        // 6. Get entity name
        let entity_name = self
            .get_entity_name(contributor_id, contributor_type)
            .await?;

        // 7. Auto-register
        let node_id = self
            .registry
            .register_economic_node(
                NodeType::CommonsContributor,
                &entity_name,
                &public_key,
                &proof,
                Some("auto_registration"),
            )
            .await?;

        info!(
            "Auto-registered Commons Contributor {} as economic node {}",
            contributor_id, node_id
        );

        Ok(Some(node_id))
    }

    /// Check if contributor meets qualification thresholds
    async fn check_qualification(&self, total: &ContributorTotal) -> Result<bool, GovernanceError> {
        // Get thresholds from registry
        let thresholds_config = self
            .registry
            .get_commons_contributor_thresholds()?;

        let mut qualifications_met = Vec::new();

        // Check merge mining threshold
        if let Some(ref config) = thresholds_config {
            if config.commons_contributor_thresholds.merge_mining.enabled {
                if total.merge_mining_btc
                    >= config.commons_contributor_thresholds.merge_mining.minimum_contribution_btc
                {
                    qualifications_met.push("merge_mining");
                }
            }

            // Check fee forwarding threshold
            if config.commons_contributor_thresholds.fee_forwarding.enabled {
                if total.fee_forwarding_btc
                    >= config.commons_contributor_thresholds.fee_forwarding.minimum_contribution_btc
                {
                    qualifications_met.push("fee_forwarding");
                }
            }

            // Check zap threshold
            if config.commons_contributor_thresholds.zaps.enabled {
                if total.zaps_btc >= config.commons_contributor_thresholds.zaps.minimum_contribution_btc {
                    qualifications_met.push("zaps");
                }
            }
        }

        // Apply qualification logic (OR by default)
        let logic = thresholds_config
            .as_ref()
            .map(|c| c.commons_contributor_thresholds.qualification_logic.as_str())
            .unwrap_or("OR");

        let qualified = match logic {
            "AND" => {
                // Count enabled thresholds
                let enabled_count = thresholds_config
                    .as_ref()
                    .map(|c| {
                        [
                            c.commons_contributor_thresholds.merge_mining.enabled,
                            c.commons_contributor_thresholds.fee_forwarding.enabled,
                            c.commons_contributor_thresholds.zaps.enabled,
                        ]
                        .iter()
                        .filter(|&&enabled| enabled)
                        .count()
                    })
                    .unwrap_or(0);
                qualifications_met.len() == enabled_count
            }
            _ => !qualifications_met.is_empty(), // OR logic
        };

        Ok(qualified)
    }

    /// Generate proof from tracked contributions
    async fn generate_proof_from_contributions(
        &self,
        contributor_id: &str,
        contributor_type: &str,
        total: &ContributorTotal,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
    ) -> Result<QualificationProof, GovernanceError> {
        let mut commons_proof = CommonsContributorProof {
            merge_mining_proof: None,
            fee_forwarding_proof: None,
            zap_proof: None,
            marketplace_sales_proof: None,
        };

        // Generate merge mining proof if contributions exist
        if total.merge_mining_btc > 0.0 {
            commons_proof.merge_mining_proof = Some(
                self.generate_merge_mining_proof(contributor_id, start_time, end_time)
                    .await?,
            );
        }

        // Generate fee forwarding proof if contributions exist
        if total.fee_forwarding_btc > 0.0 {
            commons_proof.fee_forwarding_proof = Some(
                self.generate_fee_forwarding_proof(contributor_id, start_time, end_time)
                    .await?,
            );
        }

        // Generate zap proof if contributions exist
        if total.zaps_btc > 0.0 {
            commons_proof.zap_proof = Some(
                self.generate_zap_proof(contributor_id, start_time, end_time)
                    .await?,
            );
        }

        Ok(QualificationProof {
            node_type: NodeType::CommonsContributor,
            hashpower_proof: None,
            holdings_proof: None,
            volume_proof: None,
            contact_info: ContactInfo {
                entity_name: self.get_entity_name(contributor_id, contributor_type).await?,
                contact_email: format!("{}@commons.auto", contributor_id),
                website: None,
                github_username: None,
            },
            commons_contributor_proof: Some(commons_proof),
        })
    }

    /// Generate merge mining proof from tracked data
    async fn generate_merge_mining_proof(
        &self,
        contributor_id: &str,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
    ) -> Result<MergeMiningProof, GovernanceError> {
        // Query unified_contributions for merge mining data
        let rows = sqlx::query(
            r#"
            SELECT contribution_type, amount_btc, timestamp
            FROM unified_contributions
            WHERE contributor_id = ?
              AND contribution_type LIKE 'merge_mining:%'
              AND timestamp >= ?
              AND timestamp <= ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(contributor_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to query merge mining: {}", e)))?;

        let mut total_revenue_btc = 0.0;
        let mut blocks_mined = Vec::new();

        for row in rows {
            let amount: f64 = row.get("amount_btc");
            total_revenue_btc += amount;
            // TODO: Query actual block data from merge_mining_contributions table if it exists
            // For now, create placeholder block proof
            blocks_mined.push(MergeMiningBlockProof {
                block_hash: format!("block_{}", row.get::<i64, _>("id")),
                chain_id: "unknown".to_string(), // Extract from contribution_type
                commons_fee_amount: (amount * 100_000_000.0) as u64, // Convert to satoshis
                coinbase_signature: format!("sig_{}", contributor_id),
            });
        }

        let period_days = (end_time - start_time).num_days() as u32;

        Ok(MergeMiningProof {
            total_revenue_btc,
            period_days,
            blocks_mined,
            contributor_id: contributor_id.to_string(),
        })
    }

    /// Generate fee forwarding proof from tracked data
    async fn generate_fee_forwarding_proof(
        &self,
        contributor_id: &str,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
    ) -> Result<FeeForwardingProof, GovernanceError> {
        // Query fee_forwarding_contributions table
        let rows = sqlx::query(
            r#"
            SELECT tx_hash, block_height, amount_btc, commons_address, timestamp
            FROM fee_forwarding_contributions
            WHERE contributor_id = ?
              AND timestamp >= ?
              AND timestamp <= ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(contributor_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to query fee forwarding: {}", e))
        })?;

        let mut total_fees_forwarded_btc = 0.0;
        let mut blocks_with_forwarding = Vec::new();

        for row in rows {
            let amount: f64 = row.get("amount_btc");
            total_fees_forwarded_btc += amount;
            blocks_with_forwarding.push(FeeForwardingBlockProof {
                block_hash: format!("block_{}", row.get::<i32, _>("block_height")),
                block_height: row.get("block_height"),
                forwarded_amount: (amount * 100_000_000.0) as u64, // Convert to satoshis
                commons_address: row.get("commons_address"),
                tx_hash: row.get("tx_hash"),
            });
        }

        let period_days = (end_time - start_time).num_days() as u32;

        Ok(FeeForwardingProof {
            total_fees_forwarded_btc: total_fees_forwarded_btc,
            period_days,
            blocks_with_forwarding,
            contributor_id: contributor_id.to_string(),
        })
    }

    /// Generate zap proof from tracked data
    async fn generate_zap_proof(
        &self,
        contributor_id: &str,
        start_time: chrono::DateTime<Utc>,
        end_time: chrono::DateTime<Utc>,
    ) -> Result<ZapProof, GovernanceError> {
        // Query zap_contributions table
        let rows = sqlx::query(
            r#"
            SELECT zapped_event_id, amount_btc, invoice_hash, timestamp
            FROM zap_contributions
            WHERE sender_pubkey = ?
              AND timestamp >= ?
              AND timestamp <= ?
            ORDER BY timestamp ASC
            "#,
        )
        .bind(contributor_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to query zaps: {}", e)))?;

        let mut total_zaps_btc = 0.0;
        let mut zap_events = Vec::new();

        for row in rows {
            let amount_btc: f64 = row.get("amount_btc");
            total_zaps_btc += amount_btc;
            zap_events.push(ZapEventProof {
                nostr_event_id: row.get("zapped_event_id").unwrap_or_else(|| "unknown".to_string()),
                zap_amount: (amount_btc * 100_000_000.0) as u64, // Convert to satoshis
                payment_hash: row.get("invoice_hash").unwrap_or_else(|| "unknown".to_string()),
                timestamp: row
                    .get::<chrono::DateTime<Utc>, _>("timestamp")
                    .timestamp(),
            });
        }

        let period_days = (end_time - start_time).num_days() as u32;

        Ok(ZapProof {
            total_zaps_btc,
            period_days,
            zap_events,
            contributor_id: contributor_id.to_string(), // Nostr pubkey
        })
    }

    /// Get or generate public key for contributor
    async fn get_or_generate_public_key(
        &self,
        contributor_id: &str,
        contributor_type: &str,
    ) -> Result<String, GovernanceError> {
        // Check if key already exists in contributor_keys table
        let existing: Option<String> = sqlx::query_scalar(
            r#"
            SELECT public_key FROM contributor_keys WHERE contributor_id = ?
            "#,
        )
        .bind(contributor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to query contributor keys: {}", e))
        })?;

        if let Some(key) = existing {
            return Ok(key);
        }

        // Generate new key based on contributor type
        let public_key = match contributor_type {
            "zap_user" => {
                // For zap users, use Nostr pubkey directly
                contributor_id.to_string()
            }
            "merge_miner" | "fee_forwarder" => {
                // For miners/fee forwarders, generate new keypair
                // TODO: Use actual key generation
                format!("generated_key_{}", contributor_id)
            }
            _ => {
                return Err(GovernanceError::CryptoError(format!(
                    "Unknown contributor type: {}",
                    contributor_type
                )));
            }
        };

        // Store key for future use
        sqlx::query(
            r#"
            INSERT INTO contributor_keys (contributor_id, contributor_type, public_key, created_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(contributor_id)
        .bind(contributor_type)
        .bind(&public_key)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to store contributor key: {}", e))
        })?;

        Ok(public_key)
    }

    /// Get entity name for contributor
    async fn get_entity_name(
        &self,
        contributor_id: &str,
        contributor_type: &str,
    ) -> Result<String, GovernanceError> {
        match contributor_type {
            "zap_user" => {
                // For zap users, try to get name from Nostr profile
                // TODO: Query Nostr for profile name
                Ok(format!("Zap Contributor {}", &contributor_id[..8]))
            }
            "merge_miner" => Ok(format!("Merge Miner {}", contributor_id)),
            "fee_forwarder" => Ok(format!("Fee Forwarder {}", contributor_id)),
            _ => Ok(format!("Commons Contributor {}", contributor_id)),
        }
    }

    /// Get economic node ID by contributor ID
    async fn get_node_by_contributor_id(
        &self,
        contributor_id: &str,
    ) -> Result<Option<i32>, GovernanceError> {
        // Check if contributor is already registered
        // Store contributor_id in economic_nodes table (need to add column)
        // For now, check by public key if it matches contributor_id (zap users)
        let node: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT id FROM economic_nodes 
            WHERE node_type = 'commons_contributor' 
              AND public_key = ?
            "#,
        )
        .bind(contributor_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to query economic nodes: {}", e))
        })?;

        Ok(node)
    }
}

