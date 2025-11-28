//! Governance Phase Calculator
//!
//! Determines governance maturity phase based on measurable metrics:
//! - Block height (time-based maturity)
//! - Economic node count (institutional participation)
//! - Commons contributor count (individual participation)
//!
//! Uses conservative logic: takes most conservative (earliest) phase from all metrics.

use sqlx::SqlitePool;
use tracing::info;

use crate::error::GovernanceError;

/// Governance maturity phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GovernancePhase {
    /// Early phase: < 50K blocks, < 10 nodes, < 10 contributors
    Early = 0,
    /// Growth phase: 50K-200K blocks, 10-30 nodes, 10-100 contributors
    Growth = 1,
    /// Mature phase: 200K+ blocks, 30+ nodes, 100+ contributors
    Mature = 2,
}

impl GovernancePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            GovernancePhase::Early => "early",
            GovernancePhase::Growth => "growth",
            GovernancePhase::Mature => "mature",
        }
    }
}

/// Governance phase calculator
pub struct GovernancePhaseCalculator {
    pool: SqlitePool,
}

impl GovernancePhaseCalculator {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get current governance phase based on measurable metrics
    /// Uses conservative logic: takes most conservative (earliest) phase
    pub async fn get_current_phase(&self) -> Result<GovernancePhase, GovernanceError> {
        let block_height = self.get_block_height().await?;
        let economic_nodes = self.get_economic_node_count().await?;
        let contributors = self.get_contributor_count().await?;

        // Calculate phase for each metric
        let height_phase = Self::determine_phase_by_height(block_height);
        let node_phase = Self::determine_phase_by_nodes(economic_nodes);
        let contributor_phase = Self::determine_phase_by_contributors(contributors);

        // Take most conservative (earliest) phase
        let phase = [height_phase, node_phase, contributor_phase]
            .iter()
            .min()
            .copied()
            .unwrap_or(GovernancePhase::Early);

        info!(
            "Governance phase: {} (height: {}, nodes: {}, contributors: {})",
            phase.as_str(), block_height, economic_nodes, contributors
        );

        Ok(phase)
    }

    /// Get block height from chain state
    async fn get_block_height(&self) -> Result<u64, GovernanceError> {
        // Try to get from chain state table if it exists
        let height: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT MAX(height) FROM blocks
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get block height: {}", e))
        })?;

        Ok(height.unwrap_or(0) as u64)
    }

    /// Get active economic node count
    async fn get_economic_node_count(&self) -> Result<u32, GovernanceError> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM economic_nodes WHERE status = 'active'
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get economic node count: {}", e))
        })?;

        Ok(count.unwrap_or(0) as u32)
    }

    /// Get Commons contributor count (distinct contributors from unified_contributions)
    async fn get_contributor_count(&self) -> Result<u32, GovernanceError> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT contributor_id) 
            FROM unified_contributions 
            WHERE contributor_type IN ('merge_miner', 'fee_forwarder', 'zap_user')
            "#
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get contributor count: {}", e))
        })?;

        Ok(count.unwrap_or(0) as u32)
    }

    /// Determine phase by block height
    fn determine_phase_by_height(height: u64) -> GovernancePhase {
        if height < 50_000 {
            GovernancePhase::Early
        } else if height < 200_000 {
            GovernancePhase::Growth
        } else {
            GovernancePhase::Mature
        }
    }

    /// Determine phase by economic node count
    fn determine_phase_by_nodes(count: u32) -> GovernancePhase {
        if count < 10 {
            GovernancePhase::Early
        } else if count < 30 {
            GovernancePhase::Growth
        } else {
            GovernancePhase::Mature
        }
    }

    /// Determine phase by contributor count
    fn determine_phase_by_contributors(count: u32) -> GovernancePhase {
        if count < 10 {
            GovernancePhase::Early
        } else if count < 100 {
            GovernancePhase::Growth
        } else {
            GovernancePhase::Mature
        }
    }

    /// Get adaptive parameters based on current phase
    pub async fn get_adaptive_parameters(&self) -> Result<AdaptiveParameters, GovernanceError> {
        let phase = self.get_current_phase().await?;

        Ok(match phase {
            GovernancePhase::Early => AdaptiveParameters {
                mining_pool_weight_cap: 0.10,  // 10% cap
                mining_veto_threshold: 25.0,   // 25% hashpower
                economic_veto_threshold: 35.0,  // 35% economic activity
                tier_4_threshold: 2,            // 2 nodes can block
            },
            GovernancePhase::Growth => AdaptiveParameters {
                mining_pool_weight_cap: 0.20,  // 20% cap
                mining_veto_threshold: 30.0,   // 30% hashpower
                economic_veto_threshold: 40.0, // 40% economic activity
                tier_4_threshold: 3,            // 3 nodes can block
            },
            GovernancePhase::Mature => AdaptiveParameters {
                mining_pool_weight_cap: 0.10,  // 10% cap (back to conservative)
                mining_veto_threshold: 35.0,   // 35% hashpower
                economic_veto_threshold: 45.0,  // 45% economic activity
                tier_4_threshold: 5,            // 5 nodes can block
            },
        })
    }
}

/// Adaptive parameters that adjust based on governance phase
#[derive(Debug, Clone)]
pub struct AdaptiveParameters {
    /// Maximum weight cap for mining pools (percentage of total)
    pub mining_pool_weight_cap: f64,
    /// Mining veto threshold (percentage of hashpower)
    pub mining_veto_threshold: f64,
    /// Economic veto threshold (percentage of economic activity)
    pub economic_veto_threshold: f64,
    /// Tier 4 threshold (number of nodes that can block)
    pub tier_4_threshold: u32,
}

