# Merge Mining and Transaction Fee Forwarding

## Overview

**Important Changes:**
- **Merge Mining**: Now implemented as a **separate optional module** (`blvm-merge-mining`)
- **Fee Forwarding**: **Removed** from the system (not part of monetization model)
- **Governance**: Merge mining revenue does NOT affect governance (maintainer-only multisig)

---

## Merge Mining Module

### Status: ✅ Available as Optional Module

Merge mining is now implemented as a **separate, optional module** (`blvm-merge-mining`) that integrates with the Stratum V2 module.

### Key Changes

1. **Separate Module**: `blvm-merge-mining` is not built into the core node
2. **Module Integration**: Requires `blvm-stratum-v2` module to be loaded
3. **Payment Model**: 
   - One-time activation fee required to use merge mining
   - Hardcoded revenue share: Module developer receives a fixed percentage of merge mining rewards
4. **No Governance Impact**: Merge mining revenue does NOT affect governance (maintainer-only multisig)

### Module Location

- **Module Repository**: `blvm-merge-mining/`
- **Integration**: Integrates with `blvm-stratum-v2` module via IPC
- **Documentation**: See `blvm-merge-mining/README.md`

### Architecture

```
┌─────────────────┐       ┌─────────────────────┐       ┌─────────────────────┐
│   blvm-node     │ <---> │ blvm-stratum-v2     │ <---> │ blvm-merge-mining   │
│  (Core Node)    │       │ (Stratum V2 Module) │       │ (Merge Mining Module)│
└─────────────────┘       └─────────────────────┘       └─────────────────────┘
```

### Configuration

To use merge mining:

1. **Load Stratum V2 module** (required dependency)
2. **Load merge mining module**:
   ```toml
   [modules.blvm-merge-mining]
   enabled = true
   config.merge_mining.secondary_chains = "rsk,namecoin"
   config.merge_mining.revenue_share_percentage = "0.05"
   config.merge_mining.developer_address = "bc1q..."
   ```

### Revenue Model

- **One-time activation fee**: Required to activate the module
- **Hardcoded revenue share**: Module developer receives a fixed percentage (e.g., 5%) of merge mining rewards
- **Revenue collection**: Not collected while IPC integration is pending. This process does not hold or sign the reward. An address string is not a key.

---

## Transaction Fee Forwarding

### Status: ❌ Removed

**Transaction fee forwarding has been removed from Bitcoin Commons.**

### What Changed

- **Fee forwarding**: No longer tracked or used for governance
- **Governance**: Does not affect governance (maintainer-only multisig)
- **Monetization**: Not part of the monetization model

### Historical Context

Fee forwarding was previously considered as a way for node operators to contribute to Commons by forwarding transaction fees. This has been removed as part of the governance simplification.

---

## Governance Impact

**Important**: Neither merge mining nor fee forwarding affect governance decisions.

- **Governance**: Maintainer-only multisig (no contribution-based voting)
- **Zaps**: Tracked for transparency/reporting only (don't affect governance)
- **Merge Mining**: Module revenue, not governance contribution
- **Fee Forwarding**: Removed from system

---

## Related Documentation

- [Module System](https://github.com/BTCDecoded/blvm-node/blob/main/docs/MODULE_SYSTEM.md) - How modules work
- [Monetization User Guide](./MONETIZATION_USER_GUIDE.md) - Current monetization models
- [blvm-merge-mining README](https://github.com/BTCDecoded/blvm-merge-mining/blob/main/README.md) - Merge mining module documentation
