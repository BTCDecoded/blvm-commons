#!/bin/bash

set -e

# Configuration
SERVER_ID=${1:-"governance-01"}
NOSTR_NPUB=${2:-""}
GOVERNANCE_URL="https://btcdecoded.org/governance"
REGISTRY_URL="$GOVERNANCE_URL/registries"
NOSTR_RELAY="wss://relay.damus.io"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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
            echo -e "${BLUE}ℹ${NC} $message"
            ;;
    esac
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to download file
download_file() {
    local url=$1
    local output=$2
    local description=$3
    
    if command_exists curl; then
        if curl -s -o "$output" "$url"; then
            print_status "PASS" "Downloaded $description"
            return 0
        else
            print_status "FAIL" "Failed to download $description"
            return 1
        fi
    elif command_exists wget; then
        if wget -q -O "$output" "$url"; then
            print_status "PASS" "Downloaded $description"
            return 0
        else
            print_status "FAIL" "Failed to download $description"
            return 1
        fi
    else
        print_status "FAIL" "Neither curl nor wget available"
        return 1
    fi
}

# Function to verify server authorization
verify_server_authorization() {
    local server_id=$1
    local registry_file=$2
    
    if [ ! -f "$registry_file" ]; then
        print_status "FAIL" "Registry file not found: $registry_file"
        return 1
    fi
    
    # Check if server exists in registry
    if command_exists jq; then
        if jq -e ".servers[] | select(.server_id == \"$server_id\")" "$registry_file" >/dev/null 2>&1; then
            print_status "PASS" "Server $server_id found in registry"
            
            # Get server status
            local status=$(jq -r ".servers[] | select(.server_id == \"$server_id\") | .status" "$registry_file")
            if [ "$status" = "active" ]; then
                print_status "PASS" "Server $server_id is active"
                return 0
            else
                print_status "WARN" "Server $server_id status: $status"
                return 1
            fi
        else
            print_status "FAIL" "Server $server_id not found in registry"
            return 1
        fi
    else
        print_status "WARN" "jq not available, cannot parse registry"
        return 1
    fi
}

# Function to verify Nostr events
verify_nostr_events() {
    local server_id=$1
    local nostr_npub=$2
    
    if [ -z "$nostr_npub" ]; then
        print_status "WARN" "No Nostr NPUB provided, skipping Nostr verification"
        return 0
    fi
    
    if ! command_exists nostr-cli; then
        print_status "WARN" "nostr-cli not available, skipping Nostr verification"
        return 0
    fi
    
    # Check for recent governance status events
    local events=$(nostr-cli --relay "$NOSTR_RELAY" --filter "{\"kinds\":[30078],\"#server\":[\"$server_id\"]}" --limit 1 2>/dev/null || echo "")
    
    if [ -n "$events" ]; then
        print_status "PASS" "Found Nostr events for server $server_id"
        return 0
    else
        print_status "WARN" "No recent Nostr events found for server $server_id"
        return 1
    fi
}

# Function to verify OTS proof
verify_ots_proof() {
    local registry_file=$1
    local proof_file=$2
    
    if [ ! -f "$proof_file" ]; then
        print_status "WARN" "OTS proof file not found: $proof_file"
        return 1
    fi
    
    if ! command_exists ots; then
        print_status "WARN" "ots-cli not available, skipping OTS verification"
        return 0
    fi
    
    if ots verify "$proof_file" >/dev/null 2>&1; then
        print_status "PASS" "OTS proof verified"
        return 0
    else
        print_status "FAIL" "OTS proof verification failed"
        return 1
    fi
}

# Function to check server health
check_server_health() {
    local server_id=$1
    local registry_file=$2
    
    if [ ! -f "$registry_file" ]; then
        print_status "FAIL" "Registry file not found: $registry_file"
        return 1
    fi
    
    if ! command_exists jq; then
        print_status "WARN" "jq not available, cannot check server health"
        return 0
    fi
    
    # Get server information
    local server_info=$(jq -r ".servers[] | select(.server_id == \"$server_id\")" "$registry_file" 2>/dev/null)
    
    if [ -n "$server_info" ]; then
        local operator=$(echo "$server_info" | jq -r '.operator.name')
        local jurisdiction=$(echo "$server_info" | jq -r '.operator.jurisdiction')
        local last_verified=$(echo "$server_info" | jq -r '.last_verified')
        
        print_status "INFO" "Server Operator: $operator"
        print_status "INFO" "Jurisdiction: $jurisdiction"
        print_status "INFO" "Last Verified: $last_verified"
        
        return 0
    else
        print_status "FAIL" "Server information not found"
        return 1
    fi
}

# Function to display server summary
display_server_summary() {
    local server_id=$1
    local registry_file=$2
    
    echo ""
    echo "Server Summary for $server_id:"
    echo "================================"
    
    if [ -f "$registry_file" ] && command_exists jq; then
        local server_info=$(jq -r ".servers[] | select(.server_id == \"$server_id\")" "$registry_file" 2>/dev/null)
        
        if [ -n "$server_info" ]; then
            echo "Server ID: $server_id"
            echo "Operator: $(echo "$server_info" | jq -r '.operator.name')"
            echo "Jurisdiction: $(echo "$server_info" | jq -r '.operator.jurisdiction')"
            echo "Status: $(echo "$server_info" | jq -r '.status')"
            echo "Added: $(echo "$server_info" | jq -r '.added_at')"
            echo "Last Verified: $(echo "$server_info" | jq -r '.last_verified')"
            
            if [ -n "$NOSTR_NPUB" ]; then
                echo "Nostr NPUB: $NOSTR_NPUB"
            fi
        else
            echo "Server information not available"
        fi
    else
        echo "Registry file not available or jq not installed"
    fi
}

# Main verification process
main() {
    local total_checks=0
    local passed_checks=0
    local failed_checks=0
    
    echo "Verifying server authorization for: $SERVER_ID"
    if [ -n "$NOSTR_NPUB" ]; then
        echo "Nostr NPUB: $NOSTR_NPUB"
    fi
    echo "=================================="
    
    # Create temporary directory
    local temp_dir=$(mktemp -d)
    local registry_file="$temp_dir/registry.json"
    local proof_file="$temp_dir/registry.json.ots"
    
    # Download latest registry
    print_status "INFO" "Downloading latest governance registry..."
    if download_file "$REGISTRY_URL/latest.json" "$registry_file" "governance registry"; then
        passed_checks=$((passed_checks + 1))
    else
        failed_checks=$((failed_checks + 1))
    fi
    total_checks=$((total_checks + 1))
    
    # Download OTS proof
    print_status "INFO" "Downloading OTS proof..."
    if download_file "$REGISTRY_URL/latest.json.ots" "$proof_file" "OTS proof"; then
        passed_checks=$((passed_checks + 1))
    else
        failed_checks=$((failed_checks + 1))
    fi
    total_checks=$((total_checks + 1))
    
    # Verify server authorization
    print_status "INFO" "Verifying server authorization..."
    if verify_server_authorization "$SERVER_ID" "$registry_file"; then
        passed_checks=$((passed_checks + 1))
    else
        failed_checks=$((failed_checks + 1))
    fi
    total_checks=$((total_checks + 1))
    
    # Verify Nostr events
    print_status "INFO" "Verifying Nostr events..."
    if verify_nostr_events "$SERVER_ID" "$NOSTR_NPUB"; then
        passed_checks=$((passed_checks + 1))
    else
        failed_checks=$((failed_checks + 1))
    fi
    total_checks=$((total_checks + 1))
    
    # Verify OTS proof
    print_status "INFO" "Verifying OTS proof..."
    if verify_ots_proof "$registry_file" "$proof_file"; then
        passed_checks=$((passed_checks + 1))
    else
        failed_checks=$((failed_checks + 1))
    fi
    total_checks=$((total_checks + 1))
    
    # Check server health
    print_status "INFO" "Checking server health..."
    if check_server_health "$SERVER_ID" "$registry_file"; then
        passed_checks=$((passed_checks + 1))
    else
        failed_checks=$((failed_checks + 1))
    fi
    total_checks=$((total_checks + 1))
    
    # Display server summary
    display_server_summary "$SERVER_ID" "$registry_file"
    
    # Cleanup
    rm -rf "$temp_dir"
    
    # Summary
    echo ""
    echo "Verification Summary:"
    echo "  Total checks: $total_checks"
    echo "  Passed: $passed_checks"
    echo "  Failed: $failed_checks"
    
    if [ $failed_checks -eq 0 ]; then
        print_status "PASS" "All server verification checks passed!"
        exit 0
    else
        print_status "FAIL" "Some server verification checks failed!"
        exit 1
    fi
}

# Show usage if no arguments
if [ $# -eq 0 ]; then
    echo "Usage: $0 <server_id> [nostr_npub]"
    echo ""
    echo "Arguments:"
    echo "  server_id    Server ID to verify (e.g., governance-01)"
    echo "  nostr_npub   Optional Nostr public key for verification"
    echo ""
    echo "Examples:"
    echo "  $0 governance-01"
    echo "  $0 governance-01 npub1abc123..."
    echo ""
    exit 1
fi

# Run main function
main "$@"
