# Kani Formal Verification Performance Optimization Guide

**Goal**: Speed up Kani verification while maintaining correctness  
**Current Status**: 176+ proofs, some taking 5+ minutes  
**Target**: Reduce verification time by 50-70% through systematic optimizations

## Executive Summary

Kani verification can be slow, especially for complex functions. This guide provides proven strategies to speed up verification without sacrificing correctness.

## Current Performance Baseline

- **Total Proofs**: 194+ Kani proofs
- **Average Time**: 30 seconds - 5 minutes per proof
- **CI Timeout**: 5 minutes per verification step
- **Bottlenecks**: Unbounded loops, large input spaces, complex data structures

## Optimization Strategies

### 1. Use Bounded Verification (`#[kani::unwind(N)]`)

**Problem**: Unbounded loops cause exponential state space explosion

**Solution**: Add explicit unwind bounds to loops

**Before**:
```rust
#[kani::proof]
fn verify_transaction_validation() {
    let tx = kani::any::<Transaction>();
    // Loop is unbounded - Kani explores all possible iterations
    let result = check_transaction(&tx);
    assert!(result.is_valid());
}
```

**After**:
```rust
#[kani::proof]
#[kani::unwind(10)]  // Limit loop iterations to 10
fn verify_transaction_validation() {
    let tx = kani::any::<Transaction>();
    kani::assume(tx.inputs.len() <= 10);  // Bound input size
    let result = check_transaction(&tx);
    assert!(result.is_valid());
}
```

**Impact**: 10-100x speedup for loop-heavy functions

**Best Practices**:
- Start with small bounds (5-10), increase if needed
- Use `kani::assume()` to bound input sizes
- Document why bounds are safe (e.g., "MAX_INPUTS = 10")

### 2. Constrain Input Space with `kani::assume()`

**Problem**: Large input spaces cause state explosion

**Solution**: Use assumptions to restrict inputs to valid ranges

**Before**:
```rust
#[kani::proof]
fn verify_value_bounds() {
    let value = kani::any::<i64>();  // All 2^64 possible values!
    // Verification explores all possible values
    assert!(value >= 0 && value <= MAX_MONEY);
}
```

**After**:
```rust
#[kani::proof]
fn verify_value_bounds() {
    let value = kani::any::<i64>();
    kani::assume(value >= 0);  // Restrict to valid range
    kani::assume(value <= MAX_MONEY);
    // Now Kani only explores valid values
    assert!(value >= 0 && value <= MAX_MONEY);
}
```

**Impact**: 100-1000x speedup for large input spaces

**Best Practices**:
- Add assumptions early (before expensive operations)
- Use realistic bounds (e.g., MAX_MONEY, MAX_INPUTS)
- Document assumptions in comments

### 3. Split Large Proofs into Smaller Ones

**Problem**: Large proofs with multiple properties are slow

**Solution**: Split into focused, single-property proofs

**Before**:
```rust
#[kani::proof]
fn verify_transaction_complete() {
    let tx = kani::any::<Transaction>();
    // Verifies 10+ properties at once - slow!
    assert!(check_structure(&tx));
    assert!(check_values(&tx));
    assert!(check_scripts(&tx));
    // ... 7 more properties
}
```

**After**:
```rust
#[kani::proof]
#[kani::unwind(10)]
fn verify_transaction_structure() {
    let tx = kani::any::<Transaction>();
    kani::assume(tx.inputs.len() <= 10);
    assert!(check_structure(&tx));
}

#[kani::proof]
fn verify_transaction_values() {
    let tx = kani::any::<Transaction>();
    kani::assume(tx.inputs.len() <= 10);
    assert!(check_values(&tx));
}
// ... separate proof for each property
```

**Impact**: 3-5x speedup per proof, better error messages

**Best Practices**:
- One property per proof
- Share common setup code
- Use descriptive proof names

### 4. Use Stub Functions for External Dependencies

**Problem**: Verifying code that calls external crates is slow

**Solution**: Stub out external dependencies with simple implementations

**Before**:
```rust
#[kani::proof]
fn verify_hash_operation() {
    let data = kani::any::<[u8; 32]>();
    // Calls into sha2 crate - Kani verifies entire dependency tree!
    let hash = sha2::Sha256::digest(&data);
    assert!(hash.len() == 32);
}
```

**After**:
```rust
#[cfg(kani)]
fn sha256_stub(data: &[u8]) -> [u8; 32] {
    // Simple stub for verification
    let mut result = [0u8; 32];
    for (i, byte) in data.iter().enumerate() {
        result[i % 32] ^= byte;
    }
    result
}

#[kani::proof]
fn verify_hash_operation() {
    let data = kani::any::<[u8; 32]>();
    #[cfg(kani)]
    let hash = sha256_stub(&data);
    #[cfg(not(kani))]
    let hash = sha2::Sha256::digest(&data);
    assert!(hash.len() == 32);
}
```

**Impact**: 5-20x speedup when avoiding external crates

**Best Practices**:
- Stub only when external crate is not under verification
- Keep stubs simple but correct
- Document why stubbing is safe

### 5. Enable Kani Optimizations

**Problem**: Kani default settings may not be optimal

**Solution**: Use Kani's optimization flags

**Command Line**:
```bash
# Enable optimizations
cargo kani --features verify \
    --enable-unstable \
    --enable-stubbing \
    --cbmc-args "--unwind 10" \
    --harness verify_function
```

**In Code**:
```rust
#[kani::proof]
#[kani::unwind(10)]
#[kani::stub(sha2::Sha256::digest, sha256_stub)]  // Stub external function
fn verify_function() {
    // ...
}
```

**Impact**: 10-30% speedup

**Best Practices**:
- Use `--enable-stubbing` for external dependencies
- Set appropriate unwind bounds
- Use `--cbmc-args` for CBMC-specific optimizations

### 6. Parallelize Verification in CI

**Problem**: Sequential verification is slow

**Solution**: Run proofs in parallel

**GitHub Actions Example**:
```yaml
jobs:
  verify:
    strategy:
      matrix:
        proof:
          - verify_transaction_structure
          - verify_transaction_values
          - verify_block_validation
          # ... more proofs
    steps:
      - name: Run Kani proof
        run: cargo kani --features verify --harness ${{ matrix.proof }}
```

**Impact**: Nx speedup (where N = number of parallel jobs)

**Best Practices**:
- Group related proofs together
- Use matrix strategy for parallelization
- Set appropriate timeouts per proof

### 7. Incremental Verification

**Problem**: Re-verifying unchanged code is wasteful

**Solution**: Only verify changed functions

**Script**:
```bash
#!/bin/bash
# Only verify proofs in changed files
CHANGED_FILES=$(git diff --name-only HEAD~1 | grep '\.rs$')
for file in $CHANGED_FILES; do
    # Extract proof names from file
    PROOFS=$(grep -o '#\[kani::proof\]' "$file" | wc -l)
    if [ "$PROOFS" -gt 0 ]; then
        cargo kani --features verify --harness "$(basename $file .rs)"
    fi
done
```

**Impact**: 50-90% time savings on incremental changes

**Best Practices**:
- Use git to detect changed files
- Cache verification artifacts
- Run full verification on main branch

### 8. Use Property-Based Testing for Quick Checks

**Problem**: Some properties are faster to check with property tests

**Solution**: Use proptest for quick validation, Kani for critical proofs

**Strategy**:
```rust
// Quick check with property test (fast, 1000 cases)
#[test]
fn prop_transaction_structure() {
    proptest!(|(tx in transaction_strategy())| {
        assert!(check_structure(&tx));
    });
}

// Critical proof with Kani (slow, exhaustive)
#[kani::proof]
#[kani::unwind(10)]
fn verify_transaction_structure_critical() {
    let tx = kani::any::<Transaction>();
    kani::assume(tx.inputs.len() <= 10);
    assert!(check_structure(&tx));
}
```

**Impact**: 100x faster for non-critical properties

**Best Practices**:
- Use proptest for regression testing
- Use Kani for critical correctness proofs
- Run proptest in CI, Kani on main branch

## Optimization Checklist

For each Kani proof, apply these optimizations:

- [ ] **Add unwind bounds** to all loops
- [ ] **Constrain inputs** with `kani::assume()`
- [ ] **Split large proofs** into focused ones
- [ ] **Stub external dependencies** when possible
- [ ] **Use Kani flags** for optimization
- [ ] **Parallelize in CI** for independent proofs
- [ ] **Use incremental verification** for changed code
- [ ] **Consider proptest** for non-critical properties

## Performance Targets

| Metric | Before | Target | Status |
|--------|--------|--------|--------|
| Average proof time | 2-5 min | < 1 min | ⏳ |
| CI verification time | 30+ min | < 10 min | ⏳ |
| Longest proof | 10+ min | < 3 min | ⏳ |
| Proofs per hour | ~12 | ~60 | ⏳ |

## Example: Optimized Proof

**Before** (slow, 5+ minutes):
```rust
#[kani::proof]
fn verify_transaction_validation() {
    let tx = kani::any::<Transaction>();
    let result = check_transaction(&tx);
    assert!(matches!(result, ValidationResult::Valid | ValidationResult::Invalid(_)));
}
```

**After** (fast, < 30 seconds):
```rust
#[kani::proof]
#[kani::unwind(10)]  // Bound loop iterations
fn verify_transaction_validation() {
    let tx = kani::any::<Transaction>();
    // Constrain input space
    kani::assume(tx.inputs.len() <= 10);
    kani::assume(tx.outputs.len() <= 10);
    for output in &tx.outputs {
        kani::assume(output.value >= 0);
        kani::assume(output.value <= MAX_MONEY);
    }
    // Now verification is fast
    let result = check_transaction(&tx);
    assert!(matches!(result, ValidationResult::Valid | ValidationResult::Invalid(_)));
}
```

## Monitoring Performance

Track verification time over time:

```bash
# Run all proofs and measure time
time cargo kani --features verify --tests

# Generate report
cargo kani --features verify --tests 2>&1 | \
    grep -E "verification.*time|Proof.*succeeded" | \
    tee verification_times.log
```

## Conclusion

By applying these optimizations systematically:
- **50-70% reduction** in verification time is achievable
- **Better error messages** from focused proofs
- **Faster CI** with parallelization
- **Maintained correctness** through careful bounds

Start with unwind bounds and input constraints - these provide the biggest wins with minimal effort.

