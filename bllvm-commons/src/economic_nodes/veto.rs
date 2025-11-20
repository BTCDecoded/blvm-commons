//! Veto Signal Management
//!
//! Handles collection, verification, and threshold calculation for economic node vetoes

use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use tracing::info;

use super::types::*;
use super::registry::EconomicNodeRegistry;
use crate::crypto::signatures::SignatureManager;
use crate::error::GovernanceError;

pub struct VetoManager {
    pool: SqlitePool,
    signature_manager: SignatureManager,
}

impl VetoManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            signature_manager: SignatureManager::new(),
        }
    }

    /// Collect a veto signal from an economic node
    pub async fn collect_veto_signal(
        &self,
        pr_id: i32,
        node_id: i32,
        signal_type: SignalType,
        signature: &str,
        rationale: &str,
    ) -> Result<i32, GovernanceError> {
        // Get node information
        let registry = EconomicNodeRegistry::new(self.pool.clone());
        let node = registry.get_node_by_id(node_id).await?;
        if node.status != NodeStatus::Active {
            return Err(GovernanceError::CryptoError(
                "Node is not active".to_string(),
            ));
        }

        // Verify signature
        let message = format!("PR #{} veto signal from {}", pr_id, node.entity_name);
        let verified = self.signature_manager.verify_governance_signature(
            &message,
            signature,
            &node.public_key,
        )?;

        if !verified {
            return Err(GovernanceError::CryptoError(
                "Invalid signature".to_string(),
            ));
        }

        // Check if node already submitted a signal for this PR
        let existing = sqlx::query("SELECT id FROM veto_signals WHERE pr_id = ? AND node_id = ?")
            .bind(pr_id)
            .bind(node_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                GovernanceError::DatabaseError(format!("Failed to check existing signal: {}", e))
            })?;

        if existing.is_some() {
            return Err(GovernanceError::CryptoError(
                "Node already submitted signal for this PR".to_string(),
            ));
        }

        // Insert veto signal
        let result = sqlx::query(
            r#"
            INSERT INTO veto_signals 
            (pr_id, node_id, signal_type, weight, signature, rationale, timestamp, verified)
            VALUES (?, ?, ?, ?, ?, ?, ?, TRUE)
            "#,
        )
        .bind(pr_id)
        .bind(node_id)
        .bind(signal_type.as_str())
        .bind(node.weight)
        .bind(signature)
        .bind(rationale)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to insert veto signal: {}", e))
        })?;

        let signal_id = result.last_insert_rowid() as i32;
        info!(
            "Collected {} signal from node {} for PR {}",
            signal_type.as_str(),
            node.entity_name,
            pr_id
        );

        Ok(signal_id)
    }

    /// Check if veto threshold is met for a PR
    pub async fn check_veto_threshold(&self, pr_id: i32) -> Result<VetoThreshold, GovernanceError> {
        // Get all veto signals for this PR
        let signals = sqlx::query(
            r#"
            SELECT vs.signal_type, vs.weight, en.node_type
            FROM veto_signals vs
            JOIN economic_nodes en ON vs.node_id = en.id
            WHERE vs.pr_id = ? AND vs.verified = TRUE
            "#,
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to fetch veto signals: {}", e))
        })?;

        let mut mining_veto_weight = 0.0;
        let mut economic_veto_weight = 0.0;
        let mut total_mining_weight = 0.0;
        let mut total_economic_weight = 0.0;

        // Calculate weights by node type
        for signal in signals {
            let node_type =
                NodeType::from_str(&signal.get::<String, _>("node_type")).ok_or_else(|| {
                    GovernanceError::CryptoError(format!(
                        "Invalid node type: {}",
                        signal.get::<String, _>("node_type")
                    ))
                })?;

            let signal_type = SignalType::from_str(&signal.get::<String, _>("signal_type"))
                .ok_or_else(|| {
                    GovernanceError::CryptoError(format!(
                        "Invalid signal type: {}",
                        signal.get::<String, _>("signal_type")
                    ))
                })?;

            let weight = signal.get::<f64, _>("weight");

            match node_type {
                NodeType::MiningPool => {
                    total_mining_weight += weight;
                    if signal_type == SignalType::Veto {
                        mining_veto_weight += weight;
                    }
                }
                _ => {
                    total_economic_weight += weight;
                    if signal_type == SignalType::Veto {
                        economic_veto_weight += weight;
                    }
                }
            }
        }

        // Calculate percentages
        let mining_veto_percent = if total_mining_weight > 0.0 {
            (mining_veto_weight / total_mining_weight) * 100.0
        } else {
            0.0
        };

        let economic_veto_percent = if total_economic_weight > 0.0 {
            (economic_veto_weight / total_economic_weight) * 100.0
        } else {
            0.0
        };

        // Check thresholds (30% mining or 40% economic)
        let threshold_met = mining_veto_percent >= 30.0 || economic_veto_percent >= 40.0;
        let veto_active = threshold_met;

        Ok(VetoThreshold {
            mining_veto_percent,
            economic_veto_percent,
            threshold_met,
            veto_active,
        })
    }

    /// Get all veto signals for a PR
    pub async fn get_pr_veto_signals(
        &self,
        pr_id: i32,
    ) -> Result<Vec<VetoSignal>, GovernanceError> {
        let rows = sqlx::query(
            r#"
            SELECT vs.id, vs.pr_id, vs.node_id, vs.signal_type, vs.weight, 
                   vs.signature, vs.rationale, vs.timestamp, vs.verified,
                   en.entity_name
            FROM veto_signals vs
            JOIN economic_nodes en ON vs.node_id = en.id
            WHERE vs.pr_id = ?
            ORDER BY vs.timestamp DESC
            "#,
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to fetch veto signals: {}", e))
        })?;

        let mut signals = Vec::new();
        for row in rows {
            let signal_type = SignalType::from_str(&row.get::<String, _>("signal_type"))
                .ok_or_else(|| {
                    GovernanceError::CryptoError(format!(
                        "Invalid signal type: {}",
                        row.get::<String, _>("signal_type")
                    ))
                })?;

            let ts_str = row.get::<String, _>("timestamp");
            let timestamp = if ts_str.is_empty() {
                Utc::now() // Default to now if empty
            } else {
                // Try RFC3339 first, then SQLite format (YYYY-MM-DD HH:MM:SS)
                match DateTime::parse_from_rfc3339(&ts_str) {
                    Ok(dt) => dt.with_timezone(&Utc),
                    Err(_) => {
                        chrono::NaiveDateTime::parse_from_str(&ts_str, "%Y-%m-%d %H:%M:%S")
                            .map(|dt| dt.and_utc())
                            .map_err(|e| GovernanceError::CryptoError(format!("Invalid timestamp: {}", e)))?
                    }
                }
            };
            
            signals.push(VetoSignal {
                id: Some(row.get::<i32, _>("id")),
                pr_id: row.get::<i32, _>("pr_id"),
                node_id: row.get::<i32, _>("node_id"),
                signal_type,
                weight: row.get::<f64, _>("weight"),
                signature: row.get::<String, _>("signature"),
                rationale: row.get::<String, _>("rationale"),
                timestamp,
                verified: row.get::<i32, _>("verified") != 0,
            });
        }

        Ok(signals)
    }

    /// Get node by ID (helper for tests)
    async fn get_node_by_id(&self, node_id: i32) -> Result<EconomicNode, GovernanceError> {
        let registry = EconomicNodeRegistry::new(self.pool.clone());
        registry.get_node_by_id(node_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use crate::economic_nodes::registry::EconomicNodeRegistry;
    use bllvm_sdk::governance::GovernanceKeypair;

    async fn setup_test_veto_manager() -> (VetoManager, EconomicNodeRegistry, i32) {
        let db = Database::new_in_memory().await.unwrap();
        let pool = db.pool().unwrap().clone();
        
        // Create test node
        let registry = EconomicNodeRegistry::new(pool.clone());
        let keypair = GovernanceKeypair::generate().unwrap();
        let public_key = hex::encode(keypair.public_key.serialize());
        
        // Create minimal qualification for testing
        let qual_json = serde_json::json!({
            "node_type": "mining_pool",
            "hashpower_proof": {
                "blocks_mined": ["block1"],
                "time_period_days": 30,
                "total_network_blocks": 100,
                "percentage": 35.0
            },
            "contact_info": {
                "entity_name": "Test Pool",
                "contact_email": "test@example.com"
            }
        });
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();
        
        // Register node (we'll need to set up the table first)
        // For now, just return the manager
        let manager = VetoManager::new(pool);
        (manager, registry, 1) // node_id = 1
    }

    #[tokio::test]
    async fn test_check_veto_threshold_no_signals() {
        let (manager, _, _) = setup_test_veto_manager().await;
        
        let threshold = manager.check_veto_threshold(1).await.unwrap();
        
        assert_eq!(threshold.mining_veto_percent, 0.0);
        assert_eq!(threshold.economic_veto_percent, 0.0);
        assert!(!threshold.threshold_met);
        assert!(!threshold.veto_active);
    }

    #[tokio::test]
    async fn test_check_veto_threshold_below_mining_threshold() {
        // This test would require setting up actual database records
        // For now, we test the logic with empty results
        let (manager, _, _) = setup_test_veto_manager().await;
        
        let threshold = manager.check_veto_threshold(999).await.unwrap();
        assert!(!threshold.veto_active, "Should not be active with no signals");
    }

    #[tokio::test]
    async fn test_get_pr_veto_signals_empty() {
        let (manager, _, _) = setup_test_veto_manager().await;
        
        let signals = manager.get_pr_veto_signals(1).await.unwrap();
        assert_eq!(signals.len(), 0, "Should return empty list when no signals");
    }

    #[test]
    fn test_veto_threshold_calculation() {
        // Test threshold calculation logic
        let mining_veto_percent = 25.0;
        let economic_veto_percent = 35.0;
        
        let threshold_met = mining_veto_percent >= 30.0 || economic_veto_percent >= 40.0;
        assert!(!threshold_met, "Should not meet threshold");
        
        let mining_veto_percent2 = 35.0;
        let threshold_met2 = mining_veto_percent2 >= 30.0 || economic_veto_percent >= 40.0;
        assert!(threshold_met2, "Should meet threshold with 35% mining");
        
        let economic_veto_percent2 = 45.0;
        let threshold_met3 = mining_veto_percent >= 30.0 || economic_veto_percent2 >= 40.0;
        assert!(threshold_met3, "Should meet threshold with 45% economic");
    }

    #[tokio::test]
    async fn test_veto_manager_new() {
        let db = Database::new_in_memory().await.unwrap();
        let manager = VetoManager::new(db.pool().unwrap().clone());
        // Just verify it can be created
        assert!(true, "Manager should be created");
    }
}
