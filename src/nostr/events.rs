//! Nostr Event Types for Governance Status
//!
//! Defines the structure of governance status events published to Nostr.
//! Includes governance actions, keyholder announcements, and node telemetry.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Governance status event published to Nostr
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStatus {
    pub server_id: String,
    pub timestamp: DateTime<Utc>,
    pub hashes: Hashes,
    pub health: ServerHealth,
    pub next_ots_anchor: DateTime<Utc>,
    pub audit_log_head: Option<String>,
    pub audit_log_length: Option<u64>,
}

/// File hashes for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashes {
    pub binary: String, // sha256:...
    pub config: String, // sha256:...
}

/// Server health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealth {
    pub uptime_hours: u64,
    pub last_merge_pr: Option<i32>,
    pub last_merge: Option<DateTime<Utc>>,
    pub merges_today: i64,
    pub relay_status: std::collections::HashMap<String, bool>,
}

impl GovernanceStatus {
    /// Create a new governance status event
    pub fn new(
        server_id: String,
        binary_hash: String,
        config_hash: String,
        uptime_hours: u64,
        last_merge_pr: Option<i32>,
        last_merge: Option<DateTime<Utc>>,
        merges_today: i64,
        next_ots_anchor: DateTime<Utc>,
        relay_status: std::collections::HashMap<String, bool>,
        audit_log_head: Option<String>,
        audit_log_length: Option<u64>,
    ) -> Self {
        Self {
            server_id,
            timestamp: Utc::now(),
            hashes: Hashes {
                binary: binary_hash,
                config: config_hash,
            },
            health: ServerHealth {
                uptime_hours,
                last_merge_pr,
                last_merge,
                merges_today,
                relay_status,
            },
            next_ots_anchor,
            audit_log_head,
            audit_log_length,
        }
    }

    /// Serialize to JSON for Nostr event content
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Server {}: {}h uptime, {} merges today, next OTS: {}",
            self.server_id,
            self.health.uptime_hours,
            self.health.merges_today,
            self.next_ots_anchor.format("%Y-%m-%d")
        )
    }
}

/// Governance action event (Kind 30078)
/// Published when governance actions occur (merges, releases, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceActionEvent {
    pub description: String,
    pub pr_url: Option<String>,
    pub layer_requirement: LayerRequirement,
    pub tier_requirement: TierRequirement,
    pub combined_requirement: CombinedRequirement,
    pub signatures: Vec<KeyholderSignature>,
    pub economic_veto_status: EconomicVetoStatus,
    pub review_period_ends: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerRequirement {
    pub layer: u32,
    pub signatures: String, // e.g., "6-of-7"
    pub review_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRequirement {
    pub tier: u32,
    pub signatures: String, // e.g., "3-of-5"
    pub review_days: u32,
    pub economic_veto: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedRequirement {
    pub signatures: String, // e.g., "6-of-7"
    pub review_days: u32,
    pub economic_veto: bool,
    pub source: String, // "layer" or "tier"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyholderSignature {
    pub keyholder: String,      // pubkey
    pub keyholder_type: String, // "maintainer" or "emergency_keyholder"
    pub signature: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EconomicVetoStatus {
    NotRequired,
    Pending,
    Passed,
    Vetoed,
}

impl GovernanceActionEvent {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Keyholder announcement event (Kind 0 - Metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyholderAnnouncement {
    pub name: String,
    pub about: String,
    pub role: String,
    pub governance_pubkey: String,
    pub jurisdiction: Option<String>,
    pub backup_contact: Option<String>,
    pub joined: i64,
    pub layer: Option<u32>,
    pub keyholder_type: String, // "maintainer" or "emergency_keyholder"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zap_address: Option<String>, // Lightning address for donations
}

impl KeyholderAnnouncement {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Node status report event (Kind 30078)
/// Published by nodes for telemetry (opt-in)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatusReport {
    pub node_type: String, // "full" | "archival" | "pruned"
    pub uptime_hours: u64,
    pub sync_status: String, // "synced" | "syncing"
    pub modules_enabled: Vec<String>,
    pub reported_at: i64,
}

impl NodeStatusReport {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_governance_status_creation() {
        let relay_status = HashMap::new();
        let status = GovernanceStatus::new(
            "governance-01".to_string(),
            "sha256:abc123".to_string(),
            "sha256:def456".to_string(),
            24,
            Some(123),
            Some(Utc::now()),
            5,
            Utc::now(),
            relay_status,
            Some("sha256:head123".to_string()),
            Some(1000),
        );

        assert_eq!(status.server_id, "governance-01");
        assert_eq!(status.hashes.binary, "sha256:abc123");
        assert_eq!(status.health.uptime_hours, 24);
        assert_eq!(status.health.merges_today, 5);
    }

    #[test]
    fn test_json_serialization() {
        let relay_status = HashMap::new();
        let status = GovernanceStatus::new(
            "test-server".to_string(),
            "sha256:test".to_string(),
            "sha256:config".to_string(),
            1,
            None,
            None,
            0,
            Utc::now(),
            relay_status,
            None,
            None,
        );

        let json = status.to_json().unwrap();
        assert!(json.contains("test-server"));
        assert!(json.contains("sha256:test"));
    }

    #[test]
    fn test_summary() {
        let relay_status = HashMap::new();
        let status = GovernanceStatus::new(
            "governance-01".to_string(),
            "sha256:abc".to_string(),
            "sha256:def".to_string(),
            48,
            Some(456),
            Some(Utc::now()),
            3,
            Utc::now(),
            relay_status,
            None,
            None,
        );

        let summary = status.summary();
        assert!(summary.contains("governance-01"));
        assert!(summary.contains("48h uptime"));
        assert!(summary.contains("3 merges today"));
    }
}
