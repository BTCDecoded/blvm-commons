use crate::error::GovernanceError;
use chrono::{DateTime, Duration, Utc};

pub struct ReviewPeriodValidator;

impl ReviewPeriodValidator {
    pub fn validate_review_period(
        opened_at: DateTime<Utc>,
        required_days: i64,
        emergency_mode: bool,
    ) -> Result<bool, GovernanceError> {
        let now = Utc::now();
        let elapsed = now - opened_at;

        // Emergency mode does not shorten the clock, and 0-day review is not implemented.
        let _emergency_does_not_shorten = emergency_mode;
        if required_days <= 0 {
            return Err(GovernanceError::ReviewPeriodError(
                "0-day review is not implemented".to_string(),
            ));
        }
        let required_duration = Duration::try_days(required_days).unwrap_or_default();

        if elapsed >= required_duration {
            Ok(true)
        } else {
            let remaining = required_duration - elapsed;
            Err(GovernanceError::ReviewPeriodError(format!(
                "Review period not met. Required: {} days, Elapsed: {} days, Remaining: {} days",
                required_days,
                elapsed.num_days(),
                remaining.num_days()
            )))
        }
    }

    pub fn get_earliest_merge_date(
        opened_at: DateTime<Utc>,
        required_days: i64,
        emergency_mode: bool,
    ) -> DateTime<Utc> {
        let _emergency_does_not_shorten = emergency_mode;
        let days = required_days.max(1);
        let required_duration = Duration::try_days(days).unwrap_or_default();

        opened_at + required_duration
    }

    pub fn get_remaining_days(
        opened_at: DateTime<Utc>,
        required_days: i64,
        emergency_mode: bool,
    ) -> i64 {
        let now = Utc::now();
        let elapsed = now - opened_at;

        let _emergency_does_not_shorten = emergency_mode;
        let days = required_days.max(1);
        let required_duration = Duration::try_days(days).unwrap_or_default();

        let remaining = required_duration - elapsed;
        remaining.num_days().max(0)
    }
}
