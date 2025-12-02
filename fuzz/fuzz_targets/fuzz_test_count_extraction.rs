#![no_main]
use libfuzzer_sys::fuzz_target;
use bllvm_commons::github::cross_layer_status::CrossLayerStatusChecker;

fuzz_target!(|data: &str| {
    // Fuzz test count extraction - should not panic
    let _ = CrossLayerStatusChecker::extract_test_count_from_name(data);
});

