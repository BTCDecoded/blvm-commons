//! Governance Fork Capability
//!
//! Handles governance ruleset export, versioning, adoption tracking, and fork support

pub mod adoption;
pub mod dashboard;
pub mod detection;
pub mod executor;
pub mod export;
pub mod types;
pub mod verification;
pub mod versioning;

pub use adoption::AdoptionTracker;
pub use dashboard::AdoptionDashboard;
pub use detection::{ForkAction, ForkDetectionEvent, ForkDetector, ForkTriggerType};
pub use executor::ForkExecutor;
pub use export::GovernanceExporter;
pub use types::*;
pub use verification::verify_fork_decision_signature;
pub use versioning::RulesetVersioning;
