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

    pub fn get_threshold_for_layer(layer: i32) -> (usize, usize) {
        match layer {
            1 | 2 => (6, 7), // Constitutional layers: 6-of-7
            3 => (4, 5),     // Implementation layer: 4-of-5
            4 => (3, 5),     // Application layer: 3-of-5
            5 => (2, 3),     // Extension layer: 2-of-3
            _ => (1, 1),     // Default fallback
        }
    }

    /// Layer review clock. `emergency_mode` does not shorten it.
    /// Layers 1–2 use the 365-day consensus clock. A 0-day clock is not implemented.
    pub fn get_review_period_for_layer(layer: i32, emergency_mode: bool) -> i64 {
        let _emergency_does_not_shorten = emergency_mode;
        match layer {
            1 | 2 => 365, // Constitutional / consensus: 365 days
            3 => 90,      // Implementation layer: 90 days
            4 => 60,      // Application layer: 60 days, including emergency
            5 => 14,      // Extension layer: 14 days
            _ => 30,      // Default fallback
        }
    }

    /// Layer-4 emergency stays 60 days. A met emergency quorum does not open a 0-day clock.
    pub fn layer_4_emergency_review_days(emergency_quorum_met: bool) -> i64 {
        let _quorum_does_not_zero_the_clock = emergency_quorum_met;
        60
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
    pub fn get_tier_threshold(tier: u32) -> (usize, usize) {
        match tier {
            1 => (3, 5), // Tier 1: Routine (3-of-5)
            2 => (4, 5), // Tier 2: Features (4-of-5)
            3 => (5, 5), // Tier 3: Consensus-adjacent (5-of-5)
            4 => (4, 5), // Tier 4: Emergency (4-of-5)
            5 => (5, 5), // Tier 5: Governance (5-of-5)
            _ => (1, 1), // Default fallback
        }
    }

    /// Get tier-specific review period
    pub fn get_tier_review_period(tier: u32) -> i64 {
        match tier {
            1 => 7,   // Tier 1: 7 days
            2 => 30,  // Tier 2: 30 days
            3 => 90,  // Tier 3: 90 days
            4 => 60,  // Tier 4: emergency floor. 0-day review is not implemented.
            5 => 180, // Tier 5: 180 days
            _ => 30,  // Default fallback
        }
    }

    /// Get combined requirements using "most restrictive wins" rule
    pub fn get_combined_requirements(layer: i32, tier: u32) -> (usize, usize, i64) {
        let (layer_sigs_req, layer_sigs_total) = Self::get_threshold_for_layer(layer);
        let layer_review = Self::get_review_period_for_layer(layer, false);

        let (tier_sigs_req, tier_sigs_total) = Self::get_tier_threshold(tier);
        let tier_review = Self::get_tier_review_period(tier);

        // Take most restrictive (higher requirements). Floors only raise.
        let sigs_req = layer_sigs_req.max(tier_sigs_req);
        let sigs_total = layer_sigs_total.max(tier_sigs_total);
        let review = layer_review.max(tier_review);

        (sigs_req, sigs_total, review)
    }

    /// Security-control floor for one path. `None` means the path adds no floor.
    /// Floors are 7-of-7/180, 6-of-7/90, and 5-of-7/30. They never lower a clock.
    pub fn security_control_floor(path: &str) -> Option<(usize, usize, i64)> {
        let normalized = path.replace('\\', "/");
        if normalized.contains("src/crypto/key_management.rs") || normalized.contains("genesis") {
            Some((7, 7, 180))
        } else if normalized.contains("src/crypto/") && normalized.contains("signature") {
            Some((6, 7, 90))
        } else if normalized.contains("src/crypto/") {
            Some((5, 7, 30))
        } else {
            None
        }
    }

    /// Raise `(required, total, days)` to the highest security-control floor in `paths`.
    pub fn raise_to_security_floor(
        requirements: (usize, usize, i64),
        paths: &[&str],
    ) -> (usize, usize, i64) {
        let (mut required, mut total, mut days) = requirements;
        for path in paths {
            if let Some((floor_required, floor_total, floor_days)) =
                Self::security_control_floor(path)
            {
                required = required.max(floor_required);
                total = total.max(floor_total);
                days = days.max(floor_days);
            }
        }
        (required, total, days)
    }

    /// Get requirement source (for logging/display)
    pub fn get_requirement_source(layer: i32, tier: u32) -> String {
        let (layer_sigs_req, _) = Self::get_threshold_for_layer(layer);
        let layer_review = Self::get_review_period_for_layer(layer, false);

        let (tier_sigs_req, _) = Self::get_tier_threshold(tier);
        let tier_review = Self::get_tier_review_period(tier);

        if layer_sigs_req >= tier_sigs_req && layer_review >= tier_review {
            format!("Layer {layer} requirements")
        } else if tier_sigs_req >= layer_sigs_req && tier_review >= layer_review {
            format!("Tier {tier} requirements")
        } else {
            format!("Combined Layer {layer} + Tier {tier} requirements")
        }
    }
}
