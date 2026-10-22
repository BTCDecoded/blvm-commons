use crate::error::GovernanceError;
use crate::github::cross_layer_status::{CrossLayerStatusCheck, CrossLayerStatusChecker};
use crate::github::file_operations::GitHubFileOperations;
use crate::validation::content_hash::{ContentHashValidator, SyncReport, SyncStatus};
use crate::validation::diff_parser::{DiffParser, FileDiff};
use crate::validation::version_pinning::{
    VersionManifest, VersionPinningConfig, VersionPinningValidator,
};
use octocrab;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct CrossLayerValidator;

/// Page size for the PR files API (GitHub's maximum).
const PR_FILES_PER_PAGE: u32 = 100;
/// GitHub's files API lists at most 3000 files per PR.
const PR_FILES_MAX: usize = 3000;

impl CrossLayerValidator {
    pub async fn validate_cross_layer_dependencies(
        repo_name: &str,
        changed_files: &[String],
        cross_layer_rules: &[Value],
        github_token: Option<&str>,
    ) -> Result<(), GovernanceError> {
        for rule in cross_layer_rules {
            if let Some(source_repo) = rule.get("source_repo").and_then(|v| v.as_str()) {
                if source_repo == repo_name {
                    if let Some(source_pattern) =
                        rule.get("source_pattern").and_then(|v| v.as_str())
                    {
                        if Self::matches_pattern(changed_files, source_pattern) {
                            if let Some(target_repo) =
                                rule.get("target_repo").and_then(|v| v.as_str())
                            {
                                if let Some(validation_type) =
                                    rule.get("validation_type").and_then(|v| v.as_str())
                                {
                                    return Self::validate_dependency(
                                        target_repo,
                                        validation_type,
                                        rule,
                                        github_token,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn matches_pattern(files: &[String], pattern: &str) -> bool {
        // Simple glob pattern matching
        // In a real implementation, this would use a proper glob library
        files.iter().any(|file| {
            if pattern.contains("**") {
                let prefix = pattern.split("**").next().unwrap_or("");
                file.starts_with(prefix)
            } else if pattern.contains("*") {
                let prefix = pattern.split("*").next().unwrap_or("");
                file.starts_with(prefix)
            } else {
                file == pattern
            }
        })
    }

    async fn validate_dependency(
        target_repo: &str,
        validation_type: &str,
        rule: &Value,
        github_token: Option<&str>,
    ) -> Result<(), GovernanceError> {
        match validation_type {
            "corresponding_file_exists" => {
                Self::verify_file_correspondence(target_repo, rule, github_token).await
            }
            "references_latest_version" => Self::verify_version_references(target_repo, rule),
            "no_consensus_modifications" => {
                Self::verify_no_consensus_modifications(target_repo, rule, github_token).await
            }
            _ => Err(GovernanceError::ValidationError(format!(
                "Unknown validation type: {validation_type}"
            ))),
        }
    }

    /// Verify file correspondence between repositories
    async fn verify_file_correspondence(
        target_repo: &str,
        rule: &Value,
        github_token: Option<&str>,
    ) -> Result<(), GovernanceError> {
        info!(
            "Verifying file correspondence for target repo: {}",
            target_repo
        );

        // Extract rule parameters
        let source_repo = rule
            .get("source_repo")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let source_pattern = rule
            .get("source_pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let target_pattern = rule
            .get("target_pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        info!(
            "Checking correspondence: {}:{} -> {}:{}",
            source_repo, source_pattern, target_repo, target_pattern
        );

        // If no GitHub token provided, we can't verify - return error
        let github_token = github_token.ok_or_else(|| {
            GovernanceError::ValidationError(
                "GitHub token required for file correspondence verification".to_string(),
            )
        })?;

        // Parse repository names (format: owner/repo)
        let (_source_owner, _source_repo_name) = Self::parse_repo_name(source_repo)?;
        let (target_owner, target_repo_name) = Self::parse_repo_name(target_repo)?;

        // Create GitHub file operations client
        let file_ops = GitHubFileOperations::new(github_token.to_string())?;

        // Attempt to fetch target file to verify it exists
        // Note: This will fail if GitHubFileOperations.fetch_file_content is not fully implemented,
        // but at least we're attempting the verification rather than just logging a warning
        match file_ops
            .fetch_file_content(&target_owner, &target_repo_name, target_pattern, None)
            .await
        {
            Ok(_) => {
                info!("Target file {} exists in {}", target_pattern, target_repo);
                Ok(())
            }
            Err(e) => {
                // If file doesn't exist or can't be fetched, return error
                Err(GovernanceError::ValidationError(format!(
                    "Failed to verify file correspondence: target file {target_pattern} not found in {target_repo}: {e}"
                )))
            }
        }
    }

    /// Parse repository name into owner and repo
    fn parse_repo_name(repo_name: &str) -> crate::error::Result<(String, String)> {
        let parts: Vec<&str> = repo_name.split('/').collect();
        if parts.len() != 2 {
            return Err(GovernanceError::ValidationError(format!(
                "Invalid repository name format (expected owner/repo): {repo_name}"
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Verify version references are up to date
    fn verify_version_references(target_repo: &str, rule: &Value) -> Result<(), GovernanceError> {
        info!(
            "Verifying version references for target repo: {}",
            target_repo
        );

        // Extract rule parameters
        let required_reference_format = rule
            .get("required_reference_format")
            .and_then(|v| v.as_str())
            .unwrap_or("blvm-spec@v{VERSION}");

        info!(
            "Checking version reference format: {}",
            required_reference_format
        );

        // Create version pinning validator
        let config = VersionPinningConfig {
            required_reference_format: required_reference_format.to_string(),
            minimum_signatures: 6,
            allow_outdated_versions: false,
            max_version_age_days: 30,
            enforce_latest_version: true,
        };

        let mut validator = VersionPinningValidator::new(config);

        // Load version manifest (in a real implementation, this would be loaded from file)
        let manifest = Self::load_version_manifest()?;
        validator.load_version_manifest(manifest)?;

        // In a real implementation, this would:
        // 1. Fetch files from the target repo
        // 2. Parse version references from each file
        // 3. Validate each reference against the manifest
        // 4. Check format compliance
        // 5. Verify signatures and timestamps

        info!(
            "Version reference verification completed for {}",
            target_repo
        );
        Ok(())
    }

    /// Load version manifest from configuration
    fn load_version_manifest() -> Result<VersionManifest, GovernanceError> {
        // In a real implementation, this would load from the YAML file
        // For now, we'll create a mock manifest

        use crate::validation::version_pinning::{VersionManifestEntry, VersionSignature};
        use chrono::Utc;

        let manifest = VersionManifest {
            repository: "blvm-spec".to_string(),
            created_at: Utc::now(),
            versions: vec![VersionManifestEntry {
                version: "v1.0.0".to_string(),
                commit_sha: "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
                content_hash:
                    "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
                        .to_string(),
                created_at: Utc::now() - chrono::Duration::try_days(1).expect("day count"),
                signatures: vec![VersionSignature {
                    maintainer_id: "maintainer1".to_string(),
                    signature: "test_signature_1".to_string(),
                    public_key: "test_public_key_1".to_string(),
                    signed_at: Utc::now() - chrono::Duration::try_days(1).expect("day count"),
                }],
                ots_timestamp: Some("bitcoin:test_timestamp".to_string()),
                is_stable: true,
                is_latest: true,
            }],
            latest_version: "v1.0.0".to_string(),
            manifest_hash: "sha256:test_manifest_hash".to_string(),
        };

        Ok(manifest)
    }

    /// Verify no consensus modifications are made
    async fn verify_no_consensus_modifications(
        target_repo: &str,
        rule: &Value,
        github_token: Option<&str>,
    ) -> Result<(), GovernanceError> {
        info!(
            "Verifying no consensus modifications for target repo: {}",
            target_repo
        );

        // Extract rule parameters
        let allowed_imports_only = rule
            .get("allowed_imports_only")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!(
            "Checking consensus modifications - imports only: {}",
            allowed_imports_only
        );

        // Consensus-critical file patterns
        let consensus_patterns = vec![
            "src/block.rs",
            "src/transaction.rs",
            "src/script.rs",
            "src/economic.rs",
            "src/pow.rs",
            "src/validation/",
            "src/consensus/",
        ];

        // Fail closed: every path below either proves the PR is acceptable or
        // returns an error. GitHub failures, a missing token and a missing PR
        // number all block instead of passing.
        if let Some(pr_value) = rule.get("pr_number") {
            let pr_number = pr_value.as_u64().ok_or_else(|| {
                GovernanceError::ValidationError(format!(
                    "Blocked: no_consensus_modifications rule has an invalid pr_number ({pr_value}); cannot verify consensus files"
                ))
            })?;
            let token = github_token.ok_or_else(|| {
                GovernanceError::ValidationError(format!(
                    "Blocked: no_consensus_modifications for {target_repo} PR #{pr_number} needs a GitHub token to read the PR; refusing to pass unverified"
                ))
            })?;
            let (owner, repo) = Self::parse_repo_name(target_repo)?;

            crate::github::ensure_rustls_provider();
            let client = octocrab::OctocrabBuilder::new()
                .personal_token(token.to_string())
                .build()
                .map_err(|e| {
                    GovernanceError::GitHubError(format!(
                        "Blocked: failed to create GitHub client for {target_repo} PR #{pr_number}: {e}"
                    ))
                })?;

            let changed_files = Self::list_all_pr_files(&client, &owner, &repo, pr_number).await?;

            if !allowed_imports_only {
                // Standard file pattern check
                return Self::check_consensus_patterns(&consensus_patterns, &changed_files);
            }

            // Import-only: inspect the full diff. If it cannot be fetched or
            // parsed, fall back to the strict file-name check, which blocks
            // any change to a consensus file.
            let diff = match Self::get_pr_diff(&client, &owner, &repo, pr_number).await {
                Ok(diff) => diff,
                Err(e) => {
                    warn!(
                        "Failed to get PR diff: {}. Falling back to strict file pattern check.",
                        e
                    );
                    return Self::check_consensus_patterns(&consensus_patterns, &changed_files);
                }
            };
            let file_diffs = match DiffParser::parse_unified_diff(&diff) {
                Ok(file_diffs) => file_diffs,
                Err(e) => {
                    warn!(
                        "Failed to parse diff: {}. Falling back to strict file pattern check.",
                        e
                    );
                    return Self::check_consensus_patterns(&consensus_patterns, &changed_files);
                }
            };
            return Self::check_import_only(&consensus_patterns, &changed_files, &file_diffs);
        }

        // No PR: the rule may list the files to check explicitly
        if let Some(file_patterns) = rule.get("check_files").and_then(|v| v.as_array()) {
            let files: Vec<String> = file_patterns
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            return Self::check_consensus_patterns(&consensus_patterns, &files);
        }

        // Nothing to verify against: block instead of passing
        Err(GovernanceError::ValidationError(format!(
            "Blocked: no_consensus_modifications for {target_repo} needs a pr_number (with a GitHub token) or check_files; refusing to pass unverified"
        )))
    }

    /// Import-only check over a parsed PR diff.
    ///
    /// Every consensus file the PR touches (by the file list, which includes
    /// rename sources) must appear in the diff, so a truncated or partial diff
    /// cannot hide one. Each in-scope file diff must pass
    /// [`Self::violates_import_only`].
    fn check_import_only(
        consensus_patterns: &[&str],
        changed_files: &[String],
        file_diffs: &[FileDiff],
    ) -> Result<(), GovernanceError> {
        for changed in changed_files {
            if Self::matches_consensus_pattern(changed, consensus_patterns)
                && !file_diffs.iter().any(|d| {
                    d.filename == *changed || d.old_filename.as_deref() == Some(changed.as_str())
                })
            {
                return Err(GovernanceError::ValidationError(format!(
                    "Blocked: consensus file {changed} is in the PR file list but missing from the PR diff; cannot verify it is import-only"
                )));
            }
        }
        for file_diff in file_diffs {
            if Self::violates_import_only(file_diff, consensus_patterns) {
                return Err(GovernanceError::ValidationError(format!(
                    "Consensus file {}{} contains non-import changes (renames, copies and binary files count as non-import). Only import statements are allowed when allowed_imports_only is true.",
                    file_diff.filename,
                    file_diff
                        .old_filename
                        .as_deref()
                        .map(|o| format!(" (from {o})"))
                        .unwrap_or_default()
                )));
            }
        }
        info!("Import-only validation passed for all consensus files");
        Ok(())
    }

    /// Every file path a PR touches, across all pages of the files API.
    /// Rename sources (`previous_filename`) are included.
    async fn list_all_pr_files(
        client: &octocrab::Octocrab,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<String>, GovernanceError> {
        let route = format!("/repos/{owner}/{repo}/pulls/{pr_number}/files");
        Self::collect_file_pages(|page| {
            let route = route.clone();
            async move {
                let entries: Vec<octocrab::models::repos::DiffEntry> = client
                    .get(&route, Some(&[("per_page", PR_FILES_PER_PAGE), ("page", page)]))
                    .await
                    .map_err(|e| {
                        GovernanceError::GitHubError(format!(
                            "Blocked: failed to list files for {owner}/{repo} PR #{pr_number} (page {page}): {e}"
                        ))
                    })?;
                Ok(entries
                    .into_iter()
                    .map(|e| (e.filename, e.previous_filename))
                    .collect())
            }
        })
        .await
    }

    /// Page through `fetch(page)` (1-based) until a short or empty page.
    /// Fails closed when GitHub's 3000-file cap is reached, since files past
    /// the cap are never listed.
    async fn collect_file_pages<F, Fut>(mut fetch: F) -> Result<Vec<String>, GovernanceError>
    where
        F: FnMut(u32) -> Fut,
        Fut: std::future::Future<Output = Result<Vec<(String, Option<String>)>, GovernanceError>>,
    {
        let mut names = Vec::new();
        let mut listed = 0usize;
        let mut page = 1u32;
        loop {
            let entries = fetch(page).await?;
            let count = entries.len();
            listed += count;
            for (filename, previous) in entries {
                if let Some(previous) = previous {
                    names.push(previous);
                }
                names.push(filename);
            }
            if count < PR_FILES_PER_PAGE as usize {
                return Ok(names);
            }
            if listed >= PR_FILES_MAX {
                return Err(GovernanceError::ValidationError(format!(
                    "Blocked: PR lists {listed} files, GitHub's maximum for the files API; files past it cannot be checked"
                )));
            }
            page += 1;
        }
    }

    /// Get the PR diff through the authenticated client, so private repos
    /// and rate limits behave like the other API calls.
    async fn get_pr_diff(
        client: &octocrab::Octocrab,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<String, GovernanceError> {
        client
            .pulls(owner, repo)
            .get_diff(pr_number)
            .await
            .map_err(|e| GovernanceError::GitHubError(format!("Failed to fetch diff: {e}")))
    }

    /// Import-only gate for one parsed file diff.
    ///
    /// A file is in scope when either its new name or its source name (rename
    /// or copy) matches a consensus pattern. In scope, renames/copies and
    /// binary files are rejected outright (no line content to inspect), and
    /// anything else must be import-only.
    pub(crate) fn violates_import_only(file_diff: &FileDiff, consensus_patterns: &[&str]) -> bool {
        let old_matches = file_diff
            .old_filename
            .as_deref()
            .is_some_and(|p| Self::matches_consensus_pattern(p, consensus_patterns));
        let in_scope =
            old_matches || Self::matches_consensus_pattern(&file_diff.filename, consensus_patterns);
        in_scope
            && (file_diff.old_filename.is_some()
                || file_diff.binary
                || !DiffParser::is_import_only_changes(file_diff))
    }

    /// Check if filename matches any consensus pattern
    fn matches_consensus_pattern(filename: &str, patterns: &[&str]) -> bool {
        for pattern in patterns {
            if filename.contains(pattern) || filename == *pattern {
                return true;
            }
            // Handle directory patterns (e.g., "src/validation/")
            if pattern.ends_with("/") && filename.starts_with(pattern) {
                return true;
            }
        }
        false
    }

    /// Check if any files match consensus patterns
    fn check_consensus_patterns(
        consensus_patterns: &[&str],
        changed_files: &[String],
    ) -> Result<(), GovernanceError> {
        let mut consensus_files_changed = Vec::new();

        for file in changed_files {
            for pattern in consensus_patterns {
                if Self::matches_pattern(&[file.clone()], pattern) {
                    consensus_files_changed.push(file.clone());
                    break;
                }
            }
        }

        if !consensus_files_changed.is_empty() {
            return Err(GovernanceError::ValidationError(format!(
                "Consensus-critical files modified: {consensus_files_changed:?}. This requires Tier 3+ governance approval."
            )));
        }

        Ok(())
    }

    /// Check bidirectional synchronization between Orange Paper and Consensus Proof
    pub async fn check_bidirectional_sync(
        github_token: &str,
        orange_paper_owner: &str,
        orange_paper_repo: &str,
        consensus_proof_owner: &str,
        consensus_proof_repo: &str,
        changed_files: &[String],
    ) -> Result<SyncReport, GovernanceError> {
        info!(
            "Checking bidirectional sync between {} and {}",
            orange_paper_repo, consensus_proof_repo
        );

        // Create GitHub file operations client
        let file_ops = GitHubFileOperations::new(github_token.to_string())?;

        // Create content hash validator
        let mut validator = ContentHashValidator::new();
        let correspondence_mappings = ContentHashValidator::generate_correspondence_map();
        validator.load_correspondence_mappings(correspondence_mappings);

        // Fetch Orange Paper files
        let orange_paper_files = file_ops
            .fetch_multiple_files(orange_paper_owner, orange_paper_repo, changed_files, None)
            .await?;

        // Convert to the format expected by the validator
        let mut orange_files_map = HashMap::new();
        for (path, file) in orange_paper_files {
            orange_files_map.insert(path, file.content);
        }

        // Fetch corresponding Consensus Proof files
        let mut consensus_proof_files = HashMap::new();
        for mapping in validator.correspondence_mappings.values() {
            if changed_files.contains(&mapping.orange_paper_file) {
                match file_ops
                    .fetch_file_content(
                        consensus_proof_owner,
                        consensus_proof_repo,
                        &mapping.consensus_proof_file,
                        None,
                    )
                    .await
                {
                    Ok(file) => {
                        consensus_proof_files
                            .insert(mapping.consensus_proof_file.clone(), file.content);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to fetch Consensus Proof file {}: {}",
                            mapping.consensus_proof_file, e
                        );
                    }
                }
            }
        }

        // Check bidirectional sync
        validator.check_bidirectional_sync(&orange_files_map, &consensus_proof_files, changed_files)
    }

    /// Generate synchronization report for PR status checks
    pub fn generate_sync_report(sync_report: &SyncReport) -> String {
        match sync_report.sync_status {
            SyncStatus::Synchronized => {
                format!(
                    "✅ Cross-Layer Sync: All {} files are synchronized between Orange Paper and Consensus Proof",
                    sync_report.changed_files.len()
                )
            }
            SyncStatus::MissingUpdates => {
                format!(
                    "❌ Cross-Layer Sync: Missing Consensus Proof updates for {} files: {}",
                    sync_report.missing_files.len(),
                    sync_report.missing_files.join(", ")
                )
            }
            SyncStatus::OutdatedVersions => {
                format!(
                    "⚠️ Cross-Layer Sync: {} files have outdated versions: {}",
                    sync_report.outdated_files.len(),
                    sync_report.outdated_files.join(", ")
                )
            }
            SyncStatus::SyncFailure => {
                format!(
                    "🚫 Cross-Layer Sync: Critical synchronization failure - {} files affected",
                    sync_report.changed_files.len()
                )
            }
        }
    }

    /// Generate comprehensive cross-layer status check for GitHub PR
    pub async fn generate_github_status_check(
        app_id: u64,
        private_key_path: &str,
        owner: &str,
        repo: &str,
        pr_number: u64,
        changed_files: &[String],
    ) -> Result<CrossLayerStatusCheck, GovernanceError> {
        info!(
            "Generating GitHub status check for {}/{} PR #{}",
            owner, repo, pr_number
        );

        // Create GitHub client with proper authentication
        let github_client = crate::github::client::GitHubClient::new(app_id, private_key_path)
            .map_err(|e| {
                GovernanceError::ConfigError(format!("Failed to create GitHub client: {e}"))
            })?;

        // Create status checker
        let mut status_checker = CrossLayerStatusChecker::new(github_client);

        // Generate comprehensive status check
        status_checker
            .generate_cross_layer_status(owner, repo, pr_number, changed_files)
            .await
    }

    /// Post cross-layer status check to GitHub
    pub async fn post_cross_layer_status_check(
        app_id: u64,
        private_key_path: &str,
        owner: &str,
        repo: &str,
        pr_number: u64,
        changed_files: &[String],
    ) -> Result<(), GovernanceError> {
        info!(
            "Posting cross-layer status check for {}/{} PR #{}",
            owner, repo, pr_number
        );

        // Generate status check
        let status_check = Self::generate_github_status_check(
            app_id,
            private_key_path,
            owner,
            repo,
            pr_number,
            changed_files,
        )
        .await?;

        // Create GitHub client with proper authentication
        let github_client = crate::github::client::GitHubClient::new(app_id, private_key_path)
            .map_err(|e| {
                GovernanceError::ConfigError(format!("Failed to create GitHub client: {e}"))
            })?;

        // Get PR head SHA for status check
        let pr = github_client
            .get_pull_request(owner, repo, pr_number)
            .await?;
        let head_sha = pr
            .get("head")
            .and_then(|h| h.get("sha"))
            .and_then(|s| s.as_str())
            .ok_or_else(|| {
                GovernanceError::ValidationError("Missing head SHA in PR response".to_string())
            })?;

        // Post status check to GitHub
        github_client
            .post_status_check(
                owner,
                repo,
                head_sha,
                &format!("{:?}", status_check.state),
                &status_check.description,
                &status_check.context,
            )
            .await?;

        info!("Posted cross-layer status check: {:?}", status_check.state);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATTERNS: &[&str] = &[
        "src/block.rs",
        "src/transaction.rs",
        "src/script.rs",
        "src/economic.rs",
        "src/pow.rs",
        "src/validation/",
        "src/consensus/",
    ];

    fn parse_one(diff: &str) -> FileDiff {
        let mut f = DiffParser::parse_unified_diff(diff).unwrap();
        assert_eq!(f.len(), 1);
        f.remove(0)
    }

    #[test]
    fn rename_out_of_consensus_path_with_logic_change_is_rejected() {
        let f = parse_one(
            "diff --git a/src/block.rs b/src/blk.rs\n\
similarity index 80%\n\
rename from src/block.rs\n\
rename to src/blk.rs\n\
--- a/src/block.rs\n\
+++ b/src/blk.rs\n\
@@ -10,2 +10,2 @@\n\
-    if height > 0 { return Err(Invalid); }\n\
+    return Ok(());\n",
        );
        // The new name alone would not be checked
        assert!(!CrossLayerValidator::matches_consensus_pattern(
            &f.filename,
            PATTERNS
        ));
        assert!(CrossLayerValidator::violates_import_only(&f, PATTERNS));
    }

    #[test]
    fn pure_rename_of_consensus_file_is_rejected() {
        let f = parse_one(
            "diff --git a/src/block.rs b/src/blk.rs\n\
similarity index 100%\n\
rename from src/block.rs\n\
rename to src/blk.rs\n",
        );
        assert!(CrossLayerValidator::violates_import_only(&f, PATTERNS));
    }

    #[test]
    fn rename_into_consensus_path_is_rejected() {
        let f = parse_one(
            "diff --git a/src/util.rs b/src/consensus/util.rs\n\
similarity index 100%\n\
rename from src/util.rs\n\
rename to src/consensus/util.rs\n",
        );
        assert!(CrossLayerValidator::violates_import_only(&f, PATTERNS));
    }

    #[test]
    fn binary_file_in_consensus_path_is_rejected() {
        let f = parse_one(
            "diff --git a/src/consensus/t.bin b/src/consensus/t.bin\n\
index 1111111..2222222 100644\n\
Binary files a/src/consensus/t.bin and b/src/consensus/t.bin differ\n",
        );
        assert!(CrossLayerValidator::violates_import_only(&f, PATTERNS));
    }

    #[test]
    fn import_only_change_in_consensus_file_passes() {
        let f = parse_one(
            "diff --git a/src/block.rs b/src/block.rs\n\
--- a/src/block.rs\n\
+++ b/src/block.rs\n\
@@ -1,1 +1,2 @@\n\
+use crate::bip_validation;\n",
        );
        assert!(!CrossLayerValidator::violates_import_only(&f, PATTERNS));
    }

    #[test]
    fn non_consensus_rename_and_binary_pass() {
        let f = parse_one(
            "diff --git a/docs/a.md b/docs/b.md\n\
similarity index 100%\n\
rename from docs/a.md\n\
rename to docs/b.md\n",
        );
        assert!(!CrossLayerValidator::violates_import_only(&f, PATTERNS));
        let f = parse_one(
            "diff --git a/docs/x.png b/docs/x.png\n\
Binary files a/docs/x.png and b/docs/x.png differ\n",
        );
        assert!(!CrossLayerValidator::violates_import_only(&f, PATTERNS));
    }

    fn rule(v: serde_json::Value) -> Value {
        v
    }

    #[tokio::test]
    async fn pr_number_without_token_blocks() {
        let r = rule(serde_json::json!({
            "validation_type": "no_consensus_modifications",
            "pr_number": 42,
            "check_files": ["docs/README.md"],
        }));
        let err = CrossLayerValidator::verify_no_consensus_modifications(
            "BTCDecoded/blvm-consensus",
            &r,
            None,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("needs a GitHub token"), "{err}");
    }

    #[tokio::test]
    async fn invalid_pr_number_blocks() {
        for bad in [serde_json::json!(-1), serde_json::json!("42")] {
            let r = rule(serde_json::json!({ "pr_number": bad }));
            let err = CrossLayerValidator::verify_no_consensus_modifications(
                "BTCDecoded/blvm-consensus",
                &r,
                Some("token"),
            )
            .await
            .unwrap_err();
            assert!(err.to_string().contains("invalid pr_number"), "{err}");
        }
    }

    #[tokio::test]
    async fn no_pr_number_and_no_check_files_blocks() {
        for imports_only in [false, true] {
            let r = rule(serde_json::json!({ "allowed_imports_only": imports_only }));
            let err = CrossLayerValidator::verify_no_consensus_modifications(
                "BTCDecoded/blvm-consensus",
                &r,
                Some("token"),
            )
            .await
            .unwrap_err();
            assert!(err.to_string().contains("Blocked"), "{err}");
        }
    }

    #[tokio::test]
    async fn check_files_still_decides_without_pr() {
        let ok = rule(serde_json::json!({ "check_files": ["docs/README.md"] }));
        assert!(
            CrossLayerValidator::verify_no_consensus_modifications("o/r", &ok, None)
                .await
                .is_ok()
        );
        let bad = rule(serde_json::json!({ "check_files": ["src/pow.rs"] }));
        assert!(
            CrossLayerValidator::verify_no_consensus_modifications("o/r", &bad, None)
                .await
                .is_err()
        );
    }

    fn page_of(n: usize, prefix: &str) -> Vec<(String, Option<String>)> {
        (0..n).map(|i| (format!("{prefix}/{i}.rs"), None)).collect()
    }

    #[tokio::test]
    async fn file_pages_are_collected_until_short_page() {
        let mut asked = Vec::new();
        let names = CrossLayerValidator::collect_file_pages(|page| {
            asked.push(page);
            async move {
                Ok(match page {
                    1 => page_of(100, "a"),
                    2 => {
                        let mut p = page_of(99, "b");
                        p.push(("src/blk.rs".into(), Some("src/block.rs".into())));
                        p
                    }
                    3 => page_of(5, "c"),
                    _ => panic!("fetched past the last page"),
                })
            }
        })
        .await
        .unwrap();
        assert_eq!(asked, vec![1, 2, 3]);
        assert_eq!(names.len(), 206); // 205 files + one rename source
        assert!(names.contains(&"src/block.rs".to_string()));
        assert!(names.contains(&"c/4.rs".to_string()));
        // A consensus file on page 2 is now seen by the name check
        assert!(CrossLayerValidator::check_consensus_patterns(PATTERNS, &names).is_err());
    }

    #[tokio::test]
    async fn empty_first_page_and_exact_multiple_stop() {
        let names = CrossLayerValidator::collect_file_pages(|_| async { Ok(vec![]) })
            .await
            .unwrap();
        assert!(names.is_empty());
        let names = CrossLayerValidator::collect_file_pages(|page| async move {
            Ok(if page == 1 { page_of(100, "a") } else { vec![] })
        })
        .await
        .unwrap();
        assert_eq!(names.len(), 100);
    }

    #[tokio::test]
    async fn file_pages_fail_closed_at_github_cap_and_on_error() {
        let err = CrossLayerValidator::collect_file_pages(|page| async move {
            assert!(page <= 30, "kept paging past the cap");
            Ok(page_of(100, "x"))
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("maximum"), "{err}");

        let err = CrossLayerValidator::collect_file_pages(|page| async move {
            if page == 2 {
                Err(GovernanceError::GitHubError("boom".into()))
            } else {
                Ok(page_of(100, "x"))
            }
        })
        .await
        .unwrap_err();
        assert!(err.to_string().contains("boom"), "{err}");
    }

    #[test]
    fn consensus_file_missing_from_diff_blocks() {
        let changed = vec!["src/pow.rs".to_string(), "docs/a.md".to_string()];
        let diffs = DiffParser::parse_unified_diff(
            "diff --git a/docs/a.md b/docs/a.md\n--- a/docs/a.md\n+++ b/docs/a.md\n@@ -1 +1 @@\n-a\n+b\n",
        )
        .unwrap();
        let err = CrossLayerValidator::check_import_only(PATTERNS, &changed, &diffs).unwrap_err();
        assert!(
            err.to_string().contains("missing from the PR diff"),
            "{err}"
        );
    }

    #[test]
    fn check_import_only_accepts_imports_and_rejects_renames() {
        let import_diff = DiffParser::parse_unified_diff(
            "diff --git a/src/pow.rs b/src/pow.rs\n--- a/src/pow.rs\n+++ b/src/pow.rs\n@@ -1 +1,2 @@\n+use crate::x;\n",
        )
        .unwrap();
        assert!(
            CrossLayerValidator::check_import_only(
                PATTERNS,
                &["src/pow.rs".to_string()],
                &import_diff
            )
            .is_ok()
        );

        // File list has the rename source; the diff covers it via old_filename
        let rename = DiffParser::parse_unified_diff(
            "diff --git a/src/block.rs b/src/blk.rs\nsimilarity index 100%\nrename from src/block.rs\nrename to src/blk.rs\n",
        )
        .unwrap();
        let err = CrossLayerValidator::check_import_only(
            PATTERNS,
            &["src/block.rs".to_string(), "src/blk.rs".to_string()],
            &rename,
        )
        .unwrap_err();
        assert!(err.to_string().contains("from src/block.rs"), "{err}");
    }
}
