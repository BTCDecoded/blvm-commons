//! Build orchestrator - main coordination logic

use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tracing::{info, warn, error};
use serde_json::json;

use crate::error::GovernanceError;
use crate::github::client::GitHubClient;
use crate::database::Database;

use super::dependency::DependencyGraph;
use super::monitor::{BuildMonitor, BuildStatus};
use super::artifacts::ArtifactCollector;

/// Build orchestrator for coordinating cross-repository builds
pub struct BuildOrchestrator {
    github_client: GitHubClient,
    database: Database,
    dependency_graph: DependencyGraph,
    monitor: BuildMonitor,
    artifact_collector: ArtifactCollector,
    organization: String,
}

impl BuildOrchestrator {
    pub fn new(
        github_client: GitHubClient,
        database: Database,
        organization: String,
    ) -> Self {
        let dependency_graph = DependencyGraph::new(organization.clone());
        let monitor = BuildMonitor::new(
            github_client.clone(),
            organization.clone(),
            Duration::from_secs(3600), // 1 hour timeout
            3, // Max 3 retries
        );
        let artifact_collector = ArtifactCollector::new(
            github_client.clone(),
            organization.clone(),
        );
        
        Self {
            github_client,
            database,
            dependency_graph,
            monitor,
            artifact_collector,
            organization,
        }
    }
    
    /// Handle a release event and orchestrate builds
    pub async fn handle_release_event(
        &self,
        version: &str,
        prerelease: bool,
    ) -> Result<(), GovernanceError> {
        info!("Handling release event for version {} (prerelease: {})", version, prerelease);
        
        // Get build order
        let build_order = self.dependency_graph.get_build_order()
            .map_err(|e| GovernanceError::BuildError(format!("Failed to get build order: {}", e)))?;
        
        info!("Build order: {:?}", build_order);
        
        // Trigger builds in parallel groups (respecting dependencies)
        let parallel_groups = self.dependency_graph.get_parallel_groups()
            .map_err(|e| GovernanceError::BuildError(format!("Failed to get parallel groups: {}", e)))?;
        
        info!("Build groups (can build in parallel): {:?}", parallel_groups);
        
        let mut triggered_builds = HashMap::new();
        let mut completed_repos = HashSet::new();
        
        // Process each parallel group sequentially, but repos within a group can build in parallel
        for group in &parallel_groups {
            // Wait for all dependencies of this group to complete
            for repo in group {
                let deps = self.dependency_graph.get_dependencies(repo);
                for dep in &deps {
                    if !completed_repos.contains(dep) {
                        if let Some(dep_run_id) = triggered_builds.get(dep) {
                            info!("Waiting for dependency {} to complete before building {}", dep, repo);
                            let status = self.monitor.monitor_build(dep, *dep_run_id).await?;
                            
                            if status != BuildStatus::Success {
                                return Err(GovernanceError::BuildError(format!(
                                    "Dependency {} failed with status: {:?}", dep, status
                                )));
                            }
                            completed_repos.insert(dep.clone());
                        }
                    }
                }
            }
            
            // Trigger all builds in this group (they can run in parallel)
            for repo in group {
                info!("Triggering build for {}", repo);
                let workflow_run_id = self.trigger_build(repo, version).await?;
                triggered_builds.insert(repo.clone(), workflow_run_id);
            }
        }
        
        // Monitor all builds
        info!("Monitoring {} builds", triggered_builds.len());
        let build_results = self.monitor.monitor_builds(triggered_builds.clone()).await?;
        
        // Check for failures
        for (repo, status) in &build_results {
            if *status != BuildStatus::Success {
                error!("Build failed for {}: {:?}", repo, status);
                return Err(GovernanceError::BuildError(format!(
                    "Build failed for {}: {:?}", repo, status
                )));
            }
        }
        
        // Collect artifacts
        info!("Collecting artifacts from all builds");
        let mut artifacts = self.artifact_collector.collect_all_artifacts(&triggered_builds).await?;
        
        // Download artifacts
        info!("Downloading artifacts from all repositories");
        self.artifact_collector.download_all_artifacts(&mut artifacts).await?;
        
        // Create GitHub release
        info!("Creating GitHub release for version {}", version);
        self.create_github_release(version, prerelease, &artifacts).await?;
        
        info!("Release orchestration completed successfully for version {}", version);
        Ok(())
    }
    
    /// Trigger a build for a repository via repository_dispatch
    async fn trigger_build(
        &self,
        repo: &str,
        version: &str,
    ) -> Result<u64, GovernanceError> {
        info!("Triggering build for {}/{} (version: {})", self.organization, repo, version);
        
        // Create repository_dispatch event payload
        let client_payload = json!({
            "version": version,
            "triggered_by": "bllvm-commons",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        // Trigger workflow via repository_dispatch
        // Use "build-chain" to match existing workflows, fallback to "build-request" for new repos
        let event_type = if repo == "bllvm-commons" { "build-request" } else { "build-chain" };
        let workflow_run_id = self.github_client
            .trigger_workflow(
                &self.organization,
                repo,
                event_type,
                &client_payload,
            )
            .await?;
        
        // If we got 0, try to find the workflow run by polling
        if workflow_run_id == 0 {
            warn!("Workflow run ID not found immediately, polling for it");
            // Try to find it by listing recent workflow runs
            // Try multiple workflow files (build.yml or ci.yml)
            let workflow_files = [".github/workflows/build.yml", ".github/workflows/ci.yml"];
            let mut runs = Vec::new();
            
            for workflow_file in &workflow_files {
                if let Ok(found_runs) = self.github_client
                    .list_workflow_runs(&self.organization, repo, Some(workflow_file), None, Some(1))
                    .await
                {
                    runs = found_runs;
                    if !runs.is_empty() {
                        break;
                    }
                }
            }
            
            if let Some(run) = runs.first() {
                if let Some(id) = run.get("id").and_then(|v| v.as_u64()) {
                    info!("Found workflow run ID {} for {}", id, repo);
                    return Ok(id);
                }
            }
        }
        
        Ok(workflow_run_id)
    }
    
    /// Create a GitHub release with artifacts
    async fn create_github_release(
        &self,
        version: &str,
        prerelease: bool,
        artifacts: &HashMap<String, Vec<super::artifacts::Artifact>>,
    ) -> Result<(), GovernanceError> {
        info!("Creating GitHub release: {} (prerelease: {})", version, prerelease);
        
        // Build release body with artifact list
        let mut release_body = format!(
            "Bitcoin Commons Release {}\n\nThis release was orchestrated by bllvm-commons.\n\n## Artifacts\n\n",
            version
        );
        
        for (repo, repo_artifacts) in artifacts {
            release_body.push_str(&format!("### {}\n\n", repo));
            for artifact in repo_artifacts {
                release_body.push_str(&format!("- {} ({} bytes)\n", artifact.name, artifact.size));
            }
            release_body.push_str("\n");
        }
        
        // Create release in bllvm repository
        let release = self.github_client
            .client
            .repos(&self.organization, "bllvm")
            .releases()
            .create(&json!({
                "tag_name": version,
                "name": format!("Bitcoin Commons {}", version),
                "body": release_body,
                "prerelease": prerelease,
                "draft": false,
            }))
            .send()
            .await
            .map_err(|e| {
                error!("Failed to create release: {}", e);
                GovernanceError::GitHubError(format!("Failed to create release: {}", e))
            })?;
        
        info!("Created GitHub release: {} (ID: {})", release.html_url, release.id);
        
        // Upload artifacts to the release
        let release_id = release.id;
        let mut uploaded_count = 0;
        let mut failed_count = 0;
        
        for (repo, repo_artifacts) in artifacts {
            for artifact in repo_artifacts {
                // Only upload if we have the data
                if let Some(ref data) = artifact.data {
                    // Create asset name with repo prefix to avoid conflicts
                    let asset_name = format!("{}-{}", repo, artifact.name);
                    
                    match self.github_client
                        .upload_release_asset(
                            &self.organization,
                            "bllvm",
                            release_id,
                            &asset_name,
                            data,
                            &artifact.content_type,
                        )
                        .await
                    {
                        Ok(()) => {
                            info!("Uploaded artifact: {}", asset_name);
                            uploaded_count += 1;
                        }
                        Err(e) => {
                            error!("Failed to upload artifact '{}': {}", asset_name, e);
                            failed_count += 1;
                            // Continue with other artifacts
                        }
                    }
                } else {
                    warn!("Skipping artifact '{}' from {} - no data downloaded", artifact.name, repo);
                    failed_count += 1;
                }
            }
        }
        
        info!(
            "Artifact upload complete: {} uploaded, {} failed",
            uploaded_count, failed_count
        );
        
        if uploaded_count == 0 && failed_count > 0 {
            warn!("No artifacts were uploaded successfully");
        }
        
        Ok(())
    }
}

