use crate::database::models::*;
use crate::error::GovernanceError;
use chrono::Utc;
use sqlx::{SqlitePool, Row, FromRow};

pub struct Queries;

impl Queries {
    pub async fn get_pull_request(
        pool: &SqlitePool,
        repo_name: &str,
        pr_number: i32,
    ) -> Result<Option<PullRequest>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT 
                id,
                repo_name,
                pr_number,
                opened_at,
                layer,
                head_sha,
                signatures,
                governance_status,
                linked_prs,
                emergency_mode,
                created_at,
                updated_at
            FROM pull_requests
            WHERE repo_name = ? AND pr_number = ?
            "#,
        )
        .bind(repo_name)
        .bind(pr_number)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                let pr_row = PullRequestRow::from_row(&r)?;
                Ok(Some(pr_row.into()))
            }
            None => Ok(None),
        }
    }

    pub async fn get_maintainers_for_layer(
        pool: &SqlitePool,
        layer: i32,
    ) -> Result<Vec<Maintainer>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                github_username,
                public_key,
                layer,
                active,
                last_updated
            FROM maintainers
            WHERE layer = ? AND active = true
            ORDER BY github_username
            "#,
        )
        .bind(layer)
        .fetch_all(pool)
        .await?;

        let result: Result<Vec<Maintainer>, _> = rows
            .into_iter()
            .map(|r| {
                let mr = MaintainerRow::from_row(&r)?;
                Ok(mr.into())
            })
            .collect();
        result
    }

    pub async fn get_emergency_keyholders(
        pool: &SqlitePool,
    ) -> Result<Vec<EmergencyKeyholder>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                github_username,
                public_key,
                active,
                last_updated
            FROM emergency_keyholders
            WHERE active = true
            ORDER BY github_username
            "#,
        )
        .fetch_all(pool)
        .await?;

        let result: Result<Vec<EmergencyKeyholder>, _> = rows
            .into_iter()
            .map(|r| {
                let ekr = EmergencyKeyholderRow::from_row(&r)?;
                Ok(ekr.into())
            })
            .collect();
        result
    }

    pub async fn get_governance_events(
        pool: &SqlitePool,
        limit: i64,
    ) -> Result<Vec<GovernanceEvent>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                event_type,
                repo_name,
                pr_number,
                maintainer,
                details,
                timestamp
            FROM governance_events
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let result: Result<Vec<GovernanceEvent>, _> = rows
            .into_iter()
            .map(|r| {
                let ger = GovernanceEventRow::from_row(&r)?;
                Ok(ger.into())
            })
            .collect();
        result
    }

    pub async fn create_pull_request(
        pool: &SqlitePool,
        repo_name: &str,
        pr_number: i32,
        head_sha: &str,
        layer: i32,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO pull_requests 
                (repo_name, pr_number, opened_at, layer, head_sha, signatures, governance_status, linked_prs, emergency_mode, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, '[]', 'pending', '[]', 0, ?, ?)
            ON CONFLICT(repo_name, pr_number) DO UPDATE SET
                head_sha = excluded.head_sha,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(repo_name)
        .bind(pr_number)
        .bind(now)
        .bind(layer)
        .bind(head_sha)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn add_signature(
        pool: &SqlitePool,
        repo_name: &str,
        pr_number: i32,
        signer: &str,
        signature: &str,
    ) -> Result<(), sqlx::Error> {
        // Get existing PR to update signatures
        let pr = Self::get_pull_request(pool, repo_name, pr_number).await?;
        if let Some(pr) = pr {
            // Parse existing signatures - signatures field is already Vec<Signature> in the model
            let mut signatures = pr.signatures;

            // Add new signature
            let new_sig = Signature {
                signer: signer.to_string(),
                signature: signature.to_string(),
                timestamp: Utc::now(),
                reasoning: None,
            };
            signatures.push(new_sig);

            // Update PR with new signatures
            let signatures_json = serde_json::to_string(&signatures)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            let now = Utc::now();

            sqlx::query(
                r#"
                UPDATE pull_requests
                SET signatures = ?, updated_at = ?
                WHERE repo_name = ? AND pr_number = ?
                "#,
            )
            .bind(&signatures_json)
            .bind(now)
            .bind(repo_name)
            .bind(pr_number)
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    pub async fn log_governance_event(
        pool: &SqlitePool,
        event_type: &str,
        repo_name: Option<String>,
        pr_number: Option<i32>,
        maintainer: Option<String>,
        details: serde_json::Value,
    ) -> Result<(), sqlx::Error> {
        let details_json = serde_json::to_string(&details)
            .map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        sqlx::query(
            r#"
            INSERT INTO governance_events 
                (event_type, repo_name, pr_number, maintainer, details, timestamp)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(event_type)
        .bind(repo_name)
        .bind(pr_number)
        .bind(maintainer)
        .bind(&details_json)
        .bind(Utc::now())
        .execute(pool)
        .await?;

        Ok(())
    }
}

// Helper structs for SQLx FromRow
#[derive(Debug)]
struct PullRequestRow {
    id: i32,
    repo_name: String,
    pr_number: i32,
    opened_at: chrono::DateTime<Utc>,
    layer: i32,
    head_sha: String,
    signatures: String, // JSON string
    governance_status: String,
    linked_prs: String, // JSON string
    emergency_mode: bool,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for PullRequestRow {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(PullRequestRow {
            id: row.get(0),
            repo_name: row.get(1),
            pr_number: row.get(2),
            opened_at: row.get(3),
            layer: row.get(4),
            head_sha: row.get(5),
            signatures: row.get(6),
            governance_status: row.get(7),
            linked_prs: row.get(8),
            emergency_mode: row.get::<i32, _>(9) != 0,
            created_at: row.get(10),
            updated_at: row.get(11),
        })
    }
}

impl From<PullRequestRow> for PullRequest {
    fn from(row: PullRequestRow) -> Self {
        let signatures: Vec<Signature> = serde_json::from_str(&row.signatures)
            .unwrap_or_else(|_| vec![]);
        let linked_prs: Vec<i32> = serde_json::from_str(&row.linked_prs)
            .unwrap_or_else(|_| vec![]);

        PullRequest {
            id: row.id,
            repo_name: row.repo_name,
            pr_number: row.pr_number,
            opened_at: row.opened_at,
            layer: row.layer,
            head_sha: row.head_sha,
            signatures,
            governance_status: row.governance_status,
            linked_prs,
            emergency_mode: row.emergency_mode,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug)]
struct MaintainerRow {
    id: i32,
    github_username: String,
    public_key: String,
    layer: i32,
    active: bool,
    last_updated: chrono::DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for MaintainerRow {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(MaintainerRow {
            id: row.get(0),
            github_username: row.get(1),
            public_key: row.get(2),
            layer: row.get(3),
            active: row.get::<i32, _>(4) != 0,
            last_updated: row.get(5),
        })
    }
}

impl From<MaintainerRow> for Maintainer {
    fn from(row: MaintainerRow) -> Self {
        Maintainer {
            id: row.id,
            github_username: row.github_username,
            public_key: row.public_key,
            layer: row.layer,
            active: row.active,
            last_updated: row.last_updated,
        }
    }
}

#[derive(Debug)]
struct EmergencyKeyholderRow {
    id: i32,
    github_username: String,
    public_key: String,
    active: bool,
    last_updated: chrono::DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for EmergencyKeyholderRow {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(EmergencyKeyholderRow {
            id: row.get(0),
            github_username: row.get(1),
            public_key: row.get(2),
            active: row.get::<i32, _>(3) != 0,
            last_updated: row.get(4),
        })
    }
}

impl From<EmergencyKeyholderRow> for EmergencyKeyholder {
    fn from(row: EmergencyKeyholderRow) -> Self {
        EmergencyKeyholder {
            id: row.id,
            github_username: row.github_username,
            public_key: row.public_key,
            active: row.active,
            last_updated: row.last_updated,
        }
    }
}

#[derive(Debug)]
struct GovernanceEventRow {
    id: i32,
    event_type: String,
    repo_name: Option<String>,
    pr_number: Option<i32>,
    maintainer: Option<String>,
    details: String, // JSON string
    timestamp: chrono::DateTime<Utc>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for GovernanceEventRow {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(GovernanceEventRow {
            id: row.get(0),
            event_type: row.get(1),
            repo_name: row.get(2),
            pr_number: row.get(3),
            maintainer: row.get(4),
            details: row.get(5),
            timestamp: row.get(6),
        })
    }
}

impl From<GovernanceEventRow> for GovernanceEvent {
    fn from(row: GovernanceEventRow) -> Self {
        let details: serde_json::Value = serde_json::from_str(&row.details)
            .unwrap_or_else(|_| serde_json::json!({}));

        GovernanceEvent {
            id: row.id,
            event_type: row.event_type,
            repo_name: row.repo_name,
            pr_number: row.pr_number,
            maintainer: row.maintainer,
            details,
            timestamp: row.timestamp,
        }
    }
}
