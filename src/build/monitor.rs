//! Build status monitoring

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

use crate::error::GovernanceError;
use crate::github::client::GitHubClient;

/// Build status for a repository
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BuildStatus {
    Pending,
    InProgress,
    Success,
    Failure,
    Cancelled,
    TimedOut,
}

/// Build state for a single repository
#[derive(Debug, Clone)]
pub struct BuildState {
    pub repo: String,
    pub status: BuildStatus,
    pub workflow_run_id: Option<u64>,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub error: Option<String>,
    pub retry_count: u32,
}

/// Build monitor for tracking build status across repositories
pub struct BuildMonitor {
    github_client: GitHubClient,
    organization: String,
    timeout: Duration,
    max_retries: u32,
    poll_interval: Duration,
}

impl BuildMonitor {
    pub fn new(
        github_client: GitHubClient,
        organization: String,
        timeout: Duration,
        max_retries: u32,
    ) -> Self {
        Self {
            github_client,
            organization,
            timeout,
            max_retries,
            poll_interval: Duration::from_secs(10), // Poll every 10 seconds
        }
    }
    
    /// Monitor a single build until completion
    pub async fn monitor_build(
        &self,
        repo: &str,
        workflow_run_id: u64,
    ) -> Result<BuildStatus, GovernanceError> {
        info!("Monitoring build for {}/{} (run ID: {})", self.organization, repo, workflow_run_id);
        
        let start_time = Instant::now();
        
        loop {
            // Check timeout
            if start_time.elapsed() > self.timeout {
                warn!("Build timeout for {}/{}", self.organization, repo);
                return Ok(BuildStatus::TimedOut);
            }
            
            // Get workflow run status
            match self.get_workflow_run_status(repo, workflow_run_id).await {
                Ok(status) => {
                    match status {
                        BuildStatus::Success | BuildStatus::Failure | BuildStatus::Cancelled => {
                            info!("Build completed for {}/{}: {:?}", self.organization, repo, status);
                            return Ok(status);
                        }
                        BuildStatus::InProgress | BuildStatus::Pending => {
                            // Continue monitoring
                            sleep(self.poll_interval).await;
                        }
                        _ => {
                            warn!("Unexpected build status for {}/{}: {:?}", self.organization, repo, status);
                            sleep(self.poll_interval).await;
                        }
                    }
                }
                Err(e) => {
                    error!("Error checking build status for {}/{}: {}", self.organization, repo, e);
                    sleep(self.poll_interval).await;
                }
            }
        }
    }
    
    /// Monitor multiple builds in parallel
    pub async fn monitor_builds(
        &self,
        builds: HashMap<String, u64>,
    ) -> Result<HashMap<String, BuildStatus>, GovernanceError> {
        info!("Monitoring {} builds", builds.len());
        
        let mut tasks = Vec::new();
        let mut results = HashMap::new();
        
        // Start monitoring tasks for each build
        for (repo, workflow_run_id) in builds {
            let monitor = self.clone();
            let repo_clone = repo.clone();
            let _org_clone = self.organization.clone();
            
            let task = tokio::spawn(async move {
                let status = monitor.monitor_build(&repo_clone, workflow_run_id).await;
                (repo_clone, status)
            });
            
            tasks.push(task);
        }
        
        // Wait for all builds to complete
        for task in tasks {
            match task.await {
                Ok((repo, Ok(status))) => {
                    results.insert(repo, status);
                }
                Ok((repo, Err(e))) => {
                    error!("Error monitoring build for {}: {}", repo, e);
                    results.insert(repo, BuildStatus::Failure);
                }
                Err(e) => {
                    error!("Task error: {}", e);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Get workflow run status from GitHub
    async fn get_workflow_run_status(
        &self,
        repo: &str,
        workflow_run_id: u64,
    ) -> Result<BuildStatus, GovernanceError> {
        // If workflow_run_id is 0, we need to find it by polling recent runs
        if workflow_run_id == 0 {
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
                    return self.get_workflow_run_status_by_id(repo, id).await;
                }
            }
            
            // If we still can't find it, return Pending
            return Ok(BuildStatus::Pending);
        }
        
        self.get_workflow_run_status_by_id(repo, workflow_run_id).await
    }
    
    /// Get workflow run status by ID
    async fn get_workflow_run_status_by_id(
        &self,
        repo: &str,
        workflow_run_id: u64,
    ) -> Result<BuildStatus, GovernanceError> {
        // Get workflow run status from GitHub API
        let run_status = self.github_client
            .get_workflow_run_status(&self.organization, repo, workflow_run_id)
            .await?;
        
        // Parse status from response
        let status_str = run_status
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        
        let conclusion = run_status
            .get("conclusion")
            .and_then(|v| v.as_str());
        
        match status_str {
            "Completed" => {
                match conclusion {
                    Some("Success") => Ok(BuildStatus::Success),
                    Some("Failure") => Ok(BuildStatus::Failure),
                    Some("Cancelled") => Ok(BuildStatus::Cancelled),
                    _ => Ok(BuildStatus::Failure),
                }
            }
            "InProgress" => Ok(BuildStatus::InProgress),
            "Queued" => Ok(BuildStatus::Pending),
            _ => {
                warn!("Unknown workflow status: {}", status_str);
                Ok(BuildStatus::Pending)
            }
        }
    }
}

impl Clone for BuildMonitor {
    fn clone(&self) -> Self {
        Self {
            github_client: self.github_client.clone(),
            organization: self.organization.clone(),
            timeout: self.timeout,
            max_retries: self.max_retries,
            poll_interval: self.poll_interval,
        }
    }
}

