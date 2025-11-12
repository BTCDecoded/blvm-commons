# Service Flags Implementation

## Summary

Implemented service flags for peer capability negotiation, following Bitcoin protocol patterns (similar to BIP157's `NODE_COMPACT_FILTERS`).

## Service Flags Added

### 1. NODE_UTXO_COMMITMENTS (Bit 27)

**Purpose**: Indicates support for UTXO commitment protocol extensions:
- `GetUTXOSet` / `UTXOSet` messages
- `GetFilteredBlock` / `FilteredBlock` messages

**Location**: `bllvm-node/src/network/protocol.rs`
```rust
#[cfg(feature = "utxo-commitments")]
pub const NODE_UTXO_COMMITMENTS: u64 = 1 << 27;
```

**When Set**: Automatically set if `utxo-commitments` feature is enabled

**Usage**:
```rust
if version.supports_utxo_commitments() {
    // Peer supports UTXO commitments, can send GetUTXOSet
}
```

### 2. NODE_BAN_LIST_SHARING (Bit 28)

**Purpose**: Indicates support for ban list sharing protocol:
- `GetBanList` / `BanList` messages

**Location**: `bllvm-node/src/network/protocol.rs`
```rust
pub const NODE_BAN_LIST_SHARING: u64 = 1 << 28;
```

**When Set**: Set if `ban_list_sharing_config` is configured

**Usage**:
```rust
if version.supports_ban_list_sharing() {
    // Peer supports ban list sharing, can request ban list
}
```

## Existing Service Flags

### Already Implemented:
- **NODE_COMPACT_FILTERS** (BIP157) - Bit 6 (from `bllvm_protocol::bip157`)
- **NODE_DANDELION** - Bit 24 (if `dandelion` feature enabled)
- **NODE_PACKAGE_RELAY** - Bit 25 (BIP331, always enabled)
- **NODE_FIBRE** - Bit 26 (always enabled)

## Implementation Details

### 1. Service Flag Constants

**File**: `bllvm-node/src/network/protocol.rs`

```rust
/// Service flags (bitfield in Version.services)
#[cfg(feature = "dandelion")]
pub const NODE_DANDELION: u64 = 1 << 24;
pub const NODE_PACKAGE_RELAY: u64 = 1 << 25;
pub const NODE_FIBRE: u64 = 1 << 26;
/// UTXO Commitments support (GetUTXOSet, UTXOSet, GetFilteredBlock, FilteredBlock)
#[cfg(feature = "utxo-commitments")]
pub const NODE_UTXO_COMMITMENTS: u64 = 1 << 27;
/// Ban List Sharing support (GetBanList, BanList)
pub const NODE_BAN_LIST_SHARING: u64 = 1 << 28;
```

### 2. Version Message Creation

**File**: `bllvm-node/src/network/mod.rs`

Updated `create_version_message()` to set flags based on:
- Feature flags (UTXO commitments, Dandelion)
- Configuration (ban list sharing)
- Always-on features (Package Relay, FIBRE, Compact Filters)

```rust
pub fn create_version_message(...) -> VersionMessage {
    let mut services_with_filters = services;
    
    // BIP157 Compact Block Filters (always enabled)
    services_with_filters |= NODE_COMPACT_FILTERS;
    
    // UTXO Commitments (if feature enabled)
    #[cfg(feature = "utxo-commitments")]
    {
        services_with_filters |= NODE_UTXO_COMMITMENTS;
    }
    
    // Ban List Sharing (if config enabled)
    if self.ban_list_sharing_config.is_some() {
        services_with_filters |= NODE_BAN_LIST_SHARING;
    }
    
    // ... other flags ...
}
```

### 3. Helper Methods

**File**: `bllvm-node/src/network/protocol.rs`

Added convenience methods to `VersionMessage`:

```rust
impl VersionMessage {
    #[cfg(feature = "utxo-commitments")]
    pub fn supports_utxo_commitments(&self) -> bool { ... }
    
    pub fn supports_ban_list_sharing(&self) -> bool { ... }
    
    pub fn supports_compact_filters(&self) -> bool { ... }
    
    pub fn supports_package_relay(&self) -> bool { ... }
    
    pub fn supports_fibre(&self) -> bool { ... }
    
    #[cfg(feature = "dandelion")]
    pub fn supports_dandelion(&self) -> bool { ... }
}
```

## Usage Examples

### Check Peer Capabilities

```rust
// When receiving version message from peer
let version = receive_version_message().await?;

// Check if peer supports UTXO commitments
if version.supports_utxo_commitments() {
    // Can send GetUTXOSet requests
    let commitment = request_utxo_set(peer_id, height, hash).await?;
}

// Check if peer supports ban list sharing
if version.supports_ban_list_sharing() {
    // Can request ban list
    let ban_list = request_ban_list(peer_id).await?;
}
```

### Filter Peers by Capability

```rust
// Filter peers that support UTXO commitments
let utxo_peers: Vec<_> = all_peers
    .iter()
    .filter(|peer| {
        peer.version
            .as_ref()
            .map(|v| v.supports_utxo_commitments())
            .unwrap_or(false)
    })
    .collect();
```

### Update Peer Discovery

**TODO**: Update `bllvm-consensus/src/utxo_commitments/peer_consensus.rs::discover_diverse_peers()` to:
1. Accept peer versions as parameter
2. Filter peers by `NODE_UTXO_COMMITMENTS` flag
3. Only query peers that support the feature

## Benefits

1. **Explicit Capability Negotiation**: No more "try and see" approach
2. **Efficient Peer Discovery**: Filter peers before sending requests
3. **Reduced Network Traffic**: Don't send requests to unsupporting peers
4. **Better Error Handling**: Know capabilities upfront
5. **Follows Bitcoin Patterns**: Similar to BIP157's approach

## Next Steps

1. ✅ **Add service flags** - DONE
2. ✅ **Update version message creation** - DONE
3. ✅ **Add helper methods** - DONE
4. ⏳ **Update peer discovery** - Filter by capability
5. ⏳ **Update UTXO commitments client** - Check flag before requests
6. ⏳ **Update ban list sharing** - Check flag before requests
7. ⏳ **Add tests** - Verify flag behavior

## Service Flag Bit Allocation

| Bit | Flag | Feature | Status |
|-----|------|---------|--------|
| 6 | NODE_COMPACT_FILTERS | BIP157 | ✅ Existing |
| 24 | NODE_DANDELION | Dandelion++ | ✅ Existing |
| 25 | NODE_PACKAGE_RELAY | BIP331 | ✅ Existing |
| 26 | NODE_FIBRE | FIBRE | ✅ Existing |
| 27 | NODE_UTXO_COMMITMENTS | UTXO Commitments | ✅ **NEW** |
| 28 | NODE_BAN_LIST_SHARING | Ban List Sharing | ✅ **NEW** |

## References

- Bitcoin Core service flags: https://en.bitcoin.it/wiki/Protocol_documentation#version
- BIP157: https://github.com/bitcoin/bips/blob/master/bip-0157.mediawiki
- BIP331: https://github.com/bitcoin/bips/blob/master/bip-0331.mediawiki

