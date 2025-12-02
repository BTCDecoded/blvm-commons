# Testing Setup Guide

## Quick Start

### 1. Install Testing Tools

```bash
make install-test-tools
```

Or manually:
```bash
cargo install cargo-fuzz cargo-tarpaulin cargo-insta
```

### 2. Run All Tests

```bash
make test-all
```

### 3. Generate Initial Snapshots

```bash
cargo test --test snapshot_tests
cargo insta review  # Review and accept snapshots
```

## Test Suites

### Unit Tests
- Location: `src/**/*.rs` (in `#[cfg(test)]` modules)
- Run: `cargo test --lib`

### Integration Tests
- Location: `tests/integration/`
- Run: `cargo test --test integration_*`

### Property-Based Tests
- Location: `tests/property_tests.rs`
- Run: `make test-property`
- Uses: `proptest`

### Snapshot Tests
- Location: `tests/snapshot_tests.rs`
- Run: `make test-snapshot`
- Update: `make update-snapshots`
- Uses: `insta`

### Parameterized Tests
- Location: `tests/parameterized/`
- Run: `make test-parameterized`

### Fuzzing
- Location: `fuzz/fuzz_targets/`
- Run: `make fuzz` or `make fuzz-all`
- Uses: `cargo-fuzz`

## Coverage

### Generate Coverage Report

```bash
make test-coverage
```

This generates:
- HTML report: `coverage/tarpaulin-report.html`
- Terminal output with summary

### Coverage Goals

- **Current**: ~85%+ for critical modules
- **Target**: 90%+ overall coverage
- **Critical Modules**: 95%+ coverage

## Fuzzing

### Run Fuzzing Locally

```bash
cd fuzz
cargo fuzz run fuzz_content_hash
```

### Fuzz Targets

1. `fuzz_content_hash` - Fuzz hash computation
2. `fuzz_version_parsing` - Fuzz version parsing
3. `fuzz_test_count_extraction` - Fuzz test count extraction

### Automated Fuzzing

Fuzzing runs automatically:
- Weekly via GitHub Actions
- On-demand via `workflow_dispatch`

## Snapshot Testing

### Creating Snapshots

```rust
use insta::assert_snapshot;

#[test]
fn test_example() {
    let result = compute_something();
    assert_snapshot!("snapshot_name", result);
}
```

### Updating Snapshots

When snapshots change (expected changes):
```bash
cargo insta review
```

This opens an interactive review where you can:
- Accept changes
- Reject changes
- See diffs

### Snapshot Files

- Location: `tests/snapshots/`
- Format: `.snap` files
- Version controlled: Yes

## CI/CD Integration

### GitHub Actions

1. **Test Coverage** (`.github/workflows/test-coverage.yml`)
   - Runs on: PR, push to main, weekly
   - Generates coverage report
   - Uploads to Codecov

2. **Fuzzing** (`.github/workflows/fuzz.yml`)
   - Runs on: Weekly, on-demand
   - Runs all fuzz targets (5 min each)
   - Uploads crashes as artifacts

### Local CI Simulation

```bash
# Run all tests (like CI)
make test-all

# Generate coverage (like CI)
make test-coverage

# Run fuzzing (like CI)
make fuzz-all
```

## Troubleshooting

### Snapshots Failing

If snapshots fail unexpectedly:
1. Review changes: `cargo insta review`
2. If changes are expected, accept them
3. If changes are unexpected, investigate

### Fuzzing Crashes

If fuzzing finds crashes:
1. Check `fuzz/artifacts/` for crash inputs
2. Reproduce: `cargo fuzz run <target> <crash_file>`
3. Fix the bug
4. Re-run fuzzing to verify fix

### Coverage Low

If coverage is low:
1. Identify uncovered lines: `cargo tarpaulin --out Html`
2. Add tests for uncovered code
3. Focus on critical paths first

## Best Practices

1. **Run tests before committing**
   ```bash
   make test-all
   ```

2. **Update snapshots when behavior changes intentionally**
   ```bash
   make update-snapshots
   ```

3. **Run fuzzing before releases**
   ```bash
   make fuzz-all
   ```

4. **Check coverage regularly**
   ```bash
   make test-coverage
   ```

5. **Keep snapshots in version control**
   - Commit `.snap` files
   - Review snapshot changes in PRs

## Makefile Commands

```bash
make test              # Run all tests
make test-all          # Run all test suites
make test-property     # Run property-based tests
make test-snapshot     # Run snapshot tests
make update-snapshots  # Update snapshots (interactive)
make test-coverage     # Generate coverage report
make fuzz              # Run fuzzing
make fuzz-all          # Run all fuzz targets
make install-test-tools # Install testing tools
make help              # Show all commands
```

## Next Steps

1. ✅ Install testing tools
2. ✅ Generate initial snapshots
3. ✅ Run fuzzing locally
4. ✅ Generate coverage report
5. ⏭️ Add more property tests
6. ⏭️ Add more snapshot tests
7. ⏭️ Add more fuzz targets
8. ⏭️ Set coverage targets

