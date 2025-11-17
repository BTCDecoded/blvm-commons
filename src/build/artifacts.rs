//! Artifact collection and management

use std::collections::HashMap;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};

use crate::error::GovernanceError;
use crate::github::client::GitHubClient;

/// Artifact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub size: u64,
    pub download_url: String,
    pub content_type: String,
    pub data: Option<Vec<u8>>, // Downloaded artifact data
}

/// Artifact collector for gathering build artifacts from repositories
pub struct ArtifactCollector {
    github_client: GitHubClient,
    organization: String,
}

impl ArtifactCollector {
    pub fn new(github_client: GitHubClient, organization: String) -> Self {
        Self {
            github_client,
            organization,
        }
    }
    
    /// Collect artifacts from a workflow run
    pub async fn collect_artifacts(
        &self,
        repo: &str,
        workflow_run_id: u64,
    ) -> Result<Vec<Artifact>, GovernanceError> {
        info!("Collecting artifacts from {}/{} (run ID: {})", self.organization, repo, workflow_run_id);
        
        // List artifacts from workflow run
        let artifacts_list = self.github_client
            .list_workflow_run_artifacts(&self.organization, repo, workflow_run_id)
            .await?;
        
        let mut artifacts = Vec::new();
        
        for artifact_json in artifacts_list {
            let name = artifact_json
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            
            let size = artifact_json
                .get("size_in_bytes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            
            let download_url = artifact_json
                .get("archive_download_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            artifacts.push(Artifact {
                name,
                size,
                download_url,
                content_type: "application/zip".to_string(), // GitHub artifacts are zipped
                data: None, // Will be downloaded later
            });
        }
        
        info!("Collected {} artifacts from {}/{}", artifacts.len(), self.organization, repo);
        Ok(artifacts)
    }
    
    /// Collect artifacts from multiple repositories
    pub async fn collect_all_artifacts(
        &self,
        builds: &HashMap<String, u64>,
    ) -> Result<HashMap<String, Vec<Artifact>>, GovernanceError> {
        info!("Collecting artifacts from {} repositories", builds.len());
        
        let mut all_artifacts = HashMap::new();
        
        for (repo, workflow_run_id) in builds {
            match self.collect_artifacts(repo, *workflow_run_id).await {
                Ok(artifacts) => {
                    info!("Collected {} artifacts from {}", artifacts.len(), repo);
                    all_artifacts.insert(repo.clone(), artifacts);
                }
                Err(e) => {
                    error!("Failed to collect artifacts from {}: {}", repo, e);
                    all_artifacts.insert(repo.clone(), vec![]);
                }
            }
        }
        
        Ok(all_artifacts)
    }
    
    /// Download artifacts from their download URLs
    pub async fn download_artifacts(
        &self,
        artifacts: &mut [Artifact],
    ) -> Result<(), GovernanceError> {
        info!("Downloading {} artifacts", artifacts.len());
        
        for artifact in artifacts.iter_mut() {
            if artifact.download_url.is_empty() {
                warn!("Skipping artifact '{}' - no download URL", artifact.name);
                continue;
            }
            
            match self.github_client
                .download_artifact(&artifact.download_url, &self.organization)
                .await
            {
                Ok(data) => {
                    info!("Downloaded artifact '{}': {} bytes", artifact.name, data.len());
                    artifact.data = Some(data);
                }
                Err(e) => {
                    error!("Failed to download artifact '{}': {}", artifact.name, e);
                    // Continue with other artifacts even if one fails
                }
            }
        }
        
        Ok(())
    }
    
    /// Download all artifacts from all repositories
    pub async fn download_all_artifacts(
        &self,
        all_artifacts: &mut HashMap<String, Vec<Artifact>>,
    ) -> Result<(), GovernanceError> {
        info!("Downloading artifacts from {} repositories", all_artifacts.len());
        
        for (repo, artifacts) in all_artifacts.iter_mut() {
            info!("Downloading artifacts from {}", repo);
            if let Err(e) = self.download_artifacts(artifacts).await {
                error!("Failed to download artifacts from {}: {}", repo, e);
                // Continue with other repos
            }
        }
        
        Ok(())
    }
}

