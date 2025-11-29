//! Voting Key Derivation for Privacy-Preserving Veto Signals
//!
//! Uses BIP32 key derivation to create separate voting keys from registration keys,
//! enabling privacy-preserving veto signals where votes cannot be linked to registration.

use bllvm_sdk::governance::bip32::{derive_child_private, ExtendedPrivateKey, ExtendedPublicKey};
use secp256k1::{PublicKey, SecretKey, Secp256k1};
use std::str::FromStr;

use crate::error::GovernanceError;

/// Derive a voting key from a registration key for a specific PR and signal index
///
/// Derivation path: m/0'/pr_id'/signal_index'
/// - 0' = hardened index 0 (governance voting)
/// - pr_id' = hardened PR ID (prevents cross-PR key linking)
/// - signal_index' = hardened signal index (prevents cross-signal linking)
///
/// All hardened derivation ensures privacy: voting keys cannot be derived from public key alone.
pub fn derive_voting_key(
    registration_private_key: &SecretKey,
    registration_chain_code: &[u8; 32],
    pr_id: i32,
    signal_index: u32,
) -> Result<(ExtendedPrivateKey, ExtendedPublicKey), GovernanceError> {
    use bllvm_sdk::governance::bip32::ExtendedPrivateKey;

    // Create extended private key from registration key
    // We treat the registration key as the master key for voting derivation
    let secp = Secp256k1::new();
    let registration_public_key = PublicKey::from_secret_key(&secp, registration_private_key);

    // Create extended key structure (depth 0, treating registration key as master)
    let registration_xprv = ExtendedPrivateKey {
        depth: 0,
        parent_fingerprint: [0u8; 4],
        child_number: 0,
        chain_code: *registration_chain_code,
        private_key: *registration_private_key,
    };

    // Derive path: m/0'/pr_id'/signal_index'
    // Step 1: Derive m/0' (hardened, governance voting)
    let (gov_xprv, _gov_xpub) = derive_child_private(
        &registration_xprv,
        0x80000000, // Hardened index 0
    )?;

    // Step 2: Derive m/0'/pr_id' (hardened, PR-specific)
    let pr_id_hardened = 0x80000000u32
        .checked_add(pr_id as u32)
        .ok_or_else(|| GovernanceError::CryptoError("PR ID overflow".to_string()))?;
    let (pr_xprv, _pr_xpub) = derive_child_private(&gov_xprv, pr_id_hardened)?;

    // Step 3: Derive m/0'/pr_id'/signal_index' (hardened, signal-specific)
    let signal_index_hardened = 0x80000000u32
        .checked_add(signal_index)
        .ok_or_else(|| GovernanceError::CryptoError("Signal index overflow".to_string()))?;
    let (voting_xprv, voting_xpub) = derive_child_private(&pr_xprv, signal_index_hardened)?;

    Ok((voting_xprv, voting_xpub))
}

/// Verify that a voting public key is derived from a registration public key
///
/// This is used to verify that a voting key belongs to a registered node
/// without requiring the private key (public key verification).
///
/// Note: For hardened derivation, we cannot verify from public key alone.
/// This function requires the registration private key for verification.
/// In practice, the node proves ownership by signing with the voting key,
/// and we verify the voting key is derived from their registered key.
pub fn verify_voting_key_derivation(
    registration_private_key: &SecretKey,
    registration_chain_code: &[u8; 32],
    voting_public_key: &PublicKey,
    pr_id: i32,
    signal_index: u32,
) -> Result<bool, GovernanceError> {
    // Derive the voting key
    let (derived_voting_xprv, derived_voting_xpub) = derive_voting_key(
        registration_private_key,
        registration_chain_code,
        pr_id,
        signal_index,
    )?;

    // Compare derived public key with provided voting public key
    Ok(derived_voting_xpub.public_key == *voting_public_key)
}

/// Get voting key derivation path as string (for display/debugging)
pub fn get_voting_key_path(pr_id: i32, signal_index: u32) -> String {
    format!("m/0'/{pr_id}'/{signal_index}'")
}

/// Parse chain code from hex string (for storage/retrieval)
pub fn chain_code_from_hex(hex: &str) -> Result<[u8; 32], GovernanceError> {
    let bytes = hex::decode(hex)
        .map_err(|e| GovernanceError::CryptoError(format!("Invalid hex: {}", e)))?;
    if bytes.len() != 32 {
        return Err(GovernanceError::CryptoError(
            "Chain code must be 32 bytes".to_string(),
        ));
    }
    let mut chain_code = [0u8; 32];
    chain_code.copy_from_slice(&bytes);
    Ok(chain_code)
}

/// Convert chain code to hex string (for storage)
pub fn chain_code_to_hex(chain_code: &[u8; 32]) -> String {
    hex::encode(chain_code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::rand::rngs::OsRng;

    #[test]
    fn test_voting_key_derivation() {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let registration_key = SecretKey::new(&mut rng);
        let chain_code = [0x42u8; 32]; // Test chain code

        let pr_id = 123;
        let signal_index = 0;

        let (voting_xprv, voting_xpub) =
            derive_voting_key(&registration_key, &chain_code, pr_id, signal_index).unwrap();

        // Verify the voting key is different from registration key
        assert_ne!(
            voting_xprv.private_key, registration_key,
            "Voting key should be different from registration key"
        );

        // Verify derivation path
        let path = get_voting_key_path(pr_id, signal_index);
        assert_eq!(path, "m/0'/123'/0'");
    }

    #[test]
    fn test_voting_key_verification() {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let registration_key = SecretKey::new(&mut rng);
        let chain_code = [0x42u8; 32];

        let pr_id = 456;
        let signal_index = 1;

        let (voting_xprv, voting_xpub) =
            derive_voting_key(&registration_key, &chain_code, pr_id, signal_index).unwrap();

        // Verify the voting key is correctly derived
        let verified = verify_voting_key_derivation(
            &registration_key,
            &chain_code,
            &voting_xpub.public_key,
            pr_id,
            signal_index,
        )
        .unwrap();

        assert!(verified, "Voting key verification should succeed");
    }

    #[test]
    fn test_different_prs_produce_different_keys() {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let registration_key = SecretKey::new(&mut rng);
        let chain_code = [0x42u8; 32];

        let (voting_key_1, _) =
            derive_voting_key(&registration_key, &chain_code, 100, 0).unwrap();
        let (voting_key_2, _) =
            derive_voting_key(&registration_key, &chain_code, 200, 0).unwrap();

        // Different PRs should produce different voting keys
        assert_ne!(
            voting_key_1.private_key, voting_key_2.private_key,
            "Different PRs should produce different voting keys"
        );
    }

    #[test]
    fn test_different_signal_indices_produce_different_keys() {
        let secp = Secp256k1::new();
        let mut rng = OsRng;
        let registration_key = SecretKey::new(&mut rng);
        let chain_code = [0x42u8; 32];

        let (voting_key_1, _) =
            derive_voting_key(&registration_key, &chain_code, 100, 0).unwrap();
        let (voting_key_2, _) =
            derive_voting_key(&registration_key, &chain_code, 100, 1).unwrap();

        // Different signal indices should produce different voting keys
        assert_ne!(
            voting_key_1.private_key, voting_key_2.private_key,
            "Different signal indices should produce different voting keys"
        );
    }
}

