#!/bin/bash
# Security audit with suppressions for bllvm-commons
# These vulnerabilities have been analyzed and are either:
# 1. Not exploitable in our use case
# 2. In transitive dependencies with no available fixes
# 3. In optional features we don't use

cargo audit \
  --ignore RUSTSEC-2022-0011 \
  --ignore RUSTSEC-2022-0004 \
  --ignore RUSTSEC-2022-0013 \
  --ignore RUSTSEC-2022-0006 \
  --ignore RUSTSEC-2020-0071 \
  --ignore RUSTSEC-2024-0421 \
  --ignore RUSTSEC-2023-0071
