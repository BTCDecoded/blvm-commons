# BTCDecoded Benchmark Results

**Date**: 2025-11-04  
**Toolchain**: Rust stable 1.91.0  
**Features**: None (production features disabled due to dependency constraints)

## Transaction Validation Benchmarks

### Simple Transaction
- **Mean**: 60.6 ns
- **Range**: 58.281 ns - 63.429 ns
- **Outliers**: 12/100 (12.00%)
- **Throughput**: ~16.5 million transactions/sec

### Complex Transaction (10 inputs, 10 outputs)
- **Mean**: 222.75 ns  
- **Range**: 217.03 ns - 229.17 ns
- **Outliers**: 7/100 (7.00%)
- **Throughput**: ~4.5 million transactions/sec

## Hash Operations Benchmarks

### SHA256 (1KB input)
- **Mean**: 12.19 µs
- **Range**: 11.986 µs - 12.425 µs
- **Throughput**: ~82,000 ops/sec
- **Performance**: Improved by ~18.7% vs previous run

### Double SHA256 (1KB input)
- **Mean**: 15.29 µs
- **Range**: 13.904 µs - 16.987 µs
- **Throughput**: ~65,400 ops/sec
- **Performance**: No significant change

## Notes

- **Benchmarks run successfully** with stable Rust toolchain
- **Production features disabled**: Rayon and other optimizations require newer dependencies
- **Results may improve** with production features enabled (parallel processing, caching)
- **Bitcoin Core benchmarks**: Not available (Bitcoin Core needs to be built first)

## Next Steps

To compare with Bitcoin Core:
1. Build Bitcoin Core: `cd /home/user/src/bitcoin && ./autogen.sh && ./configure && make`
2. Run full comparison: `./scripts/benchmark_comparison.sh`

To enable production optimizations:
1. Update dependencies to support Rust 1.83+
2. Run with: `cargo bench --features production`
