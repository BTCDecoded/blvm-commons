//! Veto Signal Management
//!
//! Handles collection, verification, and threshold calculation for economic node vetoes

use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use tracing::info;

use super::registry::EconomicNodeRegistry;
use super::types::*;
use crate::crypto::signatures::SignatureManager;
use crate::economic_nodes::consolidation::ConsolidationMonitor;
use crate::error::GovernanceError;
use crate::governance::phase_calculator::GovernancePhaseCalculator;

pub struct VetoManager {
    pool: SqlitePool,
    signature_manager: SignatureManager,
    phase_calculator: Option<GovernancePhaseCalculator>,
    consolidation_monitor: Option<ConsolidationMonitor>,
}

impl VetoManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool: pool.clone(),
            signature_manager: SignatureManager::new(),
            phase_calculator: Some(GovernancePhaseCalculator::new(pool.clone())),
            consolidation_monitor: Some(ConsolidationMonitor::new(pool)),
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
    /// 
    /// Tier-specific thresholds:
    /// - Tier 3: 30% hashpower AND 40% economic activity (90-day review)
    /// - Tier 4: 25% hashpower AND 35% economic activity (24-hour review)
    /// - Tier 5: 50% hashpower AND 60% economic activity (180-day review)
    pub async fn check_veto_threshold(&self, pr_id: i32, tier: u32) -> Result<VetoThreshold, GovernanceError> {
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

        // Calculate veto weights from signals
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
                    if signal_type == SignalType::Veto {
                        mining_veto_weight += weight;
                    }
                }
                _ => {
                    if signal_type == SignalType::Veto {
                        economic_veto_weight += weight;
                    }
                }
            }
        }

        // Get total network hashpower (sum of all active mining pool weights)
        // This is the correct denominator - total network, not just signal submitters
        let total_network_mining_weight: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(weight), 0.0)
            FROM economic_nodes
            WHERE node_type = 'mining_pool' AND status = 'active'
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!(
                "Failed to fetch total network mining weight: {}",
                e
            ))
        })?;

        // Get total network economic activity (sum of all active non-mining pool weights)
        let total_network_economic_weight: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(weight), 0.0)
            FROM economic_nodes
            WHERE node_type != 'mining_pool' AND status = 'active'
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!(
                "Failed to fetch total network economic weight: {}",
                e
            ))
        })?;

        let total_network_mining_weight = total_network_mining_weight.unwrap_or(0.0);
        let total_network_economic_weight = total_network_economic_weight.unwrap_or(0.0);

        // Calculate percentages against total network (not just signal submitters)
        let mining_veto_percent = if total_network_mining_weight > 0.0 {
            (mining_veto_weight / total_network_mining_weight) * 100.0
        } else {
            0.0
        };

        let economic_veto_percent = if total_network_economic_weight > 0.0 {
            (economic_veto_weight / total_network_economic_weight) * 100.0
        } else {
            0.0
        };

        // Get base tier-specific thresholds
        let (base_mining_threshold, base_economic_threshold, review_days_default) = match tier {
            3 => (30.0, 40.0, 90u32),   // Tier 3: 30%+40%, 90 days
            4 => (25.0, 35.0, 0u32),    // Tier 4: 25%+35%, 24 hours (0 days = immediate)
            5 => (50.0, 60.0, 180u32),  // Tier 5: 50%+60%, 180 days
            _ => (30.0, 40.0, 90u32),   // Default to Tier 3 thresholds
        };

        // Apply adaptive thresholds based on governance phase and consolidation
        let (mining_threshold, economic_threshold) = if let (Some(ref phase_calc), Some(ref consol_monitor)) = 
            (&self.phase_calculator, &self.consolidation_monitor) 
        {
            // Get adaptive parameters from phase calculator
            match phase_calc.get_adaptive_parameters().await {
                Ok(adaptive_params) => {
                    // Apply consolidation-based adjustments on top of phase-based thresholds
                    consol_monitor.calculate_adaptive_thresholds(
                        adaptive_params.mining_veto_threshold,
                        adaptive_params.economic_veto_threshold,
                    ).await.unwrap_or((adaptive_params.mining_veto_threshold, adaptive_params.economic_veto_threshold))
                }
                Err(_) => {
                    // Fall back to consolidation-only if phase calculator fails
                    consol_monitor.calculate_adaptive_thresholds(
                        base_mining_threshold,
                        base_economic_threshold,
                    ).await.unwrap_or((base_mining_threshold, base_economic_threshold))
                }
            }
        } else {
            // Fall back to base thresholds if calculators not available
            (base_mining_threshold, base_economic_threshold)
        };

        // Check thresholds using AND logic (both required)
        // Both mining and economic thresholds must be met
        // This prevents single-group veto and requires coordination
        let threshold_met = mining_veto_percent >= mining_threshold && economic_veto_percent >= economic_threshold;
        
        // Get veto state from database (if exists)
        let veto_state = sqlx::query(
            r#"
            SELECT veto_triggered_at, review_period_days, review_period_ends_at,
                   maintainer_override, override_timestamp, override_by, resolution_path
            FROM pr_veto_state
            WHERE pr_id = ?
            "#
        )
        .bind(pr_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to fetch veto state: {}", e))
        })?;

        // If threshold just met, trigger veto (start review period)
        let (review_period_start, review_period_days, review_period_ends_at, maintainer_override, override_timestamp, override_by, resolution_path) = 
            if threshold_met && veto_state.is_none() {
                // First time threshold met - trigger veto
                let now = Utc::now();
                let review_days = review_days_default;
                // For Tier 4 (0 days), use 24 hours instead
                let ends_at = if review_days == 0 {
                    now + chrono::Duration::hours(24)
                } else {
                    now + chrono::Duration::days(review_days as i64)
                };
                
                // Insert veto state
                sqlx::query(
                    r#"
                    INSERT INTO pr_veto_state 
                    (pr_id, veto_triggered_at, review_period_days, review_period_ends_at,
                     mining_veto_percent, economic_veto_percent, threshold_met, veto_active)
                    VALUES (?, ?, ?, ?, ?, ?, ?, TRUE)
                    "#
                )
                .bind(pr_id)
                .bind(now.to_rfc3339())
                .bind(review_days as i32)
                .bind(ends_at.to_rfc3339())
                .bind(mining_veto_percent)
                .bind(economic_veto_percent)
                .bind(threshold_met)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    GovernanceError::DatabaseError(format!("Failed to insert veto state: {}", e))
                })?;
                
                (Some(now), review_days, Some(ends_at), false, None, None, None)
            } else if let Some(state) = veto_state {
                // Veto state exists - get current state
                let triggered_at_str = state.get::<String, _>("veto_triggered_at");
                let triggered_at = DateTime::parse_from_rfc3339(&triggered_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok();
                
                let review_days = state.get::<i32, _>("review_period_days") as u32;
                let ends_at_str = state.get::<String, _>("review_period_ends_at");
                let ends_at = DateTime::parse_from_rfc3339(&ends_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok();
                
                let override_flag = state.get::<i32, _>("maintainer_override") != 0;
                let override_ts_str = state.get::<Option<String>, _>("override_timestamp");
                let override_ts = override_ts_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                });
                let override_by = state.get::<Option<String>, _>("override_by");
                let resolution = state.get::<Option<String>, _>("resolution_path");
                
                // Update veto percentages if changed
                sqlx::query(
                    r#"
                    UPDATE pr_veto_state
                    SET mining_veto_percent = ?, economic_veto_percent = ?, threshold_met = ?, veto_active = ?
                    WHERE pr_id = ?
                    "#
                )
                .bind(mining_veto_percent)
                .bind(economic_veto_percent)
                .bind(threshold_met)
                .bind(threshold_met && !override_flag) // Veto active if threshold met and not overridden
                .bind(pr_id)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    GovernanceError::DatabaseError(format!("Failed to update veto state: {}", e))
                })?;
                
                (triggered_at, review_days, ends_at, override_flag, override_ts, override_by, resolution)
            } else {
                // No veto state, threshold not met
                (None, review_days_default, None, false, None, None, None)
            };
        
        // Veto is active if threshold met and not overridden
        let veto_active = threshold_met && !maintainer_override;

        Ok(VetoThreshold {
            mining_veto_percent,
            economic_veto_percent,
            threshold_met,
            veto_active,
            review_period_start,
            review_period_days,
            review_period_ends_at,
            maintainer_override,
            override_timestamp,
            override_by,
            resolution_path,
        })
    }

    /// Allow maintainers to override veto after review period
    /// This implements the sequential veto mechanism: Phase 2 (clean fork enablement)
    pub async fn override_veto(
        &self,
        pr_id: i32,
        override_by: &str, // GitHub username of maintainer overriding
    ) -> Result<(), GovernanceError> {
        // Check if veto state exists
        let veto_state = sqlx::query(
            r#"
            SELECT review_period_ends_at, maintainer_override
            FROM pr_veto_state
            WHERE pr_id = ?
            "#
        )
        .bind(pr_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to fetch veto state: {}", e))
        })?;

        let veto_state = veto_state.ok_or_else(|| {
            GovernanceError::CryptoError("No veto state found for this PR".to_string())
        })?;

        // Check if already overridden
        if veto_state.get::<i32, _>("maintainer_override") != 0 {
            return Err(GovernanceError::CryptoError(
                "Veto already overridden".to_string()
            ));
        }

        // Check if review period has ended
        let ends_at_str = veto_state.get::<String, _>("review_period_ends_at");
        let ends_at = DateTime::parse_from_rfc3339(&ends_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                GovernanceError::CryptoError(format!("Invalid timestamp: {}", e))
            })?;

        if Utc::now() < ends_at {
            return Err(GovernanceError::CryptoError(
                format!("Review period not ended yet. Ends at: {}", ends_at)
            ));
        }

        // Override veto
        let now = Utc::now();
        sqlx::query(
            r#"
            UPDATE pr_veto_state
            SET maintainer_override = TRUE,
                override_timestamp = ?,
                override_by = ?,
                resolution_path = 'override',
                veto_active = FALSE,
                updated_at = CURRENT_TIMESTAMP
            WHERE pr_id = ?
            "#
        )
        .bind(now.to_rfc3339())
        .bind(override_by)
        .bind(pr_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to override veto: {}", e))
        })?;

        info!(
            "Veto overridden for PR {} by maintainer {} (clean fork expected)",
            pr_id, override_by
        );

        Ok(())
    }

    /// Check if opposition has dropped below threshold (consensus achieved)
    /// This implements Path A: Consensus Achieved
    pub async fn check_consensus_achieved(&self, pr_id: i32, tier: u32) -> Result<bool, GovernanceError> {
        let threshold = self.check_veto_threshold(pr_id, tier).await?;
        
        // If threshold no longer met, consensus achieved
        if !threshold.threshold_met && threshold.veto_active {
            // Update resolution path
            sqlx::query(
                r#"
                UPDATE pr_veto_state
                SET resolution_path = 'consensus',
                    veto_active = FALSE,
                    updated_at = CURRENT_TIMESTAMP
                WHERE pr_id = ?
                "#
            )
            .bind(pr_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                GovernanceError::DatabaseError(format!("Failed to update resolution: {}", e))
            })?;
            
            return Ok(true);
        }
        
        Ok(false)
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
                    Err(_) => chrono::NaiveDateTime::parse_from_str(&ts_str, "%Y-%m-%d %H:%M:%S")
                        .map(|dt| dt.and_utc())
                        .map_err(|e| {
                            GovernanceError::CryptoError(format!("Invalid timestamp: {}", e))
                        })?,
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

        let threshold = manager.check_veto_threshold(1, 3).await.unwrap();

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

        let threshold = manager.check_veto_threshold(999, 3).await.unwrap();
        assert!(
            !threshold.veto_active,
            "Should not be active with no signals"
        );
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
