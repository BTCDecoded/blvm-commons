#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

repos=(
    "consensus-proof"
    "developer-sdk"
    "protocol-engine"
    "reference-node"
    "governance"
    "governance-app"
    ".github"
    "the-orange-paper"
    "website"
)

echo "================================================"
echo "Checking Git Status for All Repositories"
echo "================================================"
echo ""

for repo in "${repos[@]}"; do
    echo -e "${YELLOW}=== Checking: $repo ===${NC}"
    cd "/home/user/src/BTCDecoded/$repo"
    
    # Check if there are any changes
    if [[ -n $(git status --porcelain) ]]; then
        echo -e "${RED}Repository has uncommitted changes:${NC}"
        git status --short
        echo ""
    else
        echo -e "${GREEN}Repository is clean (no uncommitted changes)${NC}"
    fi
    
    # Check if there are unpushed commits
    LOCAL=$(git rev-parse @ 2>/dev/null)
    REMOTE=$(git rev-parse @{u} 2>/dev/null)
    
    if [ "$LOCAL" != "$REMOTE" ] 2>/dev/null; then
        echo -e "${YELLOW}Repository has unpushed commits${NC}"
        git log --oneline @{u}..@ 2>/dev/null | head -5
    else
        echo -e "${GREEN}Repository is up to date with remote${NC}"
    fi
    
    echo ""
done

echo "================================================"
echo "Summary Complete"
echo "================================================"
