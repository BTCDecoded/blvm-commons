# Additional Configuration Options

## Overview

This document lists important configuration options that were previously hardcoded but are now configurable. These settings are critical for:

- **Security**: DoS protection, rate limiting, ban durations
- **Performance**: Connection limits, queue sizes, relay settings
- **Privacy**: Dandelion++ parameters
- **Network**: Address database, peer discovery

## New Configuration Sections

### DoS Protection (`dos_protection`)

Controls DoS protection and connection rate limiting:

```toml
[dos_protection]
# Maximum connections per IP per time window
max_connections_per_window = 10

# Time window in seconds for connection rate limiting
window_seconds = 60

# Maximum message queue size
max_message_queue_size = 10000

# Maximum active connections
max_active_connections = 200

# Auto-ban threshold (number of violations before auto-ban)
auto_ban_threshold = 3

# Default ban duration in seconds
ban_duration_seconds = 3600  # 1 hour
```

**Environment Variables:**
- `BLLVM_DOS_MAX_CONNECTIONS_PER_WINDOW=10`
- `BLLVM_DOS_WINDOW_SECONDS=60`
- `BLLVM_DOS_MAX_MESSAGE_QUEUE_SIZE=10000`
- `BLLVM_DOS_MAX_ACTIVE_CONNECTIONS=200`
- `BLLVM_DOS_AUTO_BAN_THRESHOLD=3`
- `BLLVM_DOS_BAN_DURATION_SECONDS=3600`

### Network Relay (`relay`)

Controls block and transaction relay behavior:

```toml
[relay]
# Maximum age for relayed items (seconds)
max_relay_age = 3600  # 1 hour

# Maximum number of items to track
max_tracked_items = 10000

# Enable block relay
enable_block_relay = true

# Enable transaction relay
enable_tx_relay = true

# Enable Dandelion++ privacy relay
enable_dandelion = false
```

**Environment Variables:**
- `BLLVM_RELAY_MAX_AGE=3600`
- `BLLVM_RELAY_MAX_TRACKED_ITEMS=10000`
- `BLLVM_RELAY_ENABLE_BLOCK_RELAY=true`
- `BLLVM_RELAY_ENABLE_TX_RELAY=true`
- `BLLVM_RELAY_ENABLE_DANDELION=false`

### Address Database (`address_database`)

Controls peer address storage and expiration:

```toml
[address_database]
# Maximum number of addresses to store
max_addresses = 10000

# Address expiration time in seconds
expiration_seconds = 86400  # 24 hours
```

**Environment Variables:**
- `BLLVM_ADDRESS_DB_MAX_ADDRESSES=10000`
- `BLLVM_ADDRESS_DB_EXPIRATION_SECONDS=86400`

### Dandelion++ Privacy Relay (`dandelion`)

Controls Dandelion++ privacy relay parameters (requires `dandelion` feature):

```toml
[dandelion]
# Stem phase timeout in seconds
stem_timeout_seconds = 10

# Probability of fluffing at each hop (0.0 to 1.0)
fluff_probability = 0.1  # 10%

# Maximum stem hops before forced fluff
max_stem_hops = 2
```

**Environment Variables:**
- `BLLVM_DANDELION_STEM_TIMEOUT_SECONDS=10`
- `BLLVM_DANDELION_FLUFF_PROBABILITY=0.1`
- `BLLVM_DANDELION_MAX_STEM_HOPS=2`

### Peer Rate Limiting (`peer_rate_limiting`)

Controls per-peer message rate limiting:

```toml
[peer_rate_limiting]
# Default burst size (token bucket)
default_burst = 100

# Default rate (messages per second)
default_rate = 10
```

**Environment Variables:**
- `BLLVM_PEER_RATE_BURST=100`
- `BLLVM_PEER_RATE_RATE=10`

## Use Cases

### High-Security Deployment

```toml
[dos_protection]
max_connections_per_window = 5  # Stricter limits
window_seconds = 30  # Shorter window
max_active_connections = 100  # Lower connection limit
auto_ban_threshold = 2  # Ban faster
ban_duration_seconds = 7200  # 2 hour bans
```

### High-Performance Node

```toml
[dos_protection]
max_active_connections = 500  # More connections
max_message_queue_size = 50000  # Larger queue

[relay]
max_tracked_items = 50000  # Track more items

[address_database]
max_addresses = 50000  # Store more addresses
```

### Privacy-Focused Node

```toml
[relay]
enable_dandelion = true

[dandelion]
stem_timeout_seconds = 30  # Longer stem phase
fluff_probability = 0.05  # Lower fluff probability (more privacy)
max_stem_hops = 3  # More hops before fluff
```

### Resource-Constrained Node

```toml
[dos_protection]
max_active_connections = 50  # Fewer connections
max_message_queue_size = 5000  # Smaller queue

[relay]
max_tracked_items = 5000  # Track fewer items

[address_database]
max_addresses = 5000  # Store fewer addresses
```

## Migration Notes

These settings were previously hardcoded with the following defaults:

- **DoS Protection**: 10 connections/IP/60s, 10k queue, 200 max connections, 3 violations = ban, 1 hour ban
- **Relay**: 1 hour max age, 10k tracked items
- **Address Database**: 10k max addresses, 24 hour expiration
- **Dandelion**: 10s timeout, 10% fluff probability, 2 max hops
- **Peer Rate Limiting**: 100 burst, 10 messages/second

All defaults remain the same, so existing deployments will continue to work without changes.

## Implementation Status

✅ **Config structs added** - All new config sections are defined
⏳ **Runtime integration** - Config values need to be applied to actual components
⏳ **ENV variable support** - Environment variable parsing needs to be added to `bllvm` binary

## Next Steps

1. Update `NetworkManager` to use `DosProtectionConfig`
2. Update `RelayManager` to use `RelayConfig`
3. Update `AddressDatabase` to use `AddressDatabaseConfig`
4. Update Dandelion to use `DandelionConfig`
5. Add ENV variable parsing for all new options
6. Update documentation with examples

