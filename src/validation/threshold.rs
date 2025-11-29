use crate::error::GovernanceError;
use crate::governance::config_reader::ConfigReader;
use std::sync::Arc;

pub struct ThresholdValidator {
    config: Option<Arc<ConfigReader>>,
}

impl ThresholdValidator {
    /// Create a new ThresholdValidator without config (uses hardcoded defaults)
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Create with ConfigReader for governance-controlled thresholds
    pub fn with_config(config: Arc<ConfigReader>) -> Self {
        Self { config: Some(config) }
    }
}

impl Default for ThresholdValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ThresholdValidator {

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
                "Signature threshold not met. Required: {}/{} signatures, Current: {}/{}",
                required_signatures, total_maintainers, current_signatures, total_maintainers
            )))
        }
    }

    /// Get threshold for layer (with config support)
    pub async fn get_threshold_for_layer(&self, layer: i32) -> (usize, usize) {
        if let Some(ref config) = self.config {
            if let Ok((req, total)) = config.get_layer_signatures(layer).await {
                return (req, total);
            }
        }
        // Fallback to hardcoded defaults
        match layer {
            1 | 2 => (6, 7), // Constitutional layers: 6-of-7
            3 => (4, 5),     // Implementation layer: 4-of-5
            4 => (3, 5),     // Application layer: 3-of-5
            5 => (2, 3),     // Extension layer: 2-of-3
            _ => (1, 1),     // Default fallback
        }
    }

    /// Get threshold for layer (static method for backward compatibility)
    pub fn get_threshold_for_layer_static(layer: i32) -> (usize, usize) {
        match layer {
            1 | 2 => (6, 7),
            3 => (4, 5),
            4 => (3, 5),
            5 => (2, 3),
            _ => (1, 1),
        }
    }

    /// Get review period for layer (with config support)
    pub async fn get_review_period_for_layer(&self, layer: i32, emergency_mode: bool) -> i64 {
        if emergency_mode {
            return 30; // Emergency mode: 30 days for all layers
        }

        if let Some(ref config) = self.config {
            if let Ok(period) = config.get_layer_review_period(layer).await {
                return period;
            }
        }
        // Fallback to hardcoded defaults
        match layer {
            1 | 2 => 180, // Constitutional layers: 180 days
            3 => 90,      // Implementation layer: 90 days
            4 => 60,      // Application layer: 60 days
            5 => 14,      // Extension layer: 14 days
            _ => 30,      // Default fallback
        }
    }

    /// Get review period for layer (static method for backward compatibility)
    pub fn get_review_period_for_layer_static(layer: i32, emergency_mode: bool) -> i64 {
        if emergency_mode {
            30
        } else {
            match layer {
                1 | 2 => 180,
                3 => 90,
                4 => 60,
                5 => 14,
                _ => 30,
            }
        }
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

    /// Validate threshold with economic node veto check for Tier 3+ PRs
    /// Note: Economic node veto integration will be added in a future update
    pub async fn validate_threshold_with_veto(
        _pool: &sqlx::SqlitePool,
        _pr_id: i32,
        _tier: u32,
        current_signatures: usize,
        required_signatures: usize,
        total_maintainers: usize,
    ) -> Result<bool, GovernanceError> {
        // For now, just check basic signature threshold
        // Economic node veto integration will be added later
        if current_signatures >= required_signatures {
            Ok(true)
        } else {
            Err(GovernanceError::ThresholdError(format!(
                "Signature threshold not met. Required: {}/{} signatures, Current: {}/{}",
                required_signatures, total_maintainers, current_signatures, total_maintainers
            )))
        }
    }

    /// Get tier-specific signature requirements (with config support)
    pub async fn get_tier_threshold(&self, tier: u32) -> (usize, usize) {
        if let Some(ref config) = self.config {
            if let Ok((req, total)) = config.get_tier_signatures(tier).await {
                return (req, total);
            }
        }
        // Fallback to hardcoded defaults
        match tier {
            1 => (3, 5), // Tier 1: Routine (3-of-5)
            2 => (4, 5), // Tier 2: Features (4-of-5)
            3 => (5, 5), // Tier 3: Consensus-adjacent (5-of-5)
            4 => (4, 5), // Tier 4: Emergency (4-of-5)
            5 => (5, 5), // Tier 5: Governance (5-of-5)
            _ => (1, 1), // Default fallback
        }
    }

    /// Get tier-specific signature requirements (static method for backward compatibility)
    pub fn get_tier_threshold_static(tier: u32) -> (usize, usize) {
        match tier {
            1 => (3, 5),
            2 => (4, 5),
            3 => (5, 5),
            4 => (4, 5),
            5 => (5, 5),
            _ => (1, 1),
        }
    }

    /// Get tier-specific review period (with config support)
    pub async fn get_tier_review_period(&self, tier: u32) -> i64 {
        if let Some(ref config) = self.config {
            if let Ok(period) = config.get_tier_review_period(tier).await {
                return period;
            }
        }
        // Fallback to hardcoded defaults
        match tier {
            1 => 7,   // Tier 1: 7 days
            2 => 30,  // Tier 2: 30 days
            3 => 90,  // Tier 3: 90 days
            4 => 0,   // Tier 4: Emergency (no review period)
            5 => 180, // Tier 5: 180 days
            _ => 30,  // Default fallback
        }
    }

    /// Get tier-specific review period (static method for backward compatibility)
    pub fn get_tier_review_period_static(tier: u32) -> i64 {
        match tier {
            1 => 7,
            2 => 30,
            3 => 90,
            4 => 0,
            5 => 180,
            _ => 30,
        }
    }

    /// Get combined requirements using "most restrictive wins" rule (with config support)
    pub async fn get_combined_requirements(&self, layer: i32, tier: u32) -> (usize, usize, i64) {
        let (layer_sigs_req, layer_sigs_total) = self.get_threshold_for_layer(layer).await;
        let layer_review = self.get_review_period_for_layer(layer, false).await;

        let (tier_sigs_req, tier_sigs_total) = self.get_tier_threshold(tier).await;
        let tier_review = self.get_tier_review_period(tier).await;

        // Take most restrictive (higher requirements)
        let sigs_req = layer_sigs_req.max(tier_sigs_req);
        let sigs_total = layer_sigs_total.max(tier_sigs_total);
        let review = layer_review.max(tier_review);

        (sigs_req, sigs_total, review)
    }

    /// Get combined requirements (static method for backward compatibility)
    pub fn get_combined_requirements_static(layer: i32, tier: u32) -> (usize, usize, i64) {
        let (layer_sigs_req, layer_sigs_total) = Self::get_threshold_for_layer_static(layer);
        let layer_review = Self::get_review_period_for_layer_static(layer, false);

        let (tier_sigs_req, tier_sigs_total) = Self::get_tier_threshold_static(tier);
        let tier_review = Self::get_tier_review_period_static(tier);

        let sigs_req = layer_sigs_req.max(tier_sigs_req);
        let sigs_total = layer_sigs_total.max(tier_sigs_total);
        let review = layer_review.max(tier_review);

        (sigs_req, sigs_total, review)
    }

    /// Get requirement source (for logging/display)
    pub fn get_requirement_source(layer: i32, tier: u32) -> String {
        let (layer_sigs_req, _) = Self::get_threshold_for_layer(layer);
        let layer_review = Self::get_review_period_for_layer(layer, false);

        let (tier_sigs_req, _) = Self::get_tier_threshold(tier);
        let tier_review = Self::get_tier_review_period(tier);

        if layer_sigs_req >= tier_sigs_req && layer_review >= tier_review {
            format!("Layer {} requirements", layer)
        } else if tier_sigs_req >= layer_sigs_req && tier_review >= layer_review {
            format!("Tier {} requirements", tier)
        } else {
            format!("Combined Layer {} + Tier {} requirements", layer, tier)
        }
    }

    /// Check if economic veto is required for the given layer and tier
    pub fn requires_economic_veto(_layer: i32, tier: u32) -> bool {
        // Economic veto is required for Tier 3+ regardless of layer
        tier >= 3
    }
}
