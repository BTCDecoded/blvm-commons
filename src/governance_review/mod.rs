//! Governance Review System
//!
//! Implements the maintainer governance review policy with:
//! - Graduated sanctions (private warning, public warning, removal)
//! - Time limits (180 days for cases, 90 days for appeals)
//! - Protections (whistleblower, false reports, retaliation)
//! - Conflict resolution/mediation
//! - On-platform only (off-platform activity disregarded)

pub mod appeals;
pub mod case;
pub mod deadline_notifications;
pub mod env;
pub mod github_integration;
pub mod mediation;
pub mod models;
pub mod protections;
pub mod removal;
pub mod response;
pub mod sanctions;
pub mod time_limits;

pub use appeals::AppealManager;
pub use case::GovernanceReviewCaseManager;
pub use deadline_notifications::DeadlineNotificationManager;
#[allow(unused_imports)] // re-exported for bins and integration tests
pub use env::{get_database_url, get_github_token, get_governance_repo, is_github_actions};
pub use github_integration::GovernanceReviewGitHubIntegration;
pub use mediation::MediationManager;
pub use removal::RemovalManager;
pub use sanctions::SanctionManager;
pub use time_limits::TimeLimitManager;
