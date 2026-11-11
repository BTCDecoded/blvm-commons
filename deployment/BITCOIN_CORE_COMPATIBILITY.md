# BLLVM and Bitcoin Core Compatibility

## ⚠️ Port Conflict Issue

**Both BLLVM and Bitcoin Core use the same default ports:**
- **RPC Port:** 8332 (both)
- **P2P Port:** 8333 (both)

**They CANNOT run simultaneously on the same ports.**

---

## ✅ Solution: Use Different Ports

### Option 1: Configure BLLVM to Use Different Ports (RECOMMENDED)

**Modify BLLVM config after installation:**

```bash
# Edit BLLVM config
sudo nano /etc/blvm/blvm.toml
```

**Use real `blvm.toml` keys** (`listen_addr`, `protocol_version`, …) and set **JSON-RPC** bind via **`blvm --rpc-addr`** (or `BLVM_RPC_ADDR`). Example alternate ports:

```toml
listen_addr = "0.0.0.0:8334"
protocol_version = "BitcoinV1"
transport_preference = "tcponly"
```

```bash
blvm --config /etc/blvm/blvm.toml --rpc-addr 0.0.0.0:8335
```

**Restart BLLVM:**

```bash
sudo systemctl restart blvm
```

### Option 2: Configure Bitcoin Core to Use Different Ports

**Edit Bitcoin Core config:**

```bash
# Find Bitcoin Core config (varies by installation)
# Common locations:
# - ~/.bitcoin/bitcoin.conf
# - /embassy-data/package-data/volumes/bitcoin*/bitcoin.conf

# Add to bitcoin.conf:
rpcport=8334      # Different from BLLVM's 8332
port=8335         # Different from BLLVM's 8333
```

**Restart Bitcoin Core** (via StartOS or systemctl)

---

## ✅ What Works When Running Both

### ✅ Can Share:
- **Disk space** (separate data directories)
- **Network connection** (different ports)
- **CPU/RAM** (both will use resources, but can coexist)

### ❌ Cannot Share:
- **Same ports** (must use different ports)
- **Same data directory** (each needs its own blockchain copy)
- **Same RPC credentials** (each has its own user/password)

---

## Recommended Configuration

### Bitcoin Core (Default)
- RPC: `localhost:8332`
- P2P: `0.0.0.0:8333`
- Data: `~/.bitcoin` or `/embassy-data/package-data/volumes/bitcoin*/`

### BLLVM (Modified)
- RPC: `localhost:8335` (or 8334)
- P2P: `0.0.0.0:8334` (or 8335)
- Data: `/var/lib/blvm`

**Note:** Make sure P2P ports don't conflict. If Bitcoin Core uses 8333, BLLVM should use 8334 (or vice versa).

---

## Resource Usage

**Running both simultaneously will:**
- ✅ Use ~1.2TB disk space (600GB each)
- ✅ Use more CPU/RAM (both nodes syncing)
- ✅ Use more network bandwidth (both connecting to peers)
- ✅ Take longer to sync (both downloading blockchain)

**Benefits:**
- ✅ Redundancy (two independent nodes)
- ✅ Comparison/validation between implementations
- ✅ Can use one for specific tasks (e.g., Bitcoin Core for wallet, BLLVM for governance)

---

## Installation Steps (Updated)

### 1. Check Current Bitcoin Core Ports

```bash
# Check Bitcoin Core config
grep -E "rpcport|port" ~/.bitcoin/bitcoin.conf 2>/dev/null || \
grep -E "rpcport|port" /embassy-data/package-data/volumes/bitcoin*/bitcoin.conf 2>/dev/null

# Or check what ports are in use
sudo netstat -tlnp | grep -E "8332|8333"
```

### 2. Install BLLVM with Different Ports

**Option A: Install normally, then modify config**

```bash
# Install BLLVM (will use default ports 8332/8333)
sudo ./blvm.sh install blvm --public-ip 192.168.2.101

# Edit config to use different ports
sudo nano /etc/blvm/blvm.toml
# Change ports as shown above

# Restart
sudo systemctl restart blvm
```

**Option B: Modify installer to support custom ports (future enhancement)**

Currently, the installer may not expose dedicated **`--rpc-port`** / P2P flags. To change ports after install:
1. Set **`listen_addr`** in `blvm.toml` for P2P
2. Set JSON-RPC bind via **`ExecStart=... --rpc-addr HOST:PORT`** (systemd) or **`BLVM_RPC_ADDR`**
3. Restart the service

---

## Verification

**Check both are running on different ports:**

```bash
# Check Bitcoin Core
sudo netstat -tlnp | grep 8332  # RPC
sudo netstat -tlnp | grep 8333  # P2P

# Check BLVM on the alternate ports in this example
sudo netstat -tlnp | grep 8335  # RPC
sudo netstat -tlnp | grep 8334  # P2P
```

**Test RPC on both.** With no RPC password, each process writes `{datadir}/.cookie` as `__cookie__:<hex>`.

```bash
# Bitcoin Core (default datadir)
curl --user "__cookie__:$(cut -d: -f2 ~/.bitcoin/.cookie)" http://127.0.0.1:8332 -d '{"method":"getblockchaininfo","params":[]}'

# BLVM (this example's alternate RPC port). Set BLVM_DATADIR to that node's data directory.
curl --user "__cookie__:$(cut -d: -f2 "$BLVM_DATADIR/.cookie")" http://127.0.0.1:8335 -d '{"method":"getblockchaininfo","params":[]}'
```

---

## Summary

✅ **YES, they can run simultaneously** if:
- Configured with different ports
- Have separate data directories
- Have sufficient disk space (~1.2TB)
- Have sufficient CPU/RAM

⚠️ **Port conflict must be resolved** before running both.

**Recommended:** Use Bitcoin Core default ports (8332/8333) and configure BLLVM to use alternative ports (8334/8335 or similar).

