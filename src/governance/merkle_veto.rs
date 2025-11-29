//! Merkle Tree for Privacy-Preserving Veto Signal Proofs
//!
//! Builds a Merkle tree of counted veto signals, allowing nodes to prove their signal
//! was counted without revealing which node sent it.

use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::economic_nodes::types::VetoSignal;
use crate::error::GovernanceError;

/// Merkle tree node
#[derive(Debug, Clone)]
struct MerkleNode {
    hash: [u8; 32],
    left: Option<Box<MerkleNode>>,
    right: Option<Box<MerkleNode>>,
}

impl MerkleNode {
    fn new_leaf(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize().into();
        Self {
            hash,
            left: None,
            right: None,
        }
    }

    fn new_internal(left: MerkleNode, right: MerkleNode) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&left.hash);
        hasher.update(&right.hash);
        let hash = hasher.finalize().into();
        Self {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    fn root_hash(&self) -> [u8; 32] {
        self.hash
    }
}

/// Merkle proof path (for proving a signal was included)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Root hash of the Merkle tree
    pub root_hash: String,
    /// Path from leaf to root (each element is (hash, is_left))
    pub path: Vec<(String, bool)>,
    /// Leaf data (voting public key + signal hash)
    pub leaf_data: String,
}

/// Build a Merkle tree from veto signals for a specific PR
///
/// Each leaf is: SHA256(voting_public_key || signal_hash)
/// where signal_hash = SHA256(pr_id || signal_type || weight || timestamp)
pub fn build_veto_merkle_tree(signals: &[VetoSignal]) -> Result<([u8; 32], HashMap<String, MerkleProof>), GovernanceError> {
    if signals.is_empty() {
        return Err(GovernanceError::CryptoError(
            "Cannot build Merkle tree from empty signal list".to_string(),
        ));
    }

    // Build leaves: hash(voting_public_key || signal_hash)
    let mut leaves = Vec::new();
    let mut signal_map = HashMap::new();

    for signal in signals {
        // Signal hash: pr_id || signal_type || weight || timestamp
        let signal_data = format!(
            "{}:{}:{}:{}",
            signal.pr_id,
            signal.signal_type.as_str(),
            signal.weight,
            signal.timestamp.to_rfc3339()
        );
        let mut signal_hasher = Sha256::new();
        signal_hasher.update(signal_data.as_bytes());
        let signal_hash = signal_hasher.finalize();

        // Leaf data: voting_public_key || signal_hash
        let voting_key = signal
            .voting_public_key
            .as_ref()
            .ok_or_else(|| {
                GovernanceError::CryptoError(
                    "Signal missing voting_public_key".to_string(),
                )
            })?;
        let leaf_data = format!("{}:{}", voting_key, hex::encode(signal_hash));
        let leaf = MerkleNode::new_leaf(leaf_data.as_bytes());
        leaves.push(leaf);
        signal_map.insert(voting_key.clone(), (leaf_data, signal_hash.to_vec()));
    }

    // Build tree bottom-up
    let root = build_tree(leaves)?;
    let root_hash = root.root_hash();

    // Generate proofs for each signal
    let mut proofs = HashMap::new();
    for (voting_key, (leaf_data, _signal_hash)) in signal_map {
        let proof = generate_proof(&root, &leaf_data, &voting_key)?;
        proofs.insert(voting_key, proof);
    }

    Ok((root_hash, proofs))
}

/// Build Merkle tree from leaves
fn build_tree(mut leaves: Vec<MerkleNode>) -> Result<MerkleNode, GovernanceError> {
    if leaves.is_empty() {
        return Err(GovernanceError::CryptoError(
            "Cannot build tree from empty leaves".to_string(),
        ));
    }

    // If only one leaf, duplicate it (Bitcoin-style)
    if leaves.len() == 1 {
        let leaf = leaves.remove(0);
        return Ok(MerkleNode::new_internal(leaf.clone(), leaf));
    }

    // Build tree level by level
    while leaves.len() > 1 {
        let mut next_level = Vec::new();
        let mut i = 0;
        while i < leaves.len() {
            if i + 1 < leaves.len() {
                // Pair of nodes
                let left = leaves.remove(i);
                let right = leaves.remove(i);
                next_level.push(MerkleNode::new_internal(left, right));
            } else {
                // Odd node, duplicate it (Bitcoin-style)
                let node = leaves.remove(i);
                next_level.push(MerkleNode::new_internal(node.clone(), node));
            }
        }
        leaves = next_level;
    }

    Ok(leaves.remove(0))
}

/// Generate Merkle proof for a specific leaf
fn generate_proof(
    root: &MerkleNode,
    leaf_data: &str,
    voting_key: &str,
) -> Result<MerkleProof, GovernanceError> {
    let mut path = Vec::new();
    let leaf_hash = {
        let mut hasher = Sha256::new();
        hasher.update(leaf_data.as_bytes());
        hasher.finalize().into()
    };

    // Traverse tree to find leaf and build proof path
    if !find_leaf_and_build_path(root, &leaf_hash, &mut path) {
        return Err(GovernanceError::CryptoError(
            "Leaf not found in Merkle tree".to_string(),
        ));
    }

    Ok(MerkleProof {
        root_hash: hex::encode(root.root_hash()),
        path,
        leaf_data: leaf_data.to_string(),
    })
}

/// Recursively find leaf and build proof path
fn find_leaf_and_build_path(
    node: &MerkleNode,
    target_hash: &[u8; 32],
    path: &mut Vec<(String, bool)>,
) -> bool {
    // Leaf node
    if node.left.is_none() && node.right.is_none() {
        return node.hash == *target_hash;
    }

    // Internal node - check left and right
    if let (Some(ref left), Some(ref right)) = (&node.left, &node.right) {
        if find_leaf_and_build_path(left, target_hash, path) {
            // Found in left subtree, add right hash to path
            path.push((hex::encode(right.hash), false)); // false = not left (i.e., right)
            return true;
        }
        if find_leaf_and_build_path(right, target_hash, path) {
            // Found in right subtree, add left hash to path
            path.push((hex::encode(left.hash), true)); // true = left
            return true;
        }
    }

    false
}

/// Verify a Merkle proof
pub fn verify_merkle_proof(proof: &MerkleProof) -> bool {
    // Reconstruct leaf hash
    let mut hasher = Sha256::new();
    hasher.update(proof.leaf_data.as_bytes());
    let mut current_hash = hasher.finalize().into();

    // Reconstruct root hash by following proof path
    for (sibling_hash, is_left) in &proof.path {
        let sibling_bytes = hex::decode(sibling_hash).ok()?;
        if sibling_bytes.len() != 32 {
            return false;
        }
        let mut sibling_hash_array = [0u8; 32];
        sibling_hash_array.copy_from_slice(&sibling_bytes);

        let mut hasher = Sha256::new();
        if *is_left {
            // Current is right, sibling is left
            hasher.update(&sibling_hash_array);
            hasher.update(&current_hash);
        } else {
            // Current is left, sibling is right
            hasher.update(&current_hash);
            hasher.update(&sibling_hash_array);
        }
        current_hash = hasher.finalize().into();
    }

    // Compare with root hash
    let root_bytes = hex::decode(&proof.root_hash).ok()?;
    if root_bytes.len() != 32 {
        return false;
    }
    let mut root_hash_array = [0u8; 32];
    root_hash_array.copy_from_slice(&root_bytes);

    current_hash == root_hash_array
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economic_nodes::types::{SignalType, VetoSignal};
    use chrono::Utc;

    fn create_test_signal(pr_id: i32, voting_key: &str, weight: f64) -> VetoSignal {
        VetoSignal {
            id: Some(1),
            pr_id,
            node_id: 1,
            signal_type: SignalType::Veto,
            weight,
            signature: "test_signature".to_string(),
            rationale: "Test rationale".to_string(),
            timestamp: Utc::now(),
            verified: true,
            voting_public_key: Some(voting_key.to_string()),
            voting_key_path: Some(format!("m/0'/{pr_id}'/0'")),
            signal_index: Some(0),
        }
    }

    #[test]
    fn test_build_merkle_tree_single_signal() {
        let signals = vec![create_test_signal(123, "pubkey1", 0.5)];
        let (root_hash, proofs) = build_veto_merkle_tree(&signals).unwrap();
        assert_eq!(proofs.len(), 1);
        assert!(proofs.contains_key("pubkey1"));
    }

    #[test]
    fn test_build_merkle_tree_multiple_signals() {
        let signals = vec![
            create_test_signal(123, "pubkey1", 0.5),
            create_test_signal(123, "pubkey2", 0.3),
            create_test_signal(123, "pubkey3", 0.2),
        ];
        let (root_hash, proofs) = build_veto_merkle_tree(&signals).unwrap();
        assert_eq!(proofs.len(), 3);
        assert!(proofs.contains_key("pubkey1"));
        assert!(proofs.contains_key("pubkey2"));
        assert!(proofs.contains_key("pubkey3"));
    }

    #[test]
    fn test_verify_merkle_proof() {
        let signals = vec![
            create_test_signal(123, "pubkey1", 0.5),
            create_test_signal(123, "pubkey2", 0.3),
        ];
        let (root_hash, proofs) = build_veto_merkle_tree(&signals).unwrap();
        
        let proof = proofs.get("pubkey1").unwrap();
        assert!(verify_merkle_proof(proof));
    }

    #[test]
    fn test_verify_merkle_proof_invalid() {
        let signals = vec![create_test_signal(123, "pubkey1", 0.5)];
        let (root_hash, proofs) = build_veto_merkle_tree(&signals).unwrap();
        
        let mut proof = proofs.get("pubkey1").unwrap().clone();
        proof.root_hash = "invalid".to_string();
        assert!(!verify_merkle_proof(&proof));
    }
}

