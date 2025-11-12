# Bitcoin Commons Directory Structure

This document provides a comprehensive overview of the Bitcoin Commons project directory structure. The BTCDecoded directory contains multiple independent git repositories managed by the BTCDecoded GitHub organization.

## Root Directory

```
BTCDecoded/
├── governance/                    # Governance system configuration
├── governance-app/               # Main governance application (Rust)
├── docs/                        # Project documentation
├── audit-materials/             # Security audit materials
├── deployment/                  # Deployment configurations
├── scripts/                     # Utility scripts
├── tests/                       # Test suites
└── README.md                    # Main project README
```

## Governance System (`governance/`)

The governance system is organized into logical subdirectories:

```
governance/
├── config/                      # Core governance configuration
│   ├── action-tiers.yml         # 5-tier governance model definitions
│   ├── repository-layers.yml    # Layer definitions and requirements
│   ├── tier-classification-rules.yml # PR classification rules
│   ├── economic-nodes.yml       # Economic node configuration
│   ├── emergency-tiers.yml      # Emergency action tiers
│   ├── governance-fork.yml      # Governance fork configuration
│   ├── cross-layer-rules.yml    # Cross-layer dependency rules
│   ├── ruleset-export-template.yml # Ruleset export template
│   ├── maintainers/             # Maintainer configurations by layer
│   │   ├── layer-1-2.yml        # Constitutional layers (orange-paper, consensus-proof)
│   │   ├── layer-3.yml          # Implementation layer (protocol-engine)
│   │   ├── layer-4.yml          # Application layer (reference-node)
│   │   └── emergency.yml        # Emergency keyholders
│   ├── repos/                   # Repository-specific configurations
│   │   ├── orange-paper.yml     # Layer 1 configuration
│   │   ├── consensus-proof.yml  # Layer 2 configuration
│   │   ├── protocol-engine.yml  # Layer 3 configuration
│   │   ├── reference-node.yml   # Layer 4 configuration
│   │   └── developer-sdk.yml    # Layer 5 configuration
│   └── README.md                # Configuration documentation
├── architecture/                # Architecture documentation
│   ├── CRYPTOGRAPHIC_GOVERNANCE.md
│   ├── ECONOMIC_NODES.md
│   ├── GOVERNANCE_FORK.md
│   ├── SERVER_AUTHORIZATION.md
│   └── CROSS_LAYER_DEPENDENCIES.md
└── README.md                    # Governance system overview
```

## Governance Application (`governance-app/`)

The main Rust application implementing the governance system:

```
governance-app/
├── src/                         # Source code
│   ├── bin/                     # CLI tools
│   │   ├── sign-pr.rs           # PR signing tool
│   │   ├── economic-node-register.rs
│   │   ├── economic-node-veto.rs
│   │   ├── economic-node-verify.rs
│   │   ├── fork-migrate.rs      # Governance fork migration
│   │   └── verify-audit-log.rs  # Audit log verification
│   ├── config/                  # Configuration management
│   │   ├── loader.rs            # YAML config loader
│   │   └── mod.rs
│   ├── database/                # Database layer
│   │   ├── mod.rs               # Main database interface
│   │   ├── models.rs            # Data models
│   │   ├── queries.rs           # Database queries
│   │   └── migrations/          # Database migrations
│   ├── enforcement/             # Governance enforcement
│   │   ├── merge_blocker.rs     # Merge blocking logic
│   │   ├── decision_log.rs      # Decision logging
│   │   └── mod.rs
│   ├── fork/                    # Governance fork system
│   │   ├── adoption.rs          # Adoption tracking
│   │   ├── dashboard.rs         # Fork dashboard
│   │   ├── detection.rs         # Fork detection
│   │   ├── executor.rs          # Fork execution
│   │   ├── export.rs            # Ruleset export
│   │   ├── types.rs             # Fork types
│   │   ├── versioning.rs        # Ruleset versioning
│   │   └── mod.rs
│   ├── github/                  # GitHub integration
│   │   ├── client.rs            # GitHub API client
│   │   ├── integration.rs       # GitHub integration logic
│   │   └── mod.rs
│   ├── economic_nodes/          # Economic node system
│   │   ├── registry.rs          # Node registry
│   │   ├── veto.rs              # Veto system
│   │   └── mod.rs
│   ├── nostr/                   # Nostr integration
│   │   ├── client.rs            # Nostr client
│   │   ├── publisher.rs         # Event publisher
│   │   ├── events.rs            # Event types
│   │   └── mod.rs
│   ├── ots/                     # OpenTimestamps integration
│   │   ├── client.rs            # OTS client
│   │   ├── anchorer.rs          # Registry anchoring
│   │   ├── verify.rs            # Verification utilities
│   │   └── mod.rs
│   ├── audit_log/               # Audit logging system
│   │   ├── entry.rs             # Log entry format
│   │   ├── logger.rs            # Audit logger
│   │   ├── verify.rs            # Log verification
│   │   ├── merkle.rs            # Merkle tree construction
│   │   └── mod.rs
│   ├── server_auth/             # Server authorization
│   │   ├── registry.rs          # Authorized server registry
│   │   ├── verification.rs      # Server verification
│   │   └── mod.rs
│   ├── validation/              # Validation logic
│   │   ├── tier_classification.rs # PR tier classification
│   │   └── mod.rs
│   ├── webhooks/                # Webhook handlers
│   │   ├── comment.rs           # Comment webhook handler
│   │   └── mod.rs
│   ├── crypto/                  # Cryptographic utilities
│   │   ├── signatures.rs        # Signature management
│   │   └── mod.rs
│   ├── error.rs                 # Error types
│   ├── config.rs                # Configuration types
│   └── main.rs                  # Main application entry point
├── config/                      # Configuration files
│   ├── production.toml.example  # Production configuration template
│   ├── testnet.toml             # Testnet configuration
│   └── development.toml         # Development configuration
├── docs/                        # Application documentation
│   ├── MAINTAINER_SIGNING.md    # Maintainer signing guide
│   ├── ECONOMIC_NODE_CLI.md     # Economic node CLI guide
│   ├── GOVERNANCE_FORK_GUIDE.md # Governance fork guide
│   ├── NOSTR_INTEGRATION.md     # Nostr integration guide
│   ├── OTS_INTEGRATION.md       # OTS integration guide
│   ├── AUDIT_LOG_SYSTEM.md      # Audit log system guide
│   ├── SERVER_AUTHORIZATION.md  # Server authorization guide
│   ├── CONFIG_INTEGRATION.md    # Configuration integration guide
│   └── VERIFICATION.md          # System verification guide
├── migrations/                  # Database migrations
│   └── 001_initial_schema.sql   # Initial database schema
├── Cargo.toml                   # Rust dependencies
├── DEPLOYMENT.md                # Deployment guide
├── SECURITY.md                  # Security guide
└── README.md                    # Application README
```

## Documentation (`docs/`)

Project-wide documentation:

```
docs/
├── production/                  # Production documentation
│   ├── KEY_CEREMONY.md          # Key generation ceremony
│   ├── DEPLOYMENT_GUIDE.md      # Production deployment guide
│   ├── INCIDENT_RESPONSE.md     # Incident response procedures
│   ├── MONITORING_GUIDE.md      # Monitoring and alerting guide
│   ├── SECURITY_GUIDE.md        # Security best practices
│   ├── BACKUP_RECOVERY.md       # Backup and recovery procedures
│   ├── MAINTENANCE_GUIDE.md     # Maintenance procedures
│   └── PRODUCTION_READINESS_CHECKLIST.md # Production readiness checklist
├── ECONOMIC_NODE_CLI.md         # Economic node CLI documentation
├── GOVERNANCE_FORK_GUIDE.md     # Governance fork documentation
├── MAINTAINER_SIGNING.md        # Maintainer signing documentation
└── DIRECTORY_STRUCTURE.md       # This file
```

## Audit Materials (`audit-materials/`)

Security audit materials organized by category:

```
audit-materials/
├── 00-overview/                 # Overview and introduction
│   └── README.md
├── 01-technical/                # Technical documentation
│   ├── ARCHITECTURE.md          # System architecture
│   ├── CRYPTO_DESIGN.md         # Cryptographic design
│   ├── KEY_MANAGEMENT.md        # Key management procedures
│   ├── SQLITE_PRODUCTION_CONFIG.md # SQLite production configuration
│   └── README.md
├── 02-security/                 # Security documentation
│   ├── THREAT_MODEL.md          # Threat model analysis
│   ├── DEPENDENCY_AUDIT.md      # Dependency security audit
│   ├── KNOWN_ISSUES.md          # Known security issues
│   └── README.md
├── 03-testing/                  # Testing documentation
│   ├── TEST_COVERAGE.md         # Test coverage analysis
│   └── README.md
├── 04-audit-process/            # Audit process documentation
│   ├── AUDIT_FIRM_RESEARCH.md   # Audit firm research
│   ├── RFP_TEMPLATE.md          # RFP template for audits
│   └── README.md
└── README.md                    # Audit materials overview
```

## Deployment (`deployment/`)

Deployment configurations for different environments:

```
deployment/
├── testnet/                     # Testnet deployment
│   ├── docker-compose.yml       # Testnet Docker Compose
│   ├── config.toml              # Testnet configuration
│   └── README.md                # Testnet setup guide
└── production/                  # Production deployment (future)
    └── README.md                # Production deployment guide
```

## Scripts (`scripts/`)

Utility scripts for development and deployment:

```
scripts/
├── generate-test-keys.sh        # Generate test keypairs
├── setup-testnet.sh             # Testnet environment setup
├── setup-production.sh          # Production environment setup
├── testnet-test-suite.sh        # Testnet test suite
├── verify-integration.sh        # Integration verification
└── verify-server.sh             # Server verification
```

## Tests (`tests/`)

Test suites for the governance system:

```
tests/
├── integration/                 # Integration tests
│   ├── testnet_scenarios.rs     # Testnet integration scenarios
│   └── mod.rs
└── unit/                        # Unit tests
    └── mod.rs
```

## Path Conventions

### Configuration Files
- **Governance configs**: `governance/config/*.yml`
- **Application configs**: `governance-app/config/*.toml`
- **Deployment configs**: `deployment/*/config.toml`

### Documentation Files
- **Project docs**: `docs/*.md`
- **Application docs**: `governance-app/docs/*.md`
- **Architecture docs**: `governance/architecture/*.md`
- **Audit materials**: `audit-materials/0X-category/*.md`

### Source Code
- **Rust source**: `governance-app/src/`
- **CLI tools**: `governance-app/src/bin/`
- **Tests**: `tests/`

### Scripts and Utilities
- **Setup scripts**: `scripts/setup-*.sh`
- **Test scripts**: `scripts/*-test-*.sh`
- **Verification scripts**: `scripts/verify-*.sh`

## File Naming Conventions

- **Configuration files**: `kebab-case.yml` or `kebab-case.toml`
- **Documentation files**: `UPPER_CASE.md`
- **Source files**: `snake_case.rs`
- **Scripts**: `kebab-case.sh`

## Important Notes

1. **Configuration Loading**: The governance-app loads configuration from `governance/config/` directory
2. **Documentation Links**: All internal documentation links use relative paths
3. **File References**: Code references use the new organized structure
4. **Backward Compatibility**: Old file paths are no longer supported
5. **Maintenance**: This structure should be maintained as the project evolves

## Migration History

The current structure is the result of a reorganization that moved files from flat structures to organized subdirectories:

- `governance/*.yml` → `governance/config/*.yml`
- `audit-materials/*.md` → `audit-materials/0X-category/*.md`
- Various docs moved to appropriate subdirectories

This reorganization improves maintainability and makes the project structure more intuitive for new contributors.
