#!/usr/bin/env python3
"""
Ingest JIT opcode counter dump produced by the rosettax87_jit profiling extension.

Reads the JSON written to ROSETTA_X87_PROFILE_OUT (default /tmp/rosettax87_profile.json)
and prints a summary.  Can also be called as a library by analyze.py.

Usage:
  python3 ingest_jit_counts.py [path]          # print summary
  python3 ingest_jit_counts.py --auto          # read ROSETTA_X87_PROFILE_OUT env or default
  python3 ingest_jit_counts.py --json [path]   # emit JSON for piping into analyze.py
"""

import argparse
import json
import os
import sys
from pathlib import Path

DEFAULT_JIT_OUT = "/tmp/rosettax87_profile.json"


def load_jit_counts(path: Path) -> dict:
    """
    Load and validate the JIT opcode counter JSON file.

    Returns a dict with:
        x87_opcode_counts: {opcode_name: count, ...}
        translated_total:  int
    """
    with open(path) as f:
        data = json.load(f)
    if "x87_opcode_counts" not in data:
        raise ValueError(f"Missing 'x87_opcode_counts' in {path}")
    if "translated_total" not in data:
        raise ValueError(f"Missing 'translated_total' in {path}")
    if not isinstance(data["x87_opcode_counts"], dict):
        raise ValueError("'x87_opcode_counts' must be an object")
    if not isinstance(data["translated_total"], int):
        raise ValueError("'translated_total' must be an integer")
    return data


def jit_counts_to_x87_summary(data: dict) -> dict:
    """
    Convert the raw JIT dump to the x87_summary schema used by merge_reports().

    The 'by_function' field is repurposed to hold per-opcode counts — this is
    what extract_hot_path() / build_mre.py consume for instruction weighting.
    """
    counts = data["x87_opcode_counts"]
    total = sum(counts.values())
    return {
        "total_x87_calls": total,
        "by_function": dict(counts),
        "by_module": {},
        "source": "jit",
    }


def resolve_path(args_path: Path, auto: bool) -> Path:
    if auto:
        env_path = os.environ.get("ROSETTA_X87_PROFILE_OUT", DEFAULT_JIT_OUT)
        return Path(env_path)
    return args_path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Ingest JIT x87 opcode counts from rosettax87_jit ROSETTA_X87_PROFILE"
    )
    parser.add_argument(
        "path",
        nargs="?",
        default=DEFAULT_JIT_OUT,
        type=Path,
        help=f"JIT profile JSON file (default: {DEFAULT_JIT_OUT})",
    )
    parser.add_argument(
        "--auto",
        action="store_true",
        help="Read path from ROSETTA_X87_PROFILE_OUT env var (or default)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit the raw JSON to stdout (for piping)",
    )
    args = parser.parse_args()

    path = resolve_path(args.path, args.auto)

    try:
        data = load_jit_counts(path)
    except FileNotFoundError:
        print(f"Error: JIT profile not found: {path}", file=sys.stderr)
        print(
            "  Start WoW with: ROSETTA_X87_PROFILE=1 ROSETTA_X87_PROFILE_OUT=/tmp/rosettax87_profile.json",
            file=sys.stderr,
        )
        print(
            "  Then trigger a dump: kill -USR2 $(python3 attach.py)",
            file=sys.stderr,
        )
        return 1
    except (json.JSONDecodeError, ValueError) as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps(data, indent=2))
        return 0

    counts = data["x87_opcode_counts"]
    total = data["translated_total"]
    opcode_total = sum(counts.values())

    print(f"JIT translated total (ops our JIT handled): {total:,}")
    print(f"Tracked x87 opcodes: {len(counts)} unique, {opcode_total:,} total")

    if counts:
        print("\nTop x87 opcodes by translation count:")
        ranked = sorted(counts.items(), key=lambda kv: kv[1], reverse=True)
        for op, n in ranked[:20]:
            pct = 100 * n / opcode_total if opcode_total > 0 else 0.0
            print(f"  {op:<16}  {n:>12,}  ({pct:5.1f}%)")
        if len(ranked) > 20:
            print(f"  ... and {len(ranked) - 20} more opcodes")

    return 0


if __name__ == "__main__":
    sys.exit(main())
