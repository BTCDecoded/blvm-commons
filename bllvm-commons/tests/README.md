# Build Orchestration Test Suite

## Quick Start

### Run All Tests

```bash
# Unit tests
cargo test --package bllvm-commons --lib build

# Integration tests (when implemented)
cargo test --package bllvm-commons --test build_orchestration_test

# E2E test script
./tests/e2e/release_flow_test.sh
```

## Test Structure

### Unit Tests
- `src/build/tests.rs` - Core logic tests
- Tests dependency graph, build order, error handling

### Integration Tests
- `tests/integration/build_orchestration_test.rs` - Full flow tests
- Tests with mock GitHub API

### End-to-End Tests
- `tests/e2e/release_flow_test.sh` - Real release flow
- Requires test releases and GitHub access

## Proving It Works

To prove the system works:

1. **Run unit tests** - Prove logic is correct
2. **Run integration tests** - Prove integration works
3. **Run E2E test** - Prove full flow works
4. **Compare with workflow orchestrator** - Prove results match
5. **Test failure scenarios** - Prove error handling works

**Success = All tests pass + Results match existing system**

