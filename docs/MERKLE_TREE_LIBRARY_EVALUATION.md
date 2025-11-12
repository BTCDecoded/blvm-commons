# Merkle Tree Library Evaluation for UTXO Commitments

## Top 3 Candidates Analysis

### 1. rs-merkle (v1.5.0) - PRIMARY CANDIDATE

**Status**: Most feature-complete, actively maintained (220 stars, updated Oct 2024)

**Key Features from Documentation**:
- ✅ **Transactional changes**: "making transactional changes to the tree and rolling back to any previously committed tree state, similarly to Git"
- ✅ **Multi-proofs**: Supports proofs for single and multiple elements
- ✅ **Highly customizable**: Configurable hashing function and tree construction
- ✅ **no_std support**: Can be used without standard library

**Critical Question**: Does transactional support mean efficient incremental updates?

**From API Documentation (docs.rs/rs_merkle)**:
- ✅ **`insert(&mut self, leaf)`**: Inserts new leaf (doesn't modify root until commit)
- ✅ **`append(&mut self, leaves)`**: Appends multiple leaves  
- ✅ **`commit(&mut self)`**: Commits changes to root
- ✅ **`rollback(&mut self)`**: Rolls back one commit
- ✅ **`uncommitted_root(&self)`**: Gets root without committing

**Pattern Confirmed**:
```rust
// Incremental update pattern
tree.insert(leaf1);
tree.insert(leaf2);
tree.commit(); // Apply changes (need to verify if O(log n) or O(n))
```

**Key Question**: Is `commit()` O(log n) per inserted leaf or does it rebuild tree O(n)?
- Need to verify by checking source code or benchmarking

**Pros:**
- Most advanced library (as stated in description)
- Actively maintained (updated recently)
- Well-documented (docs.rs available)
- Used in production (220 stars suggests real usage)
- Customizable hash functions (can use SHA256)

**Cons:**
- May have overhead for rollback features (if we don't need rollback)
- Need to verify incremental update performance
- May be more complex than needed

**API Methods Found**:
- ✅ `insert(&mut self, leaf)` - Add single leaf (doesn't modify root until commit)
- ✅ `append(&mut self, leaves)` - Add multiple leaves
- ✅ `commit(&mut self)` - Apply changes to root (need to verify O(log n) vs O(n))
- ❌ `remove()` - **NOT FOUND** - No direct removal method
- ❌ `update()` - **NOT FOUND** - No direct update method
- ✅ `rollback()` - Rolls back one commit (removes entire commit, not single leaf)

**Key Limitation**: 
- **No `remove()` method** - To remove a UTXO, would need to:
  - Option A: Rollback entire commit (loses all changes)
  - Option B: Rebuild tree without removed UTXO (O(n) operation)
  - Option C: Use sparse-merkle-tree instead (has `update()`)

**Next Steps**:
1. ✅ API docs reviewed - `insert`, `append`, `commit` confirmed
2. ❌ **`remove()` does NOT exist** - This is a blocker for UTXO deletions
3. ⚠️ **Consider sparse-merkle-tree** if we need efficient deletions
4. Test `commit()` performance to verify O(log n) vs O(n)
5. Test with 1M UTXO dataset to measure actual performance

---

### 2. sparse-merkle-tree (v0.6.1) - STRONG CANDIDATE FOR DELETIONS

**Status**: Optimized for sparse trees, blockchain-focused

**Key Features from API Documentation (docs.rs/sparse-merkle-tree)**:
- ✅ **`update(key, value)`**: Update a leaf, returns new merkle root (incremental!)
- ✅ **`update_all(leaves)`**: Update multiple leaves at once
- ✅ **`get(key)`**: Get value of a leaf
- ✅ **`merkle_proof(keys)`**: Generate merkle proof for multiple keys
- ✅ **Key-value interface**: Uses H256 keys (perfect for OutPoint hashes)
- ✅ **Store abstraction**: Customizable storage backends

**Critical Question**: How does it handle dense UTXO sets (150M UTXOs)?

**API Pattern for UTXO**:
```rust
// Insert UTXO
tree.update(outpoint_hash, utxo_data)?;

// Remove UTXO (update with empty/default value)
tree.update(outpoint_hash, default_value())?;

// Batch updates (efficient for block processing)
let updates = vec![(hash1, utxo1), (hash2, utxo2)];
tree.update_all(updates)?;
```

**Pros:**
- ✅ **Incremental updates confirmed**: `update()` returns new root immediately
- ✅ **Perfect for deletions**: Sparse trees handle empty leaves efficiently
- ✅ **Key-value interface**: Maps directly to OutPoint → UTXO
- ✅ **Blockchain-focused**: Designed for applications like ours
- ✅ **Store abstraction**: Can use custom storage backends

**Cons:**
- ⚠️ Less documented than rs-merkle
- ⚠️ Need to verify dense tree performance (150M UTXOs is dense, not sparse)
- ⚠️ `update()` might use default/empty value for deletion (need to verify)

**Key Question**: Does `update(key, empty_value)` efficiently delete, or do we need explicit remove?

**Next Steps**:
1. ✅ API reviewed - `update()` confirmed for incremental updates
2. Verify if `update()` with empty value efficiently deletes
3. Benchmark with dense 1M UTXO dataset
4. Check if it handles 150M UTXOs efficiently (despite being "sparse")

---

### 3. merkle_light (v0.4.0) - TERTIARY CANDIDATE

**Status**: Lightweight, cache-friendly

**Key Features**:
- ✅ **Vector-based**: Uses vector allocation (cache-friendly)
- ✅ **Lightweight**: Minimal dependencies
- ✅ **SPV support**: Simplified Payment Verification (Bitcoin-relevant)
- ✅ **std::hash::Hasher**: Compatible with standard Rust traits

**Critical Question**: Does it support incremental updates or only full rebuild?

**Pros:**
- Cache-friendly memory layout
- Lightweight (few dependencies)
- SPV support (Bitcoin-specific feature)

**Cons:**
- May not support incremental updates (needs verification)
- Less feature-rich
- Less actively maintained (need to verify)

**Next Steps**:
1. Check API for update/insert methods
2. Verify if it rebuilds tree or updates incrementally
3. Test performance if it does support incremental updates

---

## Evaluation Plan

### Phase 1: Documentation Deep Dive (Day 1)

#### rs-merkle
- [ ] Review API docs at docs.rs/rs_merkle
- [ ] Find examples directory in GitHub repo
- [ ] Look for `insert`, `remove`, `update`, `commit` methods
- [ ] Check if operations are O(log n) or O(n)

#### sparse-merkle-tree
- [ ] Review API docs at docs.rs/sparse-merkle-tree
- [ ] Check GitHub README for usage examples
- [ ] Verify dense tree performance claims
- [ ] Look for insert/delete operations

#### merkle_light
- [ ] Review API docs at docs.rs/merkle_light
- [ ] Check if it has update methods or only rebuild
- [ ] Verify SPV implementation

### Phase 2: Quick Prototype (Day 2)

Create minimal test harness:

```rust
// Test harness for evaluating libraries
fn test_incremental_updates<M: MerkleTreeLike>() {
    // Build tree with 100K UTXOs
    let mut tree = M::new();
    for i in 0..100_000 {
        tree.insert(utxo_hash(i));
    }
    
    // Measure: Insert 1000 new UTXOs
    let start = Instant::now();
    for i in 100_000..101_000 {
        tree.insert(utxo_hash(i));
    }
    let insert_time = start.elapsed();
    
    // Measure: Remove 1000 UTXOs
    let start = Instant::now();
    for i in 0..1000 {
        tree.remove(utxo_hash(i));
    }
    let remove_time = start.elapsed();
    
    // Measure: Generate 100 proofs
    let start = Instant::now();
    for i in 0..100 {
        tree.prove(i);
    }
    let prove_time = start.elapsed();
    
    // Report results
    println!("Insert: {:?}, Remove: {:?}, Prove: {:?}", 
             insert_time, remove_time, prove_time);
}
```

### Phase 3: Benchmarking (Day 3)

- [ ] Test with 100K UTXOs (small)
- [ ] Test with 1M UTXOs (medium)
- [ ] Test with 10M UTXOs (large, if feasible)
- [ ] Measure memory usage
- [ ] Compare incremental vs full rebuild performance

### Phase 4: Decision Matrix (Day 4)

| Criteria | rs-merkle | sparse-merkle-tree | merkle_light |
|----------|-----------|-------------------|--------------|
| Incremental Updates | ⏳ Verify | ⏳ Verify | ⏳ Verify |
| Large Dataset (150M) | ⏳ Test | ⏳ Test | ⏳ Test |
| Memory Efficiency | ⏳ Test | ⏳ Test | ⏳ Test |
| Proof Generation Speed | ⏳ Test | ⏳ Test | ⏳ Test |
| Documentation Quality | ✅ Good | ⏳ Check | ⏳ Check |
| Maintenance Status | ✅ Active | ⏳ Check | ⏳ Check |
| Bitcoin-Specific Features | ❌ None | ❌ None | ✅ SPV |

---

## Recommended Next Steps

### Updated Recommendation Based on API Review

**1. PRIMARY: sparse-merkle-tree** (UPDATED - was secondary)
   - ✅ **Confirmed**: Has `update()` method for incremental updates
   - ✅ **Confirmed**: Key-value interface perfect for OutPoint → UTXO mapping
   - ✅ **Confirmed**: Returns new root immediately (no commit needed)
   - ⚠️ **Need to verify**: Dense tree performance (150M UTXOs)
   - ⚠️ **Need to verify**: How deletion works (`update(key, empty)`?)

**2. SECONDARY: rs-merkle**
   - ✅ **Confirmed**: Has `insert()` and `commit()` for incremental updates
   - ❌ **Missing**: No `remove()` method (only rollback entire commit)
   - ⚠️ **Issue**: Would need to rebuild tree to remove UTXO (not ideal)
   - ✅ **Use case**: Good if we never need to remove, only append

**3. TERTIARY: merkle_light**
   - ⚠️ **Need to verify**: Has update methods
   - ✅ **Pro**: SPV support (Bitcoin-specific)
   - ❌ **Likely issue**: May not support incremental updates

### Decision Strategy

**For UTXO Commitments (need insert AND remove)**:
1. **Try sparse-merkle-tree first** (has `update()` which can do both)
2. **Verify deletion pattern**: Does `update(key, default_value)` efficiently delete?
3. **Benchmark dense performance**: Test with 1M, 10M UTXOs
4. **If sparse-merkle-tree works**: Use it (best fit)
5. **If not**: Extend rs-merkle with remove functionality OR wrap sparse-merkle-tree

**For UTXO Commitments (append-only, never remove)**:
1. **Use rs-merkle** (simpler, well-documented)
2. **Build new tree** when UTXO is spent (accept the overhead)

---

## Integration with Iroh

Regardless of which library chosen, Iroh can be used for:

1. **UTXO Set Distribution**:
   - Content-addressed: Same UTXO set = same hash (deduplication)
   - Efficient chunking for 10GB transfers
   - Automatic retry/recovery

2. **Incremental Updates**:
   - Only sync changed chunks
   - Multi-path download for speed
   - NAT traversal for mobile nodes

3. **Peer Discovery**:
   - Find nodes with UTXO sets
   - Diverse peer selection (different ASNs, geos)

**Architecture**:
```
Iroh Sync (Distribution Layer)
    ↓
UTXO Set Chunks (Content-addressed)
    ↓
Merkle Tree Library (Commitment Layer)
    ↓
UTXO Commitment (Merkle Root)
```

---

## Decision Timeline

- **Week 3, Day 1**: Documentation review
- **Week 3, Day 2**: Quick prototypes
- **Week 3, Day 3**: Benchmarks
- **Week 3, Day 4**: Decision + integration plan
- **Week 3, Day 5**: Begin implementation with chosen library
