//! Governance Module
//!
//! Handles governance contribution tracking, weight calculation, and voting.

pub mod aggregator;
pub mod contributions;
pub mod fee_forwarding;
pub mod time_lock;
pub mod vote_aggregator;
pub mod weight_calculator;

pub use aggregator::{ContributionAggregator, ContributorAggregates};
pub use contributions::{ContributionTracker, ContributorTotal};
pub use fee_forwarding::{FeeForwardingContribution, FeeForwardingTracker};
pub use vote_aggregator::{ProposalVoteResult, VoteAggregator};
pub use weight_calculator::WeightCalculator;
