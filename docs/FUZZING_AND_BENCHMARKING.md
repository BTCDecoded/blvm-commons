# Fuzzing and Benchmarking Guide

## Fuzzing Campaigns

### Enhanced Fuzzing Targets

The codebase includes enhanced fuzzing targets for comprehensive coverage:

- **transaction_validation**: Tests realistic transaction structures with inputs/outputs
- **block_validation**: Tests block validation with various configurations
- **script_execution**: Tests script execution with different flag combinations
- **compact_block_reconstruction**: Tests block operations used by compact blocks

### Running Fuzzing Campaigns

#### Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

#### Run Individual Targets

```bash
cd bllvm-consensus

# Transaction validation (24+ hours recommended)
cargo fuzz run transaction_validation -- -max_total_time=86400 -artifact_prefix=./fuzz-artifacts/

# Block validation
cargo fuzz run block_validation -- -max_total_time=86400 -artifact_prefix=./fuzz-artifacts/

# Script execution
cargo fuzz run script_execution -- -max_total_time=86400 -artifact_prefix=./fuzz-artifacts/

# Compact block operations
cargo fuzz run compact_block_reconstruction -- -max_total_time=86400 -artifact_prefix=./fuzz-artifacts/
```

#### Coverage-Guided Fuzzing with Corpus

```bash
# Create corpus directory
mkdir -p fuzz/corpus/transaction_validation

# Run with corpus (merges new findings)
cargo fuzz run transaction_validation -- -max_total_time=86400 -merge=1

# Use corpus from real Bitcoin data
# (Add real transaction/block files to corpus directory)
```

#### Parallel Fuzzing

Run multiple targets in parallel for faster coverage:

```bash
# Terminal 1
cargo fuzz run transaction_validation -- -max_total_time=86400 &

# Terminal 2
cargo fuzz run block_validation -- -max_total_time=86400 &

# Terminal 3
cargo fuzz run script_execution -- -max_total_time=86400 &
```

#### Analyzing Results

- **Crashes**: Check `fuzz/artifacts/` directory
- **Timeouts**: Check logs for slow inputs
- **Memory issues**: Use `-rss_limit_mb=2048` to limit memory
- **Coverage**: Use `cargo install cargo-fuzz-coverage` for coverage reports

### Expected Results

- **Zero crashes** after 24+ hours per target
- **Coverage**: Aim for >95% coverage of critical paths
- **Issues found**: Document all crashes/timeouts and fix them
- **Performance**: No significant regressions in hot paths

## Performance Benchmarking

### Setup

Benchmarks use the `criterion` framework for statistical analysis and regression detection.

### Running Benchmarks

#### Consensus-Proof Benchmarks

```bash
cd bllvm-consensus

# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench transaction_validation
cargo bench --bench hash_operations
cargo bench --bench block_validation
```

#### Reference-Node Benchmarks (Compact Blocks)

```bash
cd bllvm-node

# Compact block benchmarks
cargo bench --bench compact_blocks
```

### Benchmark Results

Results are saved to `target/criterion/` with:
- HTML reports with graphs and statistics
- Comparison with previous runs
- Automatic regression detection

### Key Benchmarks

#### Core Consensus Operations

- **Transaction validation**: Throughput (tx/sec), latency percentiles
- **Block validation**: Time per block, UTXO operations
- **Hash operations**: SHA256, double SHA256 performance
- **UTXO set**: Insert/remove/query operations

#### Compact Block Operations

- **Compact block creation**: Time to convert full block to compact
- **Transaction hashing**: Double SHA256 for tx IDs
- **Short ID calculation**: SipHash-2-4 performance
- **Transport-aware functions**: Overhead of transport checks

### Performance Baselines

Document baseline measurements for comparison:

```bash
# Save baseline results
cargo bench -- --save-baseline baseline

# Compare against baseline
cargo bench -- --baseline baseline
```

### CI Integration

Benchmarks can be run in CI to detect regressions:

```bash
# Generate JSON output for CI
cargo bench -- --output-format json > benchmark_results.json
```

## LLVM/Compiler Optimizations

### Current Optimizations

Both `bllvm-consensus` and `bllvm-node` are configured with:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = "fat"         # Full link-time optimization
codegen-units = 1   # Single codegen unit (best for LTO)
strip = true        # Remove symbols
panic = "abort"     # No unwinding overhead
```

### Platform-Specific Optimizations

#### x86_64 (Intel/AMD)

```bash
# Enable AVX2/AVX-512 for crypto operations
export RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2,+avx512f"
cargo build --release
```

#### ARM (Apple Silicon, Raspberry Pi)

```bash
# Enable NEON optimizations
export RUSTFLAGS="-C target-cpu=native -C target-feature=+neon"
cargo build --release
```

### Profile-Guided Optimization (PGO)

PGO can provide 10-30% performance improvements:

```bash
# Step 1: Build with instrumentation
export RUSTFLAGS="-C profile-generate=/tmp/pgo-data"
cargo build --release

# Step 2: Run benchmarks/tests to generate profile
cargo bench

# Step 3: Build with profile data
export RUSTFLAGS="-C profile-use=/tmp/pgo-data"
cargo build --release
```

**Note**: PGO requires nightly Rust or LLVM with PGO support.

### LTO Comparison

- **Fat LTO** (`lto = "fat"`): Maximum optimization, slower builds
- **Thin LTO** (`lto = "thin"`): Good optimization, faster builds
- **No LTO**: Fastest builds, less optimization

Current setting: Fat LTO for production builds (best runtime performance).

### Build Time vs Runtime Trade-offs

| Setting | Build Time | Runtime Performance |
|---------|------------|---------------------|
| `lto = "fat"`, `codegen-units = 1` | Slow | Best |
| `lto = "thin"`, `codegen-units = 16` | Fast | Good |
| No LTO, `codegen-units = 256` | Fastest | Baseline |

Recommendation: Use fat LTO for release builds, thin LTO for benchmarks.

## Next Steps

1. **Run initial fuzzing campaigns** (24+ hours each target)
2. **Establish performance baselines** with benchmarks
3. **Profile hot paths** using criterion results
4. **Apply targeted optimizations** based on profiling
5. **Verify improvements** with before/after benchmarks
6. **Document findings** and optimization strategies

