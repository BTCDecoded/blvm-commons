#![no_main]
use libfuzzer_sys::fuzz_target;
use bllvm_commons::validation::version_pinning::VersionPinningValidator;

fuzz_target!(|data: &str| {
    let validator = VersionPinningValidator::default();
    
    // Fuzz version parsing - should not panic
    let content = format!("// @orange-paper-version: {}", data);
    let _ = validator.parse_version_references("test.rs", &content);
    
    // Try various comment styles
    let styles = vec![
        format!("// @orange-paper-version: {}", data),
        format!("/* @orange-paper-version: {} */", data),
        format!("# @orange-paper-version: {}", data),
    ];
    
    for style in styles {
        let _ = validator.parse_version_references("test.rs", &style);
    }
});

