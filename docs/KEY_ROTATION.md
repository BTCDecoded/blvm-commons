# Key Rotation Guide

## Overview

The BTCDecoded Governance System implements automated key rotation to maintain security and prevent long-term compromise. This document describes the key rotation system, policies, and procedures.

## Rotation Policies

Keys are rotated according to the following schedule:

| Key Type | Rotation Period | Policy |
|----------|----------------|--------|
| Maintainer | 6 months | Routine maintainers rotate every 6 months |
| Emergency | 3 months | Emergency keyholders rotate every 3 months |
| Economic Node | 1 year | Economic nodes rotate annually |
| GitHub App | 3 months | GitHub App keys rotate every 3 months |
| System | 1 year | System keys rotate annually |

### Rotation Triggers

Keys are rotated:
- **Scheduled**: Automatically when approaching expiration (within 30 days)
- **On Compromise**: Immediately upon suspected compromise
- **Before Major Changes**: Before major governance changes
- **Manual**: On-demand via CLI command

## Usage

### Check for Keys Needing Rotation

```bash
# Check which keys need rotation
key-manager check-rotation --database-url sqlite://governance.db
```

### Manual Key Rotation

```bash
# Rotate a specific key
key-manager rotate --key-id maintainer_alice_20240101 --database-url sqlite://governance.db

# Rotate with new owner
key-manager rotate --key-id maintainer_alice_20240101 --new-owner alice_new --database-url sqlite://governance.db
```

### Automated Rotation

```bash
# Dry run (see what would be rotated)
key-manager auto-rotate --dry-run --database-url sqlite://governance.db

# Actually rotate all keys needing rotation
key-manager auto-rotate --database-url sqlite://governance.db
```

## What Happens During Rotation

When a key is rotated:

1. **New Key Generation**: A new keypair is generated with the same type and owner
2. **Registry Update**: 
   - For maintainer keys: Updates the `maintainers` table with the new public key
   - For emergency keys: Updates the `emergency_keyholders` table with the new public key
3. **Old Key Revocation**: The old key is marked as revoked in the `key_metadata` table
4. **Metadata Preservation**: Key metadata (owner, type, etc.) is preserved in the new key

## Automated Rotation Setup

### Cron Job (Recommended)

Set up a daily cron job to check and rotate keys:

```bash
# Add to crontab (crontab -e)
# Check for keys needing rotation daily at 2 AM
0 2 * * * /path/to/key-manager check-rotation --database-url sqlite:///path/to/governance.db >> /var/log/key-rotation.log 2>&1

# Auto-rotate keys weekly on Sunday at 3 AM
0 3 * * 0 /path/to/key-manager auto-rotate --database-url sqlite:///path/to/governance.db >> /var/log/key-rotation.log 2>&1
```

### Systemd Timer (Alternative)

Create a systemd service and timer:

**`/etc/systemd/system/key-rotation.service`**:
```ini
[Unit]
Description=BTCDecoded Key Rotation Service
After=network.target

[Service]
Type=oneshot
ExecStart=/path/to/key-manager auto-rotate --database-url sqlite:///path/to/governance.db
User=governance
Group=governance
```

**`/etc/systemd/system/key-rotation.timer`**:
```ini
[Unit]
Description=BTCDecoded Key Rotation Timer
Requires=key-rotation.service

[Timer]
OnCalendar=weekly
OnCalendar=Mon 03:00
Persistent=true

[Install]
WantedBy=timers.target
```

Enable and start:
```bash
sudo systemctl enable key-rotation.timer
sudo systemctl start key-rotation.timer
```

## Monitoring

### Key Statistics

```bash
# Get key statistics
key-manager stats --database-url sqlite://governance.db
```

### List Keys

```bash
# List all active keys
key-manager list --key-type maintainer --status active --database-url sqlite://governance.db

# List keys by owner
key-manager list --owner alice --database-url sqlite://governance.db
```

## Security Considerations

1. **Backup Before Rotation**: Always backup the database before running automated rotation
2. **Test First**: Use `--dry-run` to verify what will be rotated
3. **Monitor Logs**: Check rotation logs for errors
4. **Key Storage**: Ensure private keys are securely stored (HSM recommended for production)
5. **Access Control**: Limit access to the key rotation commands

## Troubleshooting

### Key Rotation Fails

If key rotation fails:

1. Check the error message for specific issues
2. Verify the key exists and is active: `key-manager get <key-id>`
3. Check database connectivity
4. Verify maintainer/emergency keyholder exists in respective tables

### Maintainer Registry Not Updated

If the maintainer registry isn't updated after rotation:

1. Verify the key type is `Maintainer` or `Emergency`
2. Check that the owner (github_username) exists in the `maintainers` or `emergency_keyholders` table
3. Verify the maintainer/emergency keyholder is marked as `active = true`

### Keys Not Detected for Rotation

If keys aren't being detected for rotation:

1. Check that keys have `status = 'active'`
2. Verify `expires_at` is set correctly (should be within 30 days)
3. Check the rotation period matches the key type

## Recent Improvements

### Fixed Issues (2025-01-XX)

1. **Rotation Periods**: Updated to match documented policy (6 months for maintainers, 3 months for emergency)
2. **Rotation Check Logic**: Fixed to properly detect keys approaching expiration (within 30 days)
3. **Registry Updates**: Added automatic maintainer and emergency keyholder registry updates during rotation
4. **Automated Rotation**: Added `auto-rotate` command for batch rotation

## See Also

- [Security Guide](../SECURITY.md) - General security practices
- [Maintainer Guide](../../governance/guides/MAINTAINER_GUIDE.md) - Maintainer procedures
- [Key Ceremony](../../docs/production/KEY_CEREMONY.md) - Initial key setup

