# Security Vulnerability Suppressions

This document explains why certain security vulnerabilities reported by `cargo audit` are suppressed in `bllvm-commons`.

## Suppressed Vulnerabilities

### 1. opentimestamps Transitive Dependencies (5 vulnerabilities)

All of these come from the `opentimestamps 0.1.2` crate, which is used for Bitcoin-anchored timestamping.

#### RUSTSEC-2022-0011: rust-crypto 0.2.36
- **Reason**: Transitive dependency from opentimestamps. Our OTS client implementation uses `sha2` directly, not `rust-crypto`. The vulnerability is in AES encryption which we don't use.
- **Status**: No alternative OTS implementation available. Monitoring for opentimestamps updates.

#### RUSTSEC-2022-0004: rustc-serialize 0.3.25
- **Reason**: Transitive dependency from opentimestamps via rust-crypto. Not directly used in our code paths.
- **Status**: No fix available. Only affects deeply nested JSON parsing which we don't use.

#### RUSTSEC-2022-0013: regex 0.2.11
- **Reason**: Transitive dependency from opentimestamps via env_logger. Only used for logging, not in security-critical paths.
- **Status**: Requires opentimestamps update. Not exploitable in logging context.

#### RUSTSEC-2022-0006: thread_local 0.3.6
- **Reason**: Transitive dependency from opentimestamps via regex. Not directly used in our code.
- **Status**: Requires opentimestamps update. Data race in Iter/IterMut not used by our code.

#### RUSTSEC-2020-0071: time 0.1.45
- **Reason**: Transitive dependency from opentimestamps via rust-crypto. Not directly used in our code.
- **Status**: Requires opentimestamps update. Potential segfault not in our code paths.

### 2. idna 0.5.0 (RUSTSEC-2024-0421)

- **Source**: Transitive dependency from `nostr-sdk 0.27` via `url-fork`
- **Reason**: 
  - We control all URLs used in our Nostr implementation
  - The vulnerability is in Punycode label decoding, which is not exploitable when we control the URLs
  - `nostr-sdk` is actively maintained and this will be fixed in a future release
- **Status**: Monitoring `nostr-sdk` for updates that fix this

### 3. rsa 0.9.8 (RUSTSEC-2023-0071)

- **Source**: Transitive dependency from `sqlx-mysql`
- **Reason**:
  - We **do not use MySQL** - only SQLite and PostgreSQL are supported
  - The vulnerability is a timing sidechannel attack that only affects MySQL connections
  - Even though `sqlx` may pull in `sqlx-mysql` transitively, we never establish MySQL connections
- **Status**: Not exploitable in our deployment. MySQL feature is not enabled.

## Verification

To run security audit with suppressions:

```bash
./.cargo-audit-ignore.sh
```

Or manually:

```bash
cargo audit \
  --ignore RUSTSEC-2022-0011 \
  --ignore RUSTSEC-2022-0004 \
  --ignore RUSTSEC-2022-0013 \
  --ignore RUSTSEC-2022-0006 \
  --ignore RUSTSEC-2020-0071 \
  --ignore RUSTSEC-2024-0421 \
  --ignore RUSTSEC-2023-0071
```

## Review Schedule

This document should be reviewed:
- When `nostr-sdk` releases a new version (check for idna fix)
- When `opentimestamps` releases a new version (check for dependency updates)
- When `sqlx` releases a new version (check if MySQL can be fully excluded)
- Quarterly to ensure suppressions remain valid

## Last Updated

2025-01-20

