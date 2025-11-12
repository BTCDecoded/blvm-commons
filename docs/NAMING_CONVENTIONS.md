# Naming Conventions

**Date**: Latest Update

## Overview

This document defines the naming conventions for Bitcoin Commons BLLVM (Bitcoin Low-Level Virtual Machine), clarifying the distinction between:
- **Technical name**: BLLVM (software/implementation)
- **Governance/branding name**: Bitcoin Commons
- **Repository organization**: BTCDecoded (GitHub organization)

## Executive Summary

- **Executables**: Use `bllvm-*` prefix (short, technical)
- **Package names**: Use `bitcoin-commons-bllvm` or `bitcoin-commons`
- **Branding**: Use "Bitcoin Commons" for user-facing documentation
- **Technical docs**: Use "BLLVM" for implementation details
- **Repository URLs**: Keep as `BTCDecoded` (historical/organizational)

## Binary Naming

### Main Node Executable
- **Binary name**: `bllvm`
- **Display name**: "Bitcoin Commons BLLVM" or "BLLVM Node"
- **Installation path**:
  - Linux: `/usr/bin/bllvm` or `/usr/local/bin/bllvm`
  - macOS: `/usr/local/bin/bllvm` or in app bundle
  - Windows: `C:\Program Files\Bitcoin Commons\bllvm.exe`

### CLI Tools (Developer SDK)
- `bllvm-keygen` - Generate governance keypairs
- `bllvm-sign` - Sign governance messages
- `bllvm-verify` - Verify signatures and multisig thresholds

### Governance Tools
- `bitcoin-commons-governance` or `bc-governance` - Governance application
- `key-manager` - Key management utility (internal tool, keep as-is)

## Package Naming

### Linux Packages
- **RPM**: `bitcoin-commons-bllvm`
- **DEB**: `bitcoin-commons-bllvm`
- **Package description**: "Bitcoin Commons BLLVM - Bitcoin Low-Level Virtual Machine implementation"

### macOS Packages
- **App bundle**: `Bitcoin Commons BLLVM.app`
- **Installer**: `BitcoinCommonsBLLVM.pkg`

### Windows Packages
- **Installer**: `BitcoinCommonsBLLVM-Setup.exe`
- **Installation directory**: `C:\Program Files\Bitcoin Commons`

## Directory Naming

### Source Code Directories
- Repository root: `BTCDecoded/` (keep as-is for GitHub organization)
- Component directories: Keep technical names
  - `bllvm-consensus/`
  - `bllvm-protocol/`
  - `bllvm-node/` (consider renaming to `bllvm-node/` in future)
  - `bllvm-sdk/`
  - `governance-app/`

### Installation Directories

#### Linux
- Configuration: `/etc/bitcoin-commons/` or `~/.config/bitcoin-commons/`
- Data: `~/.bitcoin-commons/` or `/var/lib/bitcoin-commons/`
- Logs: `/var/log/bitcoin-commons/`

#### macOS
- Application: `/Applications/Bitcoin Commons BLLVM.app`
- Configuration: `~/Library/Application Support/Bitcoin Commons/`
- Data: `~/Library/Application Support/Bitcoin Commons/`

#### Windows
- Application: `C:\Program Files\Bitcoin Commons\`
- Configuration: `%APPDATA%\Bitcoin Commons\`
- Data: `%LOCALAPPDATA%\Bitcoin Commons\`

## Documentation Naming

### User Documentation
- Use "Bitcoin Commons" as primary branding
- Example: "Bitcoin Commons BLLVM Installation Guide"
- Website: "Bitcoin Commons" or "Bitcoin Commons BLLVM"

### Technical Documentation
- Use "BLLVM" for implementation details
- Example: "BLLVM Architecture", "BLLVM API Reference"
- Code comments: Use "BLLVM" or "Bitcoin Commons BLLVM"

### Governance Documentation
- Use "Bitcoin Commons" exclusively
- Example: "Bitcoin Commons Governance Model"
- Files in `governance/` directory use "Bitcoin Commons"

## Branding Guidelines

### When to Use "Bitcoin Commons"
- User-facing documentation
- Installer packages
- Website and marketing materials
- Governance documentation
- Error messages (user-friendly)
- Application display names

### When to Use "BLLVM"
- Technical documentation
- Code comments
- API documentation
- Internal tooling
- Development workflows
- Binary executable names

### When to Use "BTCDecoded"
- GitHub repository URLs (keep unchanged)
- Git remotes
- Issue tracking
- Historical references

## Examples

### Correct Usage

**Executables**:
```bash
bllvm --help
bllvm-keygen --output key.pem
bllvm-sign release --version v1.0.0
```

**Package Installation**:
```bash
# Debian/Ubuntu
apt install bitcoin-commons-bllvm

# RPM
yum install bitcoin-commons-bllvm

# macOS
brew install bitcoin-commons-bllvm  # If available via Homebrew
```

**Documentation**:
- "Bitcoin Commons BLLVM User Guide"
- "BLLVM Architecture Documentation"
- "Bitcoin Commons Governance Process"

### Incorrect Usage (Do Not Use)

- ❌ `btcdecoded` as executable name (use `bllvm`)
- ❌ "BTCDecoded" in user-facing materials (use "Bitcoin Commons")
- ❌ "Bitcoin Commons" in technical API docs (use "BLLVM" where appropriate)
- ❌ Changing repository URLs from `BTCDecoded` (keep as-is)

## URI Scheme Registration

For BIP21 URI scheme registration:
- **Application name**: "Bitcoin Commons BLLVM"
- **Executable**: `bllvm`
- **Desktop entry**: `bitcoin-commons-bllvm-bitcoin.desktop`
- **Registry keys**: `HKEY_CLASSES_ROOT\bitcoin` (Windows)

## Migration Notes

### Already Updated
- ✅ Binary names: `bllvm-*` (renamed from `btcdecoded-*`)
- ✅ Package authors: "Bitcoin Commons Team" (renamed from "BTCDecoded Team")
- ✅ BIP21 installer guide: Uses "Bitcoin Commons BLLVM"
- ✅ Build scripts: Updated to use new binary names

### Future Considerations
- Consider renaming `bllvm-node/` to `bllvm-node/` (breaking change)
- Update website branding to "Bitcoin Commons" consistently
- Create migration guide for users switching from old names

## Repository URLs (Keep Unchanged)

All repository URLs remain as `BTCDecoded`:
- `https://github.com/BTCDecoded/bllvm-consensus`
- `https://github.com/BTCDecoded/bllvm-node`
- `https://github.com/BTCDecoded/bllvm-sdk`
- `https://github.com/BTCDecoded/governance-app`
- `https://github.com/BTCDecoded/bllvm-protocol`

This maintains consistency with existing GitHub organization structure and avoids breaking existing links/clones.

