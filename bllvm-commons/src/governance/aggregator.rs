//! Contribution Aggregator
//!
//! Aggregates contributions over time windows (30-day rolling for mining, cumulative for zaps)
//! and updates participation weights.

use crate::governance::{ContributionTracker, WeightCalculator};
use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use tracing::info;

/// Contribution aggregator for monthly aggregation
pub struct ContributionAggregator {
    pool: SqlitePool,
    contribution_tracker: ContributionTracker,
    weight_calculator: WeightCalculator,
}

impl ContributionAggregator {
    /// Create a new contribution aggregator
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool: pool.clone(),
            contribution_tracker: ContributionTracker::new(pool.clone()),
            weight_calculator: WeightCalculator::new(pool),
        }
    }

    /// Aggregate merge mining contributions (30-day rolling)
    /// Returns total BTC contributed in the last 30 days
    pub async fn aggregate_merge_mining_monthly(&self, contributor_id: &str) -> Result<f64> {
        let now = Utc::now();
        let thirty_days_ago = now - chrono::Duration::days(30);

        let total: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount_btc), 0.0) as total
            FROM unified_contributions
            WHERE contributor_id = ?
              AND contribution_type LIKE 'merge_mining:%'
              AND timestamp >= ?
            "#,
        )
        .bind(contributor_id)
        .bind(thirty_days_ago)
        .fetch_one(&self.pool)
        .await?;

        Ok(total.unwrap_or(0.0))
    }

    /// Aggregate fee forwarding contributions (30-day rolling)
    /// Returns total BTC forwarded in the last 30 days
    pub async fn aggregate_fee_forwarding_monthly(&self, contributor_id: &str) -> Result<f64> {
        let now = Utc::now();
        let thirty_days_ago = now - chrono::Duration::days(30);

        let total: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount_btc), 0.0) as total
            FROM unified_contributions
            WHERE contributor_id = ?
              AND contribution_type = 'fee_forwarding'
              AND timestamp >= ?
            "#,
        )
        .bind(contributor_id)
        .bind(thirty_days_ago)
        .fetch_one(&self.pool)
        .await?;

        Ok(total.unwrap_or(0.0))
    }

    /// Aggregate cumulative zap contributions (all-time)
    /// Returns total BTC zapped (cumulative)
    pub async fn aggregate_zaps_cumulative(&self, contributor_id: &str) -> Result<f64> {
        let total: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(amount_btc), 0.0) as total
            FROM unified_contributions
            WHERE contributor_id = ?
              AND contribution_type LIKE 'zap:%'
            "#,
        )
        .bind(contributor_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(total.unwrap_or(0.0))
    }

    /// Update all participation weights (called periodically, e.g., daily)
    pub async fn update_all_weights(&self) -> Result<()> {
        info!("Starting participation weight update for all contributors");

        // Update contribution ages first (for cooling-off calculation)
        self.contribution_tracker.update_contribution_ages().await?;

        // Update all participation weights
        self.weight_calculator
            .update_participation_weights()
            .await?;

        info!("Completed participation weight update");
        Ok(())
    }

    /// Get aggregated contributions for a contributor
    pub async fn get_contributor_aggregates(
        &self,
        contributor_id: &str,
    ) -> Result<ContributorAggregates> {
        let merge_mining = self.aggregate_merge_mining_monthly(contributor_id).await?;
        let fee_forwarding = self
            .aggregate_fee_forwarding_monthly(contributor_id)
            .await?;
        let zaps = self.aggregate_zaps_cumulative(contributor_id).await?;

        let total = merge_mining + fee_forwarding + zaps;

        // Get participation weight
        let participation_weight = self
            .weight_calculator
            .get_participation_weight(contributor_id)
            .await?
            .unwrap_or(0.0);

        Ok(ContributorAggregates {
            merge_mining_btc: merge_mining,
            fee_forwarding_btc: fee_forwarding,
            cumulative_zaps_btc: zaps,
            total_contribution_btc: total,
            participation_weight,
        })
    }
}

/// Aggregated contributions for a contributor
#[derive(Debug, Clone)]
pub struct ContributorAggregates {
    pub merge_mining_btc: f64,
    pub fee_forwarding_btc: f64,
    pub cumulative_zaps_btc: f64,
    pub total_contribution_btc: f64,
    pub participation_weight: f64,
}
