# Graceful Degradation Improvements

This document describes the graceful degradation mechanisms implemented in the node to ensure robust operation even when components fail or features are unavailable.

## Database Backend Fallback

### Automatic Backend Fallback
When initializing storage, the node now automatically falls back to an alternative database backend if the primary fails:

- **Primary**: `redb` (default)
- **Fallback**: `sled` (if available)

**Implementation**:
- `Storage::new()` attempts to use the default backend
- If initialization fails, automatically tries the fallback backend
- Logs warnings when fallback occurs
- Only errors if both backends fail or no fallback is available

**Example**:
```rust
// If redb fails, automatically tries sled
let storage = Storage::new(data_dir)?;
```

### Benefits
- **Resilience**: Node can start even if preferred backend has issues
- **Migration Safety**: Can handle database corruption or version mismatches
- **Development**: Easier testing with different backends

## Storage Operation Graceful Degradation

### Disk Size Estimation
The `disk_size()` method now gracefully handles individual operation failures:

- If block count fails → continues with UTXO and transaction estimates
- If UTXO count fails → continues with block and transaction estimates  
- If transaction count fails → continues with block and UTXO estimates
- Returns 0 if all operations fail (rather than erroring)

**Before**: Would fail if any count operation failed
**After**: Returns partial estimate, gracefully degrading

## RPC Method Graceful Degradation

### Memory Information (`getmemoryinfo`)
- **With sysinfo feature**: Returns actual memory statistics
- **Without sysinfo feature**: Returns placeholder with note explaining limitation
- Method never fails due to missing feature

### Mining Information (`getmininginfo`)
- **Difficulty**: Falls back to 1.0 if chain tip unavailable
- **Network Hashrate**: Falls back to 0.0 if calculation fails
- **Block Count**: Falls back to 0 if storage unavailable
- All degradations are logged for debugging

### Blockchain Queries
- **Height queries**: Return 0 if chain not initialized
- **Block queries**: Return appropriate errors (not panics)
- **Statistics**: Use `unwrap_or` defaults for missing data

## Network Transport Graceful Degradation

### Transport Fallback
The network manager implements comprehensive transport fallback:

1. **Connection Attempts**: Tries transports in preference order
2. **Automatic Fallback**: Falls back to TCP if Quinn/Iroh fail
3. **Error Handling**: Logs failures but continues with next transport
4. **Final Error**: Only errors if all transports fail

**Transport Priority** (outgoing connections):
- Quinn (if available and preferred)
- Iroh (if available and preferred)
- TCP (always available as fallback)

**Listener Graceful Degradation**:
- Quinn listener failures are logged but don't prevent TCP listener from starting
- Iroh listener failures are logged but don't prevent other listeners from starting
- TCP listener is always started as the base transport
- Node continues operating with available transports

**Implementation Details**:
- Each transport listener starts independently
- Failures in one transport don't affect others
- Connection attempts try all available transports before failing
- Transport preference is respected but fallback is automatic

## Feature Flag Graceful Degradation

### Optional Features
When optional features are disabled, the system degrades gracefully:

- **sysinfo**: Memory stats return placeholders
- **quinn**: Network falls back to TCP
- **iroh**: Network falls back to TCP/Quinn
- **sled/redb**: Database backend fallback (see above)

### Error Messages
All graceful degradations include:
- Clear logging of what failed and why
- Informative error messages when fallback unavailable
- Debug-level logs for troubleshooting

## Best Practices

### When Adding New Features
1. **Always provide fallbacks** for optional dependencies
2. **Use `unwrap_or` or `unwrap_or_else`** for default values
3. **Log degradations** at appropriate levels (warn for fallbacks, debug for expected)
4. **Return partial results** when possible rather than errors
5. **Document fallback behavior** in method documentation

### Error Handling Patterns
```rust
// Good: Graceful degradation with logging
if let Ok(value) = operation() {
    value
} else {
    tracing::debug!("Operation failed, using default");
    default_value
}

// Good: Fallback with error context
match primary_operation() {
    Ok(result) => Ok(result),
    Err(e) => {
        if let Some(fallback) = get_fallback() {
            warn!("Primary failed: {}, using fallback", e);
            fallback_operation()
        } else {
            Err(e)
        }
    }
}
```

## Testing

Graceful degradation is tested in:
- `tests/integration/graceful_degradation_tests.rs` - Transport fallback
- Storage tests verify fallback behavior
- RPC tests verify feature-flag degradation

## Future Improvements

Potential areas for additional graceful degradation:

1. **Partial Block Sync**: Continue with available blocks if some fail
2. **Mempool Degradation**: Continue operating with reduced mempool if full mempool fails
3. **RPC Rate Limiting**: Degrade to simpler rate limiting if advanced features unavailable
4. **Storage Queries**: Return cached/partial results if full query fails

