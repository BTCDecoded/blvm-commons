# UTXO Commitments, Filtering, and Pruning Analysis

## Overview

This document analyzes how UTXO commitments, filtering, and pruning work together, and how to enable incremental pruning during initial block download (IBD) without requiring the full blockchain to be downloaded first.

## Current State

### 1. UTXO Commitments
- **Location**: `bllvm-consensus/src/utxo_commitments/`
- **Purpose**: Cryptographic commitments to UTXO set state (84 bytes per commitment)
- **Benefits**: 
  - 98% initial sync savings (13GB vs 600GB)
  - Enables state verification without full block history
  - Foundation for aggressive pruning

### 2. Filtering
There are two types of filtering:

#### A. Spam Filtering (Transaction-level)
- **Location**: `bllvm-consensus/src/utxo_commitments/spam_filter.rs`
- **Purpose**: Filter spam transactions (Ordinals, dust, BRC-20)
- **Behavior**: Filters OUTPUTS only (inputs always processed for UTXO consistency)
- **Bandwidth Savings**: 40-60% reduction

#### B. BIP158 Compact Block Filters (Script-level)
- **Location**: `bllvm-protocol/src/bip158.rs`
- **Purpose**: Light client transaction discovery
- **Behavior**: Golomb-Rice coded sets for efficient script matching
- **Use Case**: Light clients, wallet scanning

### 3. Pruning
- **Location**: `bllvm-node/src/storage/pruning.rs`
- **Purpose**: Remove old blocks from storage
- **Current Limitation**: **Blocked during IBD** (line 157-160)
- **Modes**:
  - `Disabled`: No pruning
  - `Normal`: Keep recent blocks
  - `Aggressive`: Prune with UTXO commitments (requires feature flag)
  - `Custom`: Fine-grained control

## Can These Be Disabled or Combined?

### Yes, All Can Be Disabled

1. **UTXO Commitments**: Disabled if `utxo-commitments` feature flag is off
2. **Spam Filtering**: Can be disabled via `SpamFilterConfig`
3. **BIP158 Filters**: Disabled if `bip158` feature flag is off
4. **Pruning**: Set mode to `Disabled`

### They Can Be Combined

**Example Combinations**:

1. **UTXO Commitments + Aggressive Pruning**:
   - Download headers first
   - Get UTXO commitment at checkpoint
   - Sync forward with filtered blocks
   - Prune old blocks as you sync (incremental pruning)

2. **Spam Filtering + Pruning**:
   - Filter spam during sync
   - Prune filtered blocks after processing
   - Keep only non-spam data

3. **BIP158 Filters + Pruning**:
   - Generate filters for blocks
   - Prune block bodies but keep filters
   - Serve light clients from filters

## The Key Problem: Incremental Pruning During IBD

### Current Behavior
```rust
// From pruning.rs line 157-160
if is_ibd {
    return Err(anyhow!(
        "Cannot prune during initial block download. Wait for IBD to complete."
    ));
}
```

**Problem**: This prevents pruning during sync, forcing nodes to download the full blockchain before pruning.

### Why This Exists
The restriction exists because:
1. Without UTXO commitments, you need full blocks to verify state
2. Pruning too early could break validation
3. Safety measure to prevent data loss

### Why It Should Be Removed (With UTXO Commitments)

With UTXO commitments enabled, we can:
1. **Download headers first** (lightweight, ~80 bytes per block)
2. **Get UTXO commitment at checkpoint** (84 bytes)
3. **Sync forward incrementally** with filtered blocks
4. **Prune old blocks as we sync** because:
   - We have commitments for verification
   - We don't need full block history
   - We can verify state via commitments

### Proposed Solution: Incremental Pruning

**Algorithm**:
1. Download headers chain (all headers, lightweight)
2. Use UTXO commitments to get checkpoint UTXO set
3. Sync forward from checkpoint:
   - Download filtered blocks
   - Process blocks and update UTXO set
   - Generate commitment for each block
   - **Prune blocks older than N blocks** (e.g., keep last 144 blocks)
4. Continue until synced

**Key Insight**: With UTXO commitments, we only need to keep:
- All block headers (for PoW verification)
- UTXO commitments (for state verification)
- Recent N blocks (for recent history)

## Implementation Plan

### Phase 1: Allow Incremental Pruning During IBD

**Changes to `pruning.rs`**:
- Remove blanket IBD restriction
- Add check: if UTXO commitments enabled AND aggressive pruning mode, allow incremental pruning
- Add `incremental_prune_during_ibd` configuration option

### Phase 2: Integrate with Sync Process

**Changes to sync coordinator**:
- After processing each block during IBD:
  - Generate UTXO commitment
  - Check if pruning threshold reached
  - Prune old blocks (keep headers + commitments)

### Phase 3: Pruning Window

**Concept**: Keep a sliding window of blocks
- **Window Size**: Configurable (e.g., 144 blocks = ~1 day)
- **Pruning Trigger**: When window exceeds size, prune oldest blocks
- **What to Keep**:
  - Headers (always)
  - Commitments (if enabled)
  - Recent N blocks (configurable)

## Configuration

### New Configuration Options

```rust
pub struct PruningConfig {
    // ... existing fields ...
    
    /// Allow incremental pruning during IBD (requires UTXO commitments)
    #[serde(default = "default_false")]
    pub incremental_prune_during_ibd: bool,
    
    /// Block window size for incremental pruning (blocks to keep)
    #[serde(default = "default_prune_window_size")]
    pub prune_window_size: u64, // e.g., 144 blocks
    
    /// Minimum blocks before starting incremental pruning
    #[serde(default = "default_min_blocks_for_incremental_prune")]
    pub min_blocks_for_incremental_prune: u64, // e.g., 288 blocks
}
```

### Example Configuration

```json
{
  "storage": {
    "pruning": {
      "mode": "aggressive",
      "keep_commitments": true,
      "incremental_prune_during_ibd": true,
      "prune_window_size": 144,
      "min_blocks_for_incremental_prune": 288
    }
  }
}
```

**Behavior**:
- During IBD, after syncing 288 blocks, start incremental pruning
- Keep only last 144 blocks in storage
- Keep all headers
- Keep all UTXO commitments
- Prune block bodies older than window

## Benefits

1. **No Full Blockchain Download**: Only download what's needed
2. **Constant Storage**: Storage stays constant (window size) instead of growing linearly
3. **Faster Sync**: Less data to download and process
4. **State Verification**: Can still verify state via commitments

## Safety Considerations

1. **UTXO Commitments Required**: Incremental pruning during IBD only works with UTXO commitments
2. **Minimum Blocks**: Don't start pruning until we have enough blocks (safety margin)
3. **Commitment Generation**: Must generate commitments before pruning blocks
4. **Header Chain**: Always keep all headers (required for PoW verification)

## Testing

1. **Unit Tests**: Test incremental pruning logic
2. **Integration Tests**: Test IBD with incremental pruning
3. **State Verification**: Verify state can be verified after pruning
4. **Edge Cases**: Test with various window sizes and thresholds

## Summary

- ✅ **All features can be disabled** individually
- ✅ **Features can be combined** in various ways
- ✅ **Incremental pruning during IBD is possible** with UTXO commitments
- ⚠️ **Current implementation blocks it** - needs to be fixed
- 🎯 **Solution**: Allow incremental pruning when UTXO commitments + aggressive pruning enabled

