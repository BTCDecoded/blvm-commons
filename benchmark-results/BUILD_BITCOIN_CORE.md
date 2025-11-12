# Building Bitcoin Core for Benchmarks

Bitcoin Core requires several dependencies to build. Here's what's needed:

## Required Dependencies

```bash
# On Arch Linux
sudo pacman -S boost libevent openssl db base-devel cmake

# On Ubuntu/Debian
sudo apt-get install build-essential libtool autotools-dev automake pkg-config libssl-dev libevent-dev bsdmainutils libboost-system-dev libboost-filesystem-dev libboost-chrono-dev libboost-test-dev libboost-thread-dev cmake
```

## Build Steps

```bash
cd /home/user/src/bitcoin
mkdir -p build
cd build
cmake .. -DBUILD_BITCOIN_CLI=ON -DBUILD_BITCOIND=ON -DENABLE_WALLET=OFF
make -j$(nproc) bitcoind bitcoin-cli
```

## After Building

Once built, the binaries will be at:
- `build/src/bitcoind`
- `build/src/bitcoin-cli`

Then run:
```bash
cd /home/user/src/BTCDecoded
export BITCOIN_CORE_BIN=/home/user/src/bitcoin/build/src/bitcoind
export BITCOIN_CLI_BIN=/home/user/src/bitcoin/build/src/bitcoin-cli
./scripts/benchmark_comparison.sh
```
