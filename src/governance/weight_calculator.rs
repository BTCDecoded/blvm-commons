//! Weight Calculator
//!
//! Calculates governance participation weights using quadratic formula,
//! applies weight caps, and checks cooling-off periods.

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use tracing::{debug, info};

/// Weight calculator with quadratic formula, caps, and cooling-off
pub struct WeightCalculator {
    pool: SqlitePool,
    cap_percentage: f64,            // 0.05 = 5% cap
    cooling_off_threshold_btc: f64, // 0.1 BTC
    cooling_off_period_days: u32,   // 30 days
}

impl WeightCalculator {
    /// Create a new weight calculator
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            cap_percentage: 0.05,           // 5% cap
            cooling_off_threshold_btc: 0.1, // 0.1 BTC threshold
            cooling_off_period_days: 30,    // 30 days cooling period
        }
    }

    /// Calculate ongoing participation weight (quadratic, BTC-based)
    pub fn calculate_participation_weight(
        &self,
        merge_mining_btc: f64,
        fee_forwarding_btc: f64,
        cumulative_zaps_btc: f64,
    ) -> f64 {
        let total_btc = merge_mining_btc + fee_forwarding_btc + cumulative_zaps_btc;
        total_btc.sqrt()
    }

    /// Apply weight cap to prevent whale dominance
    pub fn apply_weight_cap(&self, calculated_weight: f64, total_system_weight: f64) -> f64 {
        let max_weight = total_system_weight * self.cap_percentage;
        calculated_weight.min(max_weight)
    }

    /// Check if contribution is eligible for voting (cooling-off period)
    pub fn check_cooling_off(
        &self,
        contribution_amount_btc: f64,
        contribution_age_days: u32,
    ) -> bool {
        if contribution_amount_btc >= self.cooling_off_threshold_btc {
            contribution_age_days >= self.cooling_off_period_days
        } else {
            true // No cooling period for small contributions
        }
    }

    /// Calculate per-proposal zap vote weight (quadratic, BTC-based)
    pub fn calculate_zap_vote_weight(&self, zap_amount_btc: f64) -> f64 {
        zap_amount_btc.sqrt()
    }

    ///
    pub fn get_proposal_vote_weight(
        &self,
        participation_weight: f64,
        proposal_zap_amount_btc: Option<f64>,
        total_system_weight: f64,
        contribution_age_days: Option<u32>, // For cooling-off check
    ) -> f64 {
        let base_weight = if let Some(zap_btc) = proposal_zap_amount_btc {
            // Check cooling-off for proposal zap
            if let Some(age) = contribution_age_days {
                if !self.check_cooling_off(zap_btc, age) {
                    // Contribution too new, use participation weight only
                    return self.apply_weight_cap(participation_weight, total_system_weight);
                }
            }
            let zap_weight = self.calculate_zap_vote_weight(zap_btc);
            // Use 10% of participation weight as minimum, or zap weight if higher
            zap_weight.max(participation_weight * 0.1)
        } else {
            participation_weight
        };

        // Apply weight cap
        self.apply_weight_cap(base_weight, total_system_weight)
    }

    /// Calculate and update participation weights for all contributors
    pub async fn update_participation_weights(&self) -> Result<()> {
        // First, update contribution ages (for cooling-off calculation)
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

        // Get all unique contributors
        #[derive(sqlx::FromRow)]
        struct ContributorRow {
            contributor_id: String,
            contributor_type: String,
        }

        let contributors = sqlx::query_as::<_, ContributorRow>(
            r#"
            SELECT DISTINCT contributor_id, contributor_type
            FROM unified_contributions
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let contributor_count = contributors.len();

        // First pass: calculate all base weights and store contribution data
        struct ContributorData {
            contributor_id: String,
            contributor_type: String,
            merge_mining_btc: f64,
            fee_forwarding_btc: f64,
            cumulative_zaps_btc: f64,
            total_contribution_btc: f64,
            base_weight: f64,
        }

        let mut contributor_data = Vec::new();
        let now = Utc::now();
        let thirty_days_ago = now - chrono::Duration::days(30);

        for contributor in contributors {
            let contributor_id = contributor.contributor_id.clone();

            // Merge mining (30-day rolling)
            let merge_mining_btc: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT COALESCE(SUM(amount_btc), 0.0) as total
                FROM unified_contributions
                WHERE contributor_id = ?
                  AND contribution_type LIKE 'merge_mining:%'
                  AND timestamp >= ?
                "#,
            )
            .bind(&contributor_id)
            .bind(thirty_days_ago)
            .fetch_one(&self.pool)
            .await?;
            let merge_mining_btc = merge_mining_btc.unwrap_or(0.0);

            // Fee forwarding (30-day rolling)
            let fee_forwarding_btc: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT COALESCE(SUM(amount_btc), 0.0) as total
                FROM unified_contributions
                WHERE contributor_id = ?
                  AND contribution_type = 'fee_forwarding'
                  AND timestamp >= ?
                "#,
            )
            .bind(&contributor_id)
            .bind(thirty_days_ago)
            .fetch_one(&self.pool)
            .await?;
            let fee_forwarding_btc = fee_forwarding_btc.unwrap_or(0.0);

            // Zaps (cumulative - all time, but exclude contributions in cooling-off period)
            let cumulative_zaps_btc: Option<f64> = sqlx::query_scalar(
                r#"
                SELECT COALESCE(SUM(amount_btc), 0.0) as total
                FROM unified_contributions
                WHERE contributor_id = ?
                  AND contribution_type LIKE 'zap:%'
                  AND (
                    amount_btc < ? OR
                    contribution_age_days >= ?
                  )
                "#,
            )
            .bind(&contributor_id)
            .bind(self.cooling_off_threshold_btc)
            .bind(self.cooling_off_period_days)
            .fetch_one(&self.pool)
            .await?;
            let cumulative_zaps_btc = cumulative_zaps_btc.unwrap_or(0.0);

            // Calculate base weight (quadratic)
            let total_contribution_btc =
                merge_mining_btc + fee_forwarding_btc + cumulative_zaps_btc;
            let base_weight = self.calculate_participation_weight(
                merge_mining_btc,
                fee_forwarding_btc,
                cumulative_zaps_btc,
            );

            contributor_data.push(ContributorData {
                contributor_id,
                contributor_type: contributor.contributor_type,
                merge_mining_btc,
                fee_forwarding_btc,
                cumulative_zaps_btc,
                total_contribution_btc,
                base_weight,
            });
        }

        // Calculate uncapped total for cap calculation
        let uncapped_total: f64 = contributor_data.iter().map(|d| d.base_weight).sum();
        
        // Apply caps based on uncapped total (one pass only - prevents iterative convergence to zero)
        // Only apply caps if there are multiple contributors (single contributor has 100% by definition)
        let mut final_total = 0.0;
        let mut capped_weights = Vec::new();
        for data in &contributor_data {
            let capped_weight = if contributor_count > 1 {
                // Cap is 5% of the uncapped total
                let max_allowed = uncapped_total * self.cap_percentage;
                data.base_weight.min(max_allowed)
            } else {
                // Single contributor: no cap applied
                data.base_weight
            };
            final_total += capped_weight;
            capped_weights.push((data.contributor_id.clone(), capped_weight));
        }

        // Update all weights in database
        for (idx, data) in contributor_data.iter().enumerate() {
            let capped_weight = capped_weights[idx].1;
            sqlx::query(
                r#"
                INSERT INTO participation_weights
                (contributor_id, contributor_type, merge_mining_btc, fee_forwarding_btc, cumulative_zaps_btc, total_contribution_btc, base_weight, capped_weight, total_system_weight, last_updated)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                ON CONFLICT(contributor_id) DO UPDATE SET
                    contributor_type = excluded.contributor_type,
                    merge_mining_btc = excluded.merge_mining_btc,
                    fee_forwarding_btc = excluded.fee_forwarding_btc,
                    cumulative_zaps_btc = excluded.cumulative_zaps_btc,
                    total_contribution_btc = excluded.total_contribution_btc,
                    base_weight = excluded.base_weight,
                    capped_weight = excluded.capped_weight,
                    total_system_weight = excluded.total_system_weight,
                    last_updated = CURRENT_TIMESTAMP
                "#,
            )
            .bind(&data.contributor_id)
            .bind(&data.contributor_type)
            .bind(data.merge_mining_btc)
            .bind(data.fee_forwarding_btc)
            .bind(data.cumulative_zaps_btc)
            .bind(data.total_contribution_btc)
            .bind(data.base_weight) // Actual base weight
            .bind(capped_weight) // Capped weight
            .bind(final_total)
            .execute(&self.pool)
            .await?;

            debug!(
                "Updated participation weight for {}: base={:.2}, capped={:.2} (contributions: {:.8} BTC)",
                data.contributor_id, data.base_weight, capped_weight, data.total_contribution_btc
            );
        }

        info!(
            "Updated participation weights for {} contributors",
            contributor_count
        );
        Ok(())
    }

    /// Calculate total system weight (sum of all capped weights)
    pub async fn calculate_total_system_weight(&self) -> Result<f64> {
        let total: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT SUM(capped_weight) as total
            FROM participation_weights
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(total.unwrap_or(0.0))
    }

    /// Get participation weight for a contributor
    pub async fn get_participation_weight(&self, contributor_id: &str) -> Result<Option<f64>> {
        let weight: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT capped_weight
            FROM participation_weights
            WHERE contributor_id = ?
            "#,
        )
        .bind(contributor_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(weight)
    }
}
