#!/usr/bin/env python3
"""
CPU sampler using macOS `sample` command.

Wraps the built-in `sample` utility to capture CPU usage data
from the Wine process running WoW.exe.
"""

import argparse
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

import toml

from attach import find_wow_process, load_config


def run_sample(pid: int, duration: int) -> str:
    """
    Run `sample` command on the given PID for the specified duration.
    
    Returns:
        Raw sample output as a string
        
    Raises:
        RuntimeError: If sample command fails
    """
    try:
        result = subprocess.run(
            ["sample", str(pid), str(duration)],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"sample command failed: {e.stderr}")
    except FileNotFoundError:
        raise RuntimeError("sample command not found. This tool requires macOS.")


def parse_sample_output(output: str) -> dict:
    """
    Parse the output of the `sample` command.
    
    Extracts:
    - Call stacks with hex addresses
    - Sample counts per address
    - Module names where available
    
    Returns:
        Parsed sample data as a dictionary
    """
    samples = []
    
    # Look for hex addresses in call stacks
    # Typical format:    0x1234567    SomeModule`function_name + 123
    # Or:                0x1234567    ???
    hex_pattern = re.compile(r"^\s*(0x[0-9a-fA-F]+)\s+(.+)$")
    
    # Also look for aggregated output sections
    # Format: Call graph:
    #    1234 Thread_12345   DispatchQueue_1: com.apple.main-thread (serial)
    #      + 1234 start  (in libdyld.dylib) + 1  [0x123456789]
    
    lines = output.split("\n")
    in_call_graph = False
    
    for line in lines:
        line = line.strip()
        
        # Detect call graph section
        if "Call graph:" in line:
            in_call_graph = True
            continue
        
        if in_call_graph and not line:
            continue
        
        match = hex_pattern.match(line)
        if match:
            addr_hex = match.group(1).lower()
            symbol = match.group(2).strip()
            
            # Try to extract module name
            module = "unknown"
            if "`" in symbol:
                parts = symbol.split("`", 1)
                module = parts[0].strip()
            elif "(in" in symbol:
                module_match = re.search(r"\(in\s+([^)]+)\)", symbol)
                if module_match:
                    module = module_match.group(1).strip()
            
            samples.append({
                "address_hex": addr_hex,
                "symbol": symbol,
                "module": module,
                "sample_count": 1,  # Will aggregate later
            })
    
    # Aggregate samples by address
    aggregated = {}
    for sample in samples:
        addr = sample["address_hex"]
        if addr not in aggregated:
            aggregated[addr] = {
                "address_hex": addr,
                "symbol": sample["symbol"],
                "module": sample["module"],
                "sample_count": 0,
            }
        aggregated[addr]["sample_count"] += 1
    
    # Sort by sample count descending
    sorted_samples = sorted(
        aggregated.values(),
        key=lambda x: x["sample_count"],
        reverse=True,
    )
    
    return {
        "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "samples": sorted_samples,
        "total_samples": len(samples),
        "unique_addresses": len(aggregated),
    }


def save_samples(data: dict, output_dir: Path) -> Path:
    """Save parsed sample data to a JSON file."""
    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    output_file = output_dir / f"cpu_samples_{timestamp}.json"
    
    output_dir.mkdir(parents=True, exist_ok=True)
    
    with open(output_file, "w") as f:
        json.dump(data, f, indent=2)
    
    return output_file


def main():
    parser = argparse.ArgumentParser(
        description="Sample CPU usage from the WoW process"
    )
    parser.add_argument(
        "--duration", "-d",
        type=int,
        default=300,
        help="Sampling duration in seconds (default: 300)",
    )
    parser.add_argument(
        "--output-dir", "-o",
        type=Path,
        help="Output directory for sample data",
    )
    parser.add_argument(
        "--json", "-j",
        action="store_true",
        help="Output result as JSON",
    )
    parser.add_argument(
        "--raw",
        action="store_true",
        help="Also save raw sample output",
    )
    
    args = parser.parse_args()
    
    try:
        config = load_config()
        
        # Validate duration
        min_dur = config["profiling"].get("min_duration", 10)
        max_dur = config["profiling"].get("max_duration", 3600)
        if args.duration < min_dur:
            print(f"Duration too short (minimum {min_dur}s)", file=sys.stderr)
            return 1
        if args.duration > max_dur:
            print(f"Duration too long (maximum {max_dur}s)", file=sys.stderr)
            return 1
        
        # Find WoW process
        pid = find_wow_process(config)
        print(f"Found WoW process: PID {pid}")
        print(f"Sampling CPU for {args.duration} seconds...")
        
        # Run sample
        raw_output = run_sample(pid, args.duration)
        
        # Parse output
        parsed = parse_sample_output(raw_output)
        parsed["pid"] = pid
        parsed["duration_seconds"] = args.duration
        
        # Save raw output if requested
        if args.raw:
            output_dir = args.output_dir or Path(config["profiling"].get("output_dir", "data/profiling"))
            output_dir = output_dir.resolve()
            output_dir.mkdir(parents=True, exist_ok=True)
            
            timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
            raw_file = output_dir / f"cpu_samples_{timestamp}_raw.txt"
            with open(raw_file, "w") as f:
                f.write(raw_output)
            print(f"Raw output saved to: {raw_file}")
        
        # Save parsed output
        output_dir = args.output_dir or Path(config["profiling"].get("output_dir", "data/profiling"))
        output_dir = output_dir.resolve()
        output_file = save_samples(parsed, output_dir)
        
        print(f"Parsed samples saved to: {output_file}")
        print(f"Total samples: {parsed['total_samples']}")
        print(f"Unique addresses: {parsed['unique_addresses']}")
        
        if parsed['samples']:
            top = parsed['samples'][0]
            print(f"Top address: {top['address_hex']} ({top['module']}) - {top['sample_count']} samples")
        
        if args.json:
            print(json.dumps(parsed, indent=2))
        
        return 0
        
    except RuntimeError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    except KeyboardInterrupt:
        print("\nSampling interrupted", file=sys.stderr)
        return 130
    except Exception as e:
        print(f"Unexpected error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
