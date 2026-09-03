//! Imperative adapters for the Rust package-planning hexagon.
//!
//! The root shell translates path, process, Cargo, rustc, and cache details to
//! Mantle-owned application observations before deterministic policy runs.

// r[impl rust_package_planning.application_owned_ports]
// r[impl rust_package_planning.hexagonal_compatibility]
pub mod cache_adapter;
pub mod cargo_adapter;
pub mod compiler_adapter;
pub mod execution_adapter;
pub mod workspace_adapter;

const PROCESS_STATUS_CODE_UNAVAILABLE: i32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessObservation {
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) status_code: i32,
}

impl ProcessObservation {
    pub(crate) fn from_output(output: std::process::Output) -> Self {
        let status_code = output.status.code().unwrap_or(PROCESS_STATUS_CODE_UNAVAILABLE);
        debug_assert!(output.status.success() || status_code != 0);
        debug_assert!(!output.status.success() || status_code == 0);
        Self {
            stdout: output.stdout,
            stderr: output.stderr,
            status_code,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustPlanAdapterError {
    pub(crate) capability: mantle_rust_plan::RustPlanCapability,
    pub(crate) executable: String,
    pub(crate) detail: String,
}

impl RustPlanAdapterError {
    pub(crate) fn launch(
        capability: mantle_rust_plan::RustPlanCapability,
        executable: &std::path::Path,
        error: &std::io::Error,
    ) -> Self {
        let executable = executable.display().to_string();
        let detail = error.to_string();
        debug_assert!(!executable.is_empty());
        debug_assert!(!detail.is_empty());
        Self {
            capability,
            executable,
            detail,
        }
    }
}

impl std::fmt::Display for RustPlanAdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Rust-plan {:?} adapter failed for {}: {}", self.capability, self.executable, self.detail)
    }
}

#[cfg(test)]
mod tests;
