#!/usr/bin/env python3
"""
Trace aggregation and ranking analyzer.

Reads JIT opcode counts (ROSETTA_X87_PROFILE=1) and/or CPU sample data,
cross-references with an address map, and produces a ranked JSON report.

Legacy Frida trace files (raw_trace_*.json) are accepted but no longer required.
"""

import argparse
import glob
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

from ingest_jit_counts import load_jit_counts, DEFAULT_JIT_OUT
from schema import merge_reports, validate_report


def extract_hot_path(report: dict) -> dict:
    """
    Extract hot-path instruction frequency data from a profiling report.
    
    Produces a JSON structure suitable for the MRE builder (build_mre.py).
    """
    x87_summary = report.get("x87_summary", {})
    by_function = x87_summary.get("by_function", {})
    total = sum(by_function.values())
    
    if total == 0:
        hot_path_instructions = []
    else:
        hot_path_instructions = sorted(
            [
                {"opcode": k, "count": v, "weight": round(v / total, 6)}
                for k, v in by_function.items()
            ],
            key=lambda x: x["count"],
            reverse=True,
        )
    
    top_functions = []
    for func in report.get("hot_functions", [])[:10]:
        top_functions.append({
            "address": func.get("address_hex", ""),
            "name": func.get("estimated_name", ""),
            "x87_calls": func.get("x87_call_count", 0),
        })
    
    return {
        "hot_path_instructions": hot_path_instructions,
        "total_calls": total,
        "top_functions": top_functions,
    }


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
    sample_files: list[Path],
    address_map: dict,
    trace_files: Optional[list[Path]] = None,
    jit_counts_file: Optional[Path] = None,
    wow_version: str = "3.3.5a",
    profile_duration_seconds: int = 300,
) -> dict:
    """
    Aggregate profiling data into a ranked report.

    x87 data comes from jit_counts_file (preferred) or trace_files (legacy Frida).
    sample_files (CPU hotspots from macOS `sample`) is always required.
    """
    merged_samples: dict = {"samples": []}
    for sample_file in sample_files:
        data = load_json_file(sample_file)
        merged_samples["samples"].extend(data.get("samples", []))

    jit_opcode_counts: Optional[dict] = None
    if jit_counts_file is not None:
        raw = load_jit_counts(jit_counts_file)
        jit_opcode_counts = raw["x87_opcode_counts"]

    merged_trace: Optional[dict] = None
    if trace_files:
        merged_trace = {"calls": []}
        for trace_file in trace_files:
            data = load_json_file(trace_file)
            merged_trace["calls"].extend(data.get("calls", []))

    return merge_reports(
        cpu_sample_data=merged_samples,
        x87_trace_data=merged_trace,
        jit_opcode_counts=jit_opcode_counts,
        address_map=address_map,
        wow_version=wow_version,
        profile_duration_seconds=profile_duration_seconds,
    )


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
        description="Aggregate profiling data into a ranked report"
    )
    parser.add_argument(
        "--jit-counts", "-j",
        type=Path,
        metavar="PATH",
        help="JIT opcode counts JSON from ROSETTA_X87_PROFILE_OUT (preferred x87 source)",
    )
    parser.add_argument(
        "--trace", "-t",
        nargs="+",
        type=Path,
        help="Legacy Frida raw_trace JSON file(s) (optional; ignored when --jit-counts is set)",
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
        help=(
            "Auto-discover files: JIT counts from ROSETTA_X87_PROFILE_OUT (or default path), "
            "CPU samples from data/profiling/cpu_samples_*.json"
        ),
    )
    parser.add_argument(
        "--output-format",
        choices=["default", "hot-path-json"],
        default="default",
        help="Output format: 'default' (full report) or 'hot-path-json' (MRE builder input)",
    )

    args = parser.parse_args()

    try:
        jit_counts_file: Optional[Path] = args.jit_counts
        trace_files: list[Path] = []
        sample_files: list[Path] = []

        if args.auto:
            sample_pattern = "data/profiling/cpu_samples_*.json"
            sample_files = [Path(f) for f in glob.glob(sample_pattern)]

            if not sample_files:
                print(f"No CPU sample files found matching {sample_pattern}", file=sys.stderr)
                return 1

            # Prefer JIT counts over legacy Frida traces.
            if jit_counts_file is None:
                env_path = os.environ.get("ROSETTA_X87_PROFILE_OUT", DEFAULT_JIT_OUT)
                candidate = Path(env_path)
                if candidate.exists():
                    jit_counts_file = candidate

            if jit_counts_file is not None:
                print(f"JIT counts: {jit_counts_file}")
            else:
                trace_pattern = "data/profiling/raw_trace_*.json"
                trace_files = [Path(f) for f in glob.glob(trace_pattern)]
                if trace_files:
                    print(f"Found {len(trace_files)} legacy trace file(s)")
                else:
                    print(
                        "No JIT counts file found and no trace files; producing CPU-only report.",
                        file=sys.stderr,
                    )

            print(f"Found {len(sample_files)} CPU sample file(s)")
        else:
            sample_files = args.samples or []
            trace_files = args.trace or []

            if not sample_files:
                print("No sample files specified. Use --samples or --auto", file=sys.stderr)
                return 1

            if jit_counts_file is None and not trace_files:
                print(
                    "No x87 data specified. Use --jit-counts, --trace, or --auto",
                    file=sys.stderr,
                )
                print("Continuing with CPU samples only.", file=sys.stderr)

        # Load address map
        address_map = load_address_map(args.address_map)
        print(f"Loaded {len(address_map)} address mappings")

        # Analyze
        print("Analyzing...")
        report = analyze(
            sample_files=sample_files,
            address_map=address_map,
            trace_files=trace_files or None,
            jit_counts_file=jit_counts_file,
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
        
        # Hot-path JSON output mode
        if args.output_format == "hot-path-json":
            hot_path = extract_hot_path(report)
            print(json.dumps(hot_path, indent=2))
            return 0
        
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
