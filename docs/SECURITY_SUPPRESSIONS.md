# Security audit suppressions (blvm-commons)

This file is referenced by **`blvm-commons/.github/workflows/ci.yml`** (`cargo audit --ignore …`).  
Suppressions are **temporary**: they apply to **transitive** crates (e.g. optional stacks such as opentimestamps, nostr-sdk, sqlx-mysql) until upstream releases remove the vulnerable versions from the resolved graph.

## Resolution process

1. After dependency bumps, run `cargo audit` **without** `--ignore` locally.
2. For each reported `RUSTSEC`, record: advisory ID, crate name, version path (`cargo tree -i`), exploitability in **this** deployment, and upstream tracking link or issue.
3. Remove `--ignore` lines from CI when the lockfile no longer pulls the vulnerable version.

## Current ignores (CI)

| Advisory | Notes |
|----------|--------|
| RUSTSEC-2022-0011 | Transitive; verify on bump |
| RUSTSEC-2022-0004 | Transitive; verify on bump |
| RUSTSEC-2022-0013 | Transitive; verify on bump |
| RUSTSEC-2022-0006 | Transitive; verify on bump |
| RUSTSEC-2020-0071 | Transitive; verify on bump |
| RUSTSEC-2024-0421 | Transitive; verify on bump |
| RUSTSEC-2023-0071 | Transitive; verify on bump |
| RUSTSEC-2026-0459 | `sqlx` 0.8.6 requires `libsqlite3-sys` ^0.30.1 (SQLite CVE-2026-119375). `sqlx` 0.9 needs rustc 1.94; this crate's toolchain is 1.91. The `sqlite` feature does not enable `bundled`, so the process links the system libsqlite3. Drop this ignore when the toolchain can take `sqlx` 0.9 and `libsqlite3-sys` >=0.39. |

Refresh this table whenever the workflow ignore list changes.
