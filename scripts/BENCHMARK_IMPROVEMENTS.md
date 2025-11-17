# Benchmark Improvements Summary

This document describes the low-hanging fruit improvements made to BTCDecoded's benchmarking capabilities to make it easier to compare with Bitcoin Core.

## Improvements Made

### 1. Enhanced Criterion JSON Parser ✅

**File**: `scripts/compare_benchmarks.py`

**Improvements**:
- Extracts **min, max, median** statistics (in addition to mean)
- Automatically calculates **throughput (ops/sec)** from latency
- Extracts **sample count** for statistical validity
- Better error messages with stack traces for debugging
- Converts all times to milliseconds for easier comparison

**Benefits**:
- More comprehensive statistics for analysis
- Throughput metrics make it easier to understand performance
- Better debugging when parsing fails

### 2. Block Validation Metrics Added ✅

**File**: `scripts/compare_benchmarks.py`

**Improvements**:
- Added support for `connect_block` and `connect_block_multi_tx` benchmarks
- Includes block validation in comparison output
- Shows results even when Core benchmark is not available

**Benefits**:
- Now tracks block validation performance (previously missing)
- Can compare block validation between systems
- Shows standalone BTCDecoded results when Core unavailable

### 3. Throughput Calculations ✅

**Files**: `scripts/compare_benchmarks.py`, `scripts/view_results.py`

**Improvements**:
- Automatically calculates ops/sec from mean latency
- Displays throughput in human-readable format (M ops/sec, K ops/sec)
- Included in comparison output

**Benefits**:
- Easier to understand performance in terms of throughput
- More intuitive than just latency numbers
- Matches how many performance discussions frame results

### 4. Standalone Benchmark Exporter ✅

**File**: `scripts/export_benchmarks.py` (NEW)

**Features**:
- Converts Criterion JSON to Core-compatible format
- Maps BTCDecoded benchmark names to Core-like names
- Includes system metadata (CPU, Rust version, platform)
- Can export without Core being available

**Usage**:
```bash
# Run benchmarks
cd bllvm-consensus
cargo bench --features production -- --output-format json > benchmarks.json

# Export in Core-compatible format
python3 scripts/export_benchmarks.py \
    --input benchmarks.json \
    --output btdcoded_benchmarks_core_format.json
```

**Benefits**:
- Export benchmarks independently of Core
- Core-compatible format makes comparison easier
- Can share benchmark results in standard format
- Includes metadata for reproducibility

### 5. Improved Comparison Output ✅

**Files**: `scripts/compare_benchmarks.py`, `scripts/view_results.py`

**Improvements**:
- Shows throughput alongside latency
- Displays notes explaining measurement differences
- Better formatting for block validation (handles multiple benchmarks)
- More informative error messages

**Benefits**:
- Clearer understanding of what's being measured
- Notes explain why comparisons might be unfair (e.g., hash operations)
- Better visualization of results

## Usage Examples

### Export Benchmarks for Comparison

```bash
# 1. Run benchmarks with production features
cd bllvm-consensus
cargo bench --features production -- --output-format json > /tmp/btdcoded_benchmarks.json

# 2. Export in Core-compatible format
python3 scripts/export_benchmarks.py \
    --input /tmp/btdcoded_benchmarks.json \
    --output /tmp/btdcoded_core_format.json

# 3. Compare with Core (if available)
python3 scripts/compare_benchmarks.py \
    --btdcoded /tmp/btdcoded_benchmarks.json \
    --bitcoin-core /path/to/core/results \
    --output comparison.json \
    --text
```

### View Results

```bash
# View comparison results
python3 scripts/view_results.py comparison.json
```

## What's Still Missing (Future Improvements)

1. **System Metadata Collection**: Currently basic, could be more comprehensive
2. **Percentile Statistics**: P50, P95, P99 percentiles (Criterion supports this)
3. **Memory Usage Tracking**: Not currently measured
4. **Warmup Detection**: Better handling of cold vs warm benchmarks
5. **Automated Regression Detection**: Compare against historical baselines

## Notes

- **Hash Operations Comparison**: The comparison script now includes a note explaining that Core's `getblockhash` measures cached database lookups, not actual hash computation. This is important context for fair comparison.

- **Block Validation**: Block validation benchmarks are now included, but Core's benchmark measures block generation (mining), not validation. This is noted in the output.

- **Production Features**: Always run benchmarks with `--features production` for fair comparison, as this enables optimizations that significantly improve performance.















