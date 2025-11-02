#!/bin/bash

# Cross-Layer Synchronization Test Runner
# This script runs comprehensive tests for the cryptographic layer synchronization system

set -e

echo "🔍 Cross-Layer Synchronization Test Suite"
echo "=========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_FAILED=0
TOTAL_TESTS=0

# Function to run a test and track results
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "\n${BLUE}Running: $test_name${NC}"
    echo "Command: $test_command"
    echo "----------------------------------------"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "Checking prerequisites..."

if ! command_exists cargo; then
    echo -e "${RED}Error: cargo not found. Please install Rust toolchain.${NC}"
    exit 1
fi

if ! command_exists rustc; then
    echo -e "${RED}Error: rustc not found. Please install Rust toolchain.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"

# Change to project root
cd "$(dirname "$0")/.."

echo -e "\n${YELLOW}Building project...${NC}"
cargo build --release

echo -e "\n${YELLOW}Running unit tests...${NC}"
cargo test --lib

echo -e "\n${YELLOW}Running integration tests...${NC}"
cargo test --test cross_layer_sync_tests

echo -e "\n${YELLOW}Running standalone tests...${NC}"

# Track 1: Content Hash Verification
run_test "Content Hash Verification" "cargo run --bin test-content-hash"

# Track 2: Version Pinning
run_test "Version Pinning Validation" "cargo run --bin test-version-pinning"

# Track 3: Equivalence Proof Validation
run_test "Equivalence Proof Validation" "cargo run --bin test-equivalence-proof"

# Track 4: Cross-Layer Integration
run_test "Cross-Layer Integration" "cargo run --bin test-cross-layer-integration"

echo -e "\n${YELLOW}Running code quality checks...${NC}"

# Clippy
run_test "Clippy Linting" "cargo clippy --all-targets --all-features -- -D warnings"

# Format check
run_test "Code Formatting" "cargo fmt --all -- --check"

# Security audit
if command_exists cargo-audit; then
    run_test "Security Audit" "cargo audit"
else
    echo -e "${YELLOW}⚠️  cargo-audit not found, skipping security audit${NC}"
fi

# Documentation generation
run_test "Documentation Generation" "cargo doc --no-deps --document-private-items"

echo -e "\n${YELLOW}Running performance tests...${NC}"

# Performance test with large datasets
run_test "Performance Test (1000 files)" "cargo run --bin test-cross-layer-integration -- --performance-test"

echo -e "\n${YELLOW}Running stress tests...${NC}"

# Stress test with concurrent operations
run_test "Stress Test (Concurrent)" "cargo test --test cross_layer_sync_tests test_cross_layer_validation_concurrent -- --nocapture"

# Memory usage test
run_test "Memory Usage Test" "cargo test --test cross_layer_sync_tests test_cross_layer_validation_performance -- --nocapture"

echo -e "\n${YELLOW}Running error handling tests...${NC}"

# Error handling tests
run_test "Error Handling Test" "cargo test --test cross_layer_sync_tests test_cross_layer_validation_error_handling -- --nocapture"

echo -e "\n${YELLOW}Running edge case tests...${NC}"

# Edge case tests
run_test "Edge Case Test (Empty Files)" "cargo test --test cross_layer_sync_tests test_cross_layer_validation_error_handling -- --nocapture"

echo -e "\n${YELLOW}Running security tests...${NC}"

# Security tests
run_test "Security Test (Hash Collision)" "cargo test --test cross_layer_sync_tests test_content_hash_verification_integration -- --nocapture"

echo -e "\n${YELLOW}Running compatibility tests...${NC}"

# Compatibility tests
run_test "Compatibility Test (Version Pinning)" "cargo test --test cross_layer_sync_tests test_version_pinning_validation_integration -- --nocapture"

echo -e "\n${YELLOW}Running regression tests...${NC}"

# Regression tests
run_test "Regression Test (Equivalence Proof)" "cargo test --test cross_layer_sync_tests test_equivalence_proof_validation_integration -- --nocapture"

# Test summary
echo -e "\n${BLUE}Test Summary${NC}"
echo "============"
echo -e "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed! Cross-layer synchronization system is working correctly.${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed. Please check the output above for details.${NC}"
    exit 1
fi



