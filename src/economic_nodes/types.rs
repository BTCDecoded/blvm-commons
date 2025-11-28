//! Economic Node Types and Data Structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Types of economic nodes that can participate in governance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    MiningPool,
    Exchange,
    Custodian,
    PaymentProcessor,
    MajorHolder,
    CommonsContributor, // Nodes contributing through merge mining, fee forwarding, zaps, treasury, marketplace, services
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::MiningPool => "mining_pool",
            NodeType::Exchange => "exchange",
            NodeType::Custodian => "custodian",
            NodeType::PaymentProcessor => "payment_processor",
            NodeType::MajorHolder => "major_holder",
            NodeType::CommonsContributor => "commons_contributor",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mining_pool" => Some(NodeType::MiningPool),
            "exchange" => Some(NodeType::Exchange),
            "custodian" => Some(NodeType::Custodian),
            "payment_processor" => Some(NodeType::PaymentProcessor),
            "major_holder" => Some(NodeType::MajorHolder),
            "commons_contributor" => Some(NodeType::CommonsContributor),
            _ => None,
        }
    }

    /// Get minimum qualification thresholds for this node type
    /// Note: For CommonsContributor, thresholds are loaded from config (maintainer-configurable)
    pub fn qualification_thresholds(&self) -> QualificationThresholds {
        match self {
            NodeType::MiningPool => QualificationThresholds {
                minimum_hashpower_percent: Some(1.0),
                minimum_holdings_btc: None,
                minimum_volume_usd: None,
                minimum_transactions_monthly: None,
                // Commons contributor thresholds (None = use config)
                minimum_merge_mining_btc: None,
                minimum_fee_forwarding_btc: None,
                minimum_zap_quantity_btc: None,
                minimum_marketplace_sales_btc: None,
            },
            NodeType::Exchange => QualificationThresholds {
                minimum_hashpower_percent: None,
                minimum_holdings_btc: Some(10_000),
                minimum_volume_usd: Some(100_000_000), // $100M daily
                minimum_transactions_monthly: None,
                minimum_merge_mining_btc: None,
                minimum_fee_forwarding_btc: None,
                minimum_zap_quantity_btc: None,
                minimum_marketplace_sales_btc: None,
            },
            NodeType::Custodian => QualificationThresholds {
                minimum_hashpower_percent: None,
                minimum_holdings_btc: Some(10_000),
                minimum_volume_usd: None,
                minimum_transactions_monthly: None,
                minimum_merge_mining_btc: None,
                minimum_fee_forwarding_btc: None,
                minimum_zap_quantity_btc: None,
                minimum_marketplace_sales_btc: None,
            },
            NodeType::PaymentProcessor => QualificationThresholds {
                minimum_hashpower_percent: None,
                minimum_holdings_btc: None,
                minimum_volume_usd: Some(50_000_000), // $50M monthly
                minimum_transactions_monthly: None,
                minimum_merge_mining_btc: None,
                minimum_fee_forwarding_btc: None,
                minimum_zap_quantity_btc: None,
                minimum_marketplace_sales_btc: None,
            },
            NodeType::MajorHolder => QualificationThresholds {
                minimum_hashpower_percent: None,
                minimum_holdings_btc: Some(5_000),
                minimum_volume_usd: None,
                minimum_transactions_monthly: None,
                minimum_merge_mining_btc: None,
                minimum_fee_forwarding_btc: None,
                minimum_zap_quantity_btc: None,
                minimum_marketplace_sales_btc: None,
            },
            NodeType::CommonsContributor => {
                // Thresholds loaded from config (maintainer-configurable)
                // Default values provided here, but should be overridden by config
                QualificationThresholds {
                    minimum_hashpower_percent: None,
                    minimum_holdings_btc: None,
                    minimum_volume_usd: None,
                    minimum_transactions_monthly: None,
                    minimum_merge_mining_btc: Some(0.01), // Default: 0.01 BTC
                    minimum_fee_forwarding_btc: Some(0.1), // Default: 0.1 BTC
                    minimum_zap_quantity_btc: Some(0.01), // Default: 0.01 BTC
                    minimum_marketplace_sales_btc: Some(0.01), // Default: 0.01 BTC
                }
            },
        }
    }
}

/// Qualification thresholds for different node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualificationThresholds {
    // Traditional economic node thresholds
    pub minimum_hashpower_percent: Option<f64>,
    pub minimum_holdings_btc: Option<u64>,
    pub minimum_volume_usd: Option<u64>,
    pub minimum_transactions_monthly: Option<u64>,
    
    // Commons contributor thresholds (maintainer-configurable)
    // All thresholds are in BTC - no USD conversion needed
    pub minimum_merge_mining_btc: Option<f64>, // Minimum merge mining contribution (90-day period)
    pub minimum_fee_forwarding_btc: Option<f64>, // Minimum fee forwarding contribution (90-day period)
    pub minimum_zap_quantity_btc: Option<f64>, // Minimum zap contributions (90-day period)
    pub minimum_marketplace_sales_btc: Option<f64>, // Minimum module marketplace sales (90-day period, BIP70 payments)
}

/// Economic node registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicNode {
    pub id: Option<i32>,
    pub node_type: NodeType,
    pub entity_name: String,
    pub public_key: String,
    pub qualification_data: serde_json::Value,
    pub weight: f64,
    pub status: NodeStatus,
    pub registered_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub last_verified_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub notes: String,
}

/// Node status in the registry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Active,
    Suspended,
    Removed,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeStatus::Pending => "pending",
            NodeStatus::Active => "active",
            NodeStatus::Suspended => "suspended",
            NodeStatus::Removed => "removed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(NodeStatus::Pending),
            "active" => Some(NodeStatus::Active),
            "suspended" => Some(NodeStatus::Suspended),
            "removed" => Some(NodeStatus::Removed),
            _ => None,
        }
    }
}

/// Veto signal from an economic node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VetoSignal {
    pub id: Option<i32>,
    pub pr_id: i32,
    pub node_id: i32,
    pub signal_type: SignalType,
    pub weight: f64,
    pub signature: String,
    pub rationale: String,
    pub timestamp: DateTime<Utc>,
    pub verified: bool,
}

/// Type of signal from economic node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    Veto,
    Support,
    Abstain,
}

impl SignalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalType::Veto => "veto",
            SignalType::Support => "support",
            SignalType::Abstain => "abstain",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "veto" => Some(SignalType::Veto),
            "support" => Some(SignalType::Support),
            "abstain" => Some(SignalType::Abstain),
            _ => None,
        }
    }
}

/// Veto threshold calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VetoThreshold {
    pub mining_veto_percent: f64,
    pub economic_veto_percent: f64,
    pub threshold_met: bool,
    pub veto_active: bool,
    /// Sequential veto mechanism: review period information
    pub review_period_start: Option<DateTime<Utc>>,
    pub review_period_days: u32,
    pub review_period_ends_at: Option<DateTime<Utc>>,
    /// Maintainer override capability
    pub maintainer_override: bool,
    pub override_timestamp: Option<DateTime<Utc>>,
    pub override_by: Option<String>,
    /// Resolution path: 'consensus', 'override', 'dissolution', or None if still in review
    pub resolution_path: Option<String>,
}

/// Economic node qualification proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualificationProof {
    pub node_type: NodeType,
    pub hashpower_proof: Option<HashpowerProof>,
    pub holdings_proof: Option<HoldingsProof>,
    pub volume_proof: Option<VolumeProof>,
    pub contact_info: ContactInfo,
    // Commons contributor proofs (for CommonsContributor node type)
    // Only includes verifiable contribution types (all in BTC)
    pub commons_contributor_proof: Option<CommonsContributorProof>,
}

/// Commons contributor qualification proof
/// Only includes verifiable contribution types (all in BTC, no USD conversion needed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonsContributorProof {
    /// Merge mining contribution proof (on-chain, BTC)
    pub merge_mining_proof: Option<MergeMiningProof>,
    /// Fee forwarding contribution proof (on-chain, BTC)
    pub fee_forwarding_proof: Option<FeeForwardingProof>,
    /// Zap contribution proof (Lightning, BTC)
    pub zap_proof: Option<ZapProof>,
    /// Marketplace sales proof (BIP70 payments, BTC)
    pub marketplace_sales_proof: Option<MarketplaceSalesProof>,
}

/// Merge mining contribution proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeMiningProof {
    pub total_revenue_btc: f64,
    pub period_days: u32,
    pub blocks_mined: Vec<MergeMiningBlockProof>,
    pub contributor_id: String,
}

/// Merge mining block proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeMiningBlockProof {
    pub block_hash: String,
    pub chain_id: String,
    pub commons_fee_amount: u64, // Satoshis
    pub coinbase_signature: String,
}

/// Fee forwarding contribution proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeForwardingProof {
    pub total_fees_forwarded_btc: f64,
    pub period_days: u32,
    pub blocks_with_forwarding: Vec<FeeForwardingBlockProof>,
    pub contributor_id: String,
}

/// Fee forwarding block proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeForwardingBlockProof {
    pub block_hash: String,
    pub block_height: u32,
    pub forwarded_amount: u64, // Satoshis
    pub commons_address: String,
    pub tx_hash: String,
}

/// Zap contribution proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZapProof {
    pub total_zaps_btc: f64,
    pub period_days: u32,
    pub zap_events: Vec<ZapEventProof>,
    pub contributor_id: String, // Nostr pubkey
}

/// Zap event proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZapEventProof {
    pub nostr_event_id: String,
    pub zap_amount: u64, // Satoshis
    pub payment_hash: String,
    pub timestamp: i64,
}

/// Marketplace sales proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSalesProof {
    pub total_sales_btc: f64, // BIP70 payments are in BTC
    pub period_days: u32,
    pub module_payments: Vec<ModulePaymentProof>,
    pub contributor_id: String,
}

/// Module payment proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePaymentProof {
    pub payment_id: String,
    pub module_id: String,
    pub amount_btc: f64, // BIP70 payments are in BTC (satoshis)
    pub payment_hash: String, // BIP70 payment hash
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashpowerProof {
    pub blocks_mined: Vec<String>, // Block hashes
    pub time_period_days: u32,
    pub total_network_blocks: u32,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingsProof {
    pub addresses: Vec<String>, // Bitcoin addresses
    pub total_btc: f64,
    pub signature_challenge: String, // Signature proving control
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeProof {
    pub daily_volume_usd: f64,
    pub monthly_volume_usd: f64,
    pub data_source: String, // External data provider
    pub verification_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub entity_name: String,
    pub contact_email: String,
    pub website: Option<String>,
    pub github_username: Option<String>,
}
