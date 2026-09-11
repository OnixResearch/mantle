//! Application-owned ports with Mantle-owned request, result, and error types.
//!
//! Every port reports capability failures as typed `AdapterError` values; no
//! CLI, process, Cargo, path, or store type appears in a signature.

use alloc::string::String;
use alloc::vec::Vec;

use mantle_rust_plan_core::BuildProfile;
use mantle_rust_plan_core::PackageFacts;
use mantle_rust_plan_core::UnitEffect;

/// Typed adapter failure classes shared by every port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterError {
    /// Stable capability class, for example `workspace-facts-unavailable`.
    pub code: String,
    /// Bounded human detail; never a raw tool transcript.
    pub detail: String,
}

impl AdapterError {
    /// Construct one typed capability failure.
    pub fn new(code: &str, detail: &str) -> Self {
        debug_assert!(!code.is_empty() && !detail.is_empty());
        Self {
            code: String::from(code),
            detail: String::from(detail),
        }
    }
}

/// Request for workspace facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFactsRequest {
    /// Root package names requested by the operator.
    pub roots: Vec<String>,
    pub profile: BuildProfile,
}

/// Admitted workspace facts returned by a facts port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFactsView {
    pub packages: Vec<PackageFacts>,
    /// Requested features per package, already normalized by the adapter.
    pub feature_requests: Vec<mantle_rust_plan_core::PackageFeatureRequest>,
}

/// Port: load bounded workspace facts.
pub trait WorkspaceFactsSource {
    fn load_workspace_facts(&mut self, request: &WorkspaceFactsRequest) -> Result<WorkspaceFactsView, AdapterError>;
}

/// Request for Cargo oracle material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleCaptureRequest {
    pub roots: Vec<String>,
    pub profile: BuildProfile,
}

/// Cargo oracle facts supplied as external evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleFacts {
    /// Canonical oracle identity supplied by the adapter.
    pub oracle_identity: String,
    /// Package keys the oracle selected, in canonical order.
    pub selected_packages: Vec<String>,
}

/// Port: capture Cargo oracle material for parity evidence.
pub trait CargoOracleCapture {
    fn capture_oracle(&mut self, request: &OracleCaptureRequest) -> Result<OracleFacts, AdapterError>;
}

/// Request for compiler inspection facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInspectionRequest {
    pub profile: BuildProfile,
}

/// Compiler facts observed by an adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerFacts {
    /// Canonical compiler identity supplied by the adapter.
    pub compiler_identity: String,
    /// Target triple the compiler emits for.
    pub target_triple: String,
}

/// Port: inspect the selected compiler.
pub trait CompilerInspection {
    fn inspect_compiler(&mut self, request: &CompilerInspectionRequest) -> Result<CompilerFacts, AdapterError>;
}

/// Request for one unit's cache state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheLookupRequest {
    pub unit_id: mantle_rust_plan_core::UnitId,
    pub unit_blake3: mantle_rust_plan_core::Blake3Digest,
}

/// Cache lookup result for one unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheLookup {
    /// The unit's outputs are already present under the requested identity.
    Cached {
        /// Canonical output identity supplied by the adapter.
        output_identity: String,
    },
    /// The unit must be executed.
    NotCached,
}

/// Port: read the Rust build cache.
pub trait RustCacheAccess {
    fn lookup_unit(&mut self, request: &CacheLookupRequest) -> Result<CacheLookup, AdapterError>;
}

/// Port: execute one planned unit effect.
pub trait UnitExecutor {
    fn execute_unit(&mut self, effect: &UnitEffect) -> Result<mantle_rust_plan_core::UnitObservation, AdapterError>;
}
