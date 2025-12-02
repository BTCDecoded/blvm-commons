# Build Orchestration Implementation

## Overview

Build orchestration has been added to the governance app to handle cross-repository build coordination, dependency management, build monitoring, and artifact collection for releases.

## What's Implemented

### 1. Build Module Structure

Created `governance-app/src/build/` with:
- **`mod.rs`** - Module exports
- **`dependency.rs`** - Dependency graph management with topological sorting
- **`monitor.rs`** - Build status monitoring with polling
- **`artifacts.rs`** - Artifact collection (placeholder)
- **`orchestrator.rs`** - Main orchestration logic

### 2. Dependency Graph

- Defines build order for all BTCDecoded repositories
- Supports topological sorting to respect dependencies
- Can identify parallel build opportunities
- Handles circular dependency detection

**Current Build Order:**
1. `bllvm-consensus` (no dependencies)
2. `bllvm-protocol` (depends on: bllvm-consensus)
3. `bllvm-node` (depends on: bllvm-protocol, bllvm-consensus)
4. `bllvm-sdk` (depends on: bllvm-node)
5. `bllvm` (depends on: bllvm-node)
6. `governance-app` (depends on: bllvm-sdk)
7. `commons` (independent)

### 3. Build Orchestrator

- Handles release events from GitHub
- Triggers builds in dependency order
- Monitors build status across all repositories
- Coordinates artifact collection
- Creates unified releases

### 4. Webhook Integration

- Added `release` event handling
- Added `repository_dispatch` event handling
- Updated webhook router to route events correctly
- Extracts event type from `x-github-event` header

### 5. GitHub Client Extensions

Added methods to `GitHubClient`:
- `trigger_workflow()` - Trigger builds via repository_dispatch
- `get_workflow_run_status()` - Get workflow run status
- `list_workflow_runs()` - List workflow runs for a repo

### 6. Error Handling

- Added `BuildError` variant to `GovernanceError`
- Proper error propagation throughout build pipeline

## What Still Needs Implementation

### 1. Workflow Run ID Retrieval (P0)

**Current Status**: Placeholder (returns 0)

**Issue**: `repository_dispatch` doesn't directly return a workflow run ID.

**Solution Options**:
1. Poll for most recent workflow run after triggering
2. Use workflow run API to find run by timestamp/event
3. Have workflows post back their run ID via repository_dispatch

**Recommended**: Option 1 - Poll for recent runs matching the event type.

### 2. Build Status Monitoring (P0)

**Current Status**: Partially implemented

**Needs**:
- Proper status parsing from GitHub API response
- Handle all workflow status types
- Better error handling for API failures
- Retry logic for transient failures

### 3. Artifact Collection (P1)

**Current Status**: Placeholder (returns empty vec)

**Needs**:
- Download artifacts from workflow runs
- Organize artifacts by repository
- Create unified release package (tarball/zip)
- Generate SHA256SUMS file
- Optional: GPG signing

### 4. GitHub Release Creation (P1)

**Current Status**: Placeholder

**Needs**:
- Create release in appropriate repository (commons?)
- Upload artifacts
- Attach SHA256SUMS
- Set prerelease flag correctly
- Generate release notes

### 5. Database Integration (P2)

**Current Status**: Not integrated

**Needs**:
- Store build state in database
- Track build history
- Enable build resumption after failures
- Query build status via API

### 6. Parallel Build Support (P2)

**Current Status**: Sequential only

**Needs**:
- Implement parallel group detection
- Trigger multiple builds simultaneously
- Wait for all dependencies before starting dependent builds

### 7. Retry Logic (P2)

**Current Status**: Basic retry count tracking

**Needs**:
- Automatic retry on transient failures
- Exponential backoff
- Max retry limits per build
- Retry strategy configuration

## Testing

### Manual Testing Steps

1. **Test Dependency Graph**:
   ```bash
   cd governance-app
   cargo test build::dependency
   ```

2. **Test Webhook Handling**:
   - Create a test release in GitHub
   - Send webhook to governance app
   - Verify orchestrator is triggered

3. **Test Build Triggering**:
   - Manually trigger a build via API
   - Verify repository_dispatch event is sent
   - Check workflow is triggered in target repo

### Integration Testing

1. **End-to-End Release Flow**:
   - Create a release tag
   - Verify all builds are triggered
   - Monitor build completion
   - Verify artifacts are collected
   - Verify unified release is created

## Configuration

### GitHub App Permissions Required

Add to GitHub App settings:
- **Actions**: Read/Write (to trigger workflows)
- **Contents**: Read/Write (for artifacts)
- **Releases**: Write (to create releases)

### Webhook Events

Subscribe to:
- `release` - When releases are published
- `repository_dispatch` - Build completion notifications
- `workflow_run` - (Optional) Direct workflow status updates

## Next Steps

1. **Implement Workflow Run ID Retrieval** (P0)
   - Add polling logic after triggering
   - Match by timestamp and event type

2. **Complete Build Monitoring** (P0)
   - Fix status parsing
   - Add proper error handling

3. **Implement Artifact Collection** (P1)
   - Download artifacts from GitHub
   - Create release package

4. **Add Database Integration** (P2)
   - Store build state
   - Enable querying

5. **Add Parallel Build Support** (P2)
   - Implement parallel group detection
   - Trigger multiple builds

## Architecture Benefits

✅ **Centralized Control**: All orchestration in one place
✅ **Simplified Workflows**: Repos only need simple build.yml
✅ **Better Error Handling**: Can retry, pause, resume
✅ **Single Source of Truth**: Build state in governance app
✅ **Leverages Existing Infrastructure**: Uses GitHub App API

## Files Created/Modified

### New Files
- `governance-app/src/build/mod.rs`
- `governance-app/src/build/dependency.rs`
- `governance-app/src/build/monitor.rs`
- `governance-app/src/build/artifacts.rs`
- `governance-app/src/build/orchestrator.rs`
- `governance-app/src/webhooks/release.rs`

### Modified Files
- `governance-app/src/main.rs` - Added build module
- `governance-app/src/webhooks/mod.rs` - Added release module
- `governance-app/src/webhooks/github.rs` - Added release/repository_dispatch handling
- `governance-app/src/github/client.rs` - Added workflow methods
- `governance-app/src/error.rs` - Added BuildError variant

## Usage

Once deployed, the governance app will automatically:
1. Receive release webhooks from GitHub
2. Trigger builds in dependency order
3. Monitor build status
4. Collect artifacts
5. Create unified releases

No manual intervention required!

