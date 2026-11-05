# Bitcoin Commons Deployment Guide

**Date:** November 17, 2025  
**Status:** Path of Least Resistance  
**Goal:** Deploy governance app, Nostr bots, archival node, and UTXO commitment nodes

---

## 🎯 What You Need to Deploy

1. **Governance App (blvm-commons)** - 1 instance
2. **Nostr Bot Keys** - 4 nsec keys (gov, dev, research, network)
3. **Archival Node** - 1 instance (full blockchain history)
4. **UTXO Commitment Nodes** - 2-3 instances (experimental features)

---

## 🚀 Path of Least Resistance

### Option 1: Docker Compose (Recommended - Easiest)

**Single command deployment for everything.**

### Option 2: Manual Systemd Services

**More control, but more setup.**

---

## 📋 Prerequisites

### Required
- Linux server (Ubuntu 22.04+ recommended)
- Docker & Docker Compose (for Option 1)
- OR systemd (for Option 2)
- 100GB+ disk space (archival node needs ~400GB)
- 8GB+ RAM (16GB recommended)
- Public IP or domain name

### Optional but Recommended
- Nginx reverse proxy
- SSL certificates (Let's Encrypt)
- Monitoring (Prometheus/Grafana)

### RPC Certificate Auth (S-001)
If using certificate-based RPC authentication, the RPC endpoint **must** be behind TLS termination (nginx, Caddy, etc.) that validates the client certificate and sets the `x-client-cert-fingerprint` header. Without this, the header is spoofable—clients can forge it. Use token-based auth if RPC is not behind such TLS.

---

## 🔐 Step 1: Generate Nostr Bot Keys

**Do this first - you'll need the npubs for config.**

```bash
# Install nostr-tool (if not installed)
cargo install nostr-tool

# Generate keys for each bot
mkdir -p ~/nostr-keys

# Governance bot
nostr-tool generate > ~/nostr-keys/gov.nsec
nostr-tool convert ~/nostr-keys/gov.nsec > ~/nostr-keys/gov.npub

# Dev bot
nostr-tool generate > ~/nostr-keys/dev.nsec
nostr-tool convert ~/nostr-keys/dev.nsec > ~/nostr-keys/dev.npub

# Research bot (optional)
nostr-tool generate > ~/nostr-keys/research.nsec
nostr-tool convert ~/nostr-keys/research.nsec > ~/nostr-keys/research.npub

# Network bot (optional)
nostr-tool generate > ~/nostr-keys/network.nsec
nostr-tool convert ~/nostr-keys/network.nsec > ~/nostr-keys/network.npub

# Save npubs (safe to commit)
cat ~/nostr-keys/*.npub
```

**Save nsecs securely:**
- Add to GitHub Secrets: `NOSTR_NSEC_GOV`, `NOSTR_NSEC_DEV`, etc.
- Store in secure location (password manager, hardware key)
- **NEVER commit nsec files to git**

---

## 🐳 Step 2: Deploy Governance App (Docker)

**Easiest method - single docker-compose file.**

### Create Deployment Directory

```bash
mkdir -p ~/btc-commons-deployment
cd ~/btc-commons-deployment
```

### Create docker-compose.yml

```yaml
version: '3.8'

services:
  governance-app:
    image: ghcr.io/btcdecoded/governance-app:latest
    container_name: blvm-commons
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:///app/data/governance.db
      - GITHUB_APP_ID=${GITHUB_APP_ID}
      - GITHUB_PRIVATE_KEY_PATH=/app/keys/github-app.pem
      - GITHUB_WEBHOOK_SECRET=${GITHUB_WEBHOOK_SECRET}
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - SERVER_ID=governance-01
      - NOSTR_ENABLED=true
      - NOSTR_SERVER_NSEC_PATH=/app/keys/nostr/gov.nsec
      - NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol,wss://relay.nostr.band
      - NOSTR_PUBLISH_INTERVAL_SECS=3600
      - GOVERNANCE_CONFIG=commons_mainnet
      - NOSTR_ZAP_ADDRESS=donations@btcdecoded.org
      - NOSTR_LOGO_URL=https://btcdecoded.org/assets/bitcoin-commons-logo.png
      - OTS_ENABLED=true
      - OTS_AGGREGATOR_URL=https://alice.btc.calendar.opentimestamps.org
      - AUDIT_ENABLED=true
      - RUST_LOG=info
    volumes:
      - ./data:/app/data
      - ./keys:/app/keys
      - ./logs:/app/logs
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

### Setup Secrets

```bash
# Create directories
mkdir -p keys/nostr data logs

# Copy Nostr keys (from Step 1)
cp ~/nostr-keys/gov.nsec keys/nostr/
cp ~/nostr-keys/dev.nsec keys/nostr/
cp ~/nostr-keys/research.nsec keys/nostr/  # Optional
cp ~/nostr-keys/network.nsec keys/nostr/   # Optional

# Add GitHub App key
# Download from GitHub App settings → Private keys
cp /path/to/github-app.pem keys/github-app.pem

# Create .env file
cat > .env << EOF
GITHUB_APP_ID=123456
GITHUB_WEBHOOK_SECRET=your_webhook_secret_here
EOF
```

### Deploy

```bash
# Pull latest image
docker-compose pull

# Start service
docker-compose up -d

# Check logs
docker-compose logs -f governance-app

# Verify health
curl http://localhost:8080/health
```

---

## 🗄️ Step 3: Deploy Archival Node

**Full blockchain history - no pruning.**

### Create Node Config

```bash
mkdir -p ~/blvm-nodes/archival
cd ~/blvm-nodes/archival
```

### Create config.toml

**Schema:** This must match **`NodeConfig`** in `blvm-node` (no `[network]` / `[rpc] enabled` shorthand). See [blvm-docs](https://docs.btcdecoded.org/blvm/) / `configuration.md`. RPC bind is **`blvm --rpc-addr`**, not `[rpc].listen_address`.

```toml
listen_addr = "0.0.0.0:8333"
protocol_version = "BitcoinV1"
max_peers = 100
transport_preference = "tcponly"
enable_self_advertisement = true

[storage]
data_dir = "/app/data"
database_backend = "auto"

[storage.pruning]
mode = { type = "disabled" }

[rpc_auth]
required = true
tokens = []  # set via RPC_AUTH_TOKENS or token_file at runtime
```

### Deploy with Docker

```yaml
# Add to docker-compose.yml
  archival-node:
    image: ghcr.io/btcdecoded/blvm:latest
    container_name: blvm-archival
    ports:
      - "8332:8332"  # RPC
      - "8333:8333"  # P2P
    environment:
      - RUST_LOG=info
    volumes:
      - ./archival-data:/app/data
    command: ["blvm", "--config", "/app/config.toml", "--rpc-addr", "0.0.0.0:8332"]
    restart: unless-stopped
```

**Or deploy manually:**

```bash
# Download binary from release
wget https://github.com/BTCDecoded/blvm/releases/latest/download/blvm-linux-x86_64.tar.gz
tar -xzf blvm-linux-x86_64.tar.gz

# Create systemd service
sudo tee /etc/systemd/system/blvm-archival.service << EOF
[Unit]
Description=BLVM Archival Node
After=network.target

[Service]
Type=simple
User=bitcoin
WorkingDirectory=/opt/blvm
ExecStart=/opt/blvm/blvm --config /etc/blvm/archival.toml
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable blvm-archival
sudo systemctl start blvm-archival
```

---

## ⚡ Step 4: Deploy UTXO Commitment Nodes (2-3 instances)

**Experimental features enabled - faster sync.**

### Node 1: UTXO Commitment Node

```bash
mkdir -p ~/blvm-nodes/utxo-commitment-1
cd ~/blvm-nodes/utxo-commitment-1
```

### Create config.toml

Uses **`NodeConfig`** shapes; experimental image must match your build flags (`utxo-commitments`, etc.). Pruning uses **`[storage.pruning]`**, not `prune_mode`.

```toml
listen_addr = "0.0.0.0:8334"
protocol_version = "BitcoinV1"
max_peers = 100
transport_preference = "tcponly"

[storage]
data_dir = "/app/data"
database_backend = "auto"

[storage.pruning]
mode = { type = "normal", keep_from_height = 0, min_recent_blocks = 288 }

[rpc_auth]
required = true
tokens = []
```

```bash
blvm --config /app/config.toml --rpc-addr 0.0.0.0:8335 --network mainnet
```

### Nodes 2 and 3

Use the same table as Node 1, but match **P2P** and **RPC** ports to your compose mapping:

| Instance | `listen_addr` (P2P) | `--rpc-addr` |
|----------|----------------------|--------------|
| 2 | `0.0.0.0:8337` | `0.0.0.0:8336` |
| 3 | `0.0.0.0:8339` | `0.0.0.0:8338` |

### Deploy with Docker

```yaml
# Add to docker-compose.yml
  utxo-commitment-node-1:
    image: ghcr.io/btcdecoded/blvm-experimental:latest
    container_name: blvm-utxo-1
    ports:
      - "8335:8335"  # RPC
      - "8334:8334"  # P2P
    environment:
      - RUST_LOG=info
    volumes:
      - ./utxo-1-data:/app/data
    command: ["blvm", "--config", "/app/config.toml", "--rpc-addr", "0.0.0.0:8335", "--network", "mainnet"]
    restart: unless-stopped

  utxo-commitment-node-2:
    image: ghcr.io/btcdecoded/blvm-experimental:latest
    container_name: blvm-utxo-2
    ports:
      - "8336:8336"  # RPC
      - "8337:8337"  # P2P
    environment:
      - RUST_LOG=info
    volumes:
      - ./utxo-2-data:/app/data
    command: ["blvm", "--config", "/app/config.toml", "--rpc-addr", "0.0.0.0:8336", "--network", "mainnet"]
    restart: unless-stopped

  utxo-commitment-node-3:
    image: ghcr.io/btcdecoded/blvm-experimental:latest
    container_name: blvm-utxo-3
    ports:
      - "8338:8338"  # RPC
      - "8339:8339"  # P2P
    environment:
      - RUST_LOG=info
    volumes:
      - ./utxo-3-data:/app/data
    command: ["blvm", "--config", "/app/config.toml", "--rpc-addr", "0.0.0.0:8338", "--network", "mainnet"]
    restart: unless-stopped
```

**Note:** Use `blvm-experimental` image (has UTXO commitments enabled)

---

## 🔧 Step 5: Configure Governance App for Multi-Bot

### Update config.toml

```toml
[nostr]
enabled = true
governance_config = "commons_mainnet"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]
publish_interval_secs = 3600

[nostr.bots.gov]
nsec_path = "/app/keys/nostr/gov.nsec"
npub = "npub1..."  # From Step 1
lightning_address = "donations@btcdecoded.org"
profile_name = "@BTCCommons_Gov"
profile_about = "Official governance announcements from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-gov.png"

[nostr.bots.dev]
nsec_path = "/app/keys/nostr/dev.nsec"
npub = "npub1..."  # From Step 1
lightning_address = "dev@btcdecoded.org"
profile_name = "@BTCCommons_Dev"
profile_about = "Development updates from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-dev.png"

[nostr.bots.research]
nsec_path = "/app/keys/nostr/research.nsec"
npub = "npub1..."  # From Step 1
lightning_address = "research@btcdecoded.org"
profile_name = "@BTCCommons_Research"
profile_about = "Educational content and research from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-research.png"

[nostr.bots.network]
nsec_path = "/app/keys/nostr/network.nsec"
npub = "npub1..."  # From Step 1
lightning_address = "network@btcdecoded.org"
profile_name = "@BTCCommons_Network"
profile_about = "Network metrics and statistics from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-network.png"
```

---

## 📦 Complete Docker Compose File

```yaml
version: '3.8'

services:
  # Governance App
  governance-app:
    image: ghcr.io/btcdecoded/governance-app:latest
    container_name: blvm-commons
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:///app/data/governance.db
      - NOSTR_ENABLED=true
      - RUST_LOG=info
    volumes:
      - ./governance-data:/app/data
      - ./governance-keys:/app/keys
      - ./governance-config:/app/config
    restart: unless-stopped

  # Archival Node (Base Build)
  archival-node:
    image: ghcr.io/btcdecoded/blvm:latest
    container_name: blvm-archival
    ports:
      - "8332:8332"  # RPC
      - "8333:8333"  # P2P
    volumes:
      - ./archival-data:/app/data
      - ./archival-config:/app/config
    restart: unless-stopped

  # UTXO Commitment Nodes (Experimental Build)
  utxo-node-1:
    image: ghcr.io/btcdecoded/blvm-experimental:latest
    container_name: blvm-utxo-1
    ports:
      - "8335:8335"  # RPC
      - "8334:8334"  # P2P
    volumes:
      - ./utxo-1-data:/app/data
      - ./utxo-1-config:/app/config
    restart: unless-stopped

  utxo-node-2:
    image: ghcr.io/btcdecoded/blvm-experimental:latest
    container_name: blvm-utxo-2
    ports:
      - "8336:8336"  # RPC
      - "8337:8337"  # P2P
    volumes:
      - ./utxo-2-data:/app/data
      - ./utxo-2-config:/app/config
    restart: unless-stopped

  utxo-node-3:
    image: ghcr.io/btcdecoded/blvm-experimental:latest
    container_name: blvm-utxo-3
    ports:
      - "8338:8338"  # RPC
      - "8339:8339"  # P2P
    volumes:
      - ./utxo-3-data:/app/data
      - ./utxo-3-config:/app/config
    restart: unless-stopped
```

---

## 🚀 Quick Start (Path of Least Resistance)

### 1. Generate Nostr Keys (5 minutes)

```bash
cargo install nostr-tool
mkdir -p ~/nostr-keys
for bot in gov dev research network; do
  nostr-tool generate > ~/nostr-keys/${bot}.nsec
  nostr-tool convert ~/nostr-keys/${bot}.nsec > ~/nostr-keys/${bot}.npub
done
cat ~/nostr-keys/*.npub  # Save these for config
```

### 2. Create Deployment Structure (2 minutes)

```bash
mkdir -p ~/btc-commons-deployment/{governance-data,governance-keys/nostr,archival-data,utxo-{1,2,3}-data}
cd ~/btc-commons-deployment

# Copy Nostr keys
cp ~/nostr-keys/*.nsec governance-keys/nostr/

# Copy GitHub App key
cp /path/to/github-app.pem governance-keys/
```

### 3. Create Config Files (10 minutes)

- Governance app config (with multi-bot setup)
- Archival node config
- 3x UTXO commitment node configs

### 4. Deploy with Docker Compose (1 minute)

```bash
docker-compose up -d
docker-compose logs -f
```

### 5. Verify (5 minutes)

```bash
# Governance app
curl http://localhost:8080/health

# Archival node
curl -u btc:password http://localhost:8332 -d '{"method":"getblockchaininfo"}'

# UTXO nodes
curl -u btc:password http://localhost:8335 -d '{"method":"getblockchaininfo"}'
```

---

## 🔒 Security Checklist

- [ ] Change all default passwords
- [ ] Use strong RPC passwords
- [ ] Secure Nostr nsec files (600 permissions)
- [ ] Use firewall (only expose needed ports)
- [ ] Enable SSL/TLS (Nginx reverse proxy)
- [ ] Regular backups (database, keys)
- [ ] Monitor logs for errors
- [ ] Keep images updated

---

## 📊 Monitoring

### Basic Health Checks

```bash
# Governance app
curl http://localhost:8080/health

# Nodes
curl -u btc:password http://localhost:8332 -d '{"method":"getblockchaininfo"}'
```

### Logs

```bash
# All services
docker-compose logs -f

# Individual service
docker-compose logs -f governance-app
docker-compose logs -f archival-node
```

---

## 🎯 What You Get

### After Deployment

1. **Governance App** running on port 8080
   - GitHub webhook handler
   - Nostr publishing (4 bots)
   - Governance enforcement

2. **Archival Node** on port 8333
   - Full blockchain history
   - RPC on 8332
   - Base build (no experimental features)

3. **3x UTXO Commitment Nodes**
   - Faster initial sync
   - Experimental features enabled
   - Different ports (8334-8339)

---

## ⚠️ Important Notes

1. **Archival Node Storage**: Needs ~400GB for full blockchain
2. **UTXO Nodes**: Can be pruned (save space)
3. **Nostr Keys**: Keep nsec files secure, never commit
4. **Ports**: Make sure ports don't conflict
5. **Firewall**: Only expose needed ports (8080 for governance, 8333 for archival)

---

## 🔄 Updates

### Update Governance App

```bash
docker-compose pull governance-app
docker-compose up -d governance-app
```

### Update Nodes

```bash
docker-compose pull archival-node utxo-node-1 utxo-node-2 utxo-node-3
docker-compose up -d
```

---

## 📝 Next Steps

1. **Test Nostr Publishing**: Use GitHub Actions workflow to publish test announcement
2. **Monitor Node Sync**: Check logs for sync progress
3. **Configure Monitoring**: Set up Prometheus/Grafana (optional)
4. **Set Up Backups**: Automate database and key backups
5. **Document Node IPs**: For network connectivity

---

**Status:** ✅ Ready to Deploy  
**Estimated Time:** 30-60 minutes  
**Difficulty:** Easy (Docker) or Medium (Systemd)

