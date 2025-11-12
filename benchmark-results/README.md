# Benchmark Results

This directory contains benchmark results and comparison reports.

## Structure

### Current Results
- `benchmark_summary.json` - Latest benchmark summary
- `BTCDECODED_BENCHMARKS.md` - Benchmark documentation
- `SUMMARY.md` - Summary of results

### Comparison Reports
- `FINAL_COMPARISON_REPORT.md` - Final comparison report
- `PRODUCTION_FEATURES_COMPARISON.md` - Production features comparison
- `PRODUCTION_FEATURES_FINAL.md` - Final production features report
- `PRODUCTION_FEATURES_RESULTS.md` - Production features results

### Benchmark Runs
- `btdcoded_YYYYMMDD_HHMMSS/` - Individual benchmark run directories
  - Each run contains JSON results for that benchmark session

### Comparison Data
- `bitcoin_core.json` - Bitcoin Core benchmark data
- `bitcoin_core_test.json` - Bitcoin Core test data
- `full_comparison.json` - Full comparison data
- `production_features_comparison.json` - Production features comparison data

### Documentation
- `BUILD_BITCOIN_CORE.md` - Instructions for building Bitcoin Core for comparison

## Usage

See [scripts/README_BENCHMARKING.md](../scripts/README_BENCHMARKING.md) for how to run benchmarks and generate these results.

## Organization

Results are organized by:
- **Type**: Summary, comparisons, individual runs
- **Date**: Run directories include timestamp
- **Component**: Some results are component-specific

## Maintenance

- Old benchmark runs can be archived
- Keep latest summary and comparison reports
- Individual run directories can be cleaned up after analysis

