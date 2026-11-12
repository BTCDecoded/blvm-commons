//! H4 and H5: emergency authority is the registered set, unknown
//! layer or tier fails closed, and one maintainer cannot lower a tier.

use blvm_commons::GovernanceError;
use blvm_commons::config::AppConfig;
use blvm_commons::crypto::signatures::SignatureManager;
use blvm_commons::database::Database;
use blvm_commons::governance_review::removal::{RemovalManager, registered_approval_count};
use blvm_commons::validation::emergency::{
    EmergencyActivation, EmergencyTier, EmergencyValidator, KeyholderSignature,
    RegisteredKeyholder, activation_signing_message,
};
use blvm_commons::validation::threshold::ThresholdValidator;
use blvm_commons::webhooks::comment::handle_comment_event;
use blvm_commons::webhooks::pull_request::handle_pull_request_event;
use serde_json::json;

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn evidence() -> String {
    "x".repeat(120)
}

fn activation_shell() -> EmergencyActivation {
    EmergencyActivation {
        tier: EmergencyTier::Critical,
        activated_by: "ops".to_string(),
        reason: "inflation bug".to_string(),
        evidence: evidence(),
        signatures: Vec::new(),
    }
}

/// Five compact signatures. `register` controls whether those same keys are the authority set.
fn five_signatures(register: bool) -> (EmergencyActivation, Vec<RegisteredKeyholder>) {
    let manager = SignatureManager::new();
    let timestamp = EmergencyValidator::calculate_expiration(EmergencyTier::Critical);
    let shell = activation_shell();
    let mut signatures = Vec::new();
    let mut registered = Vec::new();
    for index in 0..5 {
        let keypair = manager.generate_keypair().expect("keypair");
        let name = format!("holder-{index}");
        let public_key = hex_encode(&keypair.public_key);
        if register {
            registered.push(RegisteredKeyholder {
                name: name.clone(),
                public_key_hex: public_key.clone(),
            });
        }
        let mut signature = KeyholderSignature {
            keyholder: name,
            public_key,
            signature: String::new(),
            timestamp,
        };
        let message = activation_signing_message(&shell, &signature).expect("message");
        let message = String::from_utf8(message).expect("utf8");
        signature.signature = manager
            .create_governance_signature(&message, &keypair)
            .expect("sign");
        signatures.push(signature);
    }
    if !register {
        let outsider = manager.generate_keypair().expect("outsider");
        registered.push(RegisteredKeyholder {
            name: "registered-only".to_string(),
            public_key_hex: hex_encode(&outsider.public_key),
        });
    }
    let mut activation = shell;
    activation.signatures = signatures;
    (activation, registered)
}

#[test]
fn caller_supplied_keys_are_not_the_authority_set() {
    let (activation, registered) = five_signatures(false);
    let err = EmergencyValidator::validate_activation(&activation, &registered)
        .expect_err("unregistered keys must not activate");
    match err {
        GovernanceError::SignatureError(message) => {
            assert!(message.contains("not in the registered emergency authority set"));
        }
        other => panic!("expected a signature rejection, got {other:?}"),
    }
}

#[test]
fn registered_compact_signatures_activate() {
    let (activation, registered) = five_signatures(true);
    EmergencyValidator::validate_activation(&activation, &registered)
        .expect("five distinct registered signatures activate");
}

#[test]
fn one_registered_key_does_not_count_as_five() {
    let manager = SignatureManager::new();
    let keypair = manager.generate_keypair().expect("keypair");
    let public_key = hex_encode(&keypair.public_key);
    let timestamp = EmergencyValidator::calculate_expiration(EmergencyTier::Critical);
    let shell = activation_shell();
    let registered: Vec<RegisteredKeyholder> = (0..5)
        .map(|index| RegisteredKeyholder {
            name: format!("alias-{index}"),
            public_key_hex: public_key.clone(),
        })
        .collect();
    let mut signatures = Vec::new();
    for holder in &registered {
        let mut signature = KeyholderSignature {
            keyholder: holder.name.clone(),
            public_key: public_key.clone(),
            signature: String::new(),
            timestamp,
        };
        let message = activation_signing_message(&shell, &signature).expect("message");
        let message = String::from_utf8(message).expect("utf8");
        signature.signature = manager
            .create_governance_signature(&message, &keypair)
            .expect("sign");
        signatures.push(signature);
    }
    let mut activation = shell;
    activation.signatures = signatures;
    let err = EmergencyValidator::validate_activation(&activation, &registered)
        .expect_err("one key cannot meet 5-of-7");
    match err {
        GovernanceError::ValidationError(message) => {
            assert!(message.contains("found 1"));
            assert!(message.contains("required 5"));
        }
        other => panic!("expected an insufficient-signature error, got {other:?}"),
    }
}

#[test]
fn unknown_layer_or_tier_fails_closed() {
    let layer = ThresholdValidator::get_threshold_for_layer(6).expect_err("layer 6");
    assert!(layer.to_string().contains("refusing 1-of-1"));
    let tier = ThresholdValidator::get_tier_threshold(0).expect_err("tier 0");
    assert!(tier.to_string().contains("refusing 1-of-1"));
    let tier = ThresholdValidator::get_tier_threshold(9).expect_err("tier 9");
    assert!(tier.to_string().contains("refusing 1-of-1"));
    assert!(ThresholdValidator::get_combined_requirements(6, 1).is_err());
    assert!(ThresholdValidator::get_review_period_for_layer(6, false).is_err());
    assert!(ThresholdValidator::get_tier_review_period(9).is_err());

    let (required, total) = ThresholdValidator::get_threshold_for_layer(1).expect("layer 1");
    assert_eq!((required, total), (6, 7));
}

#[test]
fn unregistered_removal_approvals_do_not_count() {
    assert_eq!(registered_approval_count(&[1, 2, 3, 4], &[]), 0);
    assert_eq!(registered_approval_count(&[1, 1, 1, 1], &[1]), 1);
}

#[tokio::test]
async fn caller_supplied_emergency_ids_do_not_remove() {
    let database = Database::new_in_memory().await.expect("db");
    database
        .insert_maintainer("alice", "02aa", 1)
        .await
        .expect("maintainer");
    let maintainer = database
        .get_maintainer_by_username("alice")
        .await
        .expect("read")
        .expect("row");
    let removal = RemovalManager::new(database.pool().expect("pool").clone());
    let removed = removal
        .emergency_remove(maintainer.id, vec![1, 2, 3, 4], "caller supplied")
        .await;
    assert!(removed.is_err());
    assert!(
        removal
            .is_maintainer_active(maintainer.id)
            .await
            .expect("active")
    );
}

#[tokio::test]
async fn registered_emergency_keyholders_can_remove() {
    let database = Database::new_in_memory().await.expect("db");
    database
        .insert_maintainer("alice", "02aa", 1)
        .await
        .expect("maintainer");
    let maintainer = database
        .get_maintainer_by_username("alice")
        .await
        .expect("read")
        .expect("row");
    let mut ids = Vec::new();
    for index in 0..4 {
        ids.push(
            database
                .insert_emergency_keyholder(&format!("ek-{index}"), &format!("02{index}"))
                .await
                .expect("keyholder"),
        );
    }
    let removal = RemovalManager::new(database.pool().expect("pool").clone());
    removal
        .emergency_remove(maintainer.id, ids, "registered quorum")
        .await
        .expect("registered 4-of-5 removes");
    assert!(
        !removal
            .is_maintainer_active(maintainer.id)
            .await
            .expect("active")
    );
}

#[tokio::test]
async fn one_maintainer_cannot_lower_a_tier() {
    let database = Database::new_in_memory().await.expect("db");
    database
        .insert_maintainer("alice", "02aa", 1)
        .await
        .expect("maintainer");
    let repo = "BTCDecoded/blvm-spec";
    let raise = json!({
        "repository": {"full_name": repo},
        "issue": {"number": 9},
        "comment": {
            "user": {"login": "alice"},
            "body": "/governance-tier-override 5 \"constitutional change\""
        }
    });
    let raised = handle_comment_event(&database, &raise)
        .await
        .expect("raise");
    assert_eq!(raised.0["status"], "tier_override_set");

    let lower = json!({
        "repository": {"full_name": repo},
        "issue": {"number": 9},
        "comment": {
            "user": {"login": "alice"},
            "body": "/governance-tier-override 2 \"please go faster\""
        }
    });
    let lowered = handle_comment_event(&database, &lower)
        .await
        .expect("lower");
    assert_eq!(lowered.0["status"], "tier_lower_rejected");
    let stored = database
        .get_tier_override(repo, 9)
        .await
        .expect("read")
        .expect("override");
    assert_eq!(stored.override_tier, 5);
}

#[tokio::test]
async fn unknown_layer_does_not_store_a_pull_request() {
    let database = Database::new_in_memory().await.expect("db");
    let config = AppConfig::default();
    let payload = json!({
        "repository": {"full_name": "BTCDecoded/blvm-commons"},
        "pull_request": {
            "number": 4,
            "title": "Fix typo in README",
            "body": "docs",
            "head": {"sha": "abc123"}
        }
    });
    let err = handle_pull_request_event(&config, &database, &payload)
        .await
        .expect_err("layer 6 fails closed");
    assert_eq!(err.as_u16(), 422);
    assert!(
        database
            .get_pull_request("BTCDecoded/blvm-commons", 4)
            .await
            .expect("read")
            .is_none()
    );

    let known = json!({
        "repository": {"full_name": "BTCDecoded/blvm-spec"},
        "pull_request": {
            "number": 4,
            "title": "Fix typo in README",
            "body": "docs",
            "head": {"sha": "abc123"}
        }
    });
    handle_pull_request_event(&config, &database, &known)
        .await
        .expect("layer 1 stores");
    assert!(
        database
            .get_pull_request("BTCDecoded/blvm-spec", 4)
            .await
            .expect("read")
            .is_some()
    );
}
