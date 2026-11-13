# Testnet Setup Guide

This guide explains how to run the **Phase 2A testnet** Docker stack from the **`blvm-commons`** repository (`deployment/testnet`). The main application runs as the Docker Compose service **`blvm-commons`** (`container_name: blvm-commons-testnet`).

## Overview

The testnet environment provides:
- **blvm-commons** (governance enforcement app) with test configuration
- Test maintainer keys
- Monitoring and logging infrastructure
- Test data generation and validation
- Docker-based deployment for easy setup

## Prerequisites

- Docker and Docker Compose
- Git
- Basic understanding of governance concepts
- 4GB RAM minimum, 8GB recommended

## Quick Start

### 1. Clone and Setup

```bash
# Use the monorepo layout (blvm-commons next to blvm-sdk, etc.). Example:
git clone https://github.com/BTCDecoded/btc-commons.git
cd btc-commons/blvm-commons
```

**Docker build context:** The compose file builds from the **monorepo root** (the parent directory that contains `blvm-commons`, `blvm-sdk`, and other patched dependencies). Run `docker compose build` from `deployment/testnet` after cloning the full **btc-commons** tree so `blvm-commons/deployment/testnet/Dockerfile.testnet` can resolve sibling crates.

### 2. Generate Test Keys

```bash
# Generate test maintainer keys
./scripts/generate-test-keys.sh
```

### 3. Start Testnet

```bash
cd deployment/testnet
docker-compose up -d
```

### 4. Verify Setup

```bash
# Check if all services are running
docker-compose ps

# Check blvm-commons logs
docker-compose logs blvm-commons

# Check health endpoint
curl http://localhost:8080/health
```

## Configuration

### Environment Variables

The testnet uses the following key environment variables:

```bash
# Database
DATABASE_URL=sqlite:blvm-commons-testnet.db

# Governance
DRY_RUN_MODE=true
LOG_ENFORCEMENT_DECISIONS=true

# GitHub (test values)
GITHUB_APP_ID=123456
GITHUB_WEBHOOK_SECRET=testnet-webhook-secret

# Nostr
NOSTR_PRIVATE_KEY_PATH=/app/keys/testnet-nostr-key.pem

# Governance Fork
GOVERNANCE_FORK_ENABLED=true
```

### Configuration Files

- `config.toml`: Main application configuration
- `docker-compose.yml`: Service definitions
- `monitoring/`: Prometheus and Grafana configuration
- `nginx/`: Reverse proxy configuration

## Services

### Governance App

- **Port**: 8080
- **Health Check**: http://localhost:8080/health
- **API**: http://localhost:8080/api
- **Metrics**: http://localhost:8080/metrics

### Prometheus

- **Port**: 9091
- **URL**: http://localhost:9091
- **Purpose**: Metrics collection and storage

### Grafana

- **Port**: 3000
- **URL**: http://localhost:3000
- **Username**: admin
- **Password**: testnet123
- **Purpose**: Metrics visualization and dashboards

### Nginx

- **Port**: 80, 443
- **Purpose**: Reverse proxy and SSL termination

## Test Data

### Test Maintainers

The testnet includes 7 test maintainers:

| Username | Layer | Role |
|----------|-------|------|
| alice    | 1     | Core maintainer |
| bob      | 1     | Core maintainer |
| charlie  | 2     | Feature maintainer |
| dave     | 2     | Feature maintainer |
| eve      | 3     | Integration maintainer |
| frank    | 3     | Integration maintainer |
| grace    | 3     | Integration maintainer |

## Testing Workflows

### 1. Signature Verification

```bash
# Generate a test signature
cargo run --release --bin sign-pr sign \
  --key test-keys/alice_private.pem \
  --repo test/repo \
  --pr 1

# Post signature as comment
# /governance-sign <signature>
```

### 2. Governance Fork

```bash
# Create new ruleset
cargo run --release --bin fork-migrate create \
  --name "test-ruleset" \
  --description "Test governance ruleset"

# Migrate to new ruleset
cargo run --release --bin fork-migrate migrate \
  --ruleset "test-ruleset" \
  --backup
```

### 3. Monitoring

```bash
# Check system health
curl http://localhost:8080/health

# View metrics
curl http://localhost:8080/metrics

# Check Prometheus targets
curl http://localhost:9091/api/v1/targets
```

## Monitoring and Logging

### Logs

All logs are stored in the `logs/` directory:

- `blvm-commons.log`: Main application logs
- `enforcement-decisions.jsonl`: Governance enforcement decisions
- `audit.log`: Tamper-evident audit log
- `fork-events.jsonl`: Governance fork events

### Metrics

Key metrics to monitor:

- **Governance Events**: Signature collections, fork events
- **System Health**: Database connections, API response times
- **Fork Activity**: Ruleset adoption, migration events

### Dashboards

Grafana dashboards provide:

- **System Overview**: Overall system health and activity
- **Governance Activity**: Maintainer signatures and PR flow
- **Fork Activity**: Ruleset adoption and migration

## Troubleshooting

### Common Issues

**"Service not starting"**
```bash
# Check logs
docker-compose logs blvm-commons

# Check resource usage
docker stats

# Restart services
docker-compose restart
```

**"Database connection failed"**
```bash
# Check database file permissions
ls -la data/

# Recreate database
docker-compose down
rm -rf data/blvm-commons-testnet.db
docker-compose up -d
```

**"Key not found"**
```bash
# Regenerate test keys
./scripts/generate-test-keys.sh

# Restart services
docker-compose restart
```

**"API not responding"**
```bash
# Check if service is running
docker-compose ps

# Check health endpoint
curl http://localhost:8080/health

# Check logs for errors
docker-compose logs blvm-commons
```

### Debug Mode

Enable debug logging:

```bash
# Update environment variable
export RUST_LOG=debug

# Restart services
docker-compose restart
```

### Reset Testnet

Complete reset:

```bash
# Stop all services
docker-compose down

# Remove all data
rm -rf data/ logs/ keys/

# Regenerate everything
./scripts/setup-testnet.sh

# Start services
docker-compose up -d
```

## Development

### Adding Test Data

1. **New Test Maintainers**:
   - Add to `scripts/generate-test-keys.sh`
   - Update database schema if needed

2. **New Test Scenarios**:
   - Add to `tests/integration/testnet_scenarios.rs`
   - Update test documentation

### Custom Configuration

1. **Modify Settings**:
   - Edit `config.toml`
   - Update environment variables in `docker-compose.yml`

2. **Add Services**:
   - Add to `docker-compose.yml`
   - Update monitoring configuration

3. **Custom Monitoring**:
   - Add dashboards to `monitoring/grafana-dashboards.json`
   - Update Prometheus configuration

## Security Considerations

### Testnet Security

- **Test Keys Only**: Never use test keys in production
- **Network Isolation**: Testnet runs in isolated Docker network
- **No Real Data**: All data is synthetic and test-only
- **Regular Cleanup**: Clean up test data regularly

### Production Preparation

- **Key Rotation**: Replace test keys with production keys
- **Security Hardening**: Apply production security settings
- **Monitoring**: Set up production monitoring
- **Backup**: Implement production backup procedures

## Support

### Getting Help

- **Documentation**: Check this guide and other docs
- **Logs**: Review application and system logs
- **Issues**: Report issues on GitHub
- **Community**: Join governance discussions

### Reporting Issues

When reporting issues, include:

- **Environment**: OS, Docker version, configuration
- **Logs**: Relevant log entries
- **Steps**: Steps to reproduce the issue
- **Expected**: What should happen
- **Actual**: What actually happened

## Next Steps

After testnet setup:

1. **Run Tests**: Execute the test suite
2. **Explore Features**: Try all governance features
3. **Monitor Activity**: Watch metrics and logs
4. **Report Issues**: Document any problems
5. **Plan Production**: Prepare for production deployment
