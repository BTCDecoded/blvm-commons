# CI/CD Security Audit Configuration

## Overview

The `bllvm-commons` repository has documented security vulnerability suppressions. All CI/CD workflows that run `cargo audit` must use the suppression script to avoid false positives.

## Quick Answer

**Yes, workflows need to run `cargo audit` with suppressions.** Use the `.cargo-audit-ignore.sh` script.

## How to Run Security Audit in CI/CD

### Option 1: Use the Suppression Script (Recommended)

```yaml
- name: Run security audit
  run: |
    chmod +x .cargo-audit-ignore.sh
    ./.cargo-audit-ignore.sh
```

### Option 2: Direct Command with Suppressions

If the script isn't available, use the direct command:

```yaml
- name: Run security audit
  run: |
    cargo audit \
      --ignore RUSTSEC-2022-0011 \
      --ignore RUSTSEC-2022-0004 \
      --ignore RUSTSEC-2022-0013 \
      --ignore RUSTSEC-2022-0006 \
      --ignore RUSTSEC-2020-0071 \
      --ignore RUSTSEC-2024-0421 \
      --ignore RUSTSEC-2023-0071
```

### Option 3: Fallback Pattern (Used in Our Workflows)

```yaml
- name: Run security audit
  run: |
    if [ -f .cargo-audit-ignore.sh ]; then
      chmod +x .cargo-audit-ignore.sh
      ./.cargo-audit-ignore.sh
    else
      # Fallback to direct command with suppressions
      cargo audit \
        --ignore RUSTSEC-2022-0011 \
        --ignore RUSTSEC-2022-0004 \
        --ignore RUSTSEC-2022-0013 \
        --ignore RUSTSEC-2022-0006 \
        --ignore RUSTSEC-2020-0071 \
        --ignore RUSTSEC-2024-0421 \
        --ignore RUSTSEC-2023-0071
    fi
```

## Updated Workflows

The following workflows have been updated to use suppressions:

1. ✅ `.github/workflows/ci.yml` - Security audit job
2. ✅ `scripts/test_cross_layer_sync.sh` - Local test script

## Why Suppressions Are Needed

All 7 suppressed vulnerabilities are:
- From transitive dependencies with no available fixes
- Not exploitable in our use case (we don't use MySQL, we use sha2 not rust-crypto, etc.)
- Documented in `SECURITY_SUPPRESSIONS.md`

## Verification

To verify the suppressions work:

```bash
# Without suppressions (shows 7 vulnerabilities)
cargo audit

# With suppressions (shows 0 vulnerabilities, 5 unmaintained warnings)
./.cargo-audit-ignore.sh
```

## Documentation

- `SECURITY_SUPPRESSIONS.md` - Detailed analysis of each suppressed vulnerability
- `SECURITY_AUDIT_SUMMARY.md` - High-level summary
- `.cargo-audit-ignore.sh` - Suppression script

