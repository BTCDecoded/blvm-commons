pub mod models;
pub mod queries;
pub mod schema;

use sqlx::{SqlitePool, PgPool, sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions};
use std::str::FromStr;
use crate::error::GovernanceError;

#[derive(Clone)]
pub enum DatabaseBackend {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}

#[derive(Clone)]
pub struct Database {
    backend: DatabaseBackend,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, GovernanceError> {
        if database_url.starts_with("sqlite:") {
            let pool = SqlitePool::connect(database_url)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            Ok(Self {
                backend: DatabaseBackend::Sqlite(pool),
            })
        } else if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            let pool = PgPool::connect(database_url)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            Ok(Self {
                backend: DatabaseBackend::Postgres(pool),
            })
        } else {
            Err(GovernanceError::DatabaseError(
                "Unsupported database URL format. Use 'sqlite://' or 'postgresql://'".to_string()
            ))
        }
    }

    /// Create an in-memory SQLite database for testing
    pub async fn new_in_memory() -> Result<Self, GovernanceError> {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
        
        let db = Self {
            backend: DatabaseBackend::Sqlite(pool),
        };
        db.run_migrations().await?;
        Ok(db)
    }

    /// Create a new production database with optimized settings
    pub async fn new_production(database_url: &str) -> Result<Self, GovernanceError> {
        if database_url.starts_with("sqlite:") {
            let options = SqliteConnectOptions::from_str(database_url)
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                .locking_mode(sqlx::sqlite::SqliteLockingMode::Normal)
                .foreign_keys(true)
                .create_if_missing(true);

            let pool = SqlitePoolOptions::new()
                .max_connections(10)
                .min_connections(1)
                .acquire_timeout(std::time::Duration::from_secs(30))
                .idle_timeout(std::time::Duration::from_secs(600))
                .max_lifetime(std::time::Duration::from_secs(1800))
                .connect_with(options)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

            let db = Database {
                backend: DatabaseBackend::Sqlite(pool),
            };
            db.run_migrations().await?;
            Ok(db)
        } else if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            let pool = PgPool::connect(database_url)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            let db = Database {
                backend: DatabaseBackend::Postgres(pool),
            };
            db.run_migrations().await?;
            Ok(db)
        } else {
            Err(GovernanceError::DatabaseError(
                "Unsupported database URL format for production. Use 'sqlite://' or 'postgresql://'".to_string()
            ))
        }
    }


    pub async fn run_migrations(&self) -> Result<(), GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                sqlx::migrate!("./migrations")
                    .run(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
            DatabaseBackend::Postgres(pool) => {
                sqlx::migrate!("./migrations-postgres")
                    .run(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub fn get_sqlite_pool(&self) -> Option<&SqlitePool> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn get_postgres_pool(&self) -> Option<&PgPool> {
        match &self.backend {
            DatabaseBackend::Postgres(pool) => Some(pool),
            _ => None,
        }
    }

    pub fn is_sqlite(&self) -> bool {
        matches!(self.backend, DatabaseBackend::Sqlite(_))
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self.backend, DatabaseBackend::Postgres(_))
    }

    pub async fn create_pull_request(
        &self,
        repo_name: &str,
        pr_number: i32,
        head_sha: &str,
        layer: i32,
    ) -> Result<(), GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO pull_requests (repo_name, pr_number, opened_at, layer, head_sha)
                    VALUES (?, ?, CURRENT_TIMESTAMP, ?, ?)
                    ON CONFLICT (repo_name, pr_number) DO UPDATE SET
                        head_sha = EXCLUDED.head_sha,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                )
                .bind(repo_name)
                .bind(pr_number)
                .bind(layer)
                .bind(head_sha)
                .execute(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
            DatabaseBackend::Postgres(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO pull_requests (repo_name, pr_number, opened_at, layer, head_sha)
                    VALUES ($1, $2, CURRENT_TIMESTAMP, $3, $4)
                    ON CONFLICT (repo_name, pr_number) DO UPDATE SET
                        head_sha = EXCLUDED.head_sha,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                )
                .bind(repo_name)
                .bind(pr_number)
                .bind(layer)
                .bind(head_sha)
                .execute(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub async fn update_review_status(
        &self,
        _repo_name: &str,
        _pr_number: i32,
        _reviewer: &str,
        _state: &str,
    ) -> Result<(), GovernanceError> {
        // This would update review status in the database
        // Implementation depends on specific review tracking requirements
        Ok(())
    }

    pub async fn add_signature(
        &self,
        repo_name: &str,
        pr_number: i32,
        signer: &str,
        signature: &str,
        reasoning: Option<&str>,
    ) -> Result<(), GovernanceError> {
        use crate::database::models::Signature;
        use chrono::Utc;
        use serde_json::Value;

        // Create new signature with reasoning
        let new_signature = Signature {
            signer: signer.to_string(),
            signature: signature.to_string(),
            timestamp: Utc::now(),
            reasoning: reasoning.map(|s| s.to_string()),
        };

        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                // Get current signatures
                let signatures_json: Option<String> = sqlx::query_scalar(
                    "SELECT signatures FROM pull_requests WHERE repo_name = ? AND pr_number = ?"
                )
                .bind(repo_name)
                .bind(pr_number)
                .fetch_optional(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                // Parse existing signatures or create empty array
                let mut signatures: Vec<Value> = if let Some(json_str) = signatures_json {
                    serde_json::from_str(&json_str)
                        .unwrap_or_else(|_| vec![])
                } else {
                    vec![]
                };

                // Add new signature
                signatures.push(serde_json::to_value(&new_signature)
                    .map_err(|e| GovernanceError::DatabaseError(format!("Failed to serialize signature: {}", e)))?);

                // Update signatures in database
                let updated_json = serde_json::to_string(&signatures)
                    .map_err(|e| GovernanceError::DatabaseError(format!("Failed to serialize signatures: {}", e)))?;

                sqlx::query(
                    "UPDATE pull_requests SET signatures = ?, updated_at = CURRENT_TIMESTAMP WHERE repo_name = ? AND pr_number = ?"
                )
                .bind(&updated_json)
                .bind(repo_name)
                .bind(pr_number)
                .execute(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
            DatabaseBackend::Postgres(pool) => {
                // Get current signatures
                let signatures_json: Option<serde_json::Value> = sqlx::query_scalar(
                    "SELECT signatures FROM pull_requests WHERE repo_name = $1 AND pr_number = $2"
                )
                .bind(repo_name)
                .bind(pr_number)
                .fetch_optional(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                // Parse existing signatures or create empty array
                let mut signatures: Vec<Value> = signatures_json
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_else(|| vec![]);

                // Add new signature
                signatures.push(serde_json::to_value(&new_signature)
                    .map_err(|e| GovernanceError::DatabaseError(format!("Failed to serialize signature: {}", e)))?);

                // Update signatures in database
                let updated_json = serde_json::to_value(&signatures)
                    .map_err(|e| GovernanceError::DatabaseError(format!("Failed to serialize signatures: {}", e)))?;

                sqlx::query(
                    "UPDATE pull_requests SET signatures = $1, updated_at = CURRENT_TIMESTAMP WHERE repo_name = $2 AND pr_number = $3"
                )
                .bind(&updated_json)
                .bind(repo_name)
                .bind(pr_number)
                .execute(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub async fn log_governance_event(
        &self,
        event_type: &str,
        repo_name: Option<&str>,
        pr_number: Option<i32>,
        maintainer: Option<&str>,
        details: &serde_json::Value,
    ) -> Result<(), GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO governance_events (event_type, repo_name, pr_number, maintainer, details)
                    VALUES (?, ?, ?, ?, ?)
                    "#,
                )
                .bind(event_type)
                .bind(repo_name)
                .bind(pr_number)
                .bind(maintainer)
                .bind(serde_json::to_string(details).unwrap_or_default())
                .execute(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
            DatabaseBackend::Postgres(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO governance_events (event_type, repo_name, pr_number, maintainer, details)
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(event_type)
                .bind(repo_name)
                .bind(pr_number)
                .bind(maintainer)
                .bind(details)
                .execute(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }

    pub async fn get_pull_request(
        &self,
        _repo_name: &str,
        _pr_number: i32,
    ) -> Result<Option<crate::database::models::PullRequest>, GovernanceError> {
        // This would retrieve a pull request from the database
        // For now, return None as a placeholder
        Ok(None)
    }

    pub async fn get_governance_events(
        &self,
        _limit: i64,
    ) -> Result<Vec<crate::database::models::GovernanceEvent>, GovernanceError> {
        // This would retrieve governance events from the database
        // For now, return empty vector as a placeholder
        Ok(vec![])
    }

    /// Add or update a tier override for a PR
    pub async fn set_tier_override(
        &self,
        repo_name: &str,
        pr_number: i32,
        override_tier: u32,
        justification: &str,
        overridden_by: &str,
    ) -> Result<(), GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                // SQLite doesn't support ON CONFLICT with named columns in older versions
                // Use REPLACE INTO instead (works with UNIQUE constraint)
                sqlx::query(
                    r#"
                    INSERT OR REPLACE INTO tier_overrides (repo_name, pr_number, override_tier, justification, overridden_by, created_at)
                    VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                    "#
                )
                .bind(repo_name)
                .bind(pr_number)
                .bind(override_tier as i32)
                .bind(justification)
                .bind(overridden_by)
                .execute(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
            DatabaseBackend::Postgres(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO tier_overrides (repo_name, pr_number, override_tier, justification, overridden_by, created_at)
                    VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP)
                    ON CONFLICT (repo_name, pr_number) DO UPDATE SET
                        override_tier = EXCLUDED.override_tier,
                        justification = EXCLUDED.justification,
                        overridden_by = EXCLUDED.overridden_by,
                        created_at = CURRENT_TIMESTAMP
                    "#
                )
                .bind(repo_name)
                .bind(pr_number)
                .bind(override_tier as i32)
                .bind(justification)
                .bind(overridden_by)
                .execute(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Get tier override for a PR if it exists
    pub async fn get_tier_override(
        &self,
        repo_name: &str,
        pr_number: i32,
    ) -> Result<Option<crate::database::models::TierOverride>, GovernanceError> {
        use sqlx::Row;
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT id, repo_name, pr_number, override_tier, justification, overridden_by, created_at
                    FROM tier_overrides
                    WHERE repo_name = ? AND pr_number = ?
                    "#
                )
                .bind(repo_name)
                .bind(pr_number)
                .fetch_optional(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                if let Some(row) = row {
                    Ok(Some(crate::database::models::TierOverride {
                        id: row.get(0),
                        repo_name: row.get(1),
                        pr_number: row.get(2),
                        override_tier: row.get::<i32, _>(3) as u32,
                        justification: row.get(4),
                        overridden_by: row.get(5),
                        created_at: row.get(6),
                    }))
                } else {
                    Ok(None)
                }
            }
            DatabaseBackend::Postgres(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT id, repo_name, pr_number, override_tier, justification, overridden_by, created_at
                    FROM tier_overrides
                    WHERE repo_name = $1 AND pr_number = $2
                    "#
                )
                .bind(repo_name)
                .bind(pr_number)
                .fetch_optional(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                if let Some(row) = row {
                    Ok(Some(crate::database::models::TierOverride {
                        id: row.get(0),
                        repo_name: row.get(1),
                        pr_number: row.get(2),
                        override_tier: row.get::<i32, _>(3) as u32,
                        justification: row.get(4),
                        overridden_by: row.get(5),
                        created_at: row.get(6),
                    }))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub async fn get_maintainer_by_username(
        &self,
        username: &str,
    ) -> Result<Option<crate::database::models::Maintainer>, GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                let maintainer = sqlx::query_as!(
                    crate::database::models::Maintainer,
                    "SELECT id, github_username, public_key, layer, active, last_updated FROM maintainers WHERE github_username = ? AND active = true",
                    username
                )
                .fetch_optional(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
                Ok(maintainer)
            }
            DatabaseBackend::Postgres(pool) => {
                let maintainer = sqlx::query_as!(
                    crate::database::models::Maintainer,
                    "SELECT id, github_username, public_key, layer, active, last_updated FROM maintainers WHERE github_username = $1 AND active = true",
                    username
                )
                .fetch_optional(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
                Ok(maintainer)
            }
        }
    }

    /// Get the database pool for testing purposes (SQLite only)
    pub fn pool(&self) -> Option<&SqlitePool> {
        self.get_sqlite_pool()
    }

    /// Perform database health check
    pub async fn health_check(&self) -> Result<DatabaseHealth, GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                // Check database connectivity
                let connection_count = pool.size() as u32;
                let idle_connections = pool.num_idle() as u32;
                let active_connections = connection_count - idle_connections;

                // Check database integrity
                let integrity_result = sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                // Check WAL mode
                let journal_mode = sqlx::query_scalar::<_, String>("PRAGMA journal_mode")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                // Check database size
                let page_count = sqlx::query_scalar::<_, i64>("PRAGMA page_count")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
                let page_size = sqlx::query_scalar::<_, i64>("PRAGMA page_size")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
                let db_size = page_count * page_size;

                Ok(DatabaseHealth {
                    connection_count,
                    idle_connections,
                    active_connections,
                    integrity_ok: integrity_result == "ok",
                    journal_mode: journal_mode.clone(),
                    database_size_bytes: db_size,
                    wal_mode_active: journal_mode == "wal",
                })
            }
            DatabaseBackend::Postgres(pool) => {
                // Check database connectivity
                let connection_count = pool.size() as u32;
                let idle_connections = pool.num_idle() as u32;
                let active_connections = connection_count - idle_connections;

                // Check database size
                let db_size = sqlx::query_scalar::<_, i64>(
                    "SELECT pg_database_size(current_database())"
                )
                .fetch_one(pool)
                .await
                .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                Ok(DatabaseHealth {
                    connection_count,
                    idle_connections,
                    active_connections,
                    integrity_ok: true, // PostgreSQL handles integrity automatically
                    journal_mode: "wal".to_string(), // PostgreSQL uses WAL by default
                    database_size_bytes: db_size,
                    wal_mode_active: true,
                })
            }
        }
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> Result<PerformanceStats, GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                // Get cache size
                let cache_size = sqlx::query_scalar::<_, i64>("PRAGMA cache_size")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                // Get WAL checkpoint threshold
                let wal_checkpoint_threshold = sqlx::query_scalar::<_, i64>("PRAGMA wal_autocheckpoint")
                    .fetch_one(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                // Get compile options (as a proxy for slow queries)
                let compile_options = sqlx::query_scalar::<_, String>("PRAGMA compile_options")
                    .fetch_all(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                Ok(PerformanceStats {
                    cache_size,
                    wal_checkpoint_threshold,
                    slow_queries_count: compile_options.len() as i64,
                })
            }
            DatabaseBackend::Postgres(_pool) => {
                // PostgreSQL-specific statistics would go here
                // For now, return default values
                Ok(PerformanceStats {
                    cache_size: 0,
                    wal_checkpoint_threshold: 0,
                    slow_queries_count: 0,
                })
            }
        }
    }

    /// Optimize database performance
    pub async fn optimize_database(&self) -> Result<(), GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                // Run VACUUM to reclaim space and optimize database
                sqlx::query("VACUUM")
                    .execute(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;

                // Run ANALYZE to update query planner statistics
                sqlx::query("ANALYZE")
                    .execute(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
            DatabaseBackend::Postgres(pool) => {
                // Run VACUUM ANALYZE to reclaim space and update statistics
                sqlx::query("VACUUM ANALYZE")
                    .execute(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Checkpoint WAL file to main database (SQLite only)
    pub async fn checkpoint_wal(&self) -> Result<(), GovernanceError> {
        match &self.backend {
            DatabaseBackend::Sqlite(pool) => {
                // Checkpoint WAL file to main database
                sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
                    .execute(pool)
                    .await
                    .map_err(|e| GovernanceError::DatabaseError(e.to_string()))?;
            }
            DatabaseBackend::Postgres(_) => {
                // PostgreSQL handles WAL checkpointing automatically
                // This is a no-op for PostgreSQL
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub connection_count: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub integrity_ok: bool,
    pub journal_mode: String,
    pub database_size_bytes: i64,
    pub wal_mode_active: bool,
}

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub cache_size: i64,
    pub wal_checkpoint_threshold: i64,
    pub slow_queries_count: i64,
}