#!/bin/bash
# Blocking workflow monitor - waits for workflows to complete and fixes issues automatically

set -euo pipefail

GITHUB_TOKEN="${GITHUB_TOKEN:-}"
CHECK_INTERVAL="${CHECK_INTERVAL:-30}"  # Check every 30 seconds
MAX_WAIT="${MAX_WAIT:-3600}"  # Max 1 hour wait per workflow run

if [ -z "$GITHUB_TOKEN" ]; then
    echo "❌ Error: GITHUB_TOKEN not set"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECK_SCRIPT="$SCRIPT_DIR/check_workflows.sh"
ORG="BTCDecoded"
REPOS=("bllvm-consensus" "bllvm-protocol" "bllvm-node" "bllvm-sdk" "governance-app" "commons")

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

gh_api() {
    local endpoint="$1"
    curl -s -H "Authorization: token ${GITHUB_TOKEN}" \
         -H "Accept: application/vnd.github.v3+json" \
         "https://api.github.com${endpoint}"
}

wait_for_run_completion() {
    local repo="$1"
    local run_id="$2"
    local start_time=$(date +%s)
    local elapsed=0
    local last_status=""
    
    echo -e "${CYAN}⏳ Waiting for ${repo} run #${run_id} to complete...${NC}"
    
    while [ $elapsed -lt $MAX_WAIT ]; do
        local run_data=$(gh_api "/repos/${ORG}/${repo}/actions/runs/${run_id}")
        local status=$(echo "$run_data" | jq -r '.status // "unknown"')
        local conclusion=$(echo "$run_data" | jq -r '.conclusion // "unknown"')
        
        if [ "$status" != "$last_status" ]; then
            echo -e "   Status: ${status}"
            last_status="$status"
        fi
        
        if [ "$status" = "completed" ]; then
            echo -e "${GREEN}✅ Run #${run_id} completed with conclusion: ${conclusion}${NC}"
            echo "$conclusion"
            return 0
        fi
        
        elapsed=$(($(date +%s) - start_time))
        local minutes=$((elapsed / 60))
        local seconds=$((elapsed % 60))
        
        # Only print elapsed time every 2 minutes to avoid spam
        if [ $((elapsed % 120)) -eq 0 ] && [ $elapsed -gt 0 ]; then
            echo -e "   Still running... (${minutes}m ${seconds}s elapsed)"
        fi
        
        sleep "$CHECK_INTERVAL"
    done
    
    echo -e "${RED}❌ Timeout waiting for run #${run_id}${NC}"
    echo "unknown"
    return 1
}

get_latest_run() {
    local repo="$1"
    local runs_data=$(gh_api "/repos/${ORG}/${repo}/actions/runs?per_page=1")
    local run_id=$(echo "$runs_data" | jq -r '.workflow_runs[0].id // empty')
    local status=$(echo "$runs_data" | jq -r '.workflow_runs[0].status // "unknown"')
    local conclusion=$(echo "$runs_data" | jq -r '.workflow_runs[0].conclusion // "unknown"')
    local created_at=$(echo "$runs_data" | jq -r '.workflow_runs[0].created_at // "unknown"')
    
    if [ -n "$run_id" ] && [ "$run_id" != "null" ]; then
        echo "${run_id}|${status}|${conclusion}|${created_at}"
    fi
}

analyze_and_fix_failed_run() {
    local repo="$1"
    local run_id="$2"
    
    echo -e "${YELLOW}🔍 Analyzing failed run #${run_id} for ${repo}...${NC}"
    
    # Download logs
    "$CHECK_SCRIPT" "$repo" "$run_id" > /dev/null 2>&1 || true
    
    # Find log files
    local log_dir="workflow-logs/${repo}-${run_id}-logs"
    if [ ! -d "$log_dir" ]; then
        echo -e "${RED}   ⚠️  Could not find log directory${NC}"
        return 1
    fi
    
    # Check for compilation errors
    local test_log=$(find "$log_dir" -name "*Test*.txt" -o -name "*Build*.txt" 2>/dev/null | head -1)
    local errors_found=0
    
    if [ -n "$test_log" ]; then
        # Check for missing module errors
        if grep -q "could not find.*in the crate root\|use of unresolved module.*unlinked crate" "$test_log" 2>/dev/null; then
            echo -e "${YELLOW}   Found missing module/unresolved import errors${NC}"
            local error_lines=$(grep -i "error\[E043" "$test_log" 2>/dev/null | head -3)
            if [ -n "$error_lines" ]; then
                echo "$error_lines" | sed 's/^/      /'
            fi
            errors_found=1
        fi
        
        # Check for dependency path errors
        if grep -q "failed to get.*as a dependency\|Unable to update.*No such file" "$test_log" 2>/dev/null; then
            echo -e "${YELLOW}   Found dependency path errors${NC}"
            errors_found=1
        fi
        
        # Check for old crate name usage
        if grep -q "consensus_proof\|protocol_engine\|reference_node" "$test_log" 2>/dev/null; then
            echo -e "${YELLOW}   Found old crate name usage in code${NC}"
            errors_found=1
        fi
    fi
    
    # Try to fix issues
    if [ $errors_found -eq 1 ]; then
        fix_repo_issues "$repo" "$log_dir"
    fi
    
    return $errors_found
}

fix_repo_issues() {
    local repo="$1"
    local log_dir="$2"
    
    echo -e "${BLUE}🔧 Attempting to fix issues in ${repo}...${NC}"
    
    # Check if repo exists locally
    local repo_dir=""
    if [ -d "../${repo}" ]; then
        repo_dir="../${repo}"
    elif [ -d "${repo}" ]; then
        repo_dir="${repo}"
    else
        echo -e "${YELLOW}   ⚠️  Repository not found locally, skipping fixes${NC}"
        return 0
    fi
    
    cd "$repo_dir" || return 1
    
    # Check for old crate name in test files
    if find tests -name "*.rs" -exec grep -l "consensus_proof::\|protocol_engine::\|reference_node::" {} \; 2>/dev/null | head -1 | grep -q .; then
        echo -e "${BLUE}   Fixing old crate names in test files...${NC}"
        find tests -name "*.rs" -exec sed -i 's/consensus_proof::/bllvm_consensus::/g; s/protocol_engine::/bllvm_protocol::/g; s/reference_node::/bllvm_node::/g' {} \;
        
        if git diff --quiet tests/ 2>/dev/null; then
            echo -e "${GREEN}   ✅ No changes needed${NC}"
        else
            echo -e "${GREEN}   ✅ Fixed test imports${NC}"
            git add tests/ 2>/dev/null || true
            git commit -m "fix: Update test imports to use new crate names" 2>/dev/null || true
            git push origin main 2>/dev/null && echo -e "${GREEN}   ✅ Pushed fixes${NC}" || echo -e "${YELLOW}   ⚠️  Could not push (may need manual intervention)${NC}"
        fi
    fi
    
    # Check for missing module declarations in lib.rs
    local test_log=$(find "../workflow-logs/${repo}-"*"-logs" -name "*Build*.txt" 2>/dev/null | head -1)
    if [ -n "$test_log" ] && grep -q "could not find.*in the crate root" "$test_log" 2>/dev/null; then
        local missing_module=$(grep "could not find.*in the crate root" "$test_log" 2>/dev/null | head -1 | sed -n 's/.*could not find `\([^`]*\)`.*/\1/p')
        if [ -n "$missing_module" ] && [ -f "src/${missing_module}/mod.rs" ] || [ -f "src/${missing_module}.rs" ]; then
            echo -e "${BLUE}   Found missing module '${missing_module}', checking lib.rs...${NC}"
            if [ -f "src/lib.rs" ] && ! grep -q "pub mod ${missing_module}" "src/lib.rs" 2>/dev/null; then
                echo -e "${BLUE}   Adding missing module declaration...${NC}"
                # Find a good place to add it (after other mod declarations)
                local insert_line=$(grep -n "^pub mod " src/lib.rs 2>/dev/null | tail -1 | cut -d: -f1)
                if [ -n "$insert_line" ]; then
                    sed -i "${insert_line}a pub mod ${missing_module};" src/lib.rs
                    git add src/lib.rs 2>/dev/null || true
                    git commit -m "fix: Add missing ${missing_module} module declaration" 2>/dev/null || true
                    git push origin main 2>/dev/null && echo -e "${GREEN}   ✅ Fixed and pushed${NC}" || echo -e "${YELLOW}   ⚠️  Could not push${NC}"
                fi
            fi
        fi
    fi
    
    # Check workflow files for old repo names
    if [ -f ".github/workflows/ci.yml" ]; then
        if grep -q "BTCDecoded/consensus-proof\|BTCDecoded/protocol-engine\|BTCDecoded/reference-node" ".github/workflows/ci.yml" 2>/dev/null; then
            echo -e "${BLUE}   Fixing old repository names in workflow...${NC}"
            sed -i 's|BTCDecoded/consensus-proof|BTCDecoded/bllvm-consensus|g; s|BTCDecoded/protocol-engine|BTCDecoded/bllvm-protocol|g; s|BTCDecoded/reference-node|BTCDecoded/bllvm-node|g' .github/workflows/ci.yml
            sed -i 's|../consensus-proof|../bllvm-consensus|g; s|../protocol-engine|../bllvm-protocol|g; s|../reference-node|../bllvm-node|g' .github/workflows/ci.yml
            
            if git diff --quiet .github/workflows/ci.yml 2>/dev/null; then
                echo -e "${GREEN}   ✅ No changes needed${NC}"
            else
                git add .github/workflows/ci.yml 2>/dev/null || true
                git commit -m "fix: Update workflow to use correct repository names" 2>/dev/null || true
                git push origin main 2>/dev/null && echo -e "${GREEN}   ✅ Fixed and pushed workflow${NC}" || echo -e "${YELLOW}   ⚠️  Could not push${NC}"
            fi
        fi
    fi
    
    cd - > /dev/null
    return 0
}

monitor_repo() {
    local repo="$1"
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}📦 Monitoring: ${repo}${NC}"
    echo -e "${BLUE}========================================${NC}"
    
    # Get latest run
    local run_info=$(get_latest_run "$repo")
    if [ -z "$run_info" ]; then
        echo -e "${YELLOW}   ⚠️  No workflow runs found${NC}"
        return 0
    fi
    
    IFS='|' read -r run_id status conclusion created_at <<< "$run_info"
    
    echo -e "${CYAN}   Latest run: #${run_id}${NC}"
    echo -e "      Created: ${created_at}"
    echo -e "      Status: ${status}"
    echo -e "      Conclusion: ${conclusion}"
    
    # If still running, wait for it
    if [ "$status" != "completed" ]; then
        conclusion=$(wait_for_run_completion "$repo" "$run_id")
    fi
    
    # If failed, analyze and try to fix
    if [ "$conclusion" = "failure" ]; then
        echo -e "${RED}   ❌ Run failed${NC}"
        if analyze_and_fix_failed_run "$repo" "$run_id"; then
            echo -e "${GREEN}   ✅ Attempted fixes, waiting for new run...${NC}"
            # Wait a bit for new run to start
            sleep 10
            # Check for new run
            local new_run_info=$(get_latest_run "$repo")
            if [ -n "$new_run_info" ]; then
                IFS='|' read -r new_run_id new_status new_conclusion new_created <<< "$new_run_info"
                if [ "$new_run_id" != "$run_id" ]; then
                    echo -e "${CYAN}   New run detected: #${new_run_id}, monitoring...${NC}"
                    if [ "$new_status" != "completed" ]; then
                        new_conclusion=$(wait_for_run_completion "$repo" "$new_run_id")
                    fi
                    if [ "$new_conclusion" = "success" ]; then
                        echo -e "${GREEN}   ✅ Fix successful! New run passed${NC}"
                    elif [ "$new_conclusion" = "failure" ]; then
                        echo -e "${YELLOW}   ⚠️  New run also failed, may need manual intervention${NC}"
                    fi
                fi
            fi
        fi
    elif [ "$conclusion" = "success" ]; then
        echo -e "${GREEN}   ✅ Run succeeded${NC}"
    fi
}

# Main monitoring loop
main() {
    echo "🔍 Starting blocking workflow monitor"
    echo "   Check interval: ${CHECK_INTERVAL} seconds"
    echo "   Max wait per run: ${MAX_WAIT} seconds"
    echo ""
    
    while true; do
        local any_failures=0
        
        for repo in "${REPOS[@]}"; do
            monitor_repo "$repo"
            # Small delay between repos
            sleep 2
        done
        
        echo ""
        echo -e "${BLUE}========================================${NC}"
        echo -e "${BLUE}⏳ Waiting ${CHECK_INTERVAL}s before next check...${NC}"
        echo -e "${BLUE}========================================${NC}"
        sleep "$CHECK_INTERVAL"
    done
}

main "$@"
