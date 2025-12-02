//! Snapshot test suite using insta
//!
//! Run with: cargo test --test snapshot_tests
//! Update snapshots: cargo insta review

mod validation_snapshot_tests {
    use bllvm_commons::validation::content_hash::ContentHashValidator;
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

        assert_snapshot!(
            "directory_hash",
            format!(
                "file_count: {}\ntotal_size: {}\nmerkle_root: {}",
                result.file_count, result.total_size, result.merkle_root
            )
        );
    }

    #[test]
    fn test_version_format_snapshot() {
        let validator = VersionPinningValidator::default();
        let format = validator.generate_reference_format(
            "v1.2.3",
            "abc123def456",
            "sha256:fedcba9876543210",
        );

        assert_snapshot!("version_format", format);
    }

    #[test]
    fn test_empty_directory_hash_snapshot() {
        let validator = ContentHashValidator::new();
        let result = validator.compute_directory_hash(&[]);

        assert_snapshot!(
            "empty_directory_hash",
            format!(
                "file_count: {}\ntotal_size: {}\nmerkle_root: {}",
                result.file_count, result.total_size, result.merkle_root
            )
        );
    }
}

mod github_snapshot_tests {
    use bllvm_commons::github::cross_layer_status::CrossLayerStatusChecker;
    use insta::assert_snapshot;

    #[test]
    fn test_test_count_extraction_snapshot() {
        let test_cases = vec![
            "123 tests",
            "Tests: 456",
            "cargo test: 789",
            "1000 passed",
            "passed: 42",
        ];

        for test_case in test_cases {
            let result = CrossLayerStatusChecker::extract_test_count_from_name(test_case);
            assert_snapshot!(
                format!(
                    "test_count_extraction_{}",
                    test_case.replace(" ", "_").replace(":", "_")
                ),
                format!("input: '{}', output: {:?}", test_case, result)
            );
        }
    }
}
