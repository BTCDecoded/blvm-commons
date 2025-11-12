# Commons Filtering and Pruning Capabilities

This document describes the filtering and pruning capabilities provided by **bllvm-consensus** (Bitcoin Commons Consensus Proof) and how they're integrated into the node.

## Overview

**bllvm-consensus** provides two main filtering mechanisms:
1. **Spam Filtering** - Transaction-level filtering for bandwidth optimization
2. **BIP158 Compact Block Filters** - Script-based filtering for light clients

**Pruning** is implemented at the node level (not in Commons), but Commons provides the foundation for efficient pruning through UTXO commitments.

## 1. Spam Filtering (bllvm-consensus)

### Location
- **Module**: `bllvm-consensus/src/utxo_commitments/spam_filter.rs`
- **Integration**: Used in UTXO commitment processing and filtered block generation

### Capabilities

#### Spam Detection Types
1. **Ordinals/Inscriptions** (`SpamType::Ordinals`)
   - Detects data embedded in witness scripts (SegWit v0 or Taproot)
   - Detects OP_RETURN outputs with large data pushes
   - Detects envelope protocol patterns (OP_FALSE OP_IF ... OP_ENDIF)
   - Pattern: Large scripts (>100 bytes) or OP_RETURN with >80 bytes

2. **Dust Outputs** (`SpamType::Dust`)
   - Filters outputs below threshold (default: 546 satoshis)
   - Configurable via `SpamFilterConfig::dust_threshold`
   - All outputs must be below threshold for transaction to be considered dust

3. **BRC-20 Tokens** (`SpamType::BRC20`)
   - Detects BRC-20 JSON patterns in OP_RETURN outputs
   - Patterns: `"p":"brc-20"`, `"op":"mint"`, `"op":"transfer"`, `"op":"deploy"`

### Configuration

```rust
use bllvm_consensus::utxo_commitments::spam_filter::{SpamFilter, SpamFilterConfig};

let config = SpamFilterConfig {
    filter_ordinals: true,
    filter_dust: true,
    filter_brc20: true,
    dust_threshold: 546, // satoshis
    min_output_value: 546, // satoshis
};

let filter = SpamFilter::with_config(config);
```

### Critical Design: Output-Only Filtering

**Important**: Spam filtering applies to **OUTPUTS only**, not entire transactions.

When processing a spam transaction:
- ✅ **INPUTS are ALWAYS removed** from UTXO tree (maintains consistency)
- ❌ **OUTPUTS are filtered out** (bandwidth savings)

This ensures UTXO set consistency even when spam transactions spend non-spam inputs.

**Implementation**: See `bllvm-consensus/src/utxo_commitments/initial_sync.rs::process_filtered_block()`

### Bandwidth Savings

- **40-60% bandwidth reduction** during ongoing sync
- Maintains consensus correctness
- Enables efficient UTXO commitment synchronization

### Usage in Node

```rust
// In protocol_extensions.rs
let spam_filter = SpamFilter::new();
let (filtered_txs, spam_summary) = spam_filter.filter_block(&block.transactions);

// Spam summary provides statistics:
// - filtered_count: Number of transactions filtered
// - filtered_size: Total bytes filtered
// - by_type: Breakdown by spam type (ordinals, dust, brc20)
```

## 2. BIP158 Compact Block Filters

### Location
- **Node Implementation**: `bllvm-node/src/bip158.rs`
- **Service**: `bllvm-node/src/network/filter_service.rs`
- **Integration**: Used for light client support

### Capabilities

#### Filter Generation
- **Golomb-Rice Coded Sets (GCS)** for efficient encoding
- **False Positive Rate**: ~1 in 524,288 (P=19)
- **Filter Contents**:
  1. All spendable output scriptPubKeys in the block
  2. All scriptPubKeys from outputs spent by block's inputs

#### Filter Header Chain
- Maintains filter header chain for efficient verification
- Checkpoints every 1000 blocks (per BIP157)
- Enables light clients to verify filter integrity

### Algorithm

1. **Collect Scripts**:
   - All output scriptPubKeys from block transactions
   - All scriptPubKeys from UTXOs being spent (previous_outpoint_scripts)

2. **Hash to Range**:
   - Hash each script with SHA256
   - Map to range [0, N*M) where N = number of elements, M = 2^19

3. **Golomb-Rice Encoding**:
   - Sort hashed values
   - Compute differences between consecutive values
   - Encode differences using Golomb-Rice (unary + binary)

4. **Filter Matching**:
   - Light clients hash their scripts
   - Decode filter to reconstruct sorted set
   - Binary search to check if script hash is in set

### Usage in Node

```rust
use crate::bip158::build_block_filter;

// Generate filter for a block
let filter = build_block_filter(
    &block.transactions,
    &previous_outpoint_scripts // From UTXO set
)?;

// Match script against filter
let matches = match_filter(&filter, &script_pubkey);
```

### Integration Points

- **BlockFilterService**: Caches filters, maintains filter header chain
- **RPC Method**: `getblockfilter` - Returns BIP158 filter for a block
- **Network Protocol**: BIP157 messages for filter requests
- **UTXO Commitments**: Optional BIP158 filter in `FilteredBlockMessage`

## 3. Pruning

### Current Status

**Pruning is NOT implemented in Commons** - it's a node-level operation.

**Node Implementation**:
- **RPC Method**: `pruneblockchain` (placeholder in `bllvm-node/src/rpc/blockchain.rs`)
- **Status**: Simplified implementation - marks blocks for pruning but doesn't actually remove data

### Pruning Requirements

For full pruning implementation, the node would need to:

1. **Mark Blocks for Pruning**:
   - Keep block headers (always needed for PoW verification)
   - Mark block bodies for deletion
   - Keep UTXO set (required for validation)

2. **Update UTXO Set**:
   - Remove spent outputs from pruned blocks
   - Maintain UTXO set consistency

3. **Update Chain State**:
   - Update tip height
   - Update chain work calculations

4. **Storage Cleanup**:
   - Actually remove block data from storage
   - Update indexes

### Commons Foundation for Pruning

While Commons doesn't implement pruning directly, it provides:

1. **UTXO Commitments**: Enable efficient UTXO set verification without full blocks
2. **Spam Filtering**: Reduces data that needs to be stored
3. **Filtered Blocks**: Can serve pruned data efficiently via filtered blocks

### Future Pruning Integration

When pruning is fully implemented, it would:
- Use Commons' UTXO commitment verification for pruned block validation
- Use spam filtering to reduce storage requirements
- Use BIP158 filters to serve light clients from pruned data

## 4. Filtering Integration Summary

### Spam Filtering Flow

```
Block Received
    ↓
SpamFilter::filter_block()
    ↓
Filtered Transactions (non-spam only)
    ↓
UTXO Tree Update (process ALL transactions, filter OUTPUTS only)
    ↓
UTXO Commitment Generation
    ↓
FilteredBlockMessage (with filtered transactions + commitment)
```

### BIP158 Filter Flow

```
Block Received
    ↓
Extract Scripts (outputs + spent inputs)
    ↓
build_block_filter()
    ↓
CompactBlockFilter (GCS encoded)
    ↓
FilterHeader (chained with previous)
    ↓
BlockFilterService (cache + serve)
```

## 5. Configuration

### Spam Filter Configuration

```rust
// Default configuration
let filter = SpamFilter::new(); // All spam types enabled, 546 sat threshold

// Custom configuration
let config = SpamFilterConfig {
    filter_ordinals: true,
    filter_dust: true,
    filter_brc20: false, // Disable BRC-20 filtering
    dust_threshold: 1000, // Higher threshold
    min_output_value: 546,
};
let filter = SpamFilter::with_config(config);
```

### BIP158 Filter Configuration

- **P Parameter**: 19 (false positive rate: 2^-19 ≈ 1/524,288)
- **M Parameter**: 2^19 = 524,288
- **Checkpoint Interval**: 1000 blocks (per BIP157)

## 6. Performance Characteristics

### Spam Filtering
- **Bandwidth Savings**: 40-60% reduction
- **CPU Overhead**: Minimal (pattern matching)
- **Memory**: O(1) per transaction

### BIP158 Filters
- **Filter Size**: ~1-2 KB per block (varies with script count)
- **Generation Time**: <1ms per block
- **Matching Time**: O(log N) where N = number of scripts

## 7. Use Cases

### Spam Filtering
- **UTXO Commitment Sync**: Reduce bandwidth during initial sync
- **Ongoing Sync**: Skip spam transactions in filtered blocks
- **Bandwidth Optimization**: For nodes with limited bandwidth

### BIP158 Filters
- **Light Clients**: Determine if block contains relevant transactions
- **Wallet Scanning**: Efficiently scan blockchain for wallet addresses
- **Privacy**: Light clients don't need to reveal all addresses

## 8. Future Enhancements

### Potential Improvements
1. **Adaptive Spam Thresholds**: Adjust based on network conditions
2. **Machine Learning Spam Detection**: More sophisticated pattern recognition
3. **Filter Compression**: Further optimize BIP158 filter sizes
4. **Pruning Integration**: Full pruning with Commons verification

## Summary

**Commons (bllvm-consensus) provides**:
- ✅ **Spam Filtering**: Transaction-level filtering (Ordinals, dust, BRC-20)
- ✅ **BIP158 Filters**: Compact block filters for light clients
- ✅ **UTXO Commitments**: Foundation for efficient pruning

**Node provides**:
- ✅ **Filter Service**: BIP158 filter caching and serving
- ✅ **RPC Integration**: `getblockfilter` method
- ⚠️ **Pruning**: Placeholder implementation (needs completion)

**Key Insight**: Commons focuses on **filtering** (what to include/exclude), while **pruning** (removing old data) is a node-level storage optimization that uses Commons' filtering capabilities.

