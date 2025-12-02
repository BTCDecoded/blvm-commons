# bllvm-commons Release Inclusion

## Overview

Yes, **`bllvm-commons` binary will be included with every `bllvm` release** when using the `bllvm-commons` orchestration system.

## How It Works

### 1. Dependency Graph

`bllvm-commons` is part of the build dependency graph:

```
bllvm-consensus (no deps)
  ↓
bllvm-protocol
  ↓
bllvm-node
  ↓
bllvm
  ↓
bllvm-commons (depends on bllvm-sdk)
```

### 2. Artifact Collection

When `bllvm-commons` orchestrates a release:

1. **Builds all repos** in dependency order (including `bllvm-commons` itself)
2. **Collects artifacts** from all workflow runs (including `bllvm-commons`)
3. **Downloads artifacts** from all repositories
4. **Uploads all artifacts** to the unified release in the `bllvm` repository

### 3. Release Location

The `bllvm-commons` orchestrator creates releases in the **`bllvm` repository**, not individual repos. This means:

- All artifacts (including `bllvm-commons` binary) are uploaded to `bllvm` releases
- The release includes artifacts from: `bllvm`, `bllvm-sdk`, `bllvm-commons`, and all other repos
- Artifacts are prefixed with repo name (e.g., `bllvm-commons-bllvm-commons`, `bllvm-bllvm`)

### 4. Binary Names

The `bllvm-commons` repo produces these binaries:
- `bllvm-commons` - Main governance server
- `key-manager` - Key management utility
- `test-content-hash` - Content hash testing tool
- `test-content-hash-standalone` - Standalone content hash test

All of these will be included in the unified release.

## Legacy Scripts

The old `bllvm/scripts/collect-artifacts.sh` script has been updated to reference `bllvm-commons` instead of `governance-app`. This ensures compatibility with both:

1. **Old workflow system**: Scripts collect `bllvm-commons` binaries
2. **New orchestration system**: `bllvm-commons` collects its own artifacts automatically

## Example Release Structure

When `bllvm-commons` orchestrates a release, the `bllvm` repository release will contain:

```
v0.1.0 Release Assets:
├── bllvm-bllvm (binary)
├── bllvm-sdk-bllvm-keygen (binary)
├── bllvm-sdk-bllvm-sign (binary)
├── bllvm-sdk-bllvm-verify (binary)
├── bllvm-commons-bllvm-commons (binary)
├── bllvm-commons-key-manager (binary)
├── bllvm-commons-test-content-hash (binary)
└── bllvm-commons-test-content-hash-standalone (binary)
```

## Verification

To verify `bllvm-commons` is included:

1. Check the dependency graph: `bllvm-commons` is listed in `src/build/dependency.rs`
2. Check artifact collection: The orchestrator collects from all repos in the graph
3. Check release creation: All collected artifacts are uploaded to the release

## Summary

✅ **Yes, `bllvm-commons` binary is included in every release** orchestrated by `bllvm-commons`  
✅ **All `bllvm-commons` binaries** (main binary + utilities) are included  
✅ **Releases are in the `bllvm` repository** (unified release location)  
✅ **Legacy scripts updated** to reference `bllvm-commons` instead of `governance-app`

