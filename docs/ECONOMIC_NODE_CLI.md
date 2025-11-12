# Economic Node CLI Tools

This guide explains how to use the economic node CLI tools for registering nodes, submitting veto signals, and verifying governance participation.

## Overview

The economic node system allows mining pools, exchanges, and custodians to participate in governance by:
- Registering with proof of their economic stake
- Submitting veto signals for Tier 3+ changes
- Verifying the integrity of the governance system

## Tools Available

### 1. economic-node-register

Register and manage economic node participation.

#### Generate Keypair

```bash
cargo run --release --bin economic-node-register generate \
  --name "MyMiningPool" \
  --output ./economic-keys
```

#### Register Node

```bash
cargo run --release --bin economic-node-register register \
  --name "MyMiningPool" \
  --node-type "mining_pool" \
  --public-key ./economic-keys/MyMiningPool_public.pem \
  --hash-rate-percent 15.5 \
  --proof-file ./proof-of-hash-rate.json
```

#### Submit Proof of Stake

```bash
cargo run --release --bin economic-node-register proof \
  --name "MyMiningPool" \
  --proof-type "hash_rate" \
  --data-file ./hash-rate-data.json
```

#### Check Registration Status

```bash
cargo run --release --bin economic-node-register status \
  --name "MyMiningPool"
```

### 2. economic-node-veto

Submit and manage veto signals for governance changes.

#### Submit Veto Signal

```bash
cargo run --release --bin economic-node-veto veto \
  --node "MyMiningPool" \
  --key ./economic-keys/MyMiningPool_private.pem \
  --repo "btcdecoded/governance" \
  --pr 123 \
  --reason "This change could negatively impact mining economics" \
  --strength 100
```

#### Check Veto Status

```bash
cargo run --release --bin economic-node-veto status \
  --repo "btcdecoded/governance" \
  --pr 123
```

#### List Active Vetoes

```bash
cargo run --release --bin economic-node-veto list
cargo run --release --bin economic-node-veto list --repo "btcdecoded/governance"
```

#### Withdraw Veto

```bash
cargo run --release --bin economic-node-veto withdraw \
  --node "MyMiningPool" \
  --key ./economic-keys/MyMiningPool_private.pem \
  --repo "btcdecoded/governance" \
  --pr 123
```

### 3. economic-node-verify

Verify economic node registrations and veto signals.

#### Verify Registration

```bash
cargo run --release --bin economic-node-verify registration \
  --name "MyMiningPool"
cargo run --release --bin economic-node-verify registration \
  --name "MyMiningPool" \
  --file ./custom-registration.json
```

#### Verify Veto Signal

```bash
cargo run --release --bin economic-node-verify veto \
  --file ./veto-signals/btcdecoded_governance_123_MyMiningPool.json
```

#### Verify All Registrations

```bash
cargo run --release --bin economic-node-verify all-registrations
cargo run --release --bin economic-node-verify all-registrations --dir ./custom-registrations
```

#### Verify All Vetoes

```bash
cargo run --release --bin economic-node-verify all-vetoes
cargo run --release --bin economic-node-verify all-vetoes --dir ./custom-vetoes
```

#### Check Governance Status

```bash
cargo run --release --bin economic-node-verify status \
  --repo "btcdecoded/governance" \
  --pr 123
```

## Node Types and Requirements

### Mining Pools

- **Required**: `hash_rate_percent` (0-100%)
- **Proof**: Hash rate verification data
- **Veto Weight**: Based on hash rate percentage

### Exchanges

- **Required**: `economic_activity_percent` (0-100%)
- **Proof**: Proof of reserves data
- **Veto Weight**: Based on economic activity percentage

### Custodians

- **Required**: `economic_activity_percent` (0-100%)
- **Proof**: Proof of custody data
- **Veto Weight**: Based on economic activity percentage

## Veto System

### Veto Thresholds

- **Hash Rate**: 30% of total network hash rate
- **Economic Activity**: 40% of total economic activity
- **Combined**: Either threshold can trigger veto

### Veto Process

1. **Submit Veto**: Use `economic-node-veto veto` command
2. **Verify Veto**: Use `economic-node-verify veto` command
3. **Check Status**: Use `economic-node-veto status` command
4. **Withdraw if Needed**: Use `economic-node-veto withdraw` command

### Veto Strength

- **1-100%**: Percentage of your stake to apply to veto
- **100%**: Full veto power (recommended)
- **Lower values**: Partial veto (for testing or coordination)

## Proof of Stake Verification

### Hash Rate Proof

For mining pools, provide proof of hash rate:

```json
{
  "hash_rate_thps": 150000000000000000,
  "network_hash_rate_thps": 1000000000000000000,
  "percentage": 15.0,
  "verification_method": "pool_api",
  "verification_url": "https://pool.example.com/api/stats",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Proof of Reserves

For exchanges, provide proof of reserves:

```json
{
  "total_btc_held": 1000.5,
  "total_user_btc": 950.2,
  "reserve_ratio": 1.053,
  "verification_method": "merkle_tree",
  "merkle_root": "abc123...",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Proof of Custody

For custodians, provide proof of custody:

```json
{
  "total_btc_custodied": 500.0,
  "client_count": 1000,
  "verification_method": "attestation",
  "attestation_url": "https://custodian.example.com/attestation",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Security Best Practices

### Key Management

- Store private keys securely
- Use hardware security modules for production
- Rotate keys periodically
- Never share private keys

### Verification

- Always verify registrations before submitting
- Check veto signals for accuracy
- Verify timestamps and signatures
- Monitor for suspicious activity

### Network Security

- Use secure connections for API calls
- Verify SSL certificates
- Monitor network traffic
- Use VPNs for sensitive operations

## Integration with Governance System

### Registration Flow

1. Generate keypair
2. Create proof of stake data
3. Register with governance system
4. Submit proof for verification
5. Monitor registration status

### Veto Flow

1. Monitor governance changes
2. Evaluate impact on your operations
3. Submit veto signal if needed
4. Monitor veto status
5. Withdraw veto if circumstances change

### Verification Flow

1. Verify your own registrations
2. Verify veto signals you submit
3. Monitor overall governance status
4. Report suspicious activity

## Troubleshooting

### Common Issues

**"Registration file not found"**
- Check file path and permissions
- Ensure registration was completed
- Verify file format

**"Invalid node type"**
- Use one of: mining_pool, exchange, custodian
- Check spelling and case

**"Veto threshold not met"**
- Check total veto strength
- Verify active vetoes
- Consider coordinating with other nodes

**"Signature verification failed"**
- Check private key file
- Verify key format
- Ensure proper permissions

### Getting Help

- Check governance-app logs
- Verify file formats and permissions
- Contact governance team for assistance
- Review documentation for updates

## Production Considerations

For production use:
- Use hardware security modules
- Implement proper key rotation
- Set up monitoring and alerting
- Coordinate with other economic nodes
- Follow security best practices
- Maintain audit trails
