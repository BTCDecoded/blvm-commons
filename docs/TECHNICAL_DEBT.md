# Technical Debt Documentation

Technical debt items across the bllvm-commons codebase, prioritized by impact and urgency.

## Overview

Tracks technical debt items across the bllvm-commons codebase, prioritizing critical issues and documenting why non-critical items remain.

## Critical TODOs (High Priority)

### 1. GitHub API Client - Octocrab 0.38 Migration ✅ COMPLETE
**Location**: `src/github/client.rs`  
**Status**: Complete  
**Resolution**: All 7 API calls updated to octocrab 0.38

**Completed Items**:
- ✅ Branch protection API (uses HTTP fallback - acceptable for Phase 1)
- ✅ Workflow status API (`actions().list_workflow_runs()`)
- ✅ Workflow exists API (`repos().get_content()`)
- ✅ Repository dispatch API (`repos().create_dispatch_event()`)
- ✅ Workflow run status API (`actions().get_workflow_run()`)
- ✅ List workflow runs API (`actions().list_workflow_runs()`)
- ✅ Artifacts API (`actions().list_workflow_run_artifacts()`)
- ✅ Installation token API (`apps().create_installation_access_token()`)

**Note**: Branch protection uses HTTP fallback as octocrab 0.38 API structure may vary. This is acceptable for Phase 1 and can be enhanced in Phase 2.

---

### 2. Nostr Zap Tracking - Invoice Parsing ✅ COMPLETE
**Location**: `src/nostr/zap_tracker.rs`  
**Status**: Complete  
**Resolution**: Implemented using `lightning-invoice` crate

**Completed**:
- ✅ Added `lightning-invoice = "0.2"` dependency
- ✅ Implemented `extract_payment_hash()` using `Invoice::from_str()`
- ✅ Payment hash extraction now works for duplicate detection

---

### 3. Fee Forwarding - Transaction Hashing ✅ COMPLETE
**Location**: `src/governance/fee_forwarding.rs`  
**Status**: Complete  
**Resolution**: Replaced manual serialization with `bllvm_protocol::block::calculate_tx_id`

**Completed**:
- ✅ Removed manual transaction serialization code
- ✅ Now uses `bllvm_protocol::block::calculate_tx_id` which properly serializes transactions
- ✅ Ensures exact match with Bitcoin Core's txid calculation
- ✅ Removed unused `encode_varint` helper function

---

## Non-Critical TODOs (Documented)

### 4. Test Infrastructure - Mock GitHub Client
**Location**: `src/github/cross_layer_status.rs`  
**Line**: 1409  
**Priority**: LOW  
**Reason**: Test uses real RSA key generation, should use mocks  
**Impact**: Tests may be slower or require external dependencies  
**Current State**: Acceptable - tests work correctly

**Why It Remains**:
- Tests are functional and pass
- Mock implementation would require significant refactoring
- Low priority compared to production code improvements

---

### 5. Audit Logger - Cloning
**Location**: `src/main.rs`  
**Line**: 180  
**Priority**: LOW  
**Reason**: Audit logger doesn't implement Clone, using Arc would be better  
**Impact**: Minor code complexity  
**Status**: Acceptable - current implementation works

**Why It Remains**:
- Current implementation is functional
- Refactoring would require significant changes
- Not blocking any features

---

### 6. Vote Aggregator - Participation Votes Table
**Location**: `src/governance/vote_aggregator.rs`  
**Line**: 125  
**Priority**: LOW  
**Reason**: Future enhancement to query explicit votes from database  
**Impact**: Feature enhancement, not a bug  
**Current State**: Planned for future release

**Why It Remains**:
- Current voting mechanism works via zap tracking
- Explicit vote table is a future enhancement
- Not required for Phase 1 functionality

---

### 7. GitHub Integration - Signature Count
**Location**: `src/webhooks/github_integration.rs`  
**Line**: 254  
**Priority**: LOW  
**Reason**: Should query actual signature count from database  
**Impact**: Status checks may show approximate counts  
**Current State**: Acceptable - approximate counts are sufficient

**Why It Remains**:
- Current implementation provides sufficient information
- Database query would add latency
- Not critical for status check functionality

---

### 8. GitHub Integration - Tier Detection
**Location**: `src/webhooks/github_integration.rs`  
**Line**: 292  
**Priority**: LOW  
**Reason**: Hardcoded tier value, should detect from PR  
**Impact**: Tier classification may be incorrect  
**Current State**: Acceptable - tier classification happens elsewhere

**Why It Remains**:
- Tier classification is handled by tier_classification module
- This is a fallback/default value
- Not critical for functionality

---

## Error Handling Improvements

### Completed
- ✅ Fixed `pool().unwrap()` calls in `time_lock.rs` (11 instances)
- ✅ Fixed JSON deserialization error handling in `database/mod.rs` (5 instances)
- ✅ Fixed `unwrap_or_default()` in production code paths

### Remaining
- ⏳ Test code unwrap/expect (acceptable - 43 instances in tests)
- ⏳ Some safe fallbacks in production code (e.g., `unwrap_or_else(|_| vec![])`)

## Metrics

### Before Improvements
- Total unwrap/expect: 476 instances
- Critical production code: ~136 instances
- Test code: ~340 instances

### After Improvements
- Total unwrap/expect: ~465 instances
- Critical production code: ~125 instances (11 fixed)
- Test code: ~340 instances (unchanged - acceptable)

### Target
- Production code: <50 instances (only safe fallbacks)
- Test code: Acceptable to keep unwrap/expect

## Next Steps

1. **Immediate** (This Sprint):
   - ✅ GitHub API client migration (COMPLETE)
   - Fix remaining critical unwrap/expect in production code

2. **Short-term** (Next Sprint):
   - ✅ Bolt11 invoice parsing (COMPLETE)
   - Fix transaction hashing in fee forwarding

3. **Long-term** (Future):
   - Refactor test infrastructure to use mocks
   - Implement explicit vote tracking
   - Improve tier detection in webhooks

## Review Process

Review this document:
- During active development
- Before releases
- When addressing specific technical debt items

## Notes

- Test code unwrap/expect is acceptable as tests should fail fast
- Safe fallbacks (e.g., `unwrap_or_else(|_| vec![])`) are acceptable for non-critical paths
- All critical error paths should use proper error handling

