use crate::error::GovernanceError;

pub struct ThresholdValidator;

impl ThresholdValidator {
    pub fn validate_threshold(
        current_signatures: usize,
        required_signatures: usize,
        total_maintainers: usize,
    ) -> Result<bool, GovernanceError> {
        if current_signatures >= required_signatures {
            Ok(true)
        } else {
            Err(GovernanceError::ThresholdError(format!(
                "Signature threshold not met. Required: {required_signatures}/{total_maintainers} signatures, Current: {current_signatures}/{total_maintainers}"
            )))
        }
    }

    pub fn get_threshold_for_layer(layer: i32) -> Result<(usize, usize), GovernanceError> {
        match layer {
            1 | 2 => Ok((6, 7)), // Constitutional layers: 6-of-7
            3 => Ok((4, 5)),     // Implementation layer: 4-of-5
            4 => Ok((3, 5)),     // Application layer: 3-of-5
            5 => Ok((2, 3)),     // Extension layer: 2-of-3
            other => Err(GovernanceError::ThresholdError(format!(
                "Unknown layer {other}; refusing 1-of-1"
            ))),
        }
    }

    pub fn get_review_period_for_layer(
        layer: i32,
        emergency_mode: bool,
    ) -> Result<i64, GovernanceError> {
        let period = match layer {
            1 | 2 => 180, // Constitutional layers: 180 days
            3 => 90,      // Implementation layer: 90 days
            4 => 60,      // Application layer: 60 days
            5 => 14,      // Extension layer: 14 days
            other => {
                return Err(GovernanceError::ThresholdError(format!(
                    "Unknown layer {other}; refusing a default review period"
                )));
            }
        };
        if emergency_mode { Ok(30) } else { Ok(period) }
    }

    pub fn format_threshold_status(
        current: usize,
        required: usize,
        total: usize,
        signers: &[String],
        pending: &[String],
    ) -> String {
        format!(
            "❌ Governance: Signatures Missing\nRequired: {}-of-{} | Current: {}/{}\nSigned by: {}\nPending: {}",
            required,
            total,
            current,
            total,
            signers.join(", "),
            pending.join(", ")
        )
    }

    /// Async threshold check (signature count only; maintainer multisig).
    pub async fn validate_threshold_for_merge(
        _pool: &sqlx::SqlitePool,
        _pr_id: i32,
        _tier: u32,
        current_signatures: usize,
        required_signatures: usize,
        total_maintainers: usize,
    ) -> Result<bool, GovernanceError> {
        // Signature threshold only
        if current_signatures >= required_signatures {
            Ok(true)
        } else {
            Err(GovernanceError::ThresholdError(format!(
                "Signature threshold not met. Required: {required_signatures}/{total_maintainers} signatures, Current: {current_signatures}/{total_maintainers}"
            )))
        }
    }

    /// Get tier-specific signature requirements
    pub fn get_tier_threshold(tier: u32) -> Result<(usize, usize), GovernanceError> {
        match tier {
            1 => Ok((3, 5)), // Tier 1: Routine (3-of-5)
            2 => Ok((4, 5)), // Tier 2: Features (4-of-5)
            3 => Ok((5, 5)), // Tier 3: Consensus-adjacent (5-of-5)
            4 => Ok((4, 5)), // Tier 4: Emergency (4-of-5)
            5 => Ok((5, 5)), // Tier 5: Governance (5-of-5)
            other => Err(GovernanceError::ThresholdError(format!(
                "Unknown tier {other}; refusing 1-of-1"
            ))),
        }
    }

    /// Get tier-specific review period
    pub fn get_tier_review_period(tier: u32) -> Result<i64, GovernanceError> {
        match tier {
            1 => Ok(7),   // Tier 1: 7 days
            2 => Ok(30),  // Tier 2: 30 days
            3 => Ok(90),  // Tier 3: 90 days
            4 => Ok(0),   // Tier 4: Emergency (no review period)
            5 => Ok(180), // Tier 5: 180 days
            other => Err(GovernanceError::ThresholdError(format!(
                "Unknown tier {other}; refusing a default review period"
            ))),
        }
    }

    /// Get combined requirements using "most restrictive wins" rule
    pub fn get_combined_requirements(
        layer: i32,
        tier: u32,
    ) -> Result<(usize, usize, i64), GovernanceError> {
        let (layer_sigs_req, layer_sigs_total) = Self::get_threshold_for_layer(layer)?;
        let layer_review = Self::get_review_period_for_layer(layer, false)?;

        let (tier_sigs_req, tier_sigs_total) = Self::get_tier_threshold(tier)?;
        let tier_review = Self::get_tier_review_period(tier)?;

        // Take most restrictive (higher requirements)
        let sigs_req = layer_sigs_req.max(tier_sigs_req);
        let sigs_total = layer_sigs_total.max(tier_sigs_total);
        let review = layer_review.max(tier_review);

        Ok((sigs_req, sigs_total, review))
    }

    /// Get requirement source (for logging/display)
    pub fn get_requirement_source(layer: i32, tier: u32) -> Result<String, GovernanceError> {
        let (layer_sigs_req, _) = Self::get_threshold_for_layer(layer)?;
        let layer_review = Self::get_review_period_for_layer(layer, false)?;

        let (tier_sigs_req, _) = Self::get_tier_threshold(tier)?;
        let tier_review = Self::get_tier_review_period(tier)?;

        Ok(
            if layer_sigs_req >= tier_sigs_req && layer_review >= tier_review {
                format!("Layer {layer} requirements")
            } else if tier_sigs_req >= layer_sigs_req && tier_review >= layer_review {
                format!("Tier {tier} requirements")
            } else {
                format!("Combined Layer {layer} + Tier {tier} requirements")
            },
        )
    }
}
