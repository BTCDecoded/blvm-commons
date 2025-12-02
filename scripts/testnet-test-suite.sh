#!/bin/bash
# Testnet Test Suite
# Comprehensive testing for the governance-app testnet environment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
GOVERNANCE_APP_DIR="$PROJECT_ROOT/governance-app"
TESTNET_DIR="$PROJECT_ROOT/deployment/testnet"

echo "🧪 Starting testnet test suite..."

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
    
    echo -e "${BLUE}Running test: ${test_name}${NC}"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ ${test_name} passed${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ ${test_name} failed${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    echo ""
}

# Function to check if testnet is running
check_testnet_running() {
    echo "🔍 Checking if testnet is running..."
    
    if ! docker-compose -f "$TESTNET_DIR/docker-compose.yml" ps | grep -q "Up"; then
        echo -e "${RED}❌ Testnet is not running. Please start it first:${NC}"
        echo "   cd $TESTNET_DIR && docker-compose up -d"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Testnet is running${NC}"
}

# Function to wait for testnet to be ready
wait_for_testnet() {
    echo "⏳ Waiting for testnet to be ready..."
    
    local max_attempts=24  # 2 minutes with 5-second intervals
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Testnet is ready!${NC}"
            return 0
        fi
        
        echo "   Attempt $((attempt + 1))/$max_attempts - waiting..."
        sleep 5
        attempt=$((attempt + 1))
    done
    
    echo -e "${RED}❌ Testnet failed to become ready within timeout${NC}"
    exit 1
}

# Function to run integration tests
run_integration_tests() {
    echo "🧪 Running integration tests..."
    
    cd "$GOVERNANCE_APP_DIR"
    
    # Run testnet scenarios
    run_test "Testnet Scenarios" "cargo test --test testnet_scenarios -- --nocapture"
    
    # Run other integration tests
    run_test "Integration Tests" "cargo test --test integration -- --nocapture"
}

# Function to test CLI tools
test_cli_tools() {
    echo "🔧 Testing CLI tools..."
    
    cd "$GOVERNANCE_APP_DIR"
    
    # Test sign-pr tool
    run_test "Sign PR Tool" "cargo run --release --bin sign-pr -- --help"
    
    # Test economic node tools
    run_test "Economic Node Register" "cargo run --release --bin economic-node-register -- --help"
    run_test "Economic Node Veto" "cargo run --release --bin economic-node-veto -- --help"
    run_test "Economic Node Verify" "cargo run --release --bin economic-node-verify -- --help"
    
    # Test fork migration tool
    run_test "Fork Migrate Tool" "cargo run --release --bin fork-migrate -- --help"
    
    # Test audit log verification
    run_test "Audit Log Verification" "cargo run --release --bin verify-audit-log -- --help"
}

# Function to test API endpoints
test_api_endpoints() {
    echo "🌐 Testing API endpoints..."
    
    local base_url="http://localhost:8080"
    
    # Health endpoints
    run_test "Health Check" "curl -s $base_url/health | jq -e '.status == \"healthy\"'"
    run_test "Database Health" "curl -s $base_url/api/health/database | jq -e '.status == \"healthy\"'"
    
    # Governance endpoints
    run_test "PR List" "curl -s $base_url/api/prs | jq -e '.prs | type == \"array\"'"
    run_test "Economic Nodes" "curl -s $base_url/api/economic-nodes | jq -e '.nodes | type == \"array\"'"
    run_test "Adoption Statistics" "curl -s $base_url/api/adoption/statistics | jq -e '.total_nodes | type == \"number\"'"
    run_test "Fork Status" "curl -s $base_url/api/fork/status | jq -e '.current_ruleset | type == \"string\"'"
    
    # Metrics endpoint
    run_test "Metrics Endpoint" "curl -s $base_url/metrics | grep -q 'governance_events_total'"
}

# Function to test signature workflow
test_signature_workflow() {
    echo "🔐 Testing signature workflow..."
    
    # Create test PR
    local pr_data='{
        "action": "opened",
        "pull_request": {
            "number": 999,
            "title": "Test PR for signature workflow",
            "body": "This is a test PR",
            "head": {"sha": "test123"},
            "base": {"ref": "main"}
        },
        "repository": {"full_name": "test/governance-test"}
    }'
    
    run_test "Create Test PR" "curl -s -X POST http://localhost:8080/webhooks/github -H 'Content-Type: application/json' -d '$pr_data' | jq -e '.status == \"success\"'"
    
    # Submit test signature
    local signature_data='{
        "action": "created",
        "comment": {
            "body": "/governance-sign test_signature_123",
            "user": {"login": "alice"}
        },
        "issue": {"number": 999},
        "repository": {"full_name": "test/governance-test"}
    }'
    
    run_test "Submit Test Signature" "curl -s -X POST http://localhost:8080/webhooks/github -H 'Content-Type: application/json' -d '$signature_data' | jq -e '.status == \"signature_verified\"'"
}

# Function to test economic node workflow
test_economic_node_workflow() {
    echo "🏭 Testing economic node workflow..."
    
    # Register test economic node
    local node_data='{
        "name": "TestMiningPool",
        "node_type": "mining_pool",
        "public_key": "test_public_key",
        "hash_rate_percent": 15.0,
        "economic_activity_percent": 0.0
    }'
    
    run_test "Register Economic Node" "curl -s -X POST http://localhost:8080/api/economic-nodes/register -H 'Content-Type: application/json' -d '$node_data' | jq -e '.status == \"success\"'"
    
    # Submit veto signal
    local veto_data='{
        "node_name": "TestMiningPool",
        "repository": "test/governance-test",
        "pr_number": 999,
        "reason": "Test veto signal",
        "strength": 100,
        "signature": "test_veto_signature"
    }'
    
    run_test "Submit Veto Signal" "curl -s -X POST http://localhost:8080/api/veto -H 'Content-Type: application/json' -d '$veto_data' | jq -e '.status == \"success\"'"
}

# Function to test governance fork workflow
test_governance_fork_workflow() {
    echo "🔄 Testing governance fork workflow..."
    
    # Create test ruleset
    local ruleset_data='{
        "name": "test-ruleset",
        "description": "Test governance ruleset",
        "version": "1.0.0",
        "config": {
            "action_tiers": {},
            "economic_nodes": {},
            "maintainers": {},
            "repositories": {}
        }
    }'
    
    run_test "Create Test Ruleset" "curl -s -X POST http://localhost:8080/api/fork/rulesets -H 'Content-Type: application/json' -d '$ruleset_data' | jq -e '.status == \"success\"'"
    
    # List rulesets
    run_test "List Rulesets" "curl -s http://localhost:8080/api/fork/rulesets | jq -e '.rulesets | type == \"array\"'"
}

# Function to test monitoring and metrics
test_monitoring_metrics() {
    echo "📊 Testing monitoring and metrics..."
    
    # Check Prometheus metrics
    run_test "Prometheus Metrics" "curl -s http://localhost:8080/metrics | grep -q 'governance_events_total'"
    
    # Check Grafana is accessible
    run_test "Grafana Access" "curl -s http://localhost:3000/api/health | jq -e '.database == \"ok\"'"
    
    # Check Prometheus is accessible
    run_test "Prometheus Access" "curl -s http://localhost:9091/api/v1/status/config | jq -e '.status == \"success\"'"
}

# Function to test database integrity
test_database_integrity() {
    echo "🗄️ Testing database integrity..."
    
    # Check database health
    run_test "Database Health" "curl -s http://localhost:8080/api/health/database | jq -e '.status == \"healthy\"'"
    
    # Check database size
    run_test "Database Size" "curl -s http://localhost:8080/api/health/database | jq -e '.database_size_bytes > 0'"
    
    # Check WAL mode
    run_test "WAL Mode" "curl -s http://localhost:8080/api/health/database | jq -e '.wal_mode_active == true'"
}

# Function to test security features
test_security_features() {
    echo "🔒 Testing security features..."
    
    # Test server authorization
    run_test "Server Authorization" "curl -s http://localhost:8080/api/server-auth/status | jq -e '.enabled == true'"
    
    # Test audit log
    run_test "Audit Log" "curl -s http://localhost:8080/api/audit/log | jq -e '.entries | type == \"array\"'"
    
    # Test Nostr integration
    run_test "Nostr Integration" "curl -s http://localhost:8080/api/nostr/status | jq -e '.connected == true'"
}

# Function to run performance tests
test_performance() {
    echo "⚡ Testing performance..."
    
    # Test response times
    local start_time=$(date +%s%N)
    curl -s http://localhost:8080/health > /dev/null
    local end_time=$(date +%s%N)
    local response_time=$(( (end_time - start_time) / 1000000 ))  # Convert to milliseconds
    
    if [ $response_time -lt 1000 ]; then
        echo -e "${GREEN}✅ Health check response time: ${response_time}ms${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ Health check response time too slow: ${response_time}ms${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
}

# Function to generate test report
generate_test_report() {
    echo ""
    echo "📊 Test Suite Results:"
    echo "====================="
    echo -e "Total Tests: ${TOTAL_TESTS}"
    echo -e "${GREEN}Passed: ${TESTS_PASSED}${NC}"
    echo -e "${RED}Failed: ${TESTS_FAILED}${NC}"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}❌ Some tests failed. Please check the output above.${NC}"
        exit 1
    fi
}

# Main execution
main() {
    echo "🚀 Starting testnet test suite..."
    echo "================================="
    
    # Check if testnet is running
    check_testnet_running
    
    # Wait for testnet to be ready
    wait_for_testnet
    
    # Run all test categories
    test_cli_tools
    test_api_endpoints
    test_signature_workflow
    test_economic_node_workflow
    test_governance_fork_workflow
    test_monitoring_metrics
    test_database_integrity
    test_security_features
    test_performance
    
    # Run integration tests
    run_integration_tests
    
    # Generate test report
    generate_test_report
}

# Handle command line arguments
case "${1:-}" in
    "cli")
        check_testnet_running
        wait_for_testnet
        test_cli_tools
        generate_test_report
        ;;
    "api")
        check_testnet_running
        wait_for_testnet
        test_api_endpoints
        generate_test_report
        ;;
    "workflow")
        check_testnet_running
        wait_for_testnet
        test_signature_workflow
        test_economic_node_workflow
        test_governance_fork_workflow
        generate_test_report
        ;;
    "monitoring")
        check_testnet_running
        wait_for_testnet
        test_monitoring_metrics
        generate_test_report
        ;;
    "integration")
        check_testnet_running
        wait_for_testnet
        run_integration_tests
        generate_test_report
        ;;
    "all"|"")
        main
        ;;
    *)
        echo "Usage: $0 [cli|api|workflow|monitoring|integration|all]"
        echo ""
        echo "Test categories:"
        echo "  cli         - Test CLI tools"
        echo "  api         - Test API endpoints"
        echo "  workflow    - Test signature and veto workflows"
        echo "  monitoring  - Test monitoring and metrics"
        echo "  integration - Run integration tests"
        echo "  all         - Run all tests (default)"
        exit 1
        ;;
esac
