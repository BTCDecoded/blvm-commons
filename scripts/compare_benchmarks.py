#!/usr/bin/env python3
"""
Compare benchmark results between Bitcoin Core and BTCDecoded
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, Any, List

def load_benchmark_results(file_path: Path) -> Dict[str, Any]:
    """Load benchmark results from JSON file"""
    try:
        with open(file_path, 'r') as f:
            return json.load(f)
    except Exception as e:
        print(f"ERROR: Failed to load {file_path}: {e}")
        return {}

def parse_criterion_results(criterion_file: Path) -> Dict[str, Any]:
    """Parse Criterion benchmark results (JSON format)
    
    Extracts comprehensive statistics from Criterion JSON output:
    - Mean, min, max, median
    - Confidence intervals
    - Throughput (ops/sec)
    - Sample count
    """
    try:
        with open(criterion_file, 'r') as f:
            data = json.load(f)
        
        results = {}
        # Criterion JSON format has benchmark results in a specific structure
        if isinstance(data, dict):
            for benchmark_name, benchmark_data in data.items():
                if 'mean' in benchmark_data:
                    mean = benchmark_data['mean']
                    if 'point_estimate' in mean:
                        mean_ns = mean['point_estimate']
                        
                        # Extract additional statistics
                        stats = {
                            'mean_ns': mean_ns,
                            'mean_ms': mean_ns / 1_000_000,
                            'lower_bound_ns': mean.get('confidence_interval', {}).get('lower_bound'),
                            'upper_bound_ns': mean.get('confidence_interval', {}).get('upper_bound'),
                        }
                        
                        # Extract min/max if available
                        if 'min' in benchmark_data:
                            stats['min_ns'] = benchmark_data['min']
                            stats['min_ms'] = benchmark_data['min'] / 1_000_000
                        if 'max' in benchmark_data:
                            stats['max_ns'] = benchmark_data['max']
                            stats['max_ms'] = benchmark_data['max'] / 1_000_000
                        
                        # Extract median if available
                        if 'median' in benchmark_data:
                            median = benchmark_data['median']
                            if 'point_estimate' in median:
                                stats['median_ns'] = median['point_estimate']
                                stats['median_ms'] = median['point_estimate'] / 1_000_000
                        
                        # Calculate throughput (ops/sec)
                        if mean_ns > 0:
                            stats['throughput_ops_per_sec'] = 1_000_000_000 / mean_ns
                        
                        # Extract sample count if available
                        if 'sample_count' in benchmark_data:
                            stats['samples'] = benchmark_data['sample_count']
                        
                        results[benchmark_name] = stats
        
        return results
    except Exception as e:
        print(f"WARNING: Failed to parse Criterion results from {criterion_file}: {e}")
        import traceback
        traceback.print_exc()
        return {}

def find_benchmark_files(directory: Path) -> Dict[str, Path]:
    """Find benchmark result files in directory"""
    files = {}
    
    # Look for JSON files
    for json_file in directory.glob("*.json"):
        name = json_file.stem
        files[name] = json_file
    
    return files

def compare_benchmarks(btdcoded_dir: Path, core_dir: Path) -> Dict[str, Any]:
    """Compare benchmark results"""
    comparison = {
        "btdcoded": {},
        "bitcoin_core": {},
        "comparison": {}
    }
    
    # Load BTCDecoded results
    btdcoded_files = find_benchmark_files(btdcoded_dir)
    for name, file_path in btdcoded_files.items():
        if name == "consensus_proof":
            comparison["btdcoded"]["consensus_proof"] = parse_criterion_results(file_path)
        elif name == "reference_node":
            comparison["btdcoded"]["reference_node"] = parse_criterion_results(file_path)
    
    # Load Bitcoin Core results
    core_files = find_benchmark_files(core_dir)
    if "bitcoin_core" in core_files:
        comparison["bitcoin_core"] = load_benchmark_results(core_files["bitcoin_core"])
    
    # Compare common benchmarks
    comparison["comparison"] = compare_common_metrics(
        comparison["btdcoded"],
        comparison["bitcoin_core"]
    )
    
    return comparison

def compare_common_metrics(btdcoded: Dict, core: Dict) -> Dict[str, Any]:
    """Compare common metrics between systems"""
    comparison = {}
    
    # Transaction validation comparison
    if "consensus_proof" in btdcoded and "check_transaction" in btdcoded["consensus_proof"]:
        btdcoded_tx = btdcoded["consensus_proof"]["check_transaction"]
        if "benchmarks" in core and "transaction_validation" in core["benchmarks"]:
            core_tx = core["benchmarks"]["transaction_validation"]
            if "mean_ms" in core_tx:
                btdcoded_ms = btdcoded_tx.get("mean_ms", btdcoded_tx.get("mean_ns", 0) / 1_000_000)
                core_ms = core_tx["mean_ms"]
                
                comparison["transaction_validation"] = {
                    "btdcoded_ms": btdcoded_ms,
                    "core_ms": core_ms,
                    "ratio": core_ms / btdcoded_ms if btdcoded_ms > 0 else 0,
                    "faster": "BTCDecoded" if btdcoded_ms < core_ms else "Bitcoin Core",
                    "btdcoded_throughput": btdcoded_tx.get("throughput_ops_per_sec", 0),
                }
    
    # Hash operations comparison
    if "consensus_proof" in btdcoded and "sha256_1kb" in btdcoded["consensus_proof"]:
        btdcoded_hash = btdcoded["consensus_proof"]["sha256_1kb"]
        if "benchmarks" in core and "hash_operations" in core["benchmarks"]:
            core_hash = core["benchmarks"]["hash_operations"]
            if "mean_ms" in core_hash:
                btdcoded_ms = btdcoded_hash.get("mean_ms", btdcoded_hash.get("mean_ns", 0) / 1_000_000)
                core_ms = core_hash["mean_ms"]
                
                comparison["hash_operations"] = {
                    "btdcoded_ms": btdcoded_ms,
                    "core_ms": core_ms,
                    "ratio": core_ms / btdcoded_ms if btdcoded_ms > 0 else 0,
                    "faster": "BTCDecoded" if btdcoded_ms < core_ms else "Bitcoin Core",
                    "btdcoded_throughput": btdcoded_hash.get("throughput_ops_per_sec", 0),
                    "note": "Core measures cached getblockhash (database lookup), BTCDecoded measures actual SHA256 computation"
                }
    
    # Block validation comparison
    if "consensus_proof" in btdcoded:
        # Try to find block validation benchmarks
        block_benchmarks = {
            "connect_block": "connect_block",
            "connect_block_multi_tx": "connect_block_multi_tx",
        }
        
        for btdcoded_key, comparison_key in block_benchmarks.items():
            if btdcoded_key in btdcoded["consensus_proof"]:
                btdcoded_block = btdcoded["consensus_proof"][btdcoded_key]
                if "benchmarks" in core and "block_validation" in core["benchmarks"]:
                    core_block = core["benchmarks"]["block_validation"]
                    if "mean_ms" in core_block:
                        btdcoded_ms = btdcoded_block.get("mean_ms", btdcoded_block.get("mean_ns", 0) / 1_000_000)
                        core_ms = core_block["mean_ms"]
                        
                        comparison[f"block_validation_{comparison_key}"] = {
                            "btdcoded_ms": btdcoded_ms,
                            "core_ms": core_ms,
                            "ratio": core_ms / btdcoded_ms if btdcoded_ms > 0 else 0,
                            "faster": "BTCDecoded" if btdcoded_ms < core_ms else "Bitcoin Core",
                            "btdcoded_throughput": btdcoded_block.get("throughput_ops_per_sec", 0),
                        }
                else:
                    # Include BTCDecoded results even without Core comparison
                    btdcoded_ms = btdcoded_block.get("mean_ms", btdcoded_block.get("mean_ns", 0) / 1_000_000)
                    comparison[f"block_validation_{comparison_key}"] = {
                        "btdcoded_ms": btdcoded_ms,
                        "btdcoded_throughput": btdcoded_block.get("throughput_ops_per_sec", 0),
                        "note": "Core benchmark not available for comparison"
                    }
    
    return comparison

def format_comparison(comparison: Dict[str, Any]) -> str:
    """Format comparison results as human-readable text"""
    output = []
    output.append("=" * 60)
    output.append("Benchmark Comparison: Bitcoin Core vs BTCDecoded")
    output.append("=" * 60)
    output.append("")
    
    if "comparison" in comparison:
        comp = comparison["comparison"]
        
        if "transaction_validation" in comp:
            tv = comp["transaction_validation"]
            output.append("Transaction Validation:")
            output.append(f"  BTCDecoded:  {tv['btdcoded_ms']:.4f} ms")
            if 'core_ms' in tv:
                output.append(f"  Bitcoin Core: {tv['core_ms']:.4f} ms")
                output.append(f"  Ratio: {tv['ratio']:.2f}x")
                output.append(f"  Faster: {tv['faster']}")
            if 'btdcoded_throughput' in tv:
                throughput = tv['btdcoded_throughput']
                if throughput > 1_000_000:
                    output.append(f"  BTCDecoded Throughput: {throughput/1_000_000:.2f}M ops/sec")
                elif throughput > 1_000:
                    output.append(f"  BTCDecoded Throughput: {throughput/1_000:.2f}K ops/sec")
                else:
                    output.append(f"  BTCDecoded Throughput: {throughput:.2f} ops/sec")
            output.append("")
        
        if "hash_operations" in comp:
            ho = comp["hash_operations"]
            output.append("Hash Operations:")
            output.append(f"  BTCDecoded:  {ho['btdcoded_ms']:.4f} ms")
            if 'core_ms' in ho:
                output.append(f"  Bitcoin Core: {ho['core_ms']:.4f} ms")
                output.append(f"  Ratio: {ho['ratio']:.2f}x")
                output.append(f"  Faster: {ho['faster']}")
            if 'btdcoded_throughput' in ho:
                throughput = ho['btdcoded_throughput']
                if throughput > 1_000_000:
                    output.append(f"  BTCDecoded Throughput: {throughput/1_000_000:.2f}M ops/sec")
                elif throughput > 1_000:
                    output.append(f"  BTCDecoded Throughput: {throughput/1_000:.2f}K ops/sec")
                else:
                    output.append(f"  BTCDecoded Throughput: {throughput:.2f} ops/sec")
            if 'note' in ho:
                output.append(f"  Note: {ho['note']}")
            output.append("")
        
        # Block validation comparisons
        block_keys = [k for k in comp.keys() if k.startswith("block_validation_")]
        if block_keys:
            output.append("Block Validation:")
            for key in sorted(block_keys):
                bv = comp[key]
                output.append(f"  {key.replace('block_validation_', '').replace('_', ' ').title()}:")
                output.append(f"    BTCDecoded:  {bv['btdcoded_ms']:.4f} ms")
                if 'core_ms' in bv:
                    output.append(f"    Bitcoin Core: {bv['core_ms']:.4f} ms")
                    output.append(f"    Ratio: {bv['ratio']:.2f}x")
                    output.append(f"    Faster: {bv['faster']}")
                if 'btdcoded_throughput' in bv:
                    throughput = bv['btdcoded_throughput']
                    if throughput > 1_000_000:
                        output.append(f"    BTCDecoded Throughput: {throughput/1_000_000:.2f}M ops/sec")
                    elif throughput > 1_000:
                        output.append(f"    BTCDecoded Throughput: {throughput/1_000:.2f}K ops/sec")
                    else:
                        output.append(f"    BTCDecoded Throughput: {throughput:.2f} ops/sec")
                if 'note' in bv:
                    output.append(f"    Note: {bv['note']}")
            output.append("")
    
    return "\n".join(output)

def main():
    parser = argparse.ArgumentParser(description="Compare benchmark results")
    parser.add_argument("--btdcoded", required=True, help="BTCDecoded benchmark results directory")
    parser.add_argument("--bitcoin-core", required=True, help="Bitcoin Core benchmark results directory")
    parser.add_argument("--output", required=True, help="Output JSON file")
    parser.add_argument("--text", action="store_true", help="Also output text format")
    
    args = parser.parse_args()
    
    btdcoded_dir = Path(args.btdcoded)
    core_dir = Path(args.bitcoin_core)
    
    if not btdcoded_dir.exists():
        print(f"ERROR: BTCDecoded results directory not found: {btdcoded_dir}")
        sys.exit(1)
    
    if not core_dir.exists():
        print(f"ERROR: Bitcoin Core results directory not found: {core_dir}")
        sys.exit(1)
    
    comparison = compare_benchmarks(btdcoded_dir, core_dir)
    
    # Write JSON output
    with open(args.output, 'w') as f:
        json.dump(comparison, f, indent=2)
    
    print(f"✓ Comparison saved to: {args.output}")
    
    # Output text format if requested
    if args.text:
        print("\n" + format_comparison(comparison))

if __name__ == "__main__":
    main()





