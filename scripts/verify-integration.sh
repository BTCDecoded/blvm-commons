#!/bin/bash

set -e

echo "Verifying BTCDecoded Governance System Integration..."

# Configuration
GOVERNANCE_DIR="governance"
GOVERNANCE_APP_DIR="governance-app"
CONFIG_DIR="$GOVERNANCE_DIR/config"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "PASS")
            echo -e "${GREEN}✓${NC} $message"
            ;;
        "FAIL")
            echo -e "${RED}✗${NC} $message"
            ;;
        "WARN")
            echo -e "${YELLOW}⚠${NC} $message"
            ;;
        "INFO")
            echo -e "${YELLOW}ℹ${NC} $message"
            ;;
    esac
}

# Function to check if file exists
check_file() {
    local file=$1
    local description=$2
    if [ -f "$file" ]; then
        print_status "PASS" "$description exists"
        return 0
    else
        print_status "FAIL" "$description missing: $file"
        return 1
    fi
}

# Function to check if directory exists
check_dir() {
    local dir=$1
    local description=$2
    if [ -d "$dir" ]; then
        print_status "PASS" "$description exists"
        return 0
    else
        print_status "FAIL" "$description missing: $dir"
        return 1
    fi
}

# Function to validate YAML file
validate_yaml() {
    local file=$1
    local description=$2
    if command -v yq >/dev/null 2>&1; then
        if yq eval '.' "$file" >/dev/null 2>&1; then
            print_status "PASS" "$description YAML syntax valid"
            return 0
        else
            print_status "FAIL" "$description YAML syntax invalid"
            return 1
        fi
    else
        print_status "WARN" "yq not installed, skipping YAML validation for $description"
        return 0
    fi
}

# Function to check governance-app compilation
check_compilation() {
    local dir=$1
    if [ -d "$dir" ]; then
        cd "$dir"
        if cargo check >/dev/null 2>&1; then
            print_status "PASS" "governance-app compiles successfully"
            cd - >/dev/null
            return 0
        else
            print_status "FAIL" "governance-app compilation failed"
            cd - >/dev/null
            return 1
        fi
    else
        print_status "FAIL" "governance-app directory not found"
        return 1
    fi
}

# Function to check configuration loading
check_config_loading() {
    local dir=$1
    if [ -d "$dir" ]; then
        cd "$dir"
        if cargo test config_loading >/dev/null 2>&1; then
            print_status "PASS" "Configuration loading tests pass"
            cd - >/dev/null
            return 0
        else
            print_status "FAIL" "Configuration loading tests failed"
            cd - >/dev/null
            return 1
        fi
    else
        print_status "FAIL" "governance-app directory not found"
        return 1
    fi
}

# Function to check tier classification
check_tier_classification() {
    local dir=$1
    if [ -d "$dir" ]; then
        cd "$dir"
        if cargo test tier_classification >/dev/null 2>&1; then
            print_status "PASS" "Tier classification tests pass"
            cd - >/dev/null
            return 0
        else
            print_status "FAIL" "Tier classification tests failed"
            cd - >/dev/null
            return 1
        fi
    else
        print_status "FAIL" "governance-app directory not found"
        return 1
    fi
}

# Function to check layer/tier consistency
check_layer_tier_consistency() {
    local config_dir=$1
    local errors=0
    
    # Check if action-tiers.yml exists
    if [ ! -f "$config_dir/action-tiers.yml" ]; then
        print_status "FAIL" "action-tiers.yml missing"
        errors=$((errors + 1))
    fi
    
    # Check if repository-layers.yml exists
    if [ ! -f "$config_dir/repository-layers.yml" ]; then
        print_status "FAIL" "repository-layers.yml missing"
        errors=$((errors + 1))
    fi
    
    # Check if tier-classification-rules.yml exists
    if [ ! -f "$config_dir/tier-classification-rules.yml" ]; then
        print_status "FAIL" "tier-classification-rules.yml missing"
        errors=$((errors + 1))
    fi
    
    if [ $errors -eq 0 ]; then
        print_status "PASS" "All required configuration files exist"
        return 0
    else
        print_status "FAIL" "Missing required configuration files"
        return 1
    fi
}

# Function to check documentation consistency
check_documentation_consistency() {
    local errors=0
    
    # Check if all referenced documents exist
    local docs=(
        "$GOVERNANCE_DIR/README.md"
        "$GOVERNANCE_DIR/GOVERNANCE.md"
        "$GOVERNANCE_DIR/LAYER_TIER_MODEL.md"
        "$GOVERNANCE_DIR/SCOPE.md"
        "$GOVERNANCE_DIR/architecture/CRYPTOGRAPHIC_GOVERNANCE.md"
        "$GOVERNANCE_DIR/architecture/ECONOMIC_NODES.md"
        "$GOVERNANCE_DIR/architecture/GOVERNANCE_FORK.md"
        "$GOVERNANCE_DIR/architecture/SERVER_AUTHORIZATION.md"
        "$GOVERNANCE_DIR/architecture/CROSS_LAYER_DEPENDENCIES.md"
        "$GOVERNANCE_APP_DIR/README.md"
        "$GOVERNANCE_APP_DIR/DEPLOYMENT.md"
        "$GOVERNANCE_APP_DIR/SECURITY.md"
        "$GOVERNANCE_APP_DIR/docs/VERIFICATION.md"
        "$GOVERNANCE_APP_DIR/docs/NOSTR_INTEGRATION.md"
        "$GOVERNANCE_APP_DIR/docs/OTS_INTEGRATION.md"
        "$GOVERNANCE_APP_DIR/docs/AUDIT_LOG_SYSTEM.md"
        "$GOVERNANCE_APP_DIR/docs/SERVER_AUTHORIZATION.md"
        "$GOVERNANCE_APP_DIR/docs/CONFIG_INTEGRATION.md"
    )
    
    for doc in "${docs[@]}"; do
        if [ -f "$doc" ]; then
            print_status "PASS" "Document exists: $(basename "$doc")"
        else
            print_status "FAIL" "Document missing: $doc"
            errors=$((errors + 1))
        fi
    done
    
    if [ $errors -eq 0 ]; then
        print_status "PASS" "All documentation files exist"
        return 0
    else
        print_status "FAIL" "Missing documentation files"
        return 1
    fi
}

# Main verification process
main() {
    local total_checks=0
    local passed_checks=0
    local failed_checks=0
    
    echo "Starting integration verification..."
    echo "=================================="
    
    # Check directory structure
    print_status "INFO" "Checking directory structure..."
    check_dir "$GOVERNANCE_DIR" "Governance directory" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    check_dir "$GOVERNANCE_APP_DIR" "Governance app directory" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    check_dir "$CONFIG_DIR" "Configuration directory" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    # Check core configuration files
    print_status "INFO" "Checking core configuration files..."
    check_file "$CONFIG_DIR/action-tiers.yml" "Action tiers configuration" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    check_file "$CONFIG_DIR/repository-layers.yml" "Repository layers configuration" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    check_file "$CONFIG_DIR/tier-classification-rules.yml" "Tier classification rules" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    # Validate YAML files
    print_status "INFO" "Validating YAML files..."
    validate_yaml "$CONFIG_DIR/action-tiers.yml" "Action tiers" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    validate_yaml "$CONFIG_DIR/repository-layers.yml" "Repository layers" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    validate_yaml "$CONFIG_DIR/tier-classification-rules.yml" "Tier classification rules" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    # Check governance-app compilation
    print_status "INFO" "Checking governance-app compilation..."
    check_compilation "$GOVERNANCE_APP_DIR" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    # Check configuration loading
    print_status "INFO" "Checking configuration loading..."
    check_config_loading "$GOVERNANCE_APP_DIR" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    # Check tier classification
    print_status "INFO" "Checking tier classification..."
    check_tier_classification "$GOVERNANCE_APP_DIR" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    # Check layer/tier consistency
    print_status "INFO" "Checking layer/tier consistency..."
    check_layer_tier_consistency "$CONFIG_DIR" && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    # Check documentation consistency
    print_status "INFO" "Checking documentation consistency..."
    check_documentation_consistency && passed_checks=$((passed_checks + 1)) || failed_checks=$((failed_checks + 1))
    total_checks=$((total_checks + 1))
    
    # Summary
    echo "=================================="
    echo "Verification Summary:"
    echo "  Total checks: $total_checks"
    echo "  Passed: $passed_checks"
    echo "  Failed: $failed_checks"
    
    if [ $failed_checks -eq 0 ]; then
        print_status "PASS" "All integration checks passed!"
        exit 0
    else
        print_status "FAIL" "Some integration checks failed!"
        exit 1
    fi
}

# Run main function
main "$@"
