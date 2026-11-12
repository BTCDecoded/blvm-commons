//! M1–M3 conservative policy: the longer clock wins, layer-4 emergency stays
//! 60 days, 0-day review is not implemented, security floors only raise, and
//! nested multisig does not invent a quorum.

use blvm_commons::enforcement::merge_block::MergeBlocker;
use blvm_commons::validation::nested_multisig::{NestedMultisigVerifier, Team};
use blvm_commons::validation::review_period::ReviewPeriodValidator;
use blvm_commons::validation::threshold::ThresholdValidator;
use chrono::Utc;

#[test]
fn consensus_layers_use_365_days() {
    assert_eq!(
        ThresholdValidator::get_review_period_for_layer(1, false),
        365
    );
    assert_eq!(
        ThresholdValidator::get_review_period_for_layer(2, true),
        365
    );
    let (_required, _total, days) = ThresholdValidator::get_combined_requirements(1, 3);
    assert_eq!(days, 365);
}

#[test]
fn layer_4_emergency_stays_60_days() {
    assert_eq!(ThresholdValidator::get_tier_review_period(4), 60);
    assert_eq!(ThresholdValidator::get_review_period_for_layer(4, true), 60);
    assert_eq!(ThresholdValidator::layer_4_emergency_review_days(false), 60);
    assert_eq!(ThresholdValidator::layer_4_emergency_review_days(true), 60);
    let (required, total, days) = ThresholdValidator::get_combined_requirements(4, 4);
    assert_eq!((required, total, days), (4, 5, 60));
}

#[test]
fn zero_day_review_is_rejected() {
    let opened = Utc::now();
    let err = ReviewPeriodValidator::validate_review_period(opened, 0, true).expect_err("0-day");
    assert!(err.to_string().contains("0-day"));
}

#[test]
fn emergency_mode_does_not_skip_the_review_clock() {
    let blocked = MergeBlocker::should_block_merge(false, true, true).unwrap();
    assert!(blocked);
    let reason = MergeBlocker::get_block_reason(false, true, true);
    assert!(reason.contains("does not waive"));
}

#[test]
fn security_floor_only_raises() {
    let base = ThresholdValidator::get_combined_requirements(4, 1);
    let raised =
        ThresholdValidator::raise_to_security_floor(base, &["src/crypto/key_management.rs"]);
    assert_eq!(raised, (7, 7, 180));

    let constitutional = ThresholdValidator::get_combined_requirements(1, 5);
    let unchanged = ThresholdValidator::raise_to_security_floor(constitutional, &["README.md"]);
    assert_eq!(unchanged, constitutional);
    assert!(unchanged.2 >= 365);
}

#[test]
fn nested_multisig_does_not_invent_a_quorum() {
    let verifier = NestedMultisigVerifier::new(Vec::<Team>::new());
    let err = verifier.verify_nested_multisig(&[], 1).expect_err("tier 1");
    assert!(err.to_string().contains("refusing 1-of-1"));
    let err = verifier
        .verify_nested_multisig(&[], 9)
        .expect_err("unknown tier");
    assert!(err.to_string().contains("refusing 1-of-1"));
}
