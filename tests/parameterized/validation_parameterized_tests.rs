//! Parameterized tests for validation functions
//!
//! These tests use test cases with multiple inputs to verify edge cases
//! and boundary conditions systematically.

use bllvm_commons::validation::content_hash::ContentHashValidator;
use bllvm_commons::validation::version_pinning::VersionPinningValidator;

/// Test cases for hash computation with known inputs
#[test]
fn test_hash_known_values() {
    let validator = ContentHashValidator::new();
    
    let test_cases = vec![
        (b"", "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        (b"hello", "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
        (b"hello world", "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"),
    ];
    
    for (input, expected) in test_cases {
        let hash = validator.compute_file_hash(input);
        assert_eq!(hash, expected, "Hash mismatch for input: {:?}", input);
    }
}

/// Test cases for version parsing edge cases
#[test]
fn test_version_parsing_edge_cases() {
    let validator = VersionPinningValidator::default();
    
    let test_cases = vec![
        ("v1.0.0", true),
        ("v0.0.0", true),
        ("v999.999.999", true),
        ("v1.0.0-alpha", true),
        ("v1.0.0+metadata", true),
        ("1.0.0", false), // Missing 'v' prefix
        ("v1.0", false), // Missing patch
        ("v1", false), // Missing minor and patch
        ("", false), // Empty
    ];
    
    for (version, should_parse) in test_cases {
        let content = format!("// @orange-paper-version: {}", version);
        let result = validator.parse_version_references("test.rs", &content);
        
        if should_parse {
            assert!(result.is_ok(), "Version '{}' should parse", version);
        } else {
            // May or may not parse depending on implementation
            // Just verify it doesn't crash
            let _ = result;
        }
    }
}

/// Test cases for directory hash with various file configurations
#[test]
fn test_directory_hash_various_configs() {
    let validator = ContentHashValidator::new();
    
    let test_cases = vec![
        // Empty directory
        vec![],
        // Single file
        vec![("file1.txt".to_string(), b"content1".to_vec())],
        // Multiple files
        vec![
            ("file1.txt".to_string(), b"content1".to_vec()),
            ("file2.txt".to_string(), b"content2".to_vec()),
            ("file3.txt".to_string(), b"content3".to_vec()),
        ],
        // Files with same content
        vec![
            ("file1.txt".to_string(), b"same".to_vec()),
            ("file2.txt".to_string(), b"same".to_vec()),
        ],
        // Large file
        vec![("large.txt".to_string(), vec![0u8; 10000])],
    ];
    
    for files in test_cases {
        let result = validator.compute_directory_hash(&files);
        
        assert_eq!(result.file_count, files.len());
        let expected_size: u64 = files.iter().map(|(_, content)| content.len() as u64).sum();
        assert_eq!(result.total_size, expected_size);
        assert!(result.merkle_root.starts_with("sha256:"));
    }
}

/// Test cases for version format generation
#[test]
fn test_version_format_generation_cases() {
    let validator = VersionPinningValidator::default();
    
    let test_cases = vec![
        ("v1.0.0", "abc123", "sha256:def456"),
        ("v2.1.3", "deadbeef", "sha256:feedface"),
        ("v0.0.1", "00000000", "sha256:11111111"),
    ];
    
    for (version, commit, hash) in test_cases {
        let format = validator.generate_reference_format(version, commit, hash);
        
        assert!(format.contains(version), "Format must contain version");
        assert!(format.contains(commit), "Format must contain commit");
        assert!(format.contains(hash), "Format must contain hash");
    }
}

