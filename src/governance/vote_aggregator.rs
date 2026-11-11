//! Vote Aggregator
//!
//! Aggregates votes from maintainers for governance proposals.
//! Governance is maintainer-only multisig.
//! Zap votes are tracked for transparency/reporting but do NOT affect governance decisions.
//! Maintainer signatures come from the database (populated by webhooks from GitHub PR reviews).

use crate::database::queries::Queries;
use crate::governance::WeightCalculator;
use crate::nostr::zap_voting::ZapVotingProcessor;
use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;

/// Vote aggregator for governance proposals
pub struct VoteAggregator {
    pool: SqlitePool,
    zap_voting: ZapVotingProcessor,
    _weight_calculator: WeightCalculator,
}

impl VoteAggregator {
    /// Create a new vote aggregator
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool: pool.clone(),
            zap_voting: ZapVotingProcessor::new(pool.clone()),
            _weight_calculator: WeightCalculator::new(pool.clone()),
        }
    }

    /// Aggregate all votes for a proposal (maintainer multisig only)
    pub async fn aggregate_proposal_votes(
        &self,
        pr_id: i32,
        tier: u8,
    ) -> Result<ProposalVoteResult> {
        // Get fixed threshold for this tier (maintainer signatures required)
        let threshold = self.get_threshold_for_tier(tier)?;

        // Get zap votes for reporting/transparency only (not used for governance)
        let _zap_votes = self.zap_voting.get_proposal_votes(pr_id).await?;
        let zap_totals = self.zap_voting.get_proposal_vote_totals(pr_id).await?;

        // Governance is maintainer-only: votes come from maintainer signatures in DB
        // (populated by webhooks from GitHub PR reviews/approvals)
        let maintainer_votes = Queries::get_pull_request_by_id(&self.pool, pr_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get PR {}: {}", pr_id, e))?
            .map(|pr| pr.signatures.len() as u32)
            .unwrap_or(0);
        let total_votes = maintainer_votes as f64;

        // Check if threshold met (maintainer signature threshold)
        let threshold_met = total_votes >= threshold as f64;

        let veto_blocks = false;

        info!(
            "Proposal {} votes: maintainer_votes={}, threshold={}, met={}",
            pr_id, maintainer_votes, threshold, threshold_met
        );

        Ok(ProposalVoteResult {
            pr_id,
            tier,
            threshold,
            total_votes,
            support_votes: total_votes,
            veto_votes: 0.0,
            abstain_votes: 0.0,
            zap_vote_count: zap_totals.total_count,
            participation_vote_count: 0,
            threshold_met,
            veto_blocks,
        })
    }

    /// Get fixed vote threshold for tier
    pub fn get_threshold_for_tier(&self, tier: u8) -> Result<u32> {
        match tier {
            1 => Ok(100),   // Tier 1: Routine Maintenance
            2 => Ok(500),   // Tier 2: Minor Changes
            3 => Ok(1_000), // Tier 3: Significant Changes
            4 => Ok(2_500), // Tier 4: Major Changes
            5 => Ok(5_000), // Tier 5: Constitutional Changes
            _ => Err(anyhow::anyhow!("Invalid tier: {}", tier)),
        }
    }
}

/// Participation vote totals (reporting / transparency only)
#[derive(Debug, Clone)]
pub struct ParticipationVoteTotals {
    pub support_weight: f64,
    pub veto_weight: f64,
    pub abstain_weight: f64,
    pub total_count: u32,
}

/// Complete vote aggregation result for a proposal
#[derive(Debug, Clone)]
pub struct ProposalVoteResult {
    pub pr_id: i32,
    pub tier: u8,
    pub threshold: u32,
    pub total_votes: f64,
    pub support_votes: f64,
    pub veto_votes: f64,
    pub abstain_votes: f64,
    pub zap_vote_count: u32,
    pub participation_vote_count: u32,
    pub threshold_met: bool,
    pub veto_blocks: bool,
}
