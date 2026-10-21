# Bitcoin Commons System Overview

**Version:** 1.0  
**Last Updated:** 2025-01-XX  
**Status:** Phase 1 (Infrastructure Complete, Not Yet Activated)

> **Note**: For the most up-to-date verified system status, see [SYSTEM_STATUS.md](./SYSTEM_STATUS.md). This document provides architecture overview; SYSTEM_STATUS.md provides verified implementation details.

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture](#architecture)
3. [Technology Stack](#technology-stack)
4. [Security Guidelines](#security-guidelines)
5. [Development Guidelines](#development-guidelines)
6. [Workflows](#workflows)
7. [Build System](#build-system)
8. [Repository Organization](#repository-organization)
9. [Quick Reference](#quick-reference)

---

## System Overview

### Purpose

Bitcoin Commons is the governance tooling for Bitcoin development. It publishes a constitutional model: a 5-tier signature schedule, review periods, and an audit log.

**Core Innovation:** Apply the same cryptographic enforcement to governance that Bitcoin applies to consensus - making power visible, capture expensive, and exit cheap.

### Current Status

⚠️ **UNRELEASED: This system is not yet activated or tested in production**

- ✅ **Infrastructure Complete**: All core components implemented
- ⚠️ **Not Yet Activated**: Governance rules are not enforced
- 🔧 **Test Keys Only**: No real cryptographic enforcement
- 📋 **Development Phase**: System is in rapid AI-assisted development

**Timeline:**
- **Phase 2 Activation**: 3-6 months (governance enforcement begins)
- **Phase 3 Full Operation**: 12+ months (mature, stable system)
- **Current Phase**: Infrastructure building and testing

### Key Features

- **Constitutional Governance**: 5-tier governance model with graduated thresholds
- **Cryptographic Enforcement**: Multi-signature requirements for all changes
- **Spec-lock & tests**: **blvm-spec-lock** on `#[spec_locked]` Orange Paper contracts, plus comprehensive automated tests (see **blvm-consensus** CI and README)
- **Transparent Audit Trails**: Immutable logs of all governance decisions
- **Governance Fork Capability**: Users can fork governance rulesets

---

## Architecture

### 6-Tier Layered Architecture

Bitcoin Commons implements a **6-tier layered architecture** that builds from mathematical foundations to full implementation:

```
1. Orange Paper (mathematical foundation)
    ↓ (direct mathematical implementation)
2. blvm-consensus (pure math: CheckTransaction, ConnectBlock, etc.)
    ↓ (protocol abstraction)
3. blvm-protocol (Bitcoin abstraction: mainnet, testnet, regtest)
    ↓ (full node implementation)
4. blvm-node (validation, storage, mining, RPC)
    ↓ (ergonomic API)
5. blvm-sdk (developer toolkit)
    ↓ (cryptographic governance)
6. governance + blvm-commons (enforcement engine)
```

### Tier Descriptions

#### Tier 1: Orange Paper
- **Purpose**: Mathematical foundation - timeless consensus rules
- **Repository**: `blvm-spec/` (Orange Paper: `PROTOCOL.md`, `ARCHITECTURE.md`, `THE_ORANGE_PAPER.md`)
- **Type**: Documentation and specification
- **Governance**: Layer 1 (Constitutional - 6-of-7 maintainers, 180 days)

#### Tier 2: Consensus-Proof
- **Purpose**: Pure mathematical implementation of Orange Paper functions
- **Repository**: `blvm-consensus/`
- **Type**: Rust library (pure functions, no side effects)
- **Governance**: Layer 2 (Constitutional - 6-of-7 maintainers, 180 days)
- **Key Functions**: CheckTransaction, ConnectBlock, EvalScript, VerifyScript, etc.
- **Dependencies**: Exact version pinning for all consensus-critical crates

#### Tier 3: Protocol-Engine
- **Purpose**: Protocol abstraction layer enabling multiple Bitcoin variants
- **Repository**: `blvm-protocol/`
- **Type**: Rust library
- **Governance**: Layer 3 (Implementation - 4-of-5 maintainers, 90 days)
- **Supports**: mainnet, testnet, regtest, and future protocol variants
- **Dependencies**: blvm-consensus (exact version)

#### Tier 4: Reference-Node
- **Purpose**: Minimal, production-ready Bitcoin implementation
- **Repository**: `blvm-node/`
- **Type**: Rust binaries (full node)
- **Governance**: Layer 4 (Application - 3-of-5 maintainers, 60 days)
- **Components**: Block validation, storage (sled), P2P networking, RPC, mining
- **Dependencies**: blvm-protocol, blvm-consensus (exact versions)

#### Tier 5: Developer-SDK
- **Purpose**: Developer toolkit for building alternative Bitcoin implementations. Provides module composition framework for declaratively assembling custom Bitcoin nodes, plus governance cryptographic primitives.
- **Repository**: `blvm-sdk/`
- **Type**: Rust library and CLI tools
- **Governance**: Layer 5 (Extension - 2-of-3 maintainers, 14 days)
- **Components**: Key generation, signing, verification, multisig operations
- **CLI Tools**: `blvm-keygen`, `blvm-sign`, `blvm-verify`
- **Dependencies**: Standalone (no consensus dependencies)

#### Tier 6: Governance Infrastructure
- **Purpose**: Cryptographic governance enforcement
- **Repositories**: `governance/` (configuration), `blvm-commons/` (enforcement)
- **Type**: Rust service (GitHub App)
- **Governance**: Layer 5 (Extension - 2-of-3 maintainers, 14 days)
- **Components**: GitHub integration, signature verification, status checks, merge blocking
- **Dependencies**: blvm-sdk

### Dependency Graph

```
blvm-consensus (no dependencies)
    ↓
blvm-protocol
    ↓
blvm-node

blvm-sdk (no dependencies)
    ↓
blvm-commons
```

**Build Order:**
1. blvm-consensus (foundation)
2. blvm-sdk (parallel with blvm-consensus)
3. blvm-protocol (depends on blvm-consensus)
4. blvm-node (depends on blvm-protocol + blvm-consensus)
5. blvm-commons (depends on blvm-sdk)

### Component Interaction

- **Consensus Layer**: blvm-consensus provides pure mathematical functions
- **Protocol Layer**: blvm-protocol wraps blvm-consensus with protocol-specific parameters
- **Node Layer**: blvm-node uses blvm-protocol and blvm-consensus for validation
- **SDK Layer**: blvm-sdk provides module composition framework and governance cryptographic primitives
- **Governance Layer**: blvm-commons uses blvm-sdk for cryptographic operations

### Cross-Layer Validation

- Dependencies between layers are strictly enforced
- Consensus rule modifications are prevented in application layers
- Equivalence proofs required between Orange Paper and blvm-consensus
- Version coordination ensures compatibility across layers

See [DESIGN.md](DESIGN.md) for detailed architecture documentation.

---

## Technology Stack

### Core Technologies

- **Language**: Rust 1.70+ (memory safety, performance)
- **Cryptography**: secp256k1 v0.28.2 (Bitcoin-compatible ECDSA)
- **Build System**: Cargo with exact version pinning
- **Async Runtime**: Tokio 1.35.1 (for networking and I/O)

### Storage Technologies

- **blvm-node**: Sled 0.34.7 (embedded database for blockchain state)
- **blvm-commons**: 
  - SQLite (development, testnet)
  - PostgreSQL (production)

### Networking

- **P2P Protocol**: Custom Bitcoin protocol implementation
- **RPC**: JSON-RPC 2.0 interface
- **Webhooks**: GitHub webhook integration (blvm-commons)

### Development Tools

- **Testing**: Cargo test, property-based testing (proptest)
- **Spec-lock verification**: **blvm-spec-lock** (`cargo spec-lock verify`; optional Z3 for full contract checks in CI)
- **Code Quality**: rustfmt, clippy
- **Coverage**: tarpaulin (test coverage measurement)

### CI/CD Infrastructure

- **Platform**: GitHub Actions
- **Runners**: Self-hosted Linux x64 only
- **Workflows**: Reusable workflows in `commons/` repository
- **Artifacts**: SHA256SUMS for deterministic builds

### Dependency Management

- **Consensus-Critical**: Exact version pinning (e.g., `bitcoin = "=0.31.2"`)
- **Cryptographic**: Exact version pinning (e.g., `secp256k1 = "=0.28.2"`)
- **Other**: Semantic versioning with Cargo.lock

### Version Coordination

- **File**: `commons/versions.toml` (single source of truth)
- **Format**: TOML with version, git_tag, git_commit, requires fields
- **Enforcement**: Build system validates compatibility

---

## Security Guidelines

### Security Model

Bitcoin Commons implements a multi-layered security model:

1. **Exact Dependency Pinning**: All consensus-critical and cryptographic dependencies use exact versions
2. **Spec-lock + tests**: **blvm-spec-lock** verification and the test gates defined in **blvm-consensus** CI are required for consensus-impacting changes
3. **Cryptographic Enforcement**: Multi-signature requirements for all governance actions
4. **Security Boundaries**: Clear separation of concerns per repository
5. **Responsible Disclosure**: Coordinated vulnerability disclosure process

### Security Boundaries by Repository

#### Consensus-Proof
- **Handles**: Bitcoin consensus rule validation
- **Threats**: Consensus bypasses, cryptographic vulnerabilities, memory safety issues
- **Requirements**: 95%+ test coverage, **blvm-spec-lock** / `#[spec_locked]` gates and tests as defined in the consensus repository
- **See**: [blvm-consensus/SECURITY.md](blvm-consensus/SECURITY.md)

#### Developer-SDK
- **Handles**: Governance cryptography (keys, signatures, multisig)
- **Does NOT Handle**: User funds, network enforcement, consensus validation
- **Threats**: Cryptographic vulnerabilities, message tampering, multisig bypass
- **Requirements**: 100% test coverage for crypto operations
- **See**: [blvm-sdk/SECURITY.md](blvm-sdk/SECURITY.md)

#### Governance
- **Handles**: Governance rule configuration
- **Threats**: Maintainer key compromise, signature threshold bypasses, rule tampering
- **Requirements**: Configuration validation, tamper-evident rules
- **See**: [governance/SECURITY.md](governance/SECURITY.md)

#### Commons
- **Handles**: Build orchestration and release automation
- **Threats**: Build script injection, version coordination tampering, artifact corruption
- **Requirements**: Input validation, secure version file distribution
- **See**: [commons/SECURITY.md](commons/SECURITY.md)

### Cryptographic Requirements

- **Key Generation**: Cryptographically secure random number generation
- **Key Storage**: HSMs in production (test keys only in Phase 1)
- **Signature Algorithm**: secp256k1 ECDSA (Bitcoin-compatible)
- **Message Format**: Standardized formats to prevent replay attacks
- **Multisig Thresholds**: Strictly enforced with no bypasses

### Spec-lock and test verification

For consensus changes, the following verification stack applies (exact gates follow **blvm-consensus** CI and `SECURITY.md`):

1. **blvm-spec-lock** (required for annotated surfaces)
   - `#[spec_locked("…")]` links implementation to Orange Paper section IDs
   - `cargo spec-lock verify` runs in CI; optional Z3-backed contract checks when enabled

2. **Property-Based Testing** (required)
   - Randomized testing with `proptest`
   - Discovers edge cases; complements spec-lock with empirical coverage

3. **Mathematical specifications** (required)
   - Formal documentation of consensus rules
   - Invariants documented in code
   - Traceability to Orange Paper

**Enforcement:**
- CI blocks merge if verification fails (no human override)
- Governance App validates verification passed before allowing signatures
- Verification requirements set collectively by maintainers

### Vulnerability Reporting

**Critical Security Issues:**
- **Email**: security@thebitcoincommons.org
- **Subject**: `[SECURITY] <repository> vulnerability`
- **DO NOT**: Create public issues, discuss publicly, post on social media

**Response Timeline:**
- Acknowledgment: Within 24 hours
- Initial Assessment: Within 72 hours
- Fix Development: 1-2 weeks (depending on severity)
- Public Disclosure: Coordinated with fix release

**Vulnerability Types:**
- **P0 (Critical)**: Consensus bypasses, key compromise, build injection
- **P1 (High)**: DoS vulnerabilities, signature bypasses, configuration issues
- **P2 (Medium)**: Performance issues, documentation errors

### Security Considerations

- All consensus-critical dependencies pinned to exact versions
- secp256k1 signature verification (Bitcoin-compatible)
- Immutable audit logs for all governance actions
- No consensus rule modifications allowed in application layers
- Emergency keyholder system for crisis situations

---

## Development Guidelines

### Code Standards

#### Rust Standards
- **Formatting**: `cargo fmt` (automatic formatting)
- **Linting**: `cargo clippy -- -D warnings` (no warnings allowed)
- **Toolchain**: Per-repository `rust-toolchain.toml` files
- **Style**: Follow Rust standard library conventions

#### Code Quality
- **Documentation**: All public APIs must be documented
- **Comments**: Complex algorithms need detailed comments
- **Error Handling**: Comprehensive error handling with specific error codes
- **Logging**: Full logging of consensus decisions

### Testing Requirements

#### Coverage Thresholds
- **blvm-consensus**: 95%+ test coverage (consensus-critical)
- **blvm-sdk**: 77%+ test coverage (governance crypto)
- **Other repositories**: Comprehensive test coverage

#### Test Types
- **Unit Tests**: All new functions
- **Integration Tests**: Cross-module functionality
- **Edge Case Testing**: Boundary conditions
- **Property-Based Testing**: For cryptographic operations and consensus rules
- **Performance Testing**: Benchmarking against Bitcoin Core

#### Test Execution
- Run tests per-file basis
- Wrap Jest commands with timeout utility (5-6 second limit)
- Include `--testTimeout` flag to prevent hanging tests
- Use public data sources by default for continuous integration

### Commit Conventions

Use conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `ci`: CI/CD changes
- `build`: Build system changes
- `config`: Configuration changes

**Examples:**
```
feat(script): add OP_CHECKSIGVERIFY implementation
fix(pow): correct difficulty adjustment calculation
docs(readme): update installation instructions
ci(workflows): add version validation job
```

### Review Process

#### Pull Request Requirements
- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] Commit messages are clear
- [ ] Changes are minimal and focused

#### Review Criteria
1. **Correctness**: Does the code work as intended?
2. **Consensus Compliance**: Does it match Bitcoin Core? (for consensus code)
3. **Test Coverage**: Are all cases covered?
4. **Performance**: No regressions?
5. **Documentation**: Is it clear and complete?
6. **Security**: Any potential vulnerabilities?

#### Approval Requirements
- **Consensus-critical changes**: At least 2 approvals + security team review
- **Cryptographic changes**: Security team review required
- **Performance changes**: Performance review required
- **API changes**: Documentation review required

### Documentation Standards

- **API Documentation**: Auto-generated from code comments
- **Architecture Decisions**: Documented in ADR format
- **Mathematical Formulas**: Reference Orange Paper
- **Examples**: Provide examples for key functions
- **Cross-References**: Link to related documentation

### Development Setup

#### Prerequisites
- Rust 1.70+
- Git
- SQLite3 (for development)
- PostgreSQL (for production blvm-commons)

#### Local Development
```bash
# Clone repositories
git clone https://github.com/BTCDecoded/blvm-consensus.git
git clone https://github.com/BTCDecoded/blvm-protocol.git
# ... (other repositories)

# Build
cd blvm-consensus
cargo build
cargo test

# Run specific test suites
cargo test --test integration_tests
```

See individual repository CONTRIBUTING.md files for detailed setup instructions.

---

## Workflows

### Pull Request Process

The governance system enforces a structured PR process with cryptographic signatures:

1. **Developer opens PR**
2. **Governance App classifies tier** automatically (with temporary manual override)
3. **Maintainers review and sign**: `/governance-sign <signature>`
4. **Review period elapses** (tier-specific duration)
5. **Requirements met** → merge enabled
6. **PR merged**

### Governance Tiers

The system uses a **5-tier constitutional governance model**:

#### Tier 1: Routine Maintenance
- **Signatures**: 3-of-5 maintainers
- **Review Period**: 7 days
- **Scope**: Bug fixes, documentation, performance optimizations
- **Restriction**: Non-consensus changes only

#### Tier 2: Feature Changes
- **Signatures**: 4-of-5 maintainers
- **Review Period**: 30 days
- **Scope**: New RPC methods, P2P changes, wallet features
- **Requirement**: Must include technical specification

#### Tier 3: Consensus-Adjacent
- **Signatures**: 5-of-5 maintainers
- **Review Period**: 90 days
- **Scope**: Changes affecting consensus validation code
- **Requirement**: **blvm-spec-lock** verification and consensus CI tests satisfied per **blvm-consensus** policy

#### Tier 4: Emergency Actions
- **Signatures**: 4-of-5 maintainers
- **Review Period**: 0 days (immediate)
- **Scope**: Critical security patches, network-threatening bugs
- **Requirement**: Post-mortem required after merge
- **Sub-tiers**: Critical Emergency (7 days), Urgent Security (30 days), Elevated Priority (90 days)

#### Tier 5: Governance Changes
- **Signatures**: Special process (5-of-7 maintainers + 2-of-3 emergency keyholders)
- **Review Period**: 180 days
- **Scope**: Changes to governance rules themselves

### Layer + Tier Combination

The governance system combines two dimensions:

1. **Layers** (Repository Architecture) - Which repository the change affects
2. **Tiers** (Action Classification) - What type of change is being made

When both apply, the system uses **"most restrictive wins"** rule:

| Example | Layer | Tier | Final Signatures | Final Review | Source |
|---------|-------|------|------------------|--------------|---------|
| Bug fix in Protocol Engine | 3 | 1 | 4-of-5 | 90 days | Layer 3 |
| New feature in Developer SDK | 5 | 2 | 4-of-5 | 30 days | Tier 2 |
| Consensus change in Orange Paper | 1 | 3 | 6-of-7 | 180 days | Layer 1 |
| Emergency fix in Reference Node | 4 | 4 | 4-of-5 | 0 days | Tier 4 |

See [governance/LAYER_TIER_MODEL.md](governance/LAYER_TIER_MODEL.md) for the complete decision matrix.

### Signature Requirements by Layer

- **Layer 1-2 (Constitutional)**: 6-of-7 maintainers, 180 days (365 for consensus changes)
- **Layer 3 (Implementation)**: 4-of-5 maintainers, 90 days
- **Layer 4 (Application)**: 3-of-5 maintainers, 60 days
- **Layer 5 (Extension)**: 2-of-3 maintainers, 14 days

### Maintainer Signing Process

1. **Review PR**: Understand the change and its impact
2. **Generate Signature**: Use `blvm-sign` from blvm-sdk
3. **Post Signature**: Comment `/governance-sign <signature>` on PR
4. **Governance App Verifies**: Cryptographically verifies signature
5. **Status Check Updates**: Shows signature count progress

### Emergency Procedures

The system includes a three-tiered emergency response system:

1. **Tier 1: Critical Emergency** (Network-threatening)
   - 0 day review period
   - 4-of-7 maintainer signatures
   - 5-of-7 emergency keyholders to activate
   - 7 day maximum duration

2. **Tier 2: Urgent Security Issue**
   - 7 day review period
   - 5-of-7 maintainer signatures
   - 30 day maximum duration

3. **Tier 3: Elevated Priority**
   - 30 day review period
   - 6-of-7 maintainer signatures
   - 90 day maximum duration

See [governance/GOVERNANCE.md](governance/GOVERNANCE.md) for detailed emergency procedures.

### Development Workflows

#### Local Development
1. Clone repository
2. Create feature branch
3. Make changes
4. Run tests (`cargo test`)
5. Format code (`cargo fmt`)
6. Check linting (`cargo clippy`)
7. Commit with conventional format
8. Push and create PR

#### Testing Workflow
1. Run unit tests: `cargo test`
2. Run integration tests: `cargo test --test integration_tests`
3. Run property-based tests: `cargo test --test proptest_tests`
4. Check coverage: `cargo tarpaulin --out Html`
5. Run benchmarks: `cargo bench`

#### Release Workflow
1. Update `versions.toml` in commons/
2. Tag repositories with version tags
3. Run release orchestrator workflow
4. Build all repositories in dependency order
5. Generate artifacts and SHA256SUMS
6. Create verification bundles (blvm-consensus)
7. Publish release with attestations

### CI/CD Workflows

All workflows run on **self-hosted Linux x64 runners only**.

#### Reusable Workflows (commons/)
- `verify_consensus.yml`: Runs tests and **blvm-spec-lock** verification (see **blvm-consensus** workflow; name may vary by org)
- `build_lib.yml`: Deterministic library build with artifact hashing
- `build_docker.yml`: Docker image build and optional push
- `release_orchestrator.yml`: Sequences all builds from versions.toml

#### Workflow Characteristics
- **Deterministic Builds**: `--locked` flag, fixed toolchain
- **Artifact Hashing**: SHA256SUMS for all binaries
- **Version Validation**: Checks compatibility before building
- **Status Checks**: GitHub status checks for PR merge blocking

See [commons/WORKFLOW_METHODOLOGY.md](commons/WORKFLOW_METHODOLOGY.md) for detailed workflow documentation.

---

## Build System

### Build Orchestration

The `commons/` repository serves as the **central orchestrator** for all builds across the ecosystem.

#### Key Principles
- **Single Source of Truth**: `commons/versions.toml` pins tags per repo for a release set
- **Reusable Workflows**: Repos call commons workflows via `workflow_call` for consistency
- **Self-Hosted Only**: All CI runs on `[self-hosted, linux, x64]` runners
- **Deterministic Builds**: `--locked` builds, rust-toolchain per repo, artifact hashing
- **Security Gates**: Consensus verification (tests + **blvm-spec-lock**) precedes builds downstream
- **Clear Ordering**: L2 (blvm-consensus) → L3 (blvm-protocol) → L4 (blvm-node) → blvm-sdk → blvm-commons

### Build Order

The build system follows strict dependency ordering:

```
1. blvm-consensus (no dependencies)
   ↓
2. blvm-protocol (depends on blvm-consensus)
   ↓
3. blvm-node (depends on blvm-protocol + blvm-consensus)

Parallel:
4. blvm-sdk (no dependencies)
   ↓
5. blvm-commons (depends on blvm-sdk)
```

### Version Coordination

#### Single Source of Truth
The `commons/versions.toml` file is the authoritative version map:

```toml
[versions]
blvm-consensus = { version = "0.1.0", git_tag = "v0.1.0", ... }
blvm-protocol = { version = "0.1.0", requires = ["blvm-consensus=0.1.0"], ... }
blvm-node = { version = "0.1.0", requires = ["blvm-protocol=0.1.0", "blvm-consensus=0.1.0"], ... }
```

#### Version Validation
- Build system validates compatibility before building
- Dependency requirements must match versions.toml
- Incompatible versions block builds

### Deterministic Builds

#### Requirements
- **Cargo Lock**: `cargo build --locked --release`
- **Fixed Toolchain**: Per-repo `rust-toolchain.toml` files
- **Artifact Hashing**: SHA256SUMS for all binaries
- **Verification**: Optional deterministic verification (rebuild + compare hashes)

#### Toolchain Management
- Each repository has its own `rust-toolchain.toml`
- CI installs Rust via the org composite [`BTCDecoded/rust-ci/install-rust-toolchain`](https://github.com/BTCDecoded/rust-ci) (default pin **1.88.0**, optional `toolchain`, `toolchain-file`, or `components`; umbrella workflows often use `toolchain-file: rust-toolchain.toml`)
- The composite wraps `dtolnay/rust-toolchain`; individual workflows may pass `with: toolchain: …` when a repo needs a different version (for example **1.89.0** in some crates)

### Local Build Tools

#### build_release_set.sh
Sequences local builds from local clones:
- Builds all repositories in dependency order
- Optional blvm-commons source/docker build
- Optional `MANIFEST.json` aggregation
- Generates SHA256SUMS

#### det_build.sh
Deterministic build wrapper:
- Ensures `--locked` flag
- Verifies toolchain version
- Generates hashes

#### make_verification_bundle.sh
Generates blvm-consensus verification bundle:
- Includes test results
- Optional **blvm-spec-lock** attestation artifacts (when published with a release)
- Optional OpenTimestamps anchoring

### Artifact Management

#### Binary Collection
Built binaries are collected in `artifacts/binaries/`:
- `blvm-node` - Bitcoin reference node
- `blvm-keygen`, `blvm-sign`, `blvm-verify` - SDK tools
- `blvm-commons`, `key-manager`, `test-content-hash*` - Governance tools

#### Release Artifacts
- `SHA256SUMS` - Checksums for all binaries
- `MANIFEST.json` - Release metadata (optional)
- Verification bundles - Consensus-proof verification results
- OpenTimestamps receipts - Immutable proof of release (optional)

### CI Integration

#### Release Orchestrator
The `release_orchestrator.yml` workflow:
1. Reads `versions.toml` to get version tags
2. Sequences all builds in dependency order
3. Validates compatibility
4. Generates artifacts and hashes
5. Sends `repository_dispatch: deploy` to blvm-commons

#### Reusable Workflows
Other repositories call commons workflows:
```yaml
jobs:
  build:
    uses: BTCDecoded/commons/.github/workflows/build_lib.yml@main
    with:
      repo: blvm-node
      ref: v0.1.0
      package: blvm-node
    secrets: inherit
```

### Docker Builds

Use `docker-compose.build.yml` for containerized builds:
```bash
docker-compose -f docker-compose.build.yml build
```

This builds all components in dependency order using Docker.

### Build Script (build.sh)

The unified build script in `commons/build.sh`:
- Checks Rust toolchain (requires 1.70+)
- Verifies all repositories are present
- Builds repos in dependency order
- Collects binaries to `artifacts/binaries/`

**Usage:**
```bash
# Development build (local path dependencies)
./build.sh --mode dev

# Release build (git dependencies)
./build.sh --mode release
```

### Release Sets

A **release set** is a coordinated set of versions across all repositories:

1. **Define Set**: Edit `versions.toml` with exact tags per repo
2. **Orchestrate**: Run `release_orchestrator.yml` workflow
3. **Build**: System builds all repos in dependency order
4. **Artifacts**: Each repo uploads artifacts and `SHA256SUMS`
5. **Manifest**: Collect hashes into set manifest and timestamp with OpenTimestamps

See [commons/RELEASE_SET.md](commons/RELEASE_SET.md) for details.

### Troubleshooting

#### Build Failures
1. Check Rust version: `rustc --version` (requires 1.70+)
2. Verify all repos are cloned
3. Check dependency versions in `versions.toml`
4. Review build logs in `/tmp/<repo>-build.log`

#### Missing Binaries
- Libraries (blvm-consensus, blvm-protocol) don't produce binaries
- Only blvm-node, blvm-sdk, and blvm-commons produce binaries
- Check `target/release/` in each repo after build

#### Version Mismatches
Run `./scripts/verify-versions.sh` to check compatibility.

See [commons/docs/BUILD_SYSTEM.md](commons/docs/BUILD_SYSTEM.md) for detailed build system documentation.

---

## Repository Organization

### Core Repositories

#### 1. commons/
**Purpose**: Build orchestration, workflows, version topology, shared tooling

**Key Components:**
- `versions.toml` - Version coordination manifest
- `build.sh` - Unified build script
- `.github/workflows/` - Reusable workflows
- `tools/` - Build and release tools
- `docs/` - Build system documentation

**Governance**: Layer 5 (Extension - 2-of-3 maintainers, 14 days)

#### 2. blvm-consensus/
**Purpose**: Pure mathematical implementation of Bitcoin consensus rules

**Key Components:**
- Mathematical functions: CheckTransaction, ConnectBlock, EvalScript, etc.
- Property-based tests
- **blvm-spec-lock** verification on `#[spec_locked]` functions (see **blvm-consensus** README / CI)
- Verification bundles

**Governance**: Layer 2 (Constitutional - 6-of-7 maintainers, 180 days)

#### 3. blvm-protocol/
**Purpose**: Bitcoin protocol abstraction layer

**Key Components:**
- Network parameter definitions (mainnet, testnet, regtest)
- Protocol variant support
- Protocol-specific validation rules

**Governance**: Layer 3 (Implementation - 4-of-5 maintainers, 90 days)

#### 4. blvm-node/
**Purpose**: Minimal, production-ready Bitcoin implementation

**Key Components:**
- Block validation (uses blvm-consensus)
- Storage layer (sled)
- P2P networking
- RPC interface
- Mining coordination

**Governance**: Layer 4 (Application - 3-of-5 maintainers, 60 days)

#### 5. blvm-sdk/
**Purpose**: Bitcoin governance infrastructure and cryptographic primitives

**Key Components:**
- Key generation (`blvm-keygen`)
- Signing (`blvm-sign`)
- Verification (`blvm-verify`)
- Multisig operations
- Message formats

**Governance**: Layer 5 (Extension - 2-of-3 maintainers, 14 days)

#### 6. governance/
**Purpose**: Governance configuration and rules

**Key Components:**
- `config/` - YAML configuration files
  - `action-tiers.yml` - 5-tier governance model
  - `repository-layers.yml` - Layer definitions
  - `maintainers/` - Maintainer configurations by layer
  - `repos/` - Repository-specific configurations
- `architecture/` - Architecture documentation

**Governance**: Layer 5 (Extension - 2-of-3 maintainers, 14 days)

#### 7. blvm-commons/
**Purpose**: GitHub App for cryptographic governance enforcement

**Key Components:**
- GitHub webhook handlers
- Signature verification
- Status check posting
- Merge blocking logic
- Governance fork system
- Audit logging

**Governance**: Layer 5 (Extension - 2-of-3 maintainers, 14 days)

### Infrastructure Repositories

#### blvm-spec/ (the-orange-paper/ directory)
**Purpose**: Mathematical foundation and specification

**Governance**: Layer 1 (Constitutional - 6-of-7 maintainers, 180 days)

#### commons-website/ and website/
**Purpose**: Documentation and marketing websites

**Governance**: Layer 5 (Extension - 2-of-3 maintainers, 14 days)

### Repository Structure

Each repository follows a consistent structure:

```
repository-name/
├── src/              # Source code (Rust projects)
├── tests/            # Test suites
├── docs/            # Documentation
├── Cargo.toml       # Rust dependencies
├── rust-toolchain.toml # Toolchain version
├── README.md        # Repository overview
├── CONTRIBUTING.md  # Contribution guidelines
├── SECURITY.md      # Security policy
└── LICENSE          # License file
```

See [DIRECTORY_STRUCTURE.md](DIRECTORY_STRUCTURE.md) for detailed directory structure.

---

## Quick Reference

### Key Commands

#### Development
```bash
# Build a repository
cd blvm-consensus
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Check linting
cargo clippy -- -D warnings

# Check coverage
cargo tarpaulin --out Html
```

#### Build System
```bash
# Build all repositories (development)
cd commons
./build.sh --mode dev

# Build all repositories (release)
./build.sh --mode release

# Verify versions
./scripts/verify-versions.sh

# Create release
./scripts/create-release.sh v0.1.0
```

#### Governance
```bash
# Generate keypair
blvm-keygen

# Sign a PR
blvm-sign <message>

# Verify signature
blvm-verify <signature> <message> <public-key>
```

### Key Files

- `DESIGN.md` - System architecture and design
- `GOVERNANCE.md` - Governance process and rules
- `commons/WORKFLOW_METHODOLOGY.md` - Workflow documentation
- `commons/docs/BUILD_SYSTEM.md` - Build system details
- `commons/versions.toml` - Version coordination
- `governance/config/` - Governance configuration files

### Key URLs

- **GitHub Organization**: https://github.com/BTCDecoded
- **Security Email**: security@thebitcoincommons.org
- **Website**: https://btcdecoded.org

### Governance Quick Reference

| Tier | Signatures | Review Period | Scope |
|------|------------|----------------|-------|
| 1 | 3-of-5 | 7 days | Routine maintenance |
| 2 | 4-of-5 | 30 days | Feature changes |
| 3 | 5-of-5 | 90 days | Consensus-adjacent |
| 4 | 4-of-5 | 0 days | Emergency actions |
| 5 | 5-of-7 + 2-of-3 | 180 days | Governance changes |

| Layer | Signatures | Review Period | Repositories |
|-------|------------|---------------|--------------|
| 1-2 | 6-of-7 | 180 days | Orange Paper, blvm-consensus |
| 3 | 4-of-5 | 90 days | blvm-protocol |
| 4 | 3-of-5 | 60 days | blvm-node |
| 5 | 2-of-3 | 14 days | blvm-sdk, governance, blvm-commons |

### Build Order Quick Reference

1. blvm-consensus (no deps)
2. blvm-sdk (no deps) - parallel
3. blvm-protocol (needs blvm-consensus)
4. blvm-node (needs blvm-protocol + blvm-consensus)
5. blvm-commons (needs blvm-sdk)

---

## Additional Resources

### Documentation Index

- **Architecture**: [DESIGN.md](DESIGN.md)
- **Governance**: [governance/GOVERNANCE.md](governance/GOVERNANCE.md)
- **Workflows**: [commons/WORKFLOW_METHODOLOGY.md](commons/WORKFLOW_METHODOLOGY.md)
- **Build System**: [commons/docs/BUILD_SYSTEM.md](commons/docs/BUILD_SYSTEM.md)
- **Contributing**: Repository-specific CONTRIBUTING.md files
- **Security**: Repository-specific SECURITY.md files

### Getting Help

- **Documentation**: Check repository README.md files
- **Issues**: Search existing issues or create new ones
- **Discussions**: Use GitHub Discussions for questions
- **Security**: See security@thebitcoincommons.org for security issues

---

**Remember**: This system is in Phase 1 (Infrastructure Building). Governance rules are not yet enforced. Use at your own risk and do not deploy in production until Phase 2 activation.

