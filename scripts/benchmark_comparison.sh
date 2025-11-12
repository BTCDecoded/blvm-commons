#!/bin/bash
# Benchmark Comparison Script: Bitcoin Core vs BTCDecoded
# Designed for laptop-friendly benchmarking (lightweight, resource-conscious)

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/benchmark-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/comparison_${TIMESTAMP}.json"

# Bitcoin Core configuration (adjust paths as needed)
BITCOIN_CORE_DIR="${BITCOIN_CORE_DIR:-/home/user/src/bitcoin}"
BITCOIN_CORE_BIN="${BITCOIN_CORE_BIN:-$BITCOIN_CORE_DIR/src/bitcoind}"
BITCOIN_CLI_BIN="${BITCOIN_CLI_BIN:-$BITCOIN_CORE_DIR/src/bitcoin-cli}"
BITCOIN_DATA_DIR="${BITCOIN_DATA_DIR:-/tmp/bitcoin-benchmark-data}"

# BTCDecoded paths
BLLVM_CONSENSUS_DIR="$PROJECT_ROOT/bllvm-consensus"
BLLVM_NODE_DIR="$PROJECT_ROOT/bllvm-node"

# Create results directory
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}=== Bitcoin Core vs BTCDecoded Benchmark Comparison ===${NC}"
echo -e "${YELLOW}Timestamp: ${TIMESTAMP}${NC}"
echo ""

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"

# Check Bitcoin Core
if [ ! -f "$BITCOIN_CORE_BIN" ]; then
    echo -e "${RED}ERROR: Bitcoin Core binary not found at: $BITCOIN_CORE_BIN${NC}"
    echo "Please set BITCOIN_CORE_BIN environment variable"
    exit 1
fi

if [ ! -f "$BITCOIN_CLI_BIN" ]; then
    echo -e "${RED}ERROR: bitcoin-cli not found at: $BITCOIN_CLI_BIN${NC}"
    exit 1
fi

# Check BTCDecoded
if [ ! -d "$BLLVM_CONSENSUS_DIR" ]; then
    echo -e "${RED}ERROR: bllvm-consensus directory not found${NC}"
    exit 1
fi

# Check Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: cargo not found. Please install Rust${NC}"
    exit 1
fi

# Check Python (for comparison script)
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}ERROR: python3 not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓ All prerequisites found${NC}"
echo ""

# Function to run BTCDecoded benchmarks
run_btdcoded_benchmarks() {
    echo -e "${BLUE}Running BTCDecoded benchmarks...${NC}"
    
    local bench_results_dir="$RESULTS_DIR/btdcoded_${TIMESTAMP}"
    mkdir -p "$bench_results_dir"
    
    # Run bllvm-consensus benchmarks
    echo -e "${YELLOW}  → Running bllvm-consensus benchmarks...${NC}"
    cd "$BLLVM_CONSENSUS_DIR"
    
    # Run with production features for optimized performance
    cargo bench --features production -- \
        --output-format json \
        --save-baseline laptop-baseline \
        > "$bench_results_dir/consensus_proof.json" 2>&1 || {
        echo -e "${RED}  ✗ Consensus-proof benchmarks failed${NC}"
        return 1
    }
    
    # Run bllvm-node benchmarks if available
    if [ -d "$BLLVM_NODE_DIR" ]; then
        echo -e "${YELLOW}  → Running bllvm-node benchmarks...${NC}"
        cd "$BLLVM_NODE_DIR"
        cargo bench -- \
            --output-format json \
            --save-baseline laptop-baseline \
            > "$bench_results_dir/reference_node.json" 2>&1 || {
            echo -e "${YELLOW}  ⚠ Reference-node benchmarks failed (non-critical)${NC}"
        }
    fi
    
    echo -e "${GREEN}✓ BTCDecoded benchmarks complete${NC}"
    echo "$bench_results_dir"
}

# Function to run Bitcoin Core benchmarks (lightweight)
run_bitcoin_core_benchmarks() {
    echo -e "${BLUE}Running Bitcoin Core benchmarks...${NC}"
    
    local bench_results_dir="$RESULTS_DIR/bitcoin_core_${TIMESTAMP}"
    mkdir -p "$bench_results_dir"
    
    # Use Python script to benchmark Bitcoin Core
    echo -e "${YELLOW}  → Running Bitcoin Core validation benchmarks...${NC}"
    
    python3 "$SCRIPT_DIR/benchmark_bitcoin_core.py" \
        --bitcoind "$BITCOIN_CORE_BIN" \
        --bitcoin-cli "$BITCOIN_CLI_BIN" \
        --data-dir "$BITCOIN_DATA_DIR" \
        --output "$bench_results_dir/bitcoin_core.json" || {
        echo -e "${RED}  ✗ Bitcoin Core benchmarks failed${NC}"
        return 1
    }
    
    echo -e "${GREEN}✓ Bitcoin Core benchmarks complete${NC}"
    echo "$bench_results_dir"
}

# Function to compare results
compare_results() {
    local btdcoded_dir="$1"
    local core_dir="$2"
    
    echo -e "${BLUE}Comparing results...${NC}"
    
    python3 "$SCRIPT_DIR/compare_benchmarks.py" \
        --btdcoded "$btdcoded_dir" \
        --bitcoin-core "$core_dir" \
        --output "$RESULTS_FILE" || {
        echo -e "${RED}Comparison failed${NC}"
        return 1
    }
    
    echo -e "${GREEN}✓ Comparison complete${NC}"
    echo -e "${GREEN}Results saved to: ${RESULTS_FILE}${NC}"
}

# Main execution
main() {
    # Run BTCDecoded benchmarks
    BTDC_D_RESULTS=$(run_btdcoded_benchmarks)
    
    # Run Bitcoin Core benchmarks
    CORE_RESULTS=$(run_bitcoin_core_benchmarks)
    
    # Compare results
    compare_results "$BTDC_D_RESULTS" "$CORE_RESULTS"
    
    echo ""
    echo -e "${GREEN}=== Benchmark Comparison Complete ===${NC}"
    echo -e "${BLUE}Results: ${RESULTS_FILE}${NC}"
    echo ""
    echo -e "${YELLOW}To view results:${NC}"
    echo "  python3 $SCRIPT_DIR/view_results.py $RESULTS_FILE"
}

# Run main function
main






