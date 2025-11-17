# Hidden Gems: Non-Obvious Configuration Options

These are configuration options that most people wouldn't think to make configurable, but would actually be really useful for different deployment scenarios, performance tuning, and edge cases.

## 1. Network Timeouts & Intervals

### Peer Connection Timing
```toml
[network.timing]
# Wait time before connecting to peers from database (after persistent peers)
peer_connection_delay_seconds = 2

# Target number of peers to connect to (Bitcoin Core uses 8-125)
target_peer_count = 8

# Minimum interval between addr message broadcasts (prevents spam)
addr_relay_min_interval_seconds = 8640  # 2.4 hours

# Maximum addresses to include in a single addr message
max_addresses_per_addr_message = 1000
```

**Why it's useful:**
- **Low-resource nodes**: Lower `target_peer_count` to save bandwidth
- **High-performance nodes**: Increase `target_peer_count` for better connectivity
- **Privacy-focused**: Increase `addr_relay_min_interval_seconds` to reduce address leakage
- **Network debugging**: Adjust `peer_connection_delay_seconds` to troubleshoot connection issues

### Request Timeouts
```toml
[network.request_timeouts]
# Timeout for async request-response patterns (getheaders, getdata, etc.)
async_request_timeout_seconds = 300  # 5 minutes

# Timeout for UTXO commitment requests
utxo_commitment_request_timeout_seconds = 30

# Cleanup interval for expired pending requests
request_cleanup_interval_seconds = 60

# Maximum age for pending requests before cleanup
pending_request_max_age_seconds = 300  # 5 minutes
```

**Why it's useful:**
- **Slow networks**: Increase timeouts for satellite/cellular connections
- **Fast networks**: Decrease timeouts for faster failure detection
- **Resource-constrained**: Shorter cleanup intervals to free memory faster

## 2. DNS Seed Discovery

```toml
[network.dns_seeds]
# Maximum addresses to fetch from DNS seeds
max_addresses_from_dns = 100

# Custom DNS seeds (override defaults)
# custom_seeds = ["seed.example.com", "seed2.example.com"]

# DNS resolution timeout
dns_resolution_timeout_seconds = 5
```

**Why it's useful:**
- **Private networks**: Use custom DNS seeds for private Bitcoin networks
- **Testing**: Limit DNS seed results for faster test runs
- **Censorship resistance**: Add backup DNS seeds if primary ones are blocked

## 3. Module System Timing

```toml
[modules.timing]
# Wait time after spawning module process before checking socket
module_startup_wait_millis = 100

# Timeout for module socket to appear
module_socket_timeout_seconds = 5

# Interval between socket existence checks
module_socket_check_interval_millis = 100

# Maximum attempts to check for socket
module_socket_max_attempts = 50
```

**Why it's useful:**
- **Slow systems**: Increase timeouts for Raspberry Pi or embedded systems
- **Fast systems**: Decrease timeouts for faster startup
- **Debugging**: Adjust intervals to troubleshoot module loading issues

## 4. RPC Cache TTL

```toml
[rpc.cache]
# Cache TTL for blockchain RPC calls (getblockchaininfo, etc.)
blockchain_info_cache_ttl_seconds = 1

# Cache TTL for network info
network_info_cache_ttl_seconds = 1

# Cache TTL for mempool info
mempool_info_cache_ttl_seconds = 1
```

**Why it's useful:**
- **High-traffic RPC**: Increase TTL to reduce load (trades freshness for performance)
- **Real-time monitoring**: Decrease TTL for more accurate metrics
- **Resource-constrained**: Increase TTL to reduce CPU usage

## 5. Module Resource Limits

```toml
[modules.resource_limits]
# Default CPU limit for modules (percentage, 0-100)
default_max_cpu_percent = 50

# Default memory limit for modules (bytes)
default_max_memory_bytes = 536870912  # 512 MB

# Default file descriptor limit
default_max_file_descriptors = 256

# Default child process limit
default_max_child_processes = 10
```

**Why it's useful:**
- **Security**: Stricter limits for untrusted modules
- **Performance**: Higher limits for trusted, resource-intensive modules
- **Multi-tenant**: Different limits per module type

## 6. Metrics & Monitoring Intervals

```toml
[monitoring]
# Update metrics every N messages OR every N seconds (whichever comes first)
metrics_update_message_interval = 100
metrics_update_time_interval_seconds = 10

# Performance profiler sample size
profiler_sample_size = 1000
```

**Why it's useful:**
- **High-throughput nodes**: Increase intervals to reduce overhead
- **Debugging**: Decrease intervals for detailed performance analysis
- **Production monitoring**: Balance between accuracy and overhead

## 7. Address Database Behavior

```toml
[address_database]
# Maximum addresses to store (already configurable, but also consider...)
max_addresses = 10000

# Address expiration time (already configurable)
expiration_seconds = 86400  # 24 hours

# Minimum freshness for addresses to be considered "good"
min_freshness_seconds = 3600  # 1 hour

# Prefer addresses from certain IP ranges (for privacy/performance)
# preferred_ip_ranges = ["192.168.0.0/16", "10.0.0.0/8"]
```

**Why it's useful:**
- **Privacy**: Prefer local network addresses
- **Performance**: Prefer low-latency IP ranges
- **Testing**: Shorter expiration for faster test cycles

## 8. Peer Discovery Intervals

```toml
[network.discovery]
# Interval for periodic peer discovery tasks
peer_discovery_interval_seconds = 300  # 5 minutes

# Interval for address relay tasks
address_relay_interval_seconds = 300  # 5 minutes

# Enable/disable automatic peer discovery
auto_discover_peers = true
```

**Why it's useful:**
- **Battery-powered devices**: Less frequent discovery to save power
- **Always-on nodes**: More frequent discovery for better connectivity
- **Private networks**: Disable auto-discovery, use only persistent peers

## 9. Connection Behavior

```toml
[network.connection]
# Connection timeout for TCP connections
tcp_connection_timeout_seconds = 10

# Keepalive interval for maintaining connections
keepalive_interval_seconds = 60

# Maximum connection attempts before giving up
max_connection_attempts = 3

# Delay between connection attempts
connection_retry_delay_seconds = 5
```

**Why it's useful:**
- **Slow networks**: Longer timeouts for satellite/cellular
- **Fast networks**: Shorter timeouts for faster failure detection
- **Unreliable networks**: More retry attempts

## 10. Performance Tuning

```toml
[performance]
# Block validation parallelism (number of threads)
block_validation_threads = 4

# Mempool size limits
max_mempool_size_mb = 300

# Cache sizes (already partially configurable)
block_cache_size_mb = 100
utxo_cache_size_mb = 50
header_cache_size_mb = 10

# Batch sizes for operations
block_batch_size = 100
tx_batch_size = 1000
```

**Why it's useful:**
- **High-performance nodes**: Increase all limits
- **Resource-constrained**: Decrease all limits
- **SSD vs HDD**: Adjust cache sizes based on storage speed

## 11. Privacy & Anonymity

```toml
[privacy]
# Random delay before broadcasting transactions (milliseconds)
tx_broadcast_delay_min_millis = 0
tx_broadcast_delay_max_millis = 5000

# Randomize peer selection order
randomize_peer_selection = true

# Don't send our own address in addr messages
hide_own_address = false

# Prefer connecting through Tor (if available)
prefer_tor = false
```

**Why it's useful:**
- **Privacy-focused users**: Enable all privacy features
- **High-performance nodes**: Disable delays for faster propagation
- **Censorship resistance**: Enable Tor preference

## 12. Developer/Testing Options

```toml
[development]
# Enable mock mode (for testing without real network)
mock_mode = false

# Generate test data automatically
generate_test_data = false

# Verbose debug logging
debug_verbosity = "normal"  # minimal, normal, verbose, extreme

# Performance benchmarking mode
benchmark_mode = false
```

**Why it's useful:**
- **Testing**: Mock mode for unit tests
- **Development**: Test data generation for faster iteration
- **Debugging**: Verbose logging for troubleshooting

## Implementation Priority

### High Priority (Most Useful)
1. **Network timeouts** - Critical for different network conditions
2. **Peer connection timing** - Affects connectivity and performance
3. **Module resource limits** - Security and resource management
4. **RPC cache TTL** - Performance tuning

### Medium Priority (Nice to Have)
5. **DNS seed configuration** - Useful for private networks
6. **Metrics intervals** - Performance monitoring
7. **Connection behavior** - Network reliability

### Low Priority (Edge Cases)
8. **Privacy options** - Niche use cases
9. **Developer options** - Testing only
10. **Performance tuning** - Advanced users only

## Example Use Cases

### Raspberry Pi Node
```toml
[network.timing]
target_peer_count = 4  # Fewer peers
peer_connection_delay_seconds = 5  # Slower startup

[modules.timing]
module_socket_timeout_seconds = 10  # More time for slow system

[performance]
block_validation_threads = 2  # Fewer threads
block_cache_size_mb = 50  # Smaller cache
```

### High-Performance Mining Node
```toml
[network.timing]
target_peer_count = 50  # More peers
peer_connection_delay_seconds = 1  # Faster startup

[performance]
block_validation_threads = 8  # More threads
block_cache_size_mb = 500  # Larger cache
max_mempool_size_mb = 1000  # Larger mempool
```

### Privacy-Focused Node
```toml
[privacy]
tx_broadcast_delay_min_millis = 1000
tx_broadcast_delay_max_millis = 10000
randomize_peer_selection = true
hide_own_address = true
prefer_tor = true

[network.timing]
addr_relay_min_interval_seconds = 36000  # 10 hours (less frequent)
```

### Satellite/High-Latency Connection
```toml
[network.request_timeouts]
async_request_timeout_seconds = 1800  # 30 minutes
utxo_commitment_request_timeout_seconds = 300  # 5 minutes

[network.connection]
tcp_connection_timeout_seconds = 60
connection_retry_delay_seconds = 30
max_connection_attempts = 10
```


