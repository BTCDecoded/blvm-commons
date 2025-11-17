# Release Orchestration Consolidation Plan

## Current State: Three Duplicate Systems

### 1. GitHub Actions Workflow (`release_orchestrator.yml`)
**Location**: `bllvm/.github/workflows/release_orchestrator.yml`

**What it does**:
- Reads `versions.toml`
- Calls reusable workflows for each repo sequentially
- Uses workflow dependencies to coordinate
- Runs on self-hosted runners

**Pros**:
- ✅ Already working
- ✅ Uses GitHub Actions native features
- ✅ Cached workflows for speed

**Cons**:
- ❌ Complex workflow dependencies
- ❌ Hard to pause/resume
- ❌ Limited error recovery
- ❌ No centralized state tracking

### 2. Bash Scripts (`build-release-chain.sh`, `build.sh`)
**Location**: `bllvm/scripts/`, `commons/scripts/`

**What it does**:
- Checks out repos locally
- Builds sequentially
- Collects artifacts
- Creates release packages

**Pros**:
- ✅ Good for local development
- ✅ Simple to understand
- ✅ Can run without GitHub

**Cons**:
- ❌ Not automated
- ❌ No monitoring
- ❌ Manual error handling
- ❌ Doesn't integrate with GitHub

### 3. Rust (Governance App) - NEW
**Location**: `governance-app/src/build/`

**What it does**:
- Receives release webhooks
- Triggers builds via `repository_dispatch`
- Monitors build status
- Coordinates artifact collection
- Creates unified releases

**Pros**:
- ✅ Centralized orchestration
- ✅ Better error handling
- ✅ Can pause/resume
- ✅ State tracking in database
- ✅ Webhook-driven automation
- ✅ Single source of truth

**Cons**:
- ❌ Not yet fully implemented
- ❌ Requires governance app deployment

## Recommended Consolidation Strategy

### Phase 1: Keep Scripts for Local Development

**Keep**: Bash scripts for local builds
- `build-release-chain.sh` - Local development builds
- `build.sh` - Simple local builds
- `collect-artifacts.sh` - Artifact collection

**Reason**: Developers need local builds without GitHub

### Phase 2: Deprecate Workflow Orchestrator

**Deprecate**: `release_orchestrator.yml` workflow orchestrator

**Replace with**: Governance app orchestration

**Migration path**:
1. Keep workflow orchestrator as fallback
2. Add governance app as primary orchestrator
3. Mark workflow orchestrator as deprecated
4. Remove after governance app is proven

### Phase 3: Governance App as Primary Orchestrator

**Use**: Governance app for all automated releases

**Benefits**:
- Single source of truth
- Better error handling
- State persistence
- Webhook-driven
- Can integrate with governance rules

## Proposed Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Release Trigger                       │
│  (GitHub Release event or manual via API)               │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              Governance App (Rust)                       │
│  - Receives webhook                                      │
│  - Determines build order                                │
│  - Triggers builds via repository_dispatch               │
│  - Monitors status                                       │
│  - Collects artifacts                                    │
│  - Creates unified release                               │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│         Individual Repo Workflows (Simplified)           │
│  - build.yml: Triggered by repository_dispatch           │
│    - Build artifacts                                     │
│    - Run tests                                           │
│    - Post status back                                    │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│         Local Development (Bash Scripts)                 │
│  - build-release-chain.sh: For local testing            │
│  - build.sh: Simple local builds                         │
│  - collect-artifacts.sh: Local artifact collection       │
└─────────────────────────────────────────────────────────┘
```

## Migration Plan

### ⚠️ CRITICAL: Testing First, Deprecation Later

**DO NOT deprecate workflows until governance app is thoroughly tested and proven.**

### Step 1: Complete Governance App Implementation

**Priority tasks**:
1. ✅ Dependency graph (DONE)
2. ✅ Build triggering (DONE)
3. ⏳ Workflow run ID retrieval (TODO)
4. ⏳ Build monitoring (TODO - partially done)
5. ⏳ Artifact collection (TODO)
6. ⏳ Release creation (TODO)

### Step 2: Comprehensive Testing Strategy

**CRITICAL**: Test governance app orchestration thoroughly before deprecating workflows.

#### 2.1 Unit Tests
- [ ] Test dependency graph topological sorting
- [ ] Test build order calculation
- [ ] Test parallel group detection
- [ ] Test error handling and retries
- [ ] Test artifact collection logic

#### 2.2 Integration Tests
- [ ] Test workflow triggering via repository_dispatch
- [ ] Test build status monitoring
- [ ] Test artifact collection from GitHub
- [ ] Test release creation
- [ ] Test error recovery scenarios

#### 2.3 End-to-End Tests (Dry Run Mode)
- [ ] Test full release flow with test releases
- [ ] Test with prerelease tags
- [ ] Test with production release tags
- [ ] Test failure scenarios (build failures, timeouts)
- [ ] Test retry logic
- [ ] Test artifact collection and packaging

#### 2.4 Parallel Testing (Both Systems)
Run both systems in parallel for multiple releases:
- [ ] Governance app as primary
- [ ] Workflow orchestrator as fallback/verification
- [ ] Compare build results
- [ ] Compare artifact outputs
- [ ] Compare release packages
- [ ] Verify SHA256SUMS match
- [ ] Monitor for 5-10 releases minimum

#### 2.5 Production Validation
- [ ] Test on testnet releases first
- [ ] Test on prerelease tags
- [ ] Monitor for 1-2 months
- [ ] Collect metrics and compare
- [ ] Fix any issues discovered

### Step 3: Add Fallback to Workflow Orchestrator

In governance app, if it fails, fall back to triggering the workflow orchestrator:

```rust
// If governance app orchestration fails, fall back to workflow orchestrator
if let Err(e) = orchestrator.handle_release_event(version, prerelease).await {
    warn!("Governance app orchestration failed: {}, falling back to workflow orchestrator", e);
    github_client.trigger_workflow("BTCDecoded", "bllvm", "release-orchestrator", &json!({
        "version": version,
        "fallback": true
    })).await?;
}
```

### Step 4: Gradual Migration

**Phase 1: Testing (2-3 months)**
- Run both systems in parallel
- Governance app in dry-run mode
- Workflow orchestrator as primary
- Compare results, fix issues

**Phase 2: Soft Switch (1 month)**
- Governance app as primary
- Workflow orchestrator as fallback
- Monitor closely
- Fix any issues

**Phase 3: Validation (1 month)**
- Governance app only
- Workflow orchestrator disabled but kept
- Monitor for issues
- Ready to deprecate if no issues

**Phase 4: Deprecation (After 4-5 months total)**
- Mark workflows as deprecated
- Update documentation
- Keep as backup for 1 more month
- Then remove

### Step 5: Remove Workflow Orchestrator (Only After Full Validation)

**Prerequisites** (Proof-Based):
- ✅ All unit tests pass
- ✅ All integration tests pass
- ✅ E2E test release successful
- ✅ Comparison test shows identical results
- ✅ Failure scenarios handled correctly
- ✅ Code review approved

**Then**:
1. Mark `release_orchestrator.yml` as deprecated
2. Update all documentation
3. Keep as backup for 1 more month
4. Remove after final validation

## What to Keep

### ✅ Keep: Local Development Scripts

**Files to keep**:
- `bllvm/scripts/build-release-chain.sh` - Local builds
- `commons/scripts/build.sh` - Simple builds
- `commons/scripts/collect-artifacts.sh` - Artifact collection
- `commons/scripts/setup-build-env.sh` - Environment setup

**Reason**: Developers need local builds

### ✅ Keep: Individual Repo Workflows

**Files to keep**:
- `bllvm-consensus/.github/workflows/verify.yml` - Verification
- `bllvm-protocol/.github/workflows/build.yml` - Build
- `bllvm-node/.github/workflows/build.yml` - Build
- etc.

**Reason**: These are the actual build workers

### ⚠️ Keep (For Now): Workflow Orchestrator

**File to keep** (until governance app is proven):
- `bllvm/.github/workflows/release_orchestrator.yml`

**Reason**: 
- Still working and reliable
- Needed as fallback during testing
- Will be deprecated only after 4-6 months of successful testing
- See `TESTING_STRATEGY.md` for testing requirements

### ✅ Use: Governance App Orchestration

**Files to use**:
- `governance-app/src/build/orchestrator.rs` - Main orchestration
- `governance-app/src/build/dependency.rs` - Dependency graph
- `governance-app/src/build/monitor.rs` - Build monitoring
- `governance-app/src/webhooks/release.rs` - Release handling

**Reason**: Best architecture for centralized orchestration

## Benefits of Consolidation

1. **Single Source of Truth**: All orchestration logic in one place
2. **Better Error Handling**: Can retry, pause, resume
3. **State Persistence**: Build state in database
4. **Webhook-Driven**: Automatic on release events
5. **Simpler Workflows**: Repos only need simple build.yml
6. **Governance Integration**: Can enforce governance rules during builds

## Action Items

### Phase 1: Implementation (Weeks 1-2)
- [ ] Complete governance app implementation
  - [ ] Workflow run ID retrieval
  - [ ] Complete build monitoring
  - [ ] Complete artifact collection
  - [ ] Complete release creation
- [ ] Add comprehensive unit tests
- [ ] Add integration tests
- [ ] Add fallback to workflow orchestrator

### Phase 2: Testing (Months 1-3)
- [ ] Run end-to-end tests in dry-run mode
- [ ] Test with test releases
- [ ] Test with prerelease tags
- [ ] Run both systems in parallel
- [ ] Compare results and fix issues
- [ ] Monitor for 2-3 months minimum

### Phase 3: Soft Switch (Month 4)
- [ ] Switch governance app to primary
- [ ] Keep workflow orchestrator as fallback
- [ ] Monitor closely
- [ ] Fix any issues

### Phase 4: Validation (Month 5)
- [ ] Governance app only (workflow disabled)
- [ ] Monitor for issues
- [ ] Validate all releases successful
- [ ] Collect metrics

### Phase 5: Deprecation (Month 6+)
- [ ] Mark workflows as deprecated
- [ ] Update documentation
- [ ] Keep as backup for 1 more month
- [ ] Remove after final validation

## Timeline: Proof-Based Testing

**Goal**: Prove it works through comprehensive testing, not time-based validation.

**Fast Track Timeline**:
- **Week 1**: Complete implementation + unit/integration tests
- **Week 2**: E2E test + comparison test (both systems on same release)
- **Week 3**: Failure scenario tests + validation
- **After Proof**: Switch to governance app as primary

**Total**: 2-3 weeks to prove it works

**See**: `governance-app/PROOF_OF_WORK_TESTING.md` for detailed proof plan

## Testing Checklist

### Unit Tests
- [ ] Dependency graph topological sort
- [ ] Build order calculation
- [ ] Parallel group detection
- [ ] Error handling
- [ ] Retry logic

### Integration Tests
- [ ] Workflow triggering
- [ ] Build monitoring
- [ ] Artifact collection
- [ ] Release creation
- [ ] Error recovery

### End-to-End Tests
- [ ] Full release flow (test releases)
- [ ] Prerelease tags
- [ ] Production tags
- [ ] Failure scenarios
- [ ] Retry scenarios

### Parallel Testing
- [ ] Run both systems for 5-10 releases
- [ ] Compare build results
- [ ] Compare artifacts
- [ ] Compare SHA256SUMS
- [ ] Monitor performance

### Production Validation
- [ ] Testnet releases
- [ ] Prerelease tags
- [ ] Monitor for 1-2 months
- [ ] Collect metrics
- [ ] Fix issues

---

## Conclusion

**Proof-Based Testing Approach**: 
1. ✅ Keep all existing systems (workflows, scripts, governance app)
2. ✅ Complete governance app implementation
3. ✅ **Prove it works through comprehensive testing** (2-3 weeks)
   - Unit tests
   - Integration tests with mocks
   - E2E test with test release
   - Comparison test (both systems on same release)
   - Failure scenario tests
4. ✅ Switch to governance app as primary (after proof)
5. ⏳ Deprecate workflows (after proven)

**Final State**:
- Governance app: Primary orchestrator (after proof)
- Workflow orchestrator: Keep as fallback, deprecate after proven
- Bash scripts: Keep for local development

**Timeline**: 2-3 weeks to prove it works, then switch

**See**: `governance-app/PROOF_OF_WORK_TESTING.md` for detailed proof plan

