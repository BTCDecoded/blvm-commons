# Maintainer Key Management

## Critical: Replace Placeholder Keys

**All maintainer keys in `governance/config/maintainers/*.yml` are currently placeholders and MUST be replaced with real cryptographic keys before production use.**

## Current Status

All maintainer configuration files contain placeholder public keys:
- `layer-1-2.yml`: 5 maintainers with `0x02[PLACEHOLDER_64_CHAR_HEX]`
- `layer-3.yml`: 5 maintainers with `0x03[PLACEHOLDER_64_CHAR_HEX]`
- `layer-4.yml`: 5 maintainers with `0x03[PLACEHOLDER_64_CHAR_HEX]`
- `emergency.yml`: 7 emergency keyholders with `0x04[PLACEHOLDER_64_CHAR_HEX]`

## Security Implications

**⚠️ WARNING**: Using placeholder keys means:
- Emergency activations cannot be cryptographically verified
- PR signatures cannot be verified
- Governance enforcement is completely bypassed
- The system is vulnerable to unauthorized changes

## Required Actions

### 1. Generate Real Keys

Each maintainer must generate a secp256k1 keypair using `bllvm-sdk`:

```rust
use bllvm_sdk::governance::GovernanceKeypair;

let keypair = GovernanceKeypair::generate()?;
let public_key = keypair.public_key();
println!("Public key: {}", public_key);
```

Or use the `key-manager` CLI tool:

```bash
cd bllvm-commons
cargo run --bin key-manager generate
```

### 2. Update Configuration Files

Replace placeholder keys in:
- `governance/config/maintainers/layer-1-2.yml`
- `governance/config/maintainers/layer-3.yml`
- `governance/config/maintainers/layer-4.yml`
- `governance/config/maintainers/emergency.yml`

Format: `0x02<64_hex_chars>` for compressed public keys (33 bytes = 66 hex chars including 0x prefix)

### 3. Verify Key Format

Public keys must be:
- Compressed secp256k1 public keys (33 bytes)
- Hex-encoded with `0x` prefix
- Valid secp256k1 points

### 4. Test Signature Verification

After updating keys, test that signature verification works:

```bash
# Test emergency signature verification
cargo test --test emergency_validation

# Test maintainer signature verification
cargo test --test signature_verification
```

## Key Distribution

**IMPORTANT**: Private keys must be:
- Stored securely by each maintainer
- Never committed to the repository
- Backed up securely
- Rotated periodically

## Migration Path

1. **Phase 1**: Generate keys for all maintainers
2. **Phase 2**: Update configuration files with real keys
3. **Phase 3**: Test signature verification end-to-end
4. **Phase 4**: Deploy to production

## Related Code

- Key generation: `bllvm-commons/src/bin/key_manager.rs`
- Signature verification: `bllvm-commons/src/validation/emergency.rs:262`
- Public key parsing: `bllvm-sdk/src/governance/keys.rs`

## See Also

- [Emergency Signature Verification](../src/validation/emergency.rs)
- [bllvm-sdk Governance Keys](../../bllvm-sdk/src/governance/keys.rs)
- [Key Manager CLI](../src/bin/key_manager.rs)

