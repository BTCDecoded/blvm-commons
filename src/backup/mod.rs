//! Automated backup system for database and configuration
//!
//! Provides periodic backups with verification and retention management.

use crate::database::Database;
use crate::error::GovernanceError;
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::time::interval;
use tracing::{error, info, warn};

/// Backup configuration
#[derive(Debug, Clone)]
pub struct BackupConfig {
    /// Backup directory
    pub directory: PathBuf,
    /// Retention period in days
    pub retention_days: u32,
    /// Enable compression
    pub compression: bool,
    /// Backup interval
    pub interval: Duration,
    /// Enable automatic backups
    pub enabled: bool,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            directory: PathBuf::from("/opt/bllvm-commons/backups"),
            retention_days: 30,
            compression: true,
            interval: Duration::from_secs(86400), // Daily
            enabled: true,
        }
    }
}

/// Backup manager
pub struct BackupManager {
    database: Database,
    config: BackupConfig,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(database: Database, config: BackupConfig) -> Self {
        Self { database, config }
    }

    /// Create a backup of the database
    pub async fn create_backup(&self) -> Result<PathBuf, GovernanceError> {
        // Ensure backup directory exists
        fs::create_dir_all(&self.config.directory)
            .await
            .map_err(|e| {
                GovernanceError::ConfigError(format!(
                    "Failed to create backup directory: {}",
                    e
                ))
            })?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("governance_backup_{}.db", timestamp);
        let backup_path = self.config.directory.join(&backup_filename);

        info!("Creating database backup: {}", backup_path.display());

        // Create backup based on database type
        if self.database.is_sqlite() {
            self.backup_sqlite(&backup_path).await?;
        } else if self.database.is_postgres() {
            // PostgreSQL backups require pg_dump (external tool)
            // For now, we'll log a warning and skip
            warn!("PostgreSQL backups require external pg_dump tool - skipping automated backup");
            return Err(GovernanceError::ConfigError(
                "PostgreSQL backups require external pg_dump tool".to_string(),
            ));
        } else {
            return Err(GovernanceError::ConfigError(
                "Unknown database type for backup".to_string(),
            ));
        }

        // Verify backup
        self.verify_backup(&backup_path).await?;

        // Compress if enabled
        let final_path = if self.config.compression {
            self.compress_backup(&backup_path).await?
        } else {
            backup_path
        };

        info!("Backup created successfully: {}", final_path.display());

        Ok(final_path)
    }

    /// Backup SQLite database
    async fn backup_sqlite(&self, backup_path: &Path) -> Result<(), GovernanceError> {
        // SQLite backup using VACUUM INTO (SQLite 3.27+)
        // This creates a clean copy of the database
        if let Some(pool) = self.database.get_sqlite_pool() {
            // Escape the path for SQL (replace single quotes with double single quotes)
            let escaped_path = backup_path.to_string_lossy().replace("'", "''");
            
            // Use sqlx to execute VACUUM INTO
            sqlx::query(&format!(
                "VACUUM INTO '{}'",
                escaped_path
            ))
            .execute(pool)
            .await
            .map_err(|e| {
                GovernanceError::DatabaseError(format!("SQLite backup failed: {}", e))
            })?;

            info!("SQLite backup created: {}", backup_path.display());
            Ok(())
        } else {
            Err(GovernanceError::DatabaseError(
                "SQLite pool not available".to_string(),
            ))
        }
    }

    /// Verify backup integrity
    async fn verify_backup(&self, backup_path: &Path) -> Result<(), GovernanceError> {
        if self.database.is_sqlite() {
            // Verify SQLite backup by opening it and running integrity_check
            let backup_url = format!("sqlite:{}", backup_path.to_string_lossy());
            let backup_pool = sqlx::sqlite::SqlitePool::connect(&backup_url)
                .await
                .map_err(|e| {
                    GovernanceError::DatabaseError(format!(
                        "Failed to open backup for verification: {}",
                        e
                    ))
                })?;

            let result: (String,) = sqlx::query_as("PRAGMA integrity_check")
                .fetch_one(&backup_pool)
                .await
                .map_err(|e| {
                    GovernanceError::DatabaseError(format!(
                        "Backup verification failed: {}",
                        e
                    ))
                })?;

            if result.0 != "ok" {
                return Err(GovernanceError::DatabaseError(format!(
                    "Backup integrity check failed: {}",
                    result.0
                )));
            }

            info!("Backup verification passed");
            Ok(())
        } else {
            // For PostgreSQL, verification would require pg_restore --list
            // Skip for now
            warn!("PostgreSQL backup verification skipped (requires external tool)");
            Ok(())
        }
    }

    /// Compress backup file
    async fn compress_backup(&self, backup_path: &Path) -> Result<PathBuf, GovernanceError> {
        use std::process::Command;

        let compressed_path = format!("{}.gz", backup_path.to_string_lossy());

        // Use gzip to compress (external tool)
        let output = Command::new("gzip")
            .arg("-f") // Force overwrite
            .arg(backup_path)
            .output()
            .map_err(|e| {
                GovernanceError::ConfigError(format!("Failed to compress backup: {}", e))
            })?;

        if !output.status.success() {
            return Err(GovernanceError::ConfigError(format!(
                "Backup compression failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        info!("Backup compressed: {}", compressed_path);
        Ok(PathBuf::from(compressed_path))
    }

    /// Clean up old backups based on retention policy
    pub async fn cleanup_old_backups(&self) -> Result<usize, GovernanceError> {
        let cutoff_time = Utc::now()
            .checked_sub_signed(chrono::Duration::days(self.config.retention_days as i64))
            .ok_or_else(|| {
                GovernanceError::ConfigError("Failed to calculate cutoff time".to_string())
            })?;

        let mut deleted_count = 0;

        // List all backup files
        let mut entries = fs::read_dir(&self.config.directory)
            .await
            .map_err(|e| {
                GovernanceError::ConfigError(format!(
                    "Failed to read backup directory: {}",
                    e
                ))
            })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            GovernanceError::ConfigError(format!("Failed to read directory entry: {}", e))
        })? {
            let path = entry.path();
            if path.is_file() && path.to_string_lossy().contains("governance_backup_") {
                // Get file metadata
                let metadata = entry.metadata().await.map_err(|e| {
                    GovernanceError::ConfigError(format!("Failed to read file metadata: {}", e))
                })?;

                // Get modification time
                if let Ok(modified) = metadata.modified() {
                    let modified_time: chrono::DateTime<Utc> = modified.into();
                    if modified_time < cutoff_time {
                        fs::remove_file(&path).await.map_err(|e| {
                            GovernanceError::ConfigError(format!(
                                "Failed to delete old backup: {}",
                                e
                            ))
                        })?;
                        deleted_count += 1;
                        info!("Deleted old backup: {}", path.display());
                    }
                }
            }
        }

        if deleted_count > 0 {
            info!("Cleaned up {} old backup(s)", deleted_count);
        }

        Ok(deleted_count)
    }

    /// Start periodic backup task
    pub fn start_backup_task(self: Arc<Self>) {
        tokio::spawn(async move {
            if !self.config.enabled {
                info!("Automated backups are disabled");
                return;
            }

            let mut interval = interval(self.config.interval);
            info!(
                "Starting automated backup task (interval: {:?})",
                self.config.interval
            );

            loop {
                interval.tick().await;

                // Create backup
                match self.create_backup().await {
                    Ok(backup_path) => {
                        info!("Automated backup created: {}", backup_path.display());
                    }
                    Err(e) => {
                        error!("Automated backup failed: {}", e);
                    }
                }

                // Clean up old backups
                if let Err(e) = self.cleanup_old_backups().await {
                    warn!("Failed to cleanup old backups: {}", e);
                }
            }
        });
    }
}

use std::sync::Arc;

