#!/usr/bin/env python3
"""
Process discovery and attachment for WoW running under CrossOver/Wine.

Finds the Wine process running WoW.exe and returns its PID.
Handles multiple-match errors and provides clear error messages.
"""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Optional

import toml


def load_config() -> dict:
    """Load profiling configuration from config.toml and optional config.local.toml."""
    config_path = Path(__file__).parent / "config.toml"
    config = toml.load(config_path)
    
    local_config_path = Path(__file__).parent / "config.local.toml"
    if local_config_path.exists():
        local_config = toml.load(local_config_path)
        # Deep merge local config into base config
        _deep_merge(config, local_config)
    
    return config


def _deep_merge(base: dict, override: dict) -> None:
    """Recursively merge override dict into base dict."""
    for key, value in override.items():
        if key in base and isinstance(base[key], dict) and isinstance(value, dict):
            _deep_merge(base[key], value)
        else:
            base[key] = value


def find_wine_processes() -> list[dict]:
    """Find all Wine-related processes running on the system."""
    try:
        result = subprocess.run(
            ["ps", "-eo", "pid,ppid,comm,args"],
            capture_output=True,
            text=True,
            check=True,
        )
    except subprocess.CalledProcessError as e:
        print(f"Error running ps: {e}", file=sys.stderr)
        return []

    processes = []
    for line in result.stdout.strip().split("\n")[1:]:  # Skip header
        parts = line.split(None, 3)
        if len(parts) < 4:
            continue

        pid, ppid, comm, args = parts
        # Wine processes are typically wine64-preloader, wine, or wine64
        if comm in ("wine64-preloader", "wine", "wine64", " Wineserver", "runtime_loader") or "wine" in args.lower():
            processes.append({
                "pid": int(pid),
                "ppid": int(ppid),
                "comm": comm,
                "args": args,
            })

    return processes


def find_wow_process(config: dict) -> int:
    """
    Find the specific Wine process running WoW.exe.
    
    Returns:
        PID of the Wine process running WoW.exe
        
    Raises:
        RuntimeError: If no WoW process found or multiple matches
    """
    wow_exe_name = config["paths"].get("wow_exe", "WoW.exe")
    
    wine_processes = find_wine_processes()
    if not wine_processes:
        raise RuntimeError(
            "No Wine processes found. "
            "Make sure WoW is running via CrossOver."
        )
    
    # Filter for processes with WoW.exe in their args
    wow_processes = [
        p for p in wine_processes
        if wow_exe_name.lower() in p["args"].lower()
    ]
    
    if not wow_processes:
        # Fallback: if no exact match, check if any wine process has the wow_dir
        wow_dir = config["paths"].get("wow_dir", "")
        wow_processes = [
            p for p in wine_processes
            if wow_dir and wow_dir.lower() in p["args"].lower()
        ]
    
    if not wow_processes:
        raise RuntimeError(
            f"No Wine process found running {wow_exe_name}. "
            f"Found {len(wine_processes)} Wine process(es) but none match. "
            "Make sure WoW is running via CrossOver."
        )
    
    if len(wow_processes) > 1:
        # wine64-preloader is the actual Wine host (has Windows DLLs); runtime_loader is
        # the x87 JIT shim — prefer by basename so full-path comms (Whisky) still match.
        _COMM_PRIORITY = {"wine64-preloader": 0, "wine64": 1, "wine": 2}
        wow_processes.sort(
            key=lambda p: (_COMM_PRIORITY.get(os.path.basename(p["comm"]), 99), p["pid"])
        )
        chosen = wow_processes[0]
        print(
            f"Multiple WoW processes found ({[p['pid'] for p in wow_processes]}), "
            f"using PID {chosen['pid']} ({os.path.basename(chosen['comm'])})",
            file=sys.stderr,
        )
        return chosen["pid"]
    
    return wow_processes[0]["pid"]


def get_process_info(pid: int) -> dict:
    """Get detailed information about a process."""
    try:
        result = subprocess.run(
            ["ps", "-p", str(pid), "-o", "pid,ppid,comm,args"],
            capture_output=True,
            text=True,
            check=True,
        )
        lines = result.stdout.strip().split("\n")
        if len(lines) < 2:
            return {}
        
        parts = lines[1].split(None, 3)
        if len(parts) < 4:
            return {}
        
        return {
            "pid": int(parts[0]),
            "ppid": int(parts[1]),
            "comm": parts[2],
            "args": parts[3],
        }
    except (subprocess.CalledProcessError, ValueError):
        return {}


def main():
    parser = argparse.ArgumentParser(
        description="Find and attach to the Wine process running WoW.exe"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output result as JSON",
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Show detailed process information",
    )
    
    args = parser.parse_args()
    
    try:
        config = load_config()
        pid = find_wow_process(config)
        
        if args.verbose:
            info = get_process_info(pid)
            output = {
                "pid": pid,
                "process": info,
                "config_paths": config.get("paths", {}),
            }
        else:
            output = {"pid": pid}
        
        if args.json:
            print(json.dumps(output, indent=2))
        else:
            print(pid)
        
        return 0
        
    except RuntimeError as e:
        error_output = {"error": str(e)}
        if args.json:
            print(json.dumps(error_output, indent=2), file=sys.stderr)
        else:
            print(f"Error: {e}", file=sys.stderr)
        return 1
    except Exception as e:
        error_output = {"error": f"Unexpected error: {e}"}
        if args.json:
            print(json.dumps(error_output, indent=2), file=sys.stderr)
        else:
            print(f"Unexpected error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
