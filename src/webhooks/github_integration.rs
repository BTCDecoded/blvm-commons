//! GitHub Integration for Status Checks and Merge Blocking
//!
//! Handles posting status checks and updating merge status based on governance requirements

use chrono::Utc;
use serde_json::Value;
use tracing::{info, warn};

use crate::database::Database;
use crate::enforcement::decision_log::DecisionLogger;
use crate::enforcement::merge_block::MergeBlocker;
use crate::enforcement::status_checks::StatusCheckGenerator;
use crate::error::GovernanceError;
use crate::github::client::GitHubClient;
use crate::validation::review_period::ReviewPeriodValidator;
use crate::validation::threshold::ThresholdValidator;
use crate::validation::tier_classification;
// use crate::economic_nodes::veto::VetoManager;

pub struct GitHubIntegration {
    github_client: GitHubClient,
    database: Database,
    merge_blocker: MergeBlocker,
    decision_logger: DecisionLogger,
}

impl GitHubIntegration {
    pub fn new(
        github_client: GitHubClient,
        database: Database,
        decision_logger: DecisionLogger,
    ) -> Self {
        let merge_blocker = MergeBlocker::new(Some(github_client.clone()), decision_logger.clone());
        Self {
            github_client,
            database,
            merge_blocker,
            decision_logger,
        }
    }

    /// Handle pull request opened event
    pub async fn handle_pr_opened(&self, payload: &Value) -> Result<(), GovernanceError> {
        let repo_name = self.extract_repo_name(payload)?;
        let pr_number = self.extract_pr_number(payload)?;
        let head_sha = self.extract_head_sha(payload)?;
        let (owner, repo) = self.parse_repo_name(&repo_name)?;

        info!(
            "Handling PR opened event for {}/{}#{}",
            owner, repo, pr_number
        );

        // Classify PR tier
        let tier = tier_classification::classify_pr_tier(payload).await;
        let tier_name = self.get_tier_name(tier);

        // Post initial status check
        self.post_initial_status_check(&owner, &repo, &head_sha, tier, tier_name)
            .await?;

        // Set up required status checks for the branch
        // TODO: Implement set_required_checks method or use alternative approach
        // For now, this is handled by GitHub branch protection rules
        warn!("set_required_checks not implemented - using GitHub branch protection rules");

        Ok(())
    }

    /// Handle pull request comment event (signature collection)
    pub async fn handle_pr_comment(&self, payload: &Value) -> Result<(), GovernanceError> {
        let repo_name = self.extract_repo_name(payload)?;
        let pr_number = self.extract_pr_number(payload)?;
        let head_sha = self.extract_head_sha(payload)?;
        let (owner, repo) = self.parse_repo_name(&repo_name)?;

        info!(
            "Handling PR comment event for {}/{}#{}",
            owner, repo, pr_number
        );

        // Update status checks based on current state
        self.update_pr_status_checks(&owner, &repo, &head_sha, pr_number as u64, payload)
            .await?;

        Ok(())
    }

    /// Handle pull request updated event
    pub async fn handle_pr_updated(&self, payload: &Value) -> Result<(), GovernanceError> {
        let repo_name = self.extract_repo_name(payload)?;
        let pr_number = self.extract_pr_number(payload)?;
        let head_sha = self.extract_head_sha(payload)?;
        let (owner, repo) = self.parse_repo_name(&repo_name)?;

        info!(
            "Handling PR updated event for {}/{}#{}",
            owner, repo, pr_number
        );

        // Update all status checks
        self.update_pr_status_checks(&owner, &repo, &head_sha, pr_number as u64, payload)
            .await?;

        Ok(())
    }

    /// Post initial status check when PR is opened
    async fn post_initial_status_check(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        tier: u32,
        tier_name: &str,
    ) -> Result<(), GovernanceError> {
        let status_message = format!(
            "🔍 Governance: Analyzing PR\n\
            Tier {}: {}\n\
            Review period and signature requirements will be checked...",
            tier, tier_name
        );

        self.github_client
            .post_status_check(
                owner,
                repo,
                sha,
                "pending",
                &status_message,
                "governance/analysis",
            )
            .await?;

        Ok(())
    }

    /// Update all status checks for a PR
    async fn update_pr_status_checks(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        pr_number: u64,
        payload: &Value,
    ) -> Result<(), GovernanceError> {
        // Get PR information from database
        let pr_info = self
            .database
            .get_pull_request(owner, pr_number as i32)
            .await?;

        if let Some(pr) = pr_info {
            let layer = pr.layer;
            let tier = tier_classification::classify_pr_tier(payload).await;
            let tier_name = self.get_tier_name(tier);

            // Get combined requirements (Layer + Tier)
            // Note: Using static method for backward compatibility
            // TODO: Update to use ConfigReader-enabled validator when available
            let (sigs_req, sigs_total, review_days) =
                ThresholdValidator::get_combined_requirements_static(layer, tier);
            let _source = ThresholdValidator::get_requirement_source(layer, tier);

            // Check review period
            let review_period_met = self.check_review_period(&pr, review_days).await?;
            let review_period_status = self.generate_review_period_status(&pr, review_days).await?;

            // Check signatures
            let (signatures_met, signature_status) =
                self.check_signatures(&pr, sigs_req, sigs_total).await?;

            // Check economic node veto (Tier 3+)
            let (economic_veto_active, economic_veto_status, veto_threshold) = if tier >= 3 {
                let (active, status, threshold) = self.check_economic_veto(pr.id, tier).await?;
                (active, status, Some(threshold))
            } else {
                (false, String::new(), None)
            };

            // Post individual status checks
            self.post_review_period_status(owner, repo, sha, &review_period_status)
                .await?;
            self.post_signature_status(owner, repo, sha, &signature_status)
                .await?;

            if tier >= 3 {
                self.post_economic_veto_status(owner, repo, sha, &economic_veto_status)
                    .await?;
            }

            // Post combined status
            self.post_combined_status(
                owner,
                repo,
                sha,
                layer,
                tier,
                tier_name,
                review_period_met,
                signatures_met,
                economic_veto_active,
                &review_period_status,
                &signature_status,
                &economic_veto_status,
            )
            .await?;

            // Update merge blocking status
            // Use sequential veto mechanism if available
            let should_block = if let Some(veto) = &veto_threshold {
                crate::enforcement::merge_block::MergeBlocker::should_block_merge_with_veto(
                    review_period_met,
                    signatures_met,
                    veto,
                    tier,
                    false, // emergency_mode
                )?
            } else {
                crate::enforcement::merge_block::MergeBlocker::should_block_merge(
                    review_period_met,
                    signatures_met,
                    economic_veto_active,
                    tier,
                    false, // emergency_mode
                )?
            };
            let reason = crate::enforcement::merge_block::MergeBlocker::get_block_reason(
                review_period_met,
                signatures_met,
                economic_veto_active,
                tier,
                false, // emergency_mode
            );
            self.merge_blocker
                .post_merge_status(owner, repo, sha, should_block, &reason)
                .await?;
        }

        Ok(())
    }

    /// Check review period requirements
    async fn check_review_period(
        &self,
        pr: &crate::database::models::PullRequest,
        required_days: i64,
    ) -> Result<bool, GovernanceError> {
        let opened_at = pr.opened_at;
        Ok(ReviewPeriodValidator::validate_review_period(opened_at, required_days, false).is_ok())
    }

    /// Generate review period status message
    async fn generate_review_period_status(
        &self,
        pr: &crate::database::models::PullRequest,
        required_days: i64,
    ) -> Result<String, GovernanceError> {
        let opened_at = pr.opened_at;
        Ok(StatusCheckGenerator::generate_review_period_status(
            opened_at,
            required_days,
            false,
        ))
    }

    /// Check signature requirements
    async fn check_signatures(
        &self,
        _pr: &crate::database::models::PullRequest,
        required: usize,
        total: usize,
    ) -> Result<(bool, String), GovernanceError> {
        // TODO: Get actual signature count from database
        let current_signatures = 0; // Placeholder
        let signers = vec![]; // Placeholder
        let pending = vec![]; // Placeholder

        let signatures_met = current_signatures >= required;
        let status = StatusCheckGenerator::generate_signature_status(
            current_signatures,
            required,
            total,
            &signers,
            &pending,
        );

        Ok((signatures_met, status))
    }

    /// Check economic node veto status
    /// This now integrates with the new voting system (zap votes + participation votes)
    async fn check_economic_veto(&self, pr_id: i32, tier: u32) -> Result<(bool, String, crate::economic_nodes::VetoThreshold), GovernanceError> {
        use crate::economic_nodes::VetoManager;
        use crate::governance::VoteAggregator;

        let pool = self.database.pool().ok_or_else(|| {
            GovernanceError::DatabaseError("Database pool not available".to_string())
        })?;

        // Check traditional economic node veto with tier-specific thresholds
        // AND logic: both mining and economic thresholds must be met
        let veto_manager = VetoManager::new(pool.clone());
        let veto_threshold = veto_manager
            .check_veto_threshold(pr_id, tier)
            .await
            .map_err(|e| GovernanceError::DatabaseError(format!("Failed to check veto: {}", e)))?;

        // Also check zap votes and participation votes via VoteAggregator
        // This gives us the complete picture of all voting mechanisms
        let vote_aggregator = VoteAggregator::new(pool.clone());

        // Get tier from PR (we need to look it up)
        // For now, assume Tier 3 if we can't determine (conservative)
        let tier = 3; // TODO: Get actual tier from PR

        let economic_veto_blocks = vote_aggregator
            .check_economic_veto_blocking(pr_id, tier)
            .await
            .map_err(|e| {
                GovernanceError::DatabaseError(format!("Failed to check veto blocking: {}", e))
            })?;

        // Get tier-specific thresholds for status message
        let (mining_threshold, economic_threshold) = match tier {
            3 => (30.0, 40.0),
            4 => (25.0, 35.0),
            5 => (50.0, 60.0),
            _ => (30.0, 40.0),
        };

        // Build status message (AND logic: both thresholds must be met)
        let status = if veto_threshold.veto_active || economic_veto_blocks {
            let mut msg = format!(
                "⚠️ Economic Node Veto Active\n\
                Mining Veto: {:.1}% (threshold: {:.0}%)\n\
                Economic Veto: {:.1}% (threshold: {:.0}%)\n\
                Status: BLOCKED (Both thresholds required)",
                veto_threshold.mining_veto_percent, mining_threshold,
                veto_threshold.economic_veto_percent, economic_threshold
            );
            
            // Add review period information if available
            if let Some(ends_at) = veto_threshold.review_period_ends_at {
                let remaining = ends_at - Utc::now();
                if remaining.num_days() > 0 {
                    msg.push_str(&format!("\nReview Period: {} days remaining", remaining.num_days()));
                } else if remaining.num_hours() > 0 {
                    msg.push_str(&format!("\nReview Period: {} hours remaining", remaining.num_hours()));
                } else if veto_threshold.maintainer_override {
                    msg.push_str("\nReview Period: Expired - Maintainer override active");
                } else {
                    msg.push_str("\nReview Period: Expired - Override available");
                }
            }
            
            if let Some(path) = &veto_threshold.resolution_path {
                msg.push_str(&format!("\nResolution: {}", path));
            }
            
            msg
        } else {
            format!(
                "✅ Economic Node Veto: Not Active\n\
                Mining Veto: {:.1}% (threshold: {:.0}%)\n\
                Economic Veto: {:.1}% (threshold: {:.0}%)\n\
                Status: No veto (Both thresholds required)",
                veto_threshold.mining_veto_percent, mining_threshold,
                veto_threshold.economic_veto_percent, economic_threshold
            )
        };

        Ok((veto_threshold.veto_active || economic_veto_blocks, status, veto_threshold))
    }

    /// Post review period status check
    async fn post_review_period_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        status: &str,
    ) -> Result<(), GovernanceError> {
        let state = if status.contains("✅") {
            "success"
        } else {
            "pending"
        };

        // Log the status check
        self.decision_logger.log_status_check(
            sha.parse().unwrap_or(0),
            "governance/review-period",
            state,
            status,
        );

        self.github_client
            .post_status_check(owner, repo, sha, state, status, "governance/review-period")
            .await
    }

    /// Post signature status check
    async fn post_signature_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        status: &str,
    ) -> Result<(), GovernanceError> {
        let state = if status.contains("✅") {
            "success"
        } else {
            "pending"
        };

        // Log the status check
        self.decision_logger.log_status_check(
            sha.parse().unwrap_or(0),
            "governance/signatures",
            state,
            status,
        );

        self.github_client
            .post_status_check(owner, repo, sha, state, status, "governance/signatures")
            .await
    }

    /// Post economic veto status check
    async fn post_economic_veto_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        status: &str,
    ) -> Result<(), GovernanceError> {
        let state = if status.contains("✅") {
            "success"
        } else {
            "failure"
        };

        self.github_client
            .post_status_check(owner, repo, sha, state, status, "governance/economic-veto")
            .await
    }

    /// Post combined status check
    async fn post_combined_status(
        &self,
        owner: &str,
        repo: &str,
        sha: &str,
        _layer: i32,
        tier: u32,
        tier_name: &str,
        review_period_met: bool,
        signatures_met: bool,
        economic_veto_active: bool,
        review_period_status: &str,
        signature_status: &str,
        economic_veto_status: &str,
    ) -> Result<(), GovernanceError> {
        let status = StatusCheckGenerator::generate_tier_status(
            tier,
            tier_name,
            review_period_met,
            signatures_met,
            economic_veto_active,
            review_period_status,
            signature_status,
        );

        let state = if review_period_met && signatures_met && !economic_veto_active {
            "success"
        } else {
            "failure"
        };

        self.github_client
            .post_status_check(owner, repo, sha, state, &status, "governance/combined")
            .await
    }

    /// Extract repository name from payload
    fn extract_repo_name(&self, payload: &Value) -> Result<String, GovernanceError> {
        payload
            .get("repository")
            .and_then(|r| r.get("full_name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| GovernanceError::WebhookError("Missing repository name".to_string()))
    }

    /// Extract PR number from payload
    fn extract_pr_number(&self, payload: &Value) -> Result<i32, GovernanceError> {
        payload
            .get("pull_request")
            .and_then(|pr| pr.get("number"))
            .and_then(|n| n.as_i64())
            .map(|n| n as i32)
            .ok_or_else(|| GovernanceError::WebhookError("Missing PR number".to_string()))
    }

    /// Extract head SHA from payload
    fn extract_head_sha(&self, payload: &Value) -> Result<String, GovernanceError> {
        payload
            .get("pull_request")
            .and_then(|pr| pr.get("head"))
            .and_then(|h| h.get("sha"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| GovernanceError::WebhookError("Missing head SHA".to_string()))
    }

    /// Parse repository name into owner and repo
    fn parse_repo_name(&self, repo_name: &str) -> Result<(String, String), GovernanceError> {
        let parts: Vec<&str> = repo_name.split('/').collect();
        if parts.len() != 2 {
            return Err(GovernanceError::WebhookError(
                "Invalid repository name format".to_string(),
            ));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Get tier name from tier number
    fn get_tier_name(&self, tier: u32) -> &'static str {
        match tier {
            1 => "Routine Maintenance",
            2 => "Feature Changes",
            3 => "Consensus-Adjacent",
            4 => "Emergency Actions",
            5 => "Governance Changes",
            _ => "Unknown",
        }
    }
}
