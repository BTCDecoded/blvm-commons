//! Property-based tests for content hash validation
//!
//! These tests use proptest to verify mathematical properties of hash functions
//! and ensure correctness across a wide range of inputs.

use bllvm_commons::validation::content_hash::ContentHashValidator;
use proptest::prelude::*;

proptest! {
    /// Property: Hash function is deterministic
    /// For any input, the hash should always be the same
    #[test]
    fn test_hash_determinism(
        content in prop::collection::vec(any::<u8>(), 0..10000)
    ) {
        let validator = ContentHashValidator::new();
        let hash1 = validator.compute_file_hash(&content);
        let hash2 = validator.compute_file_hash(&content);
        
        prop_assert_eq!(hash1, hash2, "Hash must be deterministic");
    }

    /// Property: Hash function produces different outputs for different inputs
    /// (with high probability - collisions are possible but extremely rare)
    #[test]
    fn test_hash_uniqueness(
        content1 in prop::collection::vec(any::<u8>(), 1..1000),
        content2 in prop::collection::vec(any::<u8>(), 1..1000)
    ) {
        prop_assume!(content1 != content2);
        
        let validator = ContentHashValidator::new();
        let hash1 = validator.compute_file_hash(&content1);
        let hash2 = validator.compute_file_hash(&content2);
        
        // SHA256 collisions are extremely rare, but not impossible
        // This test will fail if we hit a collision (very unlikely)
        prop_assert_ne!(hash1, hash2, "Different inputs should produce different hashes");
    }

    /// Property: Hash format is always valid
    /// Hash should always start with "sha256:" and be 71 characters
    #[test]
    fn test_hash_format(
        content in prop::collection::vec(any::<u8>(), 0..100000)
    ) {
        let validator = ContentHashValidator::new();
        let hash = validator.compute_file_hash(&content);
        
        prop_assert!(hash.starts_with("sha256:"), "Hash must start with 'sha256:'");
        prop_assert_eq!(hash.len(), 71, "Hash must be 71 characters (7 + 64 hex chars)");
        
        // Verify hex characters after "sha256:"
        let hex_part = &hash[7..];
        prop_assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash hex part must contain only hex digits");
    }

    /// Property: Empty content produces known hash
    #[test]
    fn test_empty_hash() {
        let validator = ContentHashValidator::new();
        let hash = validator.compute_file_hash(&[]);
        
        // SHA256 of empty string
        let expected = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        prop_assert_eq!(hash, expected, "Empty content must produce known hash");
    }

    /// Property: Directory hash is deterministic
    #[test]
    fn test_directory_hash_determinism(
        files in prop::collection::vec(
            (prop::string::string_regex("[a-zA-Z0-9_./-]+\\.(rs|md|toml|json)").unwrap(),
             prop::collection::vec(any::<u8>(), 0..1000)),
            0..100
        )
    ) {
        let validator = ContentHashValidator::new();
        
        let files_vec: Vec<(String, Vec<u8>)> = files.clone();
        let result1 = validator.compute_directory_hash(&files_vec);
        
        let files_vec2: Vec<(String, Vec<u8>)> = files;
        let result2 = validator.compute_directory_hash(&files_vec2);
        
        prop_assert_eq!(result1.merkle_root, result2.merkle_root, "Directory hash must be deterministic");
        prop_assert_eq!(result1.file_count, result2.file_count);
        prop_assert_eq!(result1.total_size, result2.total_size);
    }

    /// Property: Directory hash file count matches input
    #[test]
    fn test_directory_hash_file_count(
        file_count in 0usize..1000
    ) {
        let validator = ContentHashValidator::new();
        let files: Vec<(String, Vec<u8>)> = (0..file_count)
            .map(|i| (format!("file{}.txt", i), vec![i as u8; 10]))
            .collect();
        
        let result = validator.compute_directory_hash(&files);
        
        prop_assert_eq!(result.file_count, file_count, "File count must match input");
    }

    /// Property: Directory hash total size matches sum of file sizes
    #[test]
    fn test_directory_hash_total_size(
        files in prop::collection::vec(
            (prop::string::string_regex("[a-zA-Z0-9_./-]+\\.txt").unwrap(),
             prop::collection::vec(any::<u8>(), 0..1000)),
            0..100
        )
    ) {
        let validator = ContentHashValidator::new();
        
        let expected_size: u64 = files.iter().map(|(_, content)| content.len() as u64).sum();
        let files_vec: Vec<(String, Vec<u8>)> = files;
        let result = validator.compute_directory_hash(&files_vec);
        
        prop_assert_eq!(result.total_size, expected_size, "Total size must match sum of file sizes");
    }

    /// Property: Empty directory produces known hash
    #[test]
    fn test_empty_directory_hash() {
        let validator = ContentHashValidator::new();
        let result = validator.compute_directory_hash(&[]);
        
        prop_assert_eq!(result.file_count, 0);
        prop_assert_eq!(result.total_size, 0);
        // Empty directory hash should be SHA256 of empty string
        let expected = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        prop_assert_eq!(result.merkle_root, expected);
    }

    /// Property: Hash is idempotent (applying hash multiple times gives same result)
    #[test]
    fn test_hash_idempotency(
        content in prop::collection::vec(any::<u8>(), 0..10000)
    ) {
        let validator = ContentHashValidator::new();
        let hash1 = validator.compute_file_hash(&content);
        let hash2 = validator.compute_file_hash(&content);
        let hash3 = validator.compute_file_hash(&content);
        
        prop_assert_eq!(hash1, hash2);
        prop_assert_eq!(hash2, hash3);
    }

    /// Property: Hash is case-insensitive for hex part (but we use lowercase)
    #[test]
    fn test_hash_hex_lowercase(
        content in prop::collection::vec(any::<u8>(), 0..1000)
    ) {
        let validator = ContentHashValidator::new();
        let hash = validator.compute_file_hash(&content);
        
        let hex_part = &hash[7..];
        prop_assert!(hex_part.chars().all(|c| c.is_lowercase() || c.is_ascii_digit()),
            "Hash hex part must be lowercase");
    }
}

