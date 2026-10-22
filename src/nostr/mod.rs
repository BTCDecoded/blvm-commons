//! Nostr Integration Module
//!
//! Status updates can be sent to Nostr relays. The default interval is 3600 seconds.
//! That timer is not a live feed of every governance action.

pub mod bot_manager;
pub mod client;
pub mod events;
pub mod governance_publisher;
pub mod helpers;
pub mod publisher;
pub mod zap_tracker;
pub mod zap_voting;

pub use client::{NostrClient, ZapEvent};
pub use events::{
    CombinedRequirement, KeyholderAnnouncement, KeyholderSignature, LayerRequirement,
    TierRequirement,
};
pub use governance_publisher::GovernanceActionPublisher;
pub use helpers::publish_merge_action;
pub use publisher::StatusPublisher;
pub use zap_tracker::ZapTracker;
pub use zap_voting::{VoteType, ZapVotingProcessor};
