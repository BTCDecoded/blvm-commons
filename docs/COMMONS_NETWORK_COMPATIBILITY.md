# Commons Network Compatibility

## Design Principle

**Commons nodes are fully compatible with standard Bitcoin nodes, with additional capabilities when interoperating with other Commons nodes.**

## Compatibility Levels

### Level 1: Standard Bitcoin Compatibility ✅

**Commons nodes work with ANY Bitcoin node** (Bitcoin Core, btcd, etc.):

- ✅ Standard Bitcoin P2P protocol messages
- ✅ Block and transaction relay
- ✅ Header synchronization
- ✅ Mempool sharing
- ✅ Standard service flags (NODE_NETWORK, etc.)

**No special requirements** - Commons nodes can connect to and communicate with any Bitcoin node.

### Level 2: Enhanced Commons Features ✅

**Additional capabilities when BOTH peers are Commons nodes**:

When both peers advertise Commons-specific service flags, they can use:

1. **UTXO Commitments** (`NODE_UTXO_COMMITMENTS`)
   - Fast initial sync (98% bandwidth savings)
   - Peer consensus verification
   - Spam filtering

2. **Ban List Sharing** (`NODE_BAN_LIST_SHARING`)
   - Network-wide protection
   - Shared knowledge of malicious peers
   - Bootstrap ban lists

3. **Enhanced Features** (if both support):
   - Compact block filters (BIP157)
   - Package relay (BIP331)
   - FIBRE fast relay

## Service Flag Detection

### How It Works

```
1. Peer A (Commons) connects to Peer B
   ↓
2. Exchange version messages
   ↓
3. Check service flags:
   - If Peer B has NODE_UTXO_COMMITMENTS → Use UTXO commitments
   - If Peer B doesn't → Fall back to standard sync
   ↓
4. Use appropriate protocol based on capabilities
```

### Example Flow

**Scenario 1: Commons ↔ Bitcoin Core**
```
Commons Node          Bitcoin Core
     |                     |
     |--- version -------->|
     |<-- version ---------|
     |                     |
     | (No UTXO commitments flag)
     |                     |
     |--- Use standard --->|
     |    Bitcoin sync     |
```

**Scenario 2: Commons ↔ Commons**
```
Commons Node A      Commons Node B
     |                   |
     |--- version ------>|
     | (NODE_UTXO_COMMITMENTS)
     |                   |
     |<-- version -------|
     | (NODE_UTXO_COMMITMENTS)
     |                   |
     |--- Use UTXO ----->|
     |    commitments    |
     |    (fast sync)    |
```

## Implementation Details

### Service Flags

**Commons-specific flags**:
- `NODE_UTXO_COMMITMENTS` (bit 27) - UTXO commitment protocol
- `NODE_BAN_LIST_SHARING` (bit 28) - Ban list sharing

**Standard Bitcoin flags** (also supported):
- `NODE_COMPACT_FILTERS` (bit 6) - BIP157
- `NODE_PACKAGE_RELAY` (bit 25) - BIP331
- `NODE_FIBRE` (bit 26) - FIBRE

### Graceful Degradation

**Commons nodes gracefully degrade when talking to standard nodes**:

1. **UTXO Commitments**:
   - If peer doesn't support → Fall back to standard block sync
   - No error, just uses different protocol

2. **Ban List Sharing**:
   - If peer doesn't support → Don't request ban list
   - Continue with standard protocol

3. **All Features**:
   - Check service flags before using
   - Use standard Bitcoin protocol as fallback
   - Never break compatibility

## Code Examples

### Checking Peer Capabilities

```rust
// When receiving version message
let version = receive_version_message().await?;

// Check if peer supports Commons features
if version.supports_utxo_commitments() {
    // Use fast UTXO commitments sync
    sync_with_utxo_commitments(peer).await?;
} else {
    // Fall back to standard Bitcoin sync
    sync_standard_blocks(peer).await?;
} else {
    // Fall back to standard Bitcoin sync
    sync_standard_blocks(peer).await?;
}
```

### Peer Discovery

```rust
// Discover peers that support UTXO commitments
let commons_peers = peer_consensus.discover_diverse_peers_with_capability(
    all_peers,
    Some(true) // Require UTXO commitments
);

// Use Commons features with Commons peers
for peer in commons_peers {
    // Fast sync with UTXO commitments
    sync_with_commitments(peer).await?;
}

// Use standard sync with other peers
for peer in standard_peers {
    // Standard Bitcoin sync
    sync_standard(peer).await?;
}
```

## Benefits

### 1. Network Compatibility ✅

- **Works with entire Bitcoin network** - No isolation
- **No breaking changes** - Standard nodes work normally
- **Gradual adoption** - Commons features optional

### 2. Enhanced Features ✅

- **Fast sync** - 98% bandwidth savings with Commons peers
- **Network protection** - Shared ban lists
- **Better performance** - When both peers support features

### 3. Backward Compatibility ✅

- **Always works** - Falls back to standard protocol
- **No errors** - Graceful degradation
- **Future-proof** - Can add more features

## Migration Path

### For Node Operators

1. **Deploy Commons node** - Works with existing network
2. **Connect to standard nodes** - Uses standard protocol
3. **Connect to Commons nodes** - Automatically uses enhanced features
4. **No configuration needed** - Service flags handle it

### For Network Growth

1. **Early adopters** - Use Commons features with each other
2. **Gradual adoption** - More nodes add Commons support
3. **Network benefits** - Better performance as adoption grows
4. **No disruption** - Standard nodes continue working

## Testing

### Compatibility Tests

```rust
// Test: Commons node with standard Bitcoin node
#[test]
fn test_commons_with_bitcoin_core() {
    // Should use standard protocol
    // Should not use UTXO commitments
    // Should work normally
}

// Test: Commons node with Commons node
#[test]
fn test_commons_with_commons() {
    // Should use UTXO commitments
    // Should use ban list sharing
    // Should use enhanced features
}
```

## Conclusion

**Commons nodes are fully compatible with the Bitcoin network** and automatically use enhanced features when both peers support them. This provides:

- ✅ **Maximum compatibility** - Works with all Bitcoin nodes
- ✅ **Enhanced features** - When both peers are Commons
- ✅ **Graceful degradation** - Falls back to standard protocol
- ✅ **No breaking changes** - Network remains compatible

The service flags implementation makes this seamless - nodes automatically detect capabilities and use the best available protocol.







