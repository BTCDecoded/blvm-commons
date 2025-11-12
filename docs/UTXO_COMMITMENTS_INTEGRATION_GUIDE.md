# UTXO Commitments Integration Guide

## Transport Support: TCP and Iroh ✅

**UTXO commitments work with both TCP and Iroh transports!** The `UtxoCommitmentsNetworkClient` trait is transport-agnostic and automatically uses the appropriate transport based on peer connection type.

See `docs/UTXO_COMMITMENTS_IROH_INTEGRATION.md` for detailed Iroh integration information.

## Overview

This guide explains how to integrate the UTXO Commitments module with the bllvm-node's network layer.

## Configuration

### Creating Configuration

The module supports JSON-based configuration:

```rust
use consensus_proof::utxo_commitments::UtxoCommitmentsConfig;

// Create default config
let config = UtxoCommitmentsConfig::default();

// Save to file
config.to_json_file(std::path::Path::new("utxo_commitments.json"))?;

// Load from file
let loaded_config = UtxoCommitmentsConfig::from_json_file(
    std::path::Path::new("utxo_commitments.json")
)?;

// Validate
loaded_config.validate()?;
```

### Configuration Options

See `examples/utxo_commitments_config_example.json` for a complete example.

**Sync Modes:**
- `PeerConsensus`: Fast initial sync using peer consensus (recommended)
- `Genesis`: Full sync from genesis (no trust required)
- `Hybrid`: Peer consensus with background genesis verification

**Verification Levels:**
- `Minimal`: Peer consensus only
- `Standard`: Peer consensus + PoW + supply checks (recommended)
- `Paranoid`: All checks + background genesis verification

**Consensus Settings:**
- `min_peers`: Minimum diverse peers required (default: 5)
- `target_peers`: Target number of peers to query (default: 10)
- `consensus_threshold`: Agreement percentage required (default: 0.8 = 80%)
- `max_peers_per_asn`: Maximum peers per ASN (default: 2)
- `safety_margin`: Blocks back from tip for checkpoint (default: 2016)

**Spam Filter Settings:**
- `filter_ordinals`: Filter Ordinals/Inscriptions (default: true)
- `filter_dust`: Filter dust outputs (default: true)
- `filter_brc20`: Filter BRC-20 tokens (default: true)
- `dust_threshold`: Dust threshold in satoshis (default: 546)
- `min_output_value`: Minimum output value to include (default: 546)

## Network Integration

### Implementing UtxoCommitmentsNetworkClient

In `bllvm-node`, implement the trait:

```rust
use consensus_proof::utxo_commitments::UtxoCommitmentsNetworkClient;

impl UtxoCommitmentsNetworkClient for NetworkManager {
    async fn request_utxo_set(
        &self,
        peer_id: &str,
        height: u64,
        block_hash: [u8; 32],
    ) -> UtxoCommitmentResult<UtxoCommitment> {
        // Send GetUTXOSet message via protocol layer
        // Wait for UTXOSet response
        // Parse and return commitment
    }
    
    async fn request_filtered_block(
        &self,
        peer_id: &str,
        block_hash: [u8; 32],
    ) -> UtxoCommitmentResult<FilteredBlock> {
        // Send GetFilteredBlock message
        // Wait for FilteredBlock response
        // Parse and return filtered block
    }
    
    fn get_peer_ids(&self) -> Vec<String> {
        // Return list of connected peer IDs
    }
}
```

### Handling Protocol Messages

Wire up message handlers in `bllvm-node/src/network/protocol_extensions.rs`:

```rust
use crate::network::protocol::{GetUTXOSetMessage, UTXOSetMessage};

pub async fn handle_get_utxo_set(
    message: GetUTXOSetMessage,
    utxo_tree: &UtxoMerkleTree,
) -> Result<UTXOSetMessage> {
    // Load UTXO set at requested height
    // Generate commitment from Merkle tree
    // Return UTXOSet message
}
```

## Usage Example

### Initial Sync

```rust
use consensus_proof::utxo_commitments::*;

// Load configuration
let config = UtxoCommitmentsConfig::from_json_file(config_path)?;
config.validate()?;

// Create initial sync manager
let consensus_config = config.to_consensus_config();
let spam_filter_config = config.to_spam_filter_config();
let initial_sync = InitialSync::with_spam_filter(consensus_config, spam_filter_config);

// Discover peers (from network manager)
let all_peers = network_manager.discover_peers();
let diverse_peers = peer_consensus.discover_diverse_peers(all_peers);

// Execute initial sync
let header_chain = fetch_header_chain()?;
let commitment = initial_sync.execute_initial_sync(diverse_peers, &header_chain).await?;
```

### Ongoing Sync with Filtered Blocks

```rust
// Create UTXO Merkle tree
let mut utxo_tree = UtxoMerkleTree::new()?;

// For each new block
let filtered_block = network_client.request_filtered_block(peer_id, block_hash).await?;

// Process filtered block
let (spam_summary, root) = initial_sync.process_filtered_block(
    &mut utxo_tree,
    block_height,
    &filtered_block.transactions,
)?;

// Verify commitment
process_and_verify_filtered_block(&filtered_block, block_height, &spam_filter)?;
```

## Testing

### Running Integration Tests

```bash
# Run all UTXO commitments tests
cargo test --features utxo-commitments

# Run integration tests only
cargo test --features utxo-commitments --test utxo_commitments_integration

# Run specific test
cargo test --features utxo-commitments test_utxo_commitment_full_workflow
```

## Performance Considerations

- **Merkle Tree Operations**: O(log n) per insert/remove
- **Spam Filtering**: O(n) where n = number of transactions per block
- **Peer Consensus**: O(m) where m = number of peers (typically < 20)
- **Initial Sync**: ~13GB download vs 600GB (98% savings)

## Security Considerations

- Peer consensus requires at least 2 of 10 diverse peers to be honest
- PoW verification ensures block header chain integrity
- Supply verification prevents inflation attacks
- Forward consistency checks ensure commitments remain valid

## Troubleshooting

### Configuration Validation Errors

- Check that `target_peers >= min_peers`
- Ensure `consensus_threshold` is between 0.0 and 1.0
- Verify all numeric values are non-negative

### Network Integration Issues

- Ensure protocol messages are properly serialized
- Check that peer IDs match between discovery and requests
- Verify message handlers return correct types

### Performance Issues

- Consider adjusting spam filter thresholds if too many transactions filtered
- Increase `target_peers` if consensus finding fails frequently
- Adjust `safety_margin` based on network stability

