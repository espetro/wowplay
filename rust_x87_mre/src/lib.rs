//! rust_x87_mre — Minimal Reproducible Example for x87 hot-path benchmarking.
//!
//! This crate reproduces the x87 instruction mix from a real WoW profiling session.
//! Run with: `cargo run --release`
//! Benchmark with: `cargo bench`

pub mod hot_path;
pub mod subprocess_bench;

pub use hot_path::{hot_path_config, run_benchmark, HotPathConfig, InstructionWeight};
