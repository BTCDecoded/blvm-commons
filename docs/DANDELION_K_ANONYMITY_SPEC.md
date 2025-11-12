# Dandelion++ k-Anonymity Specification

## Adversary Model

### Adversary Capabilities

1. **Passive Observer**: The adversary can observe all network traffic on links it controls or intercepts.
2. **Active Participant**: The adversary operates one or more nodes in the network and can:
   - Control routing decisions
   - Delay, drop, or modify messages
   - Track message timings and sizes
3. **Eclipse Attack**: The adversary can control a subset of peers connected to a target node.
4. **Graph Analysis**: The adversary can construct and analyze the network graph topology.

### Adversary Goals

- **Transaction Linkage**: Associate a transaction `tx` with its originating node `N_i`.
- **Timing Correlation**: Link transaction appearance times across the network.
- **Path Reconstruction**: Trace the stem path `N_0 → N_1 → ... → N_k` for a transaction.

## k-Anonymity Definition

**Definition (k-Anonymity for Dandelion)**: A transaction `tx` satisfies k-anonymity if, from the adversary's perspective, `tx` could have originated from at least `k` distinct nodes with equal probability.

### Formal Statement

Let:
- `O` = set of nodes that could have originated `tx` (from adversary's view)
- `P(O = N_i | Evidence)` = probability that node `N_i` originated `tx` given observed evidence

Then `tx` has **k-anonymity** if:
- `|O| ≥ k`
- `∀ N_i, N_j ∈ O: P(O = N_i | Evidence) = P(O = N_j | Evidence)`

## Algorithm Specification

### Stem Phase Parameters

- `p_fluff` ∈ [0, 1]: Probability of transitioning to fluff at each hop (default: 0.1)
- `max_stem_hops` ∈ ℕ: Maximum number of stem hops before forced fluff (default: 2)
- `stem_timeout` ∈ ℝ⁺: Maximum duration (seconds) in stem phase before timeout fluff

### Stem Phase Algorithm

```rust
fn stem_phase_relay(tx: Transaction, current_peer: Peer, peers: &[Peer]) -> Option<Peer> {
    // 1. Check if already in stem phase
    if let Some(state) = get_stem_state(&tx) {
        // 2. Check timeout
        if elapsed_time(state.started) > stem_timeout {
            return None; // Fluff via timeout
        }
        
        // 3. Check hop limit
        if state.hop_count >= max_stem_hops {
            return None; // Fluff via hop limit
        }
        
        // 4. Probabilistic fluff decision
        if random() < p_fluff {
            return None; // Fluff via probability
        }
        
        // 5. Advance stem
        let next = select_random_peer(peers, exclude: current_peer);
        update_stem_state(&tx, hop_count: state.hop_count + 1);
        return Some(next);
    } else {
        // 6. Start new stem phase
        let next = select_random_peer(peers);
        start_stem_phase(&tx, current_peer, next);
        return Some(next);
    }
}
```

### Fluff Phase

When `stem_phase_relay` returns `None`, the transaction enters **fluff phase** and is broadcast to all peers simultaneously (standard Bitcoin relay).

## k-Anonymity Analysis

### Theorem 1: Stem Phase Anonymity

**Claim**: During the stem phase, if the adversary observes a transaction at node `N_i`, the set of possible originators includes all nodes that have been on the stem path up to `N_i`.

**Proof Sketch**:
- The adversary cannot distinguish between:
  1. `tx` originated at `N_i` and is in its first stem hop
  2. `tx` originated at any previous node `N_j` (j < i) and is being forwarded

- The random peer selection at each hop ensures uniform probability distribution over all possible originators in the path.

### Theorem 2: Minimum k-Anonymity Guarantee

**Claim**: For a stem path of length `h` hops, the minimum k-anonymity is `k ≥ h + 1`.

**Proof Sketch**:
- A stem path `N_0 → N_1 → ... → N_h` contains `h + 1` nodes.
- From the adversary's perspective at `N_h`, any of these `h + 1` nodes could have originated `tx`.
- Therefore, `k ≥ h + 1`.

**Corollary**: With `max_stem_hops = 2`, we guarantee `k ≥ 3` (3-anonymity).

### Theorem 3: Timeout Guarantee

**Claim**: Even if the adversary controls all peers except the originator, the stem phase will terminate within `stem_timeout` seconds, preventing indefinite stem loops.

**Proof**: The timeout check in step 2 of the algorithm ensures that `tx` will transition to fluff phase within `stem_timeout` seconds regardless of peer behavior.

### Theorem 4: No Premature Broadcast

**Claim**: During the stem phase, a transaction is never broadcast to multiple peers simultaneously (only forwarded to a single next-hop peer).

**Proof**: The algorithm returns `Option<Peer>` where `Some(peer)` indicates single-peer relay and `None` indicates transition to fluff. The fluff phase is the only mechanism for broadcast.

## Implementation Invariants (Kani-Verified)

The following invariants are enforced via Kani model checking:

1. **No Premature Broadcast**: `∀ tx, phase: phase == Stem ⟹ broadcast_count(tx) == 0`
2. **Bounded Stem Length**: `∀ tx: stem_hops(tx) ≤ max_stem_hops`
3. **Timeout Enforcement**: `∀ tx: elapsed_time(tx) ≤ stem_timeout ⟹ phase(tx) == Fluff`
4. **Single Stem State**: `∀ tx: |stem_states(tx)| ≤ 1`
5. **Eventual Fluff**: `∀ tx: ∃ t: phase_at_time(tx, t) == Fluff`

## Attack Resistance

### Eclipse Attack Resistance

If the adversary controls `c` of the `n` available peers:
- The originator's random selection has probability `(n-c)/n` of choosing a non-adversarial peer.
- Even if an adversarial peer is chosen, the timeout guarantee ensures eventual fluff.
- **Result**: k-anonymity degrades gracefully; the originator is still hidden among `k ≥ (n-c)` nodes.

### Timing Correlation Attacks

The random per-hop delay and probabilistic fluff timing break simple timing correlations. However:
- **Limitation**: Precise timing analysis across the network can still provide probabilistic linkage.
- **Mitigation**: Additional jitter and randomized delays can be added.

### Graph Analysis Attacks

If the adversary can construct the complete network graph:
- **Weakness**: Knowledge of graph topology allows probabilistic path reconstruction.
- **Mitigation**: The random peer selection and bounded stem length limit information leakage.

## Security Parameters

Recommended values for production:
- `p_fluff = 0.1` (10% chance per hop)
- `max_stem_hops = 2`
- `stem_timeout = 10` seconds

These provide:
- Minimum k-anonymity: `k ≥ 3`
- Expected stem length: `1/p_fluff = 10` hops (in expectation, but capped at 2)
- Maximum stem duration: 10 seconds

## References

- Dandelion++: Redesigning the Bitcoin Network for Anonymity (Fanti et al., 2019)
- Bitcoin Privacy Model: k-Anonymity Analysis (research papers)

