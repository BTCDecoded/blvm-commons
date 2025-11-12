# UTXO Commitments Peer Discovery

## Current State

**Problem**: The node doesn't explicitly know which peers support UTXO commitments before trying to use them.

### Current Approach: "Try and See"

The current implementation uses a **"try and see"** approach:

1. **No Service Flag**: Unlike BIP157 (which uses `NODE_COMPACT_FILTERS`), UTXO commitments don't have a service flag in the version message
2. **Direct Requests**: The node sends `GetUTXOSet` messages to peers
3. **Error Handling**: If a peer doesn't support it:
   - They may send a `reject` message
   - They may not respond
   - They may respond with an error

### Code Locations

**Request Flow**:
- `bllvm-consensus/src/utxo_commitments/peer_consensus.rs::request_utxo_sets()` - Sends requests to peers
- `bllvm-node/src/network/utxo_commitments_client.rs::request_utxo_set()` - Network client implementation
- `bllvm-node/src/network/protocol_extensions.rs::handle_get_utxo_set()` - Handles incoming requests

**Current Behavior**:
```rust
// bllvm-consensus/src/utxo_commitments/peer_consensus.rs
pub async fn request_utxo_sets(
    &self,
    peers: &[PeerInfo],
    checkpoint_height: Natural,
    checkpoint_hash: Hash,
) -> Vec<PeerCommitment> {
    // Sends GetUTXOSet to all peers
    // Collects responses (or errors)
    // Returns valid commitments
}
```

## Proposed Solution: Service Flag

### Option 1: Add Service Flag (Recommended)

Add a service flag similar to `NODE_COMPACT_FILTERS`:

```rust
// bllvm-node/src/network/protocol.rs
pub const NODE_UTXO_COMMITMENTS: u64 = 1 << 27; // Next available bit

// In create_version_message()
#[cfg(feature = "utxo-commitments")]
{
    if utxo_commitments_enabled {
        services_with_filters |= NODE_UTXO_COMMITMENTS;
    }
}
```

**Benefits**:
- ✅ Explicit capability negotiation
- ✅ Avoids unnecessary requests
- ✅ Follows Bitcoin protocol patterns (like BIP157)
- ✅ Easy to check: `(peer_services & NODE_UTXO_COMMITMENTS) != 0`

**Implementation Steps**:
1. Add `NODE_UTXO_COMMITMENTS` constant
2. Set flag in `create_version_message()` when UTXO commitments enabled
3. Check flag before sending `GetUTXOSet` requests
4. Filter peers by capability in `discover_diverse_peers()`

### Option 2: User-Agent Detection

Check `user_agent` in version message for known implementations:

```rust
fn supports_utxo_commitments(version: &VersionMessage) -> bool {
    let ua = version.user_agent.to_lowercase();
    ua.contains("bllvm") || ua.contains("btcdecoded")
}
```

**Limitations**:
- ❌ Only works for known implementations
- ❌ Doesn't work for custom implementations
- ❌ Requires maintenance as new implementations appear

### Option 3: Hybrid Approach (Current + Service Flag)

1. **Check service flag first** (if available)
2. **Try request anyway** (for backward compatibility)
3. **Cache results** (remember which peers support it)

```rust
fn peer_supports_utxo_commitments(peer: &PeerInfo, version: &VersionMessage) -> bool {
    // Check service flag
    if (version.services & NODE_UTXO_COMMITMENTS) != 0 {
        return true;
    }
    
    // Check user agent (fallback)
    if version.user_agent.to_lowercase().contains("bllvm") {
        return true;
    }
    
    // Unknown - will try and see
    false
}
```

## Recommended Implementation

### Step 1: Add Service Flag

```rust
// bllvm-node/src/network/protocol.rs
pub const NODE_UTXO_COMMITMENTS: u64 = 1 << 27;

// bllvm-node/src/network/mod.rs
pub fn create_version_message(...) -> VersionMessage {
    let mut services_with_filters = services;
    services_with_filters |= NODE_COMPACT_FILTERS;
    
    #[cfg(feature = "utxo-commitments")]
    {
        // Check if UTXO commitments are enabled
        if self.utxo_commitments_enabled() {
            services_with_filters |= crate::network::protocol::NODE_UTXO_COMMITMENTS;
        }
    }
    
    // ... rest of version message
}
```

### Step 2: Check Flag Before Requests

```rust
// bllvm-consensus/src/utxo_commitments/peer_consensus.rs
pub fn discover_diverse_peers_with_capabilities(
    &self,
    all_peers: Vec<PeerInfo>,
    peer_versions: &HashMap<IpAddr, VersionMessage>, // Need to store versions
) -> Vec<PeerInfo> {
    use bllvm_node::network::protocol::NODE_UTXO_COMMITMENTS;
    
    let mut diverse_peers = Vec::new();
    
    for peer in all_peers {
        // Check if peer supports UTXO commitments
        if let Some(version) = peer_versions.get(&peer.address) {
            if (version.services & NODE_UTXO_COMMITMENTS) == 0 {
                continue; // Skip peers that don't support it
            }
        }
        
        // ... diversity checks ...
        diverse_peers.push(peer);
    }
    
    diverse_peers
}
```

### Step 3: Store Peer Versions

Need to store `VersionMessage` for each peer:

```rust
// bllvm-node/src/network/peer.rs
pub struct Peer {
    // ... existing fields ...
    pub version: Option<VersionMessage>, // Store version message
}

// When version message received:
peer.version = Some(version_message);
```

## Current Workaround

Until service flag is implemented, the system works by:

1. **Sending requests to all diverse peers**
2. **Handling errors gracefully**:
   ```rust
   // bllvm-node/src/network/utxo_commitments_client.rs
   match request_utxo_set(...).await {
       Ok(commitment) => { /* use commitment */ }
       Err(e) => {
           warn!("Peer {} doesn't support UTXO commitments: {}", peer_id, e);
           // Continue with other peers
       }
   }
   ```
3. **Collecting successful responses** and finding consensus

## Impact on Incremental Pruning

For **Case A (with peers)**:
- Node needs to find peers that support UTXO commitments
- Currently: Tries all peers, filters by successful responses
- With service flag: Filters by capability before requests

For **Case B (without peers)**:
- Not affected (doesn't need peer support)

## Next Steps

1. ✅ **Document current state** (this document)
2. ⏳ **Add service flag** (`NODE_UTXO_COMMITMENTS`)
3. ⏳ **Update version message creation** to set flag
4. ⏳ **Update peer discovery** to check flag
5. ⏳ **Store peer versions** for capability checks
6. ⏳ **Update tests** to verify flag behavior

## References

- BIP157 uses `NODE_COMPACT_FILTERS` service flag (bit 6)
- Bitcoin Core service flags: https://en.bitcoin.it/wiki/Protocol_documentation#version
- Current implementation: `bllvm-node/src/network/protocol.rs::create_version_message()`

