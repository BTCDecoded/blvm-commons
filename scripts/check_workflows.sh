#!/bin/bash
# GitHub Workflow Status Checker for BTCDecoded Organization
# Uses curl to directly query GitHub API for workflow runs and logs

set -euo pipefail

# Configuration
GITHUB_TOKEN="${GITHUB_TOKEN:-}"
ORG="BTCDecoded"
BRANCH="${BRANCH:-main}"
LIMIT="${LIMIT:-5}"

# Repository name mapping (workflow names -> actual repo names)
declare -A REPO_MAP=(
    ["consensus-proof"]="bllvm-consensus"
    ["protocol-engine"]="bllvm-protocol"
    ["reference-node"]="bllvm-node"
    ["developer-sdk"]="bllvm-sdk"
)

# Repositories in build order (from versions.toml dependencies)
# Level 0: No dependencies
# Level 1: Depends on level 0
# Level 2: Depends on level 1
# etc.
REPOS=(
    "bllvm-consensus"      # Level 0
    "bllvm-sdk"            # Level 0
    "bllvm-protocol"       # Level 1 (depends on bllvm-consensus)
    "bllvm-node"            # Level 2 (depends on bllvm-protocol, bllvm-consensus)
    "governance-app"       # Level 1 (depends on bllvm-sdk)
    "commons"
    "governance"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# GitHub API functions
gh_api() {
    local endpoint="$1"
    local url="https://api.github.com${endpoint}"
    
    curl -s -H "Authorization: token ${GITHUB_TOKEN}" \
         -H "Accept: application/vnd.github.v3+json" \
         "$url"
}

gh_api_paginated() {
    local endpoint="$1"
    local page=1
    local per_page=100
    local all_results="[]"
    local results
    
    while true; do
        results=$(gh_api "${endpoint}?page=${page}&per_page=${per_page}")
        
        # Check if we got results
        if [ "$(echo "$results" | jq -r 'if type=="array" then length else 0 end')" -eq 0 ]; then
            break
        fi
        
        # Merge results
        all_results=$(echo "$all_results $results" | jq -s 'add')
        
        # Check if we got fewer results than requested (last page)
        if [ "$(echo "$results" | jq -r 'if type=="array" then length else 0 end')" -lt "$per_page" ]; then
            break
        fi
        
        page=$((page + 1))
    done
    
    echo "$all_results"
}

# Check workflow runs for a repository
check_repo_workflows() {
    local repo="$1"
    echo -e "${BLUE}📦 Repository: ${repo}${NC}"
    
    # Get workflow runs
    local runs_json
    runs_json=$(gh_api "/repos/${ORG}/${repo}/actions/runs?branch=${BRANCH}&per_page=${LIMIT}")
    
    # Check if repo exists
    if echo "$runs_json" | jq -e '.message' > /dev/null 2>&1; then
        local msg=$(echo "$runs_json" | jq -r '.message')
        if [[ "$msg" == *"Not Found"* ]]; then
            echo -e "  ${YELLOW}⚠️  Repository not found on GitHub${NC}"
            return
        fi
    fi
    
    local runs=$(echo "$runs_json" | jq -r '.workflow_runs // []')
    local run_count=$(echo "$runs" | jq 'length')
    
    echo -e "  Found ${run_count} recent workflow runs"
    
    if [ "$run_count" -eq 0 ]; then
        echo ""
        return
    fi
    
    # Process each run
    echo "$runs" | jq -c '.[]' | while IFS= read -r run; do
        local run_id=$(echo "$run" | jq -r '.id')
        local name=$(echo "$run" | jq -r '.name')
        local status=$(echo "$run" | jq -r '.status')
        local conclusion=$(echo "$run" | jq -r '.conclusion // "unknown"')
        local created=$(echo "$run" | jq -r '.created_at')
        local html_url=$(echo "$run" | jq -r '.html_url')
        
        echo -e "\n  Run: ${name} (#${run_id})"
        echo -e "     Status: ${status}"
        echo -e "     Conclusion: ${conclusion}"
        echo -e "     Created: ${created}"
        echo -e "     URL: ${html_url}"
        
        # Get jobs for this run
        local jobs_json
        jobs_json=$(gh_api "/repos/${ORG}/${repo}/actions/runs/${run_id}/jobs")
        local jobs=$(echo "$jobs_json" | jq -r '.jobs // []')
        local job_count=$(echo "$jobs" | jq 'length')
        
        echo -e "     Jobs: ${job_count}"
        
        # Show job statuses
        echo "$jobs" | jq -c '.[]' | while IFS= read -r job; do
            local job_name=$(echo "$job" | jq -r '.name')
            local job_status=$(echo "$job" | jq -r '.status')
            local job_conclusion=$(echo "$job" | jq -r '.conclusion // "unknown"')
            
            local icon="⏳"
            if [ "$job_conclusion" = "success" ]; then
                icon="${GREEN}✅${NC}"
            elif [ "$job_conclusion" = "failure" ]; then
                icon="${RED}❌${NC}"
            fi
            
            echo -e "        ${icon} ${job_name}: ${job_status} / ${job_conclusion}"
        done
        
        # If failed, get log download URL
        if [ "$conclusion" = "failure" ]; then
            echo -e "     ${YELLOW}📥 Failed run - log URL:${NC}"
            local logs_url=$(echo "$run" | jq -r '.logs_url')
            echo -e "        ${logs_url}"
            echo -e "        ${YELLOW}(Use: curl -H 'Authorization: token \${GITHUB_TOKEN}' -L '${logs_url}' -o logs.zip)${NC}"
        fi
    done
    
    echo ""
}

# Download logs for a specific run
download_logs() {
    local repo="$1"
    local run_id="$2"
    local output_dir="${3:-./workflow-logs}"
    
    mkdir -p "$output_dir"
    
    echo -e "${BLUE}📥 Downloading logs for ${repo} run #${run_id}...${NC}"
    
    # Get the logs URL (GitHub redirects to actual download)
    local logs_url="https://api.github.com/repos/${ORG}/${repo}/actions/runs/${run_id}/logs"
    
    # Follow redirect and download
    local output_file="${output_dir}/${repo}-${run_id}-logs.zip"
    
    if curl -s -L -H "Authorization: token ${GITHUB_TOKEN}" \
            -H "Accept: application/vnd.github.v3+json" \
            "$logs_url" -o "$output_file"; then
        
        if [ -f "$output_file" ] && [ -s "$output_file" ]; then
            echo -e "${GREEN}✅ Downloaded to: ${output_file}${NC}"
            
            # Extract if it's a zip file
            if file "$output_file" | grep -q "Zip archive"; then
                local extract_dir="${output_dir}/${repo}-${run_id}-logs"
                mkdir -p "$extract_dir"
                unzip -q -o "$output_file" -d "$extract_dir" 2>/dev/null || true
                echo -e "${GREEN}✅ Extracted to: ${extract_dir}${NC}"
            fi
        else
            echo -e "${RED}❌ Failed to download logs${NC}"
        fi
    else
        echo -e "${RED}❌ Error downloading logs${NC}"
    fi
}

# Main function
main() {
    if [ -z "$GITHUB_TOKEN" ]; then
        echo -e "${RED}❌ Error: GITHUB_TOKEN not set${NC}"
        echo "Usage: GITHUB_TOKEN=your_token $0 [repo_name] [run_id]"
        echo ""
        echo "Options:"
        echo "  repo_name  - Check specific repository (default: all repos)"
        echo "  run_id     - Download logs for specific run (requires repo_name)"
        echo ""
        echo "Environment variables:"
        echo "  GITHUB_TOKEN - GitHub personal access token (required)"
        echo "  BRANCH       - Branch to check (default: main)"
        echo "  LIMIT        - Max runs per repo (default: 5)"
        exit 1
    fi
    
    # Check if jq is available
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}❌ Error: jq is required but not installed${NC}"
        echo "Install with: sudo apt install jq"
        exit 1
    fi
    
    # If specific repo and run_id provided, download logs
    if [ $# -ge 2 ]; then
        local repo="$1"
        local run_id="$2"
        download_logs "$repo" "$run_id"
        exit 0
    fi
    
    # If specific repo provided, check only that repo
    if [ $# -eq 1 ]; then
        check_repo_workflows "$1"
        exit 0
    fi
    
    # Otherwise check all repos
    echo -e "${BLUE}🔍 Checking workflows for BTCDecoded organization${NC}"
    echo -e "Branch: ${BRANCH}, Limit: ${LIMIT} runs per repo"
    echo "============================================================"
    echo ""
    
    local total_failed=0
    local total_success=0
    
    for repo in "${REPOS[@]}"; do
        check_repo_workflows "$repo"
    done
    
    echo "============================================================"
    echo -e "${BLUE}📊 Summary${NC}"
    echo ""
    echo "To download logs for a failed run:"
    echo "  $0 <repo_name> <run_id>"
    echo ""
    echo "Example:"
    echo "  $0 bllvm-consensus 12345678"
}

main "$@"

