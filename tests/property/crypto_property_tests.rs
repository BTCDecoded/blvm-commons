//! Property-based tests for cryptographic functions
//!
//! These tests verify mathematical properties of signature and multisig operations.

use bllvm_commons::crypto::signatures::SignatureManager;
use bllvm_commons::crypto::multisig::MultisigManager;
use proptest::prelude::*;
use secp256k1::{PublicKey, SecretKey, Secp256k1};
use std::collections::HashMap;

proptest! {
    /// Property: Signature creation and verification round-trip
    #[test]
    fn test_signature_round_trip(
        message in prop::string::string_regex(".*").unwrap()
    ) {
        let manager = SignatureManager::new();
        let secp = Secp256k1::new();
        let mut rng = proptest::test_runner::TestRng::deterministic_rng(proptest::test_runner::RngAlgorithm::ChaCha);
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        let signature = manager.create_signature(&message, &secret_key).unwrap();
        let verified = manager.verify_signature(&message, &signature, &public_key).unwrap();
        
        prop_assert!(verified, "Signature should verify for original message");
    }

    /// Property: Signature verification fails for different messages
    #[test]
    fn test_signature_different_message(
        message1 in prop::string::string_regex(".*").unwrap(),
        message2 in prop::string::string_regex(".*").unwrap()
    ) {
        prop_assume!(message1 != message2);
        
        let manager = SignatureManager::new();
        let secp = Secp256k1::new();
        let mut rng = proptest::test_runner::TestRng::deterministic_rng(proptest::test_runner::RngAlgorithm::ChaCha);
        let secret_key = SecretKey::new(&mut rng);
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        let signature = manager.create_signature(&message1, &secret_key).unwrap();
        let verified = manager.verify_signature(&message2, &signature, &public_key).unwrap();
        
        prop_assert!(!verified, "Signature should not verify for different message");
    }

    /// Property: Multisig threshold verification
    #[test]
    fn test_multisig_threshold(
        required in 1usize..10,
        total in 1usize..10,
        provided in 0usize..10
    ) {
        prop_assume!(required <= total);
        prop_assume!(provided <= total);
        
        let manager = MultisigManager::new();
        let sig_manager = SignatureManager::new();
        let secp = Secp256k1::new();
        let mut rng = proptest::test_runner::TestRng::deterministic_rng(proptest::test_runner::RngAlgorithm::ChaCha);
        
        let message = "test message";
        let mut public_keys = HashMap::new();
        let mut signatures = Vec::new();
        
        for i in 0..total {
            let secret_key = SecretKey::new(&mut rng);
            let public_key = PublicKey::from_secret_key(&secp, &secret_key);
            let public_key_str = public_key.to_string();
            public_keys.insert(format!("signer{}", i), public_key_str);
            
            if i < provided {
                let sig = sig_manager.create_signature(message, &secret_key).unwrap();
                signatures.push((format!("signer{}", i), sig.to_string()));
            }
        }
        
        let result = manager.verify_multisig(message, &signatures, &public_keys, (required, total));
        
        if provided >= required {
            prop_assert!(result.is_ok() && result.unwrap(), 
                "Should pass when provided ({}) >= required ({})", provided, required);
        } else {
            prop_assert!(result.is_err(), 
                "Should fail when provided ({}) < required ({})", provided, required);
        }
    }

    /// Property: Public key derivation is deterministic
    #[test]
    fn test_public_key_determinism(
        secret_bytes in prop::array::uniform32(any::<u8>())
    ) {
        // Only test valid secret keys (not all 32-byte arrays are valid)
        if let Ok(secret_key) = SecretKey::from_slice(&secret_bytes) {
            let manager = SignatureManager::new();
            let public_key1 = manager.public_key_from_secret(&secret_key);
            let public_key2 = manager.public_key_from_secret(&secret_key);
            
            prop_assert_eq!(public_key1, public_key2, "Public key derivation should be deterministic");
        }
    }
}

