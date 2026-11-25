pub mod audit;
pub mod authorization;
pub mod backup;
pub mod build;
pub mod config;
pub mod crypto;
pub mod database;
pub mod enforcement;
pub mod error;
pub mod fork;
pub mod github;
pub mod governance;
pub mod governance_review;
pub mod node_registry;
pub mod nostr;
pub mod reporting;
pub mod resilience;
pub mod services;
pub mod validation;
pub mod webhooks;

#[cfg(feature = "opentimestamps")]
pub mod ots;

pub use error::GovernanceError;

/// Install the process-level rustls CryptoProvider (`ring`).
///
/// rustls 0.23 panics when both the `ring` and `aws-lc-rs` features are enabled
/// unless a provider is installed explicitly. This crate's TLS stacks (sqlx,
/// reqwest, octocrab, iroh) enable `ring`; `aws-lc-rs` arrives via rustls
/// default features on transitive crates such as blvm-sdk.
pub fn install_rustls_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
