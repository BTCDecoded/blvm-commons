#![no_main]
use libfuzzer_sys::fuzz_target;
use bllvm_commons::validation::content_hash::ContentHashValidator;

fuzz_target!(|data: &[u8]| {
    let validator = ContentHashValidator::new();
    
    // Fuzz hash computation
    let hash = validator.compute_file_hash(data);
    
    // Verify hash format
    assert!(hash.starts_with("sha256:"));
    assert_eq!(hash.len(), 71);
    
    // Verify determinism
    let hash2 = validator.compute_file_hash(data);
    assert_eq!(hash, hash2);
});

