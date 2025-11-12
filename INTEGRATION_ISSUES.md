# Integration Issues and TODOs

## Critical Integration Issues

### 1. MiningCoordinator Uses Mock Instead of Real MempoolManager ⚠️

**Location**: `bllvm-node/src/node/miner.rs`

**Issue**: `MiningCoordinator` uses `MockMempoolProvider` instead of the real `MempoolManager`

```rust
pub struct MiningCoordinator {
    mempool_provider: MockMempoolProvider,  // ❌ Should use real MempoolManager
}
```

**Impact**: Mining coordinator doesn't use actual mempool state

**Fix Required**:
- Make `MempoolManager` implement `MempoolProvider` trait
- Update `MempoolProvider` trait to accept UTXO set for fee calculation
- Update `MiningCoordinator` to use real `MempoolManager`

### 2. Duplicate Fee Calculation Methods ✅ PARTIALLY FIXED

**Locations**:
- `bllvm-node/src/node/mempool.rs` - `calculate_transaction_fee()` (uses UTXO set) ✅
- `bllvm-node/src/rpc/mining.rs` - `calculate_transaction_fee()` (now uses UTXO set) ✅
- `bllvm-node/src/node/miner.rs` - `calculate_fee_rate()` (simplified mock) ⚠️ Still needs update

**Status**: 
- ✅ `MiningRpc::calculate_transaction_fee()` now uses UTXO set (matches MempoolManager logic)
- ⚠️ `MiningCoordinator::calculate_fee_rate()` still uses simplified mock calculation

**Remaining**: Update `MiningCoordinator` to use real fee calculation from `MempoolManager`

### 3. MempoolProvider Trait Missing UTXO Set Parameter ✅ FIXED

**Location**: `bllvm-node/src/node/miner.rs`

**Status**: ✅ Fixed - Trait now accepts UTXO set parameter

**Changes Made**:
- Updated trait signature: `get_prioritized_transactions(&self, limit: usize, utxo_set: &UtxoSet)`
- Updated `MempoolManager` implementation to use UTXO set
- Updated `MockMempoolProvider` to match new signature
- Updated `TransactionSelector::select_transactions()` to pass UTXO set

### 4. MiningRpc Fee Estimation Doesn't Use MempoolManager Methods ⚠️

**Location**: `bllvm-node/src/rpc/mining.rs:437`

**Issue**: `estimate_smart_fee()` uses simplified `calculate_transaction_fee()` instead of `MempoolManager` methods

**Fix Required**:
- Use `MempoolManager::get_prioritized_transactions()` which already calculates fees correctly
- Remove duplicate fee calculation logic

## TODOs and Placeholders

### High Priority TODOs

1. **RPC Methods Not Integrated** (`bllvm-node/src/rpc/`)
   - `getnetworkinfo()` - TODO: Query actual network state
   - `getpeerinfo()` - TODO: Query actual peer list
   - `ping()` - TODO: Send ping messages
   - `addnode()` - TODO: Add to persistent peer list
   - `disconnectnode()` - TODO: Disconnect peer
   - `getnettotals()` - TODO: Query network statistics
   - `listbanned()` - TODO: Query ban list
   - `setban()` - TODO: Parse subnet and ban/unban

2. **Mempool RPC Not Integrated** (`bllvm-node/src/rpc/mempool.rs`)
   - `getmempoolinfo()` - TODO: Query actual mempool state
   - `getrawmempool()` - TODO: Query actual mempool
   - `savemempool()` - TODO: Persist mempool to disk

3. **Blockchain RPC Not Integrated** (`bllvm-node/src/rpc/blockchain.rs`)
   - `getblockchaininfo()` - TODO: Query actual blockchain state
   - `getblockheader()` - TODO: Query actual block header
   - `getbestblockhash()` - TODO: Query actual best block hash
   - `getblockcount()` - TODO: Query actual block count
   - `getdifficulty()` - TODO: Query actual difficulty
   - `gettxoutsetinfo()` - TODO: Query actual UTXO set statistics
   - `verifychain()` - TODO: Implement blockchain verification

4. **Raw Transaction RPC Not Integrated** (`bllvm-node/src/rpc/rawtx.rs`)
   - `sendrawtransaction()` - TODO: Parse, validate, add to mempool
   - `testmempoolaccept()` - TODO: Parse and validate transaction
   - `decoderawtransaction()` - TODO: Parse transaction
   - `getrawtransaction()` - TODO: Look up transaction
   - `gettxout()` - TODO: Look up UTXO
   - `gettxoutproof()` - TODO: Build merkle proof
   - `verifytxoutproof()` - TODO: Verify merkle proof

5. **Network Message Processing** (`bllvm-node/src/network/mod.rs:699`)
   - TODO: Actually send via transport layer (currently just logs)

6. **BIP70 Handler** (`bllvm-node/src/network/bip70_handler.rs`)
   - `GetPaymentRequest` - TODO: Look up payment request
   - `Payment` - TODO: Look up original request, validate, process

7. **BIP157 Handler** (`bllvm-node/src/network/bip157_handler.rs`)
   - TODO: Query block index to get hashes in range

### Medium Priority TODOs

8. **Module System** (`bllvm-node/src/module/`)
   - `node_api.rs:155` - TODO: Integrate with actual event system
   - `security/validator.rs:85` - TODO: Implement rate limiting per module
   - `sandbox/process.rs:88` - TODO: Implement OS-specific sandboxing

9. **Stratum V2** (`bllvm-node/src/network/stratum_v2/`)
   - `server.rs:216` - TODO: Call coordinator.generate_block_template()
   - `pool.rs:463` - TODO: Properly extract merkle path
   - `pool.rs:471` - TODO: Implement proper transaction serialization

10. **Package Relay** (`bllvm-node/src/network/package_relay.rs:162`)
    - TODO: Calculate from UTXO set

11. **UTXO Commitments** (`bllvm-node/src/network/utxo_commitments_client.rs`)
    - TODO: In full implementation, would handle async message routing

12. **Protocol Extensions** (`bllvm-node/src/network/protocol_extensions.rs`)
    - TODO: Integrate with actual UTXO commitment module
    - TODO: Integrate with actual spam filter and block store

### Low Priority / Placeholders

13. **BIP70** (`bllvm-node/src/bip70.rs`)
    - Placeholder transaction in tests

14. **ERLAY** (`bllvm-node/src/network/erlay.rs`)
    - Placeholder implementation (requires minisketch)

15. **Iroh Transport** (`bllvm-node/src/network/iroh_transport.rs`)
    - Placeholder peer_node_id until protocol exchange

16. **Quinn Transport** (`bllvm-node/src/network/quinn_transport.rs`)
    - Placeholder when feature disabled

## Code Duplication Issues

### 1. Fee Calculation (3 implementations)
- `MempoolManager::calculate_transaction_fee()` - ✅ Correct (uses UTXO)
- `MiningRpc::calculate_transaction_fee()` - ❌ Simplified
- `MiningCoordinator::calculate_fee_rate()` - ❌ Mock

**Recommendation**: Use only `MempoolManager` method

### 2. Transaction Size Estimation (2 implementations)
- `MempoolManager::estimate_transaction_size()` - ✅
- `MiningCoordinator::calculate_transaction_size()` - ❌ Duplicate

**Recommendation**: Consolidate into `MempoolManager`

### 3. Prioritized Transactions
- `MempoolManager::get_prioritized_transactions()` - ✅ Real implementation
- `MockMempoolProvider::get_prioritized_transactions()` - ❌ Mock

**Recommendation**: Make `MempoolManager` implement `MempoolProvider`

## Integration Recommendations

### Priority 1: Fix Mining Integration
1. Make `MempoolManager` implement `MempoolProvider` trait
2. Update trait to accept UTXO set parameter
3. Update `MiningCoordinator` to use real `MempoolManager`
4. Remove duplicate fee calculation from `MiningRpc`

### Priority 2: Integrate RPC Methods
1. Connect RPC methods to actual storage/network/mempool
2. Remove placeholder responses
3. Add proper error handling

### Priority 3: Complete Network Handlers
1. Implement BIP70 payment processing
2. Implement BIP157 block filter queries
3. Complete transport layer message sending

