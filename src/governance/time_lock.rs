//! Time Lock Tracking
//!
//! Tracks time-locked governance changes and enforces minimum time locks
//! before changes can be activated.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::database::Database;

/// Time lock status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeLockStatus {
    /// Time lock is pending (not yet elapsed)
    Pending,
    /// Time lock has elapsed and change can be activated
    Ready,
    /// Change has been activated
    Activated,
    /// Change was cancelled
    Cancelled,
}

/// Time lock configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLockConfig {
    /// Minimum time lock duration for Tier 3 changes (hours)
    pub tier3_min_hours: i64,
    /// Minimum time lock duration for Tier 4 changes (hours)
    pub tier4_min_hours: i64,
    /// Minimum time lock duration for Tier 5 changes (hours)
    pub tier5_min_hours: i64,
    /// User override threshold (percentage of active nodes)
    pub user_override_threshold: f64,
}

impl Default for TimeLockConfig {
    fn default() -> Self {
        Self {
            tier3_min_hours: 168,  // 7 days
            tier4_min_hours: 336,  // 14 days
            tier5_min_hours: 720,  // 30 days
            user_override_threshold: 0.75, // 75% of active nodes
        }
    }
}

/// Time-locked governance change
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TimeLockedChange {
    /// Unique identifier for the change
    pub change_id: String,
    /// Action tier (3, 4, or 5)
    pub tier: u8,
    /// Change description
    pub description: String,
    /// PR or issue number
    pub pr_number: Option<i64>,
    /// Time lock start time
    pub lock_start: DateTime<Utc>,
    /// Minimum lock duration (hours)
    pub min_duration_hours: i64,
    /// Time lock end time (lock_start + min_duration)
    pub lock_end: DateTime<Utc>,
    /// Current status
    pub status: String,
    /// User override signals (node_id -> timestamp)
    pub override_signals: HashMap<String, DateTime<Utc>>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Time lock manager
pub struct TimeLockManager {
    db: Database,
    config: TimeLockConfig,
}

impl TimeLockManager {
    /// Create a new time lock manager
    pub fn new(db: Database, config: TimeLockConfig) -> Self {
        Self { db, config }
    }

    /// Create a time lock for a governance change
    pub async fn create_time_lock(
        &self,
        change_id: &str,
        tier: u8,
        description: &str,
        pr_number: Option<i64>,
    ) -> Result<TimeLockedChange, sqlx::Error> {
        info!("Creating time lock for change {} (Tier {})", change_id, tier);

        // Determine minimum duration based on tier
        let min_duration_hours = match tier {
            3 => self.config.tier3_min_hours,
            4 => self.config.tier4_min_hours,
            5 => self.config.tier5_min_hours,
            _ => {
                warn!("Invalid tier {}, using Tier 5 duration", tier);
                self.config.tier5_min_hours
            }
        };

        let lock_start = Utc::now();
        let lock_end = lock_start + Duration::hours(min_duration_hours);

        // Insert into database
        let change = sqlx::query_as::<_, TimeLockedChange>(
            r#"
            INSERT INTO time_locked_changes (
                change_id, tier, description, pr_number,
                lock_start, min_duration_hours, lock_end, status,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(change_id)
        .bind(tier as i64)
        .bind(description)
        .bind(pr_number)
        .bind(lock_start)
        .bind(min_duration_hours)
        .bind(lock_end)
        .bind("pending")
        .bind(lock_start)
        .bind(lock_start)
        .fetch_one(self.db.pool().unwrap())
        .await?;

        info!(
            "Time lock created: {} until {}",
            change_id,
            lock_end.to_rfc3339()
        );

        Ok(change)
    }

    /// Check if a time lock has elapsed
    pub async fn check_time_lock(&self, change_id: &str) -> Result<TimeLockStatus, sqlx::Error> {
        let change = sqlx::query_as::<_, TimeLockedChange>(
            "SELECT * FROM time_locked_changes WHERE change_id = $1",
        )
        .bind(change_id)
        .fetch_optional(self.db.pool().unwrap())
        .await?;

        let change = match change {
            Some(c) => c,
            None => {
                warn!("Time lock not found: {}", change_id);
                return Ok(TimeLockStatus::Cancelled);
            }
        };

        // Check status
        let status = match change.status.as_str() {
            "activated" => return Ok(TimeLockStatus::Activated),
            "cancelled" => return Ok(TimeLockStatus::Cancelled),
            _ => {}
        };

        // Check if time lock has elapsed
        let now = Utc::now();
        if now >= change.lock_end {
            Ok(TimeLockStatus::Ready)
        } else {
            Ok(TimeLockStatus::Pending)
        }
    }

    /// Get time remaining for a time lock
    pub async fn get_time_remaining(
        &self,
        change_id: &str,
    ) -> Result<Option<Duration>, sqlx::Error> {
        let change = sqlx::query_as::<_, TimeLockedChange>(
            "SELECT * FROM time_locked_changes WHERE change_id = $1",
        )
        .bind(change_id)
        .fetch_optional(self.db.pool().unwrap())
        .await?;

        let change = match change {
            Some(c) => c,
            None => return Ok(None),
        };

        let now = Utc::now();
        if now >= change.lock_end {
            Ok(Some(Duration::zero()))
        } else {
            Ok(Some(change.lock_end - now))
        }
    }

    /// Record user override signal
    pub async fn record_override_signal(
        &self,
        change_id: &str,
        node_id: &str,
    ) -> Result<(), sqlx::Error> {
        debug!("Recording override signal for {} from node {}", change_id, node_id);

        // Update override signals JSON
        sqlx::query(
            r#"
            UPDATE time_locked_changes
            SET override_signals = jsonb_set(
                COALESCE(override_signals, '{}'::jsonb),
                $1,
                to_jsonb($2::text),
                true
            ),
            updated_at = $3
            WHERE change_id = $4
            "#,
        )
        .bind(format!("/{}", node_id))
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now())
        .bind(change_id)
        .execute(self.db.pool().unwrap())
        .await?;

        Ok(())
    }

    /// Check if user override threshold is met
    pub async fn check_override_threshold(
        &self,
        change_id: &str,
        active_node_count: usize,
    ) -> Result<bool, sqlx::Error> {
        let change = sqlx::query_as::<_, TimeLockedChange>(
            "SELECT * FROM time_locked_changes WHERE change_id = $1",
        )
        .bind(change_id)
        .fetch_optional(self.db.pool().unwrap())
        .await?;

        let change = match change {
            Some(c) => c,
            None => return Ok(false),
        };

        let override_count = change.override_signals.len();
        let threshold_count = (active_node_count as f64 * self.config.user_override_threshold) as usize;

        Ok(override_count >= threshold_count)
    }

    /// Activate a time-locked change
    pub async fn activate_change(&self, change_id: &str) -> Result<(), sqlx::Error> {
        info!("Activating time-locked change: {}", change_id);

        sqlx::query(
            "UPDATE time_locked_changes SET status = 'activated', updated_at = $1 WHERE change_id = $2",
        )
        .bind(Utc::now())
        .bind(change_id)
        .execute(self.db.pool().unwrap())
        .await?;

        Ok(())
    }

    /// Cancel a time-locked change
    pub async fn cancel_change(&self, change_id: &str) -> Result<(), sqlx::Error> {
        info!("Cancelling time-locked change: {}", change_id);

        sqlx::query(
            "UPDATE time_locked_changes SET status = 'cancelled', updated_at = $1 WHERE change_id = $2",
        )
        .bind(Utc::now())
        .bind(change_id)
        .execute(self.db.pool().unwrap())
        .await?;

        Ok(())
    }

    /// List all pending time locks
    pub async fn list_pending(&self) -> Result<Vec<TimeLockedChange>, sqlx::Error> {
        sqlx::query_as::<_, TimeLockedChange>(
            "SELECT * FROM time_locked_changes WHERE status = 'pending' ORDER BY lock_end ASC",
        )
        .fetch_all(self.db.pool().unwrap())
        .await
    }

    /// Get time lock details
    pub async fn get_change(&self, change_id: &str) -> Result<Option<TimeLockedChange>, sqlx::Error> {
        sqlx::query_as::<_, TimeLockedChange>(
            "SELECT * FROM time_locked_changes WHERE change_id = $1",
        )
        .bind(change_id)
        .fetch_optional(self.db.pool().unwrap())
        .await
    }
}

/// Database migration for time lock tables
pub async fn migrate_time_lock_tables(db: &Database) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS time_locked_changes (
            change_id TEXT PRIMARY KEY,
            tier INTEGER NOT NULL,
            description TEXT NOT NULL,
            pr_number INTEGER,
            lock_start TIMESTAMP WITH TIME ZONE NOT NULL,
            min_duration_hours INTEGER NOT NULL,
            lock_end TIMESTAMP WITH TIME ZONE NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            override_signals JSONB DEFAULT '{}',
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL
        )
        "#,
    )
    .execute(db.pool().unwrap())
    .await?;

    // Create index on status and lock_end for efficient queries
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_time_locked_changes_status ON time_locked_changes(status)",
    )
    .execute(db.pool().unwrap())
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_time_locked_changes_lock_end ON time_locked_changes(lock_end)",
    )
    .execute(db.pool().unwrap())
    .await?;

    Ok(())
}


