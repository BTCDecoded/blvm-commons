//! Environment variable helpers for governance review
//!
//! Provides optional environment variable support with autodetection

use std::env;

/// Get GitHub token from environment
/// 
/// Priority:
/// 1. GITHUB_TOKEN (for workflows/CI)
/// 2. GITHUB_INSTALLATION_TOKEN (for app installations)
/// 3. None (optional - operations can proceed without it)
pub fn get_github_token() -> Option<String> {
    // Check GITHUB_TOKEN first (standard for GitHub Actions)
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }

    // Check GITHUB_INSTALLATION_TOKEN (for app-based auth)
    if let Ok(token) = env::var("GITHUB_INSTALLATION_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }

    None
}

/// Get governance repo owner/name from environment
pub fn get_governance_repo() -> Option<(String, String)> {
    // Check GOVERNANCE_REPO (format: "owner/repo")
    if let Ok(repo) = env::var("GOVERNANCE_REPO") {
        if let Some((owner, name)) = repo.split_once('/') {
            return Some((owner.trim().to_string(), name.trim().to_string()));
        }
    }

    // Check separate env vars
    let owner = env::var("GOVERNANCE_REPO_OWNER").ok();
    let name = env::var("GOVERNANCE_REPO_NAME").ok();

    if let (Some(owner), Some(name)) = (owner, name) {
        if !owner.is_empty() && !name.is_empty() {
            return Some((owner, name));
        }
    }

    None
}

/// Check if we're running in GitHub Actions
pub fn is_github_actions() -> bool {
    env::var("GITHUB_ACTIONS").is_ok()
}

/// Get database URL from environment
pub fn get_database_url() -> String {
    env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://governance.db".to_string())
}

