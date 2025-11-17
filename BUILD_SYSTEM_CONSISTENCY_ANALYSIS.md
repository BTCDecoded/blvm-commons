# Build System Consistency Analysis

## Executive Summary

**Status**: ⚠️ **INCONSISTENT** - Multiple mismatches between bllvm-commons orchestration and existing workflow system.

## Current State Comparison

### 1. Event Type Mismatch

**bllvm-commons expects:**
- Event type: `build-request`
- Workflow file: `.github/workflows/build.yml`

**Existing workflows use:**
- Event type: `build-chain` or `upstream-changed` (in repos)
- Event type: `deploy` (for governance-app from release_orchestrator)
- Workflow files: `ci.yml` (handles builds)

**Impact**: bllvm-commons will not trigger existing workflows.

### 2. Repository Name Mismatch

**release_orchestrator.yml uses (OLD):**
- `consensus-proof`
- `protocol-engine`
- `reference-node`
- `developer-sdk`
- `governance-app`

**bllvm-commons uses (NEW):**
- `bllvm-consensus`
- `bllvm-protocol`
- `bllvm-node`
- `bllvm-sdk`
- `bllvm-commons`

**versions.toml uses (NEW):**
- Matches bllvm-commons names

**Impact**: release_orchestrator.yml is out of date.

### 3. Dependency Graph Mismatch

**bllvm-commons dependency graph:**
```rust
bllvm-consensus → []
bllvm-protocol → [bllvm-consensus]
bllvm-node → [bllvm-protocol, bllvm-consensus]
bllvm-sdk → [bllvm-node]  // ⚠️ WRONG
bllvm → [bllvm-node]
bllvm-commons → [bllvm-sdk]
```

**Actual dependency graph (from BUILD_CHAIN_COMPLETE.md):**
```
bllvm-consensus (no deps)
bllvm-sdk (no deps)  // Independent, parallel with consensus
bllvm-protocol → [bllvm-consensus]
bllvm-node → [bllvm-protocol, bllvm-consensus]
bllvm → [bllvm-node]
bllvm-commons → [bllvm-sdk]
```

**versions.toml confirms:**
- `bllvm-sdk` has no `requires` field (independent)

**Impact**: bllvm-commons will build in wrong order (bllvm-sdk after bllvm-node, when it should be parallel).

### 4. Workflow Architecture Mismatch

**Existing system:**
- Uses reusable workflows from `bllvm/.github/workflows/`
- Calls workflows directly: `uses: ./.github/workflows/build_lib_cached.yml`
- Uses workflow dependencies (`needs:`) for sequencing
- Self-hosted runners

**bllvm-commons system:**
- Uses `repository_dispatch` to trigger workflows in each repo
- Expects each repo to have its own `build.yml` workflow
- Monitors via GitHub API
- Can work with any runner type

**Impact**: Different architectures, need to align.

## What Needs to Be Fixed

### Critical Fixes for bllvm-commons

1. **Fix dependency graph**:
   ```rust
   // WRONG:
   dependencies.insert("bllvm-sdk".to_string(), vec!["bllvm-node".to_string()]);
   
   // CORRECT:
   dependencies.insert("bllvm-sdk".to_string(), vec![]); // Independent
   ```

2. **Support existing event types**:
   - Option A: Change bllvm-commons to use `build-chain` event type
   - Option B: Add `build-request` handler to existing `ci.yml` workflows
   - Option C: Create new `build.yml` workflows that listen for `build-request`

3. **Support existing workflow files**:
   - Don't hardcode `.github/workflows/build.yml`
   - Try multiple workflow files: `build.yml`, `ci.yml`
   - Or make it configurable

4. **Update release_orchestrator.yml**:
   - Update repo names to new naming
   - Update to use `bllvm-commons` instead of `governance-app`

### Missing Workflows

Each repo needs a workflow that:
- Listens for `repository_dispatch` with type `build-request` (or adapt to use existing types)
- Builds the repository
- Posts status back to bllvm-commons (optional, but helpful)

**Current status:**
- ✅ `bllvm-node`: Has `ci.yml` with `repository_dispatch` support (type: `build-chain`)
- ✅ `bllvm-consensus`: Has `ci.yml` with `repository_dispatch` support
- ✅ `bllvm-protocol`: Has `ci.yml` with `repository_dispatch` support
- ✅ `bllvm-sdk`: Has `ci.yml` with `repository_dispatch` support
- ❌ None listen for `build-request` event type

## Recommended Solution

### Option 1: Adapt bllvm-commons to Existing System (Recommended)

**Changes to bllvm-commons:**

1. **Use existing event types**:
   ```rust
   // Change from:
   "build-request"
   
   // To:
   "build-chain"  // Matches existing workflows
   ```

2. **Support multiple workflow files**:
   ```rust
   // Try multiple workflow files
   let workflow_files = [
       ".github/workflows/build.yml",
       ".github/workflows/ci.yml",
   ];
   
   for workflow_file in workflow_files {
       let runs = self.github_client
           .list_workflow_runs(owner, repo, Some(workflow_file), None, Some(1))
           .await?;
       // ...
   }
   ```

3. **Fix dependency graph**:
   ```rust
   dependencies.insert("bllvm-sdk".to_string(), vec![]); // Independent
   ```

### Option 2: Add build-request Support to Existing Workflows

**Changes to each repo:**

Add to existing `ci.yml`:
```yaml
on:
  repository_dispatch:
    types: [build-chain, build-request]  # Add build-request
```

**Pros**: Minimal changes, works with both systems
**Cons**: Each repo needs update

### Option 3: Create New build.yml Workflows

**Create `.github/workflows/build.yml` in each repo:**
```yaml
name: Build

on:
  repository_dispatch:
    types: [build-request]

jobs:
  build:
    # Build logic here
```

**Pros**: Clean separation
**Cons**: Duplicate build logic

## Completeness Analysis

### What bllvm-commons Has ✅

1. ✅ Dependency graph (needs fix)
2. ✅ Build triggering via repository_dispatch
3. ✅ Build monitoring (polling)
4. ✅ Artifact collection (structure ready)
5. ✅ Release creation (structure ready)
6. ✅ Error handling and retries

### What bllvm-commons Needs ⏳

1. ⏳ **Fix dependency graph** (bllvm-sdk should be independent)
2. ⏳ **Support existing event types** (`build-chain` instead of `build-request`)
3. ⏳ **Support existing workflow files** (`ci.yml` in addition to `build.yml`)
4. ⏳ **Complete artifact collection** (currently placeholder)
5. ⏳ **Complete release creation** (currently placeholder)
6. ⏳ **Database persistence** (store build state)
7. ⏳ **Retry logic** (structure exists, needs implementation)
8. ⏳ **Parallel build support** (dependency graph has `get_parallel_groups()`, but orchestrator doesn't use it)

### What's Missing for Production Readiness

1. **Workflow Integration**:
   - Each repo needs to listen for bllvm-commons events
   - Or bllvm-commons needs to use existing event types

2. **State Management**:
   - Store build state in database
   - Track which builds are in progress
   - Handle restarts/recovery

3. **Artifact Handling**:
   - Download artifacts from workflow runs
   - Create unified release package
   - Upload to release

4. **Error Recovery**:
   - Retry failed builds
   - Handle partial failures
   - Rollback on critical failures

5. **Testing**:
   - Unit tests for dependency graph
   - Integration tests with mock GitHub API
   - End-to-end tests with test releases

## Action Plan

### Phase 1: Fix Inconsistencies (Immediate)

1. **Fix dependency graph**:
   ```rust
   dependencies.insert("bllvm-sdk".to_string(), vec![]);
   ```

2. **Support existing event types**:
   ```rust
   // Use "build-chain" instead of "build-request"
   .trigger_workflow(owner, repo, "build-chain", &payload)
   ```

3. **Support existing workflow files**:
   ```rust
   // Try ci.yml if build.yml doesn't exist
   ```

### Phase 2: Complete Implementation

1. Complete artifact collection
2. Complete release creation
3. Add database persistence
4. Implement retry logic
5. Add parallel build support

### Phase 3: Integration

1. Test with existing workflows
2. Add fallback to release_orchestrator
3. Run in parallel for validation
4. Switch to primary after validation

## Conclusion

**Current Status**: bllvm-commons is **incomplete and inconsistent** with existing workflows.

**To Complete**:
1. Fix dependency graph (bllvm-sdk is independent)
2. Adapt to existing event types (`build-chain`)
3. Support existing workflow files (`ci.yml`)
4. Complete artifact collection and release creation
5. Add database persistence and retry logic

**Timeline**: 1-2 weeks to fix inconsistencies and complete implementation.

