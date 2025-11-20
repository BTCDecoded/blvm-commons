<!-- 3103a8d3-e1ce-4a67-bd1c-81081a2889b3 46340826-ef27-4df0-ab5b-3023a8a4656e -->
# Unified Build and Release System for BTCDecoded

## Problem Statement

BTCDecoded consists of multiple **independent git repositories** (NOT a monorepo) organized in a directory structure, with complex dependencies:

- **consensus-proof** (foundation library, no deps)
- **protocol-engine** (library, depends on consensus-proof)
- **reference-node** (binary: `reference-node`, depends on protocol-engine + consensus-proof)
- **developer-sdk** (standalone, produces 3 CLI binaries: `bllvm-keygen`, `bllvm-sign`, `bllvm-verify`)
- **governance-app** (produces multiple binaries: `governance-app`, `key-manager`, `test-content-hash`, `test-content-hash-standalone`, depends on developer-sdk)

**Current State:**

- Each repo uses local path dependencies (`path = "../consensus-proof"`) for local development
- Repos are separate git repositories in directory structure
- No unified build/release orchestration
- Existing script `check_and_push_all.sh` manages multiple repos but doesn't build

## Recommended Architecture: `commons` Repository

The `commons` repository will house all build orchestration, release automation, and version coordination for the BTCDecoded ecosystem.

### Repository Structure

```
commons/                          # BTCDecoded/commons repository
├── README.md                     # Build system documentation
├── build.sh                      # Unified build script
├── versions.toml                 # Version coordination manifest
├── docker-compose.build.yml      # Docker build orchestration
├── .github/
│   └── workflows/
│       ├── build-all.yml         # Reusable: Build all repos in order
│       ├── release.yml           # Reusable: Create unified release
│       ├── verify-versions.yml   # Reusable: Validate version compatibility
│       └── build-single.yml      # Reusable: Build single repo with deps
├── scripts/
│   ├── collect-artifacts.sh     # Package binaries into release archives
│   ├── verify-versions.sh        # Validate versions.toml compatibility
│   ├── create-release.sh         # Release creation automation
│   └── setup-build-env.sh       # Setup build environment with all repos
└── docs/
    └── BUILD_SYSTEM.md            # Detailed build system documentation
```

### Cross-Repo Workflow Usage

**Yes, other repos CAN use workflows and scripts from `commons`!** GitHub Actions supports:

1. **Reusable Workflows** - Other repos call workflows from `commons`
2. **Checkout Action** - Get scripts/files from `commons` repo
3. **Composite Actions** - Reusable action components

#### Example: Other Repos Using Commons Workflows

**In `reference-node/.github/workflows/build.yml`:**

```yaml
name: Build

on:
  push:
    branches: [main]
  workflow_dispatch:

jobs:
  build:
    uses: BTCDecoded/commons/.github/workflows/build-single.yml@main
    with:
      repo-name: reference-node
      required-deps: consensus-proof,protocol-engine
    secrets: inherit
```

**In `developer-sdk/.github/workflows/release.yml`:**

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    uses: BTCDecoded/commons/.github/workflows/release.yml@main
    with:
      repo-name: developer-sdk
      binaries: bllvm-keygen,bllvm-sign,bllvm-verify
    secrets: inherit
```

**In `consensus-proof/.github/workflows/ci.yml`:**

```yaml
name: CI

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      # Checkout commons repo to get build scripts
      - name: Checkout commons
        uses: actions/checkout@v4
        with:
          repository: BTCDecoded/commons
          path: commons
          token: ${{ secrets.GITHUB_TOKEN }}
      
      # Use script from commons
      - name: Run build script
        run: |
          chmod +x commons/scripts/verify-versions.sh
          ./commons/scripts/verify-versions.sh consensus-proof
      
      # Or call reusable workflow
      - name: Verify versions
        uses: BTCDecoded/commons/.github/workflows/verify-versions.yml@main
        with:
          repo-name: consensus-proof
```

### Build Orchestration

#### A. Unified Build Script (`commons/build.sh`)

```bash
#!/bin/bash
# Builds all BTCDecoded repos in dependency order
# Can be run locally or in CI

# Handles:
# - Verifies all repos are present and checked out
# - Dependency order resolution (topological sort)
# - Rust toolchain verification (1.70+)
# - Parallel builds where dependencies allow
# - Mode switching: local paths (dev) vs git deps (release)
# - Artifact collection from target/release
# - Checksum generation (SHA256)
# - Release packaging (tar.gz, zip)
```

**Build Order:**

1. consensus-proof (no deps, can build in parallel)
2. developer-sdk (no deps, can build in parallel with consensus-proof)
3. protocol-engine (depends on consensus-proof)
4. reference-node (depends on protocol-engine + consensus-proof)
5. governance-app (depends on developer-sdk)

#### B. Reusable GitHub Actions Workflows

**`commons/.github/workflows/build-all.yml` (Reusable):**

```yaml
name: Build All Repositories

on:
  workflow_call:
    inputs:
      version_manifest:
        required: false
        type: string
      mode:
        description: 'Build mode: dev or release'
        required: false
        default: 'dev'
        type: choice
        options:
          - dev
          - release

jobs:
  build-all:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout commons
        uses: actions/checkout@v4
        with:
          repository: BTCDecoded/commons
      
      - name: Setup build script
        run: |
          chmod +x build.sh
          chmod +x scripts/*.sh
      
      - name: Checkout all repos
        run: ./scripts/setup-build-env.sh
      
      - name: Build all repositories
        run: ./build.sh --mode ${{ inputs.mode }}
      
      - name: Collect artifacts
        run: ./scripts/collect-artifacts.sh
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: all-binaries
          path: artifacts/
```

**`commons/.github/workflows/release.yml` (Reusable):**

```yaml
name: Create Unified Release

on:
  workflow_call:
    inputs:
      version_tag:
        required: true
        type: string

jobs:
  release:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Checkout commons
        uses: actions/checkout@v4
        with:
          repository: BTCDecoded/commons
      
      - name: Checkout all repos at version
        run: ./scripts/setup-build-env.sh --tag ${{ inputs.version_tag }}
      
      - name: Build all
        run: ./build.sh --mode release
      
      - name: Create release
        run: ./scripts/create-release.sh ${{ inputs.version_tag }}
      
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/*
          body_path: artifacts/RELEASE_NOTES.md
```

#### C. Version Coordination File (`commons/versions.toml`)

```toml
[versions]
# Libraries (no binaries)
consensus-proof = { version = "0.1.0", git_tag = "v0.1.0", git_commit = "abc123..." }
protocol-engine = { version = "0.1.0", git_tag = "v0.1.0", git_commit = "def456...", requires = ["consensus-proof=0.1.0"] }

# Binaries
reference-node = { version = "0.1.0", git_tag = "v0.1.0", git_commit = "ghi789...", requires = ["protocol-engine=0.1.0", "consensus-proof=0.1.0"] }
developer-sdk = { version = "0.1.0", git_tag = "v0.1.0", git_commit = "jkl012...", binaries = ["bllvm-keygen", "bllvm-sign", "bllvm-verify"] }
governance-app = { version = "0.1.0", git_tag = "v0.1.0", git_commit = "mno345...", requires = ["developer-sdk=0.1.0"], binaries = ["governance-app", "key-manager", "test-content-hash", "test-content-hash-standalone"] }
```

### Implementation Strategy

1. **Create `commons` repository structure**
   - Add `build.sh` with dependency graph
   - Add `versions.toml` with current versions
   - Add reusable workflows in `.github/workflows/`
   - Add helper scripts in `scripts/`

2. **Update individual repos to use commons**
   - Add workflow files that call commons reusable workflows
   - Or checkout commons scripts when needed

3. **Release Automation**
   - Tag creation in `commons` triggers unified release
   - Or manual workflow dispatch from any repo
   - Creates GitHub Release with all artifacts

4. **Version Coordination**
   - `versions.toml` tracks compatible versions
   - Updated during releases
   - Validated by governance-app

## Key Features

1. **Dependency Resolution**: Automatically builds in correct order
2. **Parallel Builds**: Where dependencies allow
3. **Artifact Collection**: All binaries in one place
4. **Multi-Format**: Binaries, Docker, archives
5. **Version Coordination**: Enforced compatibility via `versions.toml`
6. **Automated Releases**: GitHub Actions workflow
7. **Security**: Checksums, signatures, reproducible builds
8. **Cross-Repo Reusability**: Other repos can call commons workflows

## Execution Order

1. Create `commons` repository structure
2. Add `build.sh` with dependency graph
3. Add `versions.toml` with current versions
4. Add reusable GitHub Actions workflows
5. Create artifact collection scripts
6. Add Docker build support
7. Update individual repos to use commons workflows
8. Test end-to-end release process

### To-dos

- [ ] Create commons repository structure (README, directory layout)
- [ ] Create `commons/build.sh` script that handles dependency order, parallel builds, and artifact collection
- [ ] Create `commons/versions.toml` to track compatible versions across all repos
- [ ] Create `commons/.github/workflows/build-all.yml` reusable workflow
- [ ] Create `commons/.github/workflows/release.yml` reusable workflow
- [ ] Create `commons/scripts/collect-artifacts.sh` and `commons/scripts/create-release.sh` for packaging
- [ ] Create `commons/scripts/setup-build-env.sh` to checkout all repos
- [ ] Create `commons/scripts/verify-versions.sh` to validate version compatibility
- [ ] Add Docker build support (`commons/docker-compose.build.yml`)
- [ ] Add example workflows to individual repos showing how to use commons

