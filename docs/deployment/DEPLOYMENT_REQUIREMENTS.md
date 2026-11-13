# blvm-commons Deployment Requirements

## Overview

`blvm-commons` is a Rust binary that runs as a web service. It requires minimal setup but needs specific configuration and files.

## Required Components

### 1. Binary

**Status**: ✅ **Required**

The binary is built from source:
```bash
cd governance-app
cargo build --release
# Binary will be at: target/release/blvm-commons
```

**Installation**:
```bash
sudo cp target/release/blvm-commons /usr/local/bin/
sudo chmod +x /usr/local/bin/blvm-commons
```

**Note**: The binary name is `blvm-commons` (defined in `Cargo.toml`).

### 2. Environment Variables

**Status**: ✅ **Required** (no `.env` file needed, but recommended)

The application reads configuration from environment variables. You can either:
- Set environment variables directly
- Use a `.env` file (if using a tool like `dotenv`)
- Use systemd `EnvironmentFile` directive

**Required Environment Variables**:

```bash
# Database (REQUIRED)
DATABASE_URL=postgresql://user:password@localhost:5432/governance
# OR for SQLite (development):
# DATABASE_URL=sqlite://governance.db

# GitHub App (REQUIRED)
GITHUB_APP_ID=123456
GITHUB_PRIVATE_KEY_PATH=/opt/blvm-commons/keys/github-app.pem
GITHUB_WEBHOOK_SECRET=your_webhook_secret_here

# Server Configuration (OPTIONAL - has defaults)
SERVER_HOST=0.0.0.0          # Default: 0.0.0.0
SERVER_PORT=3000              # Default: 3000
SERVER_ID=governance-01       # Default: governance-01

# Governance (OPTIONAL)
GOVERNANCE_REPO=BTCDecoded/governance  # Default: BTCDecoded/governance
DRY_RUN_MODE=true             # Default: true. Set false only when enforcement is on.
LOG_ENFORCEMENT_DECISIONS=true # Default: true
ENFORCEMENT_LOG_PATH=/var/log/blvm-commons/enforcement.log  # Optional

# Nostr (OPTIONAL - disabled by default)
NOSTR_ENABLED=false
NOSTR_SERVER_NSEC_PATH=/etc/governance/server.nsec
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol
NOSTR_PUBLISH_INTERVAL_SECS=3600

# OpenTimestamps (OPTIONAL - disabled by default)
OTS_ENABLED=false
OTS_AGGREGATOR_URL=https://alice.btc.calendar.opentimestamps.org
OTS_MONTHLY_ANCHOR_DAY=1
OTS_REGISTRY_PATH=/var/lib/governance/registries
OTS_PROOFS_PATH=/var/lib/governance/ots-proofs

# Audit Logging (OPTIONAL - enabled by default)
AUDIT_ENABLED=true
AUDIT_LOG_PATH=/var/lib/governance/audit-log.jsonl
AUDIT_ROTATION_INTERVAL_DAYS=30
```

**Minimal Production Setup**:
```bash
# Create .env file
cat > /etc/blvm-commons/.env <<EOF
DATABASE_URL=postgresql://governance_user:password@localhost:5432/governance
GITHUB_APP_ID=123456
GITHUB_PRIVATE_KEY_PATH=/opt/blvm-commons/keys/github-app.pem
GITHUB_WEBHOOK_SECRET=$(openssl rand -hex 32)
SERVER_PORT=3000
EOF

chmod 600 /etc/blvm-commons/.env
```

### 3. Configuration File

**Status**: ⚠️ **Optional** (not currently loaded by code)

The `config/production.toml` file exists but **is not currently used** by the application. The code only reads from environment variables (see `src/config.rs`).

**Future Enhancement**: The config loader module exists (`src/config/loader.rs`) but isn't integrated. To use TOML config files, you'd need to:
1. Update `AppConfig::load()` to read from TOML
2. Or use a config library that supports both env vars and TOML

**For now**: Use environment variables only.

### 4. Required Files

**GitHub App Private Key** (REQUIRED):
```bash
# Download from GitHub App settings
# Place at: GITHUB_PRIVATE_KEY_PATH
sudo mkdir -p /opt/blvm-commons/keys
sudo cp github-app.pem /opt/blvm-commons/keys/
sudo chmod 600 /opt/blvm-commons/keys/github-app.pem
sudo chown governance:governance /opt/blvm-commons/keys/github-app.pem
```

**Database** (REQUIRED):
- PostgreSQL (production) or SQLite (development)
- Database must exist and be accessible
- Migrations run automatically on startup

### 5. Directory Structure

**Required Directories**:
```bash
/opt/blvm-commons/          # Application root
├── blvm-commons            # Binary (or symlink to /usr/local/bin)
├── keys/                    # Private keys (github-app.pem)
│   └── github-app.pem
├── data/                    # Database files (if SQLite)
│   └── governance.db
└── logs/                    # Application logs (optional)

/var/log/blvm-commons/      # Log directory (if using file logging)
```

**Create directories**:
```bash
sudo mkdir -p /opt/blvm-commons/{keys,data,logs}
sudo mkdir -p /var/log/blvm-commons
sudo chown -R governance:governance /opt/blvm-commons
sudo chown -R governance:governance /var/log/blvm-commons
sudo chmod 700 /opt/blvm-commons/keys
```

### 6. Systemd Service

**Status**: ✅ **Recommended**

Create `/etc/systemd/system/blvm-commons.service`:

```ini
[Unit]
Description=Bitcoin Commons (blvm-commons)
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=governance
Group=governance
WorkingDirectory=/opt/blvm-commons
ExecStart=/usr/local/bin/blvm-commons
Restart=always
RestartSec=5
Environment=RUST_LOG=info
EnvironmentFile=/etc/blvm-commons/.env

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/blvm-commons/data
ReadWritePaths=/var/log/blvm-commons

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

**Enable and start**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable blvm-commons
sudo systemctl start blvm-commons
sudo systemctl status blvm-commons
```

## Quick Deployment Checklist

- [ ] Build binary: `cargo build --release`
- [ ] Install binary: `sudo cp target/release/blvm-commons /usr/local/bin/`
- [ ] Create directories: `/opt/blvm-commons/{keys,data,logs}`
- [ ] Set up database (PostgreSQL or SQLite)
- [ ] Download GitHub App private key to `/opt/blvm-commons/keys/github-app.pem`
- [ ] Create `.env` file with required variables
- [ ] Create systemd service file
- [ ] Start service: `sudo systemctl start blvm-commons`
- [ ] Verify: `curl http://localhost:3000/health`

## Minimal Production Deployment

```bash
# 1. Build
cd governance-app
cargo build --release

# 2. Install
sudo cp target/release/blvm-commons /usr/local/bin/
sudo chmod +x /usr/local/bin/blvm-commons

# 3. Create user
sudo useradd -r -s /bin/false -d /opt/blvm-commons governance

# 4. Create directories
sudo mkdir -p /opt/blvm-commons/{keys,data}
sudo chown -R governance:governance /opt/blvm-commons

# 5. Set up database (PostgreSQL example)
sudo -u postgres createdb governance
sudo -u postgres createuser governance_user
sudo -u postgres psql -c "ALTER USER governance_user WITH PASSWORD 'secure_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE governance TO governance_user;"

# 6. Create .env file
sudo tee /etc/blvm-commons/.env > /dev/null <<EOF
DATABASE_URL=postgresql://governance_user:secure_password@localhost:5432/governance
GITHUB_APP_ID=YOUR_APP_ID
GITHUB_PRIVATE_KEY_PATH=/opt/blvm-commons/keys/github-app.pem
GITHUB_WEBHOOK_SECRET=$(openssl rand -hex 32)
SERVER_PORT=3000
EOF
sudo chmod 600 /etc/blvm-commons/.env

# 7. Copy GitHub App key
sudo cp github-app.pem /opt/blvm-commons/keys/
sudo chmod 600 /opt/blvm-commons/keys/github-app.pem
sudo chown governance:governance /opt/blvm-commons/keys/github-app.pem

# 8. Create systemd service (see above)

# 9. Start
sudo systemctl daemon-reload
sudo systemctl enable blvm-commons
sudo systemctl start blvm-commons
```

## Verification

```bash
# Check service status
sudo systemctl status blvm-commons

# Check logs
sudo journalctl -u blvm-commons -f

# Test health endpoint
curl http://localhost:3000/health

# Test status endpoint
curl http://localhost:3000/status
```

## Notes

1. **No TOML config loading**: The `config/production.toml` file exists but isn't used. Use environment variables instead.

2. **Database migrations**: Run automatically on startup via `database.run_migrations()`.

3. **Port**: Default is `3000`, configurable via `SERVER_PORT` environment variable.

4. **Binary name**: The binary is named `blvm-commons` (not `governance-app`).

5. **Environment variables**: All configuration comes from environment variables. No `.env` file loader is included, but you can use systemd's `EnvironmentFile` directive.

