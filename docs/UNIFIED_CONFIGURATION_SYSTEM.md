# Unified Configuration System for BLLVM Ecosystem

## Problem Statement

The BLLVM ecosystem spans multiple repositories:
- **bllvm** (binary wrapper)
- **bllvm-node** (node implementation)
- **bllvm-protocol** (protocol abstraction)
- **bllvm-consensus** (consensus layer)
- **bllvm-sdk** (SDK)
- **governance-app** (governance enforcement)

Currently, each component has its own configuration system:
- `bllvm-node` uses `NodeConfig` (JSON/TOML)
- `governance-app` uses separate TOML files
- `bllvm` binary has CLI flags
- No unified way to configure the entire stack

## Solution: Unified Configuration System

### Design Principles

1. **Single Source of Truth**: One config file for the entire ecosystem
2. **Hierarchical Sections**: Each component has its own section
3. **Backward Compatible**: Existing component configs still work
4. **Override Hierarchy**: Defaults → Config File → Env Vars → CLI Args
5. **Include Support**: Can include component-specific configs
6. **Validation**: Validate config at startup

### Configuration File Structure

```toml
# bllvm.toml - Unified Configuration for Bitcoin Commons BLLVM Ecosystem
# This file configures all components of the BLLVM stack

# Global settings
[global]
# Data directory (used by all components)
data_dir = "./data"
# Logging level (trace, debug, info, warn, error)
log_level = "info"
# Log format (json, text)
log_format = "text"
# Environment (development, testnet, production)
environment = "development"

# Node configuration (bllvm-node)
[node]
# Network to connect to (regtest, testnet, mainnet)
network = "regtest"
# P2P listen address
listen_addr = "0.0.0.0:8333"
# RPC server address
rpc_addr = "127.0.0.1:18332"
# Maximum number of peers
max_peers = 100
# Transport preference (tcp_only, iroh_only, hybrid, all)
transport_preference = "tcp_only"
# Enable self-advertisement
enable_self_advertisement = true

# Persistent peers
[node.persistent_peers]
# persistent_peers = ["1.2.3.4:8333", "5.6.7.8:8333"]

# Storage configuration
[node.storage]
# Backend (sled, redb)
backend = "redb"
# Enable pruning
pruning_enabled = false
# Pruning mode (normal, aggressive)
pruning_mode = "normal"
# Minimum blocks to keep
min_blocks_to_keep = 288

# Module system
[node.modules]
enabled = true
modules_dir = "modules"
data_dir = "data/modules"
socket_dir = "data/modules/sockets"
# enabled_modules = []  # Empty = auto-discover all

# Stratum V2 mining (requires compile-time feature)
[node.stratum_v2]
enabled = false
# pool_url = "tcp://pool.example.com:3333"
# listen_addr = "0.0.0.0:3333"
# merge_mining_enabled = false
# secondary_chains = []

# RPC authentication
[node.rpc_auth]
required = false
# tokens = []
# certificates = []
rate_limit_burst = 100
rate_limit_rate = 10

# Ban list sharing
[node.ban_list_sharing]
enabled = false
share_interval_seconds = 3600
max_entries = 1000

# Feature flags (runtime-configurable)
[node.features]
# Enable/disable features (requires compile-time features)
# stratum_v2 = false
# bip158 = true
# dandelion = false
# sigop = true

# Protocol configuration (bllvm-protocol)
[protocol]
# Protocol version (BitcoinV1, Testnet3, Regtest)
version = "Regtest"
# Enable UTXO commitments (requires compile-time feature)
utxo_commitments_enabled = true
# Enable production optimizations
production_optimizations = true

# Consensus configuration (bllvm-consensus)
[consensus]
# Enable formal verification (Kani proofs)
verify_enabled = false
# Enable benchmarking utilities
benchmarking_enabled = false
# Enable fuzzing tests (Bolero)
bolero_enabled = false

# Governance App configuration (governance-app)
[governance]
# Enable governance enforcement
enabled = false
# Configuration file path
config_path = "governance/config"
# Dry run mode (don't enforce, just log)
dry_run_mode = true
# Log enforcement decisions
log_enforcement_decisions = true
# Enforcement log path
enforcement_log_path = "logs/enforcement-decisions.jsonl"
# Server ID
server_id = "governance-01"

# Governance App server
[governance.server]
host = "0.0.0.0"
port = 8080
workers = 4

# Governance App database
[governance.database]
url = "sqlite:governance.db"
max_connections = 10
min_connections = 1
acquire_timeout = 30
idle_timeout = 600
max_lifetime = 1800

# Governance App GitHub integration
[governance.github]
app_id = "123456"
private_key_path = "keys/github-app-key.pem"
webhook_secret = "env:GITHUB_WEBHOOK_SECRET"
base_url = "https://api.github.com"
timeout = 30

# Governance App Nostr integration
[governance.nostr]
enabled = false
server_nsec_path = "keys/nostr-server.nsec"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol"
]
publish_interval_secs = 3600

# Governance App OpenTimestamps
[governance.ots]
enabled = false
aggregator_url = "https://alice.btc.calendar.opentimestamps.org"
monthly_anchor_day = 1
registry_path = "data/governance/registries"
proofs_path = "data/governance/ots-proofs"

# Governance App audit log
[governance.audit]
enabled = true
log_path = "data/governance/audit-log.jsonl"
rotation_interval_days = 30

# Economic nodes
[governance.economic_nodes]
enabled = false
veto_threshold_percent = 30.0
hash_rate_threshold_percent = 30.0
economic_activity_threshold_percent = 40.0

# Governance fork
[governance.fork]
enabled = false
export_path = "data/governance-exports"
adoption_tracking = true

# SDK configuration (bllvm-sdk)
[sdk]
# SDK-specific settings
# (Currently minimal, may expand)

# Include external config files (optional)
# [includes]
# node = "config/node.toml"
# governance = "config/governance.toml"
```

### Implementation Strategy

#### Phase 1: Unified Config Structure

1. **Create `BllvmConfig` struct** in `bllvm` binary:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct BllvmConfig {
       pub global: GlobalConfig,
       pub node: Option<NodeConfig>,
       pub protocol: Option<ProtocolConfig>,
       pub consensus: Option<ConsensusConfig>,
       pub governance: Option<GovernanceConfig>,
       pub sdk: Option<SdkConfig>,
   }
   ```

2. **Load unified config** in `bllvm` binary:
   - Check for `bllvm.toml` in current directory
   - Check for `~/.config/bllvm/bllvm.toml`
   - Check for `/etc/bllvm/bllvm.toml`
   - Support `--config` flag to specify path

3. **Extract component configs**:
   - Extract `node` section → `NodeConfig`
   - Extract `governance` section → `GovernanceAppConfig`
   - Pass to respective components

#### Phase 2: Component Integration

1. **Update `bllvm-node`**:
   - Accept `NodeConfig` from unified config
   - Maintain backward compatibility with standalone configs
   - Use `node` section from unified config

2. **Update `governance-app`**:
   - Accept `GovernanceConfig` from unified config
   - Maintain backward compatibility with standalone configs
   - Use `governance` section from unified config

3. **Update `bllvm` binary**:
   - Load unified config
   - Extract and pass component configs
   - Support CLI overrides

#### Phase 3: Include Support

1. **Support config includes**:
   ```toml
   [includes]
   node = "config/node.toml"
   governance = "config/governance.toml"
   ```

2. **Merge included configs**:
   - Load included files
   - Merge with main config
   - Later includes override earlier ones

### Configuration Hierarchy

Priority order (highest to lowest):

1. **CLI Arguments** - Highest priority, always wins
2. **Environment Variables** - Override config file
3. **Config File** - Main configuration source
4. **Defaults** - Built-in defaults

### Environment Variable Mapping

Environment variables follow pattern: `BLLVM_<SECTION>_<KEY>`

Examples:
- `BLLVM_NODE_LISTEN_ADDR=0.0.0.0:8333`
- `BLLVM_GOVERNANCE_ENABLED=true`
- `BLLVM_NODE_FEATURES_STRATUM_V2=true`

### CLI Override Examples

```bash
# Use unified config
bllvm --config bllvm.toml

# Override specific settings
bllvm --config bllvm.toml --node-listen-addr 0.0.0.0:8334

# Enable features via CLI
bllvm --config bllvm.toml --enable-stratum-v2 --enable-dandelion

# Disable governance
bllvm --config bllvm.toml --governance-enabled false
```

### Benefits

1. **Single Source of Truth**: One file configures everything
2. **Easier Deployment**: Copy one config file instead of multiple
3. **Consistent Structure**: All components follow same pattern
4. **Better Documentation**: One place to document all settings
5. **Validation**: Can validate entire stack configuration at once
6. **Backward Compatible**: Existing configs still work

### Migration Path

1. **Phase 1**: Add unified config support alongside existing configs
2. **Phase 2**: Document unified config as recommended approach
3. **Phase 3**: Deprecate (but still support) standalone configs
4. **Phase 4**: Make unified config the default

### File Locations

Unified config file search order:
1. `--config` flag path
2. `./bllvm.toml` (current directory)
3. `~/.config/bllvm/bllvm.toml` (user config)
4. `/etc/bllvm/bllvm.toml` (system config)
5. Defaults (no config file)

### Example Usage

```bash
# Development: Use local config
bllvm --config bllvm.toml

# Production: Use system config
bllvm  # Automatically loads /etc/bllvm/bllvm.toml

# Override with CLI
bllvm --node-network mainnet --governance-enabled true

# Override with environment
export BLLVM_NODE_NETWORK=mainnet
export BLLVM_GOVERNANCE_ENABLED=true
bllvm
```

