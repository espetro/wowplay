//! SiliconPatchAdapter port — interface between profiling output and patch application.
//!
//! Defines the contract for analyzing profiling reports to identify
//! hot x87 functions and applying/removing runtime hooks.

use crate::adapters::errors::AdapterError;

/// Status of an installed hook
#[derive(Debug, Clone, PartialEq)]
pub enum HookStatus {
    /// Hook is active and functioning
    Active,
    /// Hook is installed but not yet verified
    Pending,
    /// Hook failed verification
    Failed(String),
    /// Hook was removed
    Removed,
}

/// Target for hook installation
#[derive(Debug, Clone)]
pub struct HookTarget {
    /// Memory address of the function to hook
    pub address: u64,
    /// Estimated function name (from address map or symbols)
    pub name: String,
    /// Suggested replacement strategy
    pub strategy: String,
    /// x87 call count from profiling
    pub x87_call_count: u64,
    /// CPU sample weight
    pub cpu_sample_count: u64,
}

/// Profiling report data structure
#[derive(Debug, Clone)]
pub struct ProfilingReport {
    /// WoW version string
    pub wow_version: String,
    /// Total profiling duration
    pub duration_seconds: u64,
    /// Ranked list of hot functions
    pub hot_functions: Vec<HotFunction>,
    /// Total x87 calls observed
    pub total_x87_calls: u64,
}

/// A hot function identified by profiling
#[derive(Debug, Clone)]
pub struct HotFunction {
    /// Rank by combined metric
    pub rank: usize,
    /// Memory address
    pub address: u64,
    /// Estimated name
    pub name: String,
    /// Source of data (frida, instruments, combined)
    pub source: String,
    /// CPU sample count
    pub sample_count: u64,
    /// x87 call count
    pub x87_call_count: u64,
    /// Top x87 operations used
    pub top_x87_ops: Vec<String>,
    /// Suggested replacement strategy
    pub suggested_strategy: String,
}

/// Port for applying SiliconPatch optimizations based on profiling data.
///
/// This is the interface between the profiling harness (which identifies
/// hot x87 functions) and the patch implementation (which replaces them
/// with SSE/NEON equivalents).
pub trait SiliconPatchAdapter: Send + Sync {
    /// Identify hot functions from a profiling report.
    ///
    /// Analyzes the report and returns a prioritized list of hook targets.
    /// The list is sorted by combined importance (x87_call_count + sample_count).
    ///
    /// # Arguments
    /// * `report` — Profiling report from analyze.py
    ///
    /// # Returns
    /// Vector of hook targets, sorted by priority
    fn identify_hot_functions(&self, report: &ProfilingReport) -> Vec<HookTarget>;

    /// Apply hooks to the specified targets.
    ///
    /// Installs runtime hooks at the given addresses. The hooks redirect
    /// x87 instructions to SSE/NEON equivalents.
    ///
    /// # Arguments
    /// * `targets` — List of hook targets to apply
    ///
    /// # Returns
    /// Ok(()) on success, or an error if any hook fails
    fn apply_hooks(&self, targets: &[HookTarget]) -> Result<(), AdapterError>;

    /// Verify that all installed hooks are functioning.
    ///
    /// Checks each hook by:
    /// 1. Verifying the hook bytes are present at the target address
    /// 2. Optionally calling the function to ensure it still works
    /// 3. Checking for crashes or unexpected behavior
    ///
    /// # Returns
    /// Vector of hook statuses, one per installed hook
    fn verify_hooks(&self) -> Result<Vec<HookStatus>, AdapterError>;
}
