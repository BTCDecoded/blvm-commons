# Monetization System User Guide

## Important: Governance Model

**Bitcoin Commons uses maintainer-only multisig governance.**

- **Zaps**: Tracked for transparency/reporting only - they do NOT affect governance decisions
- **Merge Mining**: Available as optional module - revenue does NOT affect governance
- **Fee Forwarding**: Removed from system
- **Governance**: Only maintainers can vote on proposals

## How to Participate (Transparency/Reporting)

### 1. Zap to Show Support (Transparency Only)

**What is Zap Tracking?**
- Send Lightning payments (zaps) to governance proposals
- Zaps are tracked for transparency and reporting
- **Important**: Zaps do NOT affect governance decisions (maintainer-only multisig)
- Zaps can include messages: "support", "veto", or "abstain" (for reporting)

**How to Zap a Proposal**:

1. **Find the Proposal**:
   - Proposals are published to Nostr with zap addresses
   - Look for events with tag `["d", "governance-proposal"]`
   - Check the `zap` tag for the Lightning address

2. **Send a Zap**:
   - Use a Nostr client that supports zaps (e.g., Damus, Amethyst)
   - Zap the proposal event with any amount
   - Include message: "support", "veto", or "abstain" (optional, for reporting)

3. **Tracking Only**:
   - Zaps are tracked for transparency and reporting
   - **Zaps do NOT affect governance decisions** (maintainer-only multisig)
   - All zap amounts are recorded for public transparency

**Example**:
```
Zap 0.01 BTC to proposal → Tracked for transparency
Zap 1.0 BTC to proposal → Tracked for transparency
Zap 4.0 BTC to proposal → Tracked for transparency
```

### 2. Merge Mining (Optional Module)

**What is Merge Mining?**
- Mine secondary chains (e.g., RSK, Namecoin) alongside Bitcoin
- Available as optional module: `blvm-merge-mining`
- Requires one-time activation fee
- Module developer receives hardcoded revenue share

**How to Participate**:
1. Load `blvm-stratum-v2` module (required dependency)
2. Load `blvm-merge-mining` module
3. Pay one-time activation fee
4. Configure secondary chains

**Important**:
- Merge mining revenue does NOT affect governance (maintainer-only multisig)
- Revenue is collected by module developer (not governance contribution)
- See `blvm-merge-mining/README.md` for details

### 3. Fee Forwarding

**Status: ❌ Removed**

Fee forwarding has been removed from Bitcoin Commons. It is no longer part of the monetization model.

### 4. General Zaps

**What are General Zaps?**
- Zap the Commons bot directly (not tied to a proposal)
- Tracked for transparency/reporting
- **Important**: Do NOT affect governance (maintainer-only multisig)

**How to Zap**:
1. Find the Commons bot on Nostr
2. Zap the bot with any amount
3. Your zaps are tracked for transparency

**Tracking**:
- All zaps are recorded for public transparency
- No governance weight (maintainer-only multisig)

## Governance Model

### Current System

**Maintainer-Only Multisig**:
- Only maintainers can vote on proposals
- Zaps are tracked for transparency/reporting only
- No contribution-based voting

### Transparency/Reporting

All contributions (zaps, merge mining, etc.) are tracked for:
- **Public transparency**: See who contributed what
- **Reporting**: Public dashboards and reports
- **Historical record**: Track community engagement

**Important**: These do NOT affect governance decisions.

## Checking Your Status

### View Your Contributions

```bash
# Query database (if you have access)
sqlite3 governance.db "
SELECT 
    contributor_id,
    contribution_type,
    SUM(amount_btc) as total_btc
FROM unified_contributions
WHERE contributor_id = 'your_id'
GROUP BY contribution_type;
"
```

### View Your Weight

```bash
sqlite3 governance.db "
SELECT 
    contributor_id,
    total_contribution_btc,
    base_weight,
    capped_weight
FROM participation_weights
WHERE contributor_id = 'your_id';
"
```

### View Proposal Votes

```bash
sqlite3 governance.db "
SELECT 
    pr_id,
    vote_type,
    SUM(vote_weight) as total_weight,
    COUNT(*) as vote_count
FROM proposal_zap_votes
WHERE pr_id = 123
GROUP BY vote_type;
"
```

## Best Practices

1. A zap does not grant a vote. Only maintainers vote.
2. Weight columns on this page are a reporting query, not a ballot.
3. **Use Messages**: Include "support" or "veto" in zap messages when you want that string in the report.

## Troubleshooting

### My Zap Wasn't Counted

**Check**:
1. Was it sent to the correct bot pubkey?
2. Was it a valid NIP-57 zap receipt?
3. Check logs: `journalctl -u blvm-commons | grep zap`

### My Weight Isn't Updating

**Check**:
1. Is weight update task running? (Check logs)
2. Are contributions verified? (Check `verified` column)
3. Are contributions in cooling-off? (Check `contribution_age_days`)

### Proposal Vote Not Showing

**Check**:
1. Was zap sent to the proposal event?
2. Does zap have `zapped_event_id` matching proposal?
3. Check `proposal_zap_votes` table

## Support

For issues or questions:
- GitHub Issues: https://github.com/BTCDecoded/blvm-commons/issues
- Nostr: @BTCCommons_Gov
- Email: governance@btcdecoded.org

