# Incremental Pruning During IBD Guide

## Overview

This guide explains how to use incremental pruning during Initial Block Download (IBD) to avoid downloading the full blockchain before pruning. This feature is enabled when UTXO commitments are available.

## What Changed

### Before
- Pruning was **blocked during IBD**
- Nodes had to download the full blockchain (~600GB) before pruning
- Storage grew unbounded during sync

### After
- **Incremental pruning during IBD** is now possible with UTXO commitments
- Nodes can prune old blocks as they sync forward
- Storage stays constant (sliding window) instead of growing linearly

## How It Works

### Algorithm

1. **Download Headers First** (lightweight, ~80 bytes per block)
2. **Get UTXO Commitment at Checkpoint** (84 bytes)
3. **Sync Forward Incrementally**:
   - Download filtered blocks
   - Process blocks and update UTXO set
   - Generate commitment for each block
   - **Prune blocks older than window size** (e.g., keep last 144 blocks)
4. **Continue Until Synced**

### What Gets Kept

- ✅ **All block headers** (required for PoW verification)
- ✅ **UTXO commitments** (for state verification)
- ✅ **Recent N blocks** (configurable window, default 144 blocks)

### What Gets Pruned

- ❌ **Old block bodies** (beyond window size)
- ❌ **Old witness data** (if not keeping filtered blocks)
- ❌ **Old BIP158 filters** (if enabled, but headers kept)

## Configuration

### Enable Incremental Pruning

Add to your node configuration:

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

### Configuration Options

- **`incremental_prune_during_ibd`** (default: `false`)
  - Enable incremental pruning during IBD
  - Requires: UTXO commitments feature + aggressive/custom mode with commitments

- **`prune_window_size`** (default: `144`)
  - Number of recent blocks to keep
  - 144 blocks ≈ 1 day at 10 min/block
  - Storage stays roughly constant at this size

- **`min_blocks_for_incremental_prune`** (default: `288`)
  - Minimum blocks before starting incremental pruning
  - 288 blocks ≈ 2 days at 10 min/block
  - Safety margin to ensure stability

## Requirements

### Required Features

1. **UTXO Commitments Feature**: Must be enabled (`utxo-commitments` feature flag)
2. **Pruning Mode**: Must be `Aggressive` or `Custom` with `keep_commitments: true`
3. **Commitment Store**: Must be initialized and available

### Prerequisites Check

The system automatically checks:
- ✅ UTXO commitments feature enabled
- ✅ Commitment store available
- ✅ UTXO store available
- ✅ Pruning mode supports commitments

If any check fails, incremental pruning is disabled and you'll get a clear error message.

## Usage

### Automatic (Recommended)

Incremental pruning happens automatically during IBD when:
- `incremental_prune_during_ibd` is enabled
- Prerequisites are met
- Minimum block threshold is reached

### Manual

You can also call it manually during sync:

```rust
// In your sync loop
if let Some(stats) = pruning_manager.incremental_prune_during_ibd(current_height, is_ibd)? {
    info!("Pruned {} blocks, freed {} bytes", stats.blocks_pruned, stats.storage_freed);
}
```

## Benefits

### Storage Savings

- **Before**: Storage grows linearly with blockchain size (~600GB)
- **After**: Storage stays constant (~window size, e.g., 144 blocks)

### Sync Speed

- **Before**: Must download full blockchain before pruning
- **After**: Only download what's needed (headers + recent blocks)

### State Verification

- **Before**: Need full blocks to verify state
- **After**: Can verify state via UTXO commitments

## Example Scenarios

### Scenario 1: Storage-Constrained Node

**Goal**: Run a node with minimal storage

**Configuration**:
```json
{
  "storage": {
    "pruning": {
      "mode": "aggressive",
      "keep_commitments": true,
      "incremental_prune_during_ibd": true,
      "prune_window_size": 144
    }
  }
}
```

**Result**: 
- Downloads headers (~40MB)
- Gets UTXO commitment at checkpoint (~84 bytes)
- Syncs forward, keeping only last 144 blocks
- Storage: ~144 blocks + headers + commitments

### Scenario 2: Fast Initial Sync

**Goal**: Sync quickly without waiting for full download

**Configuration**:
```json
{
  "storage": {
    "pruning": {
      "mode": "aggressive",
      "keep_commitments": true,
      "incremental_prune_during_ibd": true,
      "prune_window_size": 288,
      "min_blocks_for_incremental_prune": 144
    }
  }
}
```

**Result**:
- Starts pruning earlier (after 144 blocks)
- Keeps larger window (288 blocks)
- Faster sync, more recent history

## Safety Considerations

### What's Safe

- ✅ Headers are always kept (required for PoW)
- ✅ Commitments are generated before pruning
- ✅ Minimum block threshold prevents early pruning
- ✅ State can be verified via commitments

### What to Watch

- ⚠️ **Irreversible**: Pruned blocks cannot be recovered (unless you have backups)
- ⚠️ **Window Size**: Too small may cause issues if you need older blocks
- ⚠️ **Commitment Generation**: Must complete before pruning

## Troubleshooting

### Error: "Cannot prune during initial block download"

**Cause**: Incremental pruning not enabled or prerequisites not met

**Solution**: 
1. Enable `incremental_prune_during_ibd: true`
2. Ensure UTXO commitments feature is enabled
3. Use `Aggressive` or `Custom` mode with commitments

### Error: "Cannot prune until at least N blocks are synced"

**Cause**: Not enough blocks synced yet

**Solution**: Wait until minimum block threshold is reached (default 288 blocks)

### Pruning Not Happening

**Check**:
1. Is `incremental_prune_during_ibd` enabled?
2. Are UTXO commitments available?
3. Is pruning mode correct?
4. Have minimum blocks been synced?

## Integration with Sync Process

### Recommended Integration

Call `incremental_prune_during_ibd` periodically during sync:

```rust
// After processing each block (or every N blocks)
if let Some(stats) = pruning_manager.incremental_prune_during_ibd(current_height, is_ibd)? {
    // Log pruning statistics
    info!("Incremental prune: {} blocks pruned", stats.blocks_pruned);
}
```

### When to Call

- After processing each block (fine-grained)
- Every N blocks (e.g., every 10 blocks, less overhead)
- When storage threshold reached (e.g., every 1GB)

## Performance

### Overhead

- **Commitment Generation**: ~1-10ms per block (depends on UTXO set size)
- **Pruning Operation**: ~1-5ms per block (depends on storage backend)
- **Total**: Minimal impact on sync speed

### Storage Savings

- **Without Incremental Pruning**: ~600GB (full blockchain)
- **With Incremental Pruning**: ~144 blocks + headers + commitments
  - Headers: ~40MB
  - Recent blocks: ~144 * 1MB = ~144MB
  - Commitments: ~84 bytes * height = minimal
  - **Total**: ~200MB vs 600GB (99.97% savings)

## Summary

✅ **Incremental pruning during IBD** enables:
- Constant storage (sliding window)
- Faster sync (no full download needed)
- State verification via commitments

✅ **Requirements**:
- UTXO commitments feature enabled
- Aggressive/Custom pruning mode
- Incremental pruning enabled in config

✅ **Benefits**:
- 99%+ storage savings
- Faster initial sync
- Maintains state verification capability

