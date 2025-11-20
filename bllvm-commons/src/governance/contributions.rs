//! Contribution Tracking Service
//!
//! Tracks governance contributions (merge mining, fee forwarding, zaps)
//! and records them in the unified contributions table.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use tracing::info;

/// Contribution tracking service
pub struct ContributionTracker {
    pool: SqlitePool,
}

impl ContributionTracker {
    /// Create a new contribution tracker
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Record a merge mining contribution (1% of secondary chain rewards)
    pub async fn record_merge_mining_contribution(
        &self,
        contributor_id: &str,
        chain_id: &str,
        reward_amount_btc: f64,
        contribution_amount_btc: f64, // 1% of reward
        timestamp: DateTime<Utc>,
    ) -> Result<()> {
        // Record in unified contributions table
        sqlx::query(
            r#"
            INSERT INTO unified_contributions
            (contributor_id, contributor_type, contribution_type, amount_btc, timestamp, contribution_age_days, period_type, verified)
            VALUES (?, ?, ?, ?, ?, 0, 'monthly', ?)
            "#,
        )
        .bind(contributor_id)
        .bind("merge_miner")
        .bind(format!("merge_mining:{}", chain_id))
        .bind(contribution_amount_btc)
        .bind(timestamp)
        .bind(true)  // Verified (on-chain)
        .execute(&self.pool)
        .await?;

        info!(
            "Recorded merge mining contribution: {} BTC (from {} BTC reward on {}) for {}",
            contribution_amount_btc, reward_amount_btc, chain_id, contributor_id
        );

        Ok(())
    }

    /// Record a fee forwarding contribution
    pub async fn record_fee_forwarding_contribution(
        &self,
        contributor_id: &str,
        tx_hash: &str,
        amount_btc: f64,
        commons_address: &str,
        block_height: i32,
        timestamp: DateTime<Utc>,
    ) -> Result<()> {
        // First record in fee_forwarding_contributions table
        sqlx::query(
            r#"
            INSERT INTO fee_forwarding_contributions
            (contributor_id, tx_hash, block_height, amount_btc, commons_address, timestamp, verified)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(contributor_id)
        .bind(tx_hash)
        .bind(block_height)
        .bind(amount_btc)
        .bind(commons_address)
        .bind(timestamp)
        .bind(true)  // Verified (on-chain)
        .execute(&self.pool)
        .await?;

        // Also record in unified contributions table
        sqlx::query(
            r#"
            INSERT INTO unified_contributions
            (contributor_id, contributor_type, contribution_type, amount_btc, timestamp, contribution_age_days, period_type, verified)
            VALUES (?, ?, ?, ?, ?, 0, 'monthly', ?)
            "#,
        )
        .bind(contributor_id)
        .bind("fee_forwarder")
        .bind("fee_forwarding")
        .bind(amount_btc)
        .bind(timestamp)
        .bind(true)  // Verified (on-chain)
        .execute(&self.pool)
        .await?;

        info!(
            "Recorded fee forwarding contribution: {} BTC (tx: {}) for {}",
            amount_btc, tx_hash, contributor_id
        );

        Ok(())
    }

    /// Record a zap contribution (called from zap tracker)
    pub async fn record_zap_contribution(
        &self,
        contributor_id: &str, // Sender pubkey
        amount_btc: f64,
        timestamp: DateTime<Utc>,
        is_proposal_zap: bool,
    ) -> Result<()> {
        // Record in unified contributions table
        let contribution_type = if is_proposal_zap {
            "zap:proposal"
        } else {
            "zap:general"
        };
        sqlx::query(
            r#"
            INSERT INTO unified_contributions
            (contributor_id, contributor_type, contribution_type, amount_btc, timestamp, contribution_age_days, period_type, verified)
            VALUES (?, ?, ?, ?, ?, 0, ?, ?)
            "#,
        )
        .bind(contributor_id)
        .bind("zap_user")
        .bind(contribution_type)
        .bind(amount_btc)
        .bind(timestamp)
        .bind("cumulative")
        .bind(true)  // Verified (Nostr event)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get total contributions for a contributor in a time period
    pub async fn get_contributor_total(
        &self,
        contributor_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<ContributorTotal> {
        let rows = sqlx::query_as::<_, (String, Option<f64>)>(
            r#"
            SELECT 
                contribution_type,
                SUM(amount_btc) as total_btc
            FROM unified_contributions
            WHERE contributor_id = ?
              AND timestamp >= ?
              AND timestamp <= ?
            GROUP BY contribution_type
            "#,
        )
        .bind(contributor_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;

        let mut merge_mining_btc = 0.0;
        let mut fee_forwarding_btc = 0.0;
        let mut zaps_btc = 0.0;

        for (contribution_type, total_btc) in rows {
            let total = total_btc.unwrap_or(0.0);
            if contribution_type.starts_with("merge_mining:") {
                merge_mining_btc += total;
            } else if contribution_type == "fee_forwarding" {
                fee_forwarding_btc += total;
            } else if contribution_type.starts_with("zap:") {
                zaps_btc += total;
            }
        }

        Ok(ContributorTotal {
            merge_mining_btc,
            fee_forwarding_btc,
            zaps_btc,
            total_btc: merge_mining_btc + fee_forwarding_btc + zaps_btc,
        })
    }

    /// Update contribution age for cooling-off period calculation
    pub async fn update_contribution_ages(&self) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE unified_contributions
            SET contribution_age_days = CAST(
                (julianday('now') - julianday(timestamp)) AS INTEGER
            )
            WHERE contribution_age_days != CAST(
                (julianday('now') - julianday(timestamp)) AS INTEGER
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Contributor total contributions
#[derive(Debug, Clone)]
pub struct ContributorTotal {
    pub merge_mining_btc: f64,
    pub fee_forwarding_btc: f64,
    pub zaps_btc: f64,
    pub total_btc: f64,
}
