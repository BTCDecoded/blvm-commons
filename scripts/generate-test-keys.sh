#!/bin/bash
# Generate test maintainer keys for Phase 2A testing
# This script generates test keypairs for 7 maintainers across different layers

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
KEYS_DIR="$PROJECT_ROOT/test-keys"
MAINTAINERS_FILE="$PROJECT_ROOT/scripts/populate-test-maintainers.sql"

echo "🔑 Generating test maintainer keys for Phase 2A..."

# Create keys directory
mkdir -p "$KEYS_DIR"

# Generate keys for 7 test maintainers
maintainers=(
    "alice:1"
    "bob:1" 
    "charlie:2"
    "dave:2"
    "eve:3"
    "frank:3"
    "grace:3"
)

echo "-- Test maintainer keys for Phase 2A" > "$MAINTAINERS_FILE"
echo "-- Generated on $(date)" >> "$MAINTAINERS_FILE"
echo "" >> "$MAINTAINERS_FILE"

for maintainer_info in "${maintainers[@]}"; do
    IFS=':' read -r username layer <<< "$maintainer_info"
    
    echo "Generating keypair for $username (Layer $layer)..."
    
    # Generate Ed25519 keypair using openssl
    private_key_file="$KEYS_DIR/${username}_private.pem"
    public_key_file="$KEYS_DIR/${username}_public.pem"
    
    # Generate private key
    openssl genpkey -algorithm Ed25519 -out "$private_key_file" 2>/dev/null
    
    # Extract public key
    openssl pkey -in "$private_key_file" -pubout -out "$public_key_file" 2>/dev/null
    
    # Get public key in hex format (for database storage)
    public_key_hex=$(openssl pkey -in "$private_key_file" -pubout -outform DER 2>/dev/null | xxd -p -c 256)
    
    # Store in SQL file
    echo "INSERT INTO maintainers (github_username, public_key, layer, active, last_updated) VALUES" >> "$MAINTAINERS_FILE"
    echo "('$username', '$public_key_hex', $layer, true, CURRENT_TIMESTAMP);" >> "$MAINTAINERS_FILE"
    echo "" >> "$MAINTAINERS_FILE"
    
    echo "✅ Generated keypair for $username"
done

echo ""
echo "🔑 Test key generation complete!"
echo "📁 Keys stored in: $KEYS_DIR"
echo "📄 SQL file created: $MAINTAINERS_FILE"
echo ""
echo "To load these keys into the database, run:"
echo "  sqlite3 governance-app.db < $MAINTAINERS_FILE"
echo ""
echo "⚠️  WARNING: These are TEST keys only! Do not use in production."
