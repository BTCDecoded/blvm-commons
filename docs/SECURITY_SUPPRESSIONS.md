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
| RUSTSEC-2026-0118 | hickory-proto 0.25.2 via blvm-node → iroh 0.95; no fixed release. Time-boxed until blvm-node's iroh 1.x upgrade |
| RUSTSEC-2026-0119 | hickory-proto 0.25.2 via iroh 0.95 (fix is 0.26.1, which iroh 0.95 cannot take). Time-boxed until iroh 1.x |
| RUSTSEC-2026-0338 | quinn-proto 0.11.15 (fix >=0.11.18); iroh 0.95's crypto_box pins chacha20 0.10.0-rc.2, which blocks quinn-proto >=0.11.16. Time-boxed until iroh 1.x |
| RUSTSEC-2026-0339 | quinn-proto 0.11.15 (fix >=0.11.17); same iroh 0.95 constraint. Time-boxed until iroh 1.x |
| RUSTSEC-2026-0340 | quinn-proto 0.11.15 (fix >=0.11.18); same iroh 0.95 constraint. Time-boxed until iroh 1.x |
| RUSTSEC-2026-0341 | quinn-proto 0.11.15 (fix >=0.11.17); same iroh 0.95 constraint. Time-boxed until iroh 1.x |
| RUSTSEC-2026-0342 | quinn-proto 0.11.15 (fix >=0.11.17); same iroh 0.95 constraint. Time-boxed until iroh 1.x |

Refresh this table whenever the workflow ignore list changes.
