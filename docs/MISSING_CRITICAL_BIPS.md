# Missing Critical BIPs - Implementation Plan

**Date**: 2025-01-XX  
**Status**: Implementation Required

## Summary

After comprehensive analysis, the following critical consensus BIPs are **missing or incomplete** in BTCDecoded:

1. **BIP30** - Duplicate Coinbase Prevention ❌ **NOT IMPLEMENTED**
2. **BIP34** - Block Height in Coinbase ❌ **NOT IMPLEMENTED**
3. **BIP66** - Strict DER Signatures ⚠️ **PARTIALLY IMPLEMENTED**
4. **BIP90** - Block Version Enforcement ❌ **NOT IMPLEMENTED**
5. **BIP147** - NULLDUMMY Enforcement ⚠️ **PARTIALLY IMPLEMENTED**

## Detailed Analysis

### 1. BIP30 - Duplicate Coinbase Prevention

**Status**: ❌ **NOT IMPLEMENTED**

**Requirement**: Prevents duplicate coinbase transactions (same txid) from being added to the blockchain.

**Activation**: Block 0 (always active)

**Implementation Location**: `bllvm-consensus/src/block.rs` in `connect_block()`

**Required Changes**:
- Track coinbase txids in a set (or check against UTXO set)
- Reject blocks with coinbase txids that match previous coinbase txids
- Only applies to coinbase transactions (not regular transactions)

**Reference**: Bitcoin Core checks this during block validation by ensuring coinbase txids are unique.

---

### 2. BIP34 - Block Height in Coinbase

**Status**: ❌ **NOT IMPLEMENTED**

**Requirement**: Starting at block 227,836 (mainnet), coinbase scriptSig must contain the block height encoded as a script number.

**Activation Heights**:
- Mainnet: Block 227,836
- Testnet: Block 211,111
- Regtest: Block 0 (always active)

**Implementation Location**: `bllvm-consensus/src/block.rs` in `connect_block()`

**Required Changes**:
- Parse block height from coinbase scriptSig
- Validate that parsed height matches actual block height
- Reject blocks where height encoding is missing or incorrect
- Handle height encoding format (CScriptNum)

**Reference**: Bitcoin Core validates this in `CheckBlockHeader()` and `CheckBlock()`.

---

### 3. BIP66 - Strict DER Signatures

**Status**: ⚠️ **PARTIALLY IMPLEMENTED**

**Requirement**: Enforces strict DER encoding for ECDSA signatures. Rejects signatures that are not strictly DER-encoded.

**Activation Heights**:
- Mainnet: Block 363,724
- Testnet: Block 330,776
- Regtest: Block 0 (always active)

**Current Implementation**:
- `SCRIPT_VERIFY_DERSIG` flag exists (0x04)
- `Signature::from_der()` is used, but may accept non-strict DER

**Required Changes**:
- Verify that `secp256k1` library enforces strict DER (it should)
- Ensure signature validation rejects non-strict DER signatures
- Add tests for strict DER validation

**Reference**: Bitcoin Core uses `secp256k1_ecdsa_signature_parse_der()` which enforces strict DER.

---

### 4. BIP90 - Block Version Enforcement

**Status**: ❌ **NOT IMPLEMENTED**

**Requirement**: Enforces that certain block versions are no longer valid after specific heights. Consolidates activation logic for BIP34, BIP65, and BIP66.

**Activation Heights**:
- Mainnet: Various (BIP34: 227,836, BIP65: 388,381, BIP66: 363,724)
- Testnet: Various
- Regtest: Block 0 (always active)

**Implementation Location**: `bllvm-consensus/src/block.rs` in block header validation

**Required Changes**:
- Enforce minimum block version based on height
- Reject blocks with version < 2 after BIP34 activation
- Reject blocks with version < 3 after BIP66 activation
- Reject blocks with version < 4 after BIP65 activation

**Reference**: Bitcoin Core enforces this in `CheckBlockHeader()`.

---

### 5. BIP147 - NULLDUMMY Enforcement

**Status**: ⚠️ **PARTIALLY IMPLEMENTED**

**Requirement**: OP_CHECKMULTISIG requires a dummy element on the stack. With BIP147, this dummy element must be empty (OP_0).

**Activation Heights**:
- Mainnet: Block 481,824 (SegWit activation)
- Testnet: Block 834,624
- Regtest: Block 0 (always active)

**Current Implementation**:
- `SCRIPT_VERIFY_NULLDUMMY` flag exists (0x10)
- Tests exist but implementation may be missing
- OP_CHECKMULTISIG implementation not found in codebase

**Required Changes**:
- Implement OP_CHECKMULTISIG (0xae) if missing
- Enforce NULLDUMMY check when flag is set
- Reject multisig scripts where dummy element is not empty

**Reference**: Bitcoin Core enforces this in `EvalScript()` when processing OP_CHECKMULTISIG.

---

## Implementation Priority

1. **HIGH**: BIP30, BIP34, BIP66 (consensus-critical, required for mainnet)
2. **MEDIUM**: BIP90 (simplifies activation logic)
3. **MEDIUM**: BIP147 (required for SegWit compatibility)

## Testing Requirements

Each BIP implementation must include:
- Unit tests for validation logic
- Integration tests with real block data
- Kani proofs for critical invariants (where applicable)
- Bitcoin Core test vector compliance

## References

- BIP30: https://github.com/bitcoin/bips/blob/master/bip-0030.mediawiki
- BIP34: https://github.com/bitcoin/bips/blob/master/bip-0034.mediawiki
- BIP66: https://github.com/bitcoin/bips/blob/master/bip-0066.mediawiki
- BIP90: https://github.com/bitcoin/bips/blob/master/bip-0090.mediawiki
- BIP147: https://github.com/bitcoin/bips/blob/master/bip-0147.mediawiki

