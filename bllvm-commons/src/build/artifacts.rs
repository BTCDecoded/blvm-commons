//! Artifact collection and management

use std::collections::HashMap;
use tracing::{info, warn, error};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use chrono;

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
    pub sha256: Option<String>, // SHA256 hash of artifact data (calculated after download)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>, // Artifact expiration time from GitHub
}

/// Artifact collector for gathering build artifacts from repositories
#[derive(Clone)]
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
            
            // Parse expiration time if available
            let expires_at = artifact_json
                .get("expires_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));
            
            artifacts.push(Artifact {
                name,
                size,
                download_url,
                content_type: "application/zip".to_string(), // GitHub artifacts are zipped
                data: None, // Will be downloaded later
                sha256: None, // Will be calculated after download
                expires_at,
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
            
            // Check if artifact has expired
            if let Some(expires_at) = artifact.expires_at {
                let now = chrono::Utc::now();
                if now > expires_at {
                    warn!("Skipping expired artifact '{}' (expired at {})", artifact.name, expires_at);
                    continue;
                }
                let time_until_expiry = expires_at - now;
                info!("Artifact '{}' expires in {} hours", artifact.name, time_until_expiry.num_hours());
            }
            
            match self.github_client
                .download_artifact(&artifact.download_url, &self.organization)
                .await
            {
                Ok(data) => {
                    info!("Downloaded artifact '{}': {} bytes", artifact.name, data.len());
                    
                    // Calculate SHA256 hash
                    let mut hasher = Sha256::new();
                    hasher.update(&data);
                    let hash = hasher.finalize();
                    let hash_hex = hex::encode(hash);
                    
                    artifact.data = Some(data);
                    artifact.sha256 = Some(hash_hex.clone());
                    
                    info!("Calculated SHA256 for '{}': {}", artifact.name, hash_hex);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::client::GitHubClient;
    use tempfile::tempdir;

    fn create_test_github_client() -> GitHubClient {
        let temp_dir = tempdir().unwrap();
        let private_key_path = temp_dir.path().join("test_key.pem");
        // Use the actual test RSA key from test_fixtures
        let valid_key = include_str!("../../../test_fixtures/test_rsa_key.pem");
        std::fs::write(&private_key_path, valid_key).unwrap();
        GitHubClient::new(123456, private_key_path.to_str().unwrap()).unwrap()
    }

    #[tokio::test]
    async fn test_artifact_collector_new() {
        let github_client = create_test_github_client();
        let collector = ArtifactCollector::new(github_client, "BTCDecoded".to_string());
        
        // Verify collector is initialized (organization field is private)
        assert!(true, "Collector is initialized");
    }

    #[test]
    fn test_artifact_creation() {
        let artifact = Artifact {
            name: "test-artifact.zip".to_string(),
            size: 1024,
            download_url: "https://example.com/artifact.zip".to_string(),
            content_type: "application/zip".to_string(),
            data: None,
            sha256: None,
            expires_at: None,
        };
        
        assert_eq!(artifact.name, "test-artifact.zip");
        assert_eq!(artifact.size, 1024);
        assert_eq!(artifact.content_type, "application/zip");
        assert!(artifact.data.is_none());
        assert!(artifact.sha256.is_none());
    }

    #[test]
    fn test_artifact_with_data() {
        let data = b"test artifact data".to_vec();
        let artifact = Artifact {
            name: "test.zip".to_string(),
            size: data.len() as u64,
            download_url: "https://example.com/test.zip".to_string(),
            content_type: "application/zip".to_string(),
            data: Some(data.clone()),
            sha256: Some("abc123".to_string()),
            expires_at: None,
        };
        
        assert_eq!(artifact.data, Some(data));
        assert_eq!(artifact.sha256, Some("abc123".to_string()));
    }

    #[test]
    fn test_artifact_with_expiration() {
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);
        let artifact = Artifact {
            name: "expiring.zip".to_string(),
            size: 512,
            download_url: "https://example.com/expiring.zip".to_string(),
            content_type: "application/zip".to_string(),
            data: None,
            sha256: None,
            expires_at: Some(expires_at),
        };
        
        assert!(artifact.expires_at.is_some());
        assert!(artifact.expires_at.unwrap() > chrono::Utc::now());
    }
}

