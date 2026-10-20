//! H1 and H2: the verifier uses compact secp256k1, and the message is bound
//! to repo, pull-request number, and head commit. `sign-pr` itself is checked
//! by the binary test `sign_pr_output_verifies`.

use blvm_commons::crypto::signatures::{SignatureManager, governance_pr_message};
use blvm_commons::database::Database;

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

#[test]
fn bad_signature_is_rejected() {
    let manager = SignatureManager::new();
    let keypair = manager.generate_keypair().unwrap();
    let message = governance_pr_message("BTCDecoded/blvm-spec", 4, "abc");
    let signature = manager
        .create_governance_signature(&message, &keypair)
        .unwrap();
    let mut chars = signature.into_bytes();
    chars[0] = if chars[0] == b'a' { b'b' } else { b'a' };
    let signature = String::from_utf8(chars).unwrap();
    let public_key = hex_encode(&keypair.public_key);
    let verified = manager
        .verify_governance_signature(&message, &signature, &public_key)
        .unwrap();
    assert!(!verified);
}

#[test]
fn signature_over_a_different_head_is_rejected() {
    let manager = SignatureManager::new();
    let keypair = manager.generate_keypair().unwrap();
    let signed = governance_pr_message("BTCDecoded/blvm-spec", 4, "head-a");
    let other = governance_pr_message("BTCDecoded/blvm-spec", 4, "head-b");
    let signature = manager
        .create_governance_signature(&signed, &keypair)
        .unwrap();
    let public_key = hex_encode(&keypair.public_key);
    assert!(
        manager
            .verify_governance_signature(&signed, &signature, &public_key)
            .unwrap()
    );
    assert!(
        !manager
            .verify_governance_signature(&other, &signature, &public_key)
            .unwrap()
    );
    assert_eq!(signature.len(), 128, "compact signature is 64 bytes of hex");
}

#[tokio::test]
async fn rewritten_head_drops_stored_signatures_and_keeps_opened_at() {
    let db = Database::new_in_memory().await.unwrap();
    db.create_pull_request("BTCDecoded/blvm-spec", 4, "head-a", 1)
        .await
        .unwrap();
    db.add_signature("BTCDecoded/blvm-spec", 4, "alice", "aa", None)
        .await
        .unwrap();
    let opened = db
        .get_pull_request("BTCDecoded/blvm-spec", 4)
        .await
        .unwrap()
        .unwrap()
        .opened_at;

    db.create_pull_request("BTCDecoded/blvm-spec", 4, "head-b", 1)
        .await
        .unwrap();
    let rewritten = db
        .get_pull_request("BTCDecoded/blvm-spec", 4)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rewritten.head_sha, "head-b");
    assert!(rewritten.signatures.is_empty());
    assert_eq!(rewritten.opened_at, opened);

    db.add_signature("BTCDecoded/blvm-spec", 4, "alice", "bb", None)
        .await
        .unwrap();
    db.create_pull_request("BTCDecoded/blvm-spec", 4, "head-b", 1)
        .await
        .unwrap();
    let same_head = db
        .get_pull_request("BTCDecoded/blvm-spec", 4)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(same_head.signatures.len(), 1);
    assert_eq!(same_head.opened_at, opened);
}
