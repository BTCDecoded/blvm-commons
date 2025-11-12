#!/usr/bin/env python3
"""
View benchmark comparison results in a human-readable format
"""

import argparse
import json
import sys
from pathlib import Path

def format_number(value: float, unit: str = "") -> str:
    """Format a number with appropriate precision"""
    if value >= 1000:
        return f"{value/1000:.2f} {unit}"
    elif value >= 1:
        return f"{value:.2f} {unit}"
    elif value >= 0.001:
        return f"{value*1000:.2f} m{unit}"
    else:
        return f"{value*1000000:.2f} µ{unit}"

def format_time(ns: float) -> str:
    """Format time in nanoseconds to human-readable format"""
    if ns >= 1_000_000_000:
        return f"{ns/1_000_000_000:.3f} s"
    elif ns >= 1_000_000:
        return f"{ns/1_000_000:.3f} ms"
    elif ns >= 1_000:
        return f"{ns/1_000:.3f} µs"
    else:
        return f"{ns:.2f} ns"

def view_comparison(results_file: Path):
    """View comparison results"""
    try:
        with open(results_file, 'r') as f:
            data = json.load(f)
    except Exception as e:
        print(f"ERROR: Failed to load results: {e}")
        sys.exit(1)
    
    print("=" * 80)
    print("Benchmark Comparison: Bitcoin Core vs BTCDecoded")
    print("=" * 80)
    print()
    
    if "comparison" not in data:
        print("No comparison data found")
        return
    
    comp = data["comparison"]
    
    # Transaction validation
    if "transaction_validation" in comp:
        tv = comp["transaction_validation"]
        print("Transaction Validation:")
        print(f"  BTCDecoded:  {format_time(tv.get('btdcoded_ms', 0) * 1_000_000)}")
        print(f"  Bitcoin Core: {format_time(tv.get('core_ms', 0) * 1_000_000)}")
        if 'ratio' in tv:
            print(f"  Ratio: {tv['ratio']:.2f}x ({tv.get('faster', 'Unknown')} is faster)")
        print()
    
    # Hash operations
    if "hash_operations" in comp:
        ho = comp["hash_operations"]
        print("Hash Operations:")
        print(f"  BTCDecoded:  {format_time(ho.get('btdcoded_ms', 0) * 1_000_000)}")
        print(f"  Bitcoin Core: {format_time(ho.get('core_ms', 0) * 1_000_000)}")
        if 'ratio' in ho:
            print(f"  Ratio: {ho['ratio']:.2f}x ({ho.get('faster', 'Unknown')} is faster)")
        print()
    
    # Block validation (handle multiple block validation benchmarks)
    block_keys = [k for k in comp.keys() if k.startswith("block_validation_")]
    if block_keys:
        print("Block Validation:")
        for key in sorted(block_keys):
            bv = comp[key]
            benchmark_name = key.replace('block_validation_', '').replace('_', ' ').title()
            print(f"  {benchmark_name}:")
            print(f"    BTCDecoded:  {format_time(bv.get('btdcoded_ms', 0) * 1_000_000)}")
            if 'core_ms' in bv:
                print(f"    Bitcoin Core: {format_time(bv.get('core_ms', 0) * 1_000_000)}")
                if 'ratio' in bv:
                    print(f"    Ratio: {bv['ratio']:.2f}x ({bv.get('faster', 'Unknown')} is faster)")
            if 'btdcoded_throughput' in bv:
                throughput = bv['btdcoded_throughput']
                if throughput > 1_000_000:
                    print(f"    BTCDecoded Throughput: {throughput/1_000_000:.2f}M ops/sec")
                elif throughput > 1_000:
                    print(f"    BTCDecoded Throughput: {throughput/1_000:.2f}K ops/sec")
                else:
                    print(f"    BTCDecoded Throughput: {throughput:.2f} ops/sec")
            if 'note' in bv:
                print(f"    Note: {bv['note']}")
            print()
    
    # Summary
    print("=" * 80)
    print("Note: These are lightweight benchmarks suitable for laptop testing.")
    print("For production performance testing, use full blockchain data.")
    print("=" * 80)

def main():
    parser = argparse.ArgumentParser(description="View benchmark comparison results")
    parser.add_argument("results_file", help="Comparison results JSON file")
    
    args = parser.parse_args()
    
    results_file = Path(args.results_file)
    if not results_file.exists():
        print(f"ERROR: Results file not found: {results_file}")
        sys.exit(1)
    
    view_comparison(results_file)

if __name__ == "__main__":
    main()





