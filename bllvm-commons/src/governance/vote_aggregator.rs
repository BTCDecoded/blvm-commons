//! Vote Aggregator
//!
//! Aggregates votes from multiple sources (zap votes, participation votes)
//! and calculates totals for governance proposals.

use crate::economic_nodes::VetoManager;
use crate::governance::WeightCalculator;
use crate::nostr::zap_voting::ZapVotingProcessor;
use anyhow::Result;
use sqlx::SqlitePool;
use tracing::info;

/// Vote aggregator for governance proposals
pub struct VoteAggregator {
    pool: SqlitePool,
    zap_voting: ZapVotingProcessor,
    weight_calculator: WeightCalculator,
    veto_manager: VetoManager,
}

impl VoteAggregator {
    /// Create a new vote aggregator
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool: pool.clone(),
            zap_voting: ZapVotingProcessor::new(pool.clone()),
            weight_calculator: WeightCalculator::new(pool.clone()),
            veto_manager: VetoManager::new(pool),
        }
    }
    
    /// Aggregate all votes for a proposal
    pub async fn aggregate_proposal_votes(
        &self,
        pr_id: i32,
        tier: u8,
    ) -> Result<ProposalVoteResult> {
        // Get fixed threshold for this tier
        let threshold = self.get_threshold_for_tier(tier)?;
        
        // Get all zap votes for this proposal
        let zap_votes = self.zap_voting.get_proposal_votes(pr_id).await?;
        let zap_totals = self.zap_voting.get_proposal_vote_totals(pr_id).await?;
        
        // Get participation-based votes (from economic nodes and contributors)
        let participation_votes = self.get_participation_votes(pr_id).await?;
        
        // Combine all votes
        let total_support = zap_totals.support_weight + participation_votes.support_weight;
        let total_veto = zap_totals.veto_weight + participation_votes.veto_weight;
        let total_abstain = zap_totals.abstain_weight + participation_votes.abstain_weight;
        let total_votes = total_support + total_veto + total_abstain;
        
        // Check if threshold met
        let threshold_met = total_votes >= threshold as f64;
        
        // Check if veto blocks
        // Two types of veto:
        // 1. Economic node veto (30% hashpower OR 40% economic activity) - for Tier 3+
        // 2. Zap vote veto (40% of total zap votes)
        let economic_veto_blocks = if tier >= 3 {
            self.check_economic_veto_blocking(pr_id, tier).await?
        } else {
            false
        };
        
        // Zap veto blocks if 40%+ of zap votes (not total votes including participation)
        // This ensures zap veto is independent of participation votes
        let zap_veto_blocks = if zap_totals.total_weight > 0.0 {
            (zap_totals.veto_weight / zap_totals.total_weight) >= 0.4
        } else {
            false
        };
        
        let veto_blocks = economic_veto_blocks || zap_veto_blocks;
        
        info!(
            "Proposal {} votes: support={:.2}, veto={:.2}, abstain={:.2}, total={:.2}, threshold={}, met={}, veto_blocks={}",
            pr_id, total_support, total_veto, total_abstain, total_votes, threshold, threshold_met, veto_blocks
        );
        
        Ok(ProposalVoteResult {
            pr_id,
            tier,
            threshold,
            total_votes,
            support_votes: total_support,
            veto_votes: total_veto,
            abstain_votes: total_abstain,
            zap_vote_count: zap_totals.total_count,
            participation_vote_count: participation_votes.total_count,
            threshold_met,
            veto_blocks,
        })
    }
    
    /// Get participation-based votes (from economic nodes and contributors)
    /// This integrates with the economic node veto system and participation weights
    async fn get_participation_votes(&self, pr_id: i32) -> Result<ParticipationVoteTotals> {
        // Check economic node veto threshold (30% hashpower or 40% economic activity)
        let veto_threshold = self.veto_manager.check_veto_threshold(pr_id).await
            .map_err(|e| anyhow::anyhow!("Failed to check veto threshold: {}", e))?;
        
        // Convert veto threshold to participation votes
        // Economic nodes that veto contribute to veto_weight
        // Economic nodes that support contribute to support_weight
        // The veto system tracks mining pools (hashpower) and other economic nodes (economic activity)
        
        let mut support_weight = 0.0;
        let mut veto_weight = 0.0;
        
        // If veto threshold is met, we have significant veto weight
        // The percentages represent the portion of total economic activity
        if veto_threshold.threshold_met {
            // Veto is active - calculate veto weight from percentages
            // Mining veto: 30%+ threshold means at least 30% of hashpower vetoed
            // Economic veto: 40%+ threshold means at least 40% of economic activity vetoed
            veto_weight = veto_threshold.mining_veto_percent.max(veto_threshold.economic_veto_percent);
        }
        // Note: We don't add support_weight when there's no veto, as participation votes
        // should only come from explicit votes, not from the absence of vetoes
        
        // Also get participation weights from contributors (merge miners, fee forwarders, zap users)
        // These can vote using their participation weights
        // TODO: Query participation_votes table when contributors submit explicit votes
        // For now, we rely on economic node veto system for participation votes
        
        Ok(ParticipationVoteTotals {
            support_weight,
            veto_weight,
            abstain_weight: 0.0,
            total_count: 0, // Count would come from actual vote submissions
        })
    }
    
    /// Check if economic node veto blocks this proposal (for Tier 3+)
    pub async fn check_economic_veto_blocking(&self, pr_id: i32, tier: u8) -> Result<bool> {
        if tier < 3 {
            return Ok(false); // Veto only applies to Tier 3+
        }
        
        let veto_threshold = self.veto_manager.check_veto_threshold(pr_id).await
            .map_err(|e| anyhow::anyhow!("Failed to check veto threshold: {}", e))?;
        
        // Veto blocks if: 30%+ hashpower OR 40%+ economic activity vetoes
        Ok(veto_threshold.threshold_met)
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

/// Participation vote totals (from economic nodes and contributors)
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

