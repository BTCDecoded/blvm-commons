//! Snapshot tests for validation functions
//!
//! These tests use insta to snapshot complex data structures and verify
//! they don't change unexpectedly.

use bllvm_commons::validation::content_hash::{ContentHashValidator, DirectoryHashResult};
use bllvm_commons::validation::version_pinning::VersionPinningValidator;
use insta::assert_snapshot;

#[test]
fn test_content_hash_snapshot() {
    let validator = ContentHashValidator::new();
    let content = b"test content for snapshot";
    let hash = validator.compute_file_hash(content);
    
    assert_snapshot!("content_hash", hash);
}

#[test]
fn test_directory_hash_snapshot() {
    let validator = ContentHashValidator::new();
    let files = vec![
        ("file1.txt".to_string(), b"content1".to_vec()),
        ("file2.txt".to_string(), b"content2".to_vec()),
        ("file3.txt".to_string(), b"content3".to_vec()),
    ];
    
    let result = validator.compute_directory_hash(&files);
    
    // Snapshot the entire result
    assert_snapshot!("directory_hash", format!(
        "file_count: {}\ntotal_size: {}\nmerkle_root: {}",
        result.file_count,
        result.total_size,
        result.merkle_root
    ));
}

#[test]
fn test_version_format_snapshot() {
    let validator = VersionPinningValidator::default();
    let format = validator.generate_reference_format(
        "v1.2.3",
        "abc123def456",
        "sha256:fedcba9876543210"
    );
    
    assert_snapshot!("version_format", format);
}

#[test]
fn test_version_parsing_snapshot() {
    let validator = VersionPinningValidator::default();
    let content = r#"
// @orange-paper-version: v1.2.3
// @orange-paper-commit: abc123def456789
// @orange-paper-hash: sha256:fedcba123456
"#;
    
    let refs = validator.parse_version_references("test.rs", content).unwrap();
    
    // Snapshot the parsed references
    let snapshot = refs.iter().map(|r| {
        format!(
            "type: {:?}, version: {:?}, commit: {:?}, hash: {:?}",
            r.reference_type,
            r.version,
            r.commit_sha,
            r.content_hash
        )
    }).collect::<Vec<_>>().join("\n");
    
    assert_snapshot!("version_references", snapshot);
}

#[test]
fn test_empty_directory_hash_snapshot() {
    let validator = ContentHashValidator::new();
    let result = validator.compute_directory_hash(&[]);
    
    assert_snapshot!("empty_directory_hash", format!(
        "file_count: {}\ntotal_size: {}\nmerkle_root: {}",
        result.file_count,
        result.total_size,
        result.merkle_root
    ));
}

