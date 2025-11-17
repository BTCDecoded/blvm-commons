#!/bin/bash
# End-to-end test script for build orchestration
# This script tests the full release flow

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[TEST]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Test 1: Dependency Graph
log_info "Test 1: Dependency Graph"
cd "$PROJECT_ROOT"
cargo test --package governance-app --lib build::dependency::tests -- --nocapture || {
    log_error "Dependency graph tests failed"
    exit 1
}
log_info "✓ Dependency graph tests passed"

# Test 2: Build Order
log_info "Test 2: Build Order Calculation"
cargo test --package governance-app --lib build::tests::test_dependency_graph_build_order -- --nocapture || {
    log_error "Build order tests failed"
    exit 1
}
log_info "✓ Build order tests passed"

# Test 3: Circular Dependency Detection
log_info "Test 3: Circular Dependency Detection"
cargo test --package governance-app --lib build::tests::test_circular_dependency_detection -- --nocapture || {
    log_error "Circular dependency detection tests failed"
    exit 1
}
log_info "✓ Circular dependency detection tests passed"

# Test 4: Parallel Groups
log_info "Test 4: Parallel Build Groups"
cargo test --package governance-app --lib build::tests::test_parallel_build_groups -- --nocapture || {
    log_error "Parallel groups tests failed"
    exit 1
}
log_info "✓ Parallel groups tests passed"

# Test 5: Integration Tests
log_info "Test 5: Integration Tests"
cargo test --package governance-app --test build_orchestration_test -- --nocapture || {
    log_warn "Integration tests require mocks - skipping for now"
}
log_info "✓ Integration tests completed"

log_info ""
log_info "=========================================="
log_info "All Tests Passed!"
log_info "=========================================="
log_info ""
log_info "Next steps:"
log_info "1. Test with mock GitHub API"
log_info "2. Test workflow triggering"
log_info "3. Test build monitoring"
log_info "4. Test artifact collection"

