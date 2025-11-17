# Benchmarking Setup: Bitcoin Core vs BTCDecoded

This directory contains scripts for comparing performance between Bitcoin Core and BTCDecoded on a laptop-friendly setup.

## Quick Start

### Prerequisites

1. **Bitcoin Core** compiled locally
   - Set `BITCOIN_CORE_DIR` environment variable (default: `/home/user/src/bitcoin`)
   - Or set `BITCOIN_CORE_BIN` and `BITCOIN_CLI_BIN` directly

2. **BTCDecoded** compiled locally
   - Consensus-proof and bllvm-node crates must be built

3. **Python 3** with standard library
4. **Rust** and Cargo installed

### Running Benchmarks

```bash
# From project root
cd /home/user/src/BTCDecoded
./scripts/benchmark_comparison.sh
```

This will:
1. Run BTCDecoded benchmarks (Criterion)
2. Run Bitcoin Core benchmarks (lightweight regtest)
3. Compare results and generate a JSON report

### Viewing Results

```bash
# View latest results
python3 scripts/view_results.py benchmark-results/comparison_*.json
```

## Configuration

### Environment Variables

```bash
# Bitcoin Core paths
export BITCOIN_CORE_DIR="/home/user/src/bitcoin"
export BITCOIN_CORE_BIN="/home/user/src/bitcoin/src/bitcoind"
export BITCOIN_CLI_BIN="/home/user/src/bitcoin/src/bitcoin-cli"

# Bitcoin Core data directory (regtest)
export BITCOIN_DATA_DIR="/tmp/bitcoin-benchmark-data"
```

### BTCDecoded Benchmarks

The script automatically runs:
- `bllvm-consensus` benchmarks (with `--features production`)
- `bllvm-node` benchmarks (if available)

Results are saved to `benchmark-results/btdcoded_<timestamp>/`

### Bitcoin Core Benchmarks

The script runs lightweight benchmarks using regtest mode:
- Transaction validation (via `testmempoolaccept`)
- Block validation (via `generate`)
- Hash operations (via `getblockhash` as proxy)

Results are saved to `benchmark-results/bitcoin_core_<timestamp>/`

## Laptop-Friendly Design

These benchmarks are designed to run on a laptop with:
- **Minimal resource usage**: Uses regtest (not mainnet)
- **Small datasets**: Limited to 100 blocks/transactions
- **Fast execution**: Each benchmark completes in < 5 minutes
- **No disk I/O**: Uses temporary directories

## Output Format

Results are saved as JSON with the following structure:

```json
{
  "btdcoded": {
    "consensus_proof": {
      "check_transaction": {
        "mean_ns": 53954,
        "lower_bound_ns": 52598,
        "upper_bound_ns": 55473
      }
    }
  },
  "bitcoin_core": {
    "benchmarks": {
      "transaction_validation": {
        "mean_ms": 0.123,
        "min_ms": 0.110,
        "max_ms": 0.150,
        "samples": 100
      }
    }
  },
  "comparison": {
    "transaction_validation": {
      "btdcoded_ms": 0.054,
      "core_ms": 0.123,
      "ratio": 2.28,
      "faster": "BTCDecoded"
    }
  }
}
```

## Limitations

These benchmarks are **laptop-friendly** but have limitations:

1. **Not production-representative**: Uses regtest, not mainnet data
2. **Small datasets**: Limited to 100 blocks/transactions
3. **No network overhead**: Local only, no P2P network
4. **Simplified operations**: Doesn't test full node sync

For production benchmarks, use:
- Full blockchain data
- Network testing
- Multi-hour test runs
- Resource monitoring (CPU, memory, disk I/O)

## Troubleshooting

### Bitcoin Core not found

```bash
# Check if bitcoind exists
ls -la $BITCOIN_CORE_BIN

# Build Bitcoin Core if needed
cd $BITCOIN_CORE_DIR
./autogen.sh
./configure
make
```

### BTCDecoded benchmarks fail

```bash
# Build BTCDecoded with production features
cd bllvm-consensus
cargo build --release --features production

# Run benchmarks manually to debug
cargo bench --features production
```

### Permission errors

```bash
# Make scripts executable
chmod +x scripts/*.sh scripts/*.py
```

### Regtest data directory issues

```bash
# Clean up old regtest data
rm -rf /tmp/bitcoin-benchmark-data
```

## Advanced Usage

### Run specific benchmarks only

```bash
# BTCDecoded only
cd bllvm-consensus
cargo bench --features production

# Bitcoin Core only
python3 scripts/benchmark_bitcoin_core.py \
  --bitcoind $BITCOIN_CORE_BIN \
  --bitcoin-cli $BITCOIN_CLI_BIN \
  --data-dir /tmp/bitcoin-benchmark-data \
  --output results.json
```

### Custom comparison

```bash
python3 scripts/compare_benchmarks.py \
  --btdcoded benchmark-results/btdcoded_<timestamp> \
  --bitcoin-core benchmark-results/bitcoin_core_<timestamp> \
  --output comparison.json \
  --text
```

## Next Steps

1. **Run benchmarks**: `./scripts/benchmark_comparison.sh`
2. **View results**: `python3 scripts/view_results.py benchmark-results/comparison_*.json`
3. **Compare over time**: Run multiple times and compare results
4. **Production testing**: Use full blockchain data for production benchmarks

















