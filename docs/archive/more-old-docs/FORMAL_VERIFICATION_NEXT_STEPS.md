# Formal Verification Next Steps

## Immediate Next Actions (Incremental & Methodical)

### 1. Fix Remaining Compilation Issues ✅
- [x] Fixed script.rs syntax error (double brace)
- [ ] Fix any remaining compilation errors in other modules
- [ ] Verify all Kani proofs compile successfully

### 2. Complete Phase 1: Kani Proof Expansion

#### Priority 1: Difficulty Adjustment Proofs (Next)
- **File**: `bllvm-consensus/src/pow.rs`
- **Status**: Started - added `kani_difficulty_adjustment_clamping()`
- **Remaining**:
  - [ ] Verify proof compiles
  - [ ] Add proof for difficulty adjustment correctness (adjustment factor calculation)
  - [ ] Add proof for target bounds (never exceeds MAX_TARGET)

#### Priority 2: Expand Economic Proofs
- **File**: `bllvm-consensus/src/economic.rs`
- **Status**: Enhanced - added supply limit validation proof
- **Remaining**:
  - [ ] Add proof for block subsidy halving at exact intervals
  - [ ] Add proof for asymptotic supply limit (21M BTC never exceeded at any height)

#### Priority 3: UTXO Set Consistency
- **Files**: `bllvm-consensus/src/transaction.rs`, `bllvm-consensus/src/block.rs`
- **Status**: Partial - has `kani_apply_transaction_consistency()`
- **Remaining**:
  - [ ] Add proof for no double-spending
  - [ ] Add proof for UTXO set consistency after block connection
  - [ ] Add proof for UTXO set size bounds

#### Priority 4: Script Execution Termination
- **File**: `bllvm-consensus/src/script.rs`
- **Status**: Good - has bounds proofs
- **Remaining**:
  - [ ] Add proof for script execution always terminates (no infinite loops)
  - [ ] Add proof for operation count bounded by MAX_SCRIPT_OPS

### 3. Phase 2: Property Test Expansion

#### Transaction Edge Cases
- **Target**: 20+ property tests
- **Focus Areas**:
  - All invalid input combinations
  - Value boundary cases (0, 1, MAX_MONEY-1, MAX_MONEY, MAX_MONEY+1)
  - Size boundary cases (0 inputs, max inputs, etc.)
  - Coinbase edge cases

#### Script Opcode Coverage
- **Target**: 30+ property tests
- **Focus Areas**:
  - All opcode combinations
  - Stack state transitions
  - Error conditions

#### Block Edge Cases
- **Target**: 15+ property tests
- **Focus Areas**:
  - Weight boundaries
  - Transaction count limits
  - Header validation edge cases

### 4. Phase 3: Spec Synchronization

#### Automated Comparison Tool
- **File**: `scripts/extract_consensus_rules.py` (new)
- **Functionality**:
  1. Extract validation rules from Orange Paper (markdown parsing)
  2. Extract validation rules from Rust code (AST analysis)
  3. Compare and report differences
  4. Generate coverage report

#### CI/CD Integration
- **File**: `.github/workflows/spec-drift-detection.yml` (enhance existing)
- **Actions**:
  - Run rule extraction on every PR
  - Compare Orange Paper rules vs code
  - Fail on mismatch
  - Generate diff report

### 5. Phase 4: Mathematical Proof Documentation

#### Create Proofs Document
- **File**: `docs/MATHEMATICAL_PROOFS.md` (new)
- **Structure**:
  - For each Orange Paper theorem:
    - Mathematical statement
    - Proof sketch
    - Code location
    - Kani proof reference
    - Test coverage

## Incremental Workflow

### Session Pattern

1. **Pick One Module** (e.g., `pow.rs`)
2. **Read Orange Paper Section** (e.g., Section 7: Proof of Work)
3. **Identify Missing Proofs** (e.g., difficulty adjustment clamping)
4. **Write Proof** (following existing patterns)
5. **Verify Compiles** (`cargo check --features verify`)
6. **Document** (update progress tracking)

### Verification Checklist (Per Proof)

- [ ] Proof compiles (`cargo check`)
- [ ] Proof has mathematical specification comment
- [ ] Proof tests critical invariant
- [ ] Proof is documented in coverage tracking
- [ ] Proof is linked to Orange Paper section

## Success Metrics (Updated)

| Metric | Before | Current | Target | Status |
|--------|--------|---------|--------|--------|
| Kani Proofs | 46 | 48-50 | 60+ | 🟡 80% |
| Property Tests | 3 | 3 | 100+ | 🔴 3% |
| Test Coverage | 95% | 95% | 99% | 🟢 96% |
| TODOs Fixed | 0 | 2 | 0 | 🟡 25% |
| Spec Sync | 0% | 0% | 100% | 🔴 0% |

## Files to Work On Next

1. **`bllvm-consensus/src/pow.rs`** (Difficulty proofs - HIGH PRIORITY)
2. **`bllvm-consensus/src/transaction.rs`** (UTXO consistency - HIGH PRIORITY)
3. **`bllvm-consensus/tests/**/proptest*.rs`** (Property tests - MEDIUM)
4. **`scripts/extract_consensus_rules.py`** (Spec sync - MEDIUM)
5. **`docs/MATHEMATICAL_PROOFS.md`** (Documentation - LOW)

---

**Next Session Plan**:
1. Complete difficulty adjustment proofs in pow.rs
2. Add 5-10 property tests for transaction edge cases
3. Fix any remaining TODOs
4. Update coverage metrics

