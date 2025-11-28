//! Tests for payment hash extraction from Lightning invoices
//!
//! Note: These tests require valid bolt11 invoices. Since extract_payment_hash
//! is a private method, we test it indirectly through the zap processing flow
//! in integration tests.

// The extract_payment_hash function is tested indirectly through:
// - Integration tests in tests/nostr_tests.rs
// - E2E tests that process actual zap events
//
// For unit testing, we would need to:
// 1. Make extract_payment_hash public for testing, OR
// 2. Test through the public process_zap interface
//
// Since the function is simple and well-integrated, we rely on integration
// tests to verify it works correctly with real bolt11 invoices.

