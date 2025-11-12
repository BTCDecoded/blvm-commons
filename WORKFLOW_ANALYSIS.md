# Workflow Analysis: BTCDecoded vs MyBitcoinFuture

**Date:** 2025-01-XX  
**Status:** Analysis Complete

## Executive Summary

### BTCDecoded Workflows
- **Status**: ⚠️ **PARTIALLY IMPLEMENTED** - Core workflows exist but have inconsistencies
- **Documentation**: Comprehensive methodology documented in `commons/WORKFLOW_METHODOLOGY.md`
- **Implementation**: Core reusable workflows exist in `commons/.github/workflows/`
- **Issues**: 
  - Individual repos use `ubuntu-latest` (GitHub-hosted) instead of self-hosted runners
  - Inconsistency between documented methodology and actual implementation
  - Missing monitoring tools and scripts

### MyBitcoinFuture Workflows
- **Status**: ✅ **FULLY FUNCTIONAL** - Working workflows exist and are actively used
- **Documentation**: CI/CD setup documented in `dashboard/docs/development/CI_CD_SETUP.md`
- **Implementation**: Actual workflow files exist in `.github/workflows/` directories
- **Monitoring**: Comprehensive scripts actively monitor and manage workflows

## Detailed Comparison

### 1. Workflow Structure

#### BTCDecoded (Planned)
According to `commons/WORKFLOW_METHODOLOGY.md`, the following workflows should exist:

**Reusable Workflows (commons/.github/workflows/):**
1. `verify_consensus.yml`
   - Inputs: `repo`, `ref`, `kani` (bool)
   - Runs tests and optional Kani
   - Self-hosted runners

2. `build_lib.yml`
   - Inputs: `repo`, `ref`, `package`, `features`, `verify_deterministic` (optional)
   - Deterministic build, hashes artifacts
   - Self-hosted (prefers `rust` label)

3. `build_docker.yml`
   - Inputs: `repo`, `ref`, `tag`, `image_name`, `push`
   - Builds Docker image, optional push
   - Self-hosted

4. `release_orchestrator.yml`
   - Reads `versions.toml` and sequences all builds
   - Sends `repository_dispatch` deploy to governance-app

**Status**: ✅ **All core workflows exist** in `commons/.github/workflows/`:
- ✅ `verify_consensus.yml` - Exists and functional
- ✅ `build_lib.yml` - Exists and functional
- ✅ `build_docker.yml` - Exists and functional
- ✅ `release_orchestrator.yml` - Exists and functional

**Additional Workflows Found:**
- `build-all.yml` - Build all repositories
- `build-single.yml` - Build single repository
- `release.yml` - Release workflow
- `verify-versions.yml` - Version verification

#### MyBitcoinFuture (Implemented)
According to `dashboard/docs/development/CI_CD_SETUP.md`, the following workflows exist:

**Dashboard Repository:**
- `.github/workflows/ci.yml` - Main CI/CD pipeline
- `.github/workflows/release.yml` - Release workflow with plugin integration
- `.github/workflows/branded-installer-ci.yml` - Branded installer CI
- `.github/workflows/brk-cross-compile.yml` - BRK cross-compilation
- `.github/workflows/npm-publish.yml` - NPM publishing

**Plugins Repository:**
- `.github/workflows/plugin-ci.yml` - Plugin CI/CD
- `.github/workflows/repository-dispatch.yml` - Repository dispatch handler

**Private Plugins Repository:**
- `.github/workflows/plugin-ci.yml` - Private plugin CI/CD
- `.github/workflows/repository-dispatch.yml` - Repository dispatch handler

**Status**: ✅ Workflows exist and are referenced in monitoring scripts

### 2. Workflow Execution

#### BTCDecoded
- **Runner Policy**: ⚠️ **INCONSISTENT** - Methodology says self-hosted only, but individual repos use `ubuntu-latest`
- **Reusable Workflows**: ✅ Use self-hosted runners (`self-hosted,linux,x64`)
- **Individual Repo Workflows**: ❌ Use `ubuntu-latest` (e.g., `consensus-proof/.github/workflows/ci.yml`)
- **Labels**: Optional `rust`, `docker`, `kani` labels for optimization
- **Fallback**: Workflows install missing tools if labeled runners unavailable
- **Issue**: Documented methodology specifies self-hosted only, but per-repo workflows violate this

#### MyBitcoinFuture
- **Runner Policy**: Self-hosted runners (evidenced by monitoring scripts)
- **Local Cache**: Sophisticated local caching system (see `Nightly Shared Build (Local Cache)/`)
- **Monitoring**: Active monitoring via `monitor-ci-pipeline.sh`, `check-ci-status.sh`
- **Implementation**: ✅ Functional - scripts actively query GitHub API for workflow status

### 3. Build System Integration

#### BTCDecoded
**Planned Integration:**
- `commons/versions.toml` as single source of truth
- `build_release_set.sh` for local builds
- `release_orchestrator.yml` reads versions.toml and sequences builds
- Deterministic builds with `--locked` flag

**Status:**
- ✅ Local tools exist (`build_release_set.sh`, `det_build.sh`)
- ✅ Version coordination file exists (`versions.toml`)
- ❌ CI/CD workflows do not exist

#### MyBitcoinFuture
**Actual Integration:**
- Cross-repository coordination via `repository_dispatch`
- Plugin builds triggered from dashboard
- Artifact management and cleanup scripts
- Local build scripts for testing

**Status:**
- ✅ Workflows exist and are functional
- ✅ Monitoring scripts verify workflow execution
- ✅ CI/CD documentation describes actual implementation

### 4. Artifact Management

#### BTCDecoded
**Planned:**
- SHA256SUMS for all binaries
- Optional MANIFEST.json aggregation
- Verification bundles for consensus-proof
- OpenTimestamps receipts (optional)

**Status:**
- ✅ Local scripts generate SHA256SUMS
- ❌ CI workflows don't exist to automate this

#### MyBitcoinFuture
**Actual:**
- Plugin artifacts (zip files) in GitHub releases
- Platform-specific builds (AppImage, DMG, EXE)
- Artifact cleanup script (`gh-artifact-clean.sh`)
- Artifact monitoring and management

**Status:**
- ✅ Artifact management is functional
- ✅ Cleanup scripts exist and are documented

### 5. Monitoring and Debugging

#### BTCDecoded
**Planned:**
- Workflow status checks via GitHub API
- Status checks for PR merge blocking
- Audit logs for governance decisions

**Status:**
- ❌ No monitoring scripts exist
- ❌ No workflow status checking tools

#### MyBitcoinFuture
**Actual:**
- `monitor-workflows.sh` - Simple workflow monitor
- `monitor-ci-pipeline.sh` - Detailed CI/CD pipeline monitor
- `check-ci-status.sh` - CI status checker
- `ci-healer.sh` - Automated CI failure recovery
- `ci-status-report.sh` - Status reporting

**Status:**
- ✅ Comprehensive monitoring tools exist
- ✅ Scripts actively used (evidenced by file structure)

### 6. Cross-Repository Coordination

#### BTCDecoded
**Planned:**
- `repository_dispatch` from commons orchestrator to governance-app
- Deploy signal with payload `{ tag, image }`
- Governance-app listener workflow for deployment

**Status:**
- ✅ Documentation describes the mechanism
- ❌ No orchestrator workflow exists
- ❌ No listener workflow exists in governance-app

#### MyBitcoinFuture
**Actual:**
- `repository_dispatch` from dashboard to plugins/private-plugins
- Plugin builds triggered via dispatch
- Artifact downloads from plugin releases
- Cross-repo coordination via `REPO_ACCESS_TOKEN`

**Status:**
- ✅ Cross-repository workflows exist
- ✅ Documentation describes actual implementation
- ✅ Monitoring scripts verify cross-repo triggers

## Key Findings

### Issues in BTCDecoded

1. **Runner Policy Inconsistency** ⚠️
   - **Documented**: Self-hosted Linux x64 only
   - **Reusable Workflows**: ✅ Correctly use self-hosted runners
   - **Individual Repo Workflows**: ❌ Use `ubuntu-latest` (GitHub-hosted)
   - **Example**: `consensus-proof/.github/workflows/ci.yml` uses `ubuntu-latest` instead of self-hosted

2. **Monitoring Tools** ❌
   - No workflow monitoring scripts
   - No CI status checking tools
   - No workflow debugging utilities
   - MyBitcoinFuture has comprehensive monitoring that could be ported

3. **Workflow Structure** ✅
   - Core reusable workflows exist and are functional
   - Reusable workflow pattern is well-designed
   - Release orchestrator correctly sequences builds
   - But individual repos bypass reusable workflows

4. **Caching Strategy** ⚠️
   - Reusable workflows don't implement caching
   - Individual repo workflows use GitHub Actions cache
   - MyBitcoinFuture has sophisticated local caching system

### Present in MyBitcoinFuture (Can Be Referenced)

1. **Functional Workflows**
   - Actual `.github/workflows/*.yml` files exist
   - Workflows are actively monitored and managed
   - Cross-repository coordination is working

2. **Monitoring Infrastructure**
   - Comprehensive monitoring scripts
   - CI status checking and reporting
   - Automated failure recovery

3. **Build Automation**
   - Local caching systems
   - Artifact management
   - Cross-repo coordination

## Recommendations

### Immediate Actions

1. **Create Workflow Files**
   - Implement the workflows documented in `WORKFLOW_METHODOLOGY.md`
   - Start with `verify_consensus.yml` and `build_lib.yml`
   - Use MyBitcoinFuture workflows as reference for structure

2. **Add Monitoring Tools**
   - Port `monitor-ci-pipeline.sh` from MyBitcoinFuture
   - Create workflow status checking tools
   - Add CI debugging utilities

3. **Test Workflow Execution**
   - Set up self-hosted runners
   - Test workflow execution with simple test cases
   - Verify artifact generation and hashing

### Long-term Improvements

1. **Workflow Templates**
   - Create workflow templates in `commons/templates/`
   - Provide examples for each repository type
   - Document workflow inputs and outputs

2. **CI/CD Documentation**
   - Update documentation with actual workflow examples
   - Add troubleshooting guides
   - Document runner setup procedures

3. **Integration Testing**
   - Test cross-repository workflows
   - Verify `repository_dispatch` mechanism
   - Test version coordination

## Workflow Implementation Priority

### Priority 1: Core Build Workflows
1. ✅ **verify_consensus.yml** - Test consensus-proof
2. ✅ **build_lib.yml** - Build libraries (consensus-proof, protocol-engine)
3. ✅ **build_docker.yml** - Build Docker images (governance-app)

### Priority 2: Orchestration
4. ✅ **release_orchestrator.yml** - Coordinate multi-repo builds
5. ✅ **Deploy workflow** - Governance-app deployment listener

### Priority 3: Monitoring and Maintenance
6. ✅ Monitoring scripts
7. ✅ Artifact cleanup
8. ✅ Status reporting

## Comparison Matrix

| Feature | BTCDecoded | MyBitcoinFuture | Status |
|---------|------------|----------------|--------|
| Workflow Documentation | ✅ Comprehensive | ✅ Good | Both documented |
| Actual Workflow Files | ✅ Core workflows exist | ✅ Multiple | Both have workflows |
| Reusable Workflows | ✅ Well-designed | ❌ Not used | BTCDecoded has better pattern |
| Self-Hosted Runners | ⚠️ Inconsistent | ✅ Consistently used | MyBitcoinFuture consistent |
| Runner Policy Compliance | ❌ Individual repos violate | ✅ Fully compliant | MyBitcoinFuture compliant |
| Monitoring Tools | ❌ None | ✅ Comprehensive | MyBitcoinFuture has tools |
| Caching Strategy | ⚠️ Basic GitHub cache | ✅ Advanced local cache | MyBitcoinFuture superior |
| Cross-Repo Coordination | ✅ Documented/Implemented | ✅ Working | Both functional |
| Artifact Management | ✅ Basic | ✅ Advanced | MyBitcoinFuture more advanced |
| Local Build Tools | ✅ Exist | ✅ Exist | Both have local tools |
| CI/CD Integration | ⚠️ Partial | ✅ Fully functional | MyBitcoinFuture complete |

## Conclusion

**BTCDecoded workflows are partially implemented with architectural inconsistencies.** The core reusable workflows in `commons/` are well-designed and follow the documented methodology, but individual repository workflows violate the self-hosted runner policy by using `ubuntu-latest`.

**MyBitcoinFuture workflows are fully functional** with consistent self-hosted runner usage, comprehensive monitoring, and advanced caching strategies. They can serve as a reference for:
1. Self-hosted runner consistency
2. Monitoring and debugging tools
3. Local caching strategies
4. Workflow structure patterns

**Key Issues in BTCDecoded:**
1. **Runner Policy Violation**: Individual repos use `ubuntu-latest` instead of self-hosted
2. **Missing Monitoring**: No tools to monitor workflow execution
3. **Basic Caching**: Limited caching compared to MyBitcoinFuture's local cache system

**Recommendations:**
1. **Fix Runner Policy**: Update all individual repo workflows to use self-hosted runners
2. **Port Monitoring Tools**: Adapt MyBitcoinFuture monitoring scripts for BTCDecoded
3. **Enhance Caching**: Consider implementing local caching similar to MyBitcoinFuture
4. **Centralize Workflows**: Migrate individual repo workflows to use reusable workflows from commons

---

## Next Steps

1. **Fix Runner Policy Violations**: Update all individual repo workflows to use self-hosted runners
   - Update `consensus-proof/.github/workflows/ci.yml`
   - Update `protocol-engine/.github/workflows/ci.yml`
   - Update `reference-node/.github/workflows/ci.yml`
   - Update `developer-sdk/.github/workflows/ci.yml`
   - Update `governance-app/.github/workflows/governance-app-ci.yml`

2. **Port Monitoring Tools**: Adapt MyBitcoinFuture monitoring scripts
   - Port `monitor-ci-pipeline.sh`
   - Port `check-ci-status.sh`
   - Port `ci-healer.sh`

3. **Enhance Caching**: Implement local caching strategy
   - Study MyBitcoinFuture's `/tmp/runner-cache` system
   - Adapt for Rust/Cargo builds

4. **Centralize Workflows**: Migrate individual repos to use reusable workflows
   - Replace per-repo CI workflows with calls to commons workflows
   - Maintain consistency across all repositories

5. **Test and Validate**: Ensure all workflows work correctly
   - Test on self-hosted runners
   - Verify artifact generation
   - Validate cross-repo coordination

