#!/usr/bin/env python3
"""
JSON report schema for profiling output.

Defines the structure of profiling reports and provides validation
and creation utilities.
"""

import json
from datetime import datetime, timezone
from typing import Any, Optional


# Report schema version
SCHEMA_VERSION = "1.0"

# Required top-level fields
REQUIRED_FIELDS = [
    "version",
    "timestamp",
    "wow_version",
    "profile_duration_seconds",
    "hot_functions",
    "x87_summary",
    "environment",
]

# Required hot_function fields
REQUIRED_HOT_FUNCTION_FIELDS = [
    "rank",
    "address_hex",
    "source",
    "sample_count",
]

# Required x87_summary fields
REQUIRED_X87_SUMMARY_FIELDS = [
    "total_x87_calls",
]

# Required environment fields
REQUIRED_ENVIRONMENT_FIELDS = [
    "macos_version",
    "arch",
]


def validate_report(report: dict) -> tuple[bool, list[str]]:
    """
    Validate that a report conforms to the schema.
    
    Returns:
        (is_valid, list_of_errors)
    """
    errors = []
    
    # Check required top-level fields
    for field in REQUIRED_FIELDS:
        if field not in report:
            errors.append(f"Missing required field: {field}")
    
    # Check version
    if "version" in report and report["version"] != SCHEMA_VERSION:
        errors.append(f"Unsupported schema version: {report['version']} (expected {SCHEMA_VERSION})")
    
    # Check timestamp format
    if "timestamp" in report:
        try:
            datetime.fromisoformat(report["timestamp"].replace("Z", "+00:00"))
        except ValueError:
            errors.append("Invalid timestamp format (expected ISO 8601)")
    
    # Check hot_functions
    if "hot_functions" in report:
        if not isinstance(report["hot_functions"], list):
            errors.append("hot_functions must be a list")
        else:
            for i, func in enumerate(report["hot_functions"]):
                for field in REQUIRED_HOT_FUNCTION_FIELDS:
                    if field not in func:
                        errors.append(f"hot_functions[{i}]: missing {field}")
                
                if "rank" in func and not isinstance(func["rank"], int):
                    errors.append(f"hot_functions[{i}]: rank must be an integer")
                
                if "sample_count" in func and not isinstance(func["sample_count"], int):
                    errors.append(f"hot_functions[{i}]: sample_count must be an integer")
                
                if "x87_call_count" in func and not isinstance(func["x87_call_count"], int):
                    errors.append(f"hot_functions[{i}]: x87_call_count must be an integer")
                
                if "address_hex" in func:
                    addr = func["address_hex"]
                    if not isinstance(addr, str) or not addr.startswith("0x"):
                        errors.append(f"hot_functions[{i}]: address_hex must be a hex string (0x...)")
    
    # Check x87_summary
    if "x87_summary" in report:
        summary = report["x87_summary"]
        for field in REQUIRED_X87_SUMMARY_FIELDS:
            if field not in summary:
                errors.append(f"x87_summary: missing {field}")
        
        if "total_x87_calls" in summary and not isinstance(summary["total_x87_calls"], int):
            errors.append("x87_summary.total_x87_calls must be an integer")
        
        if "by_function" in summary and not isinstance(summary["by_function"], dict):
            errors.append("x87_summary.by_function must be an object")
        
        if "by_module" in summary and not isinstance(summary["by_module"], dict):
            errors.append("x87_summary.by_module must be an object")
    
    # Check environment
    if "environment" in report:
        env = report["environment"]
        for field in REQUIRED_ENVIRONMENT_FIELDS:
            if field not in env:
                errors.append(f"environment: missing {field}")
    
    return len(errors) == 0, errors


def create_report(
    wow_version: str = "3.3.5a",
    profile_duration_seconds: int = 300,
    hot_functions: Optional[list[dict]] = None,
    x87_summary: Optional[dict] = None,
    environment: Optional[dict] = None,
) -> dict:
    """
    Create a new profiling report with the given data.
    
    Args:
        wow_version: WoW client version
        profile_duration_seconds: Duration of profiling session
        hot_functions: List of hot function entries
        x87_summary: x87 call summary statistics
        environment: Environment information
        
    Returns:
        A complete profiling report dictionary
    """
    if hot_functions is None:
        hot_functions = []
    
    if x87_summary is None:
        x87_summary = {
            "total_x87_calls": 0,
            "by_function": {},
            "by_module": {},
        }
    
    if environment is None:
        environment = {
            "macos_version": "unknown",
            "crossover_version": "unknown",
            "arch": "arm64",
        }
    
    # Ensure required summary fields
    if "total_x87_calls" not in x87_summary:
        x87_summary["total_x87_calls"] = 0
    if "by_function" not in x87_summary:
        x87_summary["by_function"] = {}
    if "by_module" not in x87_summary:
        x87_summary["by_module"] = {}
    
    # Ensure required environment fields
    if "macos_version" not in environment:
        environment["macos_version"] = "unknown"
    if "arch" not in environment:
        environment["arch"] = "arm64"
    
    report = {
        "version": SCHEMA_VERSION,
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "wow_version": wow_version,
        "profile_duration_seconds": profile_duration_seconds,
        "hot_functions": hot_functions,
        "x87_summary": x87_summary,
        "environment": environment,
    }
    
    # Validate before returning
    is_valid, errors = validate_report(report)
    if not is_valid:
        raise ValueError(f"Created invalid report: {errors}")
    
    return report


def merge_reports(
    x87_trace_data: dict,
    cpu_sample_data: dict,
    address_map: Optional[dict] = None,
    wow_version: str = "3.3.5a",
    profile_duration_seconds: int = 300,
) -> dict:
    """
    Merge x87 trace data and CPU sample data into a ranked report.
    
    Args:
        x87_trace_data: Raw x87 trace output from trace_x87.py
        cpu_sample_data: Parsed CPU sample data from sample_cpu.py
        address_map: Optional mapping of addresses to function names
        wow_version: WoW client version
        profile_duration_seconds: Duration of profiling session
        
    Returns:
        A complete profiling report
    """
    address_map = address_map or {}
    
    # Aggregate x87 calls by return address
    x87_by_address = {}
    x87_by_function = {}
    x87_by_module = {}
    
    for entry in x87_trace_data.get("calls", []):
        ret_addr = entry.get("ret_addr", "0x0")
        func = entry.get("func", "unknown")
        module = entry.get("module", "unknown")
        
        x87_by_address[ret_addr] = x87_by_address.get(ret_addr, 0) + 1
        x87_by_function[func] = x87_by_function.get(func, 0) + 1
        x87_by_module[module] = x87_by_module.get(module, 0) + 1
    
    # Aggregate CPU samples by address
    cpu_by_address = {}
    for entry in cpu_sample_data.get("samples", []):
        addr = entry.get("address_hex", "0x0")
        count = entry.get("sample_count", 0)
        cpu_by_address[addr] = cpu_by_address.get(addr, 0) + count
    
    # Combine and rank
    all_addresses = set(x87_by_address.keys()) | set(cpu_by_address.keys())
    
    hot_functions = []
    for rank, addr in enumerate(sorted(all_addresses, key=lambda a: (
        x87_by_address.get(a, 0) * (1 + cpu_by_address.get(a, 0))
    ), reverse=True), start=1):
        x87_count = x87_by_address.get(addr, 0)
        cpu_count = cpu_by_address.get(addr, 0)
        
        # Look up function name from address map
        estimated_name = address_map.get(addr, f"sub_{addr}")
        
        # Determine primary source
        if cpu_count > 0 and x87_count > 0:
            source = "combined"
        elif cpu_count > 0:
            source = "instruments"
        else:
            source = "frida"
        
        # Get top x87 ops for this address
        top_ops = []
        for entry in x87_trace_data.get("calls", []):
            if entry.get("ret_addr") == addr:
                op = entry.get("func", "")
                if op and op not in top_ops:
                    top_ops.append(op)
        
        func_entry = {
            "rank": rank,
            "address_hex": addr,
            "estimated_name": estimated_name,
            "source": source,
            "sample_count": cpu_count,
            "x87_call_count": x87_count,
        }
        
        if top_ops:
            func_entry["top_x87_ops"] = top_ops[:5]  # Top 5
        
        # Add strategy suggestion based on ops
        if any(op in top_ops for op in ["sin", "cos", "tan"]):
            func_entry["suggested_strategy"] = "Replace fsin/fcos with SSE lookup table"
        elif "sqrt" in top_ops:
            func_entry["suggested_strategy"] = "Replace fsqrt with NEON vsqrt"
        elif any(op in top_ops for op in ["pow", "exp", "log"]):
            func_entry["suggested_strategy"] = "Replace fyl2x/f2xm1 with NEON exp/log"
        
        hot_functions.append(func_entry)
    
    # Build x87 summary
    total_x87 = sum(x87_by_function.values())
    x87_summary = {
        "total_x87_calls": total_x87,
        "by_function": x87_by_function,
        "by_module": x87_by_module,
    }
    
    # Build environment info
    import platform
    environment = {
        "macos_version": platform.mac_ver()[0] or "unknown",
        "crossover_version": cpu_sample_data.get("crossover_version", "unknown"),
        "arch": platform.machine(),
    }
    
    return create_report(
        wow_version=wow_version,
        profile_duration_seconds=profile_duration_seconds,
        hot_functions=hot_functions,
        x87_summary=x87_summary,
        environment=environment,
    )


def main():
    """CLI for schema validation."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Validate profiling report JSON")
    parser.add_argument("report_file", help="Path to JSON report file")
    args = parser.parse_args()
    
    with open(args.report_file) as f:
        report = json.load(f)
    
    is_valid, errors = validate_report(report)
    
    if is_valid:
        print("✅ Report is valid")
        print(f"   Version: {report['version']}")
        print(f"   Timestamp: {report['timestamp']}")
        print(f"   Hot functions: {len(report['hot_functions'])}")
        print(f"   Total x87 calls: {report['x87_summary']['total_x87_calls']}")
        return 0
    else:
        print("❌ Report is invalid:")
        for error in errors:
            print(f"   - {error}")
        return 1


if __name__ == "__main__":
    import sys
    sys.exit(main())
