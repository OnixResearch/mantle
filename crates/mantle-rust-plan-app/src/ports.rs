//! Application-owned ports with Mantle-owned request, result, and error types.
//!
//! Every port reports capability failures as typed `AdapterError` values; no
//! CLI, process, Cargo, path, or store type appears in a signature.

use alloc::string::String;
use alloc::vec::Vec;

use mantle_rust_plan_core::ExistingUnitEffect;
use mantle_rust_plan_core::UnitObservation;

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

/// Selected workspace and profile inputs for receipt capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFactsRequest {
    /// Root package names requested by the operator.
    pub roots: Vec<String>,
    /// Exact accepted Cargo profile label, including test, bench, or custom profiles.
    pub profile: String,
}

/// CLI errors are deliberately not capability errors:
///
/// ```compile_fail
/// use mantle_rust_plan_app::{CompilerFacts, WorkspaceFactsRequest, WorkspaceFactsSource};
/// struct RunError;
/// struct CliWorkspace;
/// impl WorkspaceFactsSource for CliWorkspace {
///     fn load_workspace_facts(
///         &mut self,
///         _: &WorkspaceFactsRequest,
///         _: Option<&str>,
///         _: &CompilerFacts,
///     ) -> Result<(), RunError> {
///         unreachable!()
///     }
/// }
/// ```
pub trait WorkspaceFactsSource {
    /// Materialize the accepted receipt once from actual captured tool values.
    fn load_workspace_facts(
        &mut self,
        request: &WorkspaceFactsRequest,
        cargo_version: Option<&str>,
        compiler: &CompilerFacts,
    ) -> Result<(), AdapterError>;
}

/// Borrowed request for Cargo metadata and the unit graph. The adapter keeps
/// decoded host records privately for the subsequent workspace-facts stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleCaptureRequest<'a> {
    pub roots: &'a [String],
    pub profile: &'a str,
}

/// Port: inspect the selected Cargo binary, then (after compiler inspection)
/// capture and decode metadata and the unit graph. The no-Cargo path calls
/// neither method. Decoded Cargo JSON remains private to the shell adapter.
pub trait CargoOracleCapture {
    fn inspect_cargo_version(&mut self) -> Result<String, AdapterError>;
    fn capture_metadata_and_graph(&mut self, request: &OracleCaptureRequest<'_>) -> Result<(), AdapterError>;
}

/// Request for compiler inspection without copying the profile label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompilerInspectionRequest<'a> {
    pub profile: &'a str,
}

/// Actual selected rustc version output, consumed by receipt capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerFacts {
    pub rustc_version_verbose: String,
}

/// Port: inspect the selected compiler before metadata or native workspace
/// reads, matching the accepted CLI tool order.
pub trait CompilerInspection {
    fn inspect_compiler(&mut self, request: &CompilerInspectionRequest<'_>) -> Result<CompilerFacts, AdapterError>;
}

/// What the cache-aware executor actually observed while executing this
/// effect. `NotObserved` is not a miss: execution may have stopped before a
/// lookup, or no cache decision may be attestable from the receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheObservation {
    NotObserved,
    Miss,
    ReusedOutput,
    RestoredLocal,
    RestoredShared,
}

/// One actually executed process attempt, bound to its admitted invocation.
/// Cache hits produce no process attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessAttempt {
    pub effect: mantle_rust_plan_core::ResolvedUnitEffect,
    pub observation: mantle_rust_plan_core::ResolvedProcessObservation,
}

/// One completed unit with actual cache and process observations. Selected
/// produced artifact identities are digest strings read from the real receipt,
/// not host paths or identities invented by this application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitExecutionResult {
    pub observation: UnitObservation,
    pub cache: CacheObservation,
    pub process_attempts: Vec<ProcessAttempt>,
    /// Present only when a restored compiler artifact precedes a build script.
    pub restored_compiler: Option<mantle_rust_plan_core::RestoredCompilerArtifactObservation>,
    /// Only a real successful build-script process with failed metadata parsing.
    pub build_script_postprocess_failure: Option<BuildScriptPostprocessFailure>,
    /// Real selected artifact digest identities made available to consumers.
    pub produced_artifacts: Vec<String>,
}

/// Provenance for a metadata blocker after an actually successful build-script
/// process; no compiler or build-script execution failure is fabricated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildScriptPostprocessFailure {
    pub effect_id: mantle_rust_plan_core::EffectId,
    pub unit_id: mantle_rust_plan_core::UnitId,
    pub script_resolved_blake3: mantle_rust_plan_core::Blake3Digest,
    pub blocker_class: String,
}

/// Preparation binds dependencies and toolchain policy before any cache
/// access. A stopped preparation has an actual blocked receipt already held
/// by the shell and therefore no prepared cache request.
pub enum PrepareOutcome<P> {
    Ready {
        prepared: P,
        facts: mantle_rust_plan_core::ResolvedUnitFacts,
    },
    Stopped(UnitExecutionResult),
}

/// A cache terminal result includes verified reuse, restored artifacts, or a
/// blocked stale-cache receipt. Only `Miss` may proceed to a compiler attempt.
pub enum CacheRestore<R> {
    Terminal { receipt: R, cache: CacheObservation },
    Miss,
}

/// Port: real cache reuse and local/shared restoration over already prepared,
/// core-admitted process facts. The shell privately owns its prepared inputs.
///
/// A CLI `RunError` cannot replace the typed cache capability failure:
///
/// ```compile_fail
/// use mantle_rust_plan_app::{CacheRestore, RustCacheAccess};
/// use mantle_rust_plan_core::ResolvedUnitEffect;
/// struct RunError;
/// struct HostCache;
/// impl RustCacheAccess<(), ()> for HostCache {
///     fn restore_or_miss(
///         &mut self,
///         _: &mut (),
///         _: &ResolvedUnitEffect,
///     ) -> Result<CacheRestore<()>, RunError> {
///         unreachable!()
///     }
/// }
/// ```
pub trait RustCacheAccess<P, R> {
    fn restore_or_miss(
        &mut self,
        prepared: &mut P,
        effect: &mantle_rust_plan_core::ResolvedUnitEffect,
    ) -> Result<CacheRestore<R>, AdapterError>;
}

/// Port: prepare the selected unit, run rustc only on a real cache miss, then
/// finish topology bookkeeping for either restored or compiled artifacts.
pub trait UnitExecutor {
    type Prepared;
    type Receipt;

    fn prepare_unit(&mut self, effect: &ExistingUnitEffect) -> Result<PrepareOutcome<Self::Prepared>, AdapterError>;
    fn execute_miss(
        &mut self,
        prepared: &mut Self::Prepared,
        planned: &ExistingUnitEffect,
        effect: mantle_rust_plan_core::ResolvedUnitEffect,
    ) -> Result<Self::Receipt, AdapterError>;
    fn finish_unit(
        &mut self,
        planned: &ExistingUnitEffect,
        prepared: Self::Prepared,
        receipt: Self::Receipt,
        cache: CacheObservation,
        primary: Option<&mantle_rust_plan_core::ResolvedUnitEffect>,
    ) -> Result<UnitExecutionResult, AdapterError>;
}
