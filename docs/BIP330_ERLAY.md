# Erlay Transaction Relay (BIP330) Implementation Plan

## Overview
BIP330 (Erlay) reduces transaction relay bandwidth by ~40% using set reconciliation (minisketch library) to efficiently transmit transaction sets between peers.

## Specification
- **BIP**: 330
- **Title**: Erlay: Bandwidth-Efficient Transaction Relay Protocol
- **Status**: Draft
- **Link**: https://github.com/bitcoin/bips/blob/master/bip-0330.mediawiki

## Key Components

### 1. Set Reconciliation

**Problem**: Two peers have transaction sets A and B. Need to find A - B (transactions peer B is missing).

**Solution**: Use minisketch to compute a "sketch" of set differences:
- Sender computes sketch of A - B
- Receiver computes sketch of B - A  
- Combine sketches to find symmetric difference
- Request missing transactions

### 2. Message Types

#### `sendtxrcncl` (BIP330)
- Signals support for Erlay
- Parameters: version, local_set_size, remote_set_size

#### `reqrecon`
- Request reconciliation with a peer
- Contains reconciliation parameters

#### `sketch`
- Sends minisketch sketch of missing transactions
- Size depends on set difference

#### `reqskt`
- Request sketch from peer
- Used for reconciliation

### 3. minisketch Integration

**minisketch** is a library for set reconciliation:
- Computes compact sketches of set differences
- Enables efficient transaction set synchronization
- Bandwidth scales with set difference, not set size

## Implementation Plan

### Files to Create

1. **`bllvm-node/src/network/erlay.rs`**
   - Set reconciliation logic
   - Sketch computation and decoding
   - Transaction set management

2. **Integration**:
   - Add to `NetworkManager`
   - Update mempool synchronization
   - Protocol message handling

### Dependencies

- **minisketch-rs** (Rust bindings for minisketch C library)
  - Or: Pure Rust implementation if available
  - Or: FFI bindings to libminisketch

**Note**: minisketch is primarily C. Options:
1. Use existing Rust bindings (if available)
2. Create FFI bindings
3. Pure Rust port (if needed)

### Integration Points

- **Mempool**: Track transaction sets for reconciliation
- **Network**: Add Erlay messages to protocol
- **Bandwidth**: Monitor bandwidth savings

## Benefits

- **Bandwidth Reduction**: ~40% reduction in transaction relay bandwidth
- **Scalability**: Better performance for high-throughput nodes
- **Network Efficiency**: Reduces redundant transaction transmission

## Implementation Complexity

- **High**: Set reconciliation is complex
- **Dependencies**: minisketch integration (may require FFI)
- **Testing**: Need to verify correctness of reconciliation

## Testing

- Unit tests for set reconciliation
- Integration tests for transaction relay
- Performance tests for bandwidth reduction
- Compatibility tests with standard transaction relay

## Risks

- **Complexity**: High implementation complexity
- **Dependencies**: minisketch may require FFI (against pure Rust goal)
- **Compatibility**: Must maintain backward compatibility with non-Erlay peers
- **Correctness**: Set reconciliation must be 100% accurate

## Alternatives Considered

If minisketch integration proves too complex:
- Focus on BIP152 (Compact Blocks) first
- Consider Erlay as Phase 4 feature
- Explore pure Rust set reconciliation libraries

