#!/bin/bash
#
# Key Generation Ceremony Script
#
# Facilitates secure key generation for maintainers with proper documentation
# and verification. This script guides maintainers through the key generation
# process and creates proper documentation.
#
# Usage:
#   key-ceremony.sh --maintainer-id maintainer_1 --output /path/to/keys
#
# Environment variables:
#   KEY_CEREMONY_OUTPUT_DIR - Output directory for keys
#   KEY_CEREMONY_MAINTAINER_ID - Maintainer identifier

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CEREMONY_DIR="${SCRIPT_DIR}/../ceremonies"

# Defaults
MAINTAINER_ID="${KEY_CEREMONY_MAINTAINER_ID:-}"
OUTPUT_DIR="${KEY_CEREMONY_OUTPUT_DIR:-./keys}"
KEY_TYPE="maintainer"
VERIFY_ONLY=false

print_usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Key Generation Ceremony Script for BTCDecoded Governance System.

This script guides maintainers through secure key generation with proper
documentation and verification.

Options:
  --maintainer-id ID     Maintainer identifier (required)
  --output DIR           Output directory for keys (default: ./keys)
  --key-type TYPE        Key type: maintainer, emergency, economic_node (default: maintainer)
  --verify-only          Only verify existing keys, don't generate new ones
  --help                 Show this help message

Environment variables:
  KEY_CEREMONY_MAINTAINER_ID - Maintainer identifier
  KEY_CEREMONY_OUTPUT_DIR - Output directory for keys

Examples:
  # Generate maintainer key
  ./key-ceremony.sh --maintainer-id maintainer_1 --output ./keys

  # Generate emergency keyholder key
  ./key-ceremony.sh --maintainer-id emergency_1 --key-type emergency --output ./keys

  # Verify existing keys
  ./key-ceremony.sh --maintainer-id maintainer_1 --verify-only
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --maintainer-id)
            MAINTAINER_ID="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --key-type)
            KEY_TYPE="$2"
            shift 2
            ;;
        --verify-only)
            VERIFY_ONLY=true
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            print_usage
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$MAINTAINER_ID" ]]; then
    echo "Error: --maintainer-id or KEY_CEREMONY_MAINTAINER_ID required" >&2
    exit 1
fi

# Create output directory
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${CEREMONY_DIR}"

echo "=== Key Generation Ceremony ==="
echo "Maintainer ID: ${MAINTAINER_ID}"
echo "Key Type: ${KEY_TYPE}"
echo "Output Directory: ${OUTPUT_DIR}"
echo ""

# Check for required tools
if ! command -v bllvm-keygen >/dev/null 2>&1; then
    echo "Error: bllvm-keygen not found. Please build bllvm-sdk first." >&2
    exit 1
fi

if [[ "$VERIFY_ONLY" == "true" ]]; then
    echo "=== Verifying Existing Keys ==="
    
    KEY_FILE="${OUTPUT_DIR}/${MAINTAINER_ID}.key"
    if [[ ! -f "$KEY_FILE" ]]; then
        echo "Error: Key file not found: ${KEY_FILE}" >&2
        exit 1
    fi
    
    echo "Key file found: ${KEY_FILE}"
    
    # Verify key format
    if ! jq -e '.public_key' "$KEY_FILE" >/dev/null 2>&1; then
        echo "Error: Invalid key file format" >&2
        exit 1
    fi
    
    PUBLIC_KEY=$(jq -r '.public_key' "$KEY_FILE")
    echo "Public Key: ${PUBLIC_KEY}"
    echo ""
    echo "✅ Key verification successful"
    exit 0
fi

# Generate key
echo "=== Step 1: Key Generation ==="
echo "Generating keypair for ${MAINTAINER_ID}..."

KEY_FILE="${OUTPUT_DIR}/${MAINTAINER_ID}.key"
PUBLIC_KEY_FILE="${OUTPUT_DIR}/${MAINTAINER_ID}.pub"

# Generate keypair
bllvm-keygen \
    --output "${KEY_FILE}" \
    --format json || {
    echo "Error: Failed to generate keypair" >&2
    exit 1
}

# Extract public key
PUBLIC_KEY=$(jq -r '.public_key' "${KEY_FILE}")
echo "${PUBLIC_KEY}" > "${PUBLIC_KEY_FILE}"

echo "✅ Keypair generated successfully"
echo "Private key: ${KEY_FILE}"
echo "Public key: ${PUBLIC_KEY_FILE}"
echo ""

# Verify key
echo "=== Step 2: Key Verification ==="
echo "Verifying generated keypair..."

# Check key format
if ! jq -e '.public_key' "${KEY_FILE}" >/dev/null 2>&1; then
    echo "Error: Invalid key file format" >&2
    exit 1
fi

if ! jq -e '.secret_key' "${KEY_FILE}" >/dev/null 2>&1; then
    echo "Error: Secret key missing from key file" >&2
    exit 1
fi

echo "✅ Key format verification passed"
echo ""

# Create ceremony documentation
echo "=== Step 3: Ceremony Documentation ==="
CEREMONY_LOG="${CEREMONY_DIR}/ceremony-${MAINTAINER_ID}-$(date +%Y%m%d-%H%M%S).log"

cat > "${CEREMONY_LOG}" <<EOF
# Key Generation Ceremony Log

**Date**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Maintainer ID**: ${MAINTAINER_ID}
**Key Type**: ${KEY_TYPE}
**Ceremony Script Version**: 1.0

## Key Information

- **Public Key**: ${PUBLIC_KEY}
- **Key File**: ${KEY_FILE}
- **Public Key File**: ${PUBLIC_KEY_FILE}

## Ceremony Steps

1. ✅ Key generation completed
2. ✅ Key format verification passed
3. ✅ Ceremony documentation created

## Security Notes

- Private key stored in: ${KEY_FILE}
- Public key stored in: ${PUBLIC_KEY_FILE}
- Private key must be kept secure and never shared
- Public key should be shared with governance system administrators

## Next Steps

1. Verify key file permissions: chmod 600 ${KEY_FILE}
2. Create secure backup of private key
3. Share public key with governance system administrators
4. Register public key in governance system

## Verification

To verify this key later:
  ./key-ceremony.sh --maintainer-id ${MAINTAINER_ID} --verify-only
EOF

echo "Ceremony log: ${CEREMONY_LOG}"
echo ""

# Set secure permissions
echo "=== Step 4: Setting Permissions ==="
chmod 600 "${KEY_FILE}"
chmod 644 "${PUBLIC_KEY_FILE}"
echo "✅ Permissions set"
echo ""

# Create backup instructions
echo "=== Step 5: Backup Instructions ==="
cat > "${OUTPUT_DIR}/${MAINTAINER_ID}-backup-instructions.txt" <<EOF
# Key Backup Instructions for ${MAINTAINER_ID}

## Important Security Information

**Private Key Location**: ${KEY_FILE}
**Public Key Location**: ${PUBLIC_KEY_FILE}

## Backup Requirements

1. **Create Encrypted Backup**
   - Encrypt the private key file before backing up
   - Use strong encryption (e.g., GPG, age, or similar)
   - Store backup in secure location (HSM, encrypted storage, etc.)

2. **Backup Locations**
   - Primary backup: [Specify location]
   - Secondary backup: [Specify location]
   - Recovery backup: [Specify location]

3. **Backup Verification**
   - Test backup restoration process
   - Verify backup integrity regularly
   - Document backup restoration procedure

## Key Rotation Schedule

- **Maintainer keys**: Every 6 months
- **Emergency keys**: Every 3 months
- **Economic node keys**: Every 6 months

## Emergency Procedures

If key is compromised:
1. Immediately revoke key in governance system
2. Generate new keypair using this ceremony script
3. Update governance system with new public key
4. Document incident in governance audit log

## Contact Information

For key-related issues, contact:
- Governance administrators: [Contact info]
- Security team: [Contact info]
EOF

echo "Backup instructions: ${OUTPUT_DIR}/${MAINTAINER_ID}-backup-instructions.txt"
echo ""

echo "=== Ceremony Complete ==="
echo ""
echo "✅ Key generation ceremony completed successfully"
echo ""
echo "Next steps:"
echo "1. Review backup instructions: ${OUTPUT_DIR}/${MAINTAINER_ID}-backup-instructions.txt"
echo "2. Create secure backup of private key"
echo "3. Share public key with governance system administrators"
echo "4. Register public key in governance system"
echo ""
echo "⚠️  IMPORTANT: Keep private key secure and never share it!"


