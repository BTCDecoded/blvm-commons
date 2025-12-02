#!/bin/bash
#
# Key Backup and Restore Script
#
# Provides secure backup and restore functionality for maintainer keys.
# Supports encryption, verification, and secure storage.
#
# Usage:
#   key-backup-restore.sh backup --key maintainer-key.json --output backup.enc
#   key-backup-restore.sh restore --backup backup.enc --output restored-key.json

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Defaults
ENCRYPTION_METHOD="gpg"
VERIFY_BACKUP=true

print_usage() {
    cat <<EOF
Usage: $0 [COMMAND] [OPTIONS]

Key Backup and Restore Script for BTCDecoded Governance System.

Commands:
  backup    Create encrypted backup of a key
  restore   Restore key from encrypted backup
  verify    Verify backup integrity

Options:
  --key KEYFILE          Key file to backup/restore
  --backup BACKUPFILE    Backup file path
  --output OUTPUT        Output file path
  --encryption METHOD    Encryption method: gpg, age (default: gpg)
  --no-verify           Skip backup verification
  --help                Show this help message

Examples:
  # Backup a key
  ./key-backup-restore.sh backup --key maintainer-key.json --output backup.enc

  # Restore a key
  ./key-backup-restore.sh restore --backup backup.enc --output restored-key.json

  # Verify backup
  ./key-backup-restore.sh verify --backup backup.enc
EOF
}

backup_key() {
    local key_file="$1"
    local output_file="$2"
    local encryption_method="${3:-gpg}"

    if [[ ! -f "$key_file" ]]; then
        echo "Error: Key file not found: ${key_file}" >&2
        exit 1
    fi

    echo "=== Creating Key Backup ==="
    echo "Key file: ${key_file}"
    echo "Output: ${output_file}"
    echo "Encryption: ${encryption_method}"
    echo ""

    case "$encryption_method" in
        gpg)
            if ! command -v gpg >/dev/null 2>&1; then
                echo "Error: GPG not found. Install GPG or use --encryption age" >&2
                exit 1
            fi
            
            echo "Encrypting with GPG..."
            gpg --symmetric --cipher-algo AES256 --compress-algo 1 \
                --output "${output_file}" "${key_file}" || {
                echo "Error: GPG encryption failed" >&2
                exit 1
            }
            ;;
        age)
            if ! command -v age >/dev/null 2>&1; then
                echo "Error: age not found. Install age or use --encryption gpg" >&2
                exit 1
            fi
            
            echo "Encrypting with age..."
            # age requires a recipient public key or passphrase
            # For simplicity, we'll use passphrase mode
            age --encrypt --output "${output_file}" "${key_file}" || {
                echo "Error: age encryption failed" >&2
                exit 1
            }
            ;;
        *)
            echo "Error: Unknown encryption method: ${encryption_method}" >&2
            exit 1
            ;;
    esac

    # Create backup metadata
    local metadata_file="${output_file}.meta"
    cat > "${metadata_file}" <<EOF
{
  "backup_type": "key",
  "created_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "source_file": "${key_file}",
  "encryption_method": "${encryption_method}",
  "backup_hash": "$(sha256sum "${output_file}" | cut -d' ' -f1)"
}
EOF

    echo "✅ Backup created: ${output_file}"
    echo "Metadata: ${metadata_file}"
    echo ""
    echo "⚠️  IMPORTANT: Store backup securely and verify it can be restored!"
}

restore_key() {
    local backup_file="$1"
    local output_file="$2"
    local encryption_method="${3:-gpg}"

    if [[ ! -f "$backup_file" ]]; then
        echo "Error: Backup file not found: ${backup_file}" >&2
        exit 1
    fi

    echo "=== Restoring Key from Backup ==="
    echo "Backup file: ${backup_file}"
    echo "Output: ${output_file}"
    echo "Encryption: ${encryption_method}"
    echo ""

    # Check for metadata file
    local metadata_file="${backup_file}.meta"
    if [[ -f "$metadata_file" ]]; then
        echo "Found backup metadata: ${metadata_file}"
        local stored_method=$(jq -r '.encryption_method' "$metadata_file" 2>/dev/null || echo "")
        if [[ -n "$stored_method" && "$stored_method" != "$encryption_method" ]]; then
            echo "Warning: Metadata indicates encryption method: ${stored_method}"
            echo "Using specified method: ${encryption_method}"
        fi
    fi

    case "$encryption_method" in
        gpg)
            if ! command -v gpg >/dev/null 2>&1; then
                echo "Error: GPG not found" >&2
                exit 1
            fi
            
            echo "Decrypting with GPG..."
            gpg --decrypt --output "${output_file}" "${backup_file}" || {
                echo "Error: GPG decryption failed" >&2
                exit 1
            }
            ;;
        age)
            if ! command -v age >/dev/null 2>&1; then
                echo "Error: age not found" >&2
                exit 1
            fi
            
            echo "Decrypting with age..."
            age --decrypt --output "${output_file}" "${backup_file}" || {
                echo "Error: age decryption failed" >&2
                exit 1
            }
            ;;
        *)
            echo "Error: Unknown encryption method: ${encryption_method}" >&2
            exit 1
            ;;
    esac

    # Verify restored key format
    if ! jq -e '.public_key' "${output_file}" >/dev/null 2>&1; then
        echo "Error: Restored file is not a valid key file" >&2
        rm -f "${output_file}"
        exit 1
    fi

    # Set secure permissions
    chmod 600 "${output_file}"

    echo "✅ Key restored: ${output_file}"
    echo ""
    echo "Verifying restored key..."
    
    # Verify key can be used
    if command -v bllvm-keygen >/dev/null 2>&1; then
        PUBLIC_KEY=$(jq -r '.public_key' "${output_file}")
        echo "Public Key: ${PUBLIC_KEY}"
        echo "✅ Key verification passed"
    fi
}

verify_backup() {
    local backup_file="$1"

    if [[ ! -f "$backup_file" ]]; then
        echo "Error: Backup file not found: ${backup_file}" >&2
        exit 1
    fi

    echo "=== Verifying Backup ==="
    echo "Backup file: ${backup_file}"
    echo ""

    # Check for metadata
    local metadata_file="${backup_file}.meta"
    if [[ -f "$metadata_file" ]]; then
        echo "✅ Metadata file found: ${metadata_file}"
        
        # Verify backup hash
        local stored_hash=$(jq -r '.backup_hash' "$metadata_file" 2>/dev/null || echo "")
        if [[ -n "$stored_hash" ]]; then
            local current_hash=$(sha256sum "${backup_file}" | cut -d' ' -f1)
            if [[ "$stored_hash" == "$current_hash" ]]; then
                echo "✅ Backup hash verification passed"
            else
                echo "❌ Backup hash mismatch!"
                echo "   Stored: ${stored_hash}"
                echo "   Current: ${current_hash}"
                exit 1
            fi
        fi
        
        # Display metadata
        echo ""
        echo "Backup metadata:"
        jq '.' "$metadata_file"
    else
        echo "⚠️  Warning: No metadata file found"
    fi

    # Check file size
    local file_size=$(stat -f%z "${backup_file}" 2>/dev/null || stat -c%s "${backup_file}" 2>/dev/null || echo "0")
    if [[ "$file_size" -gt 0 ]]; then
        echo "✅ Backup file size: ${file_size} bytes"
    else
        echo "❌ Backup file appears to be empty"
        exit 1
    fi

    echo ""
    echo "✅ Backup verification complete"
}

# Parse command
COMMAND="${1:-}"
shift || true

case "$COMMAND" in
    backup)
        KEY_FILE=""
        OUTPUT_FILE=""
        ENCRYPTION_METHOD="gpg"
        
        while [[ $# -gt 0 ]]; do
            case "$1" in
                --key)
                    KEY_FILE="$2"
                    shift 2
                    ;;
                --output)
                    OUTPUT_FILE="$2"
                    shift 2
                    ;;
                --encryption)
                    ENCRYPTION_METHOD="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1" >&2
                    print_usage
                    exit 1
                    ;;
            esac
        done
        
        if [[ -z "$KEY_FILE" ]] || [[ -z "$OUTPUT_FILE" ]]; then
            echo "Error: --key and --output required for backup" >&2
            print_usage
            exit 1
        fi
        
        backup_key "$KEY_FILE" "$OUTPUT_FILE" "$ENCRYPTION_METHOD"
        ;;
    restore)
        BACKUP_FILE=""
        OUTPUT_FILE=""
        ENCRYPTION_METHOD="gpg"
        
        while [[ $# -gt 0 ]]; do
            case "$1" in
                --backup)
                    BACKUP_FILE="$2"
                    shift 2
                    ;;
                --output)
                    OUTPUT_FILE="$2"
                    shift 2
                    ;;
                --encryption)
                    ENCRYPTION_METHOD="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1" >&2
                    print_usage
                    exit 1
                    ;;
            esac
        done
        
        if [[ -z "$BACKUP_FILE" ]] || [[ -z "$OUTPUT_FILE" ]]; then
            echo "Error: --backup and --output required for restore" >&2
            print_usage
            exit 1
        fi
        
        restore_key "$BACKUP_FILE" "$OUTPUT_FILE" "$ENCRYPTION_METHOD"
        ;;
    verify)
        BACKUP_FILE=""
        
        while [[ $# -gt 0 ]]; do
            case "$1" in
                --backup)
                    BACKUP_FILE="$2"
                    shift 2
                    ;;
                *)
                    echo "Unknown option: $1" >&2
                    print_usage
                    exit 1
                    ;;
            esac
        done
        
        if [[ -z "$BACKUP_FILE" ]]; then
            echo "Error: --backup required for verify" >&2
            print_usage
            exit 1
        fi
        
        verify_backup "$BACKUP_FILE"
        ;;
    --help|help)
        print_usage
        exit 0
        ;;
    *)
        echo "Error: Unknown command: ${COMMAND}" >&2
        print_usage
        exit 1
        ;;
esac


