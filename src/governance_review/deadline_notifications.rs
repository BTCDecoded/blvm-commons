//! Deadline notification system
//!
//! Notifies maintainers about approaching deadlines for cases, appeals, mediations

use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use crate::governance_review::github_integration::GovernanceReviewGitHubIntegration;
use tracing::{info, warn};

/// Days before deadline to send notification
const NOTIFICATION_DAYS_BEFORE: i64 = 7;

pub struct DeadlineNotificationManager {
    pool: SqlitePool,
    github_integration: Option<GovernanceReviewGitHubIntegration>,
}

impl DeadlineNotificationManager {
    pub fn new(
        pool: SqlitePool,
        github_integration: Option<GovernanceReviewGitHubIntegration>,
    ) -> Self {
        Self {
            pool,
            github_integration,
        }
    }

    /// Check for approaching deadlines and send notifications
    pub async fn check_and_notify(&self) -> Result<DeadlineNotificationResult, sqlx::Error> {
        let mut result = DeadlineNotificationResult::default();

        // Check case deadlines
        let case_deadlines = self.check_case_deadlines().await?;
        result.cases_notified = case_deadlines.len();

        // Check appeal deadlines
        let appeal_deadlines = self.check_appeal_deadlines().await?;
        result.appeals_notified = appeal_deadlines.len();

        // Check mediation deadlines
        let mediation_deadlines = self.check_mediation_deadlines().await?;
        result.mediations_notified = mediation_deadlines.len();

        // Send notifications if GitHub integration available
        if let Some(ref github) = self.github_integration {
            for case_id in &case_deadlines {
                if let Err(e) = self.notify_case_deadline(*case_id, github).await {
                    warn!("Failed to notify case deadline {}: {}", case_id, e);
                }
            }

            for appeal_id in &appeal_deadlines {
                if let Err(e) = self.notify_appeal_deadline(*appeal_id, github).await {
                    warn!("Failed to notify appeal deadline {}: {}", appeal_id, e);
                }
            }

            for mediation_id in &mediation_deadlines {
                if let Err(e) = self.notify_mediation_deadline(*mediation_id, github).await {
                    warn!("Failed to notify mediation deadline {}: {}", mediation_id, e);
                }
            }
        } else {
            info!("GitHub integration not available - notifications logged only");
        }

        Ok(result)
    }

    /// Check for cases with approaching deadlines
    fn check_case_deadlines(&self) -> impl std::future::Future<Output = Result<Vec<i32>, sqlx::Error>> {
        let threshold = Utc::now() + Duration::days(NOTIFICATION_DAYS_BEFORE);

        let rows = sqlx::query(
            r#"
            SELECT id FROM governance_review_cases
            WHERE status NOT IN ('resolved', 'dismissed', 'removed', 'expired')
            AND resolution_deadline IS NOT NULL
            AND resolution_deadline <= ?
            AND resolution_deadline > ?
            "#,
        )
        .bind(threshold)
        .bind(Utc::now())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|row| row.get::<i32, _>(0)).collect())
    }

    /// Check for appeals with approaching deadlines
    async fn check_appeal_deadlines(&self) -> Result<Vec<i32>, sqlx::Error> {
        let threshold = Utc::now() + Duration::days(NOTIFICATION_DAYS_BEFORE);

        let rows = sqlx::query(
            r#"
            SELECT id FROM governance_review_appeals
            WHERE status = 'pending'
            AND appeal_deadline IS NOT NULL
            AND appeal_deadline <= ?
            AND appeal_deadline > ?
            "#,
        )
        .bind(threshold)
        .bind(Utc::now())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|row| row.get::<i32, _>(0)).collect())
    }

    /// Check for mediations with approaching deadlines
    async fn check_mediation_deadlines(&self) -> Result<Vec<i32>, sqlx::Error> {
        let threshold = Utc::now() + Duration::days(NOTIFICATION_DAYS_BEFORE);

        let rows = sqlx::query(
            r#"
            SELECT id FROM governance_review_mediation
            WHERE status = 'active'
            AND mediation_deadline IS NOT NULL
            AND mediation_deadline <= ?
            AND mediation_deadline > ?
            "#,
        )
        .bind(threshold)
        .bind(Utc::now())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|row| row.get::<i32, _>(0)).collect())
    }

    /// Notify about approaching case deadline
    async fn notify_case_deadline(
        &self,
        case_id: i32,
        github: &GovernanceReviewGitHubIntegration,
    ) -> Result<(), sqlx::Error> {
        use crate::governance_review::case::GovernanceReviewCaseManager;
        
        let case_manager = GovernanceReviewCaseManager::new(self.pool.clone());
        let case = case_manager.get_case_by_id(case_id).await?;
        
        if let Some(deadline) = case.resolution_deadline {
            let days_remaining = (deadline - Utc::now()).num_days();
            let message = format!(
                r#"## ⚠️ Deadline Approaching

**Case:** {}
**Deadline:** {} ({} days remaining)

This case must be resolved within the deadline per governance review policy.

---
*Automated notification from Governance Review System*"#,
                case.case_number,
                deadline.format("%Y-%m-%d %H:%M:%S UTC"),
                days_remaining
            );

            // Post comment to GitHub issue if linked
            if let Some(issue_number) = case.github_issue_number {
                if let Err(e) = github.post_issue_comment(
                    issue_number,
                    &message,
                ).await {
                    warn!("Failed to post deadline notification to issue #{}: {}", issue_number, e);
                } else {
                    info!("Posted deadline notification to issue #{} for case {}", issue_number, case.case_number);
                }
            } else {
                // Log notification if no issue linked
                info!("Case deadline notification: {} ({} days remaining) - no GitHub issue linked", case.case_number, days_remaining);
            }
        }

        Ok(())
    }

    /// Notify about approaching appeal deadline
    async fn notify_appeal_deadline(
        &self,
        appeal_id: i32,
        _github: &GovernanceReviewGitHubIntegration,
    ) -> Result<(), sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, case_id, appeal_deadline
            FROM governance_review_appeals
            WHERE id = ?
            "#,
        )
        .bind(appeal_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let case_id: i32 = row.get(1);
            let appeal_deadline: Option<DateTime<Utc>> = row.get(2);
            
            if let Some(deadline) = appeal_deadline {
                let days_remaining = (deadline - Utc::now()).num_days();
                info!("Appeal deadline notification: case {} ({} days remaining)", case_id, days_remaining);
            }
        }

        Ok(())
    }

    /// Notify about approaching mediation deadline
    async fn notify_mediation_deadline(
        &self,
        mediation_id: i32,
        _github: &GovernanceReviewGitHubIntegration,
    ) -> Result<(), sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT 
                id, case_id, mediation_deadline
            FROM governance_review_mediation
            WHERE id = ?
            "#,
        )
        .bind(mediation_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let case_id: i32 = row.get(1);
            let mediation_deadline: Option<DateTime<Utc>> = row.get(2);
            
            if let Some(deadline) = mediation_deadline {
                let days_remaining = (deadline - Utc::now()).num_days();
                info!("Mediation deadline notification: case {} ({} days remaining)", case_id, days_remaining);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct DeadlineNotificationResult {
    pub cases_notified: usize,
    pub appeals_notified: usize,
    pub mediations_notified: usize,
}

