//! Governance Action Publisher
//!
//! Publishes governance action events (merges, releases, etc.) to Nostr
//! with full layer + tier information and signature details.

use anyhow::{anyhow, Result};
use chrono::Utc;
use nostr_sdk::prelude::*;
use tracing::{error, info};

use crate::nostr::client::NostrClient;
use crate::nostr::events::{
    GovernanceActionEvent, LayerRequirement, TierRequirement, CombinedRequirement,
    KeyholderSignature, EconomicVetoStatus,
};

/// Publisher for governance action events
pub struct GovernanceActionPublisher {
    client: NostrClient,
    governance_config: String,  // e.g., "commons_mainnet"
    zap_address: Option<String>,  // Lightning address for donations
}

impl GovernanceActionPublisher {
    /// Create new governance action publisher
    pub fn new(
        client: NostrClient,
        governance_config: String,
        zap_address: Option<String>,
    ) -> Self {
        Self {
            client,
            governance_config,
            zap_address,
        }
    }

    /// Publish a governance action event (merge, release, etc.)
    pub async fn publish_action(
        &self,
        action: &str,  // "merge" | "release" | "budget" | "keyholder_change"
        governance_tier: u32,
        governance_layer: u32,
        repository: &str,
        final_signatures: &str,  // e.g., "6-of-7"
        final_review_days: u32,
        commit_hash: Option<&str>,
        pr_number: Option<i32>,
        description: &str,
        layer_req: LayerRequirement,
        tier_req: TierRequirement,
        combined_req: CombinedRequirement,
        signatures: Vec<KeyholderSignature>,
        economic_veto_status: EconomicVetoStatus,
        review_period_ends: Option<chrono::DateTime<chrono::Utc>>,
    ) -> crate::error::Result<()> {
        info!(
            "Publishing governance action: {} for {}/PR#{}",
            action,
            repository,
            pr_number.unwrap_or(0)
        );

        // Create governance action event
        let action_event = GovernanceActionEvent {
            description: description.to_string(),
            pr_url: pr_number.map(|n| {
                format!("https://github.com/BTCDecoded/{}/pull/{}", repository, n)
            }),
            layer_requirement: layer_req,
            tier_requirement: tier_req,
            combined_requirement: combined_req.clone(),
            signatures,
            economic_veto_status,
            review_period_ends,
        };

        // Create Nostr event
        let event = self.create_nostr_event(
            action,
            governance_tier,
            governance_layer,
            repository,
            final_signatures,
            final_review_days,
            commit_hash,
            pr_number,
            &action_event,
        )?;

        // Publish to relays
        self.client.publish_event(event).await?;

        info!("Successfully published governance action event");
        Ok(())
    }

    /// Create Nostr event from governance action
    fn create_nostr_event(
        &self,
        action: &str,
        governance_tier: u32,
        governance_layer: u32,
        repository: &str,
        final_signatures: &str,
        final_review_days: u32,
        commit_hash: Option<&str>,
        pr_number: Option<i32>,
        action_event: &GovernanceActionEvent,
    ) -> Result<Event> {
        let content = action_event.to_json()
            .map_err(|e| anyhow!("Failed to serialize action event: {}", e))?;

        let mut tags = vec![
            Tag::Generic(TagKind::Custom("d".into()), vec!["btc-commons-governance-action".to_string()]),
            Tag::Generic(TagKind::Custom("action".into()), vec![action.to_string()]),
            Tag::Generic(TagKind::Custom("governance_tier".into()), vec![governance_tier.to_string()]),
            Tag::Generic(TagKind::Custom("governance_layer".into()), vec![governance_layer.to_string()]),
            Tag::Generic(TagKind::Custom("repository".into()), vec![repository.to_string()]),
            Tag::Generic(TagKind::Custom("governance_config".into()), vec![self.governance_config.clone()]),
            Tag::Generic(TagKind::Custom("final_signatures".into()), vec![final_signatures.to_string()]),
            Tag::Generic(TagKind::Custom("final_review_days".into()), vec![final_review_days.to_string()]),
            Tag::Generic(TagKind::Custom("timestamp".into()), vec![Utc::now().timestamp().to_string()]),
        ];

        // Add optional tags
        if let Some(hash) = commit_hash {
            tags.push(Tag::Generic(TagKind::Custom("commit_hash".into()), vec![hash.to_string()]));
        }

        if let Some(pr) = pr_number {
            tags.push(Tag::Generic(TagKind::Custom("pr_number".into()), vec![pr.to_string()]));
        }

        // Add zap address if configured
        if let Some(zap) = &self.zap_address {
            tags.push(Tag::Generic(TagKind::Custom("zap".into()), vec![zap.clone()]));
        }

        // Add Bitcoin Commons tags
        tags.push(Tag::Generic(TagKind::Custom("t".into()), vec!["btc-commons".to_string(), "governance".to_string()]));

        let event = EventBuilder::new(
            Kind::Custom(30078),
            content,
            tags,
        ).to_event(&self.client.keys)
            .map_err(|e| anyhow!("Failed to create Nostr event: {}", e))?;

        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nostr::events::EconomicVetoStatus;

    // Note: Full test requires valid Nostr keys and relay connections
    // This is a placeholder - actual tests should use mock clients
}

