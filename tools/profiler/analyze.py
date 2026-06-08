#!/usr/bin/env python3
"""
Trace aggregation and ranking analyzer.

Reads raw x87 trace data and CPU sample data, aggregates by return address,
cross-references with address map, and produces a ranked JSON report.
"""

import argparse
import glob
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

from schema import merge_reports, validate_report


def load_json_file(path: Path) -> dict:
    """Load and parse a JSON file."""
    with open(path) as f:
        return json.load(f)


def find_latest_file(pattern: str) -> Optional[Path]:
    """Find the most recent file matching a glob pattern."""
    files = glob.glob(pattern)
    if not files:
        return None
    return Path(max(files, key=os.path.getmtime))


def load_address_map(path: Path) -> dict:
    """Load address-to-function mapping."""
    if not path.exists():
        print(f"Warning: Address map not found at {path}", file=sys.stderr)
        return {}
    
    with open(path) as f:
        data = json.load(f)
    
    # Support both {addr: name} and {functions: [{address, name}]}
    if isinstance(data, dict) and "functions" in data:
        return {
            f["address"]: f.get("name", f"sub_{f['address']}")
            for f in data["functions"]
        }
    
    return data


def analyze(
    trace_files: list[Path],
    sample_files: list[Path],
    address_map: dict,
    wow_version: str = "3.3.5a",
    profile_duration_seconds: int = 300,
) -> dict:
    """
    Aggregate traces and samples into a ranked report.
    
    Args:
        trace_files: List of raw trace JSON files
        sample_files: List of CPU sample JSON files
        address_map: Mapping of addresses to function names
        wow_version: WoW client version
        profile_duration_seconds: Total profiling duration
        
    Returns:
        Complete profiling report
    """
    # Merge all trace data
    merged_trace = {"calls": []}
    for trace_file in trace_files:
        data = load_json_file(trace_file)
        merged_trace["calls"].extend(data.get("calls", []))
    
    # Merge all sample data
    merged_samples = {"samples": []}
    for sample_file in sample_files:
        data = load_json_file(sample_file)
        merged_samples["samples"].extend(data.get("samples", []))
    
    # Generate report
    report = merge_reports(
        x87_trace_data=merged_trace,
        cpu_sample_data=merged_samples,
        address_map=address_map,
        wow_version=wow_version,
        profile_duration_seconds=profile_duration_seconds,
    )
    
    return report


def save_report(report: dict, output_dir: Path) -> Path:
    """Save report to a JSON file."""
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    output_file = output_dir / f"report_{timestamp}.json"
    
    output_dir.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, "w") as f:
        json.dump(report, f, indent=2)
    
    return output_file


def main():
    parser = argparse.ArgumentParser(
        description="Aggregate profiling traces into a ranked report"
    )
    parser.add_argument(
        "--trace", "-t",
        nargs="+",
        type=Path,
        help="Raw trace JSON file(s) from trace_x87.py",
    )
    parser.add_argument(
        "--samples", "-s",
        nargs="+",
        type=Path,
        help="CPU sample JSON file(s) from sample_cpu.py",
    )
    parser.add_argument(
        "--address-map", "-a",
        type=Path,
        default=Path("data/profiling/address_map.json"),
        help="Address-to-function mapping JSON file",
    )
    parser.add_argument(
        "--output-dir", "-o",
        type=Path,
        default=Path("data/profiling/reports"),
        help="Output directory for reports",
    )
    parser.add_argument(
        "--wow-version",
        default="3.3.5a",
        help="WoW client version",
    )
    parser.add_argument(
        "--duration", "-d",
        type=int,
        default=300,
        help="Profiling duration in seconds",
    )
    parser.add_argument(
        "--auto",
        action="store_true",
        help="Auto-discover latest trace and sample files",
    )
    
    args = parser.parse_args()
    
    try:
        # Auto-discover files if requested
        if args.auto:
            trace_pattern = "data/profiling/raw_trace_*.json"
            sample_pattern = "data/profiling/cpu_samples_*.json"
            
            trace_files = [Path(f) for f in glob.glob(trace_pattern)]
            sample_files = [Path(f) for f in glob.glob(sample_pattern)]
            
            if not trace_files:
                print(f"No trace files found matching {trace_pattern}", file=sys.stderr)
                return 1
            if not sample_files:
                print(f"No sample files found matching {sample_pattern}", file=sys.stderr)
                return 1
            
            print(f"Found {len(trace_files)} trace file(s) and {len(sample_files)} sample file(s)")
        else:
            trace_files = args.trace or []
            sample_files = args.samples or []
            
            if not trace_files:
                print("No trace files specified. Use --trace or --auto", file=sys.stderr)
                return 1
        
        # Load address map
        address_map = load_address_map(args.address_map)
        print(f"Loaded {len(address_map)} address mappings")
        
        # Analyze
        print("Analyzing traces and samples...")
        report = analyze(
            trace_files=trace_files,
            sample_files=sample_files,
            address_map=address_map,
            wow_version=args.wow_version,
            profile_duration_seconds=args.duration,
        )
        
        # Validate
        is_valid, errors = validate_report(report)
        if not is_valid:
            print("Generated invalid report:", file=sys.stderr)
            for error in errors:
                print(f"  - {error}", file=sys.stderr)
            return 1
        
        # Save
        output_dir = args.output_dir.resolve()
        output_file = save_report(report, output_dir)
        
        # Summary
        print(f"\nReport saved to: {output_file}")
        print(f"Total x87 calls: {report['x87_summary']['total_x87_calls']}")
        print(f"Hot functions: {len(report['hot_functions'])}")
        
        if report['hot_functions']:
            print("\nTop 5 hot functions:")
            for func in report['hot_functions'][:5]:
                print(f"  {func['rank']}. {func['estimated_name']} ({func['address_hex']})")
                print(f"     x87 calls: {func['x87_call_count']}, CPU samples: {func['sample_count']}")
                if 'suggested_strategy' in func:
                    print(f"     Strategy: {func['suggested_strategy']}")
        
        return 0
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())
