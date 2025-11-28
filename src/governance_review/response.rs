//! Response handling for governance review cases
//!
//! Allows subjects to respond to cases with their side of the story
//! and counter-evidence

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use crate::governance_review::models::GovernanceReviewResponse;

pub struct ResponseManager {
    pool: SqlitePool,
}

impl ResponseManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Submit a response to a case
    /// Policy: Subject has 30 days to respond
    pub async fn submit_response(
        &self,
        case_id: i32,
        maintainer_id: i32,
        response_text: &str,
        counter_evidence: serde_json::Value,
    ) -> Result<GovernanceReviewResponse, sqlx::Error> {
        // Verify maintainer is the subject of the case
        let case_subject: i32 = sqlx::query_scalar(
            "SELECT subject_maintainer_id FROM governance_review_cases WHERE id = ?"
        )
        .bind(case_id)
        .fetch_one(&self.pool)
        .await?;

        if case_subject != maintainer_id {
            return Err(sqlx::Error::RowNotFound); // Or custom error type
        }

        // Check if response deadline has passed
        let response_deadline: Option<DateTime<Utc>> = sqlx::query_scalar(
            "SELECT response_deadline FROM governance_review_cases WHERE id = ?"
        )
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        if let Some(deadline) = response_deadline {
            if Utc::now() > deadline {
                return Err(sqlx::Error::RowNotFound); // Or custom error type for deadline passed
            }
        }

        let counter_evidence_json = serde_json::to_string(&counter_evidence)?;
        let submitted_at = Utc::now();

        let response_id: i32 = sqlx::query_scalar(
            r#"
            INSERT INTO governance_review_responses
            (case_id, maintainer_id, response_text, counter_evidence, submitted_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(case_id)
        .bind(maintainer_id)
        .bind(response_text)
        .bind(&counter_evidence_json)
        .bind(submitted_at)
        .fetch_one(&self.pool)
        .await?
        .get(0);

        self.get_response_by_id(response_id).await
    }

    /// Get response by ID
    pub async fn get_response_by_id(&self, response_id: i32) -> Result<GovernanceReviewResponse, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, case_id, maintainer_id, response_text, counter_evidence, submitted_at
            FROM governance_review_responses
            WHERE id = ?
            "#,
        )
        .bind(response_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(GovernanceReviewResponse {
            id: row.get(0),
            case_id: row.get(1),
            maintainer_id: row.get(2),
            response_text: row.get(3),
            counter_evidence: serde_json::from_str(row.get::<String, _>(4).as_str()).unwrap_or_default(),
            submitted_at: row.get(5),
        })
    }

    /// Get responses for a case
    pub async fn get_responses_for_case(&self, case_id: i32) -> Result<Vec<GovernanceReviewResponse>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, case_id, maintainer_id, response_text, counter_evidence, submitted_at
            FROM governance_review_responses
            WHERE case_id = ?
            ORDER BY submitted_at DESC
            "#,
        )
        .bind(case_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(GovernanceReviewResponse {
                    id: row.get(0),
                    case_id: row.get(1),
                    maintainer_id: row.get(2),
                    response_text: row.get(3),
                    counter_evidence: serde_json::from_str(row.get::<String, _>(4).as_str()).unwrap_or_default(),
                    submitted_at: row.get(5),
                })
            })
            .collect()
    }
}

