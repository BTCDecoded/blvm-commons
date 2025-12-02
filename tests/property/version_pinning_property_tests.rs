//! Property-based tests for version pinning validation
//!
//! These tests verify mathematical properties of version parsing and validation.

use bllvm_commons::validation::version_pinning::VersionPinningValidator;
use proptest::prelude::*;

proptest! {
    /// Property: Version parsing is deterministic
    #[test]
    fn test_version_parsing_determinism(
        version in prop::string::string_regex(r"v\d+\.\d+\.\d+(-[a-zA-Z0-9]+)?(\+[a-zA-Z0-9]+)?").unwrap()
    ) {
        let validator = VersionPinningValidator::default();
        let content = format!("// @orange-paper-version: {}", version);
        
        let refs1 = validator.parse_version_references("test.rs", &content);
        let refs2 = validator.parse_version_references("test.rs", &content);
        
        prop_assert_eq!(refs1, refs2, "Version parsing must be deterministic");
    }

    /// Property: Valid semantic versions are always parsed correctly
    #[test]
    fn test_semantic_version_parsing(
        major in 0u64..1000,
        minor in 0u64..1000,
        patch in 0u64..1000
    ) {
        let validator = VersionPinningValidator::default();
        let version = format!("v{}.{}.{}", major, minor, patch);
        let content = format!("// @orange-paper-version: {}", version);
        
        let refs = validator.parse_version_references("test.rs", &content);
        
        prop_assert!(refs.is_ok(), "Valid semantic version must parse successfully");
        if let Ok(refs) = refs {
            prop_assert!(!refs.is_empty(), "Version reference must be found");
        }
    }

    /// Property: Version format generation is idempotent
    #[test]
    fn test_version_format_idempotency(
        version in prop::string::string_regex(r"v\d+\.\d+\.\d+").unwrap(),
        commit in prop::string::string_regex(r"[a-f0-9]{40}").unwrap(),
        hash in prop::string::string_regex(r"sha256:[a-f0-9]{64}").unwrap()
    ) {
        let validator = VersionPinningValidator::default();
        
        let format1 = validator.generate_reference_format(&version, &commit, &hash);
        let format2 = validator.generate_reference_format(&version, &commit, &hash);
        
        prop_assert_eq!(format1, format2, "Format generation must be idempotent");
    }

    /// Property: Generated format contains all input values
    #[test]
    fn test_version_format_contains_values(
        version in prop::string::string_regex(r"v\d+\.\d+\.\d+").unwrap(),
        commit in prop::string::string_regex(r"[a-f0-9]{40}").unwrap(),
        hash in prop::string::string_regex(r"sha256:[a-f0-9]{64}").unwrap()
    ) {
        let validator = VersionPinningValidator::default();
        let format = validator.generate_reference_format(&version, &commit, &hash);
        
        prop_assert!(format.contains(&version), "Format must contain version");
        prop_assert!(format.contains(&commit), "Format must contain commit");
        prop_assert!(format.contains(&hash), "Format must contain hash");
    }

    /// Property: Multiple version references in same file are all parsed
    #[test]
    fn test_multiple_version_references(
        version1 in prop::string::string_regex(r"v\d+\.\d+\.\d+").unwrap(),
        version2 in prop::string::string_regex(r"v\d+\.\d+\.\d+").unwrap()
    ) {
        let validator = VersionPinningValidator::default();
        let content = format!(
            "// @orange-paper-version: {}\n// @orange-paper-version: {}",
            version1, version2
        );
        
        let refs = validator.parse_version_references("test.rs", &content);
        
        prop_assert!(refs.is_ok(), "Multiple references must parse");
        if let Ok(refs) = refs {
            prop_assert!(refs.len() >= 2, "All references must be found");
        }
    }

    /// Property: Version reference parsing handles various comment styles
    #[test]
    fn test_comment_style_variations(
        version in prop::string::string_regex(r"v\d+\.\d+\.\d+").unwrap()
    ) {
        let validator = VersionPinningValidator::default();
        
        let styles = vec![
            format!("// @orange-paper-version: {}", version),
            format!("/* @orange-paper-version: {} */", version),
            format!("# @orange-paper-version: {}", version),
        ];
        
        for style in styles {
            let refs = validator.parse_version_references("test.rs", &style);
            // At least one style should work
            if refs.is_ok() && !refs.as_ref().unwrap().is_empty() {
                return Ok(());
            }
        }
        
        prop_assert!(false, "At least one comment style should work");
    }
}

