#!/usr/bin/env python3
"""
Lightweight Bitcoin Core Benchmarking Script
Designed to run on a laptop with minimal resource usage
"""

import argparse
import json
import subprocess
import tempfile
import shutil
import os
import time
import sys
from pathlib import Path
from typing import Dict, List, Any

class BitcoinCoreBenchmark:
    """Benchmark Bitcoin Core validation operations"""
    
    def __init__(self, bitcoind_path: str, bitcoin_cli_path: str, data_dir: str):
        self.bitcoind_path = Path(bitcoind_path)
        self.bitcoin_cli_path = Path(bitcoin_cli_path)
        self.data_dir = Path(data_dir)
        self.regtest_dir = self.data_dir / "regtest"
        # Use a random port to avoid conflicts
        import random
        self.rpc_port = 18444 + random.randint(1, 1000)  # Avoid conflicts
        self.process = None
        
    def _check_binary(self, path: Path) -> bool:
        """Check if binary exists and is executable"""
        return path.exists() and os.access(path, os.X_OK)
    
    def start_bitcoind(self) -> bool:
        """Start bitcoind in regtest mode"""
        if not self._check_binary(self.bitcoind_path):
            print(f"ERROR: bitcoind not found or not executable: {self.bitcoind_path}")
            return False
            
        # Ensure data directory exists
        self.data_dir.mkdir(parents=True, exist_ok=True)
        # Clean up old regtest directory
        if self.regtest_dir.exists():
            shutil.rmtree(self.regtest_dir)
        self.regtest_dir.mkdir(parents=True, exist_ok=True)
        
        # Start bitcoind in regtest mode
        cmd = [
            str(self.bitcoind_path),
            "-regtest",
            f"-datadir={self.data_dir}",
            f"-rpcport={self.rpc_port}",
            "-rpcuser=bench",
            "-rpcpassword=bench",
            "-server",
            "-daemon",
            "-fallbackfee=0.00001",
            "-txindex=0",  # Disable txindex for faster startup
            "-prune=0",   # No pruning
        ]
        
        try:
            result = subprocess.run(cmd, check=True, capture_output=True, text=True)
            # Wait for bitcoind to start (give it more time)
            print("  → Waiting for bitcoind to start...")
            for i in range(20):  # Wait up to 20 seconds
                time.sleep(1)
                if self.rpc_call("getblockcount") is not None:
                    print(f"  ✓ bitcoind started (after {i+1}s)")
                    return True
            print("  ⚠ bitcoind started but not responding to RPC")
            return False
        except subprocess.CalledProcessError as e:
            print(f"ERROR: Failed to start bitcoind: {e}")
            if e.stderr:
                print(f"stderr: {e.stderr}")
            if e.stdout:
                print(f"stdout: {e.stdout}")
            return False
    
    def stop_bitcoind(self):
        """Stop bitcoind"""
        if not self._check_binary(self.bitcoin_cli_path):
            return
            
        cmd = [
            str(self.bitcoin_cli_path),
            "-regtest",
            f"-datadir={self.data_dir}",
            f"-rpcport={self.rpc_port}",
            "-rpcuser=bench",
            "-rpcpassword=bench",
            "stop"
        ]
        
        try:
            subprocess.run(cmd, check=True, capture_output=True, timeout=10)
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired):
            pass  # Already stopped or taking time
    
    def rpc_call(self, method: str, *args) -> Any:
        """Make an RPC call to bitcoind"""
        cmd = [
            str(self.bitcoin_cli_path),
            "-regtest",
            f"-datadir={self.data_dir}",
            f"-rpcport={self.rpc_port}",
            "-rpcuser=bench",
            "-rpcpassword=bench",
            method,
            *[str(arg) for arg in args]
        ]
        
        try:
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
            if result.returncode == 0:
                try:
                    return json.loads(result.stdout)
                except json.JSONDecodeError:
                    return result.stdout.strip()
            return None
        except subprocess.TimeoutExpired:
            return None
    
    def benchmark_transaction_validation(self) -> Dict[str, Any]:
        """Benchmark transaction validation (lightweight)"""
        print("  → Benchmarking transaction validation...")
        
        # Generate blocks to have some UTXOs (mining rewards)
        result = self.rpc_call("generatetoaddress", 101, self.rpc_call("getnewaddress"))
        if not result:
            return {"error": "Failed to generate blocks"}
        
        # Get a UTXO to spend
        utxos = self.rpc_call("listunspent", 0)
        if not utxos or len(utxos) == 0:
            return {"error": "No UTXOs available"}
        
        # Create a transaction
        address = self.rpc_call("getnewaddress")
        utxo = utxos[0]
        
        # Create raw transaction
        raw_tx = self.rpc_call("createrawtransaction", 
                               [{"txid": utxo["txid"], "vout": utxo["vout"]}],
                               {address: 0.001})
        
        if not raw_tx:
            return {"error": "Failed to create raw transaction"}
        
        # Sign transaction
        signed_tx = self.rpc_call("signrawtransactionwithwallet", raw_tx)
        if not signed_tx or "hex" not in signed_tx:
            return {"error": "Failed to sign transaction"}
        
        # Benchmark transaction validation via testmempoolaccept
        times = []
        for _ in range(50):  # Reduced for faster execution
            start = time.perf_counter()
            result = self.rpc_call("testmempoolaccept", [signed_tx["hex"]])
            end = time.perf_counter()
            if result and len(result) > 0:
                times.append((end - start) * 1000)  # Convert to milliseconds
        
        if not times:
            return {"error": "No valid measurements"}
        
        return {
            "mean_ms": sum(times) / len(times),
            "min_ms": min(times),
            "max_ms": max(times),
            "samples": len(times)
        }
    
    def benchmark_block_validation(self) -> Dict[str, Any]:
        """Benchmark block validation (lightweight)"""
        print("  → Benchmarking block validation...")
        
        # Generate blocks and measure time
        address = self.rpc_call("getnewaddress")
        times = []
        for _ in range(10):  # Small number for laptop
            start = time.perf_counter()
            result = self.rpc_call("generatetoaddress", 1, address)
            end = time.perf_counter()
            if result:
                times.append((end - start) * 1000)  # Convert to milliseconds
        
        if not times:
            return {"error": "No valid measurements"}
        
        return {
            "mean_ms": sum(times) / len(times),
            "min_ms": min(times),
            "max_ms": max(times),
            "samples": len(times)
        }
    
    def benchmark_hash_operations(self) -> Dict[str, Any]:
        """Benchmark hash operations via RPC (limited)"""
        print("  → Benchmarking hash operations...")
        
        # Bitcoin Core doesn't expose hash operations via RPC directly
        # We'll use getblockhash as a proxy (it uses SHA256)
        # First ensure we have blocks
        current_height = self.rpc_call("getblockcount")
        if current_height is None or current_height < 100:
            address = self.rpc_call("getnewaddress")
            self.rpc_call("generatetoaddress", 100, address)
        
        times = []
        for i in range(1, 101):  # Small range for laptop
            start = time.perf_counter()
            hash_result = self.rpc_call("getblockhash", i)
            end = time.perf_counter()
            if hash_result:
                times.append((end - start) * 1000)  # Convert to milliseconds
        
        if not times:
            return {"error": "No valid measurements"}
        
        return {
            "mean_ms": sum(times) / len(times),
            "min_ms": min(times),
            "max_ms": max(times),
            "samples": len(times),
            "note": "Using getblockhash as proxy for SHA256 operations"
        }
    
    def run_all_benchmarks(self) -> Dict[str, Any]:
        """Run all benchmarks"""
        results = {
            "timestamp": time.time(),
            "system": "Bitcoin Core",
            "benchmarks": {}
        }
        
        # Start bitcoind
        if not self.start_bitcoind():
            return {"error": "Failed to start bitcoind"}
        
        try:
            # Wait for bitcoind to be ready (already checked in start_bitcoind, but double-check)
            for i in range(10):
                if self.rpc_call("getblockcount") is not None:
                    break
                time.sleep(1)
            else:
                return {"error": "bitcoind not responding after start"}
            
            # Run benchmarks
            results["benchmarks"]["transaction_validation"] = self.benchmark_transaction_validation()
            results["benchmarks"]["block_validation"] = self.benchmark_block_validation()
            results["benchmarks"]["hash_operations"] = self.benchmark_hash_operations()
            
        finally:
            self.stop_bitcoind()
            # Clean up data directory
            if self.regtest_dir.exists():
                shutil.rmtree(self.regtest_dir)
        
        return results

def main():
    parser = argparse.ArgumentParser(description="Benchmark Bitcoin Core")
    parser.add_argument("--bitcoind", required=True, help="Path to bitcoind binary")
    parser.add_argument("--bitcoin-cli", required=True, help="Path to bitcoin-cli binary")
    parser.add_argument("--data-dir", required=True, help="Data directory for regtest")
    parser.add_argument("--output", required=True, help="Output JSON file")
    
    args = parser.parse_args()
    
    benchmark = BitcoinCoreBenchmark(
        args.bitcoind,
        args.bitcoin_cli,
        args.data_dir
    )
    
    results = benchmark.run_all_benchmarks()
    
    # Write results
    with open(args.output, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\n✓ Results saved to: {args.output}")
    
    if "error" in results:
        print(f"ERROR: {results['error']}")
        sys.exit(1)

if __name__ == "__main__":
    main()

