//! Nested Multisig Signature Verification
//!
//! Implements team-based signature aggregation for nested 7×7 multisig structure.
//! Groups signatures by team, counts team approvals (6-of-7 per team),
//! then counts inter-team approvals (6-of-7 teams).

use crate::error::GovernanceError;
use std::collections::HashMap;

/// Team structure for nested multisig
#[derive(Debug, Clone)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub maintainers: Vec<TeamMaintainer>,
}

#[derive(Debug, Clone)]
pub struct TeamMaintainer {
    pub github: String,
    pub public_key: String,
}

/// Nested multisig signature verifier
pub struct NestedMultisigVerifier {
    teams: Vec<Team>,
}

impl NestedMultisigVerifier {
    pub fn new(teams: Vec<Team>) -> Self {
        Self { teams }
    }

    /// Verify nested multisig signatures
    ///
    /// Process:
    /// 1. Group signatures by team
    /// 2. Count team approvals (6-of-7 per team required)
    /// 3. Count inter-team approvals (6-of-7 teams required)
    ///
    /// Returns: (teams_approved, teams_required, maintainers_approved, maintainers_required)
    pub fn verify_nested_multisig(
        &self,
        signatures: &[(String, String)], // (github_username, signature)
        tier: u32,
    ) -> Result<NestedMultisigResult, GovernanceError> {
        // Get tier requirements
        let (teams_required, maintainers_per_team_required) = Self::get_tier_requirements(tier)?;

        // Group signatures by team
        let mut team_signatures: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for (github, signature) in signatures {
            // Find which team this maintainer belongs to
            if let Some(team_id) = self.find_maintainer_team(github) {
                team_signatures
                    .entry(team_id)
                    .or_default()
                    .push((github.clone(), signature.clone()));
            }
        }

        // Count team approvals
        let mut teams_approved = 0;
        let mut total_maintainers_approved = 0;
        let mut team_details = Vec::new();

        for team in &self.teams {
            let team_sigs = team_signatures.get(&team.id).map(|v| v.len()).unwrap_or(0);
            let team_approved = team_sigs >= maintainers_per_team_required;

            if team_approved {
                teams_approved += 1;
                total_maintainers_approved += team_sigs;
            }

            team_details.push(TeamApprovalStatus {
                team_id: team.id.clone(),
                team_name: team.name.clone(),
                maintainers_signed: team_sigs,
                maintainers_required: maintainers_per_team_required,
                approved: team_approved,
            });
        }

        let inter_team_approved = teams_approved >= teams_required;
        let total_maintainers_required = teams_required * maintainers_per_team_required;

        Ok(NestedMultisigResult {
            teams_approved,
            teams_required,
            maintainers_approved: total_maintainers_approved,
            maintainers_required: total_maintainers_required,
            inter_team_approved,
            team_details,
        })
    }

    /// Find which team a maintainer belongs to
    fn find_maintainer_team(&self, github: &str) -> Option<String> {
        for team in &self.teams {
            if team.maintainers.iter().any(|m| m.github == github) {
                return Some(team.id.clone());
            }
        }
        None
    }

    /// Nested multisig quorums are not adopted. Refuse 1-of-1 and the old team counts.
    fn get_tier_requirements(tier: u32) -> Result<(usize, usize), GovernanceError> {
        Err(GovernanceError::ThresholdError(format!(
            "Nested multisig quorum for tier {tier} is not adopted; refusing 1-of-1"
        )))
    }
}

/// Result of nested multisig verification
#[derive(Debug, Clone)]
pub struct NestedMultisigResult {
    pub teams_approved: usize,
    pub teams_required: usize,
    pub maintainers_approved: usize,
    pub maintainers_required: usize,
    pub inter_team_approved: bool,
    pub team_details: Vec<TeamApprovalStatus>,
}

#[derive(Debug, Clone)]
pub struct TeamApprovalStatus {
    pub team_id: String,
    pub team_name: String,
    pub maintainers_signed: usize,
    pub maintainers_required: usize,
    pub approved: bool,
}
