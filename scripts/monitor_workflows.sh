#!/bin/bash
# Periodic workflow monitor - checks workflows and reports/fixes issues

set -euo pipefail

GITHUB_TOKEN="${GITHUB_TOKEN:-}"
CHECK_INTERVAL="${CHECK_INTERVAL:-300}"  # 5 minutes default
MAX_ITERATIONS="${MAX_ITERATIONS:-12}"   # 1 hour total (12 * 5 min)

if [ -z "$GITHUB_TOKEN" ]; then
    echo "❌ Error: GITHUB_TOKEN not set"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHECK_SCRIPT="$SCRIPT_DIR/check_workflows.sh"

echo "🔍 Starting workflow monitor"
echo "   Check interval: ${CHECK_INTERVAL} seconds"
echo "   Max iterations: ${MAX_ITERATIONS}"
echo ""

iteration=0
while [ $iteration -lt $MAX_ITERATIONS ]; do
    iteration=$((iteration + 1))
    echo "=========================================="
    echo "📊 Check #${iteration} - $(date)"
    echo "=========================================="
    
    # Run the check script
    "$CHECK_SCRIPT" 2>&1 | tee /tmp/workflow_check_$(date +%s).txt
    
    # Count failures
    failed_count=$(grep -c "❌" /tmp/workflow_check_*.txt 2>/dev/null | tail -1 | awk '{print $NF}' || echo "0")
    
    if [ "$failed_count" -gt 0 ]; then
        echo ""
        echo "⚠️  Found ${failed_count} failed jobs"
        echo "   Review logs and fix issues as needed"
    else
        echo ""
        echo "✅ All workflows passing!"
    fi
    
    if [ $iteration -lt $MAX_ITERATIONS ]; then
        echo ""
        echo "⏳ Waiting ${CHECK_INTERVAL} seconds until next check..."
        sleep "$CHECK_INTERVAL"
    fi
done

echo ""
echo "✅ Monitoring complete"

