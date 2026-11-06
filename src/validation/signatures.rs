use crate::crypto::signatures::SignatureManager;
use crate::error::GovernanceError;
use crate::validation::nested_multisig::{NestedMultisigVerifier, Team};

pub struct SignatureValidator {
    nested_multisig_verifier: Option<NestedMultisigVerifier>,
}

impl SignatureValidator {
    pub fn new() -> Self {
        Self {
            nested_multisig_verifier: None,
        }
    }

    /// Create with nested multisig support (for Tier 3+)
    pub fn with_nested_multisig(teams: Vec<Team>) -> Self {
        Self {
            nested_multisig_verifier: Some(NestedMultisigVerifier::new(teams)),
        }
    }

    pub fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        public_key: &str,
    ) -> Result<bool, GovernanceError> {
        // Same compact secp256k1 check as the webhook. DER and Ed25519 are not accepted.
        SignatureManager::new().verify_governance_signature(message, signature, public_key)
    }

    pub fn verify_multisig_threshold(
        &self,
        message: &str,
        signatures: &[(String, String)],    // (signer, signature)
        required_threshold: (usize, usize), // (required, total)
        maintainer_keys: &std::collections::HashMap<String, String>, // username -> public_key
        tier: Option<u32>,                  // Optional tier for nested multisig
    ) -> Result<bool, GovernanceError> {
        // For Tier 3+, use nested multisig if available
        if let Some(tier_val) = tier {
            if tier_val >= 3 {
                if let Some(ref nested_verifier) = self.nested_multisig_verifier {
                    // Verify signatures first, then use nested multisig
                    let mut verified_sigs = Vec::new();
                    for (signer, signature) in signatures {
                        if let Some(public_key) = maintainer_keys.get(signer) {
                            if self.verify_signature(message, signature, public_key)? {
                                verified_sigs.push((signer.clone(), signature.clone()));
                            }
                        }
                    }

                    // Use nested multisig verification
                    let result =
                        nested_verifier.verify_nested_multisig(&verified_sigs, tier_val)?;
                    return Ok(result.inter_team_approved);
                }
            }
        }

        // Fall back to simple multisig for Tier 1-2 or if nested multisig not available
        let (required, _total) = required_threshold;
        let mut valid_signatures = 0;

        for (signer, signature) in signatures {
            if let Some(public_key) = maintainer_keys.get(signer) {
                if self.verify_signature(message, signature, public_key)? {
                    valid_signatures += 1;
                }
            }
        }

        Ok(valid_signatures >= required)
    }
}

impl Default for SignatureValidator {
    fn default() -> Self {
        Self::new()
    }
}
