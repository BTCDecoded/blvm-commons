pub mod client;
pub mod cross_layer_status;
pub mod file_operations;
pub mod types;
pub mod webhooks;

/// Install ring as the rustls process-wide crypto provider.
///
/// rustls 0.23 only picks a provider automatically when exactly one backend is
/// compiled in, and this dependency graph enables both ring and aws-lc-rs.
/// octocrab builds its TLS config from the process default, so call this
/// before every `Octocrab::builder()`. Installing twice is a no-op.
///
/// Binaries call this first thing in `main` so any TLS user (octocrab,
/// reqwest, nostr-sdk relays, sqlx) finds a provider in a fresh process.
pub fn ensure_rustls_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
