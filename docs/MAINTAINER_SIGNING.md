# Maintainer Signing Guide

This guide explains how maintainers can sign pull requests for governance approval.

## Overview

The governance system requires maintainers to cryptographically sign pull requests to approve them. This ensures that only authorized maintainers can approve changes and provides non-repudiation.

## Signing Process

### 1. Install the Signing Tool

The `sign-pr` CLI tool is included in the governance-app:

```bash
cargo build --release --bin sign-pr
```

### 2. Generate Your Keypair

If you don't have a keypair yet:

```bash
./target/release/sign-pr generate --username your-github-username --output ./keys
```

This creates:
- `your-github-username_private.pem` - Your private key (keep secret!)
- `your-github-username_public.pem` - Your public key (for database)

### 3. Sign a Pull Request

To sign PR #123 in the `btcdecoded/governance` repository:

```bash
./target/release/sign-pr sign \
  --key ./keys/your-github-username_private.pem \
  --repo btcdecoded/governance \
  --pr 123
```

This will output a command like:
```
/governance-sign 3045022100a1b2c3d4e5f6...
```

### 4. Post the Signature

Copy the `/governance-sign` command and post it as a comment on the pull request. The governance-app will automatically verify your signature and count it toward the required threshold.

## Verification

### Verify Your Own Signature

```bash
./target/release/sign-pr verify \
  --public-key ./keys/your-github-username_public.pem \
  --message "PR #123 in btcdecoded/governance" \
  --signature "3045022100a1b2c3d4e5f6..."
```

### Check PR Status

The governance-app posts status checks showing:
- Current signature count vs required
- Which maintainers have signed
- Whether the PR can be merged

## Security Best Practices

### Private Key Security
- Store private keys in a secure location
- Use strong file permissions: `chmod 600 your-key_private.pem`
- Consider using hardware security modules for production
- Never share private keys or commit them to repositories

### Signature Verification
- Always verify signatures before posting
- Check that the message matches the PR you intend to sign
- Verify the signature was generated correctly

### Key Rotation
- Rotate keys periodically (annually recommended)
- Update the database when rotating keys
- Coordinate with other maintainers for smooth transitions

## Troubleshooting

### Common Issues

**"User is not a registered maintainer"**
- Your GitHub username must be in the maintainers database
- Contact the governance administrator to add you

**"Signature verification failed"**
- Check that you're using the correct private key
- Verify the message matches exactly
- Ensure the keypair was generated correctly

**"Invalid signature format"**
- The signature must be in the correct format
- Try regenerating the signature
- Check for copy/paste errors

### Getting Help

- Check the governance-app logs for detailed error messages
- Verify your keypair with the verification command
- Contact the governance team for assistance

## Integration with GitHub

The governance-app automatically:
- Monitors PR comments for `/governance-sign` commands
- Verifies signatures against maintainer public keys
- Updates status checks with signature progress
- Blocks merges until requirements are met

## Production Considerations

For production use:
- Use hardware security modules for key storage
- Implement proper key rotation procedures
- Set up monitoring for signature verification failures
- Consider multi-signature schemes for high-value changes
