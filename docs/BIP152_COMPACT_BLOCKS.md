# Compact Block Relay (BIP152) Implementation Plan

## Overview
BIP152 introduces Compact Blocks to reduce bandwidth during block propagation by sending only block headers and short transaction IDs, allowing peers to reconstruct blocks using transactions from their mempool.

## Specification
- **BIP**: 152
- **Title**: Compact Block Relay
- **Status**: Standard
- **Link**: https://github.com/bitcoin/bips/blob/master/bip-0152.mediawiki

## Key Components

### 1. Message Types

#### `sendcmpct` (BIP152)
- Signals support for compact blocks
- Parameters: version (uint64_t), prefer_cmpct (bool)

#### `cmpctblock`
- Contains: Header, nonce, short IDs (6 bytes each)
- Short IDs are SipHash-2-4 of the wtxid, or the txid when the transaction has no witness. `k0` and `k1` are the first two little-endian 64-bit integers of `SHA256(80-byte header || nonce)`.

#### `getblocktxn`
- Request missing transactions from a compact block
- Contains: block_hash, indices

#### `blocktxn`
- Response with missing transactions
- Contains: block_hash, transactions

### 2. Short Transaction ID

**Calculation**:
```
short_id = SipHash-2-4(k0, k1, wtxid) truncated to 48 bits
```

Where:
- `k0`, `k1`: the first two little-endian 64-bit integers of `SHA256(80-byte header || nonce)`
- `wtxid`: the witness transaction id, or the txid when there is no witness

### 3. Block Reconstruction

1. Receive `cmpctblock` message
2. Extract header and short IDs
3. Match short IDs with mempool transactions
4. Request missing transactions via `getblocktxn`
5. Reconstruct full block
6. Validate block

## Implementation

`blvm-node/src/network/compact_blocks.rs` is in the tree. `calculate_short_tx_id` uses the formula above.

### Dependencies

- `siphasher` crate for SipHash-2-4
- Update message serialization for new message types

### Integration Points

- **Mempool Integration**: Query mempool by short ID
- **Block Validation**: Validate reconstructed blocks
- **Network Protocol**: Add to P2P message handling

## Benefits

- **Bandwidth Reduction**: ~40% reduction in block propagation bandwidth
- **Latency**: Faster block propagation (especially for nodes with full mempool)
- **Network Efficiency**: Reduces redundant transaction retransmission

## Iroh Integration

Compact Block Relay (BIP152) is fully integrated with Iroh QUIC transport, providing enhanced benefits when used together:

### Combined Benefits

- **Lower Latency**: QUIC's multiplexing and stream prioritization complement compact blocks
- **NAT Traversal**: Iroh's magic endpoint allows compact blocks to work through NAT/firewalls
- **Encryption**: QUIC/TLS encryption by default protects compact block data
- **Mobile Optimization**: Ideal for mobile nodes with limited bandwidth and NAT constraints

### Transport-Aware Configuration

The implementation automatically optimizes compact block usage based on transport:

- **TCP**: Compact blocks optional (version 1)
- **Quinn QUIC**: Compact blocks preferred (version 2)
- **Iroh QUIC**: Compact blocks strongly preferred (version 2)

### Feature Independence

Both features are optional and work independently:

- ✅ Compact blocks work with TCP, Quinn, or Iroh
- ✅ Iroh works with or without compact blocks
- ✅ Optimal: Iroh + Compact Blocks for mobile/NAT scenarios

## Testing

- Unit tests for short ID calculation
- Integration tests for block reconstruction
- Performance tests for bandwidth reduction
- Compatibility tests with Bitcoin Core
- **Iroh integration tests**: Verify compact blocks work over Iroh QUIC
- **Transport-aware tests**: Verify negotiation based on transport type

## Risks

- Complexity: More complex block reconstruction logic
- Edge Cases: Handling missing transactions, invalid short IDs
- Compatibility: Must work with non-compact block nodes
- **Iroh-specific**: Requires Iroh feature flag for QUIC-specific optimizations

