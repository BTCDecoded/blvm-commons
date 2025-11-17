#!/usr/bin/env python3
"""
Export BTCDecoded benchmarks in a format compatible with Bitcoin Core benchmarks.

This script reads Criterion JSON output and converts it to a format that's easier
to compare with Bitcoin Core's benchmark output format.
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, Any
import platform
import subprocess

def get_system_info() -> Dict[str, Any]:
    """Get system information for metadata"""
    info = {
        "platform": platform.platform(),
        "processor": platform.processor(),
        "machine": platform.machine(),
    }
    
    # Try to get Rust version
    try:
        result = subprocess.run(
            ["rustc", "--version"],
            capture_output=True,
            text=True,
            timeout=5
        )
        if result.returncode == 0:
            info["rust_version"] = result.stdout.strip()
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    
    # Try to get CPU info
    try:
        if platform.system() == "Linux":
            with open("/proc/cpuinfo", "r") as f:
                for line in f:
                    if line.startswith("model name"):
                        info["cpu_model"] = line.split(":")[1].strip()
                        break
    except FileNotFoundError:
        pass
    
    return info

def parse_criterion_results(criterion_file: Path) -> Dict[str, Any]:
    """Parse Criterion benchmark results (JSON format)"""
    try:
        with open(criterion_file, 'r') as f:
            data = json.load(f)
        
        results = {}
        if isinstance(data, dict):
            for benchmark_name, benchmark_data in data.items():
                if 'mean' in benchmark_data:
                    mean = benchmark_data['mean']
                    if 'point_estimate' in mean:
                        mean_ns = mean['point_estimate']
                        mean_ms = mean_ns / 1_000_000
                        
                        stats = {
                            "mean_ms": mean_ms,
                            "mean_ns": mean_ns,
                            "min_ms": None,
                            "max_ms": None,
                            "samples": None,
                            "throughput_ops_per_sec": None,
                        }
                        
                        # Extract min/max
                        if 'min' in benchmark_data:
                            stats["min_ms"] = benchmark_data['min'] / 1_000_000
                        if 'max' in benchmark_data:
                            stats["max_ms"] = benchmark_data['max'] / 1_000_000
                        
                        # Extract sample count
                        if 'sample_count' in benchmark_data:
                            stats["samples"] = benchmark_data['sample_count']
                        
                        # Calculate throughput
                        if mean_ns > 0:
                            stats["throughput_ops_per_sec"] = 1_000_000_000 / mean_ns
                        
                        results[benchmark_name] = stats
        
        return results
    except Exception as e:
        print(f"ERROR: Failed to parse Criterion results from {criterion_file}: {e}", file=sys.stderr)
        return {}

def export_benchmarks(
    criterion_file: Path,
    output_file: Path,
    include_metadata: bool = True
) -> None:
    """Export benchmarks in Core-compatible format"""
    
    results = parse_criterion_results(criterion_file)
    
    if not results:
        print(f"ERROR: No benchmark results found in {criterion_file}", file=sys.stderr)
        sys.exit(1)
    
    # Create output structure similar to Core's format
    output = {
        "system": "BTCDecoded",
        "benchmarks": {}
    }
    
    # Add metadata if requested
    if include_metadata:
        output["metadata"] = get_system_info()
        output["metadata"]["source_file"] = str(criterion_file)
    
    # Map BTCDecoded benchmarks to Core-like names
    benchmark_mapping = {
        "check_transaction": "transaction_validation",
        "check_transaction_complex": "transaction_validation_complex",
        "sha256_1kb": "hash_operations_sha256",
        "double_sha256_1kb": "hash_operations_double_sha256",
        "connect_block": "block_validation",
        "connect_block_multi_tx": "block_validation_multi_tx",
    }
    
    # Export benchmarks
    for btdcoded_name, result in results.items():
        # Use mapped name if available, otherwise use original
        core_name = benchmark_mapping.get(btdcoded_name, btdcoded_name)
        
        # Convert to Core-like format (focus on mean_ms, min_ms, max_ms, samples)
        core_format = {
            "mean_ms": result["mean_ms"],
        }
        
        if result.get("min_ms") is not None:
            core_format["min_ms"] = result["min_ms"]
        if result.get("max_ms") is not None:
            core_format["max_ms"] = result["max_ms"]
        if result.get("samples") is not None:
            core_format["samples"] = result["samples"]
        
        # Add throughput as additional info
        if result.get("throughput_ops_per_sec") is not None:
            core_format["throughput_ops_per_sec"] = result["throughput_ops_per_sec"]
        
        # Add original name for reference
        core_format["original_name"] = btdcoded_name
        
        output["benchmarks"][core_name] = core_format
    
    # Write output
    with open(output_file, 'w') as f:
        json.dump(output, f, indent=2)
    
    print(f"✓ Exported {len(output['benchmarks'])} benchmarks to {output_file}")

def main():
    parser = argparse.ArgumentParser(
        description="Export BTCDecoded benchmarks in Core-compatible format"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Input Criterion JSON file (from cargo bench --output-format json)"
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output JSON file"
    )
    parser.add_argument(
        "--no-metadata",
        action="store_true",
        help="Exclude system metadata from output"
    )
    
    args = parser.parse_args()
    
    input_file = Path(args.input)
    output_file = Path(args.output)
    
    if not input_file.exists():
        print(f"ERROR: Input file not found: {input_file}", file=sys.stderr)
        sys.exit(1)
    
    export_benchmarks(
        input_file,
        output_file,
        include_metadata=not args.no_metadata
    )

if __name__ == "__main__":
    main()















