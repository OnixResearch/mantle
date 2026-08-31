// machine-artifact-public: self-build.cargo-free-reports
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use serde_json::json;

use crate::errors::RunError;

const SCHEMA: &str = "mantle-cargo-free-self-build-v1";
const FIXED_POINT_SCHEMA: &str = "mantle-cargo-free-fixed-point-proof-v1";
const RECEIPT_FILE: &str = "receipt.json";
const STDERR_FILE: &str = "stderr.txt";
const STATUS_FILE: &str = "status.txt";
const META_FILE: &str = "meta.json";
const PRE_FLIGHT_FILE: &str = "preflight.json";
const NON_CLAIMS_FILE: &str = "non-claims.txt";
const SMOKE_STDOUT_FILE: &str = "smoke-stdout.txt";
const SMOKE_STDERR_FILE: &str = "smoke-stderr.txt";
const EXECUTION_DIR: &str = "execution";
const RUST_CHILD_ACTION_EVIDENCE_DIR: &str = "rust-child-actions";
const RUST_CHILD_ACTION_AUTHORITY_FILE: &str = "authority.json";
const RUST_CHILD_ACTION_PLAN_FILE: &str = "plan.json";
const RUST_CHILD_ACTION_AUDIT_FILE: &str = "audit.json";
const RUST_CHILD_ACTION_RECONCILIATION_FILE: &str = "reconciliation.json";
const RUST_CHILD_ACTION_AUTHORITY_FLAG: &str = "--rust-child-action-authority";
const RUST_CHILD_ACTION_EVIDENCE_DIR_FLAG: &str = "--rust-child-action-evidence-dir";
const RUST_CHILD_ACTION_ALIAS_PRODUCER_CONTEXT: &[u8] = b"mantle-rust-child-action-alias-producer-v1\0";
const RUST_CHILD_ACTION_FIXED_AUTHORITY_CONTEXT: &[u8] = b"mantle-rust-child-action-fixed-authority-v1\0";
const BUNDLE_ROOT_RELATIVE_PATH: &str = ".";
const CARGO_SHIM_FILE: &str = "cargo-forbidden";
const CARGO_SHIM_DIR: &str = "cargo-guard-bin";
const CARGO_SHIM_NAME: &str = "cargo";
const CARGO_MARKER_FILE: &str = "cargo-was-invoked";
const C_COMPILER_ALIAS: &str = "cc";
const LINKER_ALIAS: &str = "ld";
const ARCHIVER_ALIAS: &str = "ar";
const RANLIB_ALIAS: &str = "ranlib";
const PKG_CONFIG_ALIAS: &str = "pkg-config";
const PRODUCED_MANTLE_FILE: &str = "mantle";
const MANTLE_TARGET_NAME: &str = "mantle";
const MANTLE_TARGET_KIND: &str = "bin";
const SUCCESS_STATUS: &str = "success";
const BLOCKED_STATUS: &str = "blocked";
const MISMATCH_STATUS: &str = "mismatch";
const STAGE1_DIR: &str = "stage1";
const STAGE2_DIR: &str = "stage2";
const JSON_FLAG: &str = "--json";
const RUST_PLAN_COMMAND: &str = "rust-plan";
const ROOT_FLAG: &str = "--root";
const CARGO_FLAG: &str = "--cargo";
const RUSTC_FLAG: &str = "--rustc";
const NO_CARGO_ORACLE_FLAG: &str = "--no-cargo-oracle";
const DETERMINISTIC_RELEASE_PATHS_FLAG: &str = "--deterministic-release-paths";
const EXECUTE_TOPOLOGY_FLAG: &str = "--execute-topology";
const EXECUTION_OUTPUT_ROOT_FLAG: &str = "--execution-output-root";
const HELP_FLAG: &str = "--help";
const TOOLCHAIN_DIR: &str = "toolchain";
const RUSTC_WRAPPER_FILE: &str = "rustc-normalized";
const RUSTC_RUNTIME_WRAPPER_FILE: &str = "rustc-receipt-bound-runtime";
const RUSTC_LIBSTDCXX_RUNTIME_FILE: &str = "libstdc++.so.6.0.28";
const RUSTC_LIBSTDCXX_SONAME: &str = "libstdc++.so.6";
const RUSTC_DYNAMIC_LOADER_FILE: &str = "ld-musl-x86_64.so.1";
const RUSTC_DYNAMIC_BINARY_FILE: &str = "rustc.dynamic";
const RUSTC_RUNTIME_HOST_TRIPLE: &str = "x86_64-unknown-linux-musl";
const RUSTC_BOUND_LINKER_ALIAS: &str = "cc";
const DYNAMIC_LIBRARY_PATH_ENV: &str = "LD_LIBRARY_PATH";
const COMPATIBILITY_FILE: &str = "compatibility.json";
const TOOLCHAIN_COMPATIBILITY_PATH_DIR: &str = "receipt-bound-path";
const TOOLCHAIN_ALIAS_RUNTIME_DIR: &str = ".toolchain-runtime";
const TOOLCHAIN_ALIAS_UNWIND_ARCHIVE: &str = "libunwind.a";
const TOOLCHAIN_ALIAS_CRT1_OBJECT: &str = "crt1.o";
const TOOLCHAIN_ALIAS_STATIC_PIE_CRT_OBJECT: &str = "rcrt1.o";
const TOOLCHAIN_ALIAS_STATIC_PIE_FLAG: &str = "-static-pie";
const TOOLCHAIN_ALIAS_STATIC_FLAG: &str = "-static";
const TOOLCHAIN_ALIAS_NON_PIE_FLAG: &str = "-no-pie";
const RUSTC_PROBE_DIR: &str = "rustc-probe";
const RUSTC_PROBE_SOURCE_FILE: &str = "probe.rs";
const RUSTC_PROBE_SOURCE: &str = "fn main() {}\n";
const LINK_SELF_CONTAINED_PROBE_ARG: &str = "link-self-contained=no";
const LINK_SELF_CONTAINED_JOINED_ARG: &str = "-Clink-self-contained=no";
const RUSTC_BOOTSTRAP_ENV: &str = "RUSTC_BOOTSTRAP";
const REAL_RUSTC_ENV: &str = "MANTLE_REAL_RUSTC";
const EXTERNAL_WRAPPER_BLOCKER: &str = "external-wrapper";
const WRAPPER_PROBE_BYTES_MAX: usize = 4096;
const SHEBANG_BYTES: &[u8] = b"#!";
const WRAPPER_RUSTC_MARKER: &[u8] = b"rustc";
const NORMALIZATION_NONE: &str = "none";
const NORMALIZATION_RECEIPT_BOUND_RUNTIME: &str = "receipt-bound-dynamic-runtime";
const NORMALIZATION_STRIP_LINK_SELF_CONTAINED: &str = "strip-link-self-contained-no";
const SIGNAL_STATUS_TEXT: &str = "signal";
const BLOCKED_SMOKE_STDOUT: &str = "not run: blocked before binary\n";
const BLOCKED_SMOKE_STDERR_PREFIX: &str = "not run: blocked before binary";
const SOURCE_DIGEST_FIELD: &str = "source_digest";
const SOURCE_DIGEST_ALGORITHM_FIELD: &str = "algorithm";
const RUSTC_SYSROOT_PRINT_ARG: &str = "sysroot";
const TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD: &str = "source_built_toolchain_closure_policy_digest_blake3";
const SELECTED_C_COMPILER_FIELD: &str = "selected_c_compiler";
const RUST_SOURCE_PROVIDER_BINDING_SCHEMA: &str = "mantle-cargo-free-rust-source-provider-binding-v1";
const RUST_SOURCE_PROVIDER_STATUS_ABSENT: &str = "absent";
const RUST_SOURCE_PROVIDER_STATUS_VALIDATED: &str = "validated";
const RUST_SOURCE_PROVIDER_REQUIRED_ROLE_COUNT: usize = 1;
const BASE_OBSERVED_TOOLCHAIN_INPUT_COUNT: usize = 2;
const PROVIDER_FIXED_POINT_STATUS_ABSENT: &str = "absent";
const PROVIDER_FIXED_POINT_STATUS_VALID: &str = "valid";
const PROVIDER_FIXED_POINT_STATUS_INVALID: &str = "invalid";
const ENFORCED_SOURCE_BUILT_STATUS: &str = "enforced-source-built";
const NOT_CRUNCH_BOOTSTRAP_NON_CLAIM: &str = "not-crunch-bootstrap";
const NOT_RELEASE_REPRODUCIBILITY_NON_CLAIM: &str = "not-release-reproducibility";
const NOT_FULL_CARGO_COMPATIBILITY_NON_CLAIM: &str = "not-full-cargo-compatibility";
const PROVIDER_PROOF_AUDIT_WORKFLOW: &str = "self-hosting";
const PROVIDER_PROOF_AUDIT_CLAIM: &str = "self-hosting";
const HERMETICITY_AUDIT_EVENTS_FIELD: &str = "hermeticity_audit_events";
const SOURCE_DIGEST_VALUE_FIELD: &str = "value";
const SUCCESS_EXIT_CODE: i32 = 0;
const FALLBACK_ERROR_EXIT_CODE: i32 = 1;
const EXPECTED_MANTLE_UNIT_COUNT: usize = 1;
const FIXED_POINT_STAGE1_INDEX: usize = 0;
const FIXED_POINT_STAGE2_INDEX: usize = 1;
const FIXED_POINT_STAGE_COUNT: usize = 2;
#[cfg(unix)]
const CARGO_SHIM_PERMISSIONS: u32 = 0o755;
#[cfg(unix)]
const UNIX_EXECUTE_BITS: u32 = 0o111;

pub(crate) struct CargoFreeSelfBuildOptions<'a> {
    pub(crate) root: &'a Path,
    pub(crate) out_dir: &'a Path,
    pub(crate) rustc: &'a Path,
    pub(crate) targets: &'a [String],
    pub(crate) toolchain_closure: Option<&'a Path>,
    pub(crate) rust_source_provider: Option<&'a Path>,
    pub(crate) rust_action_resources: Option<crate::source_built_rust_action_plan::RustActionResourceLimits>,
    pub(crate) hermeticity_mode: crunch_pipeline::HermeticityMode,
    pub(crate) json: bool,
}

#[derive(Clone, Debug)]
struct LoadedToolchainClosure {
    status: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    manifest_path: Option<PathBuf>,
    manifest: Option<crate::source_toolchain_closure::ToolchainClosureManifest>,
}

#[derive(Clone, Debug)]
struct LoadedRustSourceProvider {
    status: RustSourceProviderBindingStatus,
    rustc: Option<PathBuf>,
    source_built_shell: Option<BoundRustExecutionShell>,
    source_built_host_tools: Vec<crate::full_source_rust_binding::FullSourceRustHostToolBinding>,
    source_built_native_artifacts: Vec<crate::full_source_rust_binding::FullSourceNativeArtifactBinding>,
    source_built_action_trust: Option<crate::source_built_rust_provider_action::RustProviderActionEvidence>,
    toolchain_closure_status: Option<crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus>,
}

#[derive(Clone, Debug)]
struct BoundRustExecutionShell {
    execution_path: PathBuf,
    content_digest_blake3: String,
    source_id: String,
    construction_receipt_digest_blake3: String,
}

#[derive(Clone, Debug)]
struct BoundRustExecutionAuthority {
    shell: Option<BoundRustExecutionShell>,
    host_tools: Vec<crate::full_source_rust_binding::FullSourceRustHostToolBinding>,
    native_artifacts: Vec<crate::full_source_rust_binding::FullSourceNativeArtifactBinding>,
}

#[derive(Clone, Debug, Serialize)]
struct RustSourceProviderBindingStatus {
    schema: &'static str,
    status: String,
    provider_dir: Option<PathBuf>,
    metadata_path: Option<PathBuf>,
    metadata_digest_blake3: Option<String>,
    policy_digest_blake3: Option<String>,
    host_triple: Option<String>,
    target_triple: Option<String>,
    artifact_count: Option<usize>,
    source_count: Option<usize>,
    receipt_count: Option<usize>,
    rustc_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BoundRustcRuntime {
    library_dir: PathBuf,
    loader: PathBuf,
}

#[derive(Clone, Debug)]
struct ExecutionToolchain {
    rustc: PathBuf,
    path_env: OsString,
    status: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    c_compiler_route: Option<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
}

#[derive(Clone, Debug)]
struct CcCompilerAliasRuntimeInputs {
    unwind_archive: Option<PathBuf>,
    crt1_object: PathBuf,
}

#[derive(Clone, Copy, Debug)]
struct ProofArtifactRequest<'a> {
    pointer: &'a str,
    label: &'a str,
}

#[derive(Clone, Copy, Debug)]
struct ExpectedString<'a> {
    pointer: &'a str,
    value: &'a str,
    label: &'a str,
}

#[derive(Clone, Copy, Debug)]
struct ExpectedNull<'a> {
    pointer: &'a str,
    label: &'a str,
}

#[derive(Clone, Copy, Debug)]
struct FixedPointStageValidationRequest<'a> {
    meta: &'a Value,
    stage_name: &'a str,
    actual_binary_digest: Option<&'a str>,
    receipt: Option<&'a Value>,
    expected_policy_digest: Option<&'a str>,
}

#[derive(Clone, Debug)]
struct FixedPointStagePlanRequest<'a> {
    name: &'static str,
    root: &'a Path,
    bundle_dir: &'a Path,
    execution_dir: &'a Path,
    rustc: &'a Path,
    targets: &'a [String],
    mantle_binary: FixedPointMantleBinary,
}

#[derive(Clone, Copy, Debug)]
struct RustPlanChildRequest<'a> {
    paths: &'a BuildPaths,
    rustc: &'a Path,
    targets: &'a [String],
    path_env: &'a OsStr,
    c_compiler_route: Option<&'a crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
    policy_digest_blake3: Option<&'a str>,
    rust_action_authority_path: Option<&'a Path>,
    rust_action_evidence_dir: Option<&'a Path>,
}

#[derive(Clone, Copy, Debug)]
struct MalformedBlockedTopologyRequest<'a> {
    execution_status: &'a str,
    classification: &'a str,
    diagnostic: &'a str,
}

#[derive(Debug)]
struct FixedPointStageAssembly<'a> {
    stage: &'a FixedPointStagePlan,
    status_code: Option<i32>,
    receipt: Option<&'a Value>,
    execution_status: String,
    is_cargo_marker_absent: bool,
    blocker: Option<String>,
    blocker_diagnostic: Option<BlockedTopologyDiagnostic>,
    produced: Option<FixedPointStageArtifact>,
    policy_digest_blake3: Option<&'a str>,
    c_compiler_route: Option<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
}

#[derive(Debug)]
struct BlockedFixedPointStageRequest<'a> {
    stage: &'a FixedPointStagePlan,
    execution_status: &'a str,
    status_code: Option<i32>,
    blocker: String,
    policy_digest_blake3: Option<&'a str>,
    c_compiler_route: Option<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
}

#[derive(Clone, Copy, Debug)]
struct FixedPointSummaryContext<'a> {
    plan: &'a FixedPointPlan,
    compatibility: &'a RustcCompatibilitySummary,
    toolchain_closure: &'a crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: &'a RustSourceProviderBindingStatus,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
}

#[derive(Clone, Copy, Debug)]
struct FixedPointFinishContext<'a> {
    summary: FixedPointSummaryContext<'a>,
    json_mode: bool,
}

#[derive(Clone, Copy, Debug)]
struct SelfBuildSummaryContext<'a> {
    paths: &'a BuildPaths,
    child: &'a ChildRun,
    produced: Option<&'a ProducedBinary>,
    toolchain_closure: &'a crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: &'a RustSourceProviderBindingStatus,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
}

#[derive(Clone, Copy, Debug)]
struct SourceBuiltClosurePredicates {
    has_complete_counts: bool,
    has_zero_seed_exceptions: bool,
}

#[derive(Debug)]
struct BuildPaths {
    root: PathBuf,
    out_dir: PathBuf,
    execution_dir: PathBuf,
    receipt_path: PathBuf,
    stderr_path: PathBuf,
    status_path: PathBuf,
    marker_path: PathBuf,
    explicit_cargo_shim: PathBuf,
    path_cargo_shim: PathBuf,
    guard_path_dir: PathBuf,
    binary_path: PathBuf,
    meta_path: PathBuf,
    rust_action_evidence_dir: PathBuf,
    rust_action_authority_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FixedPointPlan {
    pub(crate) schema: &'static str,
    pub(crate) root: PathBuf,
    pub(crate) bundle_dir: PathBuf,
    pub(crate) shared_execution_dir: PathBuf,
    pub(crate) preflight_path: PathBuf,
    pub(crate) meta_path: PathBuf,
    pub(crate) non_claims_path: PathBuf,
    pub(crate) stages: [FixedPointStagePlan; FIXED_POINT_STAGE_COUNT],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FixedPointStagePlan {
    pub(crate) name: &'static str,
    pub(crate) stage_dir: PathBuf,
    pub(crate) execution_dir: PathBuf,
    pub(crate) receipt_path: PathBuf,
    pub(crate) stderr_path: PathBuf,
    pub(crate) status_path: PathBuf,
    pub(crate) cargo_marker_path: PathBuf,
    pub(crate) explicit_cargo_shim: PathBuf,
    pub(crate) path_cargo_shim: PathBuf,
    pub(crate) guard_path_dir: PathBuf,
    pub(crate) binary_path: PathBuf,
    pub(crate) smoke_stdout_path: PathBuf,
    pub(crate) smoke_stderr_path: PathBuf,
    pub(crate) command: FixedPointStageCommandPlan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FixedPointStageCommandPlan {
    pub(crate) mantle_binary: FixedPointMantleBinary,
    pub(crate) args: Vec<OsString>,
    pub(crate) current_dir: PathBuf,
    pub(crate) cargo_env_value: PathBuf,
    pub(crate) path_guard_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FixedPointMantleBinary {
    Host,
    StageOutput { stage_name: &'static str, path: PathBuf },
}

#[derive(Debug)]
struct ChildRun {
    status_code: Option<i32>,
    receipt: Option<Value>,
    execution_status: String,
    cargo_marker_absent: bool,
    blocker: Option<String>,
    blocker_diagnostic: Option<BlockedTopologyDiagnostic>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct BlockedTopologyDiagnostic {
    classification: String,
    topology_execution_status: String,
    root_blocked_unit: Option<String>,
    package_id: Option<String>,
    execution_role: Option<String>,
    selected_triple: Option<String>,
    target_kind: Option<String>,
    predecessor_status: Option<String>,
    blocker_class: Option<String>,
    diagnostic: String,
}

#[derive(Debug)]
struct ProducedBinary {
    path: PathBuf,
    blake3: String,
    source_digest: Value,
    source_closure_digest_blake3: Option<String>,
    smoke_status_code: Option<i32>,
}

#[derive(Debug, Serialize)]
struct RustcCompatibilitySummary {
    requested_rustc: PathBuf,
    stage_rustc: PathBuf,
    normalization: &'static str,
    wrapper: Option<PathBuf>,
    wrapper_blake3: Option<String>,
}

#[derive(Debug)]
struct RustcCompatibility {
    summary: RustcCompatibilitySummary,
}

#[derive(Debug)]
struct FixedPointStageRun {
    name: &'static str,
    dir: PathBuf,
    execution_dir: PathBuf,
    receipt_path: PathBuf,
    stderr_path: PathBuf,
    status_path: PathBuf,
    status_code: Option<i32>,
    execution_status: String,
    cargo_marker_absent: bool,
    success: bool,
    unit_count: u64,
    failed_unit_count: u64,
    binary: Option<PathBuf>,
    binary_blake3: Option<String>,
    smoke_status_code: Option<i32>,
    source_built_toolchain_closure_policy_digest_blake3: Option<String>,
    selected_c_compiler: Option<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
    blocker: Option<String>,
    blocker_diagnostic: Option<BlockedTopologyDiagnostic>,
}

#[derive(Debug)]
struct FixedPointStageArtifact {
    binary: PathBuf,
    digest: String,
    smoke_status_code: i32,
}

#[derive(Debug, Serialize)]
struct FixedPointSummary {
    schema: &'static str,
    status: String,
    root: PathBuf,
    bundle_dir: PathBuf,
    fixed_point: bool,
    hermeticity_mode: String,
    proof_eligibility: crunch_release_core::StrictProofEligibilityReport,
    stage1: FixedPointStageSummary,
    stage2: Option<FixedPointStageSummary>,
    rustc_compatibility: RustcCompatibilitySummary,
    source_built_toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: RustSourceProviderBindingStatus,
    blocker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocker_diagnostic: Option<BlockedTopologyDiagnostic>,
    non_claims: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct FixedPointStageSummary {
    name: &'static str,
    dir: PathBuf,
    execution_dir: PathBuf,
    receipt: PathBuf,
    stderr: PathBuf,
    status: PathBuf,
    status_code: Option<i32>,
    execution_status: String,
    cargo_marker_absent: bool,
    success: bool,
    unit_count: u64,
    failed_unit_count: u64,
    binary: Option<PathBuf>,
    binary_blake3: Option<String>,
    smoke_status_code: Option<i32>,
    source_built_toolchain_closure_policy_digest_blake3: Option<String>,
    selected_c_compiler: Option<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
    rust_child_action_authority: Option<PathBuf>,
    rust_child_action_plan: Option<PathBuf>,
    rust_child_action_audit: Option<PathBuf>,
    rust_child_action_reconciliation: Option<PathBuf>,
    blocker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocker_diagnostic: Option<BlockedTopologyDiagnostic>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct ProviderFixedPointProofVerification {
    pub(crate) status: String,
    pub(crate) valid: bool,
    pub(crate) proof_source: String,
    pub(crate) proof_dir: Option<PathBuf>,
    pub(crate) proof_artifact_digest_blake3: Option<String>,
    pub(crate) bounded_evidence_role: Option<String>,
    pub(crate) meta_digest_blake3: Option<String>,
    pub(crate) closure_policy_digest_blake3: Option<String>,
    pub(crate) stage_binary_digest_blake3: Option<String>,
    pub(crate) release_artifact_relative_path: Option<String>,
    pub(crate) release_artifact_digest_blake3: Option<String>,
    pub(crate) stage1_unit_count: Option<u64>,
    pub(crate) stage2_unit_count: Option<u64>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) blockers: Vec<String>,
}

impl ProviderFixedPointProofVerification {
    pub(crate) fn absent(required: bool) -> Self {
        let blockers = if required {
            vec!["missing provider fixed-point proof bundle".to_string()]
        } else {
            Vec::new()
        };
        Self {
            status: PROVIDER_FIXED_POINT_STATUS_ABSENT.to_string(),
            valid: false,
            proof_source: PROVIDER_FIXED_POINT_STATUS_ABSENT.to_string(),
            proof_dir: None,
            proof_artifact_digest_blake3: None,
            bounded_evidence_role: None,
            meta_digest_blake3: None,
            closure_policy_digest_blake3: None,
            stage_binary_digest_blake3: None,
            release_artifact_relative_path: None,
            release_artifact_digest_blake3: None,
            stage1_unit_count: None,
            stage2_unit_count: None,
            non_claims: Vec::new(),
            blockers,
        }
    }
}

#[derive(Debug)]
struct ProviderFixedPointProofEvidence {
    proof_dir: PathBuf,
    meta: Option<Value>,
    meta_digest_blake3: Option<String>,
    preflight: Option<Value>,
    non_claims_text: Option<String>,
    stage1_binary_digest_actual: Option<String>,
    stage2_binary_digest_actual: Option<String>,
    stage1_receipt: Option<Value>,
    stage2_receipt: Option<Value>,
    shell_blockers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SelfBuildSummary {
    schema: &'static str,
    status: String,
    root: PathBuf,
    out_dir: PathBuf,
    binary: Option<PathBuf>,
    binary_blake3: Option<String>,
    source_digest: Option<Value>,
    source_closure_digest_blake3: Option<String>,
    receipt: PathBuf,
    stderr: PathBuf,
    status_code: Option<i32>,
    execution_status: String,
    cargo_marker_absent: bool,
    unit_count: u64,
    failed_unit_count: u64,
    smoke_status_code: Option<i32>,
    hermeticity_mode: String,
    blocker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blocker_diagnostic: Option<BlockedTopologyDiagnostic>,
    source_built_toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: RustSourceProviderBindingStatus,
    non_claims: Vec<&'static str>,
}

pub(crate) fn verify_provider_fixed_point_proof_bundle(proof_dir: &Path) -> ProviderFixedPointProofVerification {
    let mut shell_blockers = Vec::new();
    let meta_path = proof_dir.join(META_FILE);
    let preflight_path = proof_dir.join(PRE_FLIGHT_FILE);
    let non_claims_path = proof_dir.join(NON_CLAIMS_FILE);
    debug_assert_ne!(meta_path, preflight_path);
    debug_assert_ne!(preflight_path, non_claims_path);
    let (meta, meta_digest_blake3) = read_json_with_digest(&meta_path, "fixed-point meta", &mut shell_blockers);
    let (preflight, _) = read_json_with_digest(&preflight_path, "fixed-point preflight", &mut shell_blockers);
    let non_claims_text = read_text_optional(&non_claims_path, "fixed-point non-claims", &mut shell_blockers);
    let stage1_binary_digest_actual = stage_binary_digest_from_meta(
        proof_dir,
        meta.as_ref(),
        ProofArtifactRequest {
            pointer: "/stage1/binary",
            label: "stage1 binary",
        },
        &mut shell_blockers,
    );
    let stage2_binary_digest_actual = stage_binary_digest_from_meta(
        proof_dir,
        meta.as_ref(),
        ProofArtifactRequest {
            pointer: "/stage2/binary",
            label: "stage2 binary",
        },
        &mut shell_blockers,
    );
    let stage1_receipt = stage_receipt_from_meta(
        proof_dir,
        meta.as_ref(),
        ProofArtifactRequest {
            pointer: "/stage1/receipt",
            label: "stage1 receipt",
        },
        &mut shell_blockers,
    );
    let stage2_receipt = stage_receipt_from_meta(
        proof_dir,
        meta.as_ref(),
        ProofArtifactRequest {
            pointer: "/stage2/receipt",
            label: "stage2 receipt",
        },
        &mut shell_blockers,
    );
    validate_provider_fixed_point_proof_evidence(ProviderFixedPointProofEvidence {
        proof_dir: proof_dir.to_path_buf(),
        meta,
        meta_digest_blake3,
        preflight,
        non_claims_text,
        stage1_binary_digest_actual,
        stage2_binary_digest_actual,
        stage1_receipt,
        stage2_receipt,
        shell_blockers,
    })
}

fn validate_provider_fixed_point_proof_evidence(
    mut evidence: ProviderFixedPointProofEvidence,
) -> ProviderFixedPointProofVerification {
    debug_assert_ne!(FIXED_POINT_SCHEMA, SCHEMA);
    debug_assert!(evidence.shell_blockers.capacity() >= evidence.shell_blockers.len());
    let mut blockers = std::mem::take(&mut evidence.shell_blockers);
    let Some(meta) = evidence.meta.as_ref() else {
        blockers.push("fixed-point meta.json is missing or invalid".to_string());
        return provider_fixed_point_result(evidence, None, None, None, blockers);
    };
    validate_fixed_point_meta(meta, &mut blockers);
    validate_fixed_point_proof_eligibility(meta, &mut blockers);
    validate_fixed_point_preflight(meta, evidence.preflight.as_ref(), &mut blockers);
    validate_fixed_point_non_claims(meta, evidence.non_claims_text.as_deref(), &mut blockers);
    let closure_policy_digest = fixed_point_closure_policy_digest(meta, &mut blockers);
    let stage1 = validate_fixed_point_stage(
        FixedPointStageValidationRequest {
            meta,
            stage_name: STAGE1_DIR,
            actual_binary_digest: evidence.stage1_binary_digest_actual.as_deref(),
            receipt: evidence.stage1_receipt.as_ref(),
            expected_policy_digest: closure_policy_digest.as_deref(),
        },
        &mut blockers,
    );
    let stage2 = validate_fixed_point_stage(
        FixedPointStageValidationRequest {
            meta,
            stage_name: STAGE2_DIR,
            actual_binary_digest: evidence.stage2_binary_digest_actual.as_deref(),
            receipt: evidence.stage2_receipt.as_ref(),
            expected_policy_digest: closure_policy_digest.as_deref(),
        },
        &mut blockers,
    );
    let matching_digest =
        matching_stage_binary_digest(stage1.binary_digest.as_deref(), stage2.binary_digest.as_deref(), &mut blockers);
    provider_fixed_point_result(
        evidence,
        closure_policy_digest,
        matching_digest,
        Some((stage1.unit_count, stage2.unit_count)),
        blockers,
    )
}

fn provider_fixed_point_result(
    evidence: ProviderFixedPointProofEvidence,
    closure_policy_digest: Option<String>,
    stage_binary_digest: Option<String>,
    stage_unit_counts: Option<(Option<u64>, Option<u64>)>,
    blockers: Vec<String>,
) -> ProviderFixedPointProofVerification {
    let is_valid = blockers.is_empty();
    debug_assert_eq!(is_valid, blockers.is_empty());
    debug_assert_ne!(PROVIDER_FIXED_POINT_STATUS_VALID, PROVIDER_FIXED_POINT_STATUS_INVALID);
    let (stage1_unit_count, stage2_unit_count) = stage_unit_counts.unwrap_or((None, None));
    let non_claims = non_claims_from_meta(evidence.meta.as_ref());
    let proof_dir = evidence.proof_dir;
    let meta_digest_blake3 = evidence.meta_digest_blake3;
    ProviderFixedPointProofVerification {
        status: if is_valid {
            PROVIDER_FIXED_POINT_STATUS_VALID
        } else {
            PROVIDER_FIXED_POINT_STATUS_INVALID
        }
        .to_string(),
        valid: is_valid,
        proof_source: "direct".to_string(),
        proof_dir: Some(proof_dir),
        proof_artifact_digest_blake3: None,
        bounded_evidence_role: None,
        meta_digest_blake3,
        closure_policy_digest_blake3: closure_policy_digest,
        stage_binary_digest_blake3: stage_binary_digest,
        release_artifact_relative_path: None,
        release_artifact_digest_blake3: None,
        stage1_unit_count,
        stage2_unit_count,
        non_claims,
        blockers,
    }
}

#[derive(Debug, Default)]
struct StageProofFacts {
    binary_digest: Option<String>,
    unit_count: Option<u64>,
}

fn validate_fixed_point_meta(meta: &Value, blockers: &mut Vec<String>) {
    let blocker_count_before = blockers.len();
    debug_assert_ne!(FIXED_POINT_SCHEMA, SCHEMA);
    expect_string(
        meta,
        ExpectedString {
            pointer: "/schema",
            value: FIXED_POINT_SCHEMA,
            label: "fixed-point schema",
        },
        blockers,
    );
    expect_string(
        meta,
        ExpectedString {
            pointer: "/status",
            value: SUCCESS_STATUS,
            label: "fixed-point status",
        },
        blockers,
    );
    expect_bool(meta, "/fixed_point", true, "fixed-point flag", blockers);
    expect_null(
        meta,
        ExpectedNull {
            pointer: "/blocker",
            label: "fixed-point blocker",
        },
        blockers,
    );
    expect_string(
        meta,
        ExpectedString {
            pointer: "/source_built_toolchain_closure/status",
            value: ENFORCED_SOURCE_BUILT_STATUS,
            label: "source-built closure status",
        },
        blockers,
    );
    expect_bool(meta, "/source_built_toolchain_closure/claim", true, "source-built closure claim", blockers);
    expect_u64(
        meta,
        "/source_built_toolchain_closure/seed_exception_count",
        0,
        "source-built seed exceptions",
        blockers,
    );
    expect_string(
        meta,
        ExpectedString {
            pointer: "/rust_source_provider/status",
            value: RUST_SOURCE_PROVIDER_STATUS_VALIDATED,
            label: "Rust source provider status",
        },
        blockers,
    );
    let member_count = optional_u64(meta, "/source_built_toolchain_closure/member_count");
    let source_built_count = optional_u64(meta, "/source_built_toolchain_closure/source_built_member_count");
    match (member_count, source_built_count) {
        (Some(member_count), Some(source_built_count)) if member_count > 0 && member_count == source_built_count => {}
        (Some(member_count), Some(source_built_count)) => blockers.push(format!(
            "source-built closure member counts are not fully source-built: member_count={member_count} source_built_member_count={source_built_count}"
        )),
        _ => blockers.push("source-built closure member counts are missing".to_string()),
    }
    debug_assert!(blockers.len() >= blocker_count_before);
}

fn validate_fixed_point_proof_eligibility(meta: &Value, blockers: &mut Vec<String>) {
    let eligibility_decision = fixed_point_proof_eligibility_report_from_meta(meta);
    if eligibility_decision.strict_claim_satisfied {
        return;
    }
    blockers.extend(crunch_release_core::proof_eligibility_blocking_reasons(eligibility_decision));
}

fn fixed_point_proof_eligibility_report_from_meta(meta: &Value) -> crunch_release_core::StrictProofEligibilityReport {
    crunch_release_core::strict_proof_eligibility_gate(crunch_release_core::StrictProofEligibilityInput {
        workflow: PROVIDER_PROOF_AUDIT_WORKFLOW.to_string(),
        requested_claim: PROVIDER_PROOF_AUDIT_CLAIM.to_string(),
        hermeticity_mode: optional_str(meta, "/hermeticity_mode").unwrap_or_default().to_string(),
        hermeticity_audit_events: fixed_point_proof_audit_events(meta),
        closure_status: fixed_point_closure_fact_status_from_meta(meta),
        protected_environment_status: fixed_point_protected_environment_status_from_meta(meta),
        host_tool_status: fixed_point_host_tool_status_from_meta(meta),
    })
}

fn fixed_point_closure_fact_status_from_meta(meta: &Value) -> crunch_release_core::ProofFactStatus {
    debug_assert!("/source_built_toolchain_closure/status".starts_with('/'));
    debug_assert_ne!(ENFORCED_SOURCE_BUILT_STATUS, RUST_SOURCE_PROVIDER_STATUS_VALIDATED);
    let status = optional_str(meta, "/source_built_toolchain_closure/status");
    let claim = meta.pointer("/source_built_toolchain_closure/claim").and_then(Value::as_bool);
    let policy = optional_str(meta, "/source_built_toolchain_closure/policy_digest_blake3");
    let member_count = optional_u64(meta, "/source_built_toolchain_closure/member_count");
    let source_built_count = optional_u64(meta, "/source_built_toolchain_closure/source_built_member_count");
    let seed_exception_count = optional_u64(meta, "/source_built_toolchain_closure/seed_exception_count");
    let has_complete_counts = source_built_counts_are_complete(member_count, source_built_count);
    if source_built_meta_facts_are_satisfied(status, claim, policy, has_complete_counts, seed_exception_count) {
        return crunch_release_core::ProofFactStatus::Satisfied;
    }
    if source_built_meta_facts_are_missing(status, claim, policy, member_count, source_built_count) {
        return crunch_release_core::ProofFactStatus::Missing;
    }
    crunch_release_core::ProofFactStatus::Degraded
}

fn source_built_counts_are_complete(member_count: Option<u64>, source_built_count: Option<u64>) -> bool {
    matches!(
        (member_count, source_built_count),
        (Some(member_count), Some(source_built_count)) if member_count > 0 && member_count == source_built_count
    )
}

fn source_built_meta_facts_are_satisfied(
    status: Option<&str>,
    claim: Option<bool>,
    policy: Option<&str>,
    has_complete_counts: bool,
    seed_exception_count: Option<u64>,
) -> bool {
    if status != Some(ENFORCED_SOURCE_BUILT_STATUS) {
        return false;
    }
    if claim != Some(true) {
        return false;
    }
    if policy.is_none() {
        return false;
    }
    if !has_complete_counts {
        return false;
    }
    seed_exception_count == Some(0)
}

fn source_built_meta_facts_are_missing(
    status: Option<&str>,
    claim: Option<bool>,
    policy: Option<&str>,
    member_count: Option<u64>,
    source_built_count: Option<u64>,
) -> bool {
    if status.is_none() {
        return true;
    }
    if claim.is_none() {
        return true;
    }
    if policy.is_none() {
        return true;
    }
    if member_count.is_none() {
        return true;
    }
    source_built_count.is_none()
}

fn fixed_point_protected_environment_status_from_meta(meta: &Value) -> crunch_release_core::ProofFactStatus {
    proof_fact_status_label(meta, "/protected_environment_status")
        .unwrap_or(crunch_release_core::ProofFactStatus::Satisfied)
}

fn fixed_point_host_tool_status_from_meta(meta: &Value) -> crunch_release_core::ProofFactStatus {
    let status = optional_str(meta, "/rust_source_provider/status");
    if status == Some(RUST_SOURCE_PROVIDER_STATUS_VALIDATED) {
        return crunch_release_core::ProofFactStatus::Satisfied;
    }
    if status.is_none() {
        return crunch_release_core::ProofFactStatus::Missing;
    }
    crunch_release_core::ProofFactStatus::Degraded
}

fn proof_fact_status_label(meta: &Value, pointer: &str) -> Option<crunch_release_core::ProofFactStatus> {
    match optional_str(meta, pointer)? {
        "satisfied" => Some(crunch_release_core::ProofFactStatus::Satisfied),
        "missing" => Some(crunch_release_core::ProofFactStatus::Missing),
        "degraded" | "leaked" | "undeclared" => Some(crunch_release_core::ProofFactStatus::Degraded),
        _ => Some(crunch_release_core::ProofFactStatus::Degraded),
    }
}

fn fixed_point_proof_audit_events(meta: &Value) -> Vec<String> {
    let mut events = Vec::new();
    collect_audit_events_from_object(meta, &mut events);
    if let Some(stage1) = meta.get(STAGE1_DIR) {
        collect_audit_events_from_object(stage1, &mut events);
    }
    if let Some(stage2) = meta.get(STAGE2_DIR) {
        collect_audit_events_from_object(stage2, &mut events);
    }
    events.sort();
    events.dedup();
    events
}

fn collect_audit_events_from_object(value: &Value, events: &mut Vec<String>) {
    let Some(array) = value.get(HERMETICITY_AUDIT_EVENTS_FIELD).and_then(Value::as_array) else {
        return;
    };
    events.extend(array.iter().filter_map(Value::as_str).map(ToOwned::to_owned));
}

fn validate_fixed_point_preflight(meta: &Value, preflight: Option<&Value>, blockers: &mut Vec<String>) {
    debug_assert_ne!(FIXED_POINT_SCHEMA, SCHEMA);
    debug_assert!(blockers.capacity() >= blockers.len());
    let Some(preflight) = preflight else {
        blockers.push("fixed-point preflight.json is missing or invalid".to_string());
        return;
    };
    expect_string(
        preflight,
        ExpectedString {
            pointer: "/schema",
            value: FIXED_POINT_SCHEMA,
            label: "preflight schema",
        },
        blockers,
    );
    let meta_policy = optional_str(meta, "/source_built_toolchain_closure/policy_digest_blake3");
    let preflight_policy = optional_str(preflight, "/source_built_toolchain_closure/policy_digest_blake3");
    if meta_policy.is_none() {
        blockers.push("fixed-point meta is missing closure policy digest".to_string());
    }
    if preflight_policy.is_none() {
        blockers.push("fixed-point preflight is missing closure policy digest".to_string());
    }
    if meta_policy.is_some() && preflight_policy.is_some() && meta_policy != preflight_policy {
        blockers.push("fixed-point preflight closure policy digest does not match meta.json".to_string());
    }
}

fn validate_fixed_point_non_claims(meta: &Value, non_claims_text: Option<&str>, blockers: &mut Vec<String>) {
    let blocker_count_before = blockers.len();
    debug_assert_ne!(NOT_CRUNCH_BOOTSTRAP_NON_CLAIM, NOT_RELEASE_REPRODUCIBILITY_NON_CLAIM);
    let non_claims = non_claims_from_meta(Some(meta));
    for required in [
        NOT_CRUNCH_BOOTSTRAP_NON_CLAIM,
        NOT_RELEASE_REPRODUCIBILITY_NON_CLAIM,
        NOT_FULL_CARGO_COMPATIBILITY_NON_CLAIM,
    ] {
        if !non_claims.iter().any(|claim| claim == required) {
            blockers.push(format!("fixed-point meta missing bounded non-claim {required}"));
        }
    }
    if non_claims.iter().any(|claim| claim == crate::source_toolchain_closure::SOURCE_BUILT_NON_CLAIM) {
        blockers.push("provider fixed-point proof still carries source-built closure non-claim".to_string());
    }
    let Some(text) = non_claims_text else {
        blockers.push("fixed-point non-claims.txt is missing or unreadable".to_string());
        return;
    };
    for phrase in ["does not claim", "release reproducibility", "full Cargo compatibility"] {
        if !text.contains(phrase) {
            blockers.push(format!("fixed-point non-claims.txt missing phrase: {phrase}"));
        }
    }
    debug_assert!(blockers.len() >= blocker_count_before);
}

fn fixed_point_closure_policy_digest(meta: &Value, blockers: &mut Vec<String>) -> Option<String> {
    let policy = optional_str(meta, "/source_built_toolchain_closure/policy_digest_blake3");
    if policy.is_none() {
        blockers.push("fixed-point closure policy digest is missing".to_string());
    }
    policy.map(ToOwned::to_owned)
}

fn validate_fixed_point_stage(
    request: FixedPointStageValidationRequest<'_>,
    blockers: &mut Vec<String>,
) -> StageProofFacts {
    let blocker_count_before = blockers.len();
    debug_assert!(matches!(request.stage_name, STAGE1_DIR | STAGE2_DIR));
    let prefix = format!("/{}", request.stage_name);
    validate_fixed_point_stage_identity(request, &prefix, blockers);
    validate_fixed_point_stage_execution(request, &prefix, blockers);
    let unit_count = optional_u64(request.meta, &format!("{prefix}/unit_count"));
    if !matches!(unit_count, Some(count) if count > 0) {
        blockers.push(format!("{} unit count is missing or zero", request.stage_name));
    }
    let declared_digest = optional_str(request.meta, &format!("{prefix}/binary_blake3")).map(ToOwned::to_owned);
    validate_declared_stage_digest(
        request.stage_name,
        declared_digest.as_deref(),
        request.actual_binary_digest,
        blockers,
    );
    let stage_policy = optional_str(request.meta, &format!("{prefix}/{TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD}"));
    validate_stage_policy_digest(request.stage_name, stage_policy, request.expected_policy_digest, blockers);
    validate_fixed_point_stage_receipt(request.stage_name, request.receipt, unit_count, blockers);
    debug_assert!(blockers.len() >= blocker_count_before);
    StageProofFacts {
        binary_digest: declared_digest,
        unit_count,
    }
}

fn validate_fixed_point_stage_identity(
    request: FixedPointStageValidationRequest<'_>,
    prefix: &str,
    blockers: &mut Vec<String>,
) {
    debug_assert!(prefix.starts_with('/'));
    debug_assert!(matches!(request.stage_name, STAGE1_DIR | STAGE2_DIR));
    expect_string(
        request.meta,
        ExpectedString {
            pointer: &format!("{prefix}/name"),
            value: request.stage_name,
            label: "fixed-point stage name",
        },
        blockers,
    );
    expect_bool(request.meta, &format!("{prefix}/success"), true, "fixed-point stage success", blockers);
    expect_i64(
        request.meta,
        &format!("{prefix}/status_code"),
        i64::from(SUCCESS_EXIT_CODE),
        "fixed-point stage status code",
        blockers,
    );
    expect_string(
        request.meta,
        ExpectedString {
            pointer: &format!("{prefix}/execution_status"),
            value: SUCCESS_STATUS,
            label: "fixed-point stage execution status",
        },
        blockers,
    );
}

fn validate_fixed_point_stage_execution(
    request: FixedPointStageValidationRequest<'_>,
    prefix: &str,
    blockers: &mut Vec<String>,
) {
    debug_assert!(prefix.starts_with('/'));
    debug_assert!(matches!(request.stage_name, STAGE1_DIR | STAGE2_DIR));
    expect_bool(
        request.meta,
        &format!("{prefix}/cargo_marker_absent"),
        true,
        "fixed-point stage Cargo guard",
        blockers,
    );
    expect_u64(
        request.meta,
        &format!("{prefix}/failed_unit_count"),
        0,
        "fixed-point stage failed unit count",
        blockers,
    );
    expect_i64(
        request.meta,
        &format!("{prefix}/smoke_status_code"),
        i64::from(SUCCESS_EXIT_CODE),
        "fixed-point stage smoke status",
        blockers,
    );
    expect_null(
        request.meta,
        ExpectedNull {
            pointer: &format!("{prefix}/blocker"),
            label: "fixed-point stage blocker",
        },
        blockers,
    );
}

fn validate_declared_stage_digest(
    stage_name: &str,
    declared_digest: Option<&str>,
    actual_binary_digest: Option<&str>,
    blockers: &mut Vec<String>,
) {
    match (declared_digest, actual_binary_digest) {
        (Some(declared), Some(actual)) if declared == actual => {}
        (Some(declared), Some(actual)) => {
            blockers.push(format!("{stage_name} binary digest mismatch: declared {declared} actual {actual}"));
        }
        (Some(_), None) => blockers.push(format!("{stage_name} binary could not be hashed")),
        (None, _) => blockers.push(format!("{stage_name} binary digest is missing")),
    }
}

fn validate_stage_policy_digest(
    stage_name: &str,
    stage_policy: Option<&str>,
    expected_policy: Option<&str>,
    blockers: &mut Vec<String>,
) {
    if let (Some(stage_policy), Some(expected_policy)) = (stage_policy, expected_policy) {
        if stage_policy != expected_policy {
            blockers.push(format!("{stage_name} closure policy digest does not match the proof closure digest"));
        }
        return;
    }
    blockers.push(format!("{stage_name} closure policy digest is missing"));
}

fn validate_fixed_point_stage_receipt(
    stage_name: &str,
    receipt: Option<&Value>,
    expected_unit_count: Option<u64>,
    blockers: &mut Vec<String>,
) {
    debug_assert!(matches!(stage_name, STAGE1_DIR | STAGE2_DIR));
    debug_assert_ne!(STAGE1_DIR, STAGE2_DIR);
    let Some(receipt) = receipt else {
        blockers.push(format!("{stage_name} receipt is missing or invalid"));
        return;
    };
    expect_string(
        receipt,
        ExpectedString {
            pointer: "/topology_execution/execution_status",
            value: SUCCESS_STATUS,
            label: "stage receipt execution status",
        },
        blockers,
    );
    let Some(units) = receipt.pointer("/topology_execution/unit_executions").and_then(Value::as_array) else {
        blockers.push(format!("{stage_name} receipt missing unit executions"));
        return;
    };
    if let Some(expected_unit_count) = expected_unit_count
        && units.len() as u64 != expected_unit_count
    {
        blockers.push(format!(
            "{stage_name} receipt unit count {} does not match summary unit count {expected_unit_count}",
            units.len()
        ));
    }
    if units.iter().any(|unit| optional_str(unit, "/execution_status") != Some(SUCCESS_STATUS)) {
        blockers.push(format!("{stage_name} receipt contains non-success unit executions"));
    }
}

fn matching_stage_binary_digest(
    stage1_digest: Option<&str>,
    stage2_digest: Option<&str>,
    blockers: &mut Vec<String>,
) -> Option<String> {
    match (stage1_digest, stage2_digest) {
        (Some(stage1), Some(stage2)) if stage1 == stage2 => Some(stage1.to_string()),
        (Some(stage1), Some(stage2)) => {
            blockers.push(format!("fixed-point stage binary digests differ: stage1 {stage1} stage2 {stage2}"));
            None
        }
        _ => {
            blockers.push("fixed-point stage binary digests are incomplete".to_string());
            None
        }
    }
}

fn read_json_with_digest(path: &Path, label: &str, blockers: &mut Vec<String>) -> (Option<Value>, Option<String>) {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => {
            blockers.push(format!("read {label} {}: {err}", path.display()));
            return (None, None);
        }
    };
    let digest = blake3::hash(&bytes).to_hex().to_string();
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(value) => (Some(value), Some(digest)),
        Err(err) => {
            blockers.push(format!("parse {label} {}: {err}", path.display()));
            (None, Some(digest))
        }
    }
}

fn read_text_optional(path: &Path, label: &str, blockers: &mut Vec<String>) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(err) => {
            blockers.push(format!("read {label} {}: {err}", path.display()));
            None
        }
    }
}

fn stage_binary_digest_from_meta(
    proof_dir: &Path,
    meta: Option<&Value>,
    artifact: ProofArtifactRequest<'_>,
    blockers: &mut Vec<String>,
) -> Option<String> {
    let path = proof_path_from_meta(proof_dir, meta, artifact, blockers)?;
    match blake3_file(&path) {
        Ok(digest) => Some(digest),
        Err(err) => {
            blockers.push(format!("hash {} {}: {}", artifact.label, path.display(), err.message()));
            None
        }
    }
}

fn stage_receipt_from_meta(
    proof_dir: &Path,
    meta: Option<&Value>,
    artifact: ProofArtifactRequest<'_>,
    blockers: &mut Vec<String>,
) -> Option<Value> {
    let path = proof_path_from_meta(proof_dir, meta, artifact, blockers)?;
    let (value, _) = read_json_with_digest(&path, artifact.label, blockers);
    value
}

fn proof_path_from_meta(
    proof_dir: &Path,
    meta: Option<&Value>,
    artifact: ProofArtifactRequest<'_>,
    blockers: &mut Vec<String>,
) -> Option<PathBuf> {
    debug_assert!(artifact.pointer.starts_with('/'));
    debug_assert!(!artifact.label.is_empty());
    let meta = meta?;
    let Some(raw) = optional_str(meta, artifact.pointer) else {
        blockers.push(format!("fixed-point meta missing {} path at {}", artifact.label, artifact.pointer));
        return None;
    };
    let path = Path::new(raw);
    if path.is_absolute() {
        if let Some(local_path) = copied_proof_path(proof_dir, meta, path)
            && local_path.exists()
        {
            return Some(local_path);
        }
        return Some(path.to_path_buf());
    }
    Some(proof_dir.join(path))
}

fn copied_proof_path(proof_dir: &Path, meta: &Value, absolute_path: &Path) -> Option<PathBuf> {
    let bundle_dir = optional_str(meta, "/bundle_dir").map(Path::new)?;
    let relative_path = absolute_path.strip_prefix(bundle_dir).ok()?;
    if relative_path.components().any(|component| !matches!(component, Component::Normal(_))) {
        return None;
    }
    Some(proof_dir.join(relative_path))
}

fn non_claims_from_meta(meta: Option<&Value>) -> Vec<String> {
    meta.and_then(|meta| meta.pointer("/non_claims"))
        .and_then(Value::as_array)
        .map(|claims| claims.iter().filter_map(Value::as_str).map(ToOwned::to_owned).collect())
        .unwrap_or_default()
}

fn optional_str<'a>(value: &'a Value, pointer: &str) -> Option<&'a str> {
    value.pointer(pointer).and_then(Value::as_str)
}

fn optional_u64(value: &Value, pointer: &str) -> Option<u64> {
    value.pointer(pointer).and_then(Value::as_u64)
}

fn expect_string(value: &Value, expected: ExpectedString<'_>, blockers: &mut Vec<String>) {
    debug_assert!(expected.pointer.starts_with('/'));
    debug_assert!(!expected.label.is_empty());
    match optional_str(value, expected.pointer) {
        Some(actual) if actual == expected.value => {}
        Some(actual) => blockers.push(format!("{} expected {}, got {actual}", expected.label, expected.value)),
        None => blockers.push(format!("{} is missing at {}", expected.label, expected.pointer)),
    }
}

fn expect_bool(value: &Value, pointer: &str, expected: bool, label: &str, blockers: &mut Vec<String>) {
    match value.pointer(pointer).and_then(Value::as_bool) {
        Some(actual) if actual == expected => {}
        Some(actual) => blockers.push(format!("{label} expected {expected}, got {actual}")),
        None => blockers.push(format!("{label} is missing at {pointer}")),
    }
}

fn expect_i64(value: &Value, pointer: &str, expected: i64, label: &str, blockers: &mut Vec<String>) {
    match value.pointer(pointer).and_then(Value::as_i64) {
        Some(actual) if actual == expected => {}
        Some(actual) => blockers.push(format!("{label} expected {expected}, got {actual}")),
        None => blockers.push(format!("{label} is missing at {pointer}")),
    }
}

fn expect_u64(value: &Value, pointer: &str, expected: u64, label: &str, blockers: &mut Vec<String>) {
    match optional_u64(value, pointer) {
        Some(actual) if actual == expected => {}
        Some(actual) => blockers.push(format!("{label} expected {expected}, got {actual}")),
        None => blockers.push(format!("{label} is missing at {pointer}")),
    }
}

fn expect_null(value: &Value, expected: ExpectedNull<'_>, blockers: &mut Vec<String>) {
    debug_assert!(expected.pointer.starts_with('/'));
    debug_assert!(!expected.label.is_empty());
    match value.pointer(expected.pointer) {
        Some(value) if value.is_null() => {}
        Some(value) => blockers.push(format!("{} expected null, got {value}", expected.label)),
        None => blockers.push(format!("{} is missing at {}", expected.label, expected.pointer)),
    }
}

pub(crate) fn cmd_cargo_free_self_build(options: CargoFreeSelfBuildOptions<'_>) -> Result<(), RunError> {
    let paths = prepare_paths(options.root, options.out_dir)?;
    debug_assert!(paths.root.is_absolute());
    debug_assert!(!paths.out_dir.starts_with(&paths.root));
    prepare_output_dir(&paths)?;
    write_cargo_shim(&paths.explicit_cargo_shim, &paths.marker_path)?;
    write_cargo_shim(&paths.path_cargo_shim, &paths.marker_path)?;
    let loaded_toolchain_closure = load_source_built_toolchain_closure(options.toolchain_closure)?;
    let loaded_rust_provider =
        load_rust_source_provider(options.rust_source_provider, loaded_toolchain_closure.manifest.as_ref())?;
    let initial_toolchain_status =
        effective_source_built_toolchain_closure(&loaded_toolchain_closure, &loaded_rust_provider);
    write_non_claims(&paths.out_dir, &initial_toolchain_status)?;
    let requested_rustc = selected_cargo_free_rustc(&loaded_rust_provider, options.rustc);
    let execution_toolchain = prepare_execution_toolchain(
        &paths.guard_path_dir,
        requested_rustc,
        &loaded_toolchain_closure,
        &loaded_rust_provider,
    )?;
    let rust_action_enabled = prepare_rust_child_action_authority(
        "mantle-self-build",
        options.rust_action_resources.as_ref(),
        &execution_toolchain.rustc,
        &loaded_toolchain_closure,
        loaded_rust_provider.source_built_shell.as_ref(),
        &loaded_rust_provider.source_built_host_tools,
        &loaded_rust_provider.source_built_native_artifacts,
        loaded_rust_provider.source_built_action_trust.as_ref(),
        &paths.guard_path_dir,
        &paths.rust_action_authority_path,
    )?;

    let mut child = run_rust_plan_child(RustPlanChildRequest {
        paths: &paths,
        rustc: &execution_toolchain.rustc,
        targets: options.targets,
        path_env: &execution_toolchain.path_env,
        c_compiler_route: execution_toolchain.c_compiler_route.as_ref(),
        policy_digest_blake3: initial_toolchain_status.policy_digest_blake3.as_deref(),
        rust_action_authority_path: rust_action_enabled.then_some(paths.rust_action_authority_path.as_path()),
        rust_action_evidence_dir: rust_action_enabled.then_some(paths.rust_action_evidence_dir.as_path()),
    })?;
    let produced = if child.blocker.is_none() {
        materialize_or_block(&paths, &mut child)?
    } else {
        None
    };
    if child.blocker.is_some() && produced.is_none() {
        write_blocked_smoke_outputs(&paths, child.blocker.as_deref())?;
    }
    let summary = summarize(SelfBuildSummaryContext {
        paths: &paths,
        child: &child,
        produced: produced.as_ref(),
        toolchain_closure: &execution_toolchain.status,
        rust_source_provider: &loaded_rust_provider.status,
        hermeticity_mode: options.hermeticity_mode,
    });
    write_summary(&paths.meta_path, &summary)?;
    print_summary(&summary, options.json)?;

    if let Some(blocker) = child.blocker {
        return Err(RunError::Build(format!("Cargo-free self-build blocked: {blocker}")));
    }
    Ok(())
}

pub(crate) fn cmd_cargo_free_fixed_point_self_build(options: CargoFreeSelfBuildOptions<'_>) -> Result<(), RunError> {
    let root = canonicalize_root(options.root)?;
    let bundle_dir = absolutize(&root, options.out_dir);
    ensure_outside_root(&bundle_dir, &root)?;
    debug_assert!(root.is_absolute());
    debug_assert!(!bundle_dir.starts_with(&root));
    prepare_fixed_point_output_dir(&bundle_dir)?;
    let loaded_toolchain_closure = load_source_built_toolchain_closure(options.toolchain_closure)?;
    let loaded_rust_provider =
        load_rust_source_provider(options.rust_source_provider, loaded_toolchain_closure.manifest.as_ref())?;
    let requested_rustc = selected_cargo_free_rustc(&loaded_rust_provider, options.rustc);
    let compatibility_rustc = prepare_rustc_for_compatibility(requested_rustc, &loaded_toolchain_closure)?;
    let execution_shell = selected_execution_shell(&loaded_rust_provider);
    let runtime = bound_rustc_runtime(&loaded_rust_provider.source_built_native_artifacts)?;
    let compatibility = prepare_rustc_compatibility_with_shell(
        &bundle_dir,
        &compatibility_rustc,
        &loaded_toolchain_closure,
        execution_shell,
        runtime.as_ref(),
    )?;
    let plan = plan_fixed_point_paths(&root, &bundle_dir, &compatibility.summary.stage_rustc, options.targets)?;
    let toolchain_status = enforce_fixed_point_toolchain(
        &compatibility.summary.requested_rustc,
        &compatibility.summary.stage_rustc,
        &loaded_toolchain_closure,
        &loaded_rust_provider,
    )?;
    write_fixed_point_non_claims(&plan.bundle_dir, &toolchain_status)?;
    write_fixed_point_preflight(&plan, &compatibility.summary, &toolchain_status, &loaded_rust_provider.status)?;
    let finish_context = FixedPointFinishContext {
        summary: FixedPointSummaryContext {
            plan: &plan,
            compatibility: &compatibility.summary,
            toolchain_closure: &toolchain_status,
            rust_source_provider: &loaded_rust_provider.status,
            hermeticity_mode: options.hermeticity_mode,
        },
        json_mode: options.json,
    };
    let host_mantle = env::current_exe().map_err(|err| internal(format!("resolve current executable: {err}")))?;
    let stage_policy_digest = toolchain_status.policy_digest_blake3.as_deref();
    let stage1 = execute_fixed_point_stage(
        &plan.stages[FIXED_POINT_STAGE1_INDEX],
        &host_mantle,
        &compatibility.summary.stage_rustc,
        &loaded_toolchain_closure,
        loaded_rust_provider.source_built_shell.as_ref(),
        &loaded_rust_provider.source_built_host_tools,
        &loaded_rust_provider.source_built_native_artifacts,
        loaded_rust_provider.source_built_action_trust.as_ref(),
        options.rust_action_resources.as_ref(),
        stage_policy_digest,
    )?;
    if !stage1.success {
        return finish_fixed_point(&finish_context, stage1, None, BLOCKED_STATUS);
    }
    let Some(stage1_binary) = stage1.binary.as_deref() else {
        let stage1 = blocked_fixed_point_stage(stage1, "stage1 succeeded without produced binary path".to_string());
        return finish_fixed_point(&finish_context, stage1, None, BLOCKED_STATUS);
    };
    let stage2 = execute_fixed_point_stage(
        &plan.stages[FIXED_POINT_STAGE2_INDEX],
        stage1_binary,
        &compatibility.summary.stage_rustc,
        &loaded_toolchain_closure,
        loaded_rust_provider.source_built_shell.as_ref(),
        &loaded_rust_provider.source_built_host_tools,
        &loaded_rust_provider.source_built_native_artifacts,
        loaded_rust_provider.source_built_action_trust.as_ref(),
        options.rust_action_resources.as_ref(),
        stage_policy_digest,
    )?;
    if !stage2.success {
        return finish_fixed_point(&finish_context, stage1, Some(stage2), BLOCKED_STATUS);
    }
    let status = fixed_point_status(&stage1, &stage2)?;
    finish_fixed_point(&finish_context, stage1, Some(stage2), status)
}

fn prepare_paths(root: &Path, out_dir: &Path) -> Result<BuildPaths, RunError> {
    let root = canonicalize_root(root)?;
    let out_dir = absolutize(&root, out_dir);
    ensure_outside_root(&out_dir, &root)?;
    let execution_dir = out_dir.join(EXECUTION_DIR);
    let rust_action_evidence_dir = out_dir.join(RUST_CHILD_ACTION_EVIDENCE_DIR);
    let rust_action_authority_path = rust_action_evidence_dir.join(RUST_CHILD_ACTION_AUTHORITY_FILE);
    debug_assert!(root.is_absolute());
    debug_assert_eq!(execution_dir.parent(), Some(out_dir.as_path()));
    Ok(BuildPaths {
        root,
        receipt_path: out_dir.join(RECEIPT_FILE),
        stderr_path: out_dir.join(STDERR_FILE),
        status_path: out_dir.join(STATUS_FILE),
        marker_path: out_dir.join(CARGO_MARKER_FILE),
        explicit_cargo_shim: out_dir.join(CARGO_SHIM_FILE),
        path_cargo_shim: out_dir.join(CARGO_SHIM_DIR).join(CARGO_SHIM_NAME),
        guard_path_dir: out_dir.join(CARGO_SHIM_DIR),
        binary_path: out_dir.join(PRODUCED_MANTLE_FILE),
        meta_path: out_dir.join(META_FILE),
        out_dir,
        execution_dir,
        rust_action_evidence_dir,
        rust_action_authority_path,
    })
}

fn canonicalize_root(root: &Path) -> Result<PathBuf, RunError> {
    fs::canonicalize(root).map_err(|err| internal(format!("canonicalize root {}: {err}", root.display())))
}

pub(crate) fn plan_fixed_point_paths(
    root: &Path,
    out_dir: &Path,
    rustc: &Path,
    targets: &[String],
) -> Result<FixedPointPlan, RunError> {
    if !root.is_absolute() {
        return Err(RunError::Build(format!(
            "fixed-point planner requires an absolute source root, got {}",
            root.display()
        )));
    }
    if rustc.as_os_str().is_empty() {
        return Err(RunError::Build("fixed-point planner requires a non-empty rustc path".to_string()));
    }
    let bundle_dir = absolutize(root, out_dir);
    ensure_outside_root(&bundle_dir, root)?;
    let shared_execution_dir = bundle_dir.join(EXECUTION_DIR);
    let stage1_binary_path = bundle_dir.join(STAGE1_DIR).join(PRODUCED_MANTLE_FILE);
    let stages = [
        fixed_point_stage_plan(FixedPointStagePlanRequest {
            name: STAGE1_DIR,
            root,
            bundle_dir: &bundle_dir,
            execution_dir: &shared_execution_dir,
            rustc,
            targets,
            mantle_binary: FixedPointMantleBinary::Host,
        }),
        fixed_point_stage_plan(FixedPointStagePlanRequest {
            name: STAGE2_DIR,
            root,
            bundle_dir: &bundle_dir,
            execution_dir: &shared_execution_dir,
            rustc,
            targets,
            mantle_binary: FixedPointMantleBinary::StageOutput {
                stage_name: STAGE1_DIR,
                path: stage1_binary_path,
            },
        }),
    ];
    debug_assert_eq!(stages.len(), FIXED_POINT_STAGE_COUNT);
    debug_assert_eq!(stages[FIXED_POINT_STAGE1_INDEX].name, STAGE1_DIR);
    debug_assert_eq!(stages[FIXED_POINT_STAGE2_INDEX].name, STAGE2_DIR);
    Ok(FixedPointPlan {
        schema: FIXED_POINT_SCHEMA,
        root: root.to_path_buf(),
        bundle_dir: bundle_dir.clone(),
        shared_execution_dir,
        preflight_path: bundle_dir.join(PRE_FLIGHT_FILE),
        meta_path: bundle_dir.join(META_FILE),
        non_claims_path: bundle_dir.join(NON_CLAIMS_FILE),
        stages,
    })
}

fn fixed_point_stage_plan(request: FixedPointStagePlanRequest<'_>) -> FixedPointStagePlan {
    debug_assert!(!request.name.is_empty());
    debug_assert!(request.root.is_absolute());
    debug_assert!(request.bundle_dir.is_absolute());
    let stage_dir = request.bundle_dir.join(request.name);
    let explicit_cargo_shim = stage_dir.join(CARGO_SHIM_FILE);
    let guard_path_dir = stage_dir.join(CARGO_SHIM_DIR);
    let path_cargo_shim = guard_path_dir.join(CARGO_SHIM_NAME);
    let command = FixedPointStageCommandPlan {
        mantle_binary: request.mantle_binary,
        args: rust_plan_args(request.root, &explicit_cargo_shim, request.rustc, request.targets, request.execution_dir),
        current_dir: request.root.to_path_buf(),
        cargo_env_value: path_cargo_shim.clone(),
        path_guard_dir: guard_path_dir.clone(),
    };
    FixedPointStagePlan {
        name: request.name,
        stage_dir: stage_dir.clone(),
        execution_dir: request.execution_dir.to_path_buf(),
        receipt_path: stage_dir.join(RECEIPT_FILE),
        stderr_path: stage_dir.join(STDERR_FILE),
        status_path: stage_dir.join(STATUS_FILE),
        cargo_marker_path: stage_dir.join(CARGO_MARKER_FILE),
        explicit_cargo_shim,
        path_cargo_shim,
        guard_path_dir,
        binary_path: stage_dir.join(PRODUCED_MANTLE_FILE),
        smoke_stdout_path: stage_dir.join(SMOKE_STDOUT_FILE),
        smoke_stderr_path: stage_dir.join(SMOKE_STDERR_FILE),
        command,
    }
}

fn rust_plan_args(
    root: &Path,
    cargo_shim: &Path,
    rustc: &Path,
    targets: &[String],
    execution_dir: &Path,
) -> Vec<OsString> {
    let mut args = vec![
        OsString::from(JSON_FLAG),
        OsString::from(RUST_PLAN_COMMAND),
        OsString::from(ROOT_FLAG),
        root.as_os_str().to_os_string(),
        OsString::from(CARGO_FLAG),
        cargo_shim.as_os_str().to_os_string(),
        OsString::from(RUSTC_FLAG),
        rustc.as_os_str().to_os_string(),
    ];
    for target in targets {
        args.push(OsString::from("--target"));
        args.push(OsString::from(target));
    }
    args.extend([
        OsString::from(NO_CARGO_ORACLE_FLAG),
        OsString::from(DETERMINISTIC_RELEASE_PATHS_FLAG),
        OsString::from(EXECUTE_TOPOLOGY_FLAG),
        OsString::from(EXECUTION_OUTPUT_ROOT_FLAG),
        execution_dir.as_os_str().to_os_string(),
    ]);
    debug_assert_eq!(args.first(), Some(&OsString::from(JSON_FLAG)));
    debug_assert_eq!(args.last(), Some(&execution_dir.as_os_str().to_os_string()));
    args
}

fn prepare_fixed_point_output_dir(bundle_dir: &Path) -> Result<(), RunError> {
    fs::create_dir_all(bundle_dir).map_err(|err| internal(format!("create {}: {err}", bundle_dir.display())))?;
    let owned_paths = [
        bundle_dir.join(STAGE1_DIR),
        bundle_dir.join(STAGE2_DIR),
        bundle_dir.join(EXECUTION_DIR),
        bundle_dir.join(TOOLCHAIN_DIR),
        bundle_dir.join(PRE_FLIGHT_FILE),
        bundle_dir.join(META_FILE),
        bundle_dir.join(NON_CLAIMS_FILE),
    ];
    for path in &owned_paths {
        remove_owned_path(path)?;
    }
    fs::create_dir_all(bundle_dir).map_err(|err| internal(format!("create {}: {err}", bundle_dir.display())))
}

fn prepare_rustc_compatibility(
    bundle_dir: &Path,
    requested: &Path,
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<RustcCompatibility, RunError> {
    prepare_rustc_compatibility_with_shell(bundle_dir, requested, toolchain_closure, Path::new("/bin/sh"), None)
}

fn prepare_rustc_compatibility_with_shell(
    bundle_dir: &Path,
    requested: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    execution_shell: &Path,
    runtime: Option<&BoundRustcRuntime>,
) -> Result<RustcCompatibility, RunError> {
    let requested_rustc = resolve_executable(requested, "rustc")?;
    let toolchain_dir = bundle_dir.join(TOOLCHAIN_DIR);
    debug_assert_eq!(toolchain_dir.parent(), Some(bundle_dir));
    debug_assert_ne!(toolchain_dir, requested_rustc);
    fs::create_dir_all(&toolchain_dir)
        .map_err(|err| internal(format!("create toolchain dir {}: {err}", toolchain_dir.display())))?;
    let probe_path_env =
        prepare_rustc_compatibility_path_env_with_shell(&toolchain_dir, toolchain_closure, execution_shell)?;
    let receipt_bound_linker = toolchain_dir.join(TOOLCHAIN_COMPATIBILITY_PATH_DIR).join(RUSTC_BOUND_LINKER_ALIAS);
    let runtime_wrapper = runtime
        .map(|runtime| {
            let wrapper = toolchain_dir.join(RUSTC_RUNTIME_WRAPPER_FILE);
            write_bound_rustc_runtime_wrapper(
                &wrapper,
                &requested_rustc,
                runtime,
                execution_shell,
                &receipt_bound_linker,
            )?;
            Ok::<PathBuf, RunError>(wrapper)
        })
        .transpose()?;
    let probe_rustc = runtime_wrapper.as_deref().unwrap_or(&requested_rustc);
    if rustc_accepts_link_self_contained_no(probe_rustc, &toolchain_dir, probe_path_env.as_deref())? {
        let wrapper_blake3 = runtime_wrapper.as_deref().map(blake3_file).transpose()?;
        let stage_rustc = probe_rustc.to_path_buf();
        let normalization = if runtime_wrapper.is_some() {
            NORMALIZATION_RECEIPT_BOUND_RUNTIME
        } else {
            NORMALIZATION_NONE
        };
        let summary = RustcCompatibilitySummary {
            requested_rustc,
            stage_rustc,
            normalization,
            wrapper: runtime_wrapper,
            wrapper_blake3,
        };
        write_rustc_compatibility(&toolchain_dir.join(COMPATIBILITY_FILE), &summary)?;
        return Ok(RustcCompatibility { summary });
    }
    if toolchain_closure.manifest.is_some() {
        return Err(RunError::Build(
            "source-built toolchain closure blocked: rustc compatibility probe failed under receipt-bound toolchain PATH"
                .to_string(),
        ));
    }
    let wrapper = toolchain_dir.join(RUSTC_WRAPPER_FILE);
    write_rustc_wrapper(&wrapper, &requested_rustc)?;
    let wrapper_blake3 = blake3_file(&wrapper)?;
    let summary = RustcCompatibilitySummary {
        requested_rustc,
        stage_rustc: wrapper.clone(),
        normalization: NORMALIZATION_STRIP_LINK_SELF_CONTAINED,
        wrapper: Some(wrapper),
        wrapper_blake3: Some(wrapper_blake3),
    };
    write_rustc_compatibility(&toolchain_dir.join(COMPATIBILITY_FILE), &summary)?;
    Ok(RustcCompatibility { summary })
}

fn prepare_rustc_compatibility_path_env_with_shell(
    toolchain_dir: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    execution_shell: &Path,
) -> Result<Option<OsString>, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(None);
    };
    let path_dir = toolchain_dir.join(TOOLCHAIN_COMPATIBILITY_PATH_DIR);
    remove_owned_path(&path_dir)?;
    fs::create_dir_all(&path_dir)
        .map_err(|err| internal(format!("create receipt-bound rustc probe PATH dir {}: {err}", path_dir.display())))?;
    write_toolchain_path_aliases_with_shell(&path_dir, manifest, execution_shell)?;
    let path_env = env::join_paths([path_dir])
        .map_err(|err| internal(format!("construct receipt-bound rustc probe PATH: {err}")))?;
    Ok(Some(path_env))
}

fn resolve_executable(path: &Path, name: &str) -> Result<PathBuf, RunError> {
    debug_assert!(!name.is_empty());
    if path.components().count() == 1 && !path.is_absolute() {
        return resolve_executable_on_path(path, name);
    }
    let resolved =
        fs::canonicalize(path).map_err(|err| internal(format!("canonicalize {name} {}: {err}", path.display())))?;
    require_executable(&resolved)?;
    Ok(resolved)
}

fn resolve_executable_on_path(path: &Path, name: &str) -> Result<PathBuf, RunError> {
    let path_var = env::var_os("PATH").ok_or_else(|| internal(format!("PATH is unset; cannot find {name}")))?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(path);
        if candidate.is_file() {
            let resolved = fs::canonicalize(&candidate)
                .map_err(|err| internal(format!("canonicalize {}: {err}", candidate.display())))?;
            require_executable(&resolved)?;
            return Ok(resolved);
        }
    }
    Err(internal(format!("required tool not found on PATH: {name}")))
}

fn rustc_accepts_link_self_contained_no(
    rustc: &Path,
    toolchain_dir: &Path,
    path_env: Option<&OsStr>,
) -> Result<bool, RunError> {
    let probe_dir = toolchain_dir.join(RUSTC_PROBE_DIR);
    let probe_source = probe_dir.join(RUSTC_PROBE_SOURCE_FILE);
    debug_assert_eq!(probe_dir.parent(), Some(toolchain_dir));
    debug_assert_eq!(probe_source.parent(), Some(probe_dir.as_path()));
    remove_owned_path(&probe_dir)?;
    fs::create_dir_all(&probe_dir)
        .map_err(|err| internal(format!("create rustc compatibility probe dir {}: {err}", probe_dir.display())))?;
    fs::write(&probe_source, RUSTC_PROBE_SOURCE)
        .map_err(|err| internal(format!("write rustc compatibility probe {}: {err}", probe_source.display())))?;
    let mut command = Command::new(rustc);
    command
        .arg("--crate-type")
        .arg("bin")
        .arg("-C")
        .arg(LINK_SELF_CONTAINED_PROBE_ARG)
        .arg(&probe_source)
        .arg("--out-dir")
        .arg(&probe_dir);
    if let Some(path_env) = path_env {
        command.env("PATH", path_env);
    }
    let is_accepted = command.output().is_ok_and(|output| output.status.success());
    remove_owned_path(&probe_dir)?;
    Ok(is_accepted)
}

fn write_bound_rustc_runtime_wrapper(
    wrapper: &Path,
    real_rustc: &Path,
    runtime: &BoundRustcRuntime,
    execution_shell: &Path,
    receipt_bound_linker: &Path,
) -> Result<(), RunError> {
    require_executable(real_rustc)?;
    require_executable(&runtime.loader)?;
    require_executable(execution_shell)?;
    require_executable(receipt_bound_linker)?;
    let rust_root = real_rustc
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| RunError::Build("receipt-bound rustc has no provider root".to_string()))?;
    if !real_rustc.ends_with("bin/rustc") {
        return Err(RunError::Build(format!(
            "receipt-bound rustc path has unexpected shape: {}",
            real_rustc.display()
        )));
    }
    let dynamic_rustc = rust_root.join("bin").join(RUSTC_DYNAMIC_BINARY_FILE);
    require_executable(&dynamic_rustc)?;
    let rust_lib = rust_root.join("lib");
    let runtime_lib = rust_lib.join("mantle-runtime");
    let rust_stdlib = rust_lib.join("rustlib").join(RUSTC_RUNTIME_HOST_TRIPLE).join("lib");
    for directory in [&runtime_lib, &runtime.library_dir, &rust_lib, &rust_stdlib] {
        if !directory.is_dir() {
            return Err(RunError::Build(format!(
                "receipt-bound rustc runtime directory is unavailable: {}",
                directory.display()
            )));
        }
    }
    let library_path = env::join_paths([&runtime_lib, &runtime.library_dir, &rust_lib, &rust_stdlib])
        .map_err(|error| internal(format!("construct receipt-bound rustc library path: {error}")))?;
    let execution_shell = path_to_string(execution_shell)?;
    let linker_arg = format!("linker={}", receipt_bound_linker.display());
    let script = format!(
        "#!{execution_shell}\nset -eu\nhas_sysroot=false\nfor arg in \"$@\"; do\n  if [ \"$arg\" = \"--sysroot\" ]; then has_sysroot=true; break; fi\n  case \"$arg\" in --sysroot=*) has_sysroot=true; break ;; esac\ndone\nif [ \"$has_sysroot\" = false ]; then set -- --sysroot {} \"$@\"; fi\nset -- \"$@\" -C {}\n{DYNAMIC_LIBRARY_PATH_ENV}={}\nexport {DYNAMIC_LIBRARY_PATH_ENV}\nexec {} {} \"$@\"\n",
        shell_quote(rust_root),
        shell_quote(Path::new(&linker_arg)),
        shell_quote(Path::new(&library_path)),
        shell_quote(&runtime.loader),
        shell_quote(&dynamic_rustc)
    );
    write_text(wrapper, &script)?;
    set_executable(wrapper)?;
    assert!(wrapper.is_file());
    Ok(())
}

fn write_rustc_wrapper(wrapper: &Path, real_rustc: &Path) -> Result<(), RunError> {
    let script = rustc_wrapper_script(real_rustc);
    write_text(wrapper, &script)?;
    set_executable(wrapper)
}

fn rustc_wrapper_script(real_rustc: &Path) -> String {
    format!(
        "#!/bin/sh\nset -eu\nexport {RUSTC_BOOTSTRAP_ENV}=1\nexport {REAL_RUSTC_ENV}={}\nremaining=$#\nwhile [ \"$remaining\" -gt 0 ]; do\n  arg=$1\n  shift\n  remaining=$((remaining - 1))\n  if [ \"$arg\" = \"-C\" ] && [ \"$remaining\" -gt 0 ] && [ \"$1\" = \"{LINK_SELF_CONTAINED_PROBE_ARG}\" ]; then\n    shift\n    remaining=$((remaining - 1))\n    continue\n  fi\n  if [ \"$arg\" = \"{LINK_SELF_CONTAINED_JOINED_ARG}\" ]; then\n    continue\n  fi\n  set -- \"$@\" \"$arg\"\ndone\nexec \"${REAL_RUSTC_ENV}\" \"$@\"\n",
        shell_quote(real_rustc)
    )
}

fn write_rustc_compatibility(path: &Path, summary: &RustcCompatibilitySummary) -> Result<(), RunError> {
    let value = json!(summary);
    let bytes = serde_json::to_vec_pretty(&value).map_err(|err| internal(format!("serialize compatibility: {err}")))?;
    write_bytes(path, &bytes)
}

fn prepare_output_dir(paths: &BuildPaths) -> Result<(), RunError> {
    debug_assert_eq!(paths.execution_dir.parent(), Some(paths.out_dir.as_path()));
    debug_assert_eq!(paths.guard_path_dir.parent(), Some(paths.out_dir.as_path()));
    fs::create_dir_all(&paths.out_dir)
        .map_err(|err| internal(format!("create output dir {}: {err}", paths.out_dir.display())))?;
    remove_owned_path(&paths.execution_dir)?;
    remove_owned_path(&paths.guard_path_dir)?;
    remove_owned_path(&paths.rust_action_evidence_dir)?;
    let owned_files = vec![
        paths.receipt_path.clone(),
        paths.stderr_path.clone(),
        paths.status_path.clone(),
        paths.marker_path.clone(),
        paths.explicit_cargo_shim.clone(),
        paths.binary_path.clone(),
        paths.meta_path.clone(),
        paths.out_dir.join(SMOKE_STDOUT_FILE),
        paths.out_dir.join(SMOKE_STDERR_FILE),
    ];
    for path in &owned_files {
        remove_owned_path(path)?;
    }
    fs::create_dir_all(&paths.execution_dir)
        .map_err(|err| internal(format!("create execution dir {}: {err}", paths.execution_dir.display())))
}

fn remove_owned_path(path: &Path) -> Result<(), RunError> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|err| internal(format!("remove dir {}: {err}", path.display())))
    } else {
        fs::remove_file(path).map_err(|err| internal(format!("remove file {}: {err}", path.display())))
    }
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    debug_assert!(root.is_absolute());
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn ensure_outside_root(out_dir: &Path, root: &Path) -> Result<(), RunError> {
    debug_assert!(root.is_absolute());
    debug_assert!(out_dir.is_absolute());
    if out_dir.starts_with(root) {
        return Err(RunError::Build(format!(
            "--out {} is inside source root {}; choose /tmp or another outside path so build evidence does not change native source digests",
            out_dir.display(),
            root.display()
        )));
    }
    Ok(())
}

fn run_rust_plan_child(request: RustPlanChildRequest<'_>) -> Result<ChildRun, RunError> {
    debug_assert!(request.paths.root.is_absolute());
    debug_assert_ne!(request.paths.explicit_cargo_shim, request.paths.path_cargo_shim);
    let current_exe = env::current_exe().map_err(|err| internal(format!("resolve current executable: {err}")))?;
    let mut command = Command::new(&current_exe);
    command
        .arg(JSON_FLAG)
        .arg(RUST_PLAN_COMMAND)
        .arg(ROOT_FLAG)
        .arg(&request.paths.root)
        .arg(CARGO_FLAG)
        .arg(&request.paths.explicit_cargo_shim)
        .arg(RUSTC_FLAG)
        .arg(request.rustc);
    for target in request.targets {
        command.arg("--target").arg(target);
    }
    let route_json = request
        .c_compiler_route
        .map(serde_json::to_string)
        .transpose()
        .map_err(|err| internal(format!("serialize receipt-bound C compiler route: {err}")))?;
    if let Some(route_json) = route_json {
        command
            .arg("--source-built-c-compiler-route-json")
            .arg(&route_json)
            .env(crate::source_toolchain_closure::SOURCE_BUILT_C_COMPILER_ROUTE_ENV, route_json);
    }
    match (request.rust_action_authority_path, request.rust_action_evidence_dir) {
        (Some(authority_path), Some(evidence_dir)) => {
            command
                .arg(RUST_CHILD_ACTION_AUTHORITY_FLAG)
                .arg(authority_path)
                .arg(RUST_CHILD_ACTION_EVIDENCE_DIR_FLAG)
                .arg(evidence_dir);
        }
        (None, None) => {}
        _ => {
            return Err(internal(
                "Rust child-action authority path and evidence directory must be supplied together".to_string(),
            ));
        }
    }
    if let Some(policy_digest_blake3) = request.policy_digest_blake3 {
        command.env(crate::rust_plan::RUST_TOPOLOGY_TOOLCHAIN_POLICY_DIGEST_ENV, policy_digest_blake3);
    }
    let output = command
        .arg(NO_CARGO_ORACLE_FLAG)
        .arg(EXECUTE_TOPOLOGY_FLAG)
        .arg(EXECUTION_OUTPUT_ROOT_FLAG)
        .arg(&request.paths.execution_dir)
        .current_dir(&request.paths.root)
        .env("CARGO", &request.paths.path_cargo_shim)
        .env(RUSTC_BOOTSTRAP_ENV, "1")
        .env("PATH", request.path_env)
        .output()
        .map_err(|err| internal(format!("launch {} rust-plan: {err}", current_exe.display())))?;

    write_bytes(&request.paths.receipt_path, &output.stdout)?;
    write_bytes(&request.paths.stderr_path, &output.stderr)?;
    write_text(&request.paths.status_path, &status_text(output.status.code()))?;
    let receipt = parse_receipt(&request.paths.receipt_path, output.status.success())?;
    let execution_status = receipt_execution_status(receipt.as_ref());
    let is_cargo_marker_absent = !request.paths.marker_path.exists();
    let (blocker, blocker_diagnostic) = child_blocker_with_diagnostic(
        output.status.code(),
        &execution_status,
        is_cargo_marker_absent,
        receipt.as_ref(),
    );
    Ok(ChildRun {
        status_code: output.status.code(),
        receipt,
        execution_status,
        cargo_marker_absent: is_cargo_marker_absent,
        blocker,
        blocker_diagnostic,
    })
}

#[cfg(test)]
fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
    child_blocker_with_diagnostic(status_code, execution_status, cargo_marker_absent, None).0
}

fn child_blocker_with_diagnostic(
    status_code: Option<i32>,
    execution_status: &str,
    cargo_marker_absent: bool,
    receipt: Option<&Value>,
) -> (Option<String>, Option<BlockedTopologyDiagnostic>) {
    if !cargo_marker_absent {
        return (Some("cargo guard was invoked".to_string()), None);
    }
    if status_code != Some(SUCCESS_EXIT_CODE) {
        return (Some(format!("rust-plan exited with {}", status_text(status_code).trim_end())), None);
    }
    if execution_status != SUCCESS_STATUS {
        let diagnostic = classify_blocked_topology_receipt(receipt, execution_status);
        return (Some(blocked_topology_summary(&diagnostic)), Some(diagnostic));
    }
    (None, None)
}

fn classify_blocked_topology_receipt(receipt: Option<&Value>, execution_status: &str) -> BlockedTopologyDiagnostic {
    debug_assert!(execution_status != SUCCESS_STATUS);
    let Some(receipt) = receipt else {
        return malformed_blocked_topology_diagnostic(MalformedBlockedTopologyRequest {
            execution_status,
            classification: "missing-blocked-receipt",
            diagnostic: "blocked topology execution did not produce a parseable receipt",
        });
    };
    let Some(units) = receipt.pointer("/topology_execution/unit_executions").and_then(Value::as_array) else {
        return topology_level_blocked_diagnostic(receipt, execution_status);
    };
    let blocked_units = units
        .iter()
        .enumerate()
        .filter(|(_, unit)| optional_str(unit, "/execution_status") != Some(SUCCESS_STATUS))
        .collect::<Vec<_>>();
    match blocked_units.as_slice() {
        [] => topology_level_blocked_diagnostic(receipt, execution_status),
        [(unit_index, unit)] => blocked_unit_diagnostic(execution_status, units, *unit_index, unit),
        _ => malformed_blocked_topology_diagnostic(MalformedBlockedTopologyRequest {
            execution_status,
            classification: "ambiguous-blocked-units",
            diagnostic: &format!(
                "blocked topology receipt contains {} non-success unit executions",
                blocked_units.len()
            ),
        }),
    }
}

fn blocked_unit_diagnostic(
    execution_status: &str,
    units: &[Value],
    unit_index: usize,
    unit: &Value,
) -> BlockedTopologyDiagnostic {
    debug_assert!(unit_index < units.len());
    debug_assert_ne!(execution_status, SUCCESS_STATUS);
    let required = [
        ("unit_id", optional_str(unit, "/unit_id")),
        ("package_id", optional_str(unit, "/package_id")),
        ("execution_kind", optional_str(unit, "/execution_kind")),
        ("selected_triple", optional_str(unit, "/selected_triple")),
        ("target_kind", optional_str(unit, "/target_kind")),
    ];
    if let Some((field, _)) = required.iter().find(|(_, value)| value.is_none()) {
        return malformed_blocked_topology_diagnostic(MalformedBlockedTopologyRequest {
            execution_status,
            classification: "malformed-blocked-unit",
            diagnostic: &format!("blocked unit at index {unit_index} is missing required field {field}"),
        });
    }
    let blocker_class = optional_str(unit, "/blocker/class").map(ToOwned::to_owned);
    let diagnostic = match blocker_class.as_deref() {
        Some(class) => format!("blocked unit classified with blocker class {class}"),
        None => format!("blocked unit at index {unit_index} is missing blocker class"),
    };
    BlockedTopologyDiagnostic {
        classification: if blocker_class.is_some() {
            "blocked-unit".to_string()
        } else {
            "malformed-blocked-unit".to_string()
        },
        topology_execution_status: execution_status.to_string(),
        root_blocked_unit: optional_str(unit, "/unit_id").map(ToOwned::to_owned),
        package_id: optional_str(unit, "/package_id").map(ToOwned::to_owned),
        execution_role: optional_str(unit, "/execution_kind").map(ToOwned::to_owned),
        selected_triple: optional_str(unit, "/selected_triple").map(ToOwned::to_owned),
        target_kind: optional_str(unit, "/target_kind").map(ToOwned::to_owned),
        predecessor_status: predecessor_status(units, unit_index),
        blocker_class,
        diagnostic,
    }
}

fn topology_level_blocked_diagnostic(receipt: &Value, execution_status: &str) -> BlockedTopologyDiagnostic {
    debug_assert_ne!(execution_status, SUCCESS_STATUS);
    debug_assert!("/topology_execution/blocker/class".starts_with('/'));
    let top_class = optional_str(receipt, "/topology_execution/blocker/class").map(ToOwned::to_owned);
    let nested = first_nested_planner_blocker(receipt);
    let blocker_class = nested
        .as_ref()
        .and_then(|(_, blocker)| optional_str(blocker, "/class"))
        .map(ToOwned::to_owned)
        .or_else(|| top_class.clone());
    let diagnostic = topology_level_diagnostic_message(top_class.as_deref(), nested.as_ref());
    BlockedTopologyDiagnostic {
        classification: if blocker_class.is_some() {
            "topology-level-blocker".to_string()
        } else {
            "malformed-blocked-receipt".to_string()
        },
        topology_execution_status: execution_status.to_string(),
        root_blocked_unit: nested
            .as_ref()
            .and_then(|(_, blocker)| optional_str(blocker, "/unit_id"))
            .map(ToOwned::to_owned),
        package_id: nested
            .as_ref()
            .and_then(|(_, blocker)| optional_str(blocker, "/package_id"))
            .map(ToOwned::to_owned),
        execution_role: nested
            .as_ref()
            .and_then(|(_, blocker)| optional_str(blocker, "/execution_kind"))
            .map(ToOwned::to_owned),
        selected_triple: nested
            .as_ref()
            .and_then(|(_, blocker)| optional_str(blocker, "/selected_triple"))
            .map(ToOwned::to_owned),
        target_kind: nested
            .as_ref()
            .and_then(|(_, blocker)| optional_str(blocker, "/target_kind"))
            .map(ToOwned::to_owned),
        predecessor_status: None,
        blocker_class,
        diagnostic,
    }
}

fn first_nested_planner_blocker(receipt: &Value) -> Option<(&'static str, &Value)> {
    const NESTED_BLOCKER_POINTERS: [&str; 6] = [
        "/rust_plan/native_registry_source_planning/blockers/0",
        "/rust_plan/native_git_source_planning/blockers/0",
        "/rust_plan/native_package_target_planning/blockers/0",
        "/rust_plan/native_unit_graph_planning/blockers/0",
        "/rust_plan/native_host_unit_graph_planning/blockers/0",
        "/rust_plan/unit_derivation_graph/blockers/0",
    ];
    NESTED_BLOCKER_POINTERS
        .iter()
        .find_map(|pointer| receipt.pointer(pointer).map(|blocker| (*pointer, blocker)))
}

fn topology_level_diagnostic_message(top_class: Option<&str>, nested: Option<&(&'static str, &Value)>) -> String {
    match (top_class, nested) {
        (Some(top_class), Some((pointer, blocker))) => {
            let nested_class = optional_str(blocker, "/class").unwrap_or("missing-class");
            let nested_message = optional_str(blocker, "/message").unwrap_or("missing-message");
            format!("topology-level blocker class {top_class}; nested {pointer} class {nested_class}: {nested_message}")
        }
        (Some(top_class), None) => format!("topology-level blocker class {top_class}"),
        (None, Some((pointer, blocker))) => {
            let nested_class = optional_str(blocker, "/class").unwrap_or("missing-class");
            let nested_message = optional_str(blocker, "/message").unwrap_or("missing-message");
            format!("topology-level blocker has nested {pointer} class {nested_class}: {nested_message}")
        }
        (None, None) => "blocked topology receipt has no non-success unit and no topology blocker class".to_string(),
    }
}

fn malformed_blocked_topology_diagnostic(request: MalformedBlockedTopologyRequest<'_>) -> BlockedTopologyDiagnostic {
    debug_assert_ne!(request.execution_status, SUCCESS_STATUS);
    debug_assert!(!request.classification.is_empty());
    BlockedTopologyDiagnostic {
        classification: request.classification.to_string(),
        topology_execution_status: request.execution_status.to_string(),
        root_blocked_unit: None,
        package_id: None,
        execution_role: None,
        selected_triple: None,
        target_kind: None,
        predecessor_status: None,
        blocker_class: None,
        diagnostic: request.diagnostic.to_string(),
    }
}

fn predecessor_status(units: &[Value], unit_index: usize) -> Option<String> {
    let predecessor_index = unit_index.checked_sub(1)?;
    units
        .get(predecessor_index)
        .and_then(|unit| optional_str(unit, "/execution_status"))
        .map(ToOwned::to_owned)
}

fn blocked_topology_summary(diagnostic: &BlockedTopologyDiagnostic) -> String {
    let mut parts = vec![
        format!("topology execution status was {}", diagnostic.topology_execution_status),
        format!("classification={}", diagnostic.classification),
        diagnostic.diagnostic.clone(),
    ];
    push_optional_summary_part(&mut parts, "unit", diagnostic.root_blocked_unit.as_deref());
    push_optional_summary_part(&mut parts, "package", diagnostic.package_id.as_deref());
    push_optional_summary_part(&mut parts, "role", diagnostic.execution_role.as_deref());
    push_optional_summary_part(&mut parts, "triple", diagnostic.selected_triple.as_deref());
    push_optional_summary_part(&mut parts, "target_kind", diagnostic.target_kind.as_deref());
    push_optional_summary_part(&mut parts, "predecessor_status", diagnostic.predecessor_status.as_deref());
    push_optional_summary_part(&mut parts, "blocker_class", diagnostic.blocker_class.as_deref());
    parts.join("; ")
}

fn push_optional_summary_part(parts: &mut Vec<String>, label: &str, value: Option<&str>) {
    if let Some(value) = value {
        parts.push(format!("{label}={value}"));
    }
}

fn execute_fixed_point_stage(
    stage: &FixedPointStagePlan,
    mantle_bin: &Path,
    rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    bound_shell: Option<&BoundRustExecutionShell>,
    source_built_host_tools: &[crate::full_source_rust_binding::FullSourceRustHostToolBinding],
    source_built_native_artifacts: &[crate::full_source_rust_binding::FullSourceNativeArtifactBinding],
    source_built_action_trust: Option<&crate::source_built_rust_provider_action::RustProviderActionEvidence>,
    rust_action_resources: Option<&crate::source_built_rust_action_plan::RustActionResourceLimits>,
    policy_digest_blake3: Option<&str>,
) -> Result<FixedPointStageRun, RunError> {
    debug_assert!(matches!(stage.name, STAGE1_DIR | STAGE2_DIR));
    debug_assert_eq!(stage.command.path_guard_dir, stage.guard_path_dir);
    let execution_shell = bound_shell.map_or_else(|| Path::new("/bin/sh"), |shell| shell.execution_path.as_path());
    debug_assert!(execution_shell.is_absolute());
    prepare_fixed_point_stage(stage)?;
    let path_env = execution_path_env_with_shell(&stage.guard_path_dir, toolchain_closure, execution_shell)?;
    write_bound_rust_host_tool_aliases(&stage.guard_path_dir, source_built_host_tools, execution_shell)?;
    let rust_action_evidence_dir = stage.stage_dir.join(RUST_CHILD_ACTION_EVIDENCE_DIR);
    let rust_action_authority_path = rust_action_evidence_dir.join(RUST_CHILD_ACTION_AUTHORITY_FILE);
    let rust_action_enabled = prepare_rust_child_action_authority(
        fixed_point_rust_action_stage_id(stage.name)?,
        rust_action_resources,
        rustc,
        toolchain_closure,
        bound_shell,
        source_built_host_tools,
        source_built_native_artifacts,
        source_built_action_trust,
        &stage.guard_path_dir,
        &rust_action_authority_path,
    )?;
    let c_compiler_route = fixed_point_c_compiler_route(toolchain_closure)?;
    let route_json = c_compiler_route
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|err| internal(format!("serialize fixed-point receipt-bound C compiler route: {err}")))?;
    let mut command = Command::new(mantle_bin);
    command
        .args(&stage.command.args)
        .current_dir(&stage.command.current_dir)
        .env("CARGO", &stage.command.cargo_env_value)
        .env(RUSTC_BOOTSTRAP_ENV, "1")
        .env("PATH", path_env);
    if let Some(route_json) = &route_json {
        command
            .arg("--source-built-c-compiler-route-json")
            .arg(route_json)
            .env(crate::source_toolchain_closure::SOURCE_BUILT_C_COMPILER_ROUTE_ENV, route_json);
    }
    if rust_action_enabled {
        command
            .arg(RUST_CHILD_ACTION_AUTHORITY_FLAG)
            .arg(&rust_action_authority_path)
            .arg(RUST_CHILD_ACTION_EVIDENCE_DIR_FLAG)
            .arg(&rust_action_evidence_dir);
    }
    if let Some(policy_digest_blake3) = policy_digest_blake3 {
        command.env(crate::rust_plan::RUST_TOPOLOGY_TOOLCHAIN_POLICY_DIGEST_ENV, policy_digest_blake3);
    }
    let output = command.output();
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            return blocked_fixed_point_launch(stage, mantle_bin, err, policy_digest_blake3, c_compiler_route);
        }
    };
    let mut blocker = None;
    record_file_write(&stage.receipt_path, &output.stdout, &mut blocker);
    record_file_write(&stage.stderr_path, &output.stderr, &mut blocker);
    record_file_write(&stage.status_path, status_text(output.status.code()).as_bytes(), &mut blocker);
    fixed_point_stage_from_output(stage, output.status.code(), blocker, policy_digest_blake3, c_compiler_route)
}

fn fixed_point_c_compiler_route(
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<Option<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(None);
    };
    Ok(Some(receipt_bound_c_compiler_route(manifest)?))
}

fn prepare_fixed_point_stage(stage: &FixedPointStagePlan) -> Result<(), RunError> {
    fs::create_dir_all(&stage.stage_dir)
        .map_err(|err| internal(format!("create stage dir {}: {err}", stage.stage_dir.display())))?;
    remove_owned_path(&stage.execution_dir)?;
    fs::create_dir_all(&stage.execution_dir)
        .map_err(|err| internal(format!("create execution dir {}: {err}", stage.execution_dir.display())))?;
    write_cargo_shim(&stage.explicit_cargo_shim, &stage.cargo_marker_path)?;
    write_cargo_shim(&stage.path_cargo_shim, &stage.cargo_marker_path)
}

fn blocked_fixed_point_launch(
    stage: &FixedPointStagePlan,
    mantle_bin: &Path,
    err: std::io::Error,
    policy_digest_blake3: Option<&str>,
    c_compiler_route: Option<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
) -> Result<FixedPointStageRun, RunError> {
    let blocker = format!("launch {} for {}: {err}", mantle_bin.display(), stage.name);
    write_text(&stage.stderr_path, &format!("{blocker}\n"))?;
    write_text(&stage.status_path, "launch-failed\n")?;
    write_blocked_fixed_point_smoke_outputs(stage, &blocker)?;
    Ok(blocked_fixed_point_stage_from_plan(BlockedFixedPointStageRequest {
        stage,
        execution_status: "launch-failed",
        status_code: None,
        blocker,
        policy_digest_blake3,
        c_compiler_route,
    }))
}

fn fixed_point_stage_from_output(
    stage: &FixedPointStagePlan,
    status_code: Option<i32>,
    mut blocker: Option<String>,
    policy_digest_blake3: Option<&str>,
    c_compiler_route: Option<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
) -> Result<FixedPointStageRun, RunError> {
    debug_assert!(matches!(stage.name, STAGE1_DIR | STAGE2_DIR));
    debug_assert_eq!(stage.receipt_path.parent(), Some(stage.stage_dir.as_path()));
    let mut receipt = parse_receipt(&stage.receipt_path, status_code == Some(SUCCESS_EXIT_CODE))?;
    annotate_fixed_point_stage_receipt(
        &stage.receipt_path,
        receipt.as_mut(),
        policy_digest_blake3,
        c_compiler_route.as_ref(),
    )?;
    let execution_status = receipt_execution_status(receipt.as_ref());
    let is_cargo_marker_absent = !stage.cargo_marker_path.exists();
    let mut blocker_diagnostic = None;
    if blocker.is_none() {
        let classified =
            child_blocker_with_diagnostic(status_code, &execution_status, is_cargo_marker_absent, receipt.as_ref());
        blocker = classified.0;
        blocker_diagnostic = classified.1;
    }
    let produced = if blocker.is_none() {
        materialize_fixed_point_stage_artifact(stage, receipt.as_ref())?
    } else {
        None
    };
    fixed_point_stage_with_artifact(FixedPointStageAssembly {
        stage,
        status_code,
        receipt: receipt.as_ref(),
        execution_status,
        is_cargo_marker_absent,
        blocker,
        blocker_diagnostic,
        produced,
        policy_digest_blake3,
        c_compiler_route,
    })
}

fn fixed_point_stage_with_artifact(mut request: FixedPointStageAssembly<'_>) -> Result<FixedPointStageRun, RunError> {
    debug_assert!(request.receipt.is_some() || request.produced.is_none());
    debug_assert!(request.produced.is_none() || request.blocker.is_none());
    if let Some(produced) = request.produced.as_ref()
        && produced.smoke_status_code != SUCCESS_EXIT_CODE
    {
        request.blocker = Some(format!("smoke check exited with {}", produced.smoke_status_code));
    }
    if request.blocker.is_some() && request.produced.is_none() {
        write_blocked_fixed_point_smoke_outputs(
            request.stage,
            request.blocker.as_deref().unwrap_or("blocked before binary"),
        )?;
    }
    Ok(FixedPointStageRun {
        name: request.stage.name,
        dir: request.stage.stage_dir.clone(),
        execution_dir: request.stage.execution_dir.clone(),
        receipt_path: request.stage.receipt_path.clone(),
        stderr_path: request.stage.stderr_path.clone(),
        status_path: request.stage.status_path.clone(),
        status_code: request.status_code,
        execution_status: request.execution_status,
        cargo_marker_absent: request.is_cargo_marker_absent,
        success: request.blocker.is_none(),
        unit_count: request.receipt.map(unit_count).unwrap_or_default(),
        failed_unit_count: request.receipt.map(failed_unit_count).unwrap_or_default(),
        binary: request.produced.as_ref().map(|value| value.binary.clone()),
        binary_blake3: request.produced.as_ref().map(|value| value.digest.clone()),
        smoke_status_code: request.produced.as_ref().map(|value| value.smoke_status_code),
        source_built_toolchain_closure_policy_digest_blake3: request.policy_digest_blake3.map(ToOwned::to_owned),
        selected_c_compiler: request.c_compiler_route,
        blocker: request.blocker,
        blocker_diagnostic: request.blocker_diagnostic,
    })
}

fn annotate_fixed_point_stage_receipt(
    receipt_path: &Path,
    receipt: Option<&mut Value>,
    policy_digest_blake3: Option<&str>,
    c_compiler_route: Option<&crate::source_toolchain_closure::ReceiptBoundCCompilerRoute>,
) -> Result<(), RunError> {
    debug_assert_eq!(receipt_path.file_name(), Some(OsStr::new(RECEIPT_FILE)));
    debug_assert_ne!(TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD, SELECTED_C_COMPILER_FIELD);
    if policy_digest_blake3.is_none() && c_compiler_route.is_none() {
        return Ok(());
    }
    let Some(receipt) = receipt else {
        return Ok(());
    };
    let Some(object) = receipt.as_object_mut() else {
        return Err(internal("fixed-point stage receipt is not a JSON object".to_string()));
    };
    if let Some(policy_digest_blake3) = policy_digest_blake3 {
        object
            .insert(TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD.to_string(), Value::String(policy_digest_blake3.to_string()));
    }
    if let Some(c_compiler_route) = c_compiler_route {
        let route_value = serde_json::to_value(c_compiler_route)
            .map_err(|err| internal(format!("serialize selected C compiler route: {err}")))?;
        object.insert(SELECTED_C_COMPILER_FIELD.to_string(), route_value);
    }
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|err| internal(format!("serialize annotated fixed-point stage receipt: {err}")))?;
    write_bytes(receipt_path, &bytes)
}

fn materialize_fixed_point_stage_artifact(
    stage: &FixedPointStagePlan,
    receipt: Option<&Value>,
) -> Result<Option<FixedPointStageArtifact>, RunError> {
    debug_assert_eq!(stage.binary_path.parent(), Some(stage.stage_dir.as_path()));
    debug_assert_ne!(stage.execution_dir, stage.stage_dir);
    let Some(receipt) = receipt else {
        return Ok(None);
    };
    let (unit_id, _) = mantle_unit_from_receipt(receipt)?;
    let source_binary = stage.execution_dir.join(safe_path_component(&unit_id)).join(MANTLE_TARGET_NAME);
    require_executable(&source_binary)?;
    fs::copy(&source_binary, &stage.binary_path).map_err(|err| {
        internal(format!(
            "copy produced binary {} to {}: {err}",
            source_binary.display(),
            stage.binary_path.display()
        ))
    })?;
    require_executable(&stage.binary_path)?;
    let digest = blake3_file(&stage.binary_path)?;
    let smoke_status_code = run_smoke_to_paths(&stage.binary_path, &stage.smoke_stdout_path, &stage.smoke_stderr_path)?;
    Ok(Some(FixedPointStageArtifact {
        binary: stage.binary_path.clone(),
        digest,
        smoke_status_code,
    }))
}

fn blocked_fixed_point_stage(mut stage: FixedPointStageRun, blocker: String) -> FixedPointStageRun {
    stage.success = false;
    stage.blocker = Some(blocker);
    stage.blocker_diagnostic = None;
    stage
}

fn blocked_fixed_point_stage_from_plan(request: BlockedFixedPointStageRequest<'_>) -> FixedPointStageRun {
    debug_assert!(!request.blocker.is_empty());
    debug_assert_ne!(request.execution_status, SUCCESS_STATUS);
    FixedPointStageRun {
        name: request.stage.name,
        dir: request.stage.stage_dir.clone(),
        execution_dir: request.stage.execution_dir.clone(),
        receipt_path: request.stage.receipt_path.clone(),
        stderr_path: request.stage.stderr_path.clone(),
        status_path: request.stage.status_path.clone(),
        status_code: request.status_code,
        execution_status: request.execution_status.to_string(),
        cargo_marker_absent: true,
        success: false,
        unit_count: 0,
        failed_unit_count: 0,
        binary: None,
        binary_blake3: None,
        smoke_status_code: None,
        source_built_toolchain_closure_policy_digest_blake3: request.policy_digest_blake3.map(ToOwned::to_owned),
        selected_c_compiler: request.c_compiler_route,
        blocker: Some(request.blocker),
        blocker_diagnostic: None,
    }
}

fn fixed_point_status(stage1: &FixedPointStageRun, stage2: &FixedPointStageRun) -> Result<&'static str, RunError> {
    debug_assert_eq!(stage1.name, STAGE1_DIR);
    debug_assert_eq!(stage2.name, STAGE2_DIR);
    if stage1.source_built_toolchain_closure_policy_digest_blake3
        != stage2.source_built_toolchain_closure_policy_digest_blake3
    {
        return Ok(MISMATCH_STATUS);
    }
    let stage1_digest = stage1
        .binary_blake3
        .as_deref()
        .ok_or_else(|| internal("stage1 succeeded without binary digest".to_string()))?;
    let stage2_digest = stage2
        .binary_blake3
        .as_deref()
        .ok_or_else(|| internal("stage2 succeeded without binary digest".to_string()))?;
    if stage1_digest == stage2_digest {
        Ok(SUCCESS_STATUS)
    } else {
        Ok(MISMATCH_STATUS)
    }
}

fn finish_fixed_point(
    context: &FixedPointFinishContext<'_>,
    stage1: FixedPointStageRun,
    stage2: Option<FixedPointStageRun>,
    status: &str,
) -> Result<(), RunError> {
    debug_assert_eq!(stage1.name, STAGE1_DIR);
    debug_assert!(matches!(status, SUCCESS_STATUS | BLOCKED_STATUS | MISMATCH_STATUS));
    let summary = fixed_point_summary_from_context(&context.summary, &stage1, stage2.as_ref(), status);
    write_summary(&context.summary.plan.meta_path, &summary)?;
    print_fixed_point_summary(&summary, context.json_mode, &context.summary.plan.bundle_dir)?;
    if status == SUCCESS_STATUS {
        return Ok(());
    }
    Err(RunError::Build(fixed_point_error_message(&summary)))
}

fn fixed_point_summary_from_context(
    context: &FixedPointSummaryContext<'_>,
    stage1: &FixedPointStageRun,
    stage2: Option<&FixedPointStageRun>,
    status: &str,
) -> FixedPointSummary {
    debug_assert_eq!(context.plan.schema, FIXED_POINT_SCHEMA);
    debug_assert_eq!(stage1.name, STAGE1_DIR);
    let non_claims = fixed_point_non_claims(context.toolchain_closure);
    FixedPointSummary {
        schema: FIXED_POINT_SCHEMA,
        status: status.to_string(),
        root: context.plan.root.clone(),
        bundle_dir: PathBuf::from(BUNDLE_ROOT_RELATIVE_PATH),
        fixed_point: status == SUCCESS_STATUS,
        hermeticity_mode: context.hermeticity_mode.as_str().to_string(),
        proof_eligibility: fixed_point_proof_eligibility_report(
            context.hermeticity_mode,
            context.toolchain_closure,
            context.rust_source_provider,
        ),
        stage1: stage_summary(stage1, &context.plan.bundle_dir),
        stage2: stage2.map(|stage| stage_summary(stage, &context.plan.bundle_dir)),
        rustc_compatibility: RustcCompatibilitySummary {
            requested_rustc: context.compatibility.requested_rustc.clone(),
            stage_rustc: context.compatibility.stage_rustc.clone(),
            normalization: context.compatibility.normalization,
            wrapper: context.compatibility.wrapper.clone(),
            wrapper_blake3: context.compatibility.wrapper_blake3.clone(),
        },
        source_built_toolchain_closure: context.toolchain_closure.clone(),
        rust_source_provider: context.rust_source_provider.clone(),
        blocker: fixed_point_blocker(stage1, stage2, status),
        blocker_diagnostic: fixed_point_blocker_diagnostic(stage1, stage2),
        non_claims,
    }
}

#[cfg(test)]
fn fixed_point_summary(
    plan: &FixedPointPlan,
    compatibility: &RustcCompatibilitySummary,
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: &RustSourceProviderBindingStatus,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    stage1: &FixedPointStageRun,
    stage2: Option<&FixedPointStageRun>,
    status: &str,
) -> FixedPointSummary {
    fixed_point_summary_from_context(
        &FixedPointSummaryContext {
            plan,
            compatibility,
            toolchain_closure,
            rust_source_provider,
            hermeticity_mode,
        },
        stage1,
        stage2,
        status,
    )
}

fn fixed_point_proof_eligibility_report(
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: &RustSourceProviderBindingStatus,
) -> crunch_release_core::StrictProofEligibilityReport {
    crunch_release_core::strict_proof_eligibility_gate(crunch_release_core::StrictProofEligibilityInput {
        workflow: PROVIDER_PROOF_AUDIT_WORKFLOW.to_string(),
        requested_claim: PROVIDER_PROOF_AUDIT_CLAIM.to_string(),
        hermeticity_mode: hermeticity_mode.as_str().to_string(),
        hermeticity_audit_events: Vec::new(),
        closure_status: fixed_point_closure_fact_status(toolchain_closure),
        protected_environment_status: crunch_release_core::ProofFactStatus::Satisfied,
        host_tool_status: fixed_point_host_tool_fact_status(rust_source_provider),
    })
}

fn fixed_point_closure_fact_status(
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> crunch_release_core::ProofFactStatus {
    debug_assert_eq!(toolchain_closure.schema, crate::source_toolchain_closure::SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA);
    debug_assert!(
        toolchain_closure
            .source_built_member_count
            .is_none_or(|count| { toolchain_closure.member_count.is_some_and(|member_count| count <= member_count) })
    );
    let member_count = toolchain_closure.member_count.and_then(|count| u64::try_from(count).ok());
    let source_built_count = toolchain_closure.source_built_member_count.and_then(|count| u64::try_from(count).ok());
    let predicates = SourceBuiltClosurePredicates {
        has_complete_counts: source_built_counts_are_complete(member_count, source_built_count),
        has_zero_seed_exceptions: toolchain_closure.seed_exception_count == Some(0),
    };
    if source_built_closure_status_is_satisfied(toolchain_closure, predicates) {
        return crunch_release_core::ProofFactStatus::Satisfied;
    }
    if toolchain_closure.policy_digest_blake3.is_none() || toolchain_closure.member_count.is_none() {
        return crunch_release_core::ProofFactStatus::Missing;
    }
    crunch_release_core::ProofFactStatus::Degraded
}

fn source_built_closure_status_is_satisfied(
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    predicates: SourceBuiltClosurePredicates,
) -> bool {
    if !toolchain_closure.claim {
        return false;
    }
    if toolchain_closure.status != ENFORCED_SOURCE_BUILT_STATUS {
        return false;
    }
    if toolchain_closure.policy_digest_blake3.is_none() {
        return false;
    }
    if !predicates.has_complete_counts {
        return false;
    }
    predicates.has_zero_seed_exceptions
}

fn fixed_point_host_tool_fact_status(
    rust_source_provider: &RustSourceProviderBindingStatus,
) -> crunch_release_core::ProofFactStatus {
    if rust_source_provider.status == RUST_SOURCE_PROVIDER_STATUS_VALIDATED {
        return crunch_release_core::ProofFactStatus::Satisfied;
    }
    crunch_release_core::ProofFactStatus::Degraded
}

fn stage_summary(stage: &FixedPointStageRun, bundle_dir: &Path) -> FixedPointStageSummary {
    FixedPointStageSummary {
        name: stage.name,
        dir: bundle_local_path(bundle_dir, &stage.dir),
        execution_dir: bundle_local_path(bundle_dir, &stage.execution_dir),
        receipt: bundle_local_path(bundle_dir, &stage.receipt_path),
        stderr: bundle_local_path(bundle_dir, &stage.stderr_path),
        status: bundle_local_path(bundle_dir, &stage.status_path),
        status_code: stage.status_code,
        execution_status: stage.execution_status.clone(),
        cargo_marker_absent: stage.cargo_marker_absent,
        success: stage.success,
        unit_count: stage.unit_count,
        failed_unit_count: stage.failed_unit_count,
        binary: stage.binary.as_deref().map(|path| bundle_local_path(bundle_dir, path)),
        binary_blake3: stage.binary_blake3.clone(),
        smoke_status_code: stage.smoke_status_code,
        source_built_toolchain_closure_policy_digest_blake3: stage
            .source_built_toolchain_closure_policy_digest_blake3
            .clone(),
        selected_c_compiler: stage.selected_c_compiler.clone(),
        rust_child_action_authority: existing_stage_action_artifact(
            stage,
            bundle_dir,
            RUST_CHILD_ACTION_AUTHORITY_FILE,
        ),
        rust_child_action_plan: existing_stage_action_artifact(stage, bundle_dir, RUST_CHILD_ACTION_PLAN_FILE),
        rust_child_action_audit: existing_stage_action_artifact(stage, bundle_dir, RUST_CHILD_ACTION_AUDIT_FILE),
        rust_child_action_reconciliation: existing_stage_action_artifact(
            stage,
            bundle_dir,
            RUST_CHILD_ACTION_RECONCILIATION_FILE,
        ),
        blocker: stage.blocker.clone(),
        blocker_diagnostic: stage.blocker_diagnostic.clone(),
    }
}

fn existing_stage_action_artifact(stage: &FixedPointStageRun, bundle_dir: &Path, file_name: &str) -> Option<PathBuf> {
    let path = stage.dir.join(RUST_CHILD_ACTION_EVIDENCE_DIR).join(file_name);
    path.is_file().then(|| bundle_local_path(bundle_dir, &path))
}

fn bundle_local_path(bundle_dir: &Path, path: &Path) -> PathBuf {
    match path.strip_prefix(bundle_dir) {
        Ok(relative) if relative.as_os_str().is_empty() => PathBuf::from(BUNDLE_ROOT_RELATIVE_PATH),
        Ok(relative) => relative.to_path_buf(),
        Err(_) => path.to_path_buf(),
    }
}

fn fixed_point_blocker_diagnostic(
    stage1: &FixedPointStageRun,
    stage2: Option<&FixedPointStageRun>,
) -> Option<BlockedTopologyDiagnostic> {
    stage1
        .blocker_diagnostic
        .clone()
        .or_else(|| stage2.and_then(|stage| stage.blocker_diagnostic.clone()))
}

fn fixed_point_blocker(
    stage1: &FixedPointStageRun,
    stage2: Option<&FixedPointStageRun>,
    status: &str,
) -> Option<String> {
    if let Some(blocker) = &stage1.blocker {
        return Some(format!("stage1 blocked: {blocker}"));
    }
    if let Some(blocker) = stage2.and_then(|stage| stage.blocker.as_ref()) {
        return Some(format!("stage2 blocked: {blocker}"));
    }
    if status == MISMATCH_STATUS {
        if stage2.is_some_and(|stage2| {
            stage1.source_built_toolchain_closure_policy_digest_blake3
                != stage2.source_built_toolchain_closure_policy_digest_blake3
        }) {
            return Some("stage1/stage2 toolchain closure policy digests differ".to_string());
        }
        return Some("stage1/stage2 Mantle binary digests differ".to_string());
    }
    None
}

fn fixed_point_error_message(summary: &FixedPointSummary) -> String {
    summary
        .blocker
        .clone()
        .unwrap_or_else(|| format!("Cargo-free fixed-point proof ended with status {}", summary.status))
}

fn print_fixed_point_summary(summary: &FixedPointSummary, json_mode: bool, bundle_dir: &Path) -> Result<(), RunError> {
    debug_assert_eq!(summary.schema, FIXED_POINT_SCHEMA);
    debug_assert_eq!(summary.stage1.name, STAGE1_DIR);
    if json_mode {
        let rendered = serde_json::to_string(summary).map_err(|err| internal(format!("render summary: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("Cargo-free fixed-point: {}", summary.status);
    println!("bundle: {}", bundle_dir.display());
    if let Some(digest) = &summary.stage1.binary_blake3 {
        println!("stage1_binary_blake3: {digest}");
    }
    if let Some(stage2) = &summary.stage2
        && let Some(digest) = &stage2.binary_blake3
    {
        println!("stage2_binary_blake3: {digest}");
    }
    println!("hermeticity_mode: {}", summary.hermeticity_mode);
    println!("strict_proof_admission: {}", summary.proof_eligibility.admitted);
    if !summary.proof_eligibility.blocked_classes.is_empty() {
        println!("strict_proof_blocked_classes: {}", summary.proof_eligibility.blocked_classes.join(","));
    }
    Ok(())
}

fn materialize_or_block(paths: &BuildPaths, child: &mut ChildRun) -> Result<Option<ProducedBinary>, RunError> {
    match materialize_binary(paths, child.receipt.as_ref()) {
        Ok(produced) => {
            if produced.smoke_status_code != Some(SUCCESS_EXIT_CODE) {
                child.blocker =
                    Some(format!("smoke check exited with {}", status_text(produced.smoke_status_code).trim_end()));
            }
            Ok(Some(produced))
        }
        Err(err) => {
            child.blocker = Some(err.message().to_string());
            Ok(None)
        }
    }
}

fn materialize_binary(paths: &BuildPaths, receipt: Option<&Value>) -> Result<ProducedBinary, RunError> {
    debug_assert_eq!(paths.binary_path.parent(), Some(paths.out_dir.as_path()));
    debug_assert_ne!(paths.execution_dir, paths.out_dir);
    let receipt = receipt.ok_or_else(|| internal("successful rust-plan produced no receipt".to_string()))?;
    let (unit_id, source_digest) = mantle_unit_from_receipt(receipt)?;
    let source_closure_digest_blake3 = receipt
        .pointer("/rust_plan/source_closure/digest_blake3")
        .and_then(Value::as_str)
        .map(str::to_string);
    let source_binary = paths.execution_dir.join(safe_path_component(&unit_id)).join(MANTLE_TARGET_NAME);
    require_executable(&source_binary)?;
    fs::copy(&source_binary, &paths.binary_path).map_err(|err| {
        internal(format!(
            "copy produced binary {} to {}: {err}",
            source_binary.display(),
            paths.binary_path.display()
        ))
    })?;
    require_executable(&paths.binary_path)?;
    let blake3 = blake3_file(&paths.binary_path)?;
    let smoke_status_code = run_smoke(&paths.binary_path, &paths.out_dir)?;
    Ok(ProducedBinary {
        path: paths.binary_path.clone(),
        blake3,
        source_digest,
        source_closure_digest_blake3,
        smoke_status_code: Some(smoke_status_code),
    })
}

fn mantle_unit_from_receipt(receipt: &Value) -> Result<(String, Value), RunError> {
    let units = receipt
        .pointer("/topology_execution/unit_executions")
        .and_then(Value::as_array)
        .ok_or_else(|| internal("receipt has no topology_execution.unit_executions".to_string()))?;
    let matches = units.iter().filter(|unit| is_successful_mantle_unit(unit)).collect::<Vec<_>>();
    if matches.len() != EXPECTED_MANTLE_UNIT_COUNT {
        return Err(RunError::Build(format!("expected one successful mantle bin unit, found {}", matches.len())));
    }
    let unit = matches[0];
    let unit_id = unit
        .get("unit_id")
        .and_then(Value::as_str)
        .ok_or_else(|| internal("mantle bin unit lacks unit_id".to_string()))?;
    let source_digest = source_digest_from_unit(unit)?;
    Ok((unit_id.to_string(), source_digest))
}

fn source_digest_from_unit(unit: &Value) -> Result<Value, RunError> {
    let digest = unit
        .get(SOURCE_DIGEST_FIELD)
        .cloned()
        .ok_or_else(|| RunError::Build("mantle bin unit lacks source_digest".to_string()))?;
    if digest.is_null() {
        return Err(RunError::Build("mantle bin unit lacks source_digest".to_string()));
    }
    if digest.get(SOURCE_DIGEST_ALGORITHM_FIELD).and_then(Value::as_str).is_none() {
        return Err(RunError::Build("mantle bin unit source_digest lacks algorithm".to_string()));
    }
    if digest.get(SOURCE_DIGEST_VALUE_FIELD).and_then(Value::as_str).is_none() {
        return Err(RunError::Build("mantle bin unit source_digest lacks value".to_string()));
    }
    Ok(digest)
}

fn is_successful_mantle_unit(unit: &Value) -> bool {
    unit.get("target_name").and_then(Value::as_str) == Some(MANTLE_TARGET_NAME)
        && unit.get("target_kind").and_then(Value::as_str) == Some(MANTLE_TARGET_KIND)
        && unit.get("execution_status").and_then(Value::as_str) == Some(SUCCESS_STATUS)
}

fn summarize(context: SelfBuildSummaryContext<'_>) -> SelfBuildSummary {
    debug_assert_eq!(context.paths.receipt_path.parent(), Some(context.paths.out_dir.as_path()));
    debug_assert_eq!(context.paths.stderr_path.parent(), Some(context.paths.out_dir.as_path()));
    let receipt = context.child.receipt.as_ref();
    let non_claims = self_build_non_claims(context.toolchain_closure);
    SelfBuildSummary {
        schema: SCHEMA,
        status: if context.child.blocker.is_none() {
            SUCCESS_STATUS
        } else {
            BLOCKED_STATUS
        }
        .to_string(),
        root: context.paths.root.clone(),
        out_dir: context.paths.out_dir.clone(),
        binary: context.produced.map(|value| value.path.clone()),
        binary_blake3: context.produced.map(|value| value.blake3.clone()),
        source_digest: context.produced.map(|value| value.source_digest.clone()),
        source_closure_digest_blake3: context.produced.and_then(|value| value.source_closure_digest_blake3.clone()),
        receipt: context.paths.receipt_path.clone(),
        stderr: context.paths.stderr_path.clone(),
        status_code: context.child.status_code,
        execution_status: context.child.execution_status.clone(),
        cargo_marker_absent: context.child.cargo_marker_absent,
        unit_count: receipt.map(unit_count).unwrap_or_default(),
        failed_unit_count: receipt.map(failed_unit_count).unwrap_or_default(),
        smoke_status_code: context.produced.and_then(|value| value.smoke_status_code),
        hermeticity_mode: context.hermeticity_mode.as_str().to_string(),
        blocker: context.child.blocker.clone(),
        blocker_diagnostic: context.child.blocker_diagnostic.clone(),
        source_built_toolchain_closure: context.toolchain_closure.clone(),
        rust_source_provider: context.rust_source_provider.clone(),
        non_claims,
    }
}

fn unit_count(receipt: &Value) -> u64 {
    receipt
        .pointer("/topology_execution/unit_executions")
        .and_then(Value::as_array)
        .map(|items| items.len() as u64)
        .unwrap_or_default()
}

fn failed_unit_count(receipt: &Value) -> u64 {
    receipt
        .pointer("/topology_execution/unit_executions")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| item.get("execution_status").and_then(Value::as_str) != Some(SUCCESS_STATUS))
                .count() as u64
        })
        .unwrap_or_default()
}

fn parse_receipt(path: &Path, command_succeeded: bool) -> Result<Option<Value>, RunError> {
    if !command_succeeded {
        return Ok(None);
    }
    let bytes = fs::read(path).map_err(|err| internal(format!("read receipt {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|err| internal(format!("parse receipt {}: {err}", path.display())))
}

fn record_file_write(path: &Path, bytes: &[u8], blocker: &mut Option<String>) {
    if let Err(err) = fs::write(path, bytes) {
        let write_failure = format!("write {}: {err}", path.display());
        match blocker {
            Some(existing) => {
                existing.push_str("; additional failure: ");
                existing.push_str(&write_failure);
            }
            None => *blocker = Some(write_failure),
        }
    }
}

fn receipt_execution_status(receipt: Option<&Value>) -> String {
    receipt
        .and_then(|value| value.pointer("/topology_execution/execution_status"))
        .and_then(Value::as_str)
        .unwrap_or("missing")
        .to_string()
}

fn run_smoke(binary: &Path, out_dir: &Path) -> Result<i32, RunError> {
    run_smoke_to_paths(binary, &out_dir.join(SMOKE_STDOUT_FILE), &out_dir.join(SMOKE_STDERR_FILE))
}

fn run_smoke_to_paths(binary: &Path, stdout_path: &Path, stderr_path: &Path) -> Result<i32, RunError> {
    let output = Command::new(binary)
        .arg(HELP_FLAG)
        .output()
        .map_err(|err| internal(format!("run smoke {}: {err}", binary.display())))?;
    write_bytes(stdout_path, &output.stdout)?;
    write_bytes(stderr_path, &output.stderr)?;
    Ok(output.status.code().unwrap_or(FALLBACK_ERROR_EXIT_CODE))
}

fn write_blocked_smoke_outputs(paths: &BuildPaths, blocker: Option<&str>) -> Result<(), RunError> {
    let stderr = match blocker {
        Some(message) => format!("{BLOCKED_SMOKE_STDERR_PREFIX}: {message}\n"),
        None => format!("{BLOCKED_SMOKE_STDERR_PREFIX}\n"),
    };
    write_text(&paths.out_dir.join(SMOKE_STDOUT_FILE), BLOCKED_SMOKE_STDOUT)?;
    write_text(&paths.out_dir.join(SMOKE_STDERR_FILE), &stderr)
}

fn write_blocked_fixed_point_smoke_outputs(stage: &FixedPointStagePlan, blocker: &str) -> Result<(), RunError> {
    write_text(&stage.smoke_stdout_path, BLOCKED_SMOKE_STDOUT)?;
    write_text(&stage.smoke_stderr_path, &format!("{BLOCKED_SMOKE_STDERR_PREFIX}: {blocker}\n"))
}

fn prepare_execution_toolchain(
    guard_path_dir: &Path,
    requested_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    rust_source_provider: &LoadedRustSourceProvider,
) -> Result<ExecutionToolchain, RunError> {
    let execution_toolchain = if let Some(manifest) = &toolchain_closure.manifest {
        let original_rustc = resolve_executable(requested_rustc, "rustc")?;
        reject_undeclared_external_rustc_wrapper(&original_rustc, manifest)?;
        let c_compiler_route = Some(receipt_bound_c_compiler_route(manifest)?);
        let execution_shell = selected_execution_shell(rust_source_provider);
        if let Some(bound_shell) = &rust_source_provider.source_built_shell {
            debug_assert_eq!(execution_shell, bound_shell.execution_path);
            debug_assert_eq!(bound_shell.content_digest_blake3.len(), blake3::OUT_LEN * 2);
            debug_assert!(!bound_shell.source_id.is_empty());
            debug_assert_eq!(bound_shell.construction_receipt_digest_blake3.len(), blake3::OUT_LEN * 2);
        }
        let path_env = execution_path_env_with_shell(guard_path_dir, toolchain_closure, execution_shell)?;
        write_bound_rust_host_tool_aliases(
            guard_path_dir,
            &rust_source_provider.source_built_host_tools,
            execution_shell,
        )?;
        let runtime = bound_rustc_runtime(&rust_source_provider.source_built_native_artifacts)?;
        let rustc = if let Some(runtime) = runtime {
            let wrapper = guard_path_dir.join(RUSTC_RUNTIME_WRAPPER_FILE);
            let receipt_bound_linker = guard_path_dir.join(RUSTC_BOUND_LINKER_ALIAS);
            write_bound_rustc_runtime_wrapper(
                &wrapper,
                &original_rustc,
                &runtime,
                execution_shell,
                &receipt_bound_linker,
            )?;
            wrapper
        } else {
            original_rustc.clone()
        };
        let status =
            enforce_receipt_bound_toolchain_with_runtime(&original_rustc, &rustc, toolchain_closure, manifest)?;
        ExecutionToolchain {
            rustc,
            path_env,
            status,
            c_compiler_route,
        }
    } else {
        ExecutionToolchain {
            rustc: requested_rustc.to_path_buf(),
            path_env: guarded_path(guard_path_dir)?,
            status: effective_source_built_toolchain_closure(toolchain_closure, rust_source_provider),
            c_compiler_route: None,
        }
    };
    debug_assert!(!execution_toolchain.rustc.as_os_str().is_empty());
    debug_assert!(!execution_toolchain.path_env.is_empty());
    Ok(execution_toolchain)
}

fn fixed_point_rust_action_stage_id(stage_name: &str) -> Result<&'static str, RunError> {
    match stage_name {
        STAGE1_DIR => Ok(crate::source_built_fixed_point::MANTLE_STAGE1_STAGE_ID),
        STAGE2_DIR => Ok(crate::source_built_fixed_point::MANTLE_STAGE2_STAGE_ID),
        other => Err(internal(format!("unknown fixed-point Rust action stage: {other}"))),
    }
}

fn prepare_rust_child_action_authority(
    stage_id: &str,
    resources: Option<&crate::source_built_rust_action_plan::RustActionResourceLimits>,
    selected_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    bound_shell: Option<&BoundRustExecutionShell>,
    source_built_host_tools: &[crate::full_source_rust_binding::FullSourceRustHostToolBinding],
    source_built_native_artifacts: &[crate::full_source_rust_binding::FullSourceNativeArtifactBinding],
    source_built_action_trust: Option<&crate::source_built_rust_provider_action::RustProviderActionEvidence>,
    guard_path_dir: &Path,
    authority_path: &Path,
) -> Result<bool, RunError> {
    let Some(resources) = resources else {
        return Ok(false);
    };
    let manifest = toolchain_closure.manifest.as_ref().ok_or_else(|| {
        RunError::Build("Rust child-action authority requires an explicit toolchain closure manifest".to_string())
    })?;
    let bound_shell = bound_shell.ok_or_else(|| {
        RunError::Build("Rust child-action authority requires a source-built BusyBox shell binding".to_string())
    })?;
    let source_built_action_trust = source_built_action_trust.ok_or_else(|| {
        RunError::Build("Rust child-action authority requires Rust-provider action reconciliation".to_string())
    })?;
    let fixed_executables = rust_child_action_fixed_executables(
        selected_rustc,
        manifest,
        bound_shell,
        source_built_host_tools,
        source_built_native_artifacts,
        source_built_action_trust,
        guard_path_dir,
        toolchain_closure.status.policy_digest_blake3.as_deref(),
    )?;
    let authority = crate::source_built_rust_action_plan::rust_child_action_authority(
        stage_id.to_string(),
        resources.clone(),
        fixed_executables,
    )
    .map_err(rust_action_error)?;
    write_new_rust_action_authority(authority_path, &authority)?;
    assert!(authority_path.is_file());
    assert!(!authority.fixed_executables.is_empty());
    Ok(true)
}

fn rust_child_action_fixed_executables(
    selected_rustc: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
    bound_shell: &BoundRustExecutionShell,
    source_built_host_tools: &[crate::full_source_rust_binding::FullSourceRustHostToolBinding],
    source_built_native_artifacts: &[crate::full_source_rust_binding::FullSourceNativeArtifactBinding],
    source_built_action_trust: &crate::source_built_rust_provider_action::RustProviderActionEvidence,
    guard_path_dir: &Path,
    policy_digest_blake3: Option<&str>,
) -> Result<Vec<crate::source_built_rust_action_plan::RustFixedExecutableAuthority>, RunError> {
    use crate::source_built_rust_action_plan::RustFixedExecutableKind;
    let selected_rustc = canonical_action_executable(selected_rustc)?;
    let policy_digest_blake3 = policy_digest_blake3.ok_or_else(|| {
        RunError::Build("Rust child-action authority requires a toolchain closure policy digest".to_string())
    })?;
    let alias_producer = rust_action_alias_producer(policy_digest_blake3, bound_shell, source_built_action_trust);
    let closure_authorities =
        crate::source_built_rust_action_plan::fixed_executable_authorities_from_toolchain_closure(manifest)
            .map_err(rust_action_error)?;
    let mut by_path = BTreeMap::new();
    for authority in closure_authorities {
        let path = canonical_action_executable(Path::new(&authority.path))?;
        let is_rustc_authority = authority.kind == RustFixedExecutableKind::Rustc;
        let kind = if path == selected_rustc {
            RustFixedExecutableKind::Rustc
        } else if is_rustc_authority {
            RustFixedExecutableKind::NativeHelper
        } else {
            authority.kind.clone()
        };
        let producer_action_id = if is_rustc_authority {
            format!(
                "rust-provider-action:{}:{}",
                source_built_action_trust.plan_digest_blake3, source_built_action_trust.reconciliation_digest_blake3
            )
        } else {
            authority.producer_action_id
        };
        let measured = measured_rust_fixed_authority(&path, kind, &producer_action_id)?;
        insert_rust_fixed_authority(&mut by_path, measured)?;
    }
    for artifact in source_built_native_artifacts {
        let Some(kind) = rust_fixed_kind_for_native_artifact(artifact.role) else {
            continue;
        };
        let path = canonical_action_executable(Path::new(&artifact.path))?;
        let producer = format!("full-source-native-provider:{policy_digest_blake3}:{:?}", artifact.role);
        let authority = measured_rust_fixed_authority(&path, kind, &producer)?;
        insert_rust_fixed_authority(&mut by_path, authority)?;
    }
    for tool in source_built_host_tools {
        let path = canonical_action_executable(Path::new(&tool.path))?;
        let kind = if tool.role == crate::full_source_rust_binding::FullSourceRustHostToolRole::Busybox {
            RustFixedExecutableKind::Shell
        } else {
            RustFixedExecutableKind::NativeHelper
        };
        let producer = format!("rust-host-tool:{}:{}", tool.source_id, tool.construction_receipt_digest_blake3);
        let authority = measured_rust_fixed_authority(&path, kind, &producer)?;
        insert_rust_fixed_authority(&mut by_path, authority)?;
    }
    let toolchain_aliases = toolchain_path_aliases(manifest)?;
    let host_tool_aliases = bound_rust_host_tool_aliases(source_built_host_tools)?;
    for alias in toolchain_aliases.keys().chain(host_tool_aliases.keys()) {
        let path = canonical_action_executable(&guard_path_dir.join(alias))?;
        let authority = measured_rust_fixed_authority(&path, RustFixedExecutableKind::NativeHelper, &alias_producer)?;
        insert_rust_fixed_authority(&mut by_path, authority)?;
    }
    let shell_path = canonical_action_executable(&bound_shell.execution_path)?;
    let shell = measured_rust_fixed_authority(&shell_path, RustFixedExecutableKind::Shell, &alias_producer)?;
    insert_rust_fixed_authority(&mut by_path, shell)?;
    if let Some(linker) = rustc_runtime_wrapper_linker(&selected_rustc)? {
        let producer = format!("{alias_producer}:rustc-linker");
        let linker = measured_rust_fixed_authority(&linker, RustFixedExecutableKind::CCompiler, &producer)?;
        insert_rust_fixed_authority(&mut by_path, linker)?;
    }
    if !by_path.contains_key(&selected_rustc) {
        let rustc_producer = format!("{alias_producer}:selected-rustc");
        let rustc = measured_rust_fixed_authority(&selected_rustc, RustFixedExecutableKind::Rustc, &rustc_producer)?;
        insert_rust_fixed_authority(&mut by_path, rustc)?;
    }
    let authorities = by_path.into_values().collect::<Vec<_>>();
    let rustc_count = authorities.iter().filter(|authority| authority.kind == RustFixedExecutableKind::Rustc).count();
    if rustc_count != 1 {
        return Err(RunError::Build(format!(
            "Rust child-action authority requires one selected rustc, found {rustc_count}"
        )));
    }
    assert!(!authorities.is_empty());
    assert!(authorities.iter().all(|authority| Path::new(&authority.path).is_absolute()));
    Ok(authorities)
}

fn rustc_runtime_wrapper_linker(selected_rustc: &Path) -> Result<Option<PathBuf>, RunError> {
    if selected_rustc.file_name() != Some(OsStr::new(RUSTC_RUNTIME_WRAPPER_FILE)) {
        return Ok(None);
    }
    let parent = selected_rustc
        .parent()
        .ok_or_else(|| RunError::Build("receipt-bound rustc wrapper has no parent".to_string()))?;
    let candidates = [
        parent.join(RUSTC_BOUND_LINKER_ALIAS),
        parent.join(TOOLCHAIN_COMPATIBILITY_PATH_DIR).join(RUSTC_BOUND_LINKER_ALIAS),
    ]
    .into_iter()
    .filter(|candidate| candidate.is_file())
    .collect::<Vec<_>>();
    let [linker] = candidates.as_slice() else {
        return Err(RunError::Build(format!(
            "receipt-bound rustc wrapper requires one linker alias, found {}",
            candidates.len()
        )));
    };
    canonical_action_executable(linker).map(Some)
}

fn rust_action_alias_producer(
    policy_digest_blake3: &str,
    shell: &BoundRustExecutionShell,
    action_trust: &crate::source_built_rust_provider_action::RustProviderActionEvidence,
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(RUST_CHILD_ACTION_ALIAS_PRODUCER_CONTEXT);
    hasher.update(policy_digest_blake3.as_bytes());
    hasher.update(shell.content_digest_blake3.as_bytes());
    hasher.update(shell.source_id.as_bytes());
    hasher.update(shell.construction_receipt_digest_blake3.as_bytes());
    hasher.update(action_trust.plan_digest_blake3.as_bytes());
    hasher.update(action_trust.reconciliation_digest_blake3.as_bytes());
    let digest = hasher.finalize().to_hex();
    assert_eq!(digest.len(), blake3::OUT_LEN * 2);
    assert!(!shell.source_id.is_empty());
    format!("receipt-bound-aliases:{digest}")
}

fn rust_fixed_kind_for_native_artifact(
    role: crate::full_source_rust_binding::FullSourceNativeArtifactRole,
) -> Option<crate::source_built_rust_action_plan::RustFixedExecutableKind> {
    use crate::full_source_rust_binding::FullSourceNativeArtifactRole;
    use crate::source_built_rust_action_plan::RustFixedExecutableKind;
    match role {
        FullSourceNativeArtifactRole::CCompiler | FullSourceNativeArtifactRole::Preprocessor => {
            Some(RustFixedExecutableKind::CCompiler)
        }
        FullSourceNativeArtifactRole::CxxCompiler => Some(RustFixedExecutableKind::CxxCompiler),
        FullSourceNativeArtifactRole::Linker => Some(RustFixedExecutableKind::Linker),
        FullSourceNativeArtifactRole::CompilerInternal
        | FullSourceNativeArtifactRole::Assembler
        | FullSourceNativeArtifactRole::ArchiveTool
        | FullSourceNativeArtifactRole::Ranlib
        | FullSourceNativeArtifactRole::SymbolTool
        | FullSourceNativeArtifactRole::ObjectCopy
        | FullSourceNativeArtifactRole::ObjectDump
        | FullSourceNativeArtifactRole::ObjectFormat => Some(RustFixedExecutableKind::NativeHelper),
        FullSourceNativeArtifactRole::DynamicLinker => Some(RustFixedExecutableKind::NativeHelper),
        FullSourceNativeArtifactRole::CrtObject
        | FullSourceNativeArtifactRole::Libc
        | FullSourceNativeArtifactRole::Libgcc
        | FullSourceNativeArtifactRole::Libstdcxx => None,
    }
}

fn measured_rust_fixed_authority(
    path: &Path,
    kind: crate::source_built_rust_action_plan::RustFixedExecutableKind,
    producer_action_id: &str,
) -> Result<crate::source_built_rust_action_plan::RustFixedExecutableAuthority, RunError> {
    let digest_blake3 = crate::protected_exec::blake3_file_hex(path)
        .map_err(|error| RunError::Build(format!("hash Rust child-action executable {}: {error}", path.display())))?;
    let path = path_to_string(path)?;
    let material = serde_json::to_vec(&(&path, &digest_blake3, &kind, producer_action_id))
        .map_err(|error| internal(format!("encode Rust fixed executable authority: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(RUST_CHILD_ACTION_FIXED_AUTHORITY_CONTEXT);
    hasher.update(&material);
    let authority_id = format!("fixed:{}", hasher.finalize().to_hex());
    assert_eq!(digest_blake3.len(), blake3::OUT_LEN * 2);
    assert!(!producer_action_id.is_empty());
    Ok(crate::source_built_rust_action_plan::RustFixedExecutableAuthority {
        authority_id,
        producer_action_id: producer_action_id.to_string(),
        output_identity_blake3: digest_blake3.clone(),
        path,
        digest_blake3,
        kind,
    })
}

fn insert_rust_fixed_authority(
    by_path: &mut BTreeMap<PathBuf, crate::source_built_rust_action_plan::RustFixedExecutableAuthority>,
    authority: crate::source_built_rust_action_plan::RustFixedExecutableAuthority,
) -> Result<(), RunError> {
    use crate::source_built_rust_action_plan::RustFixedExecutableKind;
    let path = PathBuf::from(&authority.path);
    if let Some(existing) = by_path.get(&path) {
        if existing.digest_blake3 != authority.digest_blake3 {
            return Err(RunError::Build(format!(
                "Rust child-action executable path has conflicting bytes: {}",
                path.display()
            )));
        }
        if existing.kind == RustFixedExecutableKind::Rustc || authority.kind != RustFixedExecutableKind::Rustc {
            return Ok(());
        }
    }
    by_path.insert(path, authority);
    assert!(!by_path.is_empty());
    assert!(by_path.keys().all(|path| path.is_absolute()));
    Ok(())
}

fn canonical_action_executable(path: &Path) -> Result<PathBuf, RunError> {
    let path = fs::canonicalize(path).map_err(|error| {
        RunError::Build(format!("resolve Rust child-action executable {}: {error}", path.display()))
    })?;
    require_executable(&path)?;
    assert!(path.is_absolute());
    assert!(path.is_file());
    Ok(path)
}

fn write_new_rust_action_authority(
    path: &Path,
    authority: &crate::source_built_rust_action_plan::RustChildActionAuthority,
) -> Result<(), RunError> {
    let bytes = serde_json::to_vec_pretty(authority)
        .map_err(|error| internal(format!("encode Rust child-action authority: {error}")))?;
    let parent = path
        .parent()
        .ok_or_else(|| internal(format!("Rust child-action authority path has no parent: {}", path.display())))?;
    fs::create_dir_all(parent).map_err(|error| {
        internal(format!("create Rust child-action authority directory {}: {error}", parent.display()))
    })?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| internal(format!("create Rust child-action authority {}: {error}", path.display())))?;
    file.write_all(&bytes)
        .map_err(|error| internal(format!("write Rust child-action authority {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| internal(format!("sync Rust child-action authority {}: {error}", path.display())))?;
    assert!(!bytes.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn rust_action_error(error: crate::source_built_rust_action_plan::RustChildActionPlanError) -> RunError {
    RunError::Build(format!("Rust child-action authority blocked: {error}"))
}

fn prepare_rustc_for_compatibility(
    requested_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<PathBuf, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(requested_rustc.to_path_buf());
    };
    let rustc = resolve_executable(requested_rustc, "rustc")?;
    reject_undeclared_external_rustc_wrapper(&rustc, manifest)?;
    enforce_observed_toolchain_subset(manifest, &[observed_file_tool(
        crate::source_toolchain_closure::ToolchainRole::Rustc,
        &rustc,
    )?])?;
    Ok(rustc)
}

fn reject_undeclared_external_rustc_wrapper(
    rustc: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<(), RunError> {
    debug_assert!(rustc.is_absolute());
    if declared_toolchain_member_path_matches(manifest, crate::source_toolchain_closure::ToolchainRole::Rustc, rustc) {
        return Ok(());
    }
    if !is_probable_external_wrapper(rustc)? {
        return Ok(());
    }
    Err(RunError::Build(format!(
        "{EXTERNAL_WRAPPER_BLOCKER}: rustc {} is not a declared source-built toolchain closure member",
        rustc.display()
    )))
}

fn declared_toolchain_member_path_matches(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
    role: crate::source_toolchain_closure::ToolchainRole,
    path: &Path,
) -> bool {
    manifest.members.iter().filter(|member| member.role == role).any(|member| {
        canonicalize_toolchain_path(Path::new(&member.execution_path), role).is_ok_and(|declared| declared == path)
    })
}

fn is_probable_external_wrapper(path: &Path) -> Result<bool, RunError> {
    let mut file =
        fs::File::open(path).map_err(|err| internal(format!("open rustc candidate {}: {err}", path.display())))?;
    let mut bytes = vec![0u8; WRAPPER_PROBE_BYTES_MAX];
    let read = file
        .read(&mut bytes)
        .map_err(|err| internal(format!("read rustc candidate {}: {err}", path.display())))?;
    bytes.truncate(read);
    if !bytes.starts_with(SHEBANG_BYTES) {
        return Ok(false);
    }
    Ok(bytes.windows(WRAPPER_RUSTC_MARKER.len()).any(|window| window == WRAPPER_RUSTC_MARKER))
}

fn enforce_fixed_point_toolchain(
    requested_rustc: &Path,
    stage_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    rust_source_provider: &LoadedRustSourceProvider,
) -> Result<crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(effective_source_built_toolchain_closure(toolchain_closure, rust_source_provider));
    };
    enforce_receipt_bound_toolchain_with_runtime(requested_rustc, stage_rustc, toolchain_closure, manifest)
}

fn effective_source_built_toolchain_closure(
    toolchain_closure: &LoadedToolchainClosure,
    rust_source_provider: &LoadedRustSourceProvider,
) -> crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus {
    if toolchain_closure.manifest.is_some() {
        return toolchain_closure.status.clone();
    }
    rust_source_provider
        .toolchain_closure_status
        .clone()
        .unwrap_or_else(|| toolchain_closure.status.clone())
}

fn enforce_receipt_bound_toolchain(
    rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus, RunError> {
    enforce_receipt_bound_toolchain_with_runtime(rustc, rustc, toolchain_closure, manifest)
}

fn enforce_observed_toolchain_subset(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
    observed: &[crate::source_toolchain_closure::ToolchainObservedInput],
) -> Result<crate::source_toolchain_closure::ToolchainClosureValidation, RunError> {
    crate::source_toolchain_closure::enforce_observed_toolchain_inputs(manifest, observed)
        .map_err(|err| RunError::Build(format!("source-built toolchain closure blocked: {}", err.message())))
}

fn enforce_receipt_bound_toolchain_with_runtime(
    rustc: &Path,
    runtime_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus, RunError> {
    let manifest_path = toolchain_closure
        .manifest_path
        .clone()
        .ok_or_else(|| internal("toolchain closure manifest path missing during enforcement".to_string()))?;
    let observed = observed_toolchain_inputs(rustc, runtime_rustc, manifest)?;
    let validation = enforce_observed_toolchain_subset(manifest, &observed)?;
    Ok(crate::source_toolchain_closure::enforced_source_built_toolchain_closure(manifest_path, &validation))
}

fn observed_toolchain_inputs(
    rustc: &Path,
    runtime_rustc: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<Vec<crate::source_toolchain_closure::ToolchainObservedInput>, RunError> {
    use crate::source_toolchain_closure::ToolchainRole;
    let mut observed = Vec::with_capacity(
        manifest
            .members
            .len()
            .checked_add(BASE_OBSERVED_TOOLCHAIN_INPUT_COUNT)
            .ok_or_else(|| RunError::Build("source-built toolchain closure input capacity overflow".to_string()))?,
    );
    debug_assert!(observed.capacity() >= BASE_OBSERVED_TOOLCHAIN_INPUT_COUNT);
    observed.push(observed_file_tool(ToolchainRole::Rustc, rustc)?);
    observed.push(observed_sysroot_tool(runtime_rustc)?);
    for role in [ToolchainRole::Linker, ToolchainRole::CCompiler] {
        let member = single_member_for_role(manifest, role)?;
        observed.push(observed_member_tool(member)?);
    }
    for member in optional_tool_members(manifest) {
        observed.push(observed_member_tool(member)?);
    }
    for member in declared_file_members(manifest) {
        observed.push(observed_member_file(member)?);
    }
    debug_assert!(observed.len() <= observed.capacity());
    Ok(observed)
}

fn observed_file_tool(
    role: crate::source_toolchain_closure::ToolchainRole,
    path: &Path,
) -> Result<crate::source_toolchain_closure::ToolchainObservedInput, RunError> {
    let path = canonicalize_toolchain_path(path, role)?;
    require_executable(&path)?;
    Ok(crate::source_toolchain_closure::ToolchainObservedInput {
        role,
        execution_path: path_to_string(&path)?,
        content_digest_blake3: Some(blake3_file(&path)?),
    })
}

fn observed_member_tool(
    member: &crate::source_toolchain_closure::ToolchainClosureMember,
) -> Result<crate::source_toolchain_closure::ToolchainObservedInput, RunError> {
    observed_file_tool(member.role, Path::new(&member.execution_path))
}

fn observed_member_file(
    member: &crate::source_toolchain_closure::ToolchainClosureMember,
) -> Result<crate::source_toolchain_closure::ToolchainObservedInput, RunError> {
    let path = canonicalize_toolchain_path(Path::new(&member.execution_path), member.role)?;
    Ok(crate::source_toolchain_closure::ToolchainObservedInput {
        role: member.role,
        execution_path: path_to_string(&path)?,
        content_digest_blake3: Some(blake3_file(&path)?),
    })
}

fn observed_sysroot_tool(rustc: &Path) -> Result<crate::source_toolchain_closure::ToolchainObservedInput, RunError> {
    let sysroot = rustc_reported_sysroot(rustc)?;
    Ok(crate::source_toolchain_closure::ToolchainObservedInput {
        role: crate::source_toolchain_closure::ToolchainRole::Sysroot,
        execution_path: path_to_string(&sysroot)?,
        content_digest_blake3: None,
    })
}

fn rustc_reported_sysroot(rustc: &Path) -> Result<PathBuf, RunError> {
    debug_assert!(!rustc.as_os_str().is_empty());
    debug_assert!(!RUSTC_SYSROOT_PRINT_ARG.is_empty());
    let output = Command::new(rustc).arg("--print").arg(RUSTC_SYSROOT_PRINT_ARG).output().map_err(|err| {
        RunError::Build(format!("source-built toolchain closure blocked: rustc --print sysroot failed: {err}"))
    })?;
    if !output.status.success() {
        return Err(RunError::Build(format!(
            "source-built toolchain closure blocked: rustc --print sysroot exited with {}",
            status_text(output.status.code()).trim_end()
        )));
    }
    let text = String::from_utf8(output.stdout).map_err(|err| {
        RunError::Build(format!("source-built toolchain closure blocked: rustc sysroot was not UTF-8: {err}"))
    })?;
    let sysroot = PathBuf::from(text.trim());
    fs::canonicalize(&sysroot).map_err(|err| {
        RunError::Build(format!(
            "source-built toolchain closure blocked: canonicalize rustc sysroot {}: {err}",
            sysroot.display()
        ))
    })
}

fn single_member_for_role(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
    role: crate::source_toolchain_closure::ToolchainRole,
) -> Result<&crate::source_toolchain_closure::ToolchainClosureMember, RunError> {
    let members = manifest.members.iter().filter(|member| member.role == role).collect::<Vec<_>>();
    match members.as_slice() {
        [member] => Ok(member),
        [] => Err(RunError::Build(format!("source-built toolchain closure blocked: missing {role:?} member"))),
        _ => Err(RunError::Build(format!("source-built toolchain closure blocked: multiple {role:?} members"))),
    }
}

fn optional_tool_members(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Vec<&crate::source_toolchain_closure::ToolchainClosureMember> {
    use crate::source_toolchain_closure::ToolchainRole;
    manifest
        .members
        .iter()
        .filter(|member| matches!(member.role, ToolchainRole::PkgConfig | ToolchainRole::NativeHelper))
        .collect()
}

fn declared_file_members(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Vec<&crate::source_toolchain_closure::ToolchainClosureMember> {
    use crate::source_toolchain_closure::ToolchainRole;
    manifest
        .members
        .iter()
        .filter(|member| matches!(member.role, ToolchainRole::CrtObject | ToolchainRole::RuntimeLibrary))
        .collect()
}

fn execution_path_env(cargo_path_dir: &Path, toolchain_closure: &LoadedToolchainClosure) -> Result<OsString, RunError> {
    execution_path_env_with_shell(cargo_path_dir, toolchain_closure, Path::new("/bin/sh"))
}

fn execution_path_env_with_shell(
    cargo_path_dir: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    execution_shell: &Path,
) -> Result<OsString, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return guarded_path(cargo_path_dir);
    };
    write_toolchain_path_aliases_with_shell(cargo_path_dir, manifest, execution_shell)?;
    env::join_paths([cargo_path_dir]).map_err(|err| internal(format!("construct receipt-bound PATH: {err}")))
}

fn write_bound_rust_host_tool_aliases(
    guard_path_dir: &Path,
    tools: &[crate::full_source_rust_binding::FullSourceRustHostToolBinding],
    execution_shell: &Path,
) -> Result<(), RunError> {
    for (alias, target) in bound_rust_host_tool_aliases(tools)? {
        let path = guard_path_dir.join(alias);
        remove_owned_path(&path)?;
        write_toolchain_alias(&target, &path, execution_shell)?;
    }
    Ok(())
}

fn bound_rust_host_tool_aliases(
    tools: &[crate::full_source_rust_binding::FullSourceRustHostToolBinding],
) -> Result<BTreeMap<String, PathBuf>, RunError> {
    use crate::full_source_rust_binding::FullSourceRustHostToolRole;
    let mut aliases = BTreeMap::new();
    for tool in tools {
        let target = canonical_action_executable(Path::new(&tool.path))?;
        add_toolchain_alias(&mut aliases, path_file_name(&target)?, &target)?;
        let role_aliases = match tool.role {
            FullSourceRustHostToolRole::Make => vec!["make"],
            FullSourceRustHostToolRole::Cmake => vec!["cmake"],
            FullSourceRustHostToolRole::Python => vec!["python", "python3"],
            FullSourceRustHostToolRole::Perl => vec!["perl"],
            FullSourceRustHostToolRole::Busybox => vec!["busybox"],
        };
        for alias in role_aliases {
            add_toolchain_alias(&mut aliases, alias.to_string(), &target)?;
        }
    }
    assert!(tools.is_empty() || !aliases.is_empty());
    assert!(aliases.keys().all(|alias| !alias.is_empty()));
    Ok(aliases)
}

fn write_toolchain_path_aliases(
    guard_path_dir: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<(), RunError> {
    write_toolchain_path_aliases_with_shell(guard_path_dir, manifest, Path::new("/bin/sh"))
}

fn write_toolchain_path_aliases_with_shell(
    guard_path_dir: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
    execution_shell: &Path,
) -> Result<(), RunError> {
    if !execution_shell.is_absolute() {
        return Err(RunError::Build(format!(
            "receipt-bound toolchain shell is not absolute: {}",
            execution_shell.display()
        )));
    }
    let aliases = toolchain_path_aliases(manifest)?;
    let c_compiler = c_compiler_alias_target(manifest)?;
    let c_compiler_runtime = c_compiler_alias_runtime_inputs(manifest)?;
    for (alias, target) in aliases {
        let link = guard_path_dir.join(alias);
        if link.file_name() == Some(OsStr::new(CARGO_SHIM_NAME)) {
            return Err(RunError::Build("source-built toolchain closure blocked: Cargo must stay guarded".to_string()));
        }
        remove_owned_path(&link)?;
        if c_compiler.as_ref().is_some_and(|compiler| compiler == &target) {
            write_c_compiler_toolchain_alias(&target, &c_compiler_runtime, &link, execution_shell)?;
            continue;
        }
        write_toolchain_alias(&target, &link, execution_shell)?;
    }
    Ok(())
}

fn toolchain_path_aliases(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<BTreeMap<String, PathBuf>, RunError> {
    let mut aliases = BTreeMap::new();
    for member in executable_path_members(manifest) {
        let target = canonicalize_toolchain_path(Path::new(&member.execution_path), member.role)?;
        require_executable(&target)?;
        add_toolchain_alias(&mut aliases, path_file_name(&target)?, &target)?;
        add_toolchain_alias(&mut aliases, safe_toolchain_alias(&member.name)?, &target)?;
        add_role_aliases(&mut aliases, member, &target)?;
    }
    if let Some(c_compiler) = c_compiler_alias_target(manifest)? {
        add_toolchain_alias(&mut aliases, C_COMPILER_ALIAS.to_string(), &c_compiler)?;
    }
    Ok(aliases)
}

fn receipt_bound_c_compiler_route(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<crate::source_toolchain_closure::ReceiptBoundCCompilerRoute, RunError> {
    crate::source_toolchain_closure::select_receipt_bound_c_compiler_route(manifest)
        .map_err(|err| RunError::Build(format!("source-built toolchain closure blocked: {}", err.message())))
}

fn c_compiler_alias_target(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<Option<PathBuf>, RunError> {
    let route = receipt_bound_c_compiler_route(manifest)?;
    Ok(Some(canonicalize_toolchain_path(
        Path::new(&route.execution_path),
        crate::source_toolchain_closure::ToolchainRole::CCompiler,
    )?))
}

fn c_compiler_alias_runtime_inputs(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<CcCompilerAliasRuntimeInputs, RunError> {
    Ok(CcCompilerAliasRuntimeInputs {
        unwind_archive: declared_unwind_archive(manifest)?,
        crt1_object: declared_target_crt1_object(manifest)?,
    })
}

fn declared_unwind_archive(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<Option<PathBuf>, RunError> {
    let members = manifest
        .members
        .iter()
        .filter(|member| member.name == crate::source_toolchain_closure::NATIVE_HOST_LIBUNWIND_NAME)
        .collect::<Vec<_>>();
    match members.as_slice() {
        [member] => Ok(Some(canonicalize_toolchain_path(Path::new(&member.execution_path), member.role)?)),
        [] => Ok(None),
        _many => Err(RunError::Build(
            "source-built toolchain closure blocked: multiple host libunwind runtime members".to_string(),
        )),
    }
}

fn declared_target_crt1_object(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<PathBuf, RunError> {
    let members = manifest
        .members
        .iter()
        .filter(|member| member.name == crate::source_toolchain_closure::NATIVE_TARGET_CRT1_NAME)
        .collect::<Vec<_>>();
    match members.as_slice() {
        [member] => canonicalize_toolchain_path(Path::new(&member.execution_path), member.role),
        [] => Err(RunError::Build("source-built toolchain closure blocked: missing target crt1.o member".to_string())),
        _many => Err(RunError::Build(
            "source-built toolchain closure blocked: multiple target crt1.o members".to_string(),
        )),
    }
}

fn executable_path_members(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Vec<&crate::source_toolchain_closure::ToolchainClosureMember> {
    use crate::source_toolchain_closure::ToolchainRole;
    manifest
        .members
        .iter()
        .filter(|member| {
            matches!(
                member.role,
                ToolchainRole::Rustc
                    | ToolchainRole::Linker
                    | ToolchainRole::CCompiler
                    | ToolchainRole::CxxCompiler
                    | ToolchainRole::PkgConfig
                    | ToolchainRole::NativeHelper
            )
        })
        .collect()
}

fn add_role_aliases(
    aliases: &mut BTreeMap<String, PathBuf>,
    member: &crate::source_toolchain_closure::ToolchainClosureMember,
    target: &Path,
) -> Result<(), RunError> {
    use crate::source_toolchain_closure::ToolchainRole;
    match member.role {
        ToolchainRole::Linker => add_toolchain_alias(aliases, LINKER_ALIAS.to_string(), target),
        ToolchainRole::CCompiler => Ok(()),
        ToolchainRole::PkgConfig => add_toolchain_alias(aliases, PKG_CONFIG_ALIAS.to_string(), target),
        ToolchainRole::NativeHelper if member.name == ARCHIVER_ALIAS => {
            add_toolchain_alias(aliases, ARCHIVER_ALIAS.to_string(), target)
        }
        ToolchainRole::NativeHelper if member.name == RANLIB_ALIAS => {
            add_toolchain_alias(aliases, RANLIB_ALIAS.to_string(), target)
        }
        _ => Ok(()),
    }
}

fn add_toolchain_alias(aliases: &mut BTreeMap<String, PathBuf>, alias: String, target: &Path) -> Result<(), RunError> {
    debug_assert!(!alias.is_empty());
    debug_assert!(Path::new(&alias).components().count() == 1);
    if let Some(existing) = aliases.get(&alias) {
        if existing != target {
            return Err(RunError::Build(format!(
                "source-built toolchain closure blocked: PATH alias {alias} has conflicting targets"
            )));
        }
        return Ok(());
    }
    aliases.insert(alias, target.to_path_buf());
    Ok(())
}

fn canonicalize_toolchain_path(
    path: &Path,
    role: crate::source_toolchain_closure::ToolchainRole,
) -> Result<PathBuf, RunError> {
    fs::canonicalize(path).map_err(|err| {
        RunError::Build(format!(
            "source-built toolchain closure blocked: canonicalize {role:?} {}: {err}",
            path.display()
        ))
    })
}

fn path_file_name(path: &Path) -> Result<String, RunError> {
    let name = path.file_name().and_then(OsStr::to_str).ok_or_else(|| {
        RunError::Build(format!("source-built toolchain closure blocked: invalid tool path {}", path.display()))
    })?;
    safe_toolchain_alias(name)
}

fn safe_toolchain_alias(alias: &str) -> Result<String, RunError> {
    if alias.is_empty() {
        return Err(RunError::Build("source-built toolchain closure blocked: empty PATH alias".to_string()));
    }
    if alias.contains('/') || alias.contains('\\') {
        return Err(RunError::Build(format!("source-built toolchain closure blocked: unsafe PATH alias {alias}")));
    }
    let path = Path::new(alias);
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(value)), None) if value == OsStr::new(alias) => Ok(alias.to_string()),
        _ => Err(RunError::Build(format!("source-built toolchain closure blocked: unsafe PATH alias {alias}"))),
    }
}

fn path_to_string(path: &Path) -> Result<String, RunError> {
    path.to_str().map(ToOwned::to_owned).ok_or_else(|| {
        RunError::Build(format!("source-built toolchain closure blocked: non-UTF-8 path {}", path.display()))
    })
}

#[cfg(unix)]
fn write_toolchain_alias(target: &Path, link: &Path, execution_shell: &Path) -> Result<(), RunError> {
    write_text(link, &format!("#!{}\nexec {} \"$@\"\n", execution_shell.display(), shell_quote(target)))?;
    set_executable(link)
}

#[cfg(unix)]
fn write_c_compiler_toolchain_alias(
    target: &Path,
    runtime_inputs: &CcCompilerAliasRuntimeInputs,
    link: &Path,
    execution_shell: &Path,
) -> Result<(), RunError> {
    debug_assert_ne!(target, link);
    debug_assert!(link.parent().is_some());
    let runtime_dir = link
        .parent()
        .ok_or_else(|| internal(format!("{} has no parent", link.display())))?
        .join(TOOLCHAIN_ALIAS_RUNTIME_DIR);
    fs::create_dir_all(&runtime_dir)
        .map_err(|err| internal(format!("create toolchain alias runtime dir {}: {err}", runtime_dir.display())))?;
    if let Some(unwind_archive) = &runtime_inputs.unwind_archive {
        copy_alias_runtime_file(unwind_archive, &runtime_dir.join(TOOLCHAIN_ALIAS_UNWIND_ARCHIVE), "unwind archive")?;
    }
    copy_alias_runtime_file(
        &runtime_inputs.crt1_object,
        &runtime_dir.join(TOOLCHAIN_ALIAS_CRT1_OBJECT),
        "target crt1.o",
    )?;
    let script = format!(
        "#!{}\nruntime_dir={}\nmapped_args_set=false\nstatic_pie_normalized=false\nresponse_index=0\nrewrite_response_file() {{\n  response_source=$1\n  response_index=$((response_index + 1))\n  response_target=\"$runtime_dir/response-$response_index.rsp\"\n  : > \"$response_target\" || exit 1\n  while IFS= read -r response_arg || [ -n \"$response_arg\" ]; do\n    case \"$response_arg\" in\n      {crt1}|*/{crt1}|{rcrt1}|*/{rcrt1}) response_arg=\"$runtime_dir/{crt1}\" ;;\n      {static_pie}) response_arg=\"{static}\"; static_pie_normalized=true ;;\n    esac\n    printf '%s\\n' \"$response_arg\" >> \"$response_target\" || exit 1\n  done < \"$response_source\" || exit 1\n  mapped_arg=\"@$response_target\"\n}}\nfor arg in \"$@\"; do\n  case \"$arg\" in\n    @*) response_source=${{arg#@}}; if [ -r \"$response_source\" ]; then rewrite_response_file \"$response_source\"; else mapped_arg=\"$arg\"; fi ;;\n    {crt1}|*/{crt1}|{rcrt1}|*/{rcrt1}) mapped_arg=\"$runtime_dir/{crt1}\" ;;\n    {static_pie}) mapped_arg=\"{static}\"; static_pie_normalized=true ;;\n    *) mapped_arg=\"$arg\" ;;\n  esac\n  if [ \"$mapped_args_set\" = false ]; then\n    set -- \"$mapped_arg\"\n    mapped_args_set=true\n  else\n    set -- \"$@\" \"$mapped_arg\"\n  fi\ndone\nif [ \"$static_pie_normalized\" = true ]; then\n  set -- \"$@\" {non_pie}\nfi\nexec {} -L\"$runtime_dir\" \"$@\"\n",
        execution_shell.display(),
        shell_quote(&runtime_dir),
        shell_quote(target),
        crt1 = TOOLCHAIN_ALIAS_CRT1_OBJECT,
        rcrt1 = TOOLCHAIN_ALIAS_STATIC_PIE_CRT_OBJECT,
        static_pie = TOOLCHAIN_ALIAS_STATIC_PIE_FLAG,
        static = TOOLCHAIN_ALIAS_STATIC_FLAG,
        non_pie = TOOLCHAIN_ALIAS_NON_PIE_FLAG
    );
    write_text(link, &script)?;
    set_executable(link)
}

fn copy_alias_runtime_file(source: &Path, destination: &Path, label: &str) -> Result<(), RunError> {
    debug_assert!(!label.is_empty());
    debug_assert_ne!(source, destination);
    remove_owned_path(destination)?;
    fs::copy(source, destination).map_err(|err| {
        internal(format!("copy declared {label} {} -> {}: {err}", source.display(), destination.display()))
    })?;
    Ok(())
}

#[cfg(not(unix))]
fn write_toolchain_alias(target: &Path, link: &Path, _execution_shell: &Path) -> Result<(), RunError> {
    fs::copy(target, link)
        .map_err(|err| internal(format!("copy {} -> {}: {err}", target.display(), link.display())))?;
    set_executable(link)
}

#[cfg(not(unix))]
fn write_c_compiler_toolchain_alias(
    target: &Path,
    _runtime_inputs: &CcCompilerAliasRuntimeInputs,
    link: &Path,
    execution_shell: &Path,
) -> Result<(), RunError> {
    write_toolchain_alias(target, link, execution_shell)
}

fn guarded_path(cargo_path_dir: &Path) -> Result<OsString, RunError> {
    let mut paths = vec![cargo_path_dir.to_path_buf()];
    if let Some(path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&path));
    }
    env::join_paths(paths).map_err(|err| internal(format!("construct guarded PATH: {err}")))
}

fn write_cargo_shim(path: &Path, marker: &Path) -> Result<(), RunError> {
    let parent = path.parent().ok_or_else(|| internal(format!("{} has no parent", path.display())))?;
    fs::create_dir_all(parent).map_err(|err| internal(format!("create {}: {err}", parent.display())))?;
    let mut file = fs::File::create(path).map_err(|err| internal(format!("create {}: {err}", path.display())))?;
    writeln!(file, "#!/bin/sh").map_err(|err| internal(format!("write {}: {err}", path.display())))?;
    writeln!(file, "printf invoked > {}", shell_quote(marker))
        .map_err(|err| internal(format!("write {}: {err}", path.display())))?;
    writeln!(file, "exit 99").map_err(|err| internal(format!("write {}: {err}", path.display())))?;
    set_executable(path)
}

fn shell_quote(path: &Path) -> String {
    let raw = path.as_os_str().to_string_lossy();
    format!("'{}'", raw.replace('\'', "'\\''"))
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions =
        fs::metadata(path).map_err(|err| internal(format!("stat {}: {err}", path.display())))?.permissions();
    permissions.set_mode(CARGO_SHIM_PERMISSIONS);
    fs::set_permissions(path, permissions).map_err(|err| internal(format!("chmod {}: {err}", path.display())))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), RunError> {
    Ok(())
}

fn require_executable(path: &Path) -> Result<(), RunError> {
    let metadata = fs::metadata(path).map_err(|err| internal(format!("stat {}: {err}", path.display())))?;
    if !metadata.is_file() {
        return Err(internal(format!("{} is not a file", path.display())));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & UNIX_EXECUTE_BITS == 0 {
            return Err(internal(format!("{} is not executable", path.display())));
        }
    }
    Ok(())
}

fn safe_path_component(value: &str) -> String {
    debug_assert!(!value.is_empty());
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn blake3_file(path: &Path) -> Result<String, RunError> {
    let bytes = fs::read(path).map_err(|err| internal(format!("read {}: {err}", path.display())))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn write_non_claims(
    out_dir: &Path,
    toolchain_status: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> Result<(), RunError> {
    let text = self_build_non_claims_text(toolchain_status).join("\n");
    write_text(&out_dir.join(NON_CLAIMS_FILE), &format!("{text}\n"))
}

fn self_build_non_claims_text(
    toolchain_status: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> Vec<&'static str> {
    let mut lines = vec![
        "This build claims only bounded Mantle Cargo-free Rust topology execution.",
        "This build does not claim Mantle bootstrap or release reproducibility.",
    ];
    if !toolchain_status.claim {
        lines.push("This build does not claim source-built compiler/toolchain closure provenance.");
    }
    lines.push(
        "This build does not claim full Cargo compatibility, tests, doctests, examples, or general resolver parity.",
    );
    lines
}

fn write_fixed_point_non_claims(
    bundle_dir: &Path,
    toolchain_status: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> Result<(), RunError> {
    let text = fixed_point_non_claims_text(toolchain_status).join("\n");
    write_text(&bundle_dir.join(NON_CLAIMS_FILE), &format!("{text}\n"))
}

fn fixed_point_non_claims_text(
    toolchain_status: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> Vec<&'static str> {
    let mut lines = vec![
        "This proof claims only a bounded Mantle stage1/stage2 fixed point through native Rust topology execution.",
        "This proof does not claim Mantle bootstrap or release reproducibility.",
    ];
    if !toolchain_status.claim {
        lines.push("This proof does not claim source-built compiler/toolchain closure provenance.");
    }
    lines.push(
        "This proof does not claim full Cargo compatibility, tests, doctests, examples, or general resolver parity.",
    );
    lines.push("This proof still depends on recorded non-Rust host linker/tool environment unless a separate toolchain closure manifest is supplied.");
    lines
}

fn self_build_non_claims(
    toolchain_status: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> Vec<&'static str> {
    base_cargo_free_non_claims(toolchain_status)
}

fn fixed_point_non_claims(
    toolchain_status: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> Vec<&'static str> {
    base_cargo_free_non_claims(toolchain_status)
}

fn base_cargo_free_non_claims(
    toolchain_status: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
) -> Vec<&'static str> {
    let mut non_claims = vec!["not-crunch-bootstrap", "not-release-reproducibility"];
    if let Some(non_claim) = toolchain_status.non_claim {
        non_claims.push(non_claim);
    }
    non_claims.push("not-full-cargo-compatibility");
    non_claims
}

fn selected_cargo_free_rustc<'a>(loaded_provider: &'a LoadedRustSourceProvider, fallback_rustc: &'a Path) -> &'a Path {
    loaded_provider.rustc.as_deref().unwrap_or(fallback_rustc)
}

fn selected_execution_shell(provider: &LoadedRustSourceProvider) -> &Path {
    provider
        .source_built_shell
        .as_ref()
        .map_or_else(|| Path::new("/bin/sh"), |shell| shell.execution_path.as_path())
}

fn load_rust_source_provider(
    provider_dir: Option<&Path>,
    closure_manifest: Option<&crate::source_toolchain_closure::ToolchainClosureManifest>,
) -> Result<LoadedRustSourceProvider, RunError> {
    debug_assert_eq!(RUST_SOURCE_PROVIDER_REQUIRED_ROLE_COUNT, 1);
    debug_assert_ne!(RUST_SOURCE_PROVIDER_STATUS_ABSENT, RUST_SOURCE_PROVIDER_STATUS_VALIDATED);
    let Some(provider_dir) = provider_dir else {
        return Ok(LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: None,
            source_built_shell: None,
            source_built_host_tools: Vec::new(),
            source_built_native_artifacts: Vec::new(),
            source_built_action_trust: None,
            toolchain_closure_status: None,
        });
    };
    let provider_dir = fs::canonicalize(provider_dir)
        .map_err(|err| RunError::Build(format!("read --rust-source-provider {}: {err}", provider_dir.display())))?;
    let validation =
        crate::rust_source_provider::validate_materialized_rust_source_provider(&provider_dir).map_err(|err| {
            RunError::Build(format!(
                "source-built Rust provider blocked: --rust-source-provider {} failed validation: {err}",
                provider_dir.display()
            ))
        })?;
    let rustc_path = rust_provider_role_path(
        &provider_dir,
        &validation.metadata,
        crate::source_toolchain_closure::RustProviderRole::Rustc,
    )?;
    require_executable(&rustc_path)?;
    let toolchain_closure_status = crate::source_toolchain_closure::provided_source_built_rust_provider_closure(
        validation.metadata_path.clone(),
        &validation.validation,
    );
    let execution_authority = load_bound_rust_execution_authority(&provider_dir, closure_manifest)?;
    let action_trust_dir = provider_dir.join(crate::rust_source_provider::RUST_PROVIDER_ACTION_EVIDENCE_RELATIVE_PATH);
    let source_built_action_trust = action_trust_dir
        .is_dir()
        .then(|| crate::source_built_rust_provider_action::validate_rust_provider_action_evidence(&action_trust_dir))
        .transpose()
        .map_err(|error| RunError::Build(format!("source-built Rust provider action evidence blocked: {error}")))?;
    Ok(LoadedRustSourceProvider {
        status: validated_rust_source_provider_binding(&provider_dir, &validation, &rustc_path),
        rustc: Some(rustc_path),
        source_built_shell: execution_authority.shell,
        source_built_host_tools: execution_authority.host_tools,
        source_built_native_artifacts: execution_authority.native_artifacts,
        source_built_action_trust,
        toolchain_closure_status: Some(toolchain_closure_status),
    })
}

fn load_bound_rust_execution_authority(
    provider_dir: &Path,
    closure_manifest: Option<&crate::source_toolchain_closure::ToolchainClosureManifest>,
) -> Result<BoundRustExecutionAuthority, RunError> {
    let binding_path = provider_dir.join(crate::full_source_rust_binding_shell::FULL_SOURCE_RUST_BINDING_RELATIVE_PATH);
    if !binding_path.is_file() {
        return Ok(BoundRustExecutionAuthority {
            shell: None,
            host_tools: Vec::new(),
            native_artifacts: Vec::new(),
        });
    }
    let bytes = fs::read(&binding_path)
        .map_err(|err| RunError::Build(format!("read full-source Rust binding {}: {err}", binding_path.display())))?;
    let binding =
        serde_json::from_slice::<crate::full_source_rust_binding::FullSourceRustProviderBindingReceipt>(&bytes)
            .map_err(|err| {
                RunError::Build(format!("parse full-source Rust binding {}: {err}", binding_path.display()))
            })?;
    validate_bound_rust_execution_shell_policy(&binding)?;
    for tool in &binding.host_tools {
        validate_bound_rust_host_tool(tool)?;
    }
    let closure_manifest = closure_manifest.ok_or_else(|| {
        RunError::Build(
            "full-source Rust binding requires an explicit validated toolchain closure to resolve native artifacts"
                .to_string(),
        )
    })?;
    let native_provider_root = crate::source_toolchain_closure::source_built_native_provider_root(closure_manifest)
        .map_err(|error| {
            RunError::Build(format!(
                "resolving full-source Rust native-provider root from the validated closure failed: {error}"
            ))
        })?;
    let native_artifacts = rebase_bound_native_artifacts(&native_provider_root, &binding.native_artifacts)?;
    let candidates = binding
        .host_tools
        .iter()
        .filter(|tool| tool.role == crate::full_source_rust_binding::FullSourceRustHostToolRole::Busybox)
        .collect::<Vec<_>>();
    let [busybox] = candidates.as_slice() else {
        return Err(RunError::Build(format!(
            "full-source Rust binding must contain one BusyBox host tool, found {}",
            candidates.len()
        )));
    };
    let shell = bound_rust_execution_shell(busybox)?;
    let host_tools = binding.host_tools;
    assert!(!host_tools.is_empty());
    assert!(!native_artifacts.is_empty());
    assert!(host_tools.iter().all(|tool| Path::new(&tool.path).is_absolute()));
    Ok(BoundRustExecutionAuthority {
        shell: Some(shell),
        host_tools,
        native_artifacts,
    })
}

fn validate_bound_rust_execution_shell_policy(
    binding: &crate::full_source_rust_binding::FullSourceRustProviderBindingReceipt,
) -> Result<(), RunError> {
    if binding.ambient_tool_discovery || !binding.fallback_events.is_empty() || !binding.seed_exceptions.is_empty() {
        return Err(RunError::Build(
            "full-source Rust binding cannot authorize a shell with ambient discovery, fallback, or seed exceptions"
                .to_string(),
        ));
    }
    Ok(())
}

fn bound_rust_execution_shell(
    busybox: &crate::full_source_rust_binding::FullSourceRustHostToolBinding,
) -> Result<BoundRustExecutionShell, RunError> {
    let busybox_path = Path::new(&busybox.path);
    if !busybox_path.is_absolute() {
        return Err(RunError::Build(format!("full-source BusyBox path is not absolute: {}", busybox_path.display())));
    }
    validate_bound_rust_host_tool(busybox)?;
    let shell_path = busybox_path
        .parent()
        .ok_or_else(|| RunError::Build("full-source BusyBox path has no parent".to_string()))?
        .join("sh");
    require_executable(&shell_path)?;
    let resolved_shell = fs::canonicalize(&shell_path)
        .map_err(|err| RunError::Build(format!("resolve full-source shell {}: {err}", shell_path.display())))?;
    if resolved_shell
        != fs::canonicalize(busybox_path)
            .map_err(|err| RunError::Build(format!("resolve full-source BusyBox {}: {err}", busybox_path.display())))?
    {
        return Err(RunError::Build(format!(
            "full-source shell {} does not resolve to bound BusyBox {}",
            shell_path.display(),
            busybox_path.display()
        )));
    }
    Ok(BoundRustExecutionShell {
        execution_path: shell_path,
        content_digest_blake3: busybox.content_digest_blake3.clone(),
        source_id: busybox.source_id.clone(),
        construction_receipt_digest_blake3: busybox.construction_receipt_digest_blake3.clone(),
    })
}

fn bound_rustc_runtime(
    artifacts: &[crate::full_source_rust_binding::FullSourceNativeArtifactBinding],
) -> Result<Option<BoundRustcRuntime>, RunError> {
    use crate::full_source_rust_binding::FullSourceNativeArtifactRole;

    if artifacts.is_empty() {
        return Ok(None);
    }
    let cxx_runtime = unique_bound_runtime_artifact(
        artifacts,
        FullSourceNativeArtifactRole::Libstdcxx,
        RUSTC_LIBSTDCXX_RUNTIME_FILE,
    )?;
    let loader = unique_bound_runtime_artifact(
        artifacts,
        FullSourceNativeArtifactRole::DynamicLinker,
        RUSTC_DYNAMIC_LOADER_FILE,
    )?;
    let cxx_runtime_path = validated_bound_runtime_file(cxx_runtime, "Rust C++ runtime")?;
    let loader_path = validated_bound_runtime_file(loader, "Rust dynamic loader")?;
    require_executable(&loader_path)?;
    let library_dir = cxx_runtime_path
        .parent()
        .ok_or_else(|| RunError::Build("full-source Rust C++ runtime has no parent directory".to_string()))?;
    if loader_path.parent() != Some(library_dir) {
        return Err(RunError::Build("Rust C++ runtime and dynamic loader use different bound directories".to_string()));
    }
    let soname_path = library_dir.join(RUSTC_LIBSTDCXX_SONAME);
    let resolved_soname = fs::canonicalize(&soname_path).map_err(|error| {
        RunError::Build(format!("resolve Rust C++ runtime soname {}: {error}", soname_path.display()))
    })?;
    if resolved_soname != cxx_runtime_path {
        return Err(RunError::Build(format!(
            "Rust C++ runtime soname {} does not resolve to bound artifact {}",
            soname_path.display(),
            cxx_runtime_path.display()
        )));
    }
    assert!(library_dir.is_absolute());
    assert!(library_dir.is_dir());
    Ok(Some(BoundRustcRuntime {
        library_dir: library_dir.to_path_buf(),
        loader: loader_path,
    }))
}

fn unique_bound_runtime_artifact<'a>(
    artifacts: &'a [crate::full_source_rust_binding::FullSourceNativeArtifactBinding],
    role: crate::full_source_rust_binding::FullSourceNativeArtifactRole,
    file_name: &str,
) -> Result<&'a crate::full_source_rust_binding::FullSourceNativeArtifactBinding, RunError> {
    let candidates = artifacts
        .iter()
        .filter(|artifact| artifact.role == role)
        .filter(|artifact| Path::new(&artifact.path).file_name() == Some(OsStr::new(file_name)))
        .collect::<Vec<_>>();
    let [artifact] = candidates.as_slice() else {
        return Err(RunError::Build(format!(
            "full-source Rust runtime requires one {file_name} artifact, found {}",
            candidates.len()
        )));
    };
    Ok(*artifact)
}

fn validated_bound_runtime_file(
    artifact: &crate::full_source_rust_binding::FullSourceNativeArtifactBinding,
    label: &str,
) -> Result<PathBuf, RunError> {
    let path = Path::new(&artifact.path);
    if !path.is_absolute() {
        return Err(RunError::Build(format!("full-source {label} is not absolute: {}", path.display())));
    }
    if !path.is_file() {
        return Err(RunError::Build(format!("full-source {label} is unavailable: {}", path.display())));
    }
    let observed = crate::protected_exec::blake3_file_hex(path)
        .map_err(|error| RunError::Build(format!("hash full-source {label}: {error}")))?;
    if observed != artifact.content_digest_blake3 {
        return Err(RunError::Build(format!(
            "full-source {label} digest mismatch: expected {}, got {observed}",
            artifact.content_digest_blake3
        )));
    }
    fs::canonicalize(path).map_err(|error| RunError::Build(format!("resolve full-source {label}: {error}")))
}

fn rebase_bound_native_artifacts(
    native_provider_root: &Path,
    artifacts: &[crate::full_source_rust_binding::FullSourceNativeArtifactBinding],
) -> Result<Vec<crate::full_source_rust_binding::FullSourceNativeArtifactBinding>, RunError> {
    if !native_provider_root.is_absolute() {
        return Err(RunError::Build(format!(
            "full-source native-provider root is not absolute: {}",
            native_provider_root.display()
        )));
    }
    if !native_provider_root.is_dir() {
        return Err(RunError::Build(format!(
            "full-source native-provider root is unavailable: {}",
            native_provider_root.display()
        )));
    }
    let mut rebound = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        let relative_path = require_provider_relative_native_artifact_path(artifact)?;
        let mut runtime_artifact = artifact.clone();
        runtime_artifact.path = native_provider_root.join(relative_path).display().to_string();
        validate_bound_native_artifact(&runtime_artifact)?;
        assert!(Path::new(&runtime_artifact.path).is_absolute());
        rebound.push(runtime_artifact);
    }
    assert_eq!(rebound.len(), artifacts.len());
    Ok(rebound)
}

fn require_provider_relative_native_artifact_path(
    artifact: &crate::full_source_rust_binding::FullSourceNativeArtifactBinding,
) -> Result<&Path, RunError> {
    let path = Path::new(&artifact.path);
    if artifact.path.is_empty() {
        return Err(invalid_provider_relative_native_artifact_path(artifact));
    }
    if path.is_absolute() {
        return Err(invalid_provider_relative_native_artifact_path(artifact));
    }
    for component in path.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(invalid_provider_relative_native_artifact_path(artifact));
        }
    }
    assert!(!artifact.path.is_empty());
    assert!(!path.is_absolute());
    Ok(path)
}

fn invalid_provider_relative_native_artifact_path(
    artifact: &crate::full_source_rust_binding::FullSourceNativeArtifactBinding,
) -> RunError {
    RunError::Build(format!(
        "full-source native artifact path must stay provider-relative: role={:?} path={}",
        artifact.role, artifact.path
    ))
}

fn validate_bound_native_artifact(
    artifact: &crate::full_source_rust_binding::FullSourceNativeArtifactBinding,
) -> Result<(), RunError> {
    let path = Path::new(&artifact.path);
    if !path.is_absolute() || !path.is_file() {
        return Err(RunError::Build(format!("full-source native artifact is unavailable: {}", path.display())));
    }
    if full_source_native_artifact_is_executable(artifact.role) {
        require_executable(path)?;
    }
    let observed = crate::protected_exec::blake3_file_hex(path)
        .map_err(|error| RunError::Build(format!("hash full-source native artifact {}: {error}", path.display())))?;
    if observed != artifact.content_digest_blake3 {
        return Err(RunError::Build(format!(
            "full-source native artifact digest mismatch: expected {}, got {observed}",
            artifact.content_digest_blake3
        )));
    }
    assert_eq!(observed.len(), blake3::OUT_LEN * 2);
    assert!(path.is_file());
    Ok(())
}

fn full_source_native_artifact_is_executable(
    role: crate::full_source_rust_binding::FullSourceNativeArtifactRole,
) -> bool {
    use crate::full_source_rust_binding::FullSourceNativeArtifactRole;
    matches!(
        role,
        FullSourceNativeArtifactRole::CCompiler
            | FullSourceNativeArtifactRole::CxxCompiler
            | FullSourceNativeArtifactRole::Preprocessor
            | FullSourceNativeArtifactRole::CompilerInternal
            | FullSourceNativeArtifactRole::Assembler
            | FullSourceNativeArtifactRole::Linker
            | FullSourceNativeArtifactRole::ArchiveTool
            | FullSourceNativeArtifactRole::Ranlib
            | FullSourceNativeArtifactRole::SymbolTool
            | FullSourceNativeArtifactRole::ObjectCopy
            | FullSourceNativeArtifactRole::ObjectDump
            | FullSourceNativeArtifactRole::ObjectFormat
            | FullSourceNativeArtifactRole::DynamicLinker
    )
}

fn validate_bound_rust_host_tool(
    tool: &crate::full_source_rust_binding::FullSourceRustHostToolBinding,
) -> Result<(), RunError> {
    let tool_path = Path::new(&tool.path);
    if !tool_path.is_absolute() {
        return Err(RunError::Build(format!("full-source host-tool path is not absolute: {}", tool_path.display())));
    }
    require_executable(tool_path)?;
    let observed = crate::protected_exec::blake3_file_hex(tool_path)
        .map_err(|err| RunError::Build(format!("hash full-source host tool {}: {err}", tool_path.display())))?;
    if observed != tool.content_digest_blake3 {
        return Err(RunError::Build(format!(
            "full-source host-tool digest mismatch: expected {}, got {observed}",
            tool.content_digest_blake3
        )));
    }
    validate_bound_host_tool_receipt(tool)
}

fn validate_bound_host_tool_receipt(
    tool: &crate::full_source_rust_binding::FullSourceRustHostToolBinding,
) -> Result<(), RunError> {
    let receipt_path = Path::new(&tool.construction_receipt_path);
    if !receipt_path.is_absolute() || !receipt_path.is_file() {
        return Err(RunError::Build(format!(
            "full-source host-tool construction receipt is unavailable: {}",
            receipt_path.display()
        )));
    }
    let observed = crate::protected_exec::blake3_file_hex(receipt_path).map_err(|err| {
        RunError::Build(format!("hash host-tool construction receipt {}: {err}", receipt_path.display()))
    })?;
    if observed != tool.construction_receipt_digest_blake3 {
        return Err(RunError::Build(format!(
            "full-source host-tool construction receipt digest mismatch: expected {}, got {observed}",
            tool.construction_receipt_digest_blake3
        )));
    }
    Ok(())
}

fn absent_rust_source_provider_binding() -> RustSourceProviderBindingStatus {
    RustSourceProviderBindingStatus {
        schema: RUST_SOURCE_PROVIDER_BINDING_SCHEMA,
        status: RUST_SOURCE_PROVIDER_STATUS_ABSENT.to_string(),
        provider_dir: None,
        metadata_path: None,
        metadata_digest_blake3: None,
        policy_digest_blake3: None,
        host_triple: None,
        target_triple: None,
        artifact_count: None,
        source_count: None,
        receipt_count: None,
        rustc_path: None,
    }
}

fn validated_rust_source_provider_binding(
    provider_dir: &Path,
    validation: &crate::rust_source_provider::RustSourceProviderDirectoryValidation,
    rustc_path: &Path,
) -> RustSourceProviderBindingStatus {
    RustSourceProviderBindingStatus {
        schema: RUST_SOURCE_PROVIDER_BINDING_SCHEMA,
        status: RUST_SOURCE_PROVIDER_STATUS_VALIDATED.to_string(),
        provider_dir: Some(provider_dir.to_path_buf()),
        metadata_path: Some(validation.metadata_path.clone()),
        metadata_digest_blake3: Some(validation.metadata_digest_blake3.clone()),
        policy_digest_blake3: Some(validation.validation.policy_digest_blake3.clone()),
        host_triple: Some(validation.metadata.host_triple.clone()),
        target_triple: Some(validation.metadata.target_triple.clone()),
        artifact_count: Some(validation.validation.artifact_count),
        source_count: Some(validation.validation.source_count),
        receipt_count: Some(validation.validation.receipt_count),
        rustc_path: Some(rustc_path.to_path_buf()),
    }
}

fn rust_provider_role_path(
    provider_dir: &Path,
    metadata: &crate::source_toolchain_closure::RustSourceProviderMetadata,
    role: crate::source_toolchain_closure::RustProviderRole,
) -> Result<PathBuf, RunError> {
    let artifacts = metadata.artifacts.iter().filter(|artifact| artifact.role == role).collect::<Vec<_>>();
    if artifacts.len() != RUST_SOURCE_PROVIDER_REQUIRED_ROLE_COUNT {
        return Err(RunError::Build(format!(
            "source-built Rust provider blocked: expected one {role:?} artifact, found {}",
            artifacts.len()
        )));
    }
    Ok(provider_dir.join(&artifacts[0].path))
}

fn load_source_built_toolchain_closure(manifest_path: Option<&Path>) -> Result<LoadedToolchainClosure, RunError> {
    debug_assert!(!crate::source_toolchain_closure::SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA.is_empty());
    debug_assert_ne!(TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD, SELECTED_C_COMPILER_FIELD);
    let Some(manifest_path) = manifest_path else {
        return Ok(LoadedToolchainClosure {
            status: crate::source_toolchain_closure::absent_source_built_toolchain_closure(),
            manifest_path: None,
            manifest: None,
        });
    };
    let manifest_bytes = fs::read(manifest_path)
        .map_err(|err| RunError::Build(format!("read --toolchain-closure {}: {err}", manifest_path.display())))?;
    let manifest = serde_json::from_slice::<crate::source_toolchain_closure::ToolchainClosureManifest>(&manifest_bytes)
        .map_err(|err| RunError::Build(format!("parse --toolchain-closure {}: {err}", manifest_path.display())))?;
    let validation =
        crate::source_toolchain_closure::validate_toolchain_closure_manifest(&manifest).map_err(|err| {
            RunError::Build(format!("invalid --toolchain-closure {}: {}", manifest_path.display(), err.message()))
        })?;
    Ok(LoadedToolchainClosure {
        status: crate::source_toolchain_closure::validated_source_built_toolchain_closure(
            manifest_path.to_path_buf(),
            &validation,
        ),
        manifest_path: Some(manifest_path.to_path_buf()),
        manifest: Some(manifest),
    })
}

fn write_fixed_point_preflight(
    plan: &FixedPointPlan,
    compatibility: &RustcCompatibilitySummary,
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: &RustSourceProviderBindingStatus,
) -> Result<(), RunError> {
    let value = json!({
        "schema": plan.schema,
        "root": plan.root,
        "bundle_dir": BUNDLE_ROOT_RELATIVE_PATH,
        "shared_execution_dir": bundle_local_path(&plan.bundle_dir, &plan.shared_execution_dir),
        "rustc_compatibility": compatibility,
        "source_built_toolchain_closure": toolchain_closure,
        "rust_source_provider": rust_source_provider,
    });
    let bytes = serde_json::to_vec_pretty(&value).map_err(|err| internal(format!("serialize preflight: {err}")))?;
    write_bytes(&plan.preflight_path, &bytes)
}

fn write_summary<T: Serialize>(path: &Path, summary: &T) -> Result<(), RunError> {
    let value = json!(summary);
    let bytes = serde_json::to_vec_pretty(&value).map_err(|err| internal(format!("serialize summary: {err}")))?;
    write_bytes(path, &bytes)
}

fn print_summary(summary: &SelfBuildSummary, json_mode: bool) -> Result<(), RunError> {
    if json_mode {
        let rendered = serde_json::to_string(summary).map_err(|err| internal(format!("render summary: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("Cargo-free self-build: {}", summary.status);
    if let Some(binary) = &summary.binary {
        println!("binary: {}", binary.display());
    }
    if let Some(digest) = &summary.binary_blake3 {
        println!("binary_blake3: {digest}");
    }
    println!("receipt: {}", summary.receipt.display());
    Ok(())
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    fs::write(path, bytes).map_err(|err| internal(format!("write {}: {err}", path.display())))
}

fn write_text(path: &Path, text: &str) -> Result<(), RunError> {
    write_bytes(path, text.as_bytes())
}

fn status_text(code: Option<i32>) -> String {
    match code {
        Some(code) => format!("{code}\n"),
        None => format!("{SIGNAL_STATUS_TEXT}\n"),
    }
}

fn internal(message: String) -> RunError {
    RunError::Internal(message)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    const FIXED_POINT_TEST_DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const FIXED_POINT_TEST_DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const MUSL_TARGET_GCC_ALIAS: &str = "x86_64-linux-musl-gcc";
    const FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT: usize = 4;
    const FIXED_POINT_TEST_SEED_EXCEPTION_COUNT: usize = 1;
    const FIXED_POINT_TEST_UNIT_COUNT: u64 = 2;
    const RUST_PROVIDER_HOST_TRIPLE: &str = "x86_64-unknown-linux-gnu";
    const RUST_PROVIDER_TARGET_TRIPLE: &str = "x86_64-unknown-linux-musl";
    const RUST_PROVIDER_SOURCE_ID: &str = "rust-src";
    const RUST_PROVIDER_RECEIPT_ID: &str = "build-receipt";
    const RUST_PROVIDER_BUILD_RECIPE: &str = "bootstrap/rust-source.ncl";
    const RUST_PROVIDER_STAGE_PROGRAM: &str = "mantle-rust-source-stage";
    const RUST_PROVIDER_RUSTC_PATH: &str = "bin/rustc";
    const RUST_PROVIDER_CARGO_PATH: &str = "bin/cargo";
    const RUST_PROVIDER_HOST_RUSTLIB_PATH: &str = "lib/rustlib/x86_64-unknown-linux-gnu/lib/libstd.rlib";
    const RUST_PROVIDER_TARGET_RUSTLIB_PATH: &str = "lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib";
    const RUST_PROVIDER_RECEIPT_PATH: &str = "share/mantle-rust-provider/receipts/build.json";
    const RUST_PROVIDER_METADATA_PATH: &str = crate::source_toolchain_closure::RUST_SOURCE_PROVIDER_METADATA_PATH;

    #[test]
    fn safe_path_component_replaces_unsafe_path_bytes() {
        assert_eq!(safe_path_component("unit:with/slash"), "unit_with_slash");
        assert_eq!(safe_path_component("unit.with-dash_ok"), "unit.with-dash_ok");
    }

    #[test]
    fn child_blocker_fails_when_cargo_guard_was_invoked() {
        let blocker = child_blocker(Some(SUCCESS_EXIT_CODE), SUCCESS_STATUS, false).unwrap();
        assert_eq!(blocker, "cargo guard was invoked");
    }

    #[test]
    fn child_blocker_accepts_successful_guarded_execution() {
        assert!(child_blocker(Some(SUCCESS_EXIT_CODE), SUCCESS_STATUS, true).is_none());
    }

    #[test]
    fn blocked_topology_classifier_names_root_unit_package_role_triple_and_predecessor() {
        let receipt = json!({
            "topology_execution": {
                "execution_status": BLOCKED_STATUS,
                "unit_executions": [
                    {
                        "unit_id": "native:dep:lib:target",
                        "package_id": "registry+https://github.com/rust-lang/crates.io-index#dep@1.0.0",
                        "execution_kind": "target",
                        "selected_triple": "x86_64-unknown-linux-musl",
                        "target_kind": "lib",
                        "execution_status": SUCCESS_STATUS
                    },
                    {
                        "unit_id": "native:aws-lc-sys:build:host",
                        "package_id": "registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1",
                        "execution_kind": "host",
                        "selected_triple": "x86_64-unknown-linux-gnu",
                        "target_kind": "custom-build",
                        "execution_status": BLOCKED_STATUS,
                        "blocker": { "class": "missing-build-script-metadata-producer" }
                    }
                ]
            }
        });

        let diagnostic = classify_blocked_topology_receipt(Some(&receipt), BLOCKED_STATUS);
        let summary = blocked_topology_summary(&diagnostic);
        let (blocker, child_diagnostic) =
            child_blocker_with_diagnostic(Some(SUCCESS_EXIT_CODE), BLOCKED_STATUS, true, Some(&receipt));

        assert_eq!(diagnostic.classification, "blocked-unit");
        assert_eq!(diagnostic.root_blocked_unit.as_deref(), Some("native:aws-lc-sys:build:host"));
        assert_eq!(
            diagnostic.package_id.as_deref(),
            Some("registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1")
        );
        assert_eq!(diagnostic.execution_role.as_deref(), Some("host"));
        assert_eq!(diagnostic.selected_triple.as_deref(), Some("x86_64-unknown-linux-gnu"));
        assert_eq!(diagnostic.target_kind.as_deref(), Some("custom-build"));
        assert_eq!(diagnostic.predecessor_status.as_deref(), Some(SUCCESS_STATUS));
        assert_eq!(diagnostic.blocker_class.as_deref(), Some("missing-build-script-metadata-producer"));
        assert!(summary.contains("unit=native:aws-lc-sys:build:host"));
        assert!(summary.contains("package=registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1"));
        assert!(summary.contains("role=host"));
        assert!(summary.contains("triple=x86_64-unknown-linux-gnu"));
        assert_eq!(child_diagnostic, Some(diagnostic));
        assert_eq!(blocker.as_deref(), Some(summary.as_str()));
    }

    #[test]
    fn blocked_topology_classifier_surfaces_nested_planner_blocker() {
        let receipt = json!({
            "rust_plan": {
                "native_registry_source_planning": {
                    "blockers": [{
                        "package_id": "registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3",
                        "class": "vendor-checksum-mismatch",
                        "message": "vendor package checksum does not match Cargo.lock checksum material"
                    }]
                }
            },
            "topology_execution": {
                "execution_status": BLOCKED_STATUS,
                "blocker": {
                    "class": "native-host-unit-graph-blocked",
                    "message": "native_host_unit_graph_planning is not ready"
                },
                "unit_executions": []
            }
        });

        let diagnostic = classify_blocked_topology_receipt(Some(&receipt), BLOCKED_STATUS);
        let summary = blocked_topology_summary(&diagnostic);

        assert_eq!(diagnostic.classification, "topology-level-blocker");
        assert_eq!(diagnostic.blocker_class.as_deref(), Some("vendor-checksum-mismatch"));
        assert_eq!(
            diagnostic.package_id.as_deref(),
            Some("registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3")
        );
        assert!(diagnostic.diagnostic.contains("native-host-unit-graph-blocked"));
        assert!(diagnostic.diagnostic.contains("/rust_plan/native_registry_source_planning/blockers/0"));
        assert!(summary.contains("blocker_class=vendor-checksum-mismatch"));
        assert!(
            summary.contains("package=registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3")
        );
    }

    #[test]
    fn blocked_topology_classifier_fails_closed_on_missing_identity() {
        let receipt = json!({
            "topology_execution": {
                "execution_status": BLOCKED_STATUS,
                "unit_executions": [{
                    "unit_id": "native:missing-package",
                    "execution_kind": "target",
                    "selected_triple": "x86_64-unknown-linux-musl",
                    "target_kind": "lib",
                    "execution_status": BLOCKED_STATUS,
                    "blocker": { "class": "missing-dependency-producer" }
                }]
            }
        });

        let diagnostic = classify_blocked_topology_receipt(Some(&receipt), BLOCKED_STATUS);
        let summary = blocked_topology_summary(&diagnostic);

        assert_eq!(diagnostic.classification, "malformed-blocked-unit");
        assert!(diagnostic.diagnostic.contains("package_id"));
        assert!(diagnostic.root_blocked_unit.is_none());
        assert!(diagnostic.blocker_class.is_none());
        assert!(summary.contains("classification=malformed-blocked-unit"));
        assert!(!summary.contains("fixed_point=true"));
    }

    #[test]
    fn blocked_topology_classifier_fails_closed_on_ambiguous_units() {
        let receipt = json!({
            "topology_execution": {
                "execution_status": BLOCKED_STATUS,
                "unit_executions": [
                    {
                        "unit_id": "unit-a",
                        "package_id": "package-a",
                        "execution_kind": "target",
                        "selected_triple": "x86_64-unknown-linux-musl",
                        "target_kind": "lib",
                        "execution_status": BLOCKED_STATUS,
                        "blocker": { "class": "missing-a" }
                    },
                    {
                        "unit_id": "unit-b",
                        "package_id": "package-b",
                        "execution_kind": "host",
                        "selected_triple": "x86_64-unknown-linux-gnu",
                        "target_kind": "custom-build",
                        "execution_status": "failed",
                        "blocker": { "class": "missing-b" }
                    }
                ]
            }
        });

        let diagnostic = classify_blocked_topology_receipt(Some(&receipt), BLOCKED_STATUS);

        assert_eq!(diagnostic.classification, "ambiguous-blocked-units");
        assert!(diagnostic.diagnostic.contains("2 non-success"));
        assert!(diagnostic.root_blocked_unit.is_none());
        assert!(diagnostic.blocker_class.is_none());
    }

    #[test]
    fn mantle_unit_from_receipt_requires_source_digest() {
        let receipt = json!({
            "topology_execution": {
                "unit_executions": [{
                    "unit_id": "unit-1",
                    "target_name": MANTLE_TARGET_NAME,
                    "target_kind": MANTLE_TARGET_KIND,
                    "execution_status": SUCCESS_STATUS
                }]
            }
        });

        let err = mantle_unit_from_receipt(&receipt).unwrap_err();
        assert_eq!(err.message(), "mantle bin unit lacks source_digest");
    }

    #[test]
    fn mantle_unit_from_receipt_rejects_null_source_digest() {
        let receipt = json!({
            "topology_execution": {
                "unit_executions": [{
                    "unit_id": "unit-1",
                    "target_name": MANTLE_TARGET_NAME,
                    "target_kind": MANTLE_TARGET_KIND,
                    "execution_status": SUCCESS_STATUS,
                    "source_digest": null
                }]
            }
        });

        let err = mantle_unit_from_receipt(&receipt).unwrap_err();
        assert_eq!(err.message(), "mantle bin unit lacks source_digest");
    }

    #[test]
    fn fixed_point_plan_describes_stage_paths_and_commands() {
        let root = Path::new("/repo/mantle");
        let out_dir = Path::new("/tmp/mantle-fixed-point");
        let rustc = Path::new("/toolchain/bin/rustc");

        let plan = plan_fixed_point_paths(root, out_dir, rustc, &[]).unwrap();
        let stage1 = &plan.stages[FIXED_POINT_STAGE1_INDEX];
        let stage2 = &plan.stages[FIXED_POINT_STAGE2_INDEX];

        assert_eq!(plan.schema, FIXED_POINT_SCHEMA);
        assert_eq!(plan.root, root);
        assert_eq!(plan.bundle_dir, out_dir);
        assert_eq!(plan.shared_execution_dir, out_dir.join(EXECUTION_DIR));
        assert_eq!(plan.preflight_path, out_dir.join(PRE_FLIGHT_FILE));
        assert_eq!(plan.meta_path, out_dir.join(META_FILE));
        assert_eq!(plan.non_claims_path, out_dir.join(NON_CLAIMS_FILE));
        assert_eq!(plan.stages.len(), FIXED_POINT_STAGE_COUNT);
        assert_eq!(stage1.name, STAGE1_DIR);
        assert_eq!(stage2.name, STAGE2_DIR);
        assert_ne!(stage1.stage_dir, stage2.stage_dir);
        assert_eq!(stage1.execution_dir, plan.shared_execution_dir);
        assert_eq!(stage2.execution_dir, plan.shared_execution_dir);
        assert_eq!(stage1.binary_path, out_dir.join(STAGE1_DIR).join(PRODUCED_MANTLE_FILE));
        assert_eq!(stage2.binary_path, out_dir.join(STAGE2_DIR).join(PRODUCED_MANTLE_FILE));
        assert_eq!(stage1.command.mantle_binary, FixedPointMantleBinary::Host);
        assert_eq!(stage2.command.mantle_binary, FixedPointMantleBinary::StageOutput {
            stage_name: STAGE1_DIR,
            path: stage1.binary_path.clone(),
        });
        assert_eq!(stage1.command.current_dir, root);
        assert_eq!(stage1.command.cargo_env_value, stage1.path_cargo_shim);
        assert_eq!(stage1.command.path_guard_dir, stage1.guard_path_dir);
        assert_eq!(
            stage1.command.args,
            rust_plan_args(root, &stage1.explicit_cargo_shim, rustc, &[], &plan.shared_execution_dir)
        );
        assert!(stage1.command.args.contains(&OsString::from(DETERMINISTIC_RELEASE_PATHS_FLAG)));
        assert!(stage2.command.args.contains(&OsString::from(DETERMINISTIC_RELEASE_PATHS_FLAG)));
    }

    #[test]
    fn fixed_point_plan_threads_targets_into_stage_commands() {
        let root = Path::new("/repo/mantle");
        let out_dir = Path::new("/tmp/mantle-fixed-point");
        let rustc = Path::new("/toolchain/bin/rustc");
        let targets = vec!["x86_64-unknown-linux-musl".to_string()];

        let plan = plan_fixed_point_paths(root, out_dir, rustc, &targets).unwrap();
        let stage1_args = &plan.stages[FIXED_POINT_STAGE1_INDEX].command.args;
        let stage2_args = &plan.stages[FIXED_POINT_STAGE2_INDEX].command.args;

        assert!(has_ordered_os_arg_pair(stage1_args, "--target", "x86_64-unknown-linux-musl"));
        assert!(has_ordered_os_arg_pair(stage2_args, "--target", "x86_64-unknown-linux-musl"));
        assert!(!has_ordered_os_arg_pair(stage1_args, "--target", "x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn fixed_point_plan_rejects_bundle_inside_source_root() {
        let root = Path::new("/repo/mantle");
        let out_dir = Path::new("/repo/mantle/target/proof");
        let rustc = Path::new("/toolchain/bin/rustc");

        let err = plan_fixed_point_paths(root, out_dir, rustc, &[]).unwrap_err();
        assert!(err.message().contains("inside source root"));
        assert!(err.message().contains("choose /tmp"));
    }

    #[test]
    fn rustc_wrapper_script_strips_link_self_contained_runtime_args() {
        let script = rustc_wrapper_script(Path::new("/toolchain/bin/rustc"));

        assert!(script.starts_with("#!/bin/sh\n"));
        assert!(!script.contains("/usr/bin/env"));
        assert!(script.contains(LINK_SELF_CONTAINED_PROBE_ARG));
        assert!(script.contains(LINK_SELF_CONTAINED_JOINED_ARG));
        assert!(script.contains(RUSTC_BOOTSTRAP_ENV));
        assert!(script.contains(REAL_RUSTC_ENV));
        assert!(script.contains("set -- \"$@\" \"$arg\""));
        assert!(script.contains("exec"));
    }

    #[cfg(unix)]
    #[test]
    fn rustc_wrapper_runs_with_posix_shell_and_preserves_remaining_args() {
        let dir = tempfile::TempDir::new().unwrap();
        let real_rustc = dir.path().join("real-rustc");
        let wrapper = dir.path().join("rustc-wrapper");
        let observed = dir.path().join("observed-args.txt");
        write_fake_executable(
            &real_rustc,
            &format!(
                "#!/bin/sh\nprintf 'bootstrap=%s\\n' \"${{{RUSTC_BOOTSTRAP_ENV}-}}\" > {}\nprintf 'arg=%s\\n' \"$@\" >> {}\n",
                shell_quote(&observed),
                shell_quote(&observed)
            ),
        );
        write_rustc_wrapper(&wrapper, &real_rustc).unwrap();

        let status = Command::new(&wrapper)
            .args([
                "--crate-name",
                "demo",
                "-C",
                LINK_SELF_CONTAINED_PROBE_ARG,
                "source path.rs",
                LINK_SELF_CONTAINED_JOINED_ARG,
                "--emit",
                "link",
            ])
            .status()
            .unwrap();
        let observed = fs::read_to_string(observed).unwrap();

        assert!(status.success());
        assert_eq!(observed, "bootstrap=1\narg=--crate-name\narg=demo\narg=source path.rs\narg=--emit\narg=link\n");
        assert!(!observed.contains(LINK_SELF_CONTAINED_PROBE_ARG));
        assert!(!observed.contains(LINK_SELF_CONTAINED_JOINED_ARG));
    }

    #[test]
    fn fixed_point_plan_rejects_relative_source_root() {
        let err =
            plan_fixed_point_paths(Path::new("repo"), Path::new("/tmp/proof"), Path::new("rustc"), &[]).unwrap_err();
        assert!(err.message().contains("absolute source root"));
        assert!(err.message().contains("repo"));
    }

    #[test]
    fn fixed_point_status_accepts_matching_policy_digest() {
        let stage1 = fixed_point_stage_run(STAGE1_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_B));
        let stage2 = fixed_point_stage_run(STAGE2_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_B));

        let status = fixed_point_status(&stage1, &stage2).unwrap();
        let blocker = fixed_point_blocker(&stage1, Some(&stage2), status);

        assert_eq!(status, SUCCESS_STATUS);
        assert!(blocker.is_none());
    }

    #[test]
    fn fixed_point_status_rejects_policy_digest_mismatch_before_success() {
        let stage1 = fixed_point_stage_run(STAGE1_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_A));
        let stage2 = fixed_point_stage_run(STAGE2_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_B));

        let status = fixed_point_status(&stage1, &stage2).unwrap();
        let blocker = fixed_point_blocker(&stage1, Some(&stage2), status).unwrap();

        assert_eq!(status, MISMATCH_STATUS);
        assert_eq!(blocker, "stage1/stage2 toolchain closure policy digests differ");
    }

    #[test]
    fn fixed_point_status_rejects_policy_digest_presence_mismatch_before_success() {
        let stage1 = fixed_point_stage_run(STAGE1_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_A));
        let stage2 = fixed_point_stage_run(STAGE2_DIR, FIXED_POINT_TEST_DIGEST_A, None);

        let status = fixed_point_status(&stage1, &stage2).unwrap();
        let blocker = fixed_point_blocker(&stage1, Some(&stage2), status).unwrap();

        assert_eq!(status, MISMATCH_STATUS);
        assert_eq!(blocker, "stage1/stage2 toolchain closure policy digests differ");
    }

    #[test]
    fn fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches() {
        let root = Path::new("/repo/mantle");
        let out_dir = Path::new("/tmp/mantle-fixed-point");
        let rustc = Path::new("/toolchain/bin/rustc");
        let plan = plan_fixed_point_paths(root, out_dir, rustc, &[]).unwrap();
        let compatibility = RustcCompatibilitySummary {
            requested_rustc: rustc.to_path_buf(),
            stage_rustc: rustc.to_path_buf(),
            normalization: NORMALIZATION_NONE,
            wrapper: None,
            wrapper_blake3: None,
        };
        let toolchain_closure = enforced_test_toolchain_closure(FIXED_POINT_TEST_DIGEST_A);
        let stage1 = fixed_point_stage_run(STAGE1_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_A));
        let stage2 = fixed_point_stage_run(STAGE2_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_A));
        let rust_source_provider = absent_rust_source_provider_binding();

        let summary = fixed_point_summary(
            &plan,
            &compatibility,
            &toolchain_closure,
            &rust_source_provider,
            crunch_pipeline::HermeticityMode::Strict,
            &stage1,
            Some(&stage2),
            SUCCESS_STATUS,
        );

        assert!(summary.fixed_point);
        assert!(!summary.source_built_toolchain_closure.claim);
        assert_eq!(summary.source_built_toolchain_closure.non_claim, Some("not-source-built-toolchain-closure"));
        assert_eq!(summary.rust_source_provider.status, RUST_SOURCE_PROVIDER_STATUS_ABSENT);
        assert!(summary.non_claims.contains(&"not-source-built-toolchain-closure"));
    }

    #[test]
    fn fixed_point_summary_omits_closure_non_claim_when_provider_supplies_claim() {
        let toolchain_closure = provided_test_toolchain_closure(FIXED_POINT_TEST_DIGEST_A);

        let summary = fixed_point_summary_with_toolchain_closure(toolchain_closure);

        assert!(summary.fixed_point);
        assert!(summary.source_built_toolchain_closure.claim);
        assert_eq!(summary.source_built_toolchain_closure.status, "provided");
        assert!(summary.source_built_toolchain_closure.non_claim.is_none());
        assert!(!summary.non_claims.contains(&"not-source-built-toolchain-closure"));
        assert!(summary.non_claims.contains(&"not-release-reproducibility"));
        assert!(summary.non_claims.contains(&"not-full-cargo-compatibility"));
    }

    #[test]
    fn fixed_point_summary_omits_closure_non_claim_when_explicit_complete_closure_claims() {
        let toolchain_closure = complete_enforced_test_toolchain_closure(FIXED_POINT_TEST_DIGEST_A);

        let summary = fixed_point_summary_with_toolchain_closure(toolchain_closure);

        assert!(summary.fixed_point);
        assert!(summary.source_built_toolchain_closure.claim);
        assert_eq!(summary.source_built_toolchain_closure.status, "enforced-source-built");
        assert!(summary.source_built_toolchain_closure.non_claim.is_none());
        assert!(!summary.non_claims.contains(&"not-source-built-toolchain-closure"));
        assert!(summary.non_claims.contains(&"not-release-reproducibility"));
        assert!(summary.non_claims.contains(&"not-full-cargo-compatibility"));
    }

    #[test]
    fn fixed_point_summary_records_bundle_local_stage_paths() {
        let root = Path::new("/repo/mantle");
        let out_dir = Path::new("/tmp/mantle-fixed-point");
        let rustc = Path::new("/toolchain/bin/rustc");
        let plan = plan_fixed_point_paths(root, out_dir, rustc, &[]).unwrap();
        let compatibility = RustcCompatibilitySummary {
            requested_rustc: rustc.to_path_buf(),
            stage_rustc: rustc.to_path_buf(),
            normalization: NORMALIZATION_NONE,
            wrapper: None,
            wrapper_blake3: None,
        };
        let toolchain_closure = complete_enforced_test_toolchain_closure(FIXED_POINT_TEST_DIGEST_A);
        let stage1 = fixed_point_stage_run_in_bundle(STAGE1_DIR, out_dir, FIXED_POINT_TEST_DIGEST_A);
        let stage2 = fixed_point_stage_run_in_bundle(STAGE2_DIR, out_dir, FIXED_POINT_TEST_DIGEST_A);
        let summary = fixed_point_summary(
            &plan,
            &compatibility,
            &toolchain_closure,
            &absent_rust_source_provider_binding(),
            crunch_pipeline::HermeticityMode::Strict,
            &stage1,
            Some(&stage2),
            SUCCESS_STATUS,
        );

        assert_eq!(summary.bundle_dir, PathBuf::from(BUNDLE_ROOT_RELATIVE_PATH));
        assert_eq!(summary.stage1.dir, PathBuf::from(STAGE1_DIR));
        assert_eq!(summary.stage1.execution_dir, PathBuf::from(EXECUTION_DIR));
        let expected_stage1_binary = PathBuf::from(STAGE1_DIR).join(PRODUCED_MANTLE_FILE);
        assert_eq!(summary.stage1.receipt, PathBuf::from(STAGE1_DIR).join(RECEIPT_FILE));
        assert_eq!(summary.stage1.status, PathBuf::from(STAGE1_DIR).join(STATUS_FILE));
        assert_eq!(summary.stage1.binary.as_ref(), Some(&expected_stage1_binary));
        assert_eq!(summary.stage2.as_ref().unwrap().dir, PathBuf::from(STAGE2_DIR));
    }

    #[test]
    fn bundle_local_path_keeps_external_paths_absolute() {
        let bundle_dir = Path::new("/tmp/mantle-fixed-point");
        let external_path = Path::new("/outside/toolchain/bin/rustc");
        let local_path = bundle_dir.join(STAGE1_DIR).join(RECEIPT_FILE);

        assert_eq!(bundle_local_path(bundle_dir, external_path), external_path);
        assert_eq!(bundle_local_path(bundle_dir, &local_path), PathBuf::from(STAGE1_DIR).join(RECEIPT_FILE));
    }

    #[test]
    fn fixed_point_preflight_records_bundle_local_execution_path() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repo");
        let out_dir = dir.path().join("proof");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&out_dir).unwrap();
        let rustc = Path::new("/toolchain/bin/rustc");
        let plan = plan_fixed_point_paths(&root, &out_dir, rustc, &[]).unwrap();
        let compatibility = RustcCompatibilitySummary {
            requested_rustc: rustc.to_path_buf(),
            stage_rustc: rustc.to_path_buf(),
            normalization: NORMALIZATION_NONE,
            wrapper: None,
            wrapper_blake3: None,
        };
        write_fixed_point_preflight(
            &plan,
            &compatibility,
            &complete_enforced_test_toolchain_closure(FIXED_POINT_TEST_DIGEST_A),
            &absent_rust_source_provider_binding(),
        )
        .unwrap();

        let mut blockers = Vec::new();
        let (preflight, _) = read_json_with_digest(&plan.preflight_path, "preflight", &mut blockers);
        assert!(blockers.is_empty(), "blockers: {blockers:?}");
        let preflight = preflight.unwrap();
        assert_eq!(optional_str(&preflight, "/bundle_dir"), Some(BUNDLE_ROOT_RELATIVE_PATH));
        assert_eq!(optional_str(&preflight, "/shared_execution_dir"), Some(EXECUTION_DIR));
        assert_eq!(optional_str(&preflight, "/root"), Some(root.to_str().unwrap()));
    }

    #[test]
    fn provider_fixed_point_verifier_accepts_valid_bounded_bundle() {
        let evidence = valid_provider_fixed_point_evidence();

        let result = validate_provider_fixed_point_proof_evidence(evidence);

        assert!(result.valid);
        assert_eq!(result.status, PROVIDER_FIXED_POINT_STATUS_VALID);
        assert_eq!(result.closure_policy_digest_blake3.as_deref(), Some(FIXED_POINT_TEST_DIGEST_B));
        assert_eq!(result.stage_binary_digest_blake3.as_deref(), Some(FIXED_POINT_TEST_DIGEST_A));
        assert_eq!(result.stage1_unit_count, Some(FIXED_POINT_TEST_UNIT_COUNT));
        assert_eq!(result.stage2_unit_count, Some(FIXED_POINT_TEST_UNIT_COUNT));
        assert!(result.blockers.is_empty());
        assert!(result.non_claims.iter().any(|claim| claim == NOT_RELEASE_REPRODUCIBILITY_NON_CLAIM));
        assert!(result.non_claims.iter().any(|claim| claim == NOT_FULL_CARGO_COMPATIBILITY_NON_CLAIM));
    }

    #[test]
    fn provider_fixed_point_verifier_rejects_unapproved_proof_audit_event() {
        let mut evidence = valid_provider_fixed_point_evidence();
        evidence.meta.as_mut().unwrap()[HERMETICITY_AUDIT_EVENTS_FIELD] = json!(["future-benign-event"]);

        let result = validate_provider_fixed_point_proof_evidence(evidence);

        assert!(!result.valid);
        assert!(
            result.blockers.iter().any(|blocker| blocker.contains("future-benign-event")),
            "{:?}",
            result.blockers
        );
        assert!(
            result
                .blockers
                .iter()
                .any(|blocker| blocker.contains(crunch_release_core::STRICT_PROOF_POLICY_BASIS)),
            "{:?}",
            result.blockers
        );
    }

    #[test]
    fn provider_fixed_point_verifier_rejects_non_strict_hermeticity_modes() {
        for mode in [
            crunch_release_core::PRACTICAL_HERMETICITY_MODE,
            crunch_release_core::IMPURE_HERMETICITY_MODE,
            "",
        ] {
            let mut evidence = valid_provider_fixed_point_evidence();
            if mode.is_empty() {
                evidence.meta.as_mut().unwrap().as_object_mut().unwrap().remove("hermeticity_mode");
            } else {
                evidence.meta.as_mut().unwrap()["hermeticity_mode"] = Value::String(mode.to_string());
            }

            let result = validate_provider_fixed_point_proof_evidence(evidence);

            assert!(!result.valid, "{mode}");
            assert!(
                result.blockers.iter().any(|blocker| blocker.contains("hermeticity-mode")),
                "{mode}: {:?}",
                result.blockers
            );
        }
    }

    #[test]
    fn provider_fixed_point_verifier_rejects_protected_env_and_host_tool_degradation() {
        let mut evidence = valid_provider_fixed_point_evidence();
        evidence.meta.as_mut().unwrap()["protected_environment_status"] = Value::String("leaked".to_string());
        *evidence.meta.as_mut().unwrap().pointer_mut("/rust_source_provider/status").unwrap() =
            Value::String(RUST_SOURCE_PROVIDER_STATUS_ABSENT.to_string());

        let result = validate_provider_fixed_point_proof_evidence(evidence);

        assert!(!result.valid);
        assert!(result.blockers.iter().any(|blocker| blocker.contains("protected-environment")));
        assert!(result.blockers.iter().any(|blocker| blocker.contains("host-tool-inventory")));
    }

    #[test]
    fn provider_fixed_point_verifier_accepts_informational_proof_audit_events() {
        let mut evidence = valid_provider_fixed_point_evidence();
        evidence.meta.as_mut().unwrap()[STAGE1_DIR][HERMETICITY_AUDIT_EVENTS_FIELD] =
            json!(["store-read", "output-write"]);

        let result = validate_provider_fixed_point_proof_evidence(evidence);

        assert!(result.valid, "{:?}", result.blockers);
    }

    #[test]
    fn provider_fixed_point_verifier_rejects_stage_digest_mismatch() {
        let mut evidence = valid_provider_fixed_point_evidence();
        *evidence.meta.as_mut().unwrap().pointer_mut("/stage2/binary_blake3").unwrap() =
            Value::String(FIXED_POINT_TEST_DIGEST_B.to_string());
        evidence.stage2_binary_digest_actual = Some(FIXED_POINT_TEST_DIGEST_B.to_string());

        let result = validate_provider_fixed_point_proof_evidence(evidence);

        assert!(!result.valid);
        assert_eq!(result.status, PROVIDER_FIXED_POINT_STATUS_INVALID);
        assert!(result.stage_binary_digest_blake3.is_none());
        assert!(result.blockers.iter().any(|blocker| blocker.contains("stage binary digests differ")));
    }

    #[test]
    fn provider_fixed_point_verifier_rejects_missing_enforced_closure() {
        let mut evidence = valid_provider_fixed_point_evidence();
        *evidence.meta.as_mut().unwrap().pointer_mut("/source_built_toolchain_closure/status").unwrap() =
            Value::String("provided".to_string());

        let result = validate_provider_fixed_point_proof_evidence(evidence);

        assert!(!result.valid);
        assert!(
            result
                .blockers
                .iter()
                .any(|blocker| blocker.contains("source-built closure status expected enforced-source-built"))
        );
    }

    #[test]
    fn provider_fixed_point_verifier_rejects_missing_bounded_non_claims() {
        let mut evidence = valid_provider_fixed_point_evidence();
        *evidence.meta.as_mut().unwrap().pointer_mut("/non_claims").unwrap() =
            json!([NOT_CRUNCH_BOOTSTRAP_NON_CLAIM, NOT_RELEASE_REPRODUCIBILITY_NON_CLAIM]);
        evidence.non_claims_text = Some("This proof does not claim release reproducibility.\n".to_string());

        let result = validate_provider_fixed_point_proof_evidence(evidence);

        assert!(!result.valid);
        assert!(result.blockers.iter().any(|blocker| blocker.contains(NOT_FULL_CARGO_COMPATIBILITY_NON_CLAIM)));
        assert!(result.blockers.iter().any(|blocker| blocker.contains("full Cargo compatibility")));
    }

    #[test]
    fn provider_fixed_point_verifier_rebases_copied_bundle_stage_paths() {
        let dir = tempfile::tempdir().unwrap();
        let original_dir = dir.path().join("original-proof");
        let copied_dir = dir.path().join("copied-proof");
        let stage1_dir = copied_dir.join(STAGE1_DIR);
        let stage2_dir = copied_dir.join(STAGE2_DIR);
        fs::create_dir_all(&stage1_dir).unwrap();
        fs::create_dir_all(&stage2_dir).unwrap();
        let binary_bytes = b"copied provider fixed point binary";
        let binary_digest = blake3::hash(binary_bytes).to_hex().to_string();
        fs::write(stage1_dir.join(PRODUCED_MANTLE_FILE), binary_bytes).unwrap();
        fs::write(stage2_dir.join(PRODUCED_MANTLE_FILE), binary_bytes).unwrap();
        write_summary(&stage1_dir.join(RECEIPT_FILE), &fixed_point_stage_receipt_json()).unwrap();
        write_summary(&stage2_dir.join(RECEIPT_FILE), &fixed_point_stage_receipt_json()).unwrap();
        let mut evidence = valid_provider_fixed_point_evidence();
        let meta = evidence.meta.as_mut().unwrap();
        meta["bundle_dir"] = Value::String(original_dir.display().to_string());
        for stage in [STAGE1_DIR, STAGE2_DIR] {
            meta[stage]["binary"] =
                Value::String(original_dir.join(stage).join(PRODUCED_MANTLE_FILE).display().to_string());
            meta[stage]["binary_blake3"] = Value::String(binary_digest.clone());
            meta[stage]["receipt"] = Value::String(original_dir.join(stage).join(RECEIPT_FILE).display().to_string());
        }
        write_summary(&copied_dir.join(META_FILE), meta).unwrap();
        write_summary(&copied_dir.join(PRE_FLIGHT_FILE), evidence.preflight.as_ref().unwrap()).unwrap();
        fs::write(copied_dir.join(NON_CLAIMS_FILE), evidence.non_claims_text.as_ref().unwrap()).unwrap();

        let result = verify_provider_fixed_point_proof_bundle(&copied_dir);

        assert!(result.valid, "blockers: {:?}", result.blockers);
        assert_eq!(result.stage_binary_digest_blake3.as_deref(), Some(binary_digest.as_str()));
        assert!(!original_dir.join(STAGE1_DIR).join(PRODUCED_MANTLE_FILE).exists());
    }

    #[test]
    fn self_build_non_claims_omit_closure_non_claim_when_provider_supplies_claim() {
        let provided = provided_test_toolchain_closure(FIXED_POINT_TEST_DIGEST_A);
        let absent = crate::source_toolchain_closure::absent_source_built_toolchain_closure();

        let provided_non_claims = self_build_non_claims(&provided);
        let absent_non_claims = self_build_non_claims(&absent);
        let provided_text = self_build_non_claims_text(&provided).join("\n");
        let absent_text = self_build_non_claims_text(&absent).join("\n");

        assert!(!provided_non_claims.contains(&"not-source-built-toolchain-closure"));
        assert!(absent_non_claims.contains(&"not-source-built-toolchain-closure"));
        assert!(!provided_text.contains("source-built compiler/toolchain closure provenance"));
        assert!(absent_text.contains("source-built compiler/toolchain closure provenance"));
    }

    #[cfg(unix)]
    #[test]
    fn rust_source_provider_binding_uses_validated_provider_rustc() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        fs::create_dir(&provider_dir).unwrap();
        write_fake_rust_source_provider(&provider_dir, false);

        let loaded = load_rust_source_provider(Some(&provider_dir), None).unwrap();
        let expected_provider_dir = fs::canonicalize(&provider_dir).unwrap();
        let expected_rustc = expected_provider_dir.join("bin/rustc");

        assert_eq!(loaded.status.schema, RUST_SOURCE_PROVIDER_BINDING_SCHEMA);
        assert_eq!(loaded.status.status, RUST_SOURCE_PROVIDER_STATUS_VALIDATED);
        assert_eq!(loaded.status.provider_dir.as_deref(), Some(expected_provider_dir.as_path()));
        assert_eq!(loaded.status.rustc_path.as_deref(), Some(expected_rustc.as_path()));
        assert_eq!(loaded.status.host_triple.as_deref(), Some(RUST_PROVIDER_HOST_TRIPLE));
        assert_eq!(loaded.status.target_triple.as_deref(), Some(RUST_PROVIDER_TARGET_TRIPLE));
        assert_eq!(loaded.rustc.as_deref(), Some(expected_rustc.as_path()));
        let status = loaded.toolchain_closure_status.as_ref().unwrap();
        assert_eq!(status.status, "provided");
        assert!(status.claim);
        assert!(status.non_claim.is_none());
        assert_eq!(status.policy_digest_blake3, loaded.status.policy_digest_blake3);
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_rustc_runtime_wrapper_uses_only_the_validated_cxx_runtime() {
        use crate::full_source_rust_binding::FullSourceNativeArtifactBinding;
        use crate::full_source_rust_binding::FullSourceNativeArtifactRole;

        let dir = tempfile::tempdir().unwrap();
        let runtime_dir = dir.path().join("native-provider/x86_64-linux-musl/lib");
        let cxx_runtime = runtime_dir.join(RUSTC_LIBSTDCXX_RUNTIME_FILE);
        let soname = runtime_dir.join(RUSTC_LIBSTDCXX_SONAME);
        let loader = runtime_dir.join(RUSTC_DYNAMIC_LOADER_FILE);
        fs::create_dir_all(&runtime_dir).unwrap();
        fs::write(&cxx_runtime, b"bound cxx runtime\n").unwrap();
        std::os::unix::fs::symlink(RUSTC_LIBSTDCXX_RUNTIME_FILE, &soname).unwrap();
        write_fake_executable(
            &loader,
            "#!/bin/sh\nprintf '%s\\n' \"${LD_LIBRARY_PATH-unset}\"\nprintf '%s\\n' \"$@\"\n",
        );
        let cxx_artifact = FullSourceNativeArtifactBinding {
            role: FullSourceNativeArtifactRole::Libstdcxx,
            path: cxx_runtime.display().to_string(),
            content_digest_blake3: crate::protected_exec::blake3_file_hex(&cxx_runtime).unwrap(),
        };
        let loader_artifact = FullSourceNativeArtifactBinding {
            role: FullSourceNativeArtifactRole::DynamicLinker,
            path: loader.display().to_string(),
            content_digest_blake3: crate::protected_exec::blake3_file_hex(&loader).unwrap(),
        };
        let rust_root = dir.path().join("rust-provider");
        let real_rustc = rust_root.join("bin/rustc");
        let dynamic_rustc = rust_root.join("bin").join(RUSTC_DYNAMIC_BINARY_FILE);
        let runtime_lib = rust_root.join("lib/mantle-runtime");
        let rust_lib = rust_root.join("lib");
        let rust_stdlib = rust_lib.join("rustlib").join(RUSTC_RUNTIME_HOST_TRIPLE).join("lib");
        fs::create_dir_all(&runtime_lib).unwrap();
        fs::create_dir_all(&rust_stdlib).unwrap();
        fs::create_dir_all(real_rustc.parent().unwrap()).unwrap();
        write_fake_executable(&real_rustc, "#!/bin/sh\nexit 0\n");
        write_fake_executable(&dynamic_rustc, "#!/bin/sh\nexit 0\n");
        let wrapper = dir.path().join(RUSTC_RUNTIME_WRAPPER_FILE);
        let receipt_bound_linker = dir.path().join(TOOLCHAIN_COMPATIBILITY_PATH_DIR).join(RUSTC_BOUND_LINKER_ALIAS);
        fs::create_dir_all(receipt_bound_linker.parent().unwrap()).unwrap();
        write_fake_executable(&receipt_bound_linker, "#!/bin/sh\nexit 0\n");
        let artifacts = [cxx_artifact.clone(), loader_artifact];

        let selected = bound_rustc_runtime(&artifacts).unwrap().expect("bound Rust runtime");
        write_bound_rustc_runtime_wrapper(
            &wrapper,
            &real_rustc,
            &selected,
            Path::new("/bin/sh"),
            &receipt_bound_linker,
        )
        .unwrap();
        let output = Command::new(&wrapper).env(DYNAMIC_LIBRARY_PATH_ENV, "/ambient/not-authorized").output().unwrap();
        fs::remove_file(&soname).unwrap();
        let wrong_runtime = runtime_dir.join("wrong-runtime");
        fs::write(&wrong_runtime, b"wrong runtime\n").unwrap();
        std::os::unix::fs::symlink("wrong-runtime", &soname).unwrap();
        let error = bound_rustc_runtime(&artifacts).unwrap_err();
        let action_linker = rustc_runtime_wrapper_linker(&wrapper).unwrap().expect("runtime wrapper linker");
        let ambiguous_linker = wrapper.parent().unwrap().join(RUSTC_BOUND_LINKER_ALIAS);
        write_fake_executable(&ambiguous_linker, "#!/bin/sh\nexit 0\n");
        let ambiguous_linker_error = rustc_runtime_wrapper_linker(&wrapper).unwrap_err();
        let expected_library_path = env::join_paths([runtime_lib, runtime_dir, rust_lib, rust_stdlib]).unwrap();
        let stdout = String::from_utf8(output.stdout).unwrap();

        assert!(output.status.success());
        assert!(stdout.contains(expected_library_path.to_string_lossy().as_ref()));
        assert!(stdout.contains(dynamic_rustc.to_string_lossy().as_ref()));
        assert!(stdout.contains(&format!("linker={}", receipt_bound_linker.display())));
        assert_eq!(action_linker, fs::canonicalize(&receipt_bound_linker).unwrap());
        assert!(ambiguous_linker_error.message().contains("found 2"));
        assert!(!stdout.contains("ambient/not-authorized"));
        assert!(!String::from_utf8(output.stderr).unwrap().contains("ambient"));
        assert!(error.message().contains("does not resolve to bound artifact"));
        assert!(full_source_native_artifact_is_executable(FullSourceNativeArtifactRole::DynamicLinker));
        assert_eq!(
            rust_fixed_kind_for_native_artifact(FullSourceNativeArtifactRole::DynamicLinker),
            Some(crate::source_built_rust_action_plan::RustFixedExecutableKind::NativeHelper)
        );
    }

    #[cfg(unix)]
    #[test]
    fn relative_native_binding_rebases_under_the_closure_root_and_rejects_escape() {
        use crate::full_source_rust_binding::FullSourceNativeArtifactBinding;
        use crate::full_source_rust_binding::FullSourceNativeArtifactRole;

        let dir = tempfile::tempdir().unwrap();
        let native_root = dir.path().join("native-provider");
        let archiver = native_root.join("bin/ar");
        fs::create_dir_all(archiver.parent().unwrap()).unwrap();
        write_fake_executable(&archiver, "#!/bin/sh\nexit 0\n");
        let artifact = FullSourceNativeArtifactBinding {
            role: FullSourceNativeArtifactRole::ArchiveTool,
            path: "bin/ar".to_string(),
            content_digest_blake3: crate::protected_exec::blake3_file_hex(&archiver).unwrap(),
        };

        let rebound = rebase_bound_native_artifacts(&native_root, std::slice::from_ref(&artifact)).unwrap();
        let mut escaping = artifact;
        escaping.path = "../bin/ar".to_string();
        let error = rebase_bound_native_artifacts(&native_root, &[escaping]).unwrap_err();

        assert_eq!(rebound.len(), 1);
        assert_eq!(rebound[0].path, archiver.display().to_string());
        assert!(Path::new(&rebound[0].path).is_absolute());
        assert!(error.message().contains("must stay provider-relative"));
    }

    #[cfg(unix)]
    #[test]
    fn bound_rust_execution_shell_requires_exact_busybox_and_receipt_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        let busybox = bin.join("busybox");
        let shell = bin.join("sh");
        let receipt = dir.path().join("busybox-receipt.json");
        fs::create_dir_all(&bin).unwrap();
        write_fake_executable(&busybox, "#!/bin/sh\nexit 0\n");
        std::os::unix::fs::symlink(&busybox, &shell).unwrap();
        fs::write(&receipt, b"{\"status\":\"built\"}\n").unwrap();
        let busybox_digest = crate::protected_exec::blake3_file_hex(&busybox).unwrap();
        let receipt_digest = crate::protected_exec::blake3_file_hex(&receipt).unwrap();
        let binding = crate::full_source_rust_binding::FullSourceRustHostToolBinding {
            role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Busybox,
            path: busybox.display().to_string(),
            content_digest_blake3: busybox_digest.clone(),
            source_id: "busybox-source".to_string(),
            construction_receipt_path: receipt.display().to_string(),
            construction_receipt_digest_blake3: receipt_digest.clone(),
        };

        let bound = bound_rust_execution_shell(&binding).unwrap();
        let mut wrong_tool = binding.clone();
        wrong_tool.content_digest_blake3 = FIXED_POINT_TEST_DIGEST_A.to_string();
        let tool_error = bound_rust_execution_shell(&wrong_tool).unwrap_err();
        let mut wrong_receipt = binding;
        wrong_receipt.construction_receipt_digest_blake3 = FIXED_POINT_TEST_DIGEST_B.to_string();
        let receipt_error = bound_rust_execution_shell(&wrong_receipt).unwrap_err();

        assert_eq!(bound.execution_path, shell);
        assert_eq!(bound.content_digest_blake3, busybox_digest);
        assert_eq!(bound.construction_receipt_digest_blake3, receipt_digest);
        assert!(tool_error.message().contains("host-tool digest mismatch"));
        assert!(receipt_error.message().contains("construction receipt digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn rust_action_fixed_authority_includes_receipt_bound_aliases_and_host_tools() {
        use crate::source_built_rust_action_plan::RustFixedExecutableKind;
        let dir = tempfile::tempdir().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, None);
        let guard = dir.path().join("guard");
        let host_bin = dir.path().join("host-tools");
        let busybox = host_bin.join("busybox");
        let shell = host_bin.join("sh");
        let receipt = dir.path().join("busybox-receipt.json");
        fs::create_dir_all(&guard).unwrap();
        fs::create_dir_all(&host_bin).unwrap();
        write_fake_executable(&busybox, "#!/bin/sh\nexit 0\n");
        std::os::unix::fs::symlink(&busybox, &shell).unwrap();
        fs::write(&receipt, b"{\"status\":\"built\"}\n").unwrap();
        let binding = crate::full_source_rust_binding::FullSourceRustHostToolBinding {
            role: crate::full_source_rust_binding::FullSourceRustHostToolRole::Busybox,
            path: busybox.display().to_string(),
            content_digest_blake3: crate::protected_exec::blake3_file_hex(&busybox).unwrap(),
            source_id: "busybox-source".to_string(),
            construction_receipt_path: receipt.display().to_string(),
            construction_receipt_digest_blake3: crate::protected_exec::blake3_file_hex(&receipt).unwrap(),
        };
        let bound_shell = bound_rust_execution_shell(&binding).unwrap();
        write_toolchain_path_aliases_with_shell(&guard, &manifest, &bound_shell.execution_path).unwrap();
        write_bound_rust_host_tool_aliases(&guard, std::slice::from_ref(&binding), &bound_shell.execution_path)
            .unwrap();
        let action_trust = crate::source_built_rust_provider_action::RustProviderActionEvidence {
            plan_path: dir.path().join("rust-provider-action-plan.json"),
            plan_digest_blake3: FIXED_POINT_TEST_DIGEST_A.to_string(),
            audit_path: dir.path().join("rust-provider-action-audit.json"),
            audit_digest_blake3: FIXED_POINT_TEST_DIGEST_A.to_string(),
            reconciliation_path: dir.path().join("rust-provider-action-reconciliation.json"),
            reconciliation_digest_blake3: FIXED_POINT_TEST_DIGEST_B.to_string(),
            planned_action_count: 1,
            matched_action_count: 1,
            observed_event_count: 1,
            matched_event_count: 1,
        };

        let authorities = rust_child_action_fixed_executables(
            &tools.rustc,
            &manifest,
            &bound_shell,
            &[binding],
            &[],
            &action_trust,
            &guard,
            Some(FIXED_POINT_TEST_DIGEST_A),
        )
        .unwrap();
        let paths = authorities.iter().map(|authority| authority.path.as_str()).collect::<BTreeSet<_>>();

        assert_eq!(authorities.iter().filter(|authority| authority.kind == RustFixedExecutableKind::Rustc).count(), 1);
        assert_eq!(paths.len(), authorities.len());
        assert!(paths.contains(path_to_string(&fs::canonicalize(&busybox).unwrap()).unwrap().as_str()));
        assert!(paths.contains(path_to_string(&fs::canonicalize(guard.join("busybox")).unwrap()).unwrap().as_str()));
        assert!(authorities.iter().all(|authority| authority.output_identity_blake3 == authority.digest_blake3));
    }

    #[test]
    fn rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback() {
        let fallback = PathBuf::from("/fallback/rustc");
        let provider_rustc = PathBuf::from("/provider/bin/rustc");
        let absent = LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: None,
            source_built_shell: None,
            source_built_host_tools: Vec::new(),
            source_built_native_artifacts: Vec::new(),
            source_built_action_trust: None,
            toolchain_closure_status: None,
        };
        let validated = LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: Some(provider_rustc.clone()),
            source_built_shell: None,
            source_built_host_tools: Vec::new(),
            source_built_native_artifacts: Vec::new(),
            source_built_action_trust: None,
            toolchain_closure_status: None,
        };

        assert_eq!(selected_cargo_free_rustc(&absent, &fallback), fallback.as_path());
        assert_ne!(selected_cargo_free_rustc(&absent, &fallback), provider_rustc.as_path());
        assert_eq!(selected_cargo_free_rustc(&validated, &fallback), provider_rustc.as_path());
        assert_ne!(selected_cargo_free_rustc(&validated, &fallback), fallback.as_path());
    }

    #[test]
    fn effective_closure_uses_provider_only_when_manifest_absent() {
        let absent_closure = load_source_built_toolchain_closure(None).unwrap();
        let provider_status = provided_test_toolchain_closure(FIXED_POINT_TEST_DIGEST_A);
        let provider = LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: Some(PathBuf::from("/provider/bin/rustc")),
            source_built_shell: None,
            source_built_host_tools: Vec::new(),
            source_built_native_artifacts: Vec::new(),
            source_built_action_trust: None,
            toolchain_closure_status: Some(provider_status.clone()),
        };

        let effective = effective_source_built_toolchain_closure(&absent_closure, &provider);

        assert_eq!(effective.status, "provided");
        assert!(effective.claim);
        assert_eq!(effective.policy_digest_blake3, provider_status.policy_digest_blake3);
    }

    #[test]
    fn effective_closure_keeps_absent_non_claim_without_provider() {
        let absent_closure = load_source_built_toolchain_closure(None).unwrap();
        let provider = LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: None,
            source_built_shell: None,
            source_built_host_tools: Vec::new(),
            source_built_native_artifacts: Vec::new(),
            source_built_action_trust: None,
            toolchain_closure_status: None,
        };

        let effective = effective_source_built_toolchain_closure(&absent_closure, &provider);

        assert_eq!(effective.status, "not-provided");
        assert!(!effective.claim);
        assert_eq!(effective.non_claim, Some("not-source-built-toolchain-closure"));
    }

    #[cfg(unix)]
    #[test]
    fn rust_source_provider_binding_rejects_prebuilt_provider_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let provider_dir = dir.path().join("provider");
        fs::create_dir(&provider_dir).unwrap();
        write_fake_rust_source_provider(&provider_dir, true);

        let err = load_rust_source_provider(Some(&provider_dir), None).unwrap_err();
        let message = err.message();

        assert!(message.contains("source-built Rust provider blocked"));
        assert!(message.contains("failed validation"));
        assert!(message.contains("not source-built"));
    }

    fn fixed_point_stage_run_in_bundle(
        name: &'static str,
        bundle_dir: &Path,
        binary_digest: &str,
    ) -> FixedPointStageRun {
        FixedPointStageRun {
            name,
            dir: bundle_dir.join(name),
            execution_dir: bundle_dir.join(EXECUTION_DIR),
            receipt_path: bundle_dir.join(name).join(RECEIPT_FILE),
            stderr_path: bundle_dir.join(name).join(STDERR_FILE),
            status_path: bundle_dir.join(name).join(STATUS_FILE),
            status_code: Some(SUCCESS_EXIT_CODE),
            execution_status: SUCCESS_STATUS.to_string(),
            cargo_marker_absent: true,
            success: true,
            unit_count: FIXED_POINT_TEST_UNIT_COUNT,
            failed_unit_count: 0,
            binary: Some(bundle_dir.join(name).join(PRODUCED_MANTLE_FILE)),
            binary_blake3: Some(binary_digest.to_string()),
            smoke_status_code: Some(SUCCESS_EXIT_CODE),
            source_built_toolchain_closure_policy_digest_blake3: Some(FIXED_POINT_TEST_DIGEST_A.to_string()),
            selected_c_compiler: None,
            blocker: None,
            blocker_diagnostic: None,
        }
    }

    fn fixed_point_stage_run(
        name: &'static str,
        binary_digest: &str,
        policy_digest: Option<&str>,
    ) -> FixedPointStageRun {
        FixedPointStageRun {
            name,
            dir: PathBuf::from(format!("/tmp/{name}")),
            execution_dir: PathBuf::from("/tmp/execution"),
            receipt_path: PathBuf::from(format!("/tmp/{name}/receipt.json")),
            stderr_path: PathBuf::from(format!("/tmp/{name}/stderr.txt")),
            status_path: PathBuf::from(format!("/tmp/{name}/status.txt")),
            status_code: Some(SUCCESS_EXIT_CODE),
            execution_status: SUCCESS_STATUS.to_string(),
            cargo_marker_absent: true,
            success: true,
            unit_count: 1,
            failed_unit_count: 0,
            binary: Some(PathBuf::from(format!("/tmp/{name}/mantle"))),
            binary_blake3: Some(binary_digest.to_string()),
            smoke_status_code: Some(SUCCESS_EXIT_CODE),
            source_built_toolchain_closure_policy_digest_blake3: policy_digest.map(ToOwned::to_owned),
            selected_c_compiler: None,
            blocker: None,
            blocker_diagnostic: None,
        }
    }

    fn enforced_test_toolchain_closure(
        policy_digest: &str,
    ) -> crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus {
        crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus {
            schema: crate::source_toolchain_closure::SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
            status: "validated-enforced",
            claim: false,
            non_claim: Some(crate::source_toolchain_closure::SOURCE_BUILT_NON_CLAIM),
            manifest_path: Some(PathBuf::from("/tmp/toolchain-closure.json")),
            policy_digest_blake3: Some(policy_digest.to_string()),
            member_count: Some(FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT),
            source_built_member_count: Some(
                FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT - FIXED_POINT_TEST_SEED_EXCEPTION_COUNT,
            ),
            seed_exception_count: Some(FIXED_POINT_TEST_SEED_EXCEPTION_COUNT),
        }
    }

    fn complete_enforced_test_toolchain_closure(
        policy_digest: &str,
    ) -> crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus {
        crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus {
            schema: crate::source_toolchain_closure::SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
            status: "enforced-source-built",
            claim: true,
            non_claim: None,
            manifest_path: Some(PathBuf::from("/tmp/toolchain-closure.json")),
            policy_digest_blake3: Some(policy_digest.to_string()),
            member_count: Some(FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT),
            source_built_member_count: Some(FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT),
            seed_exception_count: Some(0),
        }
    }

    fn provided_test_toolchain_closure(
        policy_digest: &str,
    ) -> crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus {
        crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus {
            schema: crate::source_toolchain_closure::SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
            status: "provided",
            claim: true,
            non_claim: None,
            manifest_path: Some(PathBuf::from("/tmp/rust-provider.json")),
            policy_digest_blake3: Some(policy_digest.to_string()),
            member_count: Some(FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT),
            source_built_member_count: Some(FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT),
            seed_exception_count: Some(0),
        }
    }

    fn valid_provider_fixed_point_evidence() -> ProviderFixedPointProofEvidence {
        let proof_dir = PathBuf::from("/tmp/provider-fixed-point-proof");
        let stage1 = fixed_point_stage_summary_json(STAGE1_DIR, FIXED_POINT_TEST_DIGEST_A, FIXED_POINT_TEST_DIGEST_B);
        let stage2 = fixed_point_stage_summary_json(STAGE2_DIR, FIXED_POINT_TEST_DIGEST_A, FIXED_POINT_TEST_DIGEST_B);
        let meta = json!({
            "schema": FIXED_POINT_SCHEMA,
            "status": SUCCESS_STATUS,
            "root": "/repo/mantle",
            "bundle_dir": proof_dir,
            "fixed_point": true,
            "hermeticity_mode": crunch_release_core::STRICT_HERMETICITY_MODE,
            "stage1": stage1,
            "stage2": stage2,
            "rustc_compatibility": {
                "requested_rustc": "/provider/bin/rustc",
                "stage_rustc": "/provider/bin/rustc",
                "normalization": NORMALIZATION_NONE,
                "wrapper": null,
                "wrapper_blake3": null
            },
            "source_built_toolchain_closure": {
                "schema": crate::source_toolchain_closure::SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA,
                "status": ENFORCED_SOURCE_BUILT_STATUS,
                "claim": true,
                "non_claim": null,
                "manifest_path": "/tmp/toolchain-closure.json",
                "policy_digest_blake3": FIXED_POINT_TEST_DIGEST_B,
                "member_count": FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT,
                "source_built_member_count": FIXED_POINT_TEST_TOOLCHAIN_MEMBER_COUNT,
                "seed_exception_count": 0
            },
            "rust_source_provider": {
                "schema": RUST_SOURCE_PROVIDER_BINDING_SCHEMA,
                "status": RUST_SOURCE_PROVIDER_STATUS_VALIDATED,
                "provider_dir": "/provider",
                "metadata_path": "/provider/share/mantle-rust-provider/provider.json",
                "metadata_digest_blake3": FIXED_POINT_TEST_DIGEST_A,
                "policy_digest_blake3": FIXED_POINT_TEST_DIGEST_B,
                "host_triple": RUST_PROVIDER_TARGET_TRIPLE,
                "target_triple": RUST_PROVIDER_TARGET_TRIPLE,
                "artifact_count": 6,
                "source_count": 2,
                "receipt_count": 1,
                "rustc_path": "/provider/bin/rustc"
            },
            "blocker": null,
            "non_claims": [
                NOT_CRUNCH_BOOTSTRAP_NON_CLAIM,
                NOT_RELEASE_REPRODUCIBILITY_NON_CLAIM,
                NOT_FULL_CARGO_COMPATIBILITY_NON_CLAIM
            ]
        });
        let preflight = json!({
            "schema": FIXED_POINT_SCHEMA,
            "root": "/repo/mantle",
            "bundle_dir": "/tmp/provider-fixed-point-proof",
            "source_built_toolchain_closure": {
                "status": ENFORCED_SOURCE_BUILT_STATUS,
                "claim": true,
                "policy_digest_blake3": FIXED_POINT_TEST_DIGEST_B
            }
        });
        ProviderFixedPointProofEvidence {
            proof_dir,
            meta: Some(meta),
            meta_digest_blake3: Some(FIXED_POINT_TEST_DIGEST_A.to_string()),
            preflight: Some(preflight),
            non_claims_text: Some(
                "This proof does not claim Mantle bootstrap or release reproducibility.\nThis proof does not claim full Cargo compatibility.\n"
                    .to_string(),
            ),
            stage1_binary_digest_actual: Some(FIXED_POINT_TEST_DIGEST_A.to_string()),
            stage2_binary_digest_actual: Some(FIXED_POINT_TEST_DIGEST_A.to_string()),
            stage1_receipt: Some(fixed_point_stage_receipt_json()),
            stage2_receipt: Some(fixed_point_stage_receipt_json()),
            shell_blockers: Vec::new(),
        }
    }

    fn fixed_point_stage_summary_json(stage_name: &str, binary_digest: &str, policy_digest: &str) -> Value {
        let mut value = json!({
            "name": stage_name,
            "dir": format!("/tmp/provider-fixed-point-proof/{stage_name}"),
            "execution_dir": "/tmp/provider-fixed-point-proof/execution",
            "receipt": format!("/tmp/provider-fixed-point-proof/{stage_name}/receipt.json"),
            "stderr": format!("/tmp/provider-fixed-point-proof/{stage_name}/stderr.txt"),
            "status": format!("/tmp/provider-fixed-point-proof/{stage_name}/status.txt"),
            "status_code": SUCCESS_EXIT_CODE,
            "execution_status": SUCCESS_STATUS,
            "cargo_marker_absent": true,
            "success": true,
            "unit_count": FIXED_POINT_TEST_UNIT_COUNT,
            "failed_unit_count": 0,
            "binary": format!("/tmp/provider-fixed-point-proof/{stage_name}/mantle"),
            "binary_blake3": binary_digest,
            "smoke_status_code": SUCCESS_EXIT_CODE,
            "blocker": null
        });
        value[TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD] = Value::String(policy_digest.to_string());
        value
    }

    fn fixed_point_stage_receipt_json() -> Value {
        json!({
            "topology_execution": {
                "execution_status": SUCCESS_STATUS,
                "unit_executions": [
                    { "unit_id": "unit-1", "execution_status": SUCCESS_STATUS },
                    { "unit_id": "unit-2", "execution_status": SUCCESS_STATUS }
                ]
            }
        })
    }

    fn fixed_point_summary_with_toolchain_closure(
        toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    ) -> FixedPointSummary {
        let root = Path::new("/repo/mantle");
        let out_dir = Path::new("/tmp/mantle-fixed-point");
        let rustc = Path::new("/toolchain/bin/rustc");
        let plan = plan_fixed_point_paths(root, out_dir, rustc, &[]).unwrap();
        let compatibility = RustcCompatibilitySummary {
            requested_rustc: rustc.to_path_buf(),
            stage_rustc: rustc.to_path_buf(),
            normalization: NORMALIZATION_NONE,
            wrapper: None,
            wrapper_blake3: None,
        };
        let stage1 = fixed_point_stage_run(STAGE1_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_A));
        let stage2 = fixed_point_stage_run(STAGE2_DIR, FIXED_POINT_TEST_DIGEST_A, Some(FIXED_POINT_TEST_DIGEST_A));
        let rust_source_provider = absent_rust_source_provider_binding();

        fixed_point_summary(
            &plan,
            &compatibility,
            &toolchain_closure,
            &rust_source_provider,
            crunch_pipeline::HermeticityMode::Strict,
            &stage1,
            Some(&stage2),
            SUCCESS_STATUS,
        )
    }

    #[cfg(unix)]
    #[test]
    fn effective_closure_prefers_explicit_manifest_over_provider_status() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest);
        let provider_status = provided_test_toolchain_closure(FIXED_POINT_TEST_DIGEST_A);
        let provider = LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: Some(PathBuf::from("/provider/bin/rustc")),
            source_built_shell: None,
            source_built_host_tools: Vec::new(),
            source_built_native_artifacts: Vec::new(),
            source_built_action_trust: None,
            toolchain_closure_status: Some(provider_status),
        };

        let effective = effective_source_built_toolchain_closure(&closure, &provider);

        assert_eq!(effective.status, "validated-not-enforced");
        assert!(!effective.claim);
        assert_eq!(effective.non_claim, Some("not-source-built-toolchain-closure"));
        assert_ne!(effective.policy_digest_blake3, Some(FIXED_POINT_TEST_DIGEST_A.to_string()));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_env_omits_ambient_path_entries() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest);
        let guard_dir = dir.path().join("guard-bin");
        fs::create_dir_all(&guard_dir).unwrap();

        let path_env = execution_path_env(&guard_dir, &closure).unwrap();
        let path_entries = env::split_paths(&path_env).collect::<Vec<_>>();

        assert_eq!(path_entries, vec![guard_dir.clone()]);
        assert!(guard_dir.join(C_COMPILER_ALIAS).exists());
        assert!(!path_env.to_string_lossy().contains("/nix/var/nix/profiles"));
        assert!(!path_env.to_string_lossy().contains("/run/current-system/sw"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_c_compiler_alias_materializes_declared_runtime_inputs() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let observed_args = dir.path().join("observed-cc-args.txt");
        write_fake_executable(
            &tools.c_compiler,
            &format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > {}\nexit 0\n", shell_quote(&observed_args)),
        );
        let manifest = fake_toolchain_manifest(&tools, None);
        let guard_dir = dir.path().join("guard-bin");
        fs::create_dir_all(&guard_dir).unwrap();
        let mut crt1_permissions = fs::metadata(&tools.target_crt1).unwrap().permissions();
        crt1_permissions.set_readonly(true);
        fs::set_permissions(&tools.target_crt1, crt1_permissions).unwrap();

        write_toolchain_path_aliases(&guard_dir, &manifest).unwrap();
        write_toolchain_path_aliases(&guard_dir, &manifest).unwrap();
        let status = Command::new(guard_dir.join(C_COMPILER_ALIAS))
            .args([
                TOOLCHAIN_ALIAS_STATIC_PIE_CRT_OBJECT,
                TOOLCHAIN_ALIAS_STATIC_PIE_FLAG,
                "input.o",
            ])
            .status()
            .unwrap();

        let cc_alias = fs::read_to_string(guard_dir.join(C_COMPILER_ALIAS)).unwrap();
        let runtime_dir = guard_dir.join(TOOLCHAIN_ALIAS_RUNTIME_DIR);
        let runtime_unwind = runtime_dir.join(TOOLCHAIN_ALIAS_UNWIND_ARCHIVE);
        let runtime_crt1 = runtime_dir.join(TOOLCHAIN_ALIAS_CRT1_OBJECT);
        let observed = fs::read_to_string(observed_args).unwrap();
        assert!(status.success());
        assert!(cc_alias.contains("-L\"$runtime_dir\""));
        assert!(cc_alias.contains(TOOLCHAIN_ALIAS_RUNTIME_DIR));
        assert!(cc_alias.contains("rcrt1.o|*/rcrt1.o"));
        assert!(cc_alias.contains("static_pie_normalized=false"));
        assert!(cc_alias.contains("-static-pie) mapped_arg=\"-static\"; static_pie_normalized=true ;;"));
        assert!(cc_alias.contains("set -- \"$@\" -no-pie"));
        assert_eq!(fs::read(&runtime_unwind).unwrap(), fs::read(&tools.unwind_archive).unwrap());
        assert_eq!(fs::read(&runtime_crt1).unwrap(), fs::read(&tools.target_crt1).unwrap());
        assert!(observed.contains(&runtime_crt1.to_string_lossy().to_string()));
        assert!(observed.contains(TOOLCHAIN_ALIAS_STATIC_FLAG));
        assert!(observed.contains(TOOLCHAIN_ALIAS_NON_PIE_FLAG));
        assert!(!observed.contains(TOOLCHAIN_ALIAS_STATIC_PIE_CRT_OBJECT));
        assert!(!observed.contains(TOOLCHAIN_ALIAS_STATIC_PIE_FLAG));
        assert!(!guard_dir.join(CARGO_SHIM_NAME).exists());
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_c_compiler_alias_rewrites_response_file_runtime_inputs() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let observed_args = dir.path().join("observed-cc-response-args.txt");
        write_fake_executable(
            &tools.c_compiler,
            &format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > {}\nexit 0\n", shell_quote(&observed_args)),
        );
        let manifest = fake_toolchain_manifest(&tools, None);
        let guard_dir = dir.path().join("guard-bin");
        let response_file = dir.path().join("linker.rsp");
        fs::create_dir_all(&guard_dir).unwrap();
        fs::write(
            &response_file,
            format!("{}\n{}\ninput.o\n", TOOLCHAIN_ALIAS_STATIC_PIE_CRT_OBJECT, TOOLCHAIN_ALIAS_STATIC_PIE_FLAG),
        )
        .unwrap();

        write_toolchain_path_aliases(&guard_dir, &manifest).unwrap();
        let status = Command::new(guard_dir.join(C_COMPILER_ALIAS))
            .arg(format!("@{}", response_file.display()))
            .status()
            .unwrap();

        let runtime_dir = guard_dir.join(TOOLCHAIN_ALIAS_RUNTIME_DIR);
        let runtime_response = runtime_dir.join("response-1.rsp");
        let runtime_crt1 = runtime_dir.join(TOOLCHAIN_ALIAS_CRT1_OBJECT);
        let observed = fs::read_to_string(observed_args).unwrap();
        let rewritten = fs::read_to_string(&runtime_response).unwrap();
        assert!(status.success());
        assert!(observed.contains(&format!("@{}", runtime_response.display())));
        assert!(observed.contains(TOOLCHAIN_ALIAS_NON_PIE_FLAG));
        assert!(rewritten.contains(&runtime_crt1.to_string_lossy().to_string()));
        assert!(rewritten.contains(TOOLCHAIN_ALIAS_STATIC_FLAG));
        assert!(rewritten.contains("input.o"));
        assert!(!rewritten.contains(TOOLCHAIN_ALIAS_STATIC_PIE_CRT_OBJECT));
        assert!(!rewritten.contains(TOOLCHAIN_ALIAS_STATIC_PIE_FLAG));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_c_compiler_alias_rejects_missing_target_crt() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let mut manifest = fake_toolchain_manifest(&tools, None);
        manifest
            .members
            .retain(|member| member.name != crate::source_toolchain_closure::NATIVE_TARGET_CRT1_NAME);
        let guard_dir = dir.path().join("guard-bin");
        fs::create_dir_all(&guard_dir).unwrap();

        let err = write_toolchain_path_aliases(&guard_dir, &manifest).unwrap_err();

        assert!(err.message().contains("missing target crt1.o member"));
        assert!(!guard_dir.join(C_COMPILER_ALIAS).exists());
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_c_compiler_alias_rejects_ambiguous_target_crt() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let mut manifest = fake_toolchain_manifest(&tools, None);
        manifest.members.push(fake_member(
            crate::source_toolchain_closure::ToolchainRole::CrtObject,
            crate::source_toolchain_closure::NATIVE_TARGET_CRT1_NAME,
            &tools.target_crt1,
            None,
        ));
        let guard_dir = dir.path().join("guard-bin");
        fs::create_dir_all(&guard_dir).unwrap();

        let err = write_toolchain_path_aliases(&guard_dir, &manifest).unwrap_err();

        assert!(err.message().contains("duplicate member 'x86_64-linux-musl-crt1.o'"));
        assert!(!guard_dir.join(C_COMPILER_ALIAS).exists());
    }

    #[cfg(unix)]
    #[test]
    fn compatibility_probe_uses_receipt_bound_path_for_explicit_closure() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let probe_report = dir.path().join("probe-path.txt");
        write_fake_executable(
            &tools.rustc,
            &format!("#!/bin/sh\nprintf '%s\\n' \"$PATH\" > {}\nexit 0\n", shell_quote(&probe_report)),
        );
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest);
        let bundle_dir = dir.path().join("bundle");

        let compatibility = prepare_rustc_compatibility(&bundle_dir, &tools.rustc, &closure).unwrap();
        let recorded_path = fs::read_to_string(&probe_report).unwrap();

        assert_eq!(compatibility.summary.normalization, NORMALIZATION_NONE);
        assert!(recorded_path.contains(TOOLCHAIN_COMPATIBILITY_PATH_DIR));
        assert!(!recorded_path.contains("/run/current-system/sw"));
    }

    #[cfg(unix)]
    #[test]
    fn compatibility_probe_failure_with_explicit_closure_fails_closed() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        write_fake_executable(&tools.rustc, "#!/bin/sh\nexit 42\n");
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest);
        let bundle_dir = dir.path().join("bundle");

        let err = prepare_rustc_compatibility(&bundle_dir, &tools.rustc, &closure).unwrap_err();

        assert!(err.message().contains("receipt-bound toolchain PATH"));
        assert!(!bundle_dir.join(TOOLCHAIN_DIR).join(RUSTC_WRAPPER_FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn fixed_point_preflight_rejects_undeclared_external_rustc_wrapper() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let wrapper = dir.path().join("outside-rustc-wrapper");
        write_fake_executable(&wrapper, &format!("#!/bin/sh\nexec {} \"$@\"\n", shell_quote(&tools.rustc)));
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest);

        let err = prepare_rustc_for_compatibility(&wrapper, &closure).unwrap_err();

        assert!(err.message().contains(EXTERNAL_WRAPPER_BLOCKER));
        assert!(err.message().contains("not a declared source-built toolchain closure member"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_uses_original_rustc_identity_and_runtime_sysroot() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());
        let runtime_wrapper = dir.path().join("runtime-rustc");
        write_fake_executable(&runtime_wrapper, &format!("#!/bin/sh\nexec {} \"$@\"\n", shell_quote(&tools.rustc)));

        let status =
            enforce_receipt_bound_toolchain_with_runtime(&tools.rustc, &runtime_wrapper, &closure, &manifest).unwrap();
        let leaked_sysroot = dir.path().join("leaked-sysroot");
        fs::create_dir(&leaked_sysroot).unwrap();
        write_fake_executable(&runtime_wrapper, &rustc_sysroot_script(&leaked_sysroot));
        let error = enforce_receipt_bound_toolchain_with_runtime(&tools.rustc, &runtime_wrapper, &closure, &manifest)
            .unwrap_err();

        assert_eq!(status.status, ENFORCED_SOURCE_BUILT_STATUS);
        assert!(status.claim);
        assert!(error.message().contains("host-tool-leakage"));
        assert!(error.message().contains("Sysroot"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_sysroot_leakage() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let leaked_sysroot = dir.path().join("leaked-sysroot");
        fs::create_dir_all(&leaked_sysroot).unwrap();
        let rustc_script = rustc_sysroot_script(&leaked_sysroot);
        write_fake_executable(&tools.rustc, &rustc_script);
        let manifest = fake_toolchain_manifest(&tools, None);
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"), "unexpected error: {}", err.message());
        assert!(err.message().contains("Sysroot"), "unexpected error: {}", err.message());
        assert!(err.message().contains(path_to_string(&leaked_sysroot).unwrap().as_str()));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_linker_digest_mismatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, Some(("ld", fake_digest())));
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("Linker"));
        assert!(err.message().contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_c_compiler_digest_mismatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, Some(("c-compiler", fake_digest())));
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("CCompiler"), "unexpected error: {}", err.message());
        assert!(err.message().contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_pkg_config_digest_mismatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, true);
        let manifest = fake_toolchain_manifest(&tools, Some((PKG_CONFIG_ALIAS, fake_digest())));
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"), "unexpected error: {}", err.message());
        assert!(err.message().contains("PkgConfig"), "unexpected error: {}", err.message());
        assert!(err.message().contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_enforcement_rejects_runtime_library_digest_mismatch() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let runtime = dir.path().join("toolchain").join("lib").join("libc.so");
        fs::create_dir_all(runtime.parent().unwrap()).unwrap();
        write_text(&runtime, "runtime-v1").unwrap();
        let mut manifest = fake_toolchain_manifest(&tools, None);
        manifest.members.push(fake_member(
            crate::source_toolchain_closure::ToolchainRole::RuntimeLibrary,
            "host-libc.so",
            &runtime,
            Some(&("host-libc.so", fake_digest())),
        ));
        let closure = loaded_toolchain_closure(dir.path().join("closure.json"), manifest.clone());

        let err = enforce_receipt_bound_toolchain(&tools.rustc, &closure, &manifest).unwrap_err();

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("RuntimeLibrary"));
        assert!(err.message().contains("digest mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_expose_declared_linker_for_collect2() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, None);
        let aliases = toolchain_path_aliases(&manifest).unwrap();

        assert_eq!(aliases.get("ld"), Some(&tools.linker));
        assert_eq!(aliases.get("cc"), Some(&tools.c_compiler));
        assert_eq!(aliases.get("c-compiler"), Some(&tools.c_compiler));
        assert_eq!(aliases.get(ARCHIVER_ALIAS), Some(&tools.archiver));
        assert_eq!(aliases.get(RANLIB_ALIAS), Some(&tools.ranlib));
        assert!(!aliases.contains_key("cargo"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_bind_the_selected_source_built_shell() {
        let dir = tempfile::tempdir().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, None);
        let guard_dir = dir.path().join("guard-bin");
        let shell = dir.path().join("host-tools/bin/sh");
        fs::create_dir_all(&guard_dir).unwrap();
        fs::create_dir_all(shell.parent().unwrap()).unwrap();
        write_fake_executable(&shell, "#!/bin/sh\nexit 0\n");

        write_toolchain_path_aliases_with_shell(&guard_dir, &manifest, &shell).unwrap();
        let linker_alias = fs::read_to_string(guard_dir.join(LINKER_ALIAS)).unwrap();
        let c_alias = fs::read_to_string(guard_dir.join(C_COMPILER_ALIAS)).unwrap();

        assert!(linker_alias.starts_with(&format!("#!{}\n", shell.display())));
        assert!(c_alias.starts_with(&format!("#!{}\n", shell.display())));
        assert!(!linker_alias.starts_with("#!/bin/sh\n"));
        assert!(!c_alias.starts_with("#!/bin/sh\n"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_expose_target_prefixed_member_name() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let target_tool = dir.path().join("toolchain").join("bin").join("musl-gcc-wrapper");
        write_fake_executable(&target_tool, "#!/bin/sh\nexit 0\n");
        let mut manifest = fake_toolchain_manifest(&tools, None);
        manifest.members.push(fake_member(
            crate::source_toolchain_closure::ToolchainRole::NativeHelper,
            MUSL_TARGET_GCC_ALIAS,
            &target_tool,
            None,
        ));

        let aliases = toolchain_path_aliases(&manifest).unwrap();

        assert_eq!(aliases.get(MUSL_TARGET_GCC_ALIAS), Some(&target_tool));
        assert_eq!(aliases.get(C_COMPILER_ALIAS), Some(&tools.c_compiler));
        assert_eq!(aliases.get("cc"), Some(&tools.c_compiler));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_reject_unsafe_member_name() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let mut manifest = fake_toolchain_manifest(&tools, None);
        manifest
            .members
            .iter_mut()
            .find(|member| member.role == crate::source_toolchain_closure::ToolchainRole::CCompiler)
            .unwrap()
            .name = "bin/gcc".to_string();

        let err = toolchain_path_aliases(&manifest).unwrap_err();

        assert!(err.message().contains("unsafe PATH alias"));
        assert!(err.message().contains("bin/gcc"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_reject_member_name_conflict() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let mut manifest = fake_toolchain_manifest(&tools, None);
        manifest
            .members
            .iter_mut()
            .find(|member| member.role == crate::source_toolchain_closure::ToolchainRole::CCompiler)
            .unwrap()
            .name = MUSL_TARGET_GCC_ALIAS.to_string();
        manifest.members.push(fake_member(
            crate::source_toolchain_closure::ToolchainRole::NativeHelper,
            MUSL_TARGET_GCC_ALIAS,
            &tools.archiver,
            None,
        ));

        let err = toolchain_path_aliases(&manifest).unwrap_err();

        assert!(err.message().contains("PATH alias"));
        assert!(err.message().contains("conflicting targets"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_expose_declared_pkg_config_only() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, true);
        let manifest = fake_toolchain_manifest(&tools, None);
        let aliases = toolchain_path_aliases(&manifest).unwrap();

        let pkg_config = tools.pkg_config.as_ref().unwrap();
        assert!(aliases.contains_key(PKG_CONFIG_ALIAS));
        assert_eq!(aliases.get(PKG_CONFIG_ALIAS), Some(pkg_config));
        assert!(!aliases.contains_key("nix"));
        assert!(!aliases.contains_key("nix-store"));
    }

    #[cfg(unix)]
    #[test]
    fn receipt_bound_path_aliases_reject_undeclared_nix_profile_tools() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let profile_dir = dir.path().join("nix-profile-bin");
        fs::create_dir_all(&profile_dir).unwrap();
        write_fake_executable(&profile_dir.join("nix"), "#!/bin/sh\nexit 99\n");
        write_fake_executable(&profile_dir.join("nix-store"), "#!/bin/sh\nexit 99\n");
        let manifest = fake_toolchain_manifest(&tools, None);
        let guard_dir = dir.path().join("guard-bin");
        fs::create_dir_all(&guard_dir).unwrap();

        write_toolchain_path_aliases(&guard_dir, &manifest).unwrap();
        let aliases = toolchain_path_aliases(&manifest).unwrap();

        assert!(profile_dir.join("nix").is_file());
        assert!(!aliases.contains_key("nix"));
        assert!(!aliases.contains_key("nix-store"));
        assert!(!guard_dir.join("nix").exists());
        assert!(!guard_dir.join("nix-store").exists());
    }

    #[cfg(unix)]
    #[derive(Debug)]
    struct FakeToolchain {
        rustc: PathBuf,
        linker: PathBuf,
        c_compiler: PathBuf,
        archiver: PathBuf,
        ranlib: PathBuf,
        sysroot: PathBuf,
        pkg_config: Option<PathBuf>,
        unwind_archive: PathBuf,
        target_crt1: PathBuf,
    }

    #[cfg(unix)]
    fn write_fake_rust_source_provider(root: &Path, prebuilt: bool) {
        write_provider_bytes(root, RUST_PROVIDER_RUSTC_PATH, b"rustc");
        set_executable(&root.join(RUST_PROVIDER_RUSTC_PATH)).unwrap();
        write_provider_bytes(root, RUST_PROVIDER_CARGO_PATH, b"cargo");
        write_provider_bytes(root, RUST_PROVIDER_HOST_RUSTLIB_PATH, b"host-std");
        write_provider_bytes(root, RUST_PROVIDER_TARGET_RUSTLIB_PATH, b"target-std");

        let receipt = fake_rust_provider_receipt(root);
        write_provider_json(&root.join(RUST_PROVIDER_RECEIPT_PATH), &receipt);
        let metadata = fake_rust_provider_metadata(root, prebuilt);
        write_provider_json(&root.join(RUST_PROVIDER_METADATA_PATH), &metadata);
    }

    #[cfg(unix)]
    fn fake_rust_provider_receipt(root: &Path) -> crate::source_toolchain_closure::RustSourceProviderBuildReceipt {
        use crate::source_toolchain_closure::*;

        RustSourceProviderBuildReceipt {
            schema: RUST_SOURCE_PROVIDER_RECEIPT_SCHEMA.to_string(),
            receipt_id: RUST_PROVIDER_RECEIPT_ID.to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: RUST_PROVIDER_HOST_TRIPLE.to_string(),
            target_triple: RUST_PROVIDER_TARGET_TRIPLE.to_string(),
            source_ids: vec![RUST_PROVIDER_SOURCE_ID.to_string()],
            output_artifacts: fake_rust_provider_receipt_artifacts(root),
            build_steps: vec![RustProviderReceiptStep {
                name: "compile-rust-from-source".to_string(),
                program: RUST_PROVIDER_STAGE_PROGRAM.to_string(),
                arguments: vec![RUST_PROVIDER_BUILD_RECIPE.to_string()],
            }],
        }
    }

    #[cfg(unix)]
    fn fake_rust_provider_metadata(
        root: &Path,
        prebuilt: bool,
    ) -> crate::source_toolchain_closure::RustSourceProviderMetadata {
        use crate::source_toolchain_closure::*;

        RustSourceProviderMetadata {
            schema: RUST_SOURCE_PROVIDER_SCHEMA.to_string(),
            provider_id: RUST_SOURCE_PROVIDER_ID.to_string(),
            host_triple: RUST_PROVIDER_HOST_TRIPLE.to_string(),
            target_triple: RUST_PROVIDER_TARGET_TRIPLE.to_string(),
            provenance: RustSourceProviderProvenance {
                source_built: !prebuilt,
                uses_prebuilt_rust: prebuilt,
                build_recipe: RUST_PROVIDER_BUILD_RECIPE.to_string(),
            },
            sources: vec![RustProviderSourceIdentity {
                id: RUST_PROVIDER_SOURCE_ID.to_string(),
                kind: ToolchainSourceKind::Tarball,
                name: "rust-compiler-source".to_string(),
                digest_blake3: blake3::hash(b"source").to_hex().to_string(),
            }],
            build_receipts: vec![RustProviderBuildReceiptIdentity {
                id: RUST_PROVIDER_RECEIPT_ID.to_string(),
                kind: ToolchainBuildReceiptKind::MantleDerivation,
                name: "rust-source-build-receipt".to_string(),
                path: RUST_PROVIDER_RECEIPT_PATH.to_string(),
                digest_blake3: blake3_file(&root.join(RUST_PROVIDER_RECEIPT_PATH)).unwrap(),
            }],
            artifacts: fake_rust_provider_artifacts(root),
        }
    }

    #[cfg(unix)]
    fn fake_rust_provider_artifacts(root: &Path) -> Vec<crate::source_toolchain_closure::RustProviderArtifact> {
        use crate::source_toolchain_closure::RustProviderRole;

        vec![
            rust_provider_artifact(root, RustProviderRole::Rustc, "rustc", RUST_PROVIDER_RUSTC_PATH),
            rust_provider_artifact(root, RustProviderRole::Cargo, "cargo", RUST_PROVIDER_CARGO_PATH),
            rust_provider_artifact(
                root,
                RustProviderRole::HostRustlib,
                "host-rustlib",
                RUST_PROVIDER_HOST_RUSTLIB_PATH,
            ),
            rust_provider_artifact(
                root,
                RustProviderRole::TargetRustlib,
                "target-rustlib",
                RUST_PROVIDER_TARGET_RUSTLIB_PATH,
            ),
            rust_provider_artifact(
                root,
                RustProviderRole::ProviderReceipt,
                RUST_PROVIDER_RECEIPT_ID,
                RUST_PROVIDER_RECEIPT_PATH,
            ),
        ]
    }

    #[cfg(unix)]
    fn fake_rust_provider_receipt_artifacts(
        root: &Path,
    ) -> Vec<crate::source_toolchain_closure::RustProviderReceiptArtifact> {
        use crate::source_toolchain_closure::RustProviderRole;

        vec![
            rust_provider_receipt_artifact(root, RustProviderRole::Rustc, "rustc", RUST_PROVIDER_RUSTC_PATH),
            rust_provider_receipt_artifact(root, RustProviderRole::Cargo, "cargo", RUST_PROVIDER_CARGO_PATH),
            rust_provider_receipt_artifact(
                root,
                RustProviderRole::HostRustlib,
                "host-rustlib",
                RUST_PROVIDER_HOST_RUSTLIB_PATH,
            ),
            rust_provider_receipt_artifact(
                root,
                RustProviderRole::TargetRustlib,
                "target-rustlib",
                RUST_PROVIDER_TARGET_RUSTLIB_PATH,
            ),
        ]
    }

    #[cfg(unix)]
    fn rust_provider_artifact(
        root: &Path,
        role: crate::source_toolchain_closure::RustProviderRole,
        name: &str,
        path: &str,
    ) -> crate::source_toolchain_closure::RustProviderArtifact {
        crate::source_toolchain_closure::RustProviderArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: blake3_file(&root.join(path)).unwrap(),
            source_id: "rust-src".to_string(),
            build_receipt_id: "build-receipt".to_string(),
        }
    }

    #[cfg(unix)]
    fn rust_provider_receipt_artifact(
        root: &Path,
        role: crate::source_toolchain_closure::RustProviderRole,
        name: &str,
        path: &str,
    ) -> crate::source_toolchain_closure::RustProviderReceiptArtifact {
        crate::source_toolchain_closure::RustProviderReceiptArtifact {
            role,
            name: name.to_string(),
            path: path.to_string(),
            content_digest_blake3: blake3_file(&root.join(path)).unwrap(),
        }
    }

    #[cfg(unix)]
    fn write_provider_bytes(root: &Path, relative: &str, bytes: &[u8]) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[cfg(unix)]
    fn write_provider_json<T: serde::Serialize>(path: &Path, value: &T) {
        let bytes = serde_json::to_vec_pretty(value).unwrap();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    #[cfg(unix)]
    fn fake_toolchain(dir: &tempfile::TempDir, include_pkg_config: bool) -> FakeToolchain {
        let tool_dir = dir.path().join("toolchain").join("bin");
        let sysroot = dir.path().join("toolchain").join("sysroot");
        let runtime_dir = dir.path().join("toolchain").join("lib");
        fs::create_dir_all(&tool_dir).unwrap();
        fs::create_dir_all(&sysroot).unwrap();
        fs::create_dir_all(&runtime_dir).unwrap();
        let rustc = tool_dir.join("rustc");
        let linker = tool_dir.join("ld");
        let c_compiler = tool_dir.join("cc");
        let archiver = tool_dir.join(ARCHIVER_ALIAS);
        let ranlib = tool_dir.join(RANLIB_ALIAS);
        let unwind_archive = runtime_dir.join("libgcc_eh.a");
        let target_crt1 = runtime_dir.join(TOOLCHAIN_ALIAS_CRT1_OBJECT);
        write_fake_executable(&rustc, &rustc_sysroot_script(&sysroot));
        write_fake_executable(&linker, "#!/bin/sh\nexit 0\n");
        write_fake_executable(&c_compiler, "#!/bin/sh\nexit 0\n");
        write_fake_executable(&archiver, "#!/bin/sh\nexit 0\n");
        write_fake_executable(&ranlib, "#!/bin/sh\nexit 0\n");
        write_text(&unwind_archive, "fake unwind archive\n").unwrap();
        write_text(&target_crt1, "fake target crt1\n").unwrap();
        let pkg_config = include_pkg_config.then(|| {
            let path = tool_dir.join(PKG_CONFIG_ALIAS);
            write_fake_executable(&path, "#!/bin/sh\nexit 0\n");
            path
        });
        FakeToolchain {
            rustc,
            linker,
            c_compiler,
            archiver,
            ranlib,
            sysroot,
            pkg_config,
            unwind_archive,
            target_crt1,
        }
    }

    #[cfg(unix)]
    fn has_ordered_os_arg_pair(args: &[OsString], flag: &str, value: &str) -> bool {
        args.windows(2).any(|window| window[0] == OsStr::new(flag) && window[1] == OsStr::new(value))
    }

    fn fake_toolchain_manifest(
        tools: &FakeToolchain,
        override_digest: Option<(&str, String)>,
    ) -> crate::source_toolchain_closure::ToolchainClosureManifest {
        use crate::source_toolchain_closure::*;
        let mut members = vec![
            fake_member(ToolchainRole::Rustc, "rustc", &tools.rustc, override_digest.as_ref()),
            fake_member(ToolchainRole::Linker, "ld", &tools.linker, override_digest.as_ref()),
            fake_member(ToolchainRole::CCompiler, "c-compiler", &tools.c_compiler, override_digest.as_ref()),
            fake_member(ToolchainRole::NativeHelper, ARCHIVER_ALIAS, &tools.archiver, override_digest.as_ref()),
            fake_member(ToolchainRole::NativeHelper, RANLIB_ALIAS, &tools.ranlib, override_digest.as_ref()),
            fake_member(
                ToolchainRole::RuntimeLibrary,
                NATIVE_HOST_LIBUNWIND_NAME,
                &tools.unwind_archive,
                override_digest.as_ref(),
            ),
            fake_member(
                ToolchainRole::CrtObject,
                NATIVE_TARGET_CRT1_NAME,
                &tools.target_crt1,
                override_digest.as_ref(),
            ),
            fake_member(ToolchainRole::Sysroot, "sysroot", &tools.sysroot, override_digest.as_ref()),
        ];
        if let Some(pkg_config) = &tools.pkg_config {
            members.push(fake_member(ToolchainRole::PkgConfig, PKG_CONFIG_ALIAS, pkg_config, override_digest.as_ref()));
        }
        ToolchainClosureManifest {
            schema: SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA.to_string(),
            members,
            seed_exceptions: Vec::new(),
        }
    }

    #[cfg(unix)]
    fn fake_member(
        role: crate::source_toolchain_closure::ToolchainRole,
        name: &str,
        path: &Path,
        override_digest: Option<&(&str, String)>,
    ) -> crate::source_toolchain_closure::ToolchainClosureMember {
        use crate::source_toolchain_closure::*;
        let content_digest_blake3 = override_digest
            .filter(|(target_name, _digest)| *target_name == name)
            .map(|(_target_name, digest)| digest.clone())
            .unwrap_or_else(|| {
                if path.is_file() {
                    blake3_file(path).unwrap()
                } else {
                    fake_digest()
                }
            });
        ToolchainClosureMember {
            role,
            name: name.to_string(),
            execution_path: path_to_string(path).unwrap(),
            content_digest_blake3,
            trust: ToolchainTrust::SourceBuilt,
            source: Some(ToolchainSourceIdentity {
                kind: ToolchainSourceKind::Generated,
                name: format!("{name}-source"),
                digest_blake3: fake_digest(),
            }),
            build_receipt: Some(ToolchainBuildReceiptIdentity {
                kind: ToolchainBuildReceiptKind::MantleRustTopology,
                name: format!("{name}-receipt"),
                digest_blake3: fake_digest(),
            }),
        }
    }

    #[cfg(unix)]
    fn loaded_toolchain_closure(
        manifest_path: PathBuf,
        manifest: crate::source_toolchain_closure::ToolchainClosureManifest,
    ) -> LoadedToolchainClosure {
        let validation = crate::source_toolchain_closure::validate_toolchain_closure_manifest(&manifest).unwrap();
        LoadedToolchainClosure {
            status: crate::source_toolchain_closure::validated_source_built_toolchain_closure(
                manifest_path.clone(),
                &validation,
            ),
            manifest_path: Some(manifest_path),
            manifest: Some(manifest),
        }
    }

    #[cfg(unix)]
    fn write_fake_executable(path: &Path, contents: &str) {
        write_text(path, contents).unwrap();
        set_executable(path).unwrap();
    }

    #[cfg(unix)]
    fn rustc_sysroot_script(sysroot: &Path) -> String {
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"--print\" ] && [ \"$2\" = \"{RUSTC_SYSROOT_PRINT_ARG}\" ]; then\n  printf '%s\\n' {}\n  exit 0\nfi\nexit 0\n",
            shell_quote(sysroot)
        )
    }

    #[cfg(unix)]
    fn fake_digest() -> String {
        "abababababababababababababababababababababababababababababababab".to_string()
    }
}
