# Build Orchestration Testing Strategy

## Overview

Before deprecating existing release workflows, we must thoroughly test the governance app orchestration system. This document outlines a comprehensive testing strategy.

## Testing Phases

### Phase 1: Unit Tests ✅ (Can Start Now)

**Location**: `governance-app/src/build/tests.rs`

**Tests Needed**:
- [x] Dependency graph topological sorting
- [x] Build order calculation
- [x] Circular dependency detection
- [x] Parallel group detection
- [ ] Error handling
- [ ] Retry logic
- [ ] Artifact collection logic

**Run**: `cargo test --package governance-app --lib build::tests`

### Phase 2: Integration Tests (After Implementation Complete)

**Location**: `governance-app/tests/integration/`

**Tests Needed**:
- [ ] Mock GitHub API responses
- [ ] Test workflow triggering
- [ ] Test build status monitoring
- [ ] Test artifact collection
- [ ] Test release creation
- [ ] Test error recovery

**Run**: `cargo test --package governance-app --test integration`

### Phase 3: End-to-End Tests (Dry Run Mode)

**Setup**: Governance app in dry-run mode, test releases only

**Test Scenarios**:
1. **Happy Path**
   - Create test release tag (e.g., `v0.1.0-test.1`)
   - Verify all builds triggered
   - Verify builds complete successfully
   - Verify artifacts collected
   - Verify unified release created

2. **Prerelease Tags**
   - Create prerelease tag (e.g., `v0.1.0-prerelease.1`)
   - Verify prerelease flag set correctly
   - Verify all builds triggered

3. **Build Failures**
   - Simulate build failure in one repo
   - Verify error handling
   - Verify retry logic
   - Verify fallback to workflow orchestrator

4. **Timeout Scenarios**
   - Simulate build timeout
   - Verify timeout handling
   - Verify error reporting

5. **Dependency Failures**
   - Simulate dependency build failure
   - Verify dependent builds don't start
   - Verify error propagation

**Success Criteria**:
- All builds complete successfully
- Artifacts match expected outputs
- SHA256SUMS are correct
- Release packages are valid

### Phase 4: Parallel Testing (Both Systems)

**Duration**: 2-3 months minimum

**Process**:
1. Run governance app orchestration (primary)
2. Run workflow orchestrator (verification)
3. Compare results for each release:
   - Build success/failure
   - Build duration
   - Artifact contents
   - SHA256SUMS
   - Release packages

**Test Releases**:
- At least 5-10 test releases
- Mix of prerelease and production tags
- Include failure scenarios

**Comparison Checklist**:
- [ ] Build order matches
- [ ] All repos built successfully
- [ ] Artifact files match
- [ ] SHA256SUMS match
- [ ] Release packages match
- [ ] Build times comparable
- [ ] Error handling equivalent

### Phase 5: Production Validation

**Duration**: 1-2 months

**Process**:
1. Use governance app for all releases
2. Keep workflow orchestrator as fallback
3. Monitor closely
4. Collect metrics

**Metrics to Collect**:
- Build success rate
- Average build time
- Error rate
- Retry rate
- Artifact collection success rate
- Release creation success rate

**Success Criteria**:
- 100% build success rate (or better than workflow orchestrator)
- No critical issues
- All releases successful
- Community confidence

## Test Data

### Test Releases

Create test releases for testing:
- `v0.1.0-test.1` - Basic test
- `v0.1.0-test.2` - Prerelease test
- `v0.1.0-test.3` - Failure scenario test
- `v0.1.0-test.4` - Timeout scenario test
- `v0.1.0-test.5` - Dependency failure test

### Mock Data

For integration tests, use mock GitHub API responses:
- Mock workflow run responses
- Mock artifact responses
- Mock release creation responses
- Mock error responses

## Test Infrastructure

### Test Environment

- Separate test GitHub organization/repos (optional)
- Test releases clearly marked
- Test artifacts in separate location
- Test database for governance app

### Monitoring

- Log all orchestration steps
- Track build durations
- Track error rates
- Compare with workflow orchestrator

## Rollback Plan

If issues are discovered:

1. **Immediate**: Switch back to workflow orchestrator
2. **Investigate**: Identify root cause
3. **Fix**: Implement fix in governance app
4. **Retest**: Run tests again
5. **Resume**: Continue parallel testing

## Success Criteria for Deprecation

Before deprecating workflows, ALL of these must be true:

- [ ] Unit tests: 100% pass
- [ ] Integration tests: 100% pass
- [ ] End-to-end tests: 5+ successful test releases
- [ ] Parallel testing: 5-10 releases, all successful
- [ ] Production validation: 1-2 months, no critical issues
- [ ] Metrics: Equal or better than workflow orchestrator
- [ ] Community confidence: No major concerns
- [ ] Documentation: Complete and accurate

## Timeline: Proof-Based, Not Time-Based

**Goal**: Prove it works through comprehensive testing, not waiting months.

**Fast Track**:
- Phase 1 (Unit Tests): 1-2 days
- Phase 2 (Integration Tests): 2-3 days  
- Phase 3 (E2E Tests): 3-5 days
- Phase 4 (Comparison Test): 2-3 days
- Phase 5 (Failure Tests): 1-2 days

**Total**: 2-3 weeks to prove it works

**See**: `PROOF_OF_WORK_TESTING.md` for detailed proof plan

## Risk Mitigation

1. **Keep Workflow Orchestrator**: Don't remove until proven
2. **Fallback Mechanism**: Always have fallback
3. **Gradual Migration**: Switch gradually, not all at once
4. **Monitoring**: Monitor closely during transition
5. **Rollback Plan**: Be ready to rollback if needed

---

**Remember**: Better to test thoroughly than to break production releases!

