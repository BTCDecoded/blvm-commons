# Verification Bundle Format Specification

## Overview

Verification bundles provide cryptographic proof that binaries match verified code (Kani proofs). This document specifies the format and structure of verification bundles.

## Bundle Structure

A verification bundle consists of:

1. **Bundle Archive** (`verify-artifacts.tar.gz`) - Contains verification artifacts
2. **Bundle Metadata** (`verify-artifacts.json`) - JSON metadata with hashes and proof results
3. **Bundle Signature** (`verify-artifacts.json.sig`) - Maintainer multisig signature of metadata
4. **Bundle Checksum** (`verify-artifacts.tar.gz.sha256`) - SHA256 checksum of archive
5. **OpenTimestamps Proof** (`verify-artifacts.tar.gz.ots`) - Optional Bitcoin blockchain timestamp

## Bundle Archive Contents

The `verify-artifacts.tar.gz` archive contains:

```
verify-artifacts/
├── tests.log              # Test execution results
├── kani.log               # Kani proof results
├── cargo_metadata.json    # Cargo metadata
└── (other artifacts)
```

## Bundle Metadata JSON Schema

The `verify-artifacts.json` file follows this schema:

```json
{
  "version": "1.0",
  "bundle_type": "consensus-proof",
  "created_at": "2024-01-15T10:30:00Z",
  "source_repo": "bllvm-consensus",
  "source_commit": "abc123def456...",
  "source_hash": "sha256:abc123...",
  "build_config_hash": "sha256:def456...",
  "spec_hash": "sha256:ghi789...",
  "bundle_hash": "sha256:jkl012...",
  "verification_results": {
    "tests": {
      "status": "passed",
      "test_count": 100,
      "passed": 100,
      "failed": 0,
      "log_file": "tests.log"
    },
    "kani": {
      "status": "verified",
      "proof_count": 60,
      "verified": 60,
      "failed": 0,
      "log_file": "kani.log"
    }
  },
  "artifacts": {
    "bundle_archive": "verify-artifacts.tar.gz",
    "bundle_hash": "sha256:jkl012...",
    "checksum_file": "verify-artifacts.tar.gz.sha256"
  },
  "signatures": {
    "bundle_metadata": {
      "signatures": [
        {
          "signer": "maintainer1",
          "signature": "3045022100...",
          "public_key": "02...",
          "signed_at": "2024-01-15T10:30:00Z"
        }
      ],
      "threshold": "6-of-7",
      "threshold_met": true
    }
  },
  "opentimestamps": {
    "proof_file": "verify-artifacts.tar.gz.ots",
    "status": "pending|confirmed",
    "block_height": 12345
  }
}
```

## Signature Requirements

### Consensus Binaries (Layer 1-2)
- **Threshold**: 6-of-7 maintainer signatures
- **Required for**: Orange Paper, Consensus Proof binaries

### Protocol Binaries (Layer 3)
- **Threshold**: 4-of-5 maintainer signatures
- **Required for**: Protocol Engine binaries

### Application Binaries (Layer 4-5)
- **Threshold**: 3-of-5 maintainer signatures
- **Required for**: Reference Node, Developer SDK binaries

## Verification Process

### User Verification

1. **Download Artifacts**:
   - Binary file
   - SHA256SUMS file
   - Verification bundle (`verify-artifacts.tar.gz`)
   - Bundle metadata (`verify-artifacts.json`)
   - Bundle signature (`verify-artifacts.json.sig`)

2. **Verify SHA256SUMS**:
   ```bash
   sha256sum -c SHA256SUMS
   ```

3. **Verify Bundle Signature**:
   ```bash
   bllvm-verify-binary bundle \
     --file verify-artifacts.tar.gz \
     --signatures verify-artifacts.json.sig \
     --pubkeys maintainer-keys.json \
     --threshold 6-of-7
   ```

4. **Verify Bundle Contents**:
   - Extract bundle: `tar -xzf verify-artifacts.tar.gz`
   - Verify source hash matches git commit
   - Verify Kani proofs passed (from `kani.log`)
   - Verify tests passed (from `tests.log`)

5. **Verify Binary Matches Bundle**:
   - Verify binary hash matches SHA256SUMS
   - Verify bundle metadata references correct binary hash

### Automated Verification

Node software can verify on startup (optional, opt-in):

1. Check for verification bundle
2. Verify bundle signature
3. Verify bundle contents
4. Verify binary hash matches bundle metadata
5. Reject if any verification fails

## OpenTimestamps Integration

Verification bundles can be anchored to the Bitcoin blockchain via OpenTimestamps:

1. **Stamping**: `ots stamp verify-artifacts.tar.gz`
2. **Verification**: `ots verify verify-artifacts.tar.gz.ots`
3. **Upgrade**: `ots upgrade verify-artifacts.tar.gz.ots`

This provides immutable proof that verification state existed at a specific time.

## Example Usage

### Creating a Verification Bundle

```bash
# Create bundle
./make_verification_bundle.sh --repo /path/to/bllvm-consensus

# Sign bundle metadata
bllvm-sign-binary bundle \
  --file verify-artifacts.tar.gz \
  --source-hash $(git rev-parse HEAD) \
  --key maintainer-key.json \
  --output verify-artifacts.json.sig

# Aggregate signatures (when multiple maintainers sign)
bllvm-aggregate-signatures \
  --signatures sig1.json,sig2.json,sig3.json \
  --output verify-artifacts.json.sig
```

### Verifying a Verification Bundle

```bash
# Verify bundle signature
bllvm-verify-binary bundle \
  --file verify-artifacts.tar.gz \
  --signatures verify-artifacts.json.sig \
  --pubkeys maintainer-keys.json \
  --threshold 6-of-7

# Verify bundle contents
tar -xzf verify-artifacts.tar.gz
grep "VERIFICATION SUCCESSFUL" verify-artifacts/kani.log
grep "test result: ok" verify-artifacts/tests.log
```

## Security Considerations

1. **Signature Verification**: Always verify signatures before trusting bundle contents
2. **Hash Verification**: Always verify hashes match expected values
3. **Source Verification**: Verify source hash matches git commit
4. **Proof Verification**: Verify Kani proofs actually passed
5. **Timestamp Verification**: Verify OpenTimestamps proof if available

## Related Documentation

- [Binary Signing Guide](BINARY_SIGNING_GUIDE.md) - How to sign binaries
- [Verification Guide](VERIFICATION_GUIDE.md) - How to verify binaries
- [Release Process](RELEASE_PROCESS.md) - Complete release workflow


