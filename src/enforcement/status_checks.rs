use crate::validation::review_period::ReviewPeriodValidator;
use crate::validation::threshold::ThresholdValidator;
use chrono::{DateTime, Utc};

pub struct StatusCheckGenerator;

impl StatusCheckGenerator {
    pub fn generate_review_period_status(
        opened_at: DateTime<Utc>,
        required_days: i64,
        emergency_mode: bool,
    ) -> String {
        Self::generate_review_period_status_with_dry_run(
            opened_at,
            required_days,
            emergency_mode,
            false,
        )
    }

    pub fn generate_review_period_status_with_dry_run(
        opened_at: DateTime<Utc>,
        required_days: i64,
        emergency_mode: bool,
        dry_run: bool,
    ) -> String {
        let remaining_days =
            ReviewPeriodValidator::get_remaining_days(opened_at, required_days, emergency_mode);

        let prefix = if dry_run { "[DRY-RUN] " } else { "" };

        if remaining_days > 0 {
            let earliest_merge = ReviewPeriodValidator::get_earliest_merge_date(
                opened_at,
                required_days,
                emergency_mode,
            );

            format!(
                "{}❌ Governance: Review Period Not Met\nRequired: {} days | Elapsed: {} days\nEarliest merge: {}",
                prefix,
                required_days,
                (Utc::now() - opened_at).num_days(),
                earliest_merge.format("%Y-%m-%d")
            )
        } else {
            format!("{}✅ Governance: Review Period Met", prefix)
        }
    }

    pub fn generate_signature_status(
        current_signatures: usize,
        required_signatures: usize,
        total_maintainers: usize,
        signers: &[String],
        pending: &[String],
    ) -> String {
        Self::generate_signature_status_with_dry_run(
            current_signatures,
            required_signatures,
            total_maintainers,
            signers,
            pending,
            false,
        )
    }

    pub fn generate_signature_status_with_dry_run(
        current_signatures: usize,
        required_signatures: usize,
        total_maintainers: usize,
        signers: &[String],
        pending: &[String],
        dry_run: bool,
    ) -> String {
        let prefix = if dry_run { "[DRY-RUN] " } else { "" };

        if current_signatures >= required_signatures {
            format!("{}✅ Governance: Signatures Complete", prefix)
        } else {
            let base_status = ThresholdValidator::format_threshold_status(
                current_signatures,
                required_signatures,
                total_maintainers,
                signers,
                pending,
            );
            format!("{}{}", prefix, base_status)
        }
    }

    pub fn generate_combined_status(
        review_period_met: bool,
        signatures_met: bool,
        review_period_status: &str,
        signature_status: &str,
    ) -> String {
        if review_period_met && signatures_met {
            "✅ Governance: All Requirements Met - Ready to Merge".to_string()
        } else {
            format!(
                "❌ Governance: Requirements Not Met\n\n{}\n\n{}",
                review_period_status, signature_status
            )
        }
    }

    /// Generate status check with tier classification and economic node veto status
    pub fn generate_tier_status(
        tier: u32,
        tier_name: &str,
        review_period_met: bool,
        signatures_met: bool,
        economic_veto_active: bool,
        review_period_status: &str,
        signature_status: &str,
    ) -> String {
        let tier_emoji = match tier {
            1 => "🔧", // Routine
            2 => "✨", // Feature
            3 => "⚡", // Consensus-Adjacent
            4 => "🚨", // Emergency
            5 => "🏛️", // Governance
            _ => "❓",
        };

        let mut status = format!("{} Tier {}: {}\n", tier_emoji, tier, tier_name);

        if economic_veto_active && tier >= 3 {
            status.push_str("⚠️ Economic Node Veto Active\n");
        }

        if review_period_met && signatures_met && !economic_veto_active {
            status.push_str("✅ Governance: All Requirements Met - Ready to Merge");
        } else {
            status.push_str("❌ Governance: Requirements Not Met\n");
            status.push_str(&format!(
                "\n{}\n\n{}",
                review_period_status, signature_status
            ));

            if economic_veto_active && tier >= 3 {
                status.push_str("\n\n⚠️ Economic Node Veto: 30%+ hashpower AND 40%+ economic activity has vetoed this change");
            }
        }

        status
    }

    /// Generate economic node veto status
    pub fn generate_economic_veto_status(
        veto_active: bool,
        mining_veto_percent: f64,
        economic_veto_percent: f64,
        total_nodes: u32,
        veto_count: u32,
    ) -> String {
        if veto_active {
            format!(
                "⚠️ Economic Node Veto Active\n\
                Mining Veto: {:.1}% (threshold: 30%)\n\
                Economic Veto: {:.1}% (threshold: 40%)\n\
                Total Nodes: {} | Veto Count: {}",
                mining_veto_percent, economic_veto_percent, total_nodes, veto_count
            )
        } else {
            format!(
                "✅ Economic Node Veto: Not Active\n\
                Mining Veto: {:.1}% (threshold: 30%)\n\
                Economic Veto: {:.1}% (threshold: 40%)\n\
                Total Nodes: {} | Veto Count: {}",
                mining_veto_percent, economic_veto_percent, total_nodes, veto_count
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_generate_review_period_status_met() {
        let opened_at = Utc::now() - Duration::days(10);
        let required_days = 7;

        let status =
            StatusCheckGenerator::generate_review_period_status(opened_at, required_days, false);

        assert!(status.contains("✅"), "Should show review period met");
    }

    #[test]
    fn test_generate_review_period_status_not_met() {
        let opened_at = Utc::now() - Duration::days(3);
        let required_days = 7;

        let status =
            StatusCheckGenerator::generate_review_period_status(opened_at, required_days, false);

        assert!(status.contains("❌"), "Should show review period not met");
        assert!(
            status.contains("Required: 7 days"),
            "Should show required days"
        );
    }

    #[test]
    fn test_generate_review_period_status_dry_run() {
        let opened_at = Utc::now() - Duration::days(3);
        let required_days = 7;

        let status = StatusCheckGenerator::generate_review_period_status_with_dry_run(
            opened_at,
            required_days,
            false,
            true, // dry_run
        );

        assert!(
            status.contains("[DRY-RUN]"),
            "Should include dry-run prefix"
        );
    }

    #[test]
    fn test_generate_signature_status_complete() {
        let status = StatusCheckGenerator::generate_signature_status(
            5, // current_signatures
            4, // required_signatures
            5, // total_maintainers
            &["alice".to_string(), "bob".to_string()],
            &[],
        );

        assert!(status.contains("✅"), "Should show signatures complete");
    }

    #[test]
    fn test_generate_signature_status_incomplete() {
        let status = StatusCheckGenerator::generate_signature_status(
            2, // current_signatures
            4, // required_signatures
            5, // total_maintainers
            &["alice".to_string(), "bob".to_string()],
            &["charlie".to_string()],
        );

        assert!(
            !status.contains("✅ Governance: Signatures Complete"),
            "Should not show complete when threshold not met"
        );
    }

    #[test]
    fn test_generate_signature_status_dry_run() {
        let status = StatusCheckGenerator::generate_signature_status_with_dry_run(
            5,
            4,
            5,
            &["alice".to_string()],
            &[],
            true, // dry_run
        );

        assert!(
            status.contains("[DRY-RUN]"),
            "Should include dry-run prefix"
        );
    }

    #[test]
    fn test_generate_combined_status_all_met() {
        let status = StatusCheckGenerator::generate_combined_status(
            true, // review_period_met
            true, // signatures_met
            "Review period met",
            "Signatures complete",
        );

        assert!(status.contains("✅"), "Should show all requirements met");
        assert!(
            status.contains("Ready to Merge"),
            "Should indicate ready to merge"
        );
    }

    #[test]
    fn test_generate_combined_status_not_met() {
        let status = StatusCheckGenerator::generate_combined_status(
            false, // review_period_met
            true,  // signatures_met
            "Review period not met",
            "Signatures complete",
        );

        assert!(status.contains("❌"), "Should show requirements not met");
        assert!(
            status.contains("Review period not met"),
            "Should include review period status"
        );
    }

    #[test]
    fn test_generate_tier_status_routine() {
        let status = StatusCheckGenerator::generate_tier_status(
            1, // tier
            "Routine Maintenance",
            true,
            true,
            false,
            "Review period met",
            "Signatures complete",
        );

        assert!(status.contains("🔧"), "Should have routine emoji");
        assert!(status.contains("Tier 1"), "Should show tier number");
    }

    #[test]
    fn test_generate_tier_status_emergency() {
        let status = StatusCheckGenerator::generate_tier_status(
            4, // tier
            "Emergency",
            true,
            true,
            false,
            "Review period met",
            "Signatures complete",
        );

        assert!(status.contains("🚨"), "Should have emergency emoji");
        assert!(status.contains("Tier 4"), "Should show tier number");
    }

    #[test]
    fn test_generate_tier_status_with_veto() {
        let status = StatusCheckGenerator::generate_tier_status(
            3, // tier
            "Consensus-Adjacent",
            true,
            true,
            true, // economic_veto_active
            "Review period met",
            "Signatures complete",
        );

        assert!(
            status.contains("⚠️ Economic Node Veto Active"),
            "Should show veto active"
        );
    }

    #[test]
    fn test_generate_economic_veto_status_active() {
        let status = StatusCheckGenerator::generate_economic_veto_status(
            true, // veto_active
            35.0, // mining_veto_percent
            45.0, // economic_veto_percent
            10,   // total_nodes
            4,    // veto_count
        );

        assert!(status.contains("⚠️"), "Should show veto active");
        assert!(status.contains("35.0%"), "Should show mining veto percent");
        assert!(
            status.contains("45.0%"),
            "Should show economic veto percent"
        );
    }

    #[test]
    fn test_generate_economic_veto_status_not_active() {
        let status = StatusCheckGenerator::generate_economic_veto_status(
            false, // veto_active
            20.0,  // mining_veto_percent
            30.0,  // economic_veto_percent
            10,    // total_nodes
            2,     // veto_count
        );

        assert!(status.contains("✅"), "Should show veto not active");
        assert!(status.contains("20.0%"), "Should show mining veto percent");
    }
}
