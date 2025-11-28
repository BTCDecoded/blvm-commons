//! Economic Node Registry
//!
//! Handles registration, qualification verification, and weight calculation

use sqlx::{Row, SqlitePool};
use tracing::{info, warn};

use super::types::*;
use crate::error::GovernanceError;
use crate::config::loader::CommonsContributorThresholdsConfig;

pub struct EconomicNodeRegistry {
    pool: SqlitePool,
    /// Maintainer-configurable thresholds for Commons contributors
    /// Loaded from governance/config/commons-contributor-thresholds.yml
    commons_contributor_thresholds: Option<CommonsContributorThresholdsConfig>,
}

impl EconomicNodeRegistry {
    pub fn new(pool: SqlitePool) -> Self {
        Self { 
            pool,
            commons_contributor_thresholds: None,
        }
    }

    /// Create registry with Commons contributor thresholds loaded from config
    pub fn with_thresholds(
        pool: SqlitePool,
        thresholds: Option<CommonsContributorThresholdsConfig>,
    ) -> Self {
        Self {
            pool,
            commons_contributor_thresholds: thresholds,
        }
    }

    /// Set Commons contributor thresholds (for runtime configuration updates)
    pub fn set_commons_contributor_thresholds(
        &mut self,
        thresholds: Option<CommonsContributorThresholdsConfig>,
    ) {
        self.commons_contributor_thresholds = thresholds;
    }

    /// Register a new economic node with qualification proof
    pub async fn register_economic_node(
        &self,
        node_type: NodeType,
        entity_name: &str,
        public_key: &str,
        qualification_data: &QualificationProof,
        created_by: Option<&str>,
    ) -> Result<i32, GovernanceError> {
        // Check for duplicate public key
        let existing = sqlx::query("SELECT id FROM economic_nodes WHERE public_key = ?")
            .bind(public_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                GovernanceError::DatabaseError(format!("Failed to check for duplicate: {}", e))
            })?;

        if existing.is_some() {
            return Err(GovernanceError::CryptoError(format!(
                "Node with public key {} already registered",
                public_key
            )));
        }

        // Verify qualification meets thresholds
        let verified = self
            .verify_qualification(node_type.clone(), qualification_data)
            .await?;
        if !verified {
            return Err(GovernanceError::CryptoError(
                "Node does not meet qualification thresholds".to_string(),
            ));
        }

        // Calculate initial weight
        let weight = self
            .calculate_weight(node_type.clone(), qualification_data)
            .await?;

        // Insert into database
        let result = sqlx::query(
            r#"
            INSERT INTO economic_nodes 
            (node_type, entity_name, public_key, qualification_data, weight, status, created_by)
            VALUES (?, ?, ?, ?, ?, 'pending', ?)
            "#,
        )
        .bind(node_type.as_str())
        .bind(entity_name)
        .bind(public_key)
        .bind(serde_json::to_string(qualification_data)?)
        .bind(weight)
        .bind(created_by)
        .execute(&self.pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to register node: {}", e)))?;

        let node_id = result.last_insert_rowid() as i32;
        info!("Registered economic node {} (ID: {})", entity_name, node_id);
        Ok(node_id)
    }

    /// Verify that a node meets qualification thresholds
    pub async fn verify_qualification(
        &self,
        node_type: NodeType,
        qualification_data: &QualificationProof,
    ) -> Result<bool, GovernanceError> {
        let thresholds = node_type.qualification_thresholds();

        // Check hashpower threshold (mining pools)
        if let Some(min_hashpower) = thresholds.minimum_hashpower_percent {
            if let Some(hashpower_proof) = &qualification_data.hashpower_proof {
                if hashpower_proof.percentage < min_hashpower {
                    warn!(
                        "Hashpower {}% below threshold {}%",
                        hashpower_proof.percentage, min_hashpower
                    );
                    return Ok(false);
                }
            } else {
                warn!("Hashpower proof required for mining pools");
                return Ok(false);
            }
        }

        // Check holdings threshold
        if let Some(min_holdings) = thresholds.minimum_holdings_btc {
            if let Some(holdings_proof) = &qualification_data.holdings_proof {
                if holdings_proof.total_btc < min_holdings as f64 {
                    warn!(
                        "Holdings {} BTC below threshold {} BTC",
                        holdings_proof.total_btc, min_holdings
                    );
                    return Ok(false);
                }
            } else {
                warn!("Holdings proof required for this node type");
                return Ok(false);
            }
        }

        // Check volume threshold
        if let Some(min_volume) = thresholds.minimum_volume_usd {
            if let Some(volume_proof) = &qualification_data.volume_proof {
                let volume = if node_type == NodeType::Exchange {
                    volume_proof.daily_volume_usd
                } else {
                    volume_proof.monthly_volume_usd
                };

                if volume < min_volume as f64 {
                    warn!("Volume ${} below threshold ${}", volume, min_volume);
                    return Ok(false);
                }
            } else {
                warn!("Volume proof required for this node type");
                return Ok(false);
            }
        }

        // Check Commons contributor thresholds (maintainer-configurable)
        if node_type == NodeType::CommonsContributor {
            return self.verify_commons_contributor_qualification(qualification_data).await;
        }

        Ok(true)
    }

    /// Verify Commons contributor qualification using maintainer-configurable thresholds
    async fn verify_commons_contributor_qualification(
        &self,
        qualification_data: &QualificationProof,
    ) -> Result<bool, GovernanceError> {
        let thresholds_config = match &self.commons_contributor_thresholds {
            Some(config) => &config.commons_contributor_thresholds,
            None => {
                warn!("Commons contributor thresholds not configured, using defaults");
                // Fall back to hardcoded defaults if config not loaded
                return self.verify_commons_contributor_defaults(qualification_data).await;
            }
        };

        let proof = match &qualification_data.commons_contributor_proof {
            Some(p) => p,
            None => {
                warn!("Commons contributor proof required");
                return Ok(false);
            }
        };

        let mut qualifications_met = Vec::new();

        // Check merge mining threshold
        if thresholds_config.merge_mining.enabled {
            if let Some(merge_proof) = &proof.merge_mining_proof {
                if merge_proof.total_revenue_btc >= thresholds_config.merge_mining.minimum_contribution_btc
                    && merge_proof.period_days >= thresholds_config.measurement_period_days
                {
                    qualifications_met.push("merge_mining");
                }
            }
        }

        // Check fee forwarding threshold
        if thresholds_config.fee_forwarding.enabled {
            if let Some(fee_proof) = &proof.fee_forwarding_proof {
                if fee_proof.total_fees_forwarded_btc >= thresholds_config.fee_forwarding.minimum_contribution_btc
                    && fee_proof.period_days >= thresholds_config.measurement_period_days
                {
                    qualifications_met.push("fee_forwarding");
                }
            }
        }

        // Check zap threshold
        if thresholds_config.zaps.enabled {
            if let Some(zap_proof) = &proof.zap_proof {
                if zap_proof.total_zaps_btc >= thresholds_config.zaps.minimum_contribution_btc
                    && zap_proof.period_days >= thresholds_config.measurement_period_days
                {
                    qualifications_met.push("zaps");
                }
            }
        }

        // Check marketplace sales threshold (BTC via BIP70 payments)
        if thresholds_config.marketplace.enabled {
            if let Some(marketplace_proof) = &proof.marketplace_sales_proof {
                let min_btc = thresholds_config.marketplace.minimum_sales_btc
                    .unwrap_or(0.0);
                if marketplace_proof.total_sales_btc >= min_btc
                    && marketplace_proof.period_days >= thresholds_config.measurement_period_days
                {
                    qualifications_met.push("marketplace");
                }
            }
        }

        // Apply qualification logic (OR or AND)
        let qualified = match thresholds_config.qualification_logic.as_str() {
            "OR" => !qualifications_met.is_empty(),
            "AND" => {
                // Count enabled thresholds
                let enabled_count = [
                    thresholds_config.merge_mining.enabled,
                    thresholds_config.fee_forwarding.enabled,
                    thresholds_config.zaps.enabled,
                    thresholds_config.marketplace.enabled,
                ]
                .iter()
                .filter(|&&enabled| enabled)
                .count();
                
                qualifications_met.len() == enabled_count
            }
            _ => {
                warn!("Invalid qualification_logic: {}, defaulting to OR", thresholds_config.qualification_logic);
                !qualifications_met.is_empty()
            }
        };

        if !qualified {
            warn!(
                "Commons contributor qualification not met. Met: {:?}, Logic: {}",
                qualifications_met, thresholds_config.qualification_logic
            );
        }

        Ok(qualified)
    }

    /// Fallback verification using hardcoded defaults if config not loaded
    async fn verify_commons_contributor_defaults(
        &self,
        qualification_data: &QualificationProof,
    ) -> Result<bool, GovernanceError> {
        let proof = match &qualification_data.commons_contributor_proof {
            Some(p) => p,
            None => return Ok(false),
        };

        // Default thresholds (from documentation)
        let mut qualifications_met = Vec::new();

        if let Some(merge_proof) = &proof.merge_mining_proof {
            if merge_proof.total_revenue_btc >= 0.01 && merge_proof.period_days >= 90 {
                qualifications_met.push("merge_mining");
            }
        }

        if let Some(fee_proof) = &proof.fee_forwarding_proof {
            if fee_proof.total_fees_forwarded_btc >= 0.1 && fee_proof.period_days >= 90 {
                qualifications_met.push("fee_forwarding");
            }
        }

        if let Some(zap_proof) = &proof.zap_proof {
            if zap_proof.total_zaps_btc >= 0.01 && zap_proof.period_days >= 90 {
                qualifications_met.push("zaps");
            }
        }

        // Default to OR logic
        Ok(!qualifications_met.is_empty())
    }

    /// Calculate weight for an economic node
    pub async fn calculate_weight(
        &self,
        node_type: NodeType,
        qualification_data: &QualificationProof,
    ) -> Result<f64, GovernanceError> {
        match node_type {
            NodeType::MiningPool => {
                // Weight = hashpower percentage
                if let Some(hashpower_proof) = &qualification_data.hashpower_proof {
                    Ok(hashpower_proof.percentage / 100.0)
                } else {
                    Err(GovernanceError::CryptoError(
                        "Hashpower proof required for mining pools".to_string(),
                    ))
                }
            }
            NodeType::Exchange => {
                // Weight = 70% holdings + 30% volume (trust-discounted)
                let holdings_weight =
                    if let Some(holdings_proof) = &qualification_data.holdings_proof {
                        // Normalize to 0-1 scale (10K BTC = 1.0)
                        (holdings_proof.total_btc / 10_000.0).min(1.0) * 0.7
                    } else {
                        0.0
                    };

                let volume_weight = if let Some(volume_proof) = &qualification_data.volume_proof {
                    // Normalize to 0-1 scale ($100M daily = 1.0)
                    (volume_proof.daily_volume_usd / 100_000_000.0).min(1.0) * 0.3
                } else {
                    0.0
                };

                Ok(holdings_weight + volume_weight)
            }
            NodeType::Custodian => {
                // Weight = holdings percentage
                if let Some(holdings_proof) = &qualification_data.holdings_proof {
                    // Normalize to 0-1 scale (10K BTC = 1.0)
                    Ok((holdings_proof.total_btc / 10_000.0).min(1.0))
                } else {
                    Err(GovernanceError::CryptoError(
                        "Holdings proof required for custodians".to_string(),
                    ))
                }
            }
            NodeType::PaymentProcessor => {
                // Weight = transaction volume
                if let Some(volume_proof) = &qualification_data.volume_proof {
                    // Normalize to 0-1 scale ($50M monthly = 1.0)
                    Ok((volume_proof.monthly_volume_usd / 50_000_000.0).min(1.0))
                } else {
                    Err(GovernanceError::CryptoError(
                        "Volume proof required for payment processors".to_string(),
                    ))
                }
            }
            NodeType::MajorHolder => {
                // Weight = holdings percentage
                if let Some(holdings_proof) = &qualification_data.holdings_proof {
                    // Normalize to 0-1 scale (5K BTC = 1.0)
                    Ok((holdings_proof.total_btc / 5_000.0).min(1.0))
                } else {
                    Err(GovernanceError::CryptoError(
                        "Holdings proof required for major holders".to_string(),
                    ))
                }
            }
            NodeType::CommonsContributor => {
                // Weight = sqrt(total_contribution_btc / normalization_factor)
                // Uses quadratic weighting for fairness
                self.calculate_commons_contributor_weight(qualification_data).await
            }
        }
    }

    /// Calculate weight for Commons contributor using quadratic formula
    async fn calculate_commons_contributor_weight(
        &self,
        qualification_data: &QualificationProof,
    ) -> Result<f64, GovernanceError> {
        let proof = match &qualification_data.commons_contributor_proof {
            Some(p) => p,
            None => {
                return Err(GovernanceError::CryptoError(
                    "Commons contributor proof required".to_string(),
                ));
            }
        };

        // Get normalization factor from config, or use default
        let normalization_factor = self
            .commons_contributor_thresholds
            .as_ref()
            .and_then(|c| Some(c.weight_calculation.normalization_factor))
            .unwrap_or(1.0);

        let mut total_contribution_btc = 0.0;

        // Sum all contribution types
        if let Some(merge_proof) = &proof.merge_mining_proof {
            total_contribution_btc += merge_proof.total_revenue_btc;
        }

        if let Some(fee_proof) = &proof.fee_forwarding_proof {
            total_contribution_btc += fee_proof.total_fees_forwarded_btc;
        }

        if let Some(zap_proof) = &proof.zap_proof {
            total_contribution_btc += zap_proof.total_zaps_btc;
        }

        // Convert USD revenue to BTC using moving average price (smooths volatility)
        // This prevents contributors from being penalized by sudden price movements
        let btc_price_ma = match &self.btc_price_service {
            Some(service) => service.get_moving_average(),
            None => {
                warn!("BTC price service not available, using default price");
                50000.0 // Fallback default
            }
        };

        // Marketplace sales are already in BTC (BIP70 payments)
        if let Some(marketplace_proof) = &proof.marketplace_sales_proof {
            total_contribution_btc += marketplace_proof.total_sales_btc;
        }

        if let Some(treasury_proof) = &proof.treasury_sales_proof {
            // Convert USD to BTC using moving average (prevents volatility penalty)
            let treasury_btc = treasury_proof.total_sales_usd / btc_price_ma;
            total_contribution_btc += treasury_btc;
            info!(
                "Converted treasury sales: ${:.2} USD -> {:.8} BTC (using ${:.2} MA price)",
                treasury_proof.total_sales_usd, treasury_btc, btc_price_ma
            );
        }

        if let Some(service_proof) = &proof.service_sales_proof {
            // Convert USD to BTC using moving average (prevents volatility penalty)
            let service_btc = service_proof.total_sales_usd / btc_price_ma;
            total_contribution_btc += service_btc;
            info!(
                "Converted service sales: ${:.2} USD -> {:.8} BTC (using ${:.2} MA price)",
                service_proof.total_sales_usd, service_btc, btc_price_ma
            );
        }

        // Apply quadratic formula: sqrt(total_contribution_btc / normalization_factor)
        let weight = (total_contribution_btc / normalization_factor).sqrt();

        // Apply minimum weight if configured
        let min_weight = self
            .commons_contributor_thresholds
            .as_ref()
            .and_then(|c| Some(c.weight_calculation.minimum_weight))
            .unwrap_or(0.01);

        Ok(weight.max(min_weight))
    }

    /// Get all active economic nodes
    pub async fn get_active_nodes(&self) -> Result<Vec<EconomicNode>, GovernanceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, node_type, entity_name, public_key, qualification_data, 
                   weight, status, registered_at, verified_at, last_verified_at, 
                   created_by, notes
            FROM economic_nodes 
            WHERE status = 'active'
            ORDER BY weight DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to fetch nodes: {}", e)))?;

        let mut nodes = Vec::new();
        for row in rows {
            let node_type =
                NodeType::from_str(&row.get::<String, _>("node_type")).ok_or_else(|| {
                    GovernanceError::CryptoError(format!(
                        "Invalid node type: {}",
                        row.get::<String, _>("node_type")
                    ))
                })?;

            let status =
                NodeStatus::from_str(&row.get::<String, _>("status")).ok_or_else(|| {
                    GovernanceError::CryptoError(format!(
                        "Invalid status: {}",
                        row.get::<String, _>("status")
                    ))
                })?;

            let qualification_data: String = row.get("qualification_data");
            let qualification_data: QualificationProof = serde_json::from_str(&qualification_data)
                .map_err(|e| {
                    GovernanceError::CryptoError(format!("Invalid qualification data: {}", e))
                })?;

            nodes.push(EconomicNode {
                id: row.get("id"),
                node_type,
                entity_name: row.get("entity_name"),
                public_key: row.get("public_key"),
                qualification_data: serde_json::to_value(&qualification_data)
                    .unwrap_or_else(|_| serde_json::json!({})),
                weight: row.get("weight"),
                status,
                registered_at: row.get("registered_at"),
                verified_at: row.get("verified_at"),
                last_verified_at: row.get("last_verified_at"),
                created_by: row.get("created_by"),
                notes: row.get("notes"),
            });
        }

        Ok(nodes)
    }

    /// Get node by ID
    pub async fn get_node_by_id(&self, node_id: i32) -> Result<EconomicNode, GovernanceError> {
        let row = sqlx::query(
            r#"
            SELECT id, node_type, entity_name, public_key, qualification_data, 
                   weight, status, registered_at, verified_at, last_verified_at, 
                   created_by, notes
            FROM economic_nodes 
            WHERE id = ?
            "#,
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to fetch node: {}", e)))?;

        let row = row.ok_or_else(|| {
            GovernanceError::CryptoError(format!("Node with ID {} not found", node_id))
        })?;

        let node_type =
            NodeType::from_str(&row.get::<String, _>("node_type")).ok_or_else(|| {
                GovernanceError::CryptoError(format!(
                    "Invalid node type: {}",
                    row.get::<String, _>("node_type")
                ))
            })?;

        let status = NodeStatus::from_str(&row.get::<String, _>("status")).ok_or_else(|| {
            GovernanceError::CryptoError(format!(
                "Invalid status: {}",
                row.get::<String, _>("status")
            ))
        })?;

        let qualification_data: String = row.get("qualification_data");
        let qualification_data: QualificationProof = serde_json::from_str(&qualification_data)
            .map_err(|e| {
                GovernanceError::CryptoError(format!("Invalid qualification data: {}", e))
            })?;

        Ok(EconomicNode {
            id: row.get("id"),
            node_type,
            entity_name: row.get("entity_name"),
            public_key: row.get("public_key"),
            qualification_data: serde_json::to_value(&qualification_data)
                .unwrap_or_else(|_| serde_json::json!({})),
            weight: row.get("weight"),
            status,
            registered_at: row.get("registered_at"),
            verified_at: row.get("verified_at"),
            last_verified_at: row.get("last_verified_at"),
            created_by: row.get("created_by"),
            notes: row.get("notes"),
        })
    }

    /// Update node status
    pub async fn update_node_status(
        &self,
        node_id: i32,
        status: NodeStatus,
    ) -> Result<(), GovernanceError> {
        sqlx::query("UPDATE economic_nodes SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(node_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                GovernanceError::DatabaseError(format!("Failed to update node status: {}", e))
            })?;

        info!("Updated node {} status to {}", node_id, status.as_str());
        Ok(())
    }

    /// Recalculate weights for all nodes based on current qualification data
    pub async fn recalculate_all_weights(&self) -> Result<(), GovernanceError> {
        let nodes = sqlx::query("SELECT id, node_type, qualification_data FROM economic_nodes")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| GovernanceError::DatabaseError(format!("Failed to fetch nodes: {}", e)))?;

        for row in nodes {
            let node_id: i32 = row.get("id");
            let node_type_str: String = row.get("node_type");
            let node_type = NodeType::from_str(&node_type_str).ok_or_else(|| {
                GovernanceError::CryptoError(format!("Invalid node type: {}", node_type_str))
            })?;

            let qualification_data_str: String = row.get("qualification_data");
            let qualification_data: QualificationProof =
                serde_json::from_str(&qualification_data_str).map_err(|e| {
                    GovernanceError::CryptoError(format!("Invalid qualification data: {}", e))
                })?;

            let new_weight = self
                .calculate_weight(node_type, &qualification_data)
                .await?;

            sqlx::query("UPDATE economic_nodes SET weight = ? WHERE id = ?")
                .bind(new_weight)
                .bind(node_id)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    GovernanceError::DatabaseError(format!("Failed to update weight: {}", e))
                })?;
        }

        info!("Recalculated weights for all nodes");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use crate::economic_nodes::types::{
        ContactInfo, HoldingsProof, QualificationProof, VolumeProof,
    };

    async fn setup_test_registry() -> EconomicNodeRegistry {
        let db = Database::new_in_memory().await.unwrap();
        EconomicNodeRegistry::new(db.pool().unwrap().clone())
    }

    fn create_mining_pool_qualification(hashpower: f64) -> serde_json::Value {
        serde_json::json!({
            "node_type": "mining_pool",
            "hashpower_proof": {
                "blocks_mined": ["block1", "block2"],
                "time_period_days": 30,
                "total_network_blocks": 100,
                "percentage": hashpower
            },
            "contact_info": {
                "entity_name": "Test Pool",
                "contact_email": "test@example.com"
            }
        })
    }

    fn create_exchange_qualification(
        holdings_btc: f64,
        daily_volume_usd: f64,
    ) -> serde_json::Value {
        serde_json::json!({
            "node_type": "exchange",
            "holdings_proof": {
                "addresses": ["addr1"],
                "total_btc": holdings_btc,
                "signature_challenge": "challenge"
            },
            "volume_proof": {
                "daily_volume_usd": daily_volume_usd,
                "monthly_volume_usd": daily_volume_usd * 30.0,
                "data_source": "test"
            },
            "contact_info": {
                "entity_name": "Test Exchange",
                "contact_email": "test@example.com"
            }
        })
    }

    #[tokio::test]
    async fn test_calculate_weight_mining_pool() {
        let registry = setup_test_registry().await;
        let qual_json = create_mining_pool_qualification(35.0);
        let qual: QualificationProof = serde_json::from_value(qual_json.clone()).unwrap();

        let weight = registry
            .calculate_weight(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        assert_eq!(weight, 0.35, "Mining pool weight should be hashpower / 100");
    }

    #[tokio::test]
    async fn test_calculate_weight_mining_pool_no_proof() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::MiningPool,
            hashpower_proof: None,
            holdings_proof: None,
            volume_proof: None,
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let result = registry.calculate_weight(NodeType::MiningPool, &qual).await;
        assert!(result.is_err(), "Should fail without hashpower proof");
    }

    #[tokio::test]
    async fn test_calculate_weight_exchange() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(5000.0, 50_000_000.0);
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let weight = registry
            .calculate_weight(NodeType::Exchange, &qual)
            .await
            .unwrap();
        // holdings: 5000/10000 * 0.7 = 0.35
        // volume: 50M/100M * 0.3 = 0.15
        // total: 0.5
        assert!((weight - 0.5).abs() < 0.01, "Exchange weight should be 0.5");
    }

    #[tokio::test]
    async fn test_calculate_weight_exchange_capped() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(20000.0, 200_000_000.0);
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let weight = registry
            .calculate_weight(NodeType::Exchange, &qual)
            .await
            .unwrap();
        // Should be capped at 1.0
        assert!(weight <= 1.0, "Weight should be capped at 1.0");
    }

    #[tokio::test]
    async fn test_calculate_weight_custodian() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::Custodian,
            hashpower_proof: None,
            holdings_proof: Some(HoldingsProof {
                addresses: vec!["addr1".to_string()],
                total_btc: 5000.0,
                signature_challenge: "challenge".to_string(),
            }),
            volume_proof: None,
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let weight = registry
            .calculate_weight(NodeType::Custodian, &qual)
            .await
            .unwrap();
        // 5000/10000 = 0.5
        assert!(
            (weight - 0.5).abs() < 0.01,
            "Custodian weight should be 0.5"
        );
    }

    #[tokio::test]
    async fn test_calculate_weight_payment_processor() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::PaymentProcessor,
            hashpower_proof: None,
            holdings_proof: None,
            volume_proof: Some(VolumeProof {
                daily_volume_usd: 0.0,
                monthly_volume_usd: 25_000_000.0,
                data_source: "test".to_string(),
                verification_url: None,
            }),
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let weight = registry
            .calculate_weight(NodeType::PaymentProcessor, &qual)
            .await
            .unwrap();
        // 25M/50M = 0.5
        assert!(
            (weight - 0.5).abs() < 0.01,
            "Payment processor weight should be 0.5"
        );
    }

    #[tokio::test]
    async fn test_calculate_weight_major_holder() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::MajorHolder,
            hashpower_proof: None,
            holdings_proof: Some(HoldingsProof {
                addresses: vec!["addr1".to_string()],
                total_btc: 2500.0,
                signature_challenge: "challenge".to_string(),
            }),
            volume_proof: None,
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let weight = registry
            .calculate_weight(NodeType::MajorHolder, &qual)
            .await
            .unwrap();
        // 2500/5000 = 0.5
        assert!(
            (weight - 0.5).abs() < 0.01,
            "Major holder weight should be 0.5"
        );
    }

    #[tokio::test]
    async fn test_verify_qualification_mining_pool_meets_threshold() {
        let registry = setup_test_registry().await;
        let qual_json = create_mining_pool_qualification(35.0); // Above 1% threshold (from types.rs)
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        assert!(verified, "Should verify when hashpower meets threshold");
    }

    #[tokio::test]
    async fn test_verify_qualification_mining_pool_below_threshold() {
        let registry = setup_test_registry().await;
        let qual_json = create_mining_pool_qualification(0.5); // Below 1% threshold
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        assert!(
            !verified,
            "Should not verify when hashpower below threshold"
        );
    }

    #[tokio::test]
    async fn test_verify_qualification_mining_pool_no_proof() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::MiningPool,
            hashpower_proof: None,
            holdings_proof: None,
            volume_proof: None,
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let verified = registry
            .verify_qualification(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        assert!(!verified, "Should not verify without hashpower proof");
    }

    #[tokio::test]
    async fn test_verify_qualification_exchange_meets_threshold() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(10000.0, 100_000_000.0); // Meets thresholds
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::Exchange, &qual)
            .await
            .unwrap();
        assert!(verified, "Should verify when exchange meets thresholds");
    }

    #[tokio::test]
    async fn test_verify_qualification_exchange_below_holdings() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(9000.0, 100_000_000.0); // Below 10K BTC threshold
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::Exchange, &qual)
            .await
            .unwrap();
        assert!(!verified, "Should not verify when holdings below threshold");
    }

    #[tokio::test]
    async fn test_verify_qualification_exchange_below_volume() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(10000.0, 90_000_000.0); // Below $100M daily threshold
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::Exchange, &qual)
            .await
            .unwrap();
        assert!(!verified, "Should not verify when volume below threshold");
    }

    #[tokio::test]
    async fn test_get_node_by_id_not_found() {
        let registry = setup_test_registry().await;

        let result = registry.get_node_by_id(999).await;
        assert!(result.is_err(), "Should fail for non-existent node");
    }

    #[tokio::test]
    async fn test_get_active_nodes_empty() {
        let registry = setup_test_registry().await;

        let nodes = registry.get_active_nodes().await.unwrap();
        assert_eq!(
            nodes.len(),
            0,
            "Should return empty list when no active nodes"
        );
    }
}
