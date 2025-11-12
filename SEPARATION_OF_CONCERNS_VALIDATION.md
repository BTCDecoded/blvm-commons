# Separation of Concerns Plan Validation

**Date**: 2025-01-XX  
**Purpose**: Validate the plan to move protocol-level code from bllvm-node to bllvm-protocol

---

## Validation Results

### ✅ 1. bech32m.rs → bllvm-protocol/src/address.rs

**Dependencies**:
- `bech32` crate (external) ✅
- No bllvm-node dependencies ✅
- No bllvm-protocol dependencies ✅

**Usage in bllvm-node**:
- Only exported in `lib.rs` (line 37)
- No actual usage found in codebase

**Required Changes**:
1. Add `bech32 = "=0.9"` to `bllvm-protocol/Cargo.toml`
2. Move file to `bllvm-protocol/src/address.rs`
3. Update `bllvm-protocol/src/lib.rs` to export `pub mod address;`
4. Remove from `bllvm-node/src/lib.rs`
5. Update any imports (none found currently)

**Validation**: ✅ **SAFE TO MOVE** - Pure protocol-level code, no dependencies

---

### ✅ 2. bip157.rs → bllvm-protocol/src/bip157.rs

**Dependencies**:
- `crate::bip158::CompactBlockFilter` (sibling module - needs to move together) ⚠️
- `bllvm_protocol::{BlockHeader, Hash}` (protocol types) ✅
- `sha2` crate (external) ✅

**Usage in bllvm-node**:
- `network/mod.rs` - Uses `NODE_COMPACT_FILTERS` constant and message types
- `network/protocol.rs` - Uses `NODE_COMPACT_FILTERS` constant
- `network/compact_blocks.rs` - Uses `NODE_COMPACT_FILTERS` constant
- `network/bip157_handler.rs` - Uses message types (GetCfilters, CFilter, etc.)
- `network/filter_service.rs` - Uses `FilterHeader` type

**Required Changes**:
1. Move `bip158.rs` first (bip157 depends on it)
2. Move `bip157.rs` to `bllvm-protocol/src/bip157.rs`
3. Update `bllvm-protocol/src/lib.rs` to export both modules
4. Update imports in bllvm-node:
   - `use crate::bip157::*` → `use bllvm_protocol::bip157::*`
   - `use crate::bip158::*` → `use bllvm_protocol::bip158::*`
5. bllvm-protocol already has `sha2` dependency ✅

**Validation**: ✅ **SAFE TO MOVE** - Only depends on protocol types and bip158 (which also moves)

---

### ✅ 3. bip158.rs → bllvm-protocol/src/bip158.rs

**Dependencies**:
- `bllvm_protocol::Transaction` (protocol type) ✅
- `sha2` crate (external) ✅
- `std::collections::HashSet` (standard library) ✅

**Usage in bllvm-node**:
- `bip157.rs` - Uses `CompactBlockFilter` type
- `network/filter_service.rs` - Uses `build_block_filter()` and `CompactBlockFilter`
- `rpc/blockchain.rs` - Uses `build_block_filter()` function

**Required Changes**:
1. Move `bip158.rs` to `bllvm-protocol/src/bip158.rs` (or keep as `bip158.rs`)
2. Update `bllvm-protocol/src/lib.rs` to export `pub mod bip158;`
3. Update imports in bllvm-node:
   - `use crate::bip158::*` → `use bllvm_protocol::bip158::*`
4. bllvm-protocol already has `sha2` dependency ✅

**Validation**: ✅ **SAFE TO MOVE** - Only depends on protocol types

**Note**: Must move before `bip157.rs` since `bip157.rs` depends on it

---

### ✅ 4. bip70.rs → bllvm-protocol/src/payment.rs

**Dependencies**:
- `secp256k1` crate (external) ⚠️
- `serde` crate (external) ✅ (already in bllvm-protocol)
- `sha2` crate (external) ✅ (already in bllvm-protocol)
- `std::collections::HashMap` (standard library) ✅

**Usage in bllvm-node**:
- `network/bip70_handler.rs` - Uses message types and client/server traits

**Required Changes**:
1. Add `secp256k1 = "=0.28.2"` to `bllvm-protocol/Cargo.toml`
2. Move `bip70.rs` to `bllvm-protocol/src/payment.rs` (or keep as `bip70.rs`)
3. Update `bllvm-protocol/src/lib.rs` to export `pub mod payment;` (or `pub mod bip70;`)
4. Update imports in bllvm-node:
   - `use crate::bip70::*` → `use bllvm_protocol::payment::*` (or `bllvm_protocol::bip70::*`)
5. bllvm-protocol already has `serde` and `sha2` dependencies ✅

**Validation**: ✅ **SAFE TO MOVE** - Only depends on external crates, no node dependencies

---

## Dependency Order

**Move Order** (to avoid breaking dependencies):
1. ✅ `bip158.rs` (no dependencies on other modules being moved)
2. ✅ `bip157.rs` (depends on bip158)
3. ✅ `bech32m.rs` (independent)
4. ✅ `bip70.rs` (independent)

---

## Required Dependency Additions to bllvm-protocol

**Current bllvm-protocol dependencies**:
```toml
serde = "=1.0.193"
sha2 = "=0.10.9"
```

**Additional dependencies needed**:
```toml
bech32 = "=0.9"        # For address encoding
secp256k1 = "=0.28.2"  # For BIP70 payment protocol signatures
```

**Note**: These are protocol-level dependencies, appropriate for bllvm-protocol

---

## Import Update Summary

### bllvm-node/src/lib.rs
**Remove**:
```rust
pub mod bech32m;
pub mod bip158;
pub mod bip157;
pub mod bip70;
```

### bllvm-node/src/network/mod.rs
**Change**:
```rust
// Before
use crate::bip157::NODE_COMPACT_FILTERS;

// After
use bllvm_protocol::bip157::NODE_COMPACT_FILTERS;
```

### bllvm-node/src/network/protocol.rs
**Change**:
```rust
// Before
use crate::bip157::NODE_COMPACT_FILTERS;

// After
use bllvm_protocol::bip157::NODE_COMPACT_FILTERS;
```

### bllvm-node/src/network/filter_service.rs
**Change**:
```rust
// Before
use crate::bip157;
use crate::bip158::{build_block_filter, CompactBlockFilter};

// After
use bllvm_protocol::bip157;
use bllvm_protocol::bip158::{build_block_filter, CompactBlockFilter};
```

### bllvm-node/src/network/bip157_handler.rs
**Change**:
```rust
// Before
// Uses types from protocol.rs which reference bip157 types
// These will need to be updated to use bllvm_protocol::bip157::*

// After
use bllvm_protocol::bip157::*;
```

### bllvm-node/src/network/bip70_handler.rs
**Change**:
```rust
// Before
use crate::bip70::{Bip70Error, PaymentProtocolClient, PaymentProtocolServer, PaymentRequest};

// After
use bllvm_protocol::payment::{Bip70Error, PaymentProtocolClient, PaymentProtocolServer, PaymentRequest};
// OR if keeping name as bip70:
use bllvm_protocol::bip70::{Bip70Error, PaymentProtocolClient, PaymentProtocolServer, PaymentRequest};
```

### bllvm-node/src/rpc/blockchain.rs
**Change**:
```rust
// Before
use crate::bip158::build_block_filter;

// After
use bllvm_protocol::bip158::build_block_filter;
```

---

## Potential Issues

### 1. Circular Dependencies
**Risk**: None ✅
- bllvm-protocol does not depend on bllvm-node
- Moving code from node to protocol maintains this separation

### 2. Breaking Changes
**Risk**: Medium ⚠️
- All imports in bllvm-node need to be updated
- Tests may need updates
- **Mitigation**: Update all imports in single commit, run full test suite

### 3. Test Dependencies
**Risk**: Low ✅
- Tests in moved modules should still work
- Tests in bllvm-node that use these modules need import updates

### 4. Handler Dependencies
**Risk**: Low ✅
- Handlers (`bip157_handler.rs`, `bip70_handler.rs`) stay in bllvm-node
- They just import from bllvm-protocol instead of crate

---

## Validation Summary

### ✅ All Moves Are Safe

1. **bech32m.rs**: ✅ Safe - Pure protocol code, no dependencies
2. **bip158.rs**: ✅ Safe - Only depends on protocol types
3. **bip157.rs**: ✅ Safe - Depends on bip158 (which also moves) and protocol types
4. **bip70.rs**: ✅ Safe - Only depends on external crates

### Required Actions

1. ✅ Add `bech32 = "=0.9"` to bllvm-protocol/Cargo.toml
2. ✅ Add `secp256k1 = "=0.28.2"` to bllvm-protocol/Cargo.toml
3. ✅ Move files in dependency order (bip158 → bip157 → bech32m → bip70)
4. ✅ Update exports in bllvm-protocol/src/lib.rs
5. ✅ Update imports in bllvm-node (8 files need updates)
6. ✅ Remove module declarations from bllvm-node/src/lib.rs
7. ✅ Run full test suite to verify

### Estimated Effort

- **Dependency additions**: 5 minutes
- **File moves**: 10 minutes
- **Import updates**: 30-60 minutes (8 files)
- **Testing**: 30 minutes
- **Total**: 1.5-2 hours

---

## Recommendation

✅ **PROCEED WITH PLAN** - All moves are safe and well-justified. The separation of concerns will be significantly improved.

**Next Steps**:
1. Add dependencies to bllvm-protocol/Cargo.toml
2. Move files in dependency order
3. Update exports and imports
4. Run test suite
5. Verify no circular dependencies

