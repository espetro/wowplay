//! Placeholder — replaced by build_mre.py during MRE generation.
//!
//! This file exists so `cargo check` passes before the MRE is generated.
//! Run `python3 tools/profiler/build_mre.py <report.json> --output rust_x87_mre/`
//! to generate the real implementation.

#![allow(dead_code)]

use std::time::Instant;

#[derive(Debug)]
pub struct HotPathConfig {
    pub instructions: Vec<InstructionWeight>,
    pub total_calls: usize,
}

#[derive(Debug)]
pub struct InstructionWeight {
    pub opcode: &'static str,
    pub bytes: &'static [u8],
    pub weight: f64,
    pub count: usize,
}

pub fn hot_path_config() -> HotPathConfig {
    HotPathConfig {
        instructions: vec![],
        total_calls: 0,
    }
}

pub fn run_benchmark(iterations: usize) -> f64 {
    let _config = hot_path_config();
    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(1_usize);
    }
    let elapsed = start.elapsed().as_secs_f64();
    if elapsed > 0.0 { iterations as f64 / elapsed } else { 0.0 }
}

fn main() {
    let config = hot_path_config();
    println!("Hot-path MRE: {} unique opcodes (placeholder)", config.instructions.len());
    println!("Total x87 calls in sample: {}", config.total_calls);
    println!("Run build_mre.py to generate from profiling data.");
    let ips = run_benchmark(1_000_000);
    println!("Result: {:.0} iterations/sec (baseline)", ips);
}
