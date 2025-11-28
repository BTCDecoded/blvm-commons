//! Governance Review System
//!
//! Implements the maintainer governance review policy with:
//! - Graduated sanctions (private warning, public warning, removal)
//! - Time limits (180 days for cases, 90 days for appeals)
//! - Protections (whistleblower, false reports, retaliation)
//! - Conflict resolution/mediation
//! - On-platform only (off-platform activity disregarded)

pub mod case;
pub mod models;
pub mod sanctions;
pub mod time_limits;
pub mod protections;
pub mod removal;
pub mod appeals;
pub mod mediation;
pub mod github_integration;
pub mod env;
pub mod deadline_notifications;
pub mod response;

pub use case::GovernanceReviewCaseManager;
pub use models::*;
pub use sanctions::SanctionManager;
pub use time_limits::TimeLimitManager;
pub use protections::ProtectionManager;
pub use removal::RemovalManager;
pub use appeals::AppealManager;
pub use mediation::MediationManager;
pub use github_integration::GovernanceReviewGitHubIntegration;
pub use deadline_notifications::DeadlineNotificationManager;
pub use response::ResponseManager;
pub use env::{get_github_token, get_governance_repo, get_database_url, is_github_actions};

