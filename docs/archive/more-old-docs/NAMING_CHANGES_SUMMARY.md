# Naming Changes Summary

**Date**: Latest Update

## Overview

Updated naming conventions to align with Bitcoin Commons branding while maintaining technical clarity:
- **Technical name**: BLLVM (Bitcoin Low-Level Virtual Machine)
- **Governance/branding**: Bitcoin Commons
- **Repository organization**: BTCDecoded (kept unchanged)

## Changes Made

### ✅ Binary Names (Renamed)

**Developer SDK Tools**:
- `btcdecoded-keygen` → `bllvm-keygen`
- `btcdecoded-sign` → `bllvm-sign`
- `btcdecoded-verify` → `bllvm-verify`

**Files Renamed**:
- `bllvm-sdk/src/bin/btcdecoded-keygen.rs` → `bllvm-keygen.rs`
- `bllvm-sdk/src/bin/btcdecoded-sign.rs` → `bllvm-sign.rs`
- `bllvm-sdk/src/bin/btcdecoded-verify.rs` → `bllvm-verify.rs`

### ✅ Package Metadata (Updated)

**Cargo.toml Authors**:
- `["BTCDecoded Team"]` → `["Bitcoin Commons Team"]`
- `["BTCDecoded Contributors"]` → `["Bitcoin Commons Contributors"]`

**Cargo.toml Descriptions**:
- Added "Bitcoin Commons BLLVM:" prefix to descriptions
- Updated: `bllvm-consensus`, `bllvm-protocol`, `bllvm-node`, `bllvm-sdk`

### ✅ BIP21 Implementation (Updated)

**Test Cases**:
- Executable paths: `btcdecoded` → `bllvm`
- Application names: `BTCDecoded` → `Bitcoin Commons BLLVM`
- Installation directories: `BTCDecoded` → `Bitcoin Commons`

**Installer Guide**:
- Updated all examples to use `bllvm` executable
- Updated package names to `bitcoin-commons-bllvm`
- Updated desktop entry names

### ✅ Build System (Updated)

**Scripts**:
- `commons/build.sh`: Binary names updated
- `commons/scripts/collect-artifacts.sh`: Binary list updated, archive name changed
- `commons/scripts/create-release.sh`: Binary names and archive names updated
- `commons/versions.toml`: Binary names updated

**Archive Names**:
- `btcdecoded-<platform>.tar.gz` → `bitcoin-commons-bllvm-<platform>.tar.gz`

### ✅ Documentation (Updated)

**Developer SDK**:
- `README.md`: CLI tool names updated
- `src/lib.rs`: CLI tool references updated
- `docs/api-reference.md`: All CLI documentation updated

**Build Documentation**:
- `commons/docs/BUILD_SYSTEM.md`: Binary names and archive names updated
- `unified-build-and-release-system.plan.md`: Binary names updated
- `DESIGN.md`: File structure references updated

**Governance App**:
- `README.md`: Title updated to "Bitcoin Commons Governance App"
- `src/bin/security-gate.rs`: Tool description and output updated

### ✅ Other References

- Build system log messages updated
- Release notes templates updated
- Package installation examples updated

## Not Changed (As Requested)

### Repository URLs (Kept Unchanged)
- All `https://github.com/BTCDecoded/*` URLs remain unchanged
- Repository organization name: `BTCDecoded` (unchanged)
- Git remote URLs: Unchanged

### Directory Names
- `bllvm-node/` - Kept as-is (consider renaming to `bllvm-node/` in future)
- All other component directories: Unchanged

## Migration Impact

### For Users

**Breaking Changes**:
- CLI tools renamed: `btcdecoded-*` → `bllvm-*`
- Package names changed: Installation commands will differ
- Binary paths: Executables now at `/usr/bin/bllvm-*`

**Migration Steps**:
1. Update PATH if using full paths to old binaries
2. Update scripts that call `btcdecoded-*` commands
3. Reinstall packages to get new binary names

### For Developers

**No Breaking Changes**:
- Cargo crate names unchanged (`bllvm-sdk`, `bllvm-node`, etc.)
- API surface unchanged
- Repository URLs unchanged

**Build System**:
- Build scripts automatically use new binary names
- Release artifacts use new naming

## Files Modified

### Source Files
- `bllvm-sdk/src/bin/*.rs` (3 files renamed + content updated)
- `bllvm-sdk/Cargo.toml`
- `bllvm-node/src/bip21.rs` (tests and examples)
- `governance-app/src/bin/security-gate.rs`
- All `Cargo.toml` files (authors/descriptions)

### Build & Release
- `commons/build.sh`
- `commons/scripts/collect-artifacts.sh`
- `commons/scripts/create-release.sh`
- `commons/versions.toml`

### Documentation
- `bllvm-sdk/README.md`
- `bllvm-sdk/src/lib.rs`
- `bllvm-sdk/docs/api-reference.md`
- `bllvm-node/src/bip21_installer_guide.md`
- `commons/docs/BUILD_SYSTEM.md`
- `DESIGN.md`
- `governance-app/README.md`
- `unified-build-and-release-system.plan.md`

## Testing Status

- ✅ `bllvm-sdk` compiles successfully with new binary names
- ✅ BIP21 tests updated and should pass
- ⚠️ Full integration testing recommended before release

## Next Steps

1. **Update remaining references** in SECURITY.md files (cosmetic)
2. **Update website/documentation** to use "Bitcoin Commons" consistentl
3. **Update CI/CD workflows** if they reference binary names directly
4. **Consider renaming** `bllvm-node/` → `bllvm-node/` (future consideration)

