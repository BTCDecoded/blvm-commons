# Missing RPC Methods vs Bitcoin Core

## Currently Implemented: 28 Methods

### Blockchain (8 methods)
- ✅ getblockchaininfo
- ✅ getblock
- ✅ getblockhash
- ✅ getblockheader
- ✅ getbestblockhash
- ✅ getblockcount
- ✅ getdifficulty
- ✅ gettxoutsetinfo
- ✅ verifychain

### Raw Transaction (7 methods)
- ✅ getrawtransaction
- ✅ sendrawtransaction
- ✅ testmempoolaccept
- ✅ decoderawtransaction
- ✅ gettxout
- ✅ gettxoutproof
- ✅ verifytxoutproof

### Mempool (3 methods)
- ✅ getmempoolinfo
- ✅ getrawmempool
- ✅ savemempool

### Network (9 methods)
- ✅ getnetworkinfo
- ✅ getpeerinfo
- ✅ getconnectioncount
- ✅ ping
- ✅ addnode
- ✅ disconnectnode
- ✅ getnettotals
- ✅ clearbanned
- ✅ setban
- ✅ listbanned

### Mining (4 methods)
- ✅ getmininginfo
- ✅ getblocktemplate
- ✅ submitblock
- ✅ estimatesmartfee

## Missing RPC Methods (Bitcoin Core has ~100+)

### Control/Utility (Missing)
- ❌ stop - Stop Bitcoin node
- ❌ uptime - Get uptime
- ❌ getmemoryinfo - Memory usage stats
- ❌ getrpcinfo - RPC server info
- ❌ help - List commands
- ❌ logging - Control logging

### Blockchain (Missing)
- ❌ getchaintips - Get chain tips
- ❌ getchaintxstats - Chain transaction statistics
- ❌ getblockstats - Block statistics
- ❌ pruneblockchain - Prune blockchain
- ❌ invalidateblock - Mark block as invalid
- ❌ reconsiderblock - Reconsider invalid block
- ❌ waitfornewblock - Wait for new block
- ❌ waitforblock - Wait for specific block
- ❌ waitforblockheight - Wait for block height

### Raw Transaction (Missing)
- ❌ createrawtransaction - Create raw transaction
- ❌ signrawtransactionwithkey - Sign transaction
- ❌ sendrawtransaction - Enhanced options
- ❌ analyzepsbt - Analyze PSBT
- ❌ combinepsbt - Combine PSBTs
- ❌ convertopsbt - Convert PSBT
- ❌ createpsbt - Create PSBT
- ❌ decodepsbt - Decode PSBT
- ❌ finalizepsbt - Finalize PSBT
- ❌ joinpsbts - Join PSBTs
- ❌ utxoupdatepsbt - Update PSBT with UTXO data

### Mempool (Missing)
- ❌ getmempoolancestors - Get mempool ancestors
- ❌ getmempooldescendants - Get mempool descendants
- ❌ getmempoolentry - Get specific mempool entry
- ❌ getmempoolentry - Get mempool entry details

### Network (Missing)
- ❌ getaddednodeinfo - Get added node info
- ❌ getnodeaddresses - Get node addresses
- ❌ setnetworkactive - Enable/disable networking

### Mining (Missing)
- ❌ getnetworkhashps - Network hashrate (we have this in getmininginfo)
- ❌ prioritisetransaction - Prioritize transaction
- ❌ getmininginfo - Enhanced (we have basic version)

### Utility (Missing)
- ❌ createmultisig - Create multisig address
- ❌ deriveaddresses - Derive addresses
- ❌ getdescriptorinfo - Get descriptor info
- ❌ validateaddress - Validate address
- ❌ verifymessage - Verify message signature
- ❌ signmessage - Sign message

### Indexing (Missing)
- ❌ getblockfilter - Get block filter (BIP158)
- ❌ getindexinfo - Get index info

### Signer (Missing - Wallet-related, but useful)
- ❌ enumeratesigners - Enumerate signers
- ❌ walletdisplayaddress - Display address

### Stats (Missing)
- ❌ gettxoutsetinfo - Enhanced stats
- ❌ getblockchaininfo - Enhanced with more fields

## Summary

**Implemented**: 28 methods
**Bitcoin Core Total**: ~100+ methods
**Missing**: ~70+ methods

### Categories of Missing Methods:
1. **Wallet-related** (by design - we exclude wallet)
   - Most PSBT methods
   - Address creation/validation
   - Signing methods

2. **Advanced Blockchain Queries**
   - Chain statistics
   - Block statistics
   - Chain tips

3. **Control/Utility**
   - Node control (stop, uptime)
   - Memory info
   - Logging control

4. **Advanced Mempool**
   - Ancestor/descendant queries
   - Entry details

5. **Network Advanced**
   - Node address queries
   - Network control

### Priority Missing Methods (Non-Wallet):
1. **getchaintips** - Useful for chain analysis
2. **getblockstats** - Block statistics
3. **getmempoolancestors/descendants** - Mempool analysis
4. **getmemoryinfo** - System monitoring
5. **stop** - Node control
6. **getindexinfo** - Index status
7. **pruneblockchain** - Storage management
8. **waitfornewblock/block/blockheight** - Block waiting

Most missing methods are either:
- Wallet-related (intentionally excluded)
- Advanced querying (nice-to-have)
- Control utilities (useful but not critical)

**Core functionality is covered** - the 28 methods we have cover all essential Bitcoin node operations.

