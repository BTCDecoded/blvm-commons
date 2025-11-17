# Practical Configuration Implementation Plan

## Critical Bug Found

**Current Issue**: Config is loaded but **never applied to the node**!

```rust
// Config is loaded...
let mut config = NodeConfig::from_file(...);

// But node is created WITHOUT config!
let mut node = ReferenceNode::new(...); // Config not passed!

// Comment says "Apply configuration" but nothing happens!
// Apply configuration to node (if node supports runtime config)
```

**Fix Required**: Actually apply the config using `with_config()` and `with_modules_from_config()`.

## What Needs Implementation

### Tier 1: Critical Fixes (DO FIRST)

1. **Apply config to node** ✅ **MUST FIX**
   - Use `node.with_config(config)`
   - Use `node.with_modules_from_config(&config)`
   - Currently config is loaded but ignored!

2. **ENV variable support** ✅ **MUST HAVE**
   - Deployment-critical settings (data_dir, network, addresses)
   - Secrets (never in config files)
   - Feature flags (convenience)

3. **Config hierarchy** ✅ **MUST HAVE**
   - Priority: CLI > ENV > Config File > Defaults
   - Currently only CLI and Config File work

### Tier 2: Enhancements (SHOULD HAVE)

4. **ENV for feature flags** - Convenience for deployment
5. **CLI for common overrides** - `--max-peers`, `--transport`

### Tier 3: Nice to Have (CAN WAIT)

6. **Unified config system** - Single file for all components
7. **Config validation** - Validate at startup
8. **Config includes** - Include component-specific configs

## What Needs ENV Overrides?

### Must Have (Deployment Critical)
- `BLLVM_DATA_DIR` - Where to store blockchain data
- `BLLVM_NETWORK` - Which network (regtest/testnet/mainnet)
- `BLLVM_LISTEN_ADDR` - P2P listen address
- `BLLVM_RPC_ADDR` - RPC server address
- `BLLVM_LOG_LEVEL` - Logging verbosity

### Should Have (Common Overrides)
- `BLLVM_NODE_MAX_PEERS` - Maximum peer connections
- `BLLVM_NODE_TRANSPORT` - Transport preference

### Feature Flags (Convenience)
- `BLLVM_NODE_FEATURES_STRATUM_V2=true/false`
- `BLLVM_NODE_FEATURES_DANDELION=true/false`
- `BLLVM_NODE_FEATURES_BIP158=true/false`

### Secrets Only (ENV, never config file)
- `BLLVM_RPC_AUTH_TOKEN` - RPC authentication token
- `BLLVM_GITHUB_WEBHOOK_SECRET` - GitHub webhook secret
- `BLLVM_NOSTR_SERVER_NSEC` - Nostr server key

## What Needs CLI Overrides?

### Already Have (Good as-is)
- `--network` ✅
- `--data-dir` ✅
- `--listen-addr` ✅
- `--rpc-addr` ✅
- `--config` ✅
- `--verbose` ✅
- Feature flags ✅

### Should Add (Quick Overrides)
- `--max-peers` - Override max peer connections
- `--transport` - Override transport preference

### Don't Need CLI (Use Config File)
- Complex nested configs (modules, storage, governance)
- Multiple values (persistent peers, relay lists)
- Detailed settings (pruning config, RPC auth details)

## Implementation Plan

### Step 1: Fix Config Application (CRITICAL)

**Current (BROKEN):**
```rust
let mut config = NodeConfig::from_file(...);
let mut node = ReferenceNode::new(...); // Config ignored!
```

**Fixed:**
```rust
let mut config = build_final_config(&cli); // Load with hierarchy
let mut node = ReferenceNode::new(...)
    .with_config(config.clone())  // Apply config!
    .with_modules_from_config(&config)?;  // Apply modules!
```

### Step 2: Add ENV Variable Support

```rust
fn load_from_env() -> EnvOverrides {
    EnvOverrides {
        data_dir: env::var("BLLVM_DATA_DIR").ok(),
        network: env::var("BLLVM_NETWORK").ok(),
        listen_addr: env::var("BLLVM_LISTEN_ADDR")
            .ok()
            .and_then(|s| s.parse().ok()),
        rpc_addr: env::var("BLLVM_RPC_ADDR")
            .ok()
            .and_then(|s| s.parse().ok()),
        log_level: env::var("BLLVM_LOG_LEVEL").ok(),
        max_peers: env::var("BLLVM_NODE_MAX_PEERS")
            .ok()
            .and_then(|s| s.parse().ok()),
        transport: env::var("BLLVM_NODE_TRANSPORT").ok(),
        // Feature flags
        stratum_v2: env::var("BLLVM_NODE_FEATURES_STRATUM_V2")
            .ok()
            .and_then(|s| s.parse().ok()),
        dandelion: env::var("BLLVM_NODE_FEATURES_DANDELION")
            .ok()
            .and_then(|s| s.parse().ok()),
    }
}
```

### Step 3: Implement Config Hierarchy

```rust
fn build_final_config(cli: &Cli) -> NodeConfig {
    // 1. Start with defaults
    let mut config = NodeConfig::default();
    
    // 2. Load config file (if provided or found)
    if let Some(config_path) = find_config_file(&cli.config) {
        if let Ok(file_config) = NodeConfig::from_file(&config_path) {
            config = file_config; // Config file overrides defaults
        }
    }
    
    // 3. Apply ENV overrides
    let env = load_from_env();
    if let Some(data_dir) = env.data_dir {
        // ENV overrides config file (but CLI will override this)
    }
    if let Some(network) = env.network {
        // ENV overrides config file
    }
    if let Some(max_peers) = env.max_peers {
        config.max_peers = Some(max_peers);
    }
    
    // 4. Apply CLI overrides (highest priority)
    config.listen_addr = Some(cli.listen_addr); // CLI always wins
    config.protocol_version = Some(format!("{:?}", cli.network));
    if cli.data_dir != "./data" { // Only if explicitly set
        // CLI overrides everything
    }
    
    config
}
```

## Answer to Your Questions

### Do we need to implement these configuration options?
**Yes, but prioritize:**
1. **CRITICAL**: Fix config application bug (config loaded but not used)
2. **MUST**: Add ENV support for deployment (data_dir, network, addresses)
3. **SHOULD**: ENV for feature flags, CLI for common overrides
4. **NICE**: Unified config system (can wait)

### Do they need ENV overrides?
**Selectively:**
- **Yes**: Deployment-critical (data_dir, network, addresses, secrets)
- **Yes**: Feature flags (convenience for deployment)
- **No**: Complex nested configs (use config file)

### Do they need CLI overrides?
**Selectively:**
- **Yes**: Common operations (network, data_dir, addresses, feature flags) ✅ Already have
- **Should**: Quick overrides (`--max-peers`, `--transport`)
- **No**: Complex nested configs (use config file)

### How does this play out?

**Phase 1: Fix Critical Issues (IMMEDIATE)**
1. Fix config application bug
2. Add ENV variable support
3. Implement config hierarchy (CLI > ENV > Config > Defaults)

**Phase 2: Enhancements (SHORT-TERM)**
4. Add ENV for feature flags
5. Add CLI for common overrides (`--max-peers`, `--transport`)

**Phase 3: Nice to Have (LONG-TERM)**
6. Unified config system (single file for all components)
7. Config validation
8. Config includes

**Bottom Line**: Start with fixing the bug and adding ENV support. Don't over-engineer. Keep it practical.
