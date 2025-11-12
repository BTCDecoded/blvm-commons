# Documentation Commit Instructions

Since BTCDecoded is NOT a monorepo, documentation updates need to be committed to the appropriate repositories.

## Organization-Level Documentation

The following files are organization-level and should be committed to a main BTCDecoded repository (if it exists) or the `commons` repository:

- `SYSTEM_STATUS.md` - Master status document
- `DOCUMENTATION_AUDIT_CATALOG.md` - Document inventory
- `docs/DOCUMENTATION_AUDIT_REPORT.md` - Audit report
- `GAP_ANALYSIS.md` - Gap analysis
- `README.md` - Updated with reference to SYSTEM_STATUS.md
- `SYSTEM_OVERVIEW.md` - Updated with reference to SYSTEM_STATUS.md
- `docs/CURRENT_STATUS.md` - Deprecated notice added
- `docs/FORMAL_VERIFICATION_STATUS.md` - Deprecated notice added
- `docs/FORMAL_VERIFICATION_STATUS_FINAL.md` - Deprecated notice added
- `docs/FORMAL_VERIFICATION_99_PERCENT_ACHIEVED.md` - Deprecated notice added

## Component-Specific Updates

Each component repository should have its README.md checked for consistency with the master status.

## Commit Message Template

```
docs: Update documentation with verified system status

- Add master status document (SYSTEM_STATUS.md)
- Resolve formal verification conflicts (verified: 176 proofs)
- Add deprecation notices to conflicting documents
- Update primary documentation with references to master status

All status claims verified against actual codebase.
```

## Next Steps

1. Determine if there's a main BTCDecoded repository for organization-level docs
2. If not, commit to `commons` repository (build orchestrator)
3. Verify each component repo README is consistent
4. Push all changes

