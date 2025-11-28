//! Nostr Integration Module
//!
//! This module provides real-time transparency for governance operations
//! by publishing status updates to the Nostr protocol.

pub mod bot_manager;
pub mod client;
pub mod events;
pub mod governance_publisher;
pub mod helpers;
pub mod publisher;
pub mod zap_tracker;
pub mod zap_voting;

pub use bot_manager::NostrBotManager;
pub use client::{NostrClient, ZapEvent};
pub use events::{
    CombinedRequirement, EconomicVetoStatus, GovernanceActionEvent, GovernanceStatus, Hashes,
    KeyholderAnnouncement, KeyholderSignature, LayerRequirement, NodeStatusReport, ServerHealth,
    TierRequirement,
};
pub use governance_publisher::GovernanceActionPublisher;
pub use helpers::{
    create_keyholder_announcement_event, publish_merge_action, publish_review_period_notification,
};
pub use publisher::StatusPublisher;
pub use zap_tracker::{ZapContribution, ZapTracker};
pub use zap_voting::{VoteTotals, VoteType, ZapVote, ZapVotingProcessor};
