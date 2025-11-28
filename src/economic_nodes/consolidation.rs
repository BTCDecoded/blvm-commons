//! Consolidation Monitoring
//!
//! Monitors mining pool and economic node consolidation to adjust thresholds adaptively.
//! Tracks top pool percentages and top economic node percentages.

use sqlx::SqlitePool;
use tracing::info;

use crate::error::GovernanceError;

/// Consolidation monitor for adaptive threshold adjustment
pub struct ConsolidationMonitor {
    pool: SqlitePool,
}

impl ConsolidationMonitor {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get mining consolidation metrics
    pub async fn get_mining_consolidation(&self) -> Result<MiningConsolidationMetrics, GovernanceError> {
        // Get top pool hashpower
        let top_pool: Option<(String, f64)> = sqlx::query_as(
            r#"
            SELECT entity_name, weight
            FROM economic_nodes
            WHERE node_type = 'mining_pool' AND status = 'active'
            ORDER BY weight DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get top pool: {}", e))
        })?;

        // Get total mining weight
        let total_mining_weight: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(weight), 0.0)
            FROM economic_nodes
            WHERE node_type = 'mining_pool' AND status = 'active'
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get total mining weight: {}", e))
        })?;

        let total_mining_weight = total_mining_weight.unwrap_or(0.0);

        // Get top 3 pools
        let top_3_pools: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT entity_name, weight
            FROM economic_nodes
            WHERE node_type = 'mining_pool' AND status = 'active'
            ORDER BY weight DESC
            LIMIT 3
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get top 3 pools: {}", e))
        })?;

        let top_pool_percent = if total_mining_weight > 0.0 {
            top_pool.as_ref().map(|(_, w)| (w / total_mining_weight) * 100.0).unwrap_or(0.0)
        } else {
            0.0
        };

        let top_3_percent = if total_mining_weight > 0.0 {
            let top_3_total: f64 = top_3_pools.iter().map(|(_, w)| w).sum();
            (top_3_total / total_mining_weight) * 100.0
        } else {
            0.0
        };

        Ok(MiningConsolidationMetrics {
            top_pool_name: top_pool.map(|(n, _)| n),
            top_pool_percent,
            top_3_percent,
            total_pools: top_3_pools.len() as u32,
        })
    }

    /// Get economic node consolidation metrics
    pub async fn get_economic_consolidation(&self) -> Result<EconomicConsolidationMetrics, GovernanceError> {
        // Get top 3 economic nodes (non-mining pools)
        let top_3_nodes: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT entity_name, weight
            FROM economic_nodes
            WHERE node_type != 'mining_pool' AND status = 'active'
            ORDER BY weight DESC
            LIMIT 3
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get top 3 economic nodes: {}", e))
        })?;

        // Get total economic weight
        let total_economic_weight: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(weight), 0.0)
            FROM economic_nodes
            WHERE node_type != 'mining_pool' AND status = 'active'
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get total economic weight: {}", e))
        })?;

        let total_economic_weight = total_economic_weight.unwrap_or(0.0);

        let top_3_percent = if total_economic_weight > 0.0 {
            let top_3_total: f64 = top_3_nodes.iter().map(|(_, w)| w).sum();
            (top_3_total / total_economic_weight) * 100.0
        } else {
            0.0
        };

        Ok(EconomicConsolidationMetrics {
            top_3_node_names: top_3_nodes.iter().map(|(n, _)| n.clone()).collect(),
            top_3_percent,
            total_nodes: top_3_nodes.len() as u32,
        })
    }

    /// Calculate adaptive thresholds based on consolidation
    pub async fn calculate_adaptive_thresholds(
        &self,
        base_mining_threshold: f64,
        base_economic_threshold: f64,
    ) -> Result<(f64, f64), GovernanceError> {
        let mining_metrics = self.get_mining_consolidation().await?;
        let economic_metrics = self.get_economic_consolidation().await?;

        // Adjust mining threshold based on consolidation
        let mining_threshold = if mining_metrics.top_pool_percent > 30.0 {
            base_mining_threshold * 1.5 // 50% increase if single pool > 30%
        } else if mining_metrics.top_pool_percent > 20.0 {
            base_mining_threshold * 1.25 // 25% increase if single pool > 20%
        } else if mining_metrics.top_3_percent > 70.0 {
            base_mining_threshold * 1.3 // 30% increase if top 3 > 70%
        } else if mining_metrics.top_3_percent > 50.0 {
            base_mining_threshold * 1.15 // 15% increase if top 3 > 50%
        } else {
            base_mining_threshold // No adjustment if decentralized
        };

        // Adjust economic threshold based on consolidation
        let economic_threshold = if economic_metrics.top_3_percent > 70.0 {
            base_economic_threshold * 1.2 // 20% increase if top 3 > 70%
        } else if economic_metrics.top_3_percent > 50.0 {
            base_economic_threshold * 1.1 // 10% increase if top 3 > 50%
        } else {
            base_economic_threshold // No adjustment if decentralized
        };

        info!(
            "Adaptive thresholds: mining={:.1}% (base={:.1}%), economic={:.1}% (base={:.1}%)",
            mining_threshold, base_mining_threshold, economic_threshold, base_economic_threshold
        );

        Ok((mining_threshold, economic_threshold))
    }
}

#[derive(Debug, Clone)]
pub struct MiningConsolidationMetrics {
    pub top_pool_name: Option<String>,
    pub top_pool_percent: f64,
    pub top_3_percent: f64,
    pub total_pools: u32,
}

#[derive(Debug, Clone)]
pub struct EconomicConsolidationMetrics {
    pub top_3_node_names: Vec<String>,
    pub top_3_percent: f64,
    pub total_nodes: u32,
}

