# Governance App Scripts

This directory contains setup, testing, and deployment scripts for the governance-app.

## Scripts

### Setup Scripts
- `setup-production.sh` - Set up production environment
- `setup-testnet.sh` - Set up testnet environment
- `setup_sqlx.sh` - Set up SQLx for database migrations
- `generate-test-keys.sh` - Generate test cryptographic keys

### Testing Scripts
- `testnet-test-suite.sh` - Run testnet test suite
- `test_cross_layer_sync.sh` - Test cross-layer synchronization
- `verify-integration.sh` - Verify integration
- `verify-server.sh` - Verify server setup

### Database Scripts
- `migrate_sqlite_to_postgres.sh` - Migrate from SQLite to PostgreSQL
- `backup_sqlite.sh` - Backup SQLite database

## Usage

See [governance-app/README.md](../README.md) for governance-app documentation and usage instructions.

## Related

- `../config/` - Configuration files
- `../docs/` - Application documentation

