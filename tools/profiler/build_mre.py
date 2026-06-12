#!/usr/bin/env python3
"""
MRE Builder — converts a profiling report into a Rust benchmark crate.

Usage:
    python3 build_mre.py data/profiling/profiling-report.json --output rust_x87_mre/
"""
import argparse
import json
from pathlib import Path

OPCODES: dict[str, list[int]] = {
    "fld":        [0xD9, 0x00],
    "fstp":       [0xDD, 0xD8],
    "fadd":       [0xD8, 0xC1],
    "fsub":       [0xD8, 0xE1],
    "fmul":       [0xD8, 0xC9],
    "fdiv":       [0xD8, 0xF1],
    "fsqrt":      [0xD9, 0xFA],
    "fsin":       [0xD9, 0xFE],
    "fcos":       [0xD9, 0xFF],
    "fxch":       [0xD9, 0xC9],
    "fild":       [0xDB, 0x00],
    "fistp":      [0xDF, 0x10],
    "fld1":       [0xD9, 0xE8],
    "fldz":       [0xD9, 0xEE],
    "fchs":       [0xD9, 0xE0],
    "fabs":       [0xD9, 0xE1],
    "frndint":    [0xD9, 0xFC],
    "fldpi":      [0xD9, 0xEB],
    "fldln2":     [0xD9, 0xED],
    "fldlg2":     [0xD9, 0xEC],
    "fcomp":      [0xD8, 0xD9],
    "fucomp":     [0xDD, 0xE1],
    "ftst":       [0xD9, 0xE4],
    "fxam":       [0xD9, 0xE5],
    "fscale":     [0xD9, 0xFD],
}


def load_profile_report(path: Path) -> dict:
    with open(path) as f:
        return json.load(f)


def extract_hot_path(report: dict) -> list[dict]:
    x87_summary = report.get("x87_summary", {})
    by_function = x87_summary.get("by_function", {})
    total = sum(by_function.values())
    if total == 0:
        return []
    return sorted(
        [{"opcode": k, "count": v, "weight": v / total} for k, v in by_function.items()],
        key=lambda x: x["count"],
        reverse=True,
    )


def generate_cargo_toml(name: str = "rust_x87_mre") -> str:
    return f"""[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "rust_x87_mre"
path = "src/hot_path.rs"

[dependencies]
libc = "0.2"
which = "6"

[profile.release]
opt-level = 3
lto = true
"""


def generate_hot_path_rs(instructions: list[dict], total_calls: int) -> str:
    lines = [
        "//! Auto-generated hot-path benchmark.",
        "//! Generated from profiling data by tools/profiler/build_mre.py.",
        "",
        "use std::time::Instant;",
        "",
        "#[derive(Debug)]",
        "pub struct HotPathConfig {",
        "    pub instructions: Vec<InstructionWeight>,",
        "    pub total_calls: usize,",
        "}",
        "",
        "#[derive(Debug)]",
        "pub struct InstructionWeight {",
        "    pub opcode: &'static str,",
        "    pub bytes: &'static [u8],",
        "    pub weight: f64,",
        "    pub count: usize,",
        "}",
        "",
        "pub fn hot_path_config() -> HotPathConfig {",
        "    HotPathConfig {",
        "        instructions: vec![",
    ]
    for inst in instructions:
        opcode = inst["opcode"]
        bytes_repr = bytes(OPCODES.get(opcode, [0xD9, 0x00])).hex()
        bytes_literal = "[" + ", ".join(f"0x{b:02X}" for b in OPCODES.get(opcode, [0xD9, 0x00])) + "]"
        lines.append(
            f'            InstructionWeight {{ opcode: "{opcode}", bytes: &{bytes_literal}, '
            f'weight: {inst["weight"]:.4f}, count: {inst["count"]} }},'
        )
    lines.extend([
        "        ],",
        f"        total_calls: {total_calls},",
        "    }",
        "}",
        "",
        "pub fn run_benchmark(iterations: usize) -> f64 {",
        "    let config = hot_path_config();",
        "    let start = Instant::now();",
        "    for _ in 0..iterations {",
        "        std::hint::black_box(&config.instructions);",
        "    }",
        "    let elapsed = start.elapsed().as_secs_f64();",
        "    if elapsed > 0.0 { iterations as f64 / elapsed } else { 0.0 }",
        "}",
        "",
        "fn main() {",
        "    let config = hot_path_config();",
        '    println!("Hot-path MRE: {} unique opcodes", config.instructions.len());',
        '    println!("Total x87 calls in sample: {}", config.total_calls);',
        '    println!("Top 5 opcodes:");',
        "    for inst in config.instructions.iter().take(5) {",
        '        println!("  {}: {} calls ({:.1}%)",',
        '               inst.opcode, inst.count, inst.weight * 100.0);',
        "    }",
        '    println!("\\nBenchmarking...");',
        "    let ips = run_benchmark(1_000_000);",
        '    println!("Result: {:.0} iterations/sec", ips);',
        "}",
    ])
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description="Build MRE from profiling report")
    parser.add_argument("report", type=Path, help="Path to profiling-report.json")
    parser.add_argument("--output", type=Path, default=Path("rust_x87_mre"),
                        help="Output directory for MRE crate")
    args = parser.parse_args()

    report = load_profile_report(args.report)
    hot_path = extract_hot_path(report)
    total = sum(i["count"] for i in hot_path)

    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "Cargo.toml").write_text(generate_cargo_toml(args.output.name))
    (args.output / "src").mkdir(exist_ok=True)
    (args.output / "src" / "hot_path.rs").write_text(
        generate_hot_path_rs(hot_path, total)
    )
    print(f"Generated MRE at {args.output}/")


if __name__ == "__main__":
    main()
