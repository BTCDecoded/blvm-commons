//! Zap-to-Vote Logic
//!
//! Processes zaps to governance events and converts them into votes.
//! Calculates vote weights using quadratic formula and records votes in database.

use crate::nostr::zap_tracker::ZapContribution;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use tracing::{info, warn};

/// Zap vote record
#[derive(Debug, Clone)]
pub struct ZapVote {
    pub governance_event_id: String,
    pub sender_pubkey: String,
    pub amount_msat: u64,
    pub amount_btc: f64,
    pub vote_weight: f64, // sqrt(amount_btc)
    pub vote_type: VoteType,
    pub timestamp: DateTime<Utc>,
    pub pr_id: i32,
}

/// Vote type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoteType {
    Support,
    Veto,
    Abstain,
}

impl VoteType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VoteType::Support => "support",
            VoteType::Veto => "veto",
            VoteType::Abstain => "abstain",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "veto" | "oppose" | "against" => VoteType::Veto,
            "abstain" | "neutral" => VoteType::Abstain,
            _ => VoteType::Support, // Default to support
        }
    }
}

/// Zap-to-vote processor
pub struct ZapVotingProcessor {
    pool: SqlitePool,
}

impl ZapVotingProcessor {
    /// Create a new zap voting processor
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Process a zap contribution and convert it to a vote if it's a proposal zap
    pub async fn process_proposal_zap(
        &self,
        zap: &ZapContribution,
        pr_id: i32,
        governance_event_id: &str,
    ) -> Result<Option<ZapVote>> {
        // Only process proposal zaps (zaps to governance events)
        if !zap.is_proposal_zap {
            return Ok(None);
        }

        // Verify this zap is for the correct governance event
        if zap.governance_event_id.as_deref() != Some(governance_event_id) {
            return Ok(None);
        }

        // Get sender pubkey (required for voting)
        let sender_pubkey = zap
            .sender_pubkey
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Proposal zap missing sender pubkey"))?;

        // Calculate vote weight using quadratic formula
        let vote_weight = (zap.amount_btc).sqrt();

        // Determine vote type from zap message (default to support)
        let vote_type = if let Some(ref message) = zap.message {
            Self::parse_vote_type_from_message(message)
        } else {
            VoteType::Support // Default to support if no message
        };

        // Check if vote already exists (prevent duplicates)
        let existing: Option<i32> = sqlx::query_scalar(
            r#"
            SELECT id FROM proposal_zap_votes
            WHERE governance_event_id = ? AND sender_pubkey = ?
            "#,
        )
        .bind(governance_event_id)
        .bind(sender_pubkey)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            warn!(
                "Vote already exists for {} from {}",
                governance_event_id, sender_pubkey
            );
            return Ok(None);
        }

        // Record vote in database
        sqlx::query(
            r#"
            INSERT INTO proposal_zap_votes
            (pr_id, governance_event_id, sender_pubkey, amount_msat, amount_btc, vote_weight, vote_type, timestamp, verified)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(pr_id)
        .bind(governance_event_id)
        .bind(sender_pubkey)
        .bind(zap.amount_msat as i64)
        .bind(zap.amount_btc)
        .bind(vote_weight)
        .bind(vote_type.as_str())
        .bind(zap.timestamp)
        .bind(true)  // Verified if it came from zap tracker
        .execute(&self.pool)
        .await?;

        info!(
            "Recorded zap vote: {} {} votes (weight: {:.2}) for proposal {} from {}",
            vote_type.as_str(),
            zap.amount_btc,
            vote_weight,
            governance_event_id,
            sender_pubkey
        );

        Ok(Some(ZapVote {
            governance_event_id: governance_event_id.to_string(),
            sender_pubkey: sender_pubkey.clone(),
            amount_msat: zap.amount_msat,
            amount_btc: zap.amount_btc,
            vote_weight,
            vote_type,
            timestamp: zap.timestamp,
            pr_id,
        }))
    }

    /// Parse vote type from zap message
    /// Messages can contain "veto", "abstain", or default to support
    fn parse_vote_type_from_message(message: &str) -> VoteType {
        let msg_lower = message.to_lowercase();

        if msg_lower.contains("veto")
            || msg_lower.contains("oppose")
            || msg_lower.contains("against")
        {
            VoteType::Veto
        } else if msg_lower.contains("abstain") || msg_lower.contains("neutral") {
            VoteType::Abstain
        } else {
            VoteType::Support // Default to support
        }
    }

    /// Get all votes for a proposal
    pub async fn get_proposal_votes(&self, pr_id: i32) -> Result<Vec<ZapVote>> {
        #[derive(sqlx::FromRow)]
        struct ZapVoteRow {
            governance_event_id: String,
            sender_pubkey: String,
            amount_msat: i64,
            amount_btc: f64,
            vote_weight: f64,
            vote_type: String,
            timestamp: DateTime<Utc>,
            pr_id: i32,
        }

        let rows = sqlx::query_as::<_, ZapVoteRow>(
            r#"
            SELECT governance_event_id, sender_pubkey, amount_msat, amount_btc, vote_weight, vote_type, timestamp, pr_id
            FROM proposal_zap_votes
            WHERE pr_id = ?
            ORDER BY timestamp DESC
            "#,
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| ZapVote {
                governance_event_id: row.governance_event_id,
                sender_pubkey: row.sender_pubkey,
                amount_msat: row.amount_msat as u64,
                amount_btc: row.amount_btc,
                vote_weight: row.vote_weight,
                vote_type: match row.vote_type.as_str() {
                    "support" => VoteType::Support,
                    "veto" => VoteType::Veto,
                    "abstain" => VoteType::Abstain,
                    _ => VoteType::Support,
                },
                timestamp: row.timestamp,
                pr_id: row.pr_id,
            })
            .collect())
    }

    /// Get vote totals for a proposal
    pub async fn get_proposal_vote_totals(&self, pr_id: i32) -> Result<VoteTotals> {
        #[derive(sqlx::FromRow)]
        struct VoteTotalsRow {
            vote_type: String,
            total_weight: Option<f64>,
            vote_count: i64,
        }

        let rows = sqlx::query_as::<_, VoteTotalsRow>(
            r#"
            SELECT vote_type, SUM(vote_weight) as total_weight, COUNT(*) as vote_count
            FROM proposal_zap_votes
            WHERE pr_id = ?
            GROUP BY vote_type
            "#,
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await?;

        let mut support_weight = 0.0;
        let mut veto_weight = 0.0;
        let mut abstain_weight = 0.0;
        let mut support_count = 0;
        let mut veto_count = 0;
        let mut abstain_count = 0;

        for row in rows {
            let weight = row.total_weight.unwrap_or(0.0);
            let count = row.vote_count as u32;

            match row.vote_type.as_str() {
                "support" => {
                    support_weight = weight;
                    support_count = count;
                }
                "veto" => {
                    veto_weight = weight;
                    veto_count = count;
                }
                "abstain" => {
                    abstain_weight = weight;
                    abstain_count = count;
                }
                _ => {}
            }
        }

        Ok(VoteTotals {
            support_weight,
            veto_weight,
            abstain_weight,
            total_weight: support_weight + veto_weight + abstain_weight,
            support_count,
            veto_count,
            abstain_count,
            total_count: support_count + veto_count + abstain_count,
        })
    }
}

/// Vote totals for a proposal
#[derive(Debug, Clone)]
pub struct VoteTotals {
    pub support_weight: f64,
    pub veto_weight: f64,
    pub abstain_weight: f64,
    pub total_weight: f64,
    pub support_count: u32,
    pub veto_count: u32,
    pub abstain_count: u32,
    pub total_count: u32,
}
