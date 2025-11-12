# Bitcoin Commons System Design Document

## Overview

Bitcoin Commons is a comprehensive Bitcoin implementation ecosystem consisting of five tiers that build upon the Orange Paper's mathematical specifications. The system provides direct mathematical implementation of consensus rules, protocol abstraction, a minimal reference implementation, and a developer-friendly SDK. This directory is managed by the BTCDecoded GitHub organization.

## 5-Tier Component Architecture

```
1. Orange Paper (mathematical foundation)
    ↓ (direct mathematical implementation)
2. bllvm-consensus (pure math: CheckTransaction, ConnectBlock, etc.)
    ↓ (protocol abstraction)
3. bllvm-protocol (Bitcoin abstraction: mainnet, testnet, regtest)
    ↓ (full node implementation)
4. bllvm-node (validation, storage, mining, RPC)
    ↓ (ergonomic API)
5. bllvm-sdk (developer toolkit)
    ↓ (cryptographic governance)
6. governance + governance-app (enforcement engine)
```

### Architecture Rationale

This 6-tier architecture supports Bitcoin's evolution for the next 500 years:

1. **Tier 1 (Orange Paper)**: Mathematical foundation - timeless consensus rules
2. **Tier 2 (bllvm-consensus)**: Pure mathematical implementation - no interpretation
3. **Tier 3 (bllvm-protocol)**: Protocol abstraction - supports variants and evolution
4. **Tier 4 (bllvm-node)**: Full implementation - Bitcoin protocol including mining
5. **Tier 5 (bllvm-sdk)**: Developer experience - ergonomic interfaces
6. **Tier 6 (governance + governance-app)**: Cryptographic governance - multi-signature enforcement

## 1. Consensus-Proof Component

### Purpose
Direct mathematical implementation of Bitcoin consensus rules from the Orange Paper. This is NOT a parser or extractor - it directly implements the mathematical functions specified in the Orange Paper.

### Core Requirements

#### Direct Implementation
- Implement Orange Paper functions as pure mathematical functions
- No parsing, extraction, or analysis of the Orange Paper
- Direct translation of mathematical specifications to code
- Side-effect-free, deterministic functions

#### Mathematical Functions
- CheckTransaction: 𝒯𝒳 → {valid, invalid}
- CheckTxInputs: 𝒯𝒳 × 𝒰𝒮 × ℕ → {valid, invalid} × ℤ
- EvalScript: 𝒮𝒞 × 𝒮𝒯 × ℕ → {true, false}
- VerifyScript: 𝒮𝒞 × 𝒮𝒞 × 𝒲 × ℕ → {true, false}
- ConnectBlock: ℬ × 𝒰𝒮 × ℕ → {valid, invalid} × 𝒰𝒮
- ApplyTransaction: 𝒯𝒳 × 𝒰𝒮 → 𝒰𝒮
- GetBlockSubsidy: ℕ → ℤ
- TotalSupply: ℕ → ℤ
- GetNextWorkRequired: ℋ × ℋ* → ℕ
- CheckProofOfWork: ℋ → {true, false}
- AcceptToMemoryPool: 𝒯𝒳 × 𝒰𝒮 → {accepted, rejected}
- IsStandardTx: 𝒯𝒳 → {true, false}
- ReplacementChecks: 𝒯𝒳 × 𝒯𝒳 → {true, false}
- CreateNewBlock: 𝒰𝒮 × 𝒯𝒳* → ℬ
- MineBlock: ℬ × ℕ → ℬ × {success, failure}

#### Validation Framework
- Comprehensive test coverage for all mathematical functions
- Edge case testing for boundary conditions
- Integration testing between different consensus systems
- Performance testing for large-scale operations

#### Output Artifacts
- Pure Rust library implementing Orange Paper functions
- Complete test suite with maximum coverage
- Integration tests between consensus systems
- Performance benchmarks

### Technical Specifications

#### File Structure
```
bllvm-consensus/
├── src/
│   ├── types.rs         # Core Bitcoin data structures
│   ├── constants.rs     # Bitcoin consensus constants
│   ├── transaction.rs   # Transaction validation functions
│   ├── script.rs        # Script execution engine
│   ├── block.rs         # Block validation functions
│   ├── economic.rs      # Economic model functions
│   ├── pow.rs          # Proof of work functions
│   ├── mempool.rs      # Mempool validation functions
│   ├── mining.rs       # Mining and block creation
│   ├── error.rs        # Error handling
│   └── lib.rs          # Public API
├── tests/
│   ├── integration_tests.rs      # Basic integration tests
│   └── integration_opportunities.rs  # Cross-system integration tests
└── Cargo.toml          # Exact version pinning for consensus-critical crates
```

#### Key Interfaces
- `ConsensusProof`: Main struct providing all consensus functions
- `validate_transaction`: Transaction validation
- `validate_tx_inputs`: Input validation against UTXO set
- `validate_block`: Complete block validation
- `verify_script`: Script execution and verification
- `check_proof_of_work`: Proof of work validation
- `get_block_subsidy`: Economic model functions
- `accept_to_memory_pool`: Mempool validation
- `create_new_block`: Block creation from mempool
- `mine_block`: Mining protocol

#### Dependencies (Exact Versions)
- `bitcoin = "=0.31.2"`
- `bitcoin_hashes = "=0.11.0"`
- `secp256k1 = "=0.28.2"`
- `serde = "=1.0.226"`
- All consensus-critical crates pinned to exact versions

#### Success Criteria
- 100% of Orange Paper mathematical functions implemented
- 95%+ test coverage across all functions
- All functions are pure, side-effect-free, and deterministic
- Integration tests between all consensus systems
- Performance suitable for production use

## 2. Protocol-Engine Component

### Purpose
Bitcoin protocol abstraction layer that sits between pure mathematical consensus rules (bllvm-consensus) and full node implementation (bllvm-node). Enables multiple Bitcoin variants and protocol evolution.

### Core Requirements

#### Protocol Abstraction
- Support multiple Bitcoin variants (mainnet, testnet, regtest)
- Enable protocol evolution (Bitcoin V1, V2, etc.)
- Abstract economic models for future variants
- Provide network-specific parameters

#### Protocol Variants
- **BitcoinV1**: Production Bitcoin mainnet
- **Testnet3**: Bitcoin test network  
- **Regtest**: Regression testing network

#### Network Parameters
- Magic bytes for P2P protocol identification
- Default network ports
- Genesis blocks for each network
- Difficulty targets and halving intervals

#### Validation Rules
- Protocol-specific size limits
- Feature flags (SegWit, Taproot, RBF)
- Minimum/maximum fee rates
- Protocol evolution tracking

### Technical Specifications

#### File Structure
```
bllvm-protocol/
├── src/
│   ├── lib.rs              # Core protocol engine
│   ├── variants.rs         # Protocol variant definitions
│   ├── validation.rs       # Protocol-specific validation
│   └── network_params.rs   # Network parameters
├── tests/
│   └── integration/        # Protocol integration tests
└── Cargo.toml             # Dependencies (uses bllvm-consensus)
```

#### Key Interfaces
- `BitcoinProtocolEngine`: Main protocol abstraction
- `ProtocolVersion`: Enum for different Bitcoin variants
- `NetworkParameters`: Network-specific configuration
- `ProtocolValidationRules`: Protocol-specific validation rules
- `ProtocolValidationContext`: Validation context with block height

#### Dependencies
- `bllvm-consensus = "=0.1.0"` (exact version)
- Uses bllvm-consensus functions for all validation
- No re-implementation of consensus rules

#### Success Criteria
- Support for mainnet, testnet, and regtest
- Clean abstraction enabling protocol evolution
- Zero consensus rule violations
- Comprehensive test coverage

## 3. Reference-Node Component

### Purpose
Minimal, production-ready Bitcoin implementation that uses bllvm-protocol for protocol abstraction and bllvm-consensus for consensus rules. This is NOT a re-implementation - it directly uses these lower tiers.

### Core Requirements

#### Consensus Engine
- Import bllvm-consensus crate as dependency
- Import bllvm-protocol for protocol abstraction
- Use bllvm-consensus functions directly for all consensus decisions
- No re-implementation of consensus rules
- Zero tolerance for consensus rule modifications

#### Core Functionality
- Block validation using bllvm-consensus functions
- Transaction validation using bllvm-consensus functions
- Protocol variant support via bllvm-protocol
- Chain state management
- Storage layer (sled database)
- P2P networking for block/transaction relay
- RPC interface for external interaction
- Mining coordination (part of Bitcoin protocol)

#### Implementation Constraints
- Written in Rust for memory safety and performance
- Direct import of bllvm-consensus functions
- Comprehensive error handling with specific error codes
- Full logging of consensus decisions

#### Testing Integration
- Use bllvm-consensus test suites as acceptance criteria
- All bllvm-consensus tests must pass
- Additional implementation-specific tests for non-consensus code
- Performance benchmarking against Bitcoin Core

### Technical Specifications

#### File Structure
```
bllvm-node/
├── src/
│   ├── consensus/       # Wrapper around bllvm-consensus functions
│   ├── storage/        # Sled database layer
│   │   ├── blockstore.rs      # Block storage
│   │   ├── utxostore.rs       # UTXO set storage
│   │   ├── chainstate.rs      # Chain metadata
│   │   ├── txindex.rs         # Transaction index
│   │   └── hashing.rs         # Bitcoin hashing
│   ├── network/        # P2P networking
│   │   ├── peer.rs            # Peer connections
│   │   ├── protocol.rs        # Bitcoin wire protocol
│   │   ├── inventory.rs       # Inventory management
│   │   └── relay.rs           # Block/tx relay
│   ├── rpc/            # RPC interface
│   │   ├── server.rs          # JSON-RPC server
│   │   ├── blockchain.rs      # Blockchain methods
│   │   ├── network.rs         # Network methods
│   │   └── mining.rs          # Mining methods
│   ├── node/           # Node orchestration
│   │   ├── sync.rs            # Blockchain sync
│   │   ├── mempool.rs         # Mempool manager
│   │   └── miner.rs           # Mining coordinator
│   └── lib.rs          # Main node interface
├── tests/
│   ├── storage_tests.rs       # Storage layer tests
│   ├── network_tests.rs       # Network tests
│   ├── rpc_tests.rs           # RPC tests
│   └── node_tests.rs          # Node orchestration tests
└── SECURITY.md         # Security boundaries documentation
```

#### Key Interfaces
- `ConsensusEngine`: Wrapper around bllvm-consensus functions
- `BlockValidator`: Uses bllvm-consensus block validation
- `TransactionValidator`: Uses bllvm-consensus transaction validation
- `ChainState`: Blockchain state management
- `NetworkHandler`: P2P communication
- `RpcServer`: External API interface

#### Dependencies
- `bllvm-consensus = "=0.1.0"` (exact version)
- `bllvm-protocol = "=0.1.0"` (exact version)
- `sled = "=0.34.7"` (storage)
- `tokio = "=1.35.1"` (async runtime)
- All dependencies pinned to exact versions for security

#### Success Criteria
- 100% bllvm-consensus test suite passage
- Successful sync with Bitcoin mainnet
- Validation of entire blockchain history
- Performance within 2x of Bitcoin Core
- Zero consensus failures under any conditions

## 4. Developer-SDK Component

### Purpose
Bitcoin governance infrastructure providing cryptographic primitives for institutional governance operations. Implements modular governance architecture with merge mining economics and cryptographic enforcement.

### Core Requirements

#### API Design
- High-level abstractions for common Bitcoin operations
- Type-safe interfaces preventing consensus violations
- Comprehensive documentation with examples
- Multiple language bindings (Rust, Python, JavaScript, Go)

#### Core Functionality
- **Governance Primitives**: Cryptographic key management, signature creation/verification
- **Multisig Threshold Logic**: Collective decision making with configurable thresholds
- **Message Formats**: Releases, module approvals, budget decisions
- **CLI Tools**: Key generation, signing, and verification utilities
- **Composition Framework**: Declarative module composition for alternative Bitcoin implementations
- **Cryptographic Governance**: secp256k1 signatures and multisig for release signing

#### Developer Experience
- Extensive documentation with tutorials
- Code examples for common use cases
- Interactive documentation with runnable examples
- Clear error messages with resolution guidance
- Performance optimization helpers

#### Safety Guarantees
- Impossible to create invalid transactions through API
- Automatic consensus rule validation via bllvm-node
- Clear separation between consensus-critical and convenience functions
- Runtime validation of all operations

### Technical Specifications

#### File Structure
```
bllvm-sdk/
├── src/
│   ├── governance/     # Governance cryptographic primitives
│   │   ├── keys.rs         # Key generation and management
│   │   ├── signatures.rs   # Signature creation/verification
│   │   ├── multisig.rs     # Multisig threshold logic
│   │   ├── messages.rs     # Governance message formats
│   │   └── verification.rs # Signature verification
│   ├── cli/           # CLI tools
│   │   ├── input.rs        # Input parsing utilities
│   │   └── output.rs       # Output formatting
│   └── bin/           # CLI binaries
│       ├── bllvm-keygen.rs
│       ├── bllvm-sign.rs
│       └── bllvm-verify.rs
├── examples/          # Governance workflow examples
├── docs/             # API documentation
├── tests/            # Comprehensive test suite
└── .github/workflows/ # CI/CD automation
```

#### Key Interfaces
- `GovernanceKeypair`: Cryptographic key management
- `Signature`: ECDSA signature creation and verification
- `Multisig`: Threshold signature logic
- `GovernanceMessage`: Release, approval, and budget message formats
- `CLI Tools`: Key generation, signing, and verification utilities

#### Dependencies
- `secp256k1 = "=0.28.2"` (exact version for cryptographic operations)
- `serde = "=1.0.226"` (exact version for serialization)
- `clap = "=4.4.11"` (exact version for CLI tools)
- All dependencies pinned to exact versions for security

#### Success Criteria
- ✅ **77.30% test coverage** on all governance crypto code
- ✅ **Zero compiler warnings** and comprehensive error handling
- ✅ **Production-ready CLI tools** for key generation, signing, verification
- ✅ **Comprehensive test suite** with 143/185 lines covered
- ✅ **CI/CD automation** with testing, linting, and security audits
- ✅ **Complete documentation** with API reference and examples

## Cross-Component Integration

### Data Flow
1. Orange Paper provides mathematical consensus specifications
2. Consensus-proof directly implements mathematical functions
3. Protocol-engine provides Bitcoin variant abstraction
4. Reference-node uses bllvm-protocol and bllvm-consensus
5. Developer-SDK provides governance infrastructure for Bitcoin governance operations

### Validation Chain
- Consensus-proof implements mathematical correctness
- Protocol-engine provides protocol abstraction with zero consensus changes
- Reference-node uses both for full Bitcoin implementation
- Developer-SDK provides governance primitives with cryptographic security
- All components share same test suites where applicable

### Versioning Strategy
- Semantic versioning across all components
- Consensus-proof changes require major version bump
- Protocol-engine must match bllvm-consensus version
- Reference-node must match bllvm-protocol and bllvm-consensus versions
- Developer-SDK can have independent minor/patch versions (future)

### Testing Strategy
- Shared test suites flow from bllvm-consensus to bllvm-node
- Reference-node tests become acceptance criteria for developer-SDK
- Continuous integration across all components
- Cross-component integration tests

## Development Workflow

### Implementation Order
1. Start with bllvm-consensus to implement Orange Paper functions ✅
2. Build bllvm-protocol for Bitcoin variant abstraction ✅
3. Build bllvm-node using bllvm-protocol and bllvm-consensus ✅
4. Create developer-SDK for governance infrastructure ✅
5. Iterate based on real-world usage and feedback

### Quality Gates
- Each component must pass all tests from lower levels
- No component can proceed without predecessor completion
- Exact version pinning required for consensus-critical dependencies
- Performance benchmarking against existing implementations

### Documentation Requirements
- Each component needs comprehensive README
- API documentation auto-generated from code
- Architecture decision records for major design choices
- Governance model documentation in each repo

## Success Metrics

### Technical Metrics
- 99%+ test coverage across all components
- 100% consensus rule implementation accuracy
- Performance within competitive ranges
- Zero consensus failures under any conditions

### Adoption Metrics
- Multiple independent implementations built on SDK
- Active developer community engagement
- Production usage in real applications
- Positive security audit results

### Governance Metrics
- Clear decision-making processes in place
- Active contributor base across components
- Transparent conflict resolution mechanisms
- Sustainable maintenance model established

## 6. Governance Infrastructure Components

### Purpose
Cryptographic governance system that makes Bitcoin governance 6x harder to capture through multi-signature requirements, transparent audit trails, and graduated thresholds across the 5-tier architecture.

### Core Components

#### 6.1 Governance Configuration Repository
- **Purpose**: Central source of truth for governance rules across all BTCDecoded repositories
- **Structure**: YAML-based configuration for repos, maintainers, and cross-layer validation rules
- **Features**: Layer-based signature thresholds, review periods, emergency keyholder system
- **Repository**: [governance](https://github.com/BTCDecoded/governance)

#### 6.2 Governance App (GitHub App)
- **Purpose**: Rust-based GitHub App for cryptographic governance enforcement
- **Features**: Signature verification, review period enforcement, status checks, merge blocking
- **Technology**: secp256k1 (Bitcoin-compatible), PostgreSQL, Axum web framework
- **Repository**: [governance-app](https://github.com/BTCDecoded/governance-app)

### Governance Model

#### Layer-Based Thresholds
- **Constitutional (Layers 1-2)**: 6-of-7 maintainers, 180 days
- **Implementation (Layer 3)**: 4-of-5 maintainers, 90 days
- **Application (Layer 4)**: 3-of-5 maintainers, 60 days
- **Extension (Layer 5)**: 2-of-3 maintainers, 14 days

#### Emergency Mode
- Activated by 5-of-7 emergency keyholders
- Review periods reduced to 30 days
- Signature thresholds unchanged
- Auto-expires after 90 days

#### Cross-Layer Validation
- Ensures dependencies between layers are maintained
- Prevents consensus rule modifications in application layers
- Validates equivalence proofs between Orange Paper and bllvm-consensus

### Technical Implementation

#### Signature Verification
- Uses secp256k1 (Bitcoin-compatible cryptography)
- Multi-signature threshold validation
- Public key management by layer
- Immutable audit trails

#### Status Checks
- Real-time GitHub status updates
- Review period progress tracking
- Signature threshold monitoring
- Merge blocking until requirements met

#### Database Schema
- Pull request tracking
- Maintainer key management
- Emergency mode state
- Governance event audit log
- Cross-layer rule validation

### Security Considerations
- All consensus-critical dependencies pinned to exact versions
- secp256k1 signature verification (Bitcoin-compatible)
- Immutable audit logs for all governance actions
- No consensus rule modifications allowed in application layers
- Emergency keyholder system for crisis situations