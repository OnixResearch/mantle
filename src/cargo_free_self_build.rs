use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs;
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
const CARGO_SHIM_FILE: &str = "cargo-forbidden";
const CARGO_SHIM_DIR: &str = "cargo-guard-bin";
const CARGO_SHIM_NAME: &str = "cargo";
const CARGO_MARKER_FILE: &str = "cargo-was-invoked";
const C_COMPILER_ALIAS: &str = "cc";
const LINKER_ALIAS: &str = "ld";
const ARCHIVER_ALIAS: &str = "ar";
const RANLIB_ALIAS: &str = "ranlib";
const PKG_CONFIG_ALIAS: &str = "pkg-config";
const MUSL_TARGET_GCC_ALIAS: &str = "x86_64-linux-musl-gcc";
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
const EXECUTE_TOPOLOGY_FLAG: &str = "--execute-topology";
const EXECUTION_OUTPUT_ROOT_FLAG: &str = "--execution-output-root";
const HELP_FLAG: &str = "--help";
const TOOLCHAIN_DIR: &str = "toolchain";
const RUSTC_WRAPPER_FILE: &str = "rustc-normalized";
const COMPATIBILITY_FILE: &str = "compatibility.json";
const TOOLCHAIN_COMPATIBILITY_PATH_DIR: &str = "receipt-bound-path";
const TOOLCHAIN_ALIAS_RUNTIME_DIR: &str = ".toolchain-runtime";
const TOOLCHAIN_ALIAS_UNWIND_ARCHIVE: &str = "libunwind.a";
const RUSTC_PROBE_DIR: &str = "rustc-probe";
const RUSTC_PROBE_SOURCE_FILE: &str = "probe.rs";
const RUSTC_PROBE_SOURCE: &str = "fn main() {}\n";
const LINK_SELF_CONTAINED_PROBE_ARG: &str = "link-self-contained=no";
const LINK_SELF_CONTAINED_JOINED_ARG: &str = "-Clink-self-contained=no";
const RUSTC_BOOTSTRAP_ENV: &str = "RUSTC_BOOTSTRAP";
const REAL_RUSTC_ENV: &str = "MANTLE_REAL_RUSTC";
const NORMALIZATION_NONE: &str = "none";
const NORMALIZATION_STRIP_LINK_SELF_CONTAINED: &str = "strip-link-self-contained-no";
const SIGNAL_STATUS_TEXT: &str = "signal";
const BLOCKED_SMOKE_STDOUT: &str = "not run: blocked before binary\n";
const BLOCKED_SMOKE_STDERR_PREFIX: &str = "not run: blocked before binary";
const SOURCE_DIGEST_FIELD: &str = "source_digest";
const SOURCE_DIGEST_ALGORITHM_FIELD: &str = "algorithm";
const RUSTC_SYSROOT_PRINT_ARG: &str = "sysroot";
const TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD: &str = "source_built_toolchain_closure_policy_digest_blake3";
const RUST_SOURCE_PROVIDER_BINDING_SCHEMA: &str = "mantle-cargo-free-rust-source-provider-binding-v1";
const RUST_SOURCE_PROVIDER_STATUS_ABSENT: &str = "absent";
const RUST_SOURCE_PROVIDER_STATUS_VALIDATED: &str = "validated";
const RUST_SOURCE_PROVIDER_REQUIRED_ROLE_COUNT: usize = 1;
const PROVIDER_FIXED_POINT_STATUS_ABSENT: &str = "absent";
const PROVIDER_FIXED_POINT_STATUS_VALID: &str = "valid";
const PROVIDER_FIXED_POINT_STATUS_INVALID: &str = "invalid";
const ENFORCED_SOURCE_BUILT_STATUS: &str = "enforced-source-built";
const NOT_CRUNCH_BOOTSTRAP_NON_CLAIM: &str = "not-crunch-bootstrap";
const NOT_RELEASE_REPRODUCIBILITY_NON_CLAIM: &str = "not-release-reproducibility";
const NOT_FULL_CARGO_COMPATIBILITY_NON_CLAIM: &str = "not-full-cargo-compatibility";
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
    toolchain_closure_status: Option<crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus>,
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

#[derive(Clone, Debug)]
struct ExecutionToolchain {
    rustc: PathBuf,
    path_env: OsString,
    status: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
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
    blocker: Option<String>,
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
    stage1: FixedPointStageSummary,
    stage2: Option<FixedPointStageSummary>,
    rustc_compatibility: RustcCompatibilitySummary,
    source_built_toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: RustSourceProviderBindingStatus,
    blocker: Option<String>,
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
    blocker: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct ProviderFixedPointProofVerification {
    pub(crate) status: String,
    pub(crate) valid: bool,
    pub(crate) proof_dir: Option<PathBuf>,
    pub(crate) meta_digest_blake3: Option<String>,
    pub(crate) closure_policy_digest_blake3: Option<String>,
    pub(crate) stage_binary_digest_blake3: Option<String>,
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
            proof_dir: None,
            meta_digest_blake3: None,
            closure_policy_digest_blake3: None,
            stage_binary_digest_blake3: None,
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
    blocker: Option<String>,
    source_built_toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: RustSourceProviderBindingStatus,
    non_claims: Vec<&'static str>,
}

pub(crate) fn verify_provider_fixed_point_proof_bundle(proof_dir: &Path) -> ProviderFixedPointProofVerification {
    let mut shell_blockers = Vec::new();
    let meta_path = proof_dir.join(META_FILE);
    let preflight_path = proof_dir.join(PRE_FLIGHT_FILE);
    let non_claims_path = proof_dir.join(NON_CLAIMS_FILE);
    let (meta, meta_digest_blake3) = read_json_with_digest(&meta_path, "fixed-point meta", &mut shell_blockers);
    let (preflight, _) = read_json_with_digest(&preflight_path, "fixed-point preflight", &mut shell_blockers);
    let non_claims_text = read_text_optional(&non_claims_path, "fixed-point non-claims", &mut shell_blockers);
    let stage1_binary_digest_actual =
        stage_binary_digest_from_meta(proof_dir, meta.as_ref(), "/stage1/binary", "stage1 binary", &mut shell_blockers);
    let stage2_binary_digest_actual =
        stage_binary_digest_from_meta(proof_dir, meta.as_ref(), "/stage2/binary", "stage2 binary", &mut shell_blockers);
    let stage1_receipt =
        stage_receipt_from_meta(proof_dir, meta.as_ref(), "/stage1/receipt", "stage1 receipt", &mut shell_blockers);
    let stage2_receipt =
        stage_receipt_from_meta(proof_dir, meta.as_ref(), "/stage2/receipt", "stage2 receipt", &mut shell_blockers);
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
    let mut blockers = std::mem::take(&mut evidence.shell_blockers);
    let Some(meta) = evidence.meta.as_ref() else {
        blockers.push("fixed-point meta.json is missing or invalid".to_string());
        return provider_fixed_point_result(evidence, None, None, None, blockers);
    };
    validate_fixed_point_meta(meta, &mut blockers);
    validate_fixed_point_preflight(meta, evidence.preflight.as_ref(), &mut blockers);
    validate_fixed_point_non_claims(meta, evidence.non_claims_text.as_deref(), &mut blockers);
    let closure_policy_digest = fixed_point_closure_policy_digest(meta, &mut blockers);
    let stage1 = validate_fixed_point_stage(
        meta,
        "stage1",
        evidence.stage1_binary_digest_actual.as_deref(),
        evidence.stage1_receipt.as_ref(),
        closure_policy_digest.as_deref(),
        &mut blockers,
    );
    let stage2 = validate_fixed_point_stage(
        meta,
        "stage2",
        evidence.stage2_binary_digest_actual.as_deref(),
        evidence.stage2_receipt.as_ref(),
        closure_policy_digest.as_deref(),
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
    let valid = blockers.is_empty();
    let (stage1_unit_count, stage2_unit_count) = stage_unit_counts.unwrap_or((None, None));
    let non_claims = non_claims_from_meta(evidence.meta.as_ref());
    let proof_dir = evidence.proof_dir;
    let meta_digest_blake3 = evidence.meta_digest_blake3;
    ProviderFixedPointProofVerification {
        status: if valid {
            PROVIDER_FIXED_POINT_STATUS_VALID
        } else {
            PROVIDER_FIXED_POINT_STATUS_INVALID
        }
        .to_string(),
        valid,
        proof_dir: Some(proof_dir),
        meta_digest_blake3,
        closure_policy_digest_blake3: closure_policy_digest,
        stage_binary_digest_blake3: stage_binary_digest,
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
    expect_string(meta, "/schema", FIXED_POINT_SCHEMA, "fixed-point schema", blockers);
    expect_string(meta, "/status", SUCCESS_STATUS, "fixed-point status", blockers);
    expect_bool(meta, "/fixed_point", true, "fixed-point flag", blockers);
    expect_null(meta, "/blocker", "fixed-point blocker", blockers);
    expect_string(
        meta,
        "/source_built_toolchain_closure/status",
        ENFORCED_SOURCE_BUILT_STATUS,
        "source-built closure status",
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
        "/rust_source_provider/status",
        RUST_SOURCE_PROVIDER_STATUS_VALIDATED,
        "Rust source provider status",
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
}

fn validate_fixed_point_preflight(meta: &Value, preflight: Option<&Value>, blockers: &mut Vec<String>) {
    let Some(preflight) = preflight else {
        blockers.push("fixed-point preflight.json is missing or invalid".to_string());
        return;
    };
    expect_string(preflight, "/schema", FIXED_POINT_SCHEMA, "preflight schema", blockers);
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
}

fn fixed_point_closure_policy_digest(meta: &Value, blockers: &mut Vec<String>) -> Option<String> {
    let policy = optional_str(meta, "/source_built_toolchain_closure/policy_digest_blake3");
    if policy.is_none() {
        blockers.push("fixed-point closure policy digest is missing".to_string());
    }
    policy.map(ToOwned::to_owned)
}

fn validate_fixed_point_stage(
    meta: &Value,
    stage_name: &str,
    actual_binary_digest: Option<&str>,
    receipt: Option<&Value>,
    expected_policy_digest: Option<&str>,
    blockers: &mut Vec<String>,
) -> StageProofFacts {
    let prefix = format!("/{stage_name}");
    expect_string(meta, &format!("{prefix}/name"), stage_name, "fixed-point stage name", blockers);
    expect_bool(meta, &format!("{prefix}/success"), true, "fixed-point stage success", blockers);
    expect_i64(
        meta,
        &format!("{prefix}/status_code"),
        SUCCESS_EXIT_CODE as i64,
        "fixed-point stage status code",
        blockers,
    );
    expect_string(
        meta,
        &format!("{prefix}/execution_status"),
        SUCCESS_STATUS,
        "fixed-point stage execution status",
        blockers,
    );
    expect_bool(meta, &format!("{prefix}/cargo_marker_absent"), true, "fixed-point stage Cargo guard", blockers);
    expect_u64(meta, &format!("{prefix}/failed_unit_count"), 0, "fixed-point stage failed unit count", blockers);
    expect_i64(
        meta,
        &format!("{prefix}/smoke_status_code"),
        SUCCESS_EXIT_CODE as i64,
        "fixed-point stage smoke status",
        blockers,
    );
    expect_null(meta, &format!("{prefix}/blocker"), "fixed-point stage blocker", blockers);
    let unit_count = optional_u64(meta, &format!("{prefix}/unit_count"));
    if !matches!(unit_count, Some(count) if count > 0) {
        blockers.push(format!("{stage_name} unit count is missing or zero"));
    }
    let declared_digest = optional_str(meta, &format!("{prefix}/binary_blake3")).map(ToOwned::to_owned);
    match (declared_digest.as_deref(), actual_binary_digest) {
        (Some(declared), Some(actual)) if declared == actual => {}
        (Some(declared), Some(actual)) => {
            blockers.push(format!("{stage_name} binary digest mismatch: declared {declared} actual {actual}"))
        }
        (Some(_), None) => blockers.push(format!("{stage_name} binary could not be hashed")),
        (None, _) => blockers.push(format!("{stage_name} binary digest is missing")),
    }
    let stage_policy = optional_str(meta, &format!("{prefix}/{TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD}"));
    if let (Some(stage_policy), Some(expected_policy)) = (stage_policy, expected_policy_digest) {
        if stage_policy != expected_policy {
            blockers.push(format!("{stage_name} closure policy digest does not match the proof closure digest"));
        }
    } else {
        blockers.push(format!("{stage_name} closure policy digest is missing"));
    }
    validate_fixed_point_stage_receipt(stage_name, receipt, unit_count, blockers);
    StageProofFacts {
        binary_digest: declared_digest,
        unit_count,
    }
}

fn validate_fixed_point_stage_receipt(
    stage_name: &str,
    receipt: Option<&Value>,
    expected_unit_count: Option<u64>,
    blockers: &mut Vec<String>,
) {
    let Some(receipt) = receipt else {
        blockers.push(format!("{stage_name} receipt is missing or invalid"));
        return;
    };
    expect_string(
        receipt,
        "/topology_execution/execution_status",
        SUCCESS_STATUS,
        "stage receipt execution status",
        blockers,
    );
    let Some(units) = receipt.pointer("/topology_execution/unit_executions").and_then(Value::as_array) else {
        blockers.push(format!("{stage_name} receipt missing unit executions"));
        return;
    };
    if let Some(expected_unit_count) = expected_unit_count {
        if units.len() as u64 != expected_unit_count {
            blockers.push(format!(
                "{stage_name} receipt unit count {} does not match summary unit count {expected_unit_count}",
                units.len()
            ));
        }
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
    pointer: &str,
    label: &str,
    blockers: &mut Vec<String>,
) -> Option<String> {
    let path = proof_path_from_meta(proof_dir, meta, pointer, label, blockers)?;
    match blake3_file(&path) {
        Ok(digest) => Some(digest),
        Err(err) => {
            blockers.push(format!("hash {label} {}: {}", path.display(), err.message()));
            None
        }
    }
}

fn stage_receipt_from_meta(
    proof_dir: &Path,
    meta: Option<&Value>,
    pointer: &str,
    label: &str,
    blockers: &mut Vec<String>,
) -> Option<Value> {
    let path = proof_path_from_meta(proof_dir, meta, pointer, label, blockers)?;
    let (value, _) = read_json_with_digest(&path, label, blockers);
    value
}

fn proof_path_from_meta(
    proof_dir: &Path,
    meta: Option<&Value>,
    pointer: &str,
    label: &str,
    blockers: &mut Vec<String>,
) -> Option<PathBuf> {
    let Some(meta) = meta else {
        return None;
    };
    let Some(raw) = optional_str(meta, pointer) else {
        blockers.push(format!("fixed-point meta missing {label} path at {pointer}"));
        return None;
    };
    let path = Path::new(raw);
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        Some(proof_dir.join(path))
    }
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

fn expect_string(value: &Value, pointer: &str, expected: &str, label: &str, blockers: &mut Vec<String>) {
    match optional_str(value, pointer) {
        Some(actual) if actual == expected => {}
        Some(actual) => blockers.push(format!("{label} expected {expected}, got {actual}")),
        None => blockers.push(format!("{label} is missing at {pointer}")),
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

fn expect_null(value: &Value, pointer: &str, label: &str, blockers: &mut Vec<String>) {
    match value.pointer(pointer) {
        Some(value) if value.is_null() => {}
        Some(value) => blockers.push(format!("{label} expected null, got {value}")),
        None => blockers.push(format!("{label} is missing at {pointer}")),
    }
}

pub(crate) fn cmd_cargo_free_self_build(options: CargoFreeSelfBuildOptions<'_>) -> Result<(), RunError> {
    let paths = prepare_paths(options.root, options.out_dir)?;
    prepare_output_dir(&paths)?;
    write_cargo_shim(&paths.explicit_cargo_shim, &paths.marker_path)?;
    write_cargo_shim(&paths.path_cargo_shim, &paths.marker_path)?;
    let loaded_rust_provider = load_rust_source_provider(options.rust_source_provider)?;
    let loaded_toolchain_closure = load_source_built_toolchain_closure(options.toolchain_closure)?;
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

    let mut child =
        run_rust_plan_child(&paths, &execution_toolchain.rustc, options.targets, &execution_toolchain.path_env)?;
    let produced = if child.blocker.is_none() {
        materialize_or_block(&paths, &mut child)?
    } else {
        None
    };
    if child.blocker.is_some() && produced.is_none() {
        write_blocked_smoke_outputs(&paths, child.blocker.as_deref())?;
    }
    let summary = summarize(&paths, &child, produced.as_ref(), execution_toolchain.status, loaded_rust_provider.status);
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
    prepare_fixed_point_output_dir(&bundle_dir)?;
    let loaded_rust_provider = load_rust_source_provider(options.rust_source_provider)?;
    let loaded_toolchain_closure = load_source_built_toolchain_closure(options.toolchain_closure)?;
    let requested_rustc = selected_cargo_free_rustc(&loaded_rust_provider, options.rustc);
    let compatibility_rustc = prepare_rustc_for_compatibility(requested_rustc, &loaded_toolchain_closure)?;
    let compatibility = prepare_rustc_compatibility(&bundle_dir, &compatibility_rustc, &loaded_toolchain_closure)?;
    let plan = plan_fixed_point_paths(&root, &bundle_dir, &compatibility.summary.stage_rustc, options.targets)?;
    let toolchain_status = enforce_fixed_point_toolchain(
        &compatibility.summary.stage_rustc,
        &loaded_toolchain_closure,
        &loaded_rust_provider,
    )?;
    write_fixed_point_non_claims(&plan.bundle_dir, &toolchain_status)?;
    write_fixed_point_preflight(&plan, &compatibility.summary, &toolchain_status, &loaded_rust_provider.status)?;

    let host_mantle = env::current_exe().map_err(|err| internal(format!("resolve current executable: {err}")))?;
    let stage_policy_digest = toolchain_status.policy_digest_blake3.as_deref();
    let stage1 = execute_fixed_point_stage(
        &plan.stages[FIXED_POINT_STAGE1_INDEX],
        &host_mantle,
        &loaded_toolchain_closure,
        stage_policy_digest,
    )?;
    if !stage1.success {
        return finish_fixed_point(
            options.json,
            &plan,
            &compatibility.summary,
            &toolchain_status,
            &loaded_rust_provider.status,
            stage1,
            None,
            BLOCKED_STATUS,
        );
    }
    let Some(stage1_binary) = stage1.binary.as_deref() else {
        let stage1 = blocked_fixed_point_stage(stage1, "stage1 succeeded without produced binary path".to_string());
        return finish_fixed_point(
            options.json,
            &plan,
            &compatibility.summary,
            &toolchain_status,
            &loaded_rust_provider.status,
            stage1,
            None,
            BLOCKED_STATUS,
        );
    };
    let stage2 = execute_fixed_point_stage(
        &plan.stages[FIXED_POINT_STAGE2_INDEX],
        stage1_binary,
        &loaded_toolchain_closure,
        stage_policy_digest,
    )?;
    if !stage2.success {
        return finish_fixed_point(
            options.json,
            &plan,
            &compatibility.summary,
            &toolchain_status,
            &loaded_rust_provider.status,
            stage1,
            Some(stage2),
            BLOCKED_STATUS,
        );
    }
    let status = fixed_point_status(&stage1, &stage2)?;
    finish_fixed_point(
        options.json,
        &plan,
        &compatibility.summary,
        &toolchain_status,
        &loaded_rust_provider.status,
        stage1,
        Some(stage2),
        status,
    )
}

fn prepare_paths(root: &Path, out_dir: &Path) -> Result<BuildPaths, RunError> {
    let root = canonicalize_root(root)?;
    let out_dir = absolutize(&root, out_dir);
    ensure_outside_root(&out_dir, &root)?;
    let execution_dir = out_dir.join(EXECUTION_DIR);
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
        fixed_point_stage_plan(
            STAGE1_DIR,
            root,
            &bundle_dir,
            &shared_execution_dir,
            rustc,
            targets,
            FixedPointMantleBinary::Host,
        ),
        fixed_point_stage_plan(
            STAGE2_DIR,
            root,
            &bundle_dir,
            &shared_execution_dir,
            rustc,
            targets,
            FixedPointMantleBinary::StageOutput {
                stage_name: STAGE1_DIR,
                path: stage1_binary_path,
            },
        ),
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

fn fixed_point_stage_plan(
    name: &'static str,
    root: &Path,
    bundle_dir: &Path,
    execution_dir: &Path,
    rustc: &Path,
    targets: &[String],
    mantle_binary: FixedPointMantleBinary,
) -> FixedPointStagePlan {
    debug_assert!(!name.is_empty());
    debug_assert!(root.is_absolute());
    debug_assert!(bundle_dir.is_absolute());
    let stage_dir = bundle_dir.join(name);
    let explicit_cargo_shim = stage_dir.join(CARGO_SHIM_FILE);
    let guard_path_dir = stage_dir.join(CARGO_SHIM_DIR);
    let path_cargo_shim = guard_path_dir.join(CARGO_SHIM_NAME);
    let command = FixedPointStageCommandPlan {
        mantle_binary,
        args: rust_plan_args(root, &explicit_cargo_shim, rustc, targets, execution_dir),
        current_dir: root.to_path_buf(),
        cargo_env_value: path_cargo_shim.clone(),
        path_guard_dir: guard_path_dir.clone(),
    };
    FixedPointStagePlan {
        name,
        stage_dir: stage_dir.clone(),
        execution_dir: execution_dir.to_path_buf(),
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
    let requested_rustc = resolve_executable(requested, "rustc")?;
    let toolchain_dir = bundle_dir.join(TOOLCHAIN_DIR);
    fs::create_dir_all(&toolchain_dir)
        .map_err(|err| internal(format!("create toolchain dir {}: {err}", toolchain_dir.display())))?;
    let probe_path_env = prepare_rustc_compatibility_path_env(&toolchain_dir, toolchain_closure)?;
    if rustc_accepts_link_self_contained_no(&requested_rustc, &toolchain_dir, probe_path_env.as_deref()) {
        let summary = RustcCompatibilitySummary {
            requested_rustc: requested_rustc.clone(),
            stage_rustc: requested_rustc,
            normalization: NORMALIZATION_NONE,
            wrapper: None,
            wrapper_blake3: None,
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

fn prepare_rustc_compatibility_path_env(
    toolchain_dir: &Path,
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<Option<OsString>, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(None);
    };
    let path_dir = toolchain_dir.join(TOOLCHAIN_COMPATIBILITY_PATH_DIR);
    remove_owned_path(&path_dir)?;
    fs::create_dir_all(&path_dir)
        .map_err(|err| internal(format!("create receipt-bound rustc probe PATH dir {}: {err}", path_dir.display())))?;
    write_toolchain_path_aliases(&path_dir, manifest)?;
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

fn rustc_accepts_link_self_contained_no(rustc: &Path, toolchain_dir: &Path, path_env: Option<&OsStr>) -> bool {
    let probe_dir = toolchain_dir.join(RUSTC_PROBE_DIR);
    let probe_source = probe_dir.join(RUSTC_PROBE_SOURCE_FILE);
    if remove_owned_path(&probe_dir).is_err() {
        return false;
    }
    if fs::create_dir_all(&probe_dir).is_err() {
        return false;
    }
    if fs::write(&probe_source, RUSTC_PROBE_SOURCE).is_err() {
        let _ = remove_owned_path(&probe_dir);
        return false;
    }
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
    let success = command.output().is_ok_and(|output| output.status.success());
    let _ = remove_owned_path(&probe_dir);
    success
}

fn write_rustc_wrapper(wrapper: &Path, real_rustc: &Path) -> Result<(), RunError> {
    let script = rustc_wrapper_script(real_rustc);
    write_text(wrapper, &script)?;
    set_executable(wrapper)
}

fn rustc_wrapper_script(real_rustc: &Path) -> String {
    format!(
        "#!/usr/bin/env bash\nset -euo pipefail\nexport {RUSTC_BOOTSTRAP_ENV}=1\nexport {REAL_RUSTC_ENV}={}\nargs=()\nwhile (($#)); do\n  arg=\"$1\"\n  shift\n  if [[ \"$arg\" == \"-C\" && \"${{1-}}\" == \"{LINK_SELF_CONTAINED_PROBE_ARG}\" ]]; then\n    shift\n    continue\n  fi\n  if [[ \"$arg\" == \"{LINK_SELF_CONTAINED_JOINED_ARG}\" ]]; then\n    continue\n  fi\n  args+=(\"$arg\")\ndone\nexec \"${REAL_RUSTC_ENV}\" \"${{args[@]}}\"\n",
        shell_quote(real_rustc)
    )
}

fn write_rustc_compatibility(path: &Path, summary: &RustcCompatibilitySummary) -> Result<(), RunError> {
    let value = json!(summary);
    let bytes = serde_json::to_vec_pretty(&value).map_err(|err| internal(format!("serialize compatibility: {err}")))?;
    write_bytes(path, &bytes)
}

fn prepare_output_dir(paths: &BuildPaths) -> Result<(), RunError> {
    fs::create_dir_all(&paths.out_dir)
        .map_err(|err| internal(format!("create output dir {}: {err}", paths.out_dir.display())))?;
    remove_owned_path(&paths.execution_dir)?;
    remove_owned_path(&paths.guard_path_dir)?;
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

fn run_rust_plan_child(
    paths: &BuildPaths,
    rustc: &Path,
    targets: &[String],
    path_env: &OsStr,
) -> Result<ChildRun, RunError> {
    let current_exe = env::current_exe().map_err(|err| internal(format!("resolve current executable: {err}")))?;
    let mut command = Command::new(&current_exe);
    command
        .arg("--json")
        .arg("rust-plan")
        .arg("--root")
        .arg(&paths.root)
        .arg("--cargo")
        .arg(&paths.explicit_cargo_shim)
        .arg("--rustc")
        .arg(rustc);
    for target in targets {
        command.arg("--target").arg(target);
    }
    let output = command
        .arg("--no-cargo-oracle")
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(&paths.execution_dir)
        .current_dir(&paths.root)
        .env("CARGO", &paths.path_cargo_shim)
        .env(RUSTC_BOOTSTRAP_ENV, "1")
        .env("PATH", path_env)
        .output()
        .map_err(|err| internal(format!("launch {} rust-plan: {err}", current_exe.display())))?;

    write_bytes(&paths.receipt_path, &output.stdout)?;
    write_bytes(&paths.stderr_path, &output.stderr)?;
    write_text(&paths.status_path, &status_text(output.status.code()))?;
    let receipt = parse_receipt(&paths.receipt_path, output.status.success())?;
    let execution_status = receipt_execution_status(receipt.as_ref());
    let cargo_marker_absent = !paths.marker_path.exists();
    let blocker = child_blocker(output.status.code(), &execution_status, cargo_marker_absent);
    Ok(ChildRun {
        status_code: output.status.code(),
        receipt,
        execution_status,
        cargo_marker_absent,
        blocker,
    })
}

fn child_blocker(status_code: Option<i32>, execution_status: &str, cargo_marker_absent: bool) -> Option<String> {
    if !cargo_marker_absent {
        return Some("cargo guard was invoked".to_string());
    }
    if status_code != Some(SUCCESS_EXIT_CODE) {
        return Some(format!("rust-plan exited with {}", status_text(status_code).trim_end()));
    }
    if execution_status != SUCCESS_STATUS {
        return Some(format!("topology execution status was {execution_status}"));
    }
    None
}

fn execute_fixed_point_stage(
    stage: &FixedPointStagePlan,
    mantle_bin: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    policy_digest_blake3: Option<&str>,
) -> Result<FixedPointStageRun, RunError> {
    prepare_fixed_point_stage(stage)?;
    let path_env = execution_path_env(&stage.guard_path_dir, toolchain_closure)?;
    let output = Command::new(mantle_bin)
        .args(&stage.command.args)
        .current_dir(&stage.command.current_dir)
        .env("CARGO", &stage.command.cargo_env_value)
        .env(RUSTC_BOOTSTRAP_ENV, "1")
        .env("PATH", path_env)
        .output();
    let output = match output {
        Ok(output) => output,
        Err(err) => return blocked_fixed_point_launch(stage, mantle_bin, err, policy_digest_blake3),
    };
    let mut blocker = None;
    record_file_write(&stage.receipt_path, &output.stdout, &mut blocker);
    record_file_write(&stage.stderr_path, &output.stderr, &mut blocker);
    record_file_write(&stage.status_path, status_text(output.status.code()).as_bytes(), &mut blocker);
    fixed_point_stage_from_output(stage, output.status.code(), blocker, policy_digest_blake3)
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
) -> Result<FixedPointStageRun, RunError> {
    let blocker = format!("launch {} for {}: {err}", mantle_bin.display(), stage.name);
    write_text(&stage.stderr_path, &format!("{blocker}\n"))?;
    write_text(&stage.status_path, "launch-failed\n")?;
    write_blocked_fixed_point_smoke_outputs(stage, &blocker)?;
    Ok(blocked_fixed_point_stage_from_plan(stage, "launch-failed", None, blocker, policy_digest_blake3))
}

fn fixed_point_stage_from_output(
    stage: &FixedPointStagePlan,
    status_code: Option<i32>,
    mut blocker: Option<String>,
    policy_digest_blake3: Option<&str>,
) -> Result<FixedPointStageRun, RunError> {
    let mut receipt = parse_receipt(&stage.receipt_path, status_code == Some(SUCCESS_EXIT_CODE))?;
    annotate_fixed_point_stage_receipt(&stage.receipt_path, receipt.as_mut(), policy_digest_blake3)?;
    let execution_status = receipt_execution_status(receipt.as_ref());
    let cargo_marker_absent = !stage.cargo_marker_path.exists();
    if blocker.is_none() {
        blocker = child_blocker(status_code, &execution_status, cargo_marker_absent);
    }
    let produced = if blocker.is_none() {
        materialize_fixed_point_stage_artifact(stage, receipt.as_ref())?
    } else {
        None
    };
    fixed_point_stage_with_artifact(
        stage,
        status_code,
        receipt.as_ref(),
        execution_status,
        cargo_marker_absent,
        blocker,
        produced,
        policy_digest_blake3,
    )
}

fn fixed_point_stage_with_artifact(
    stage: &FixedPointStagePlan,
    status_code: Option<i32>,
    receipt: Option<&Value>,
    execution_status: String,
    cargo_marker_absent: bool,
    mut blocker: Option<String>,
    produced: Option<FixedPointStageArtifact>,
    policy_digest_blake3: Option<&str>,
) -> Result<FixedPointStageRun, RunError> {
    if let Some(produced) = produced.as_ref() {
        if produced.smoke_status_code != SUCCESS_EXIT_CODE {
            blocker = Some(format!("smoke check exited with {}", produced.smoke_status_code));
        }
    }
    if blocker.is_some() && produced.is_none() {
        write_blocked_fixed_point_smoke_outputs(stage, blocker.as_deref().unwrap_or("blocked before binary"))?;
    }
    Ok(FixedPointStageRun {
        name: stage.name,
        dir: stage.stage_dir.clone(),
        execution_dir: stage.execution_dir.clone(),
        receipt_path: stage.receipt_path.clone(),
        stderr_path: stage.stderr_path.clone(),
        status_path: stage.status_path.clone(),
        status_code,
        execution_status,
        cargo_marker_absent,
        success: blocker.is_none(),
        unit_count: receipt.map(unit_count).unwrap_or_default(),
        failed_unit_count: receipt.map(failed_unit_count).unwrap_or_default(),
        binary: produced.as_ref().map(|value| value.binary.clone()),
        binary_blake3: produced.as_ref().map(|value| value.digest.clone()),
        smoke_status_code: produced.as_ref().map(|value| value.smoke_status_code),
        source_built_toolchain_closure_policy_digest_blake3: policy_digest_blake3.map(ToOwned::to_owned),
        blocker,
    })
}

fn annotate_fixed_point_stage_receipt(
    receipt_path: &Path,
    receipt: Option<&mut Value>,
    policy_digest_blake3: Option<&str>,
) -> Result<(), RunError> {
    let Some(policy_digest_blake3) = policy_digest_blake3 else {
        return Ok(());
    };
    let Some(receipt) = receipt else {
        return Ok(());
    };
    let Some(object) = receipt.as_object_mut() else {
        return Err(internal("fixed-point stage receipt is not a JSON object".to_string()));
    };
    object.insert(TOOLCHAIN_CLOSURE_POLICY_DIGEST_FIELD.to_string(), Value::String(policy_digest_blake3.to_string()));
    let bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|err| internal(format!("serialize annotated fixed-point stage receipt: {err}")))?;
    write_bytes(receipt_path, &bytes)
}

fn materialize_fixed_point_stage_artifact(
    stage: &FixedPointStagePlan,
    receipt: Option<&Value>,
) -> Result<Option<FixedPointStageArtifact>, RunError> {
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
    stage
}

fn blocked_fixed_point_stage_from_plan(
    stage: &FixedPointStagePlan,
    execution_status: &str,
    status_code: Option<i32>,
    blocker: String,
    policy_digest_blake3: Option<&str>,
) -> FixedPointStageRun {
    FixedPointStageRun {
        name: stage.name,
        dir: stage.stage_dir.clone(),
        execution_dir: stage.execution_dir.clone(),
        receipt_path: stage.receipt_path.clone(),
        stderr_path: stage.stderr_path.clone(),
        status_path: stage.status_path.clone(),
        status_code,
        execution_status: execution_status.to_string(),
        cargo_marker_absent: true,
        success: false,
        unit_count: 0,
        failed_unit_count: 0,
        binary: None,
        binary_blake3: None,
        smoke_status_code: None,
        source_built_toolchain_closure_policy_digest_blake3: policy_digest_blake3.map(ToOwned::to_owned),
        blocker: Some(blocker),
    }
}

fn fixed_point_status(stage1: &FixedPointStageRun, stage2: &FixedPointStageRun) -> Result<&'static str, RunError> {
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
    json_mode: bool,
    plan: &FixedPointPlan,
    compatibility: &RustcCompatibilitySummary,
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: &RustSourceProviderBindingStatus,
    stage1: FixedPointStageRun,
    stage2: Option<FixedPointStageRun>,
    status: &str,
) -> Result<(), RunError> {
    let summary = fixed_point_summary(
        plan,
        compatibility,
        toolchain_closure,
        rust_source_provider,
        &stage1,
        stage2.as_ref(),
        status,
    );
    write_summary(&plan.meta_path, &summary)?;
    print_fixed_point_summary(&summary, json_mode)?;
    if status == SUCCESS_STATUS {
        return Ok(());
    }
    Err(RunError::Build(fixed_point_error_message(&summary)))
}

fn fixed_point_summary(
    plan: &FixedPointPlan,
    compatibility: &RustcCompatibilitySummary,
    toolchain_closure: &crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: &RustSourceProviderBindingStatus,
    stage1: &FixedPointStageRun,
    stage2: Option<&FixedPointStageRun>,
    status: &str,
) -> FixedPointSummary {
    let non_claims = fixed_point_non_claims(toolchain_closure);
    FixedPointSummary {
        schema: FIXED_POINT_SCHEMA,
        status: status.to_string(),
        root: plan.root.clone(),
        bundle_dir: plan.bundle_dir.clone(),
        fixed_point: status == SUCCESS_STATUS,
        stage1: stage_summary(stage1),
        stage2: stage2.map(stage_summary),
        rustc_compatibility: RustcCompatibilitySummary {
            requested_rustc: compatibility.requested_rustc.clone(),
            stage_rustc: compatibility.stage_rustc.clone(),
            normalization: compatibility.normalization,
            wrapper: compatibility.wrapper.clone(),
            wrapper_blake3: compatibility.wrapper_blake3.clone(),
        },
        source_built_toolchain_closure: toolchain_closure.clone(),
        rust_source_provider: rust_source_provider.clone(),
        blocker: fixed_point_blocker(stage1, stage2, status),
        non_claims,
    }
}

fn stage_summary(stage: &FixedPointStageRun) -> FixedPointStageSummary {
    FixedPointStageSummary {
        name: stage.name,
        dir: stage.dir.clone(),
        execution_dir: stage.execution_dir.clone(),
        receipt: stage.receipt_path.clone(),
        stderr: stage.stderr_path.clone(),
        status: stage.status_path.clone(),
        status_code: stage.status_code,
        execution_status: stage.execution_status.clone(),
        cargo_marker_absent: stage.cargo_marker_absent,
        success: stage.success,
        unit_count: stage.unit_count,
        failed_unit_count: stage.failed_unit_count,
        binary: stage.binary.clone(),
        binary_blake3: stage.binary_blake3.clone(),
        smoke_status_code: stage.smoke_status_code,
        source_built_toolchain_closure_policy_digest_blake3: stage
            .source_built_toolchain_closure_policy_digest_blake3
            .clone(),
        blocker: stage.blocker.clone(),
    }
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

fn print_fixed_point_summary(summary: &FixedPointSummary, json_mode: bool) -> Result<(), RunError> {
    if json_mode {
        let rendered = serde_json::to_string(summary).map_err(|err| internal(format!("render summary: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("Cargo-free fixed-point: {}", summary.status);
    println!("bundle: {}", summary.bundle_dir.display());
    if let Some(digest) = &summary.stage1.binary_blake3 {
        println!("stage1_binary_blake3: {digest}");
    }
    if let Some(stage2) = &summary.stage2 {
        if let Some(digest) = &stage2.binary_blake3 {
            println!("stage2_binary_blake3: {digest}");
        }
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

fn summarize(
    paths: &BuildPaths,
    child: &ChildRun,
    produced: Option<&ProducedBinary>,
    toolchain_closure: crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus,
    rust_source_provider: RustSourceProviderBindingStatus,
) -> SelfBuildSummary {
    let receipt = child.receipt.as_ref();
    let non_claims = self_build_non_claims(&toolchain_closure);
    SelfBuildSummary {
        schema: SCHEMA,
        status: if child.blocker.is_none() {
            SUCCESS_STATUS
        } else {
            BLOCKED_STATUS
        }
        .to_string(),
        root: paths.root.clone(),
        out_dir: paths.out_dir.clone(),
        binary: produced.map(|value| value.path.clone()),
        binary_blake3: produced.map(|value| value.blake3.clone()),
        source_digest: produced.map(|value| value.source_digest.clone()),
        source_closure_digest_blake3: produced.and_then(|value| value.source_closure_digest_blake3.clone()),
        receipt: paths.receipt_path.clone(),
        stderr: paths.stderr_path.clone(),
        status_code: child.status_code,
        execution_status: child.execution_status.clone(),
        cargo_marker_absent: child.cargo_marker_absent,
        unit_count: receipt.map(unit_count).unwrap_or_default(),
        failed_unit_count: receipt.map(failed_unit_count).unwrap_or_default(),
        smoke_status_code: produced.and_then(|value| value.smoke_status_code),
        blocker: child.blocker.clone(),
        source_built_toolchain_closure: toolchain_closure,
        rust_source_provider,
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
    if blocker.is_some() {
        let _ = fs::write(path, bytes);
        return;
    }
    if let Err(err) = fs::write(path, bytes) {
        *blocker = Some(format!("write {}: {err}", path.display()));
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
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(ExecutionToolchain {
            rustc: requested_rustc.to_path_buf(),
            path_env: guarded_path(guard_path_dir)?,
            status: effective_source_built_toolchain_closure(toolchain_closure, rust_source_provider),
        });
    };
    let rustc = resolve_executable(requested_rustc, "rustc")?;
    let status = enforce_receipt_bound_toolchain(&rustc, toolchain_closure, manifest)?;
    Ok(ExecutionToolchain {
        rustc,
        path_env: execution_path_env(guard_path_dir, toolchain_closure)?,
        status,
    })
}

fn prepare_rustc_for_compatibility(
    requested_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
) -> Result<PathBuf, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(requested_rustc.to_path_buf());
    };
    let rustc = resolve_executable(requested_rustc, "rustc")?;
    enforce_observed_toolchain_subset(manifest, &[observed_file_tool(
        crate::source_toolchain_closure::ToolchainRole::Rustc,
        &rustc,
    )?])?;
    Ok(rustc)
}

fn enforce_fixed_point_toolchain(
    stage_rustc: &Path,
    toolchain_closure: &LoadedToolchainClosure,
    rust_source_provider: &LoadedRustSourceProvider,
) -> Result<crate::source_toolchain_closure::SourceBuiltToolchainClosureStatus, RunError> {
    let Some(manifest) = &toolchain_closure.manifest else {
        return Ok(effective_source_built_toolchain_closure(toolchain_closure, rust_source_provider));
    };
    enforce_receipt_bound_toolchain(stage_rustc, toolchain_closure, manifest)
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
    let manifest_path = toolchain_closure
        .manifest_path
        .clone()
        .ok_or_else(|| internal("toolchain closure manifest path missing during enforcement".to_string()))?;
    let observed = observed_toolchain_inputs(rustc, manifest)?;
    let validation = enforce_observed_toolchain_subset(manifest, &observed)?;
    Ok(crate::source_toolchain_closure::enforced_source_built_toolchain_closure(manifest_path, &validation))
}

fn enforce_observed_toolchain_subset(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
    observed: &[crate::source_toolchain_closure::ToolchainObservedInput],
) -> Result<crate::source_toolchain_closure::ToolchainClosureValidation, RunError> {
    crate::source_toolchain_closure::enforce_observed_toolchain_inputs(manifest, observed)
        .map_err(|err| RunError::Build(format!("source-built toolchain closure blocked: {}", err.message())))
}

fn observed_toolchain_inputs(
    rustc: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<Vec<crate::source_toolchain_closure::ToolchainObservedInput>, RunError> {
    use crate::source_toolchain_closure::ToolchainRole;
    let mut observed = Vec::new();
    observed.push(observed_file_tool(ToolchainRole::Rustc, rustc)?);
    observed.push(observed_sysroot_tool(rustc)?);
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
    let Some(manifest) = &toolchain_closure.manifest else {
        return guarded_path(cargo_path_dir);
    };
    write_toolchain_path_aliases(cargo_path_dir, manifest)?;
    env::join_paths([cargo_path_dir]).map_err(|err| internal(format!("construct receipt-bound PATH: {err}")))
}

fn write_toolchain_path_aliases(
    guard_path_dir: &Path,
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<(), RunError> {
    let aliases = toolchain_path_aliases(manifest)?;
    let c_compiler = c_compiler_alias_target(manifest)?;
    let unwind_archive = declared_unwind_archive(manifest)?;
    for (alias, target) in aliases {
        let link = guard_path_dir.join(alias);
        if link.file_name() == Some(OsStr::new(CARGO_SHIM_NAME)) {
            return Err(RunError::Build("source-built toolchain closure blocked: Cargo must stay guarded".to_string()));
        }
        remove_owned_path(&link)?;
        if c_compiler.as_ref().is_some_and(|compiler| compiler == &target) {
            if let Some(unwind_archive) = &unwind_archive {
                write_c_compiler_toolchain_alias(&target, unwind_archive, &link)?;
                continue;
            }
        }
        write_toolchain_alias(&target, &link)?;
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
    Ok(aliases)
}

fn c_compiler_alias_target(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<Option<PathBuf>, RunError> {
    use crate::source_toolchain_closure::ToolchainRole;
    let members = manifest.members.iter().filter(|member| member.role == ToolchainRole::CCompiler).collect::<Vec<_>>();
    match members.as_slice() {
        [member] => Ok(Some(canonicalize_toolchain_path(Path::new(&member.execution_path), member.role)?)),
        [] => Ok(None),
        _many => Err(RunError::Build(
            "source-built toolchain closure blocked: multiple CCompiler members for PATH alias generation".to_string(),
        )),
    }
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
        ToolchainRole::CCompiler => add_toolchain_alias(aliases, C_COMPILER_ALIAS.to_string(), target),
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
fn write_toolchain_alias(target: &Path, link: &Path) -> Result<(), RunError> {
    write_text(link, &format!("#!/bin/sh\nexec {} \"$@\"\n", shell_quote(target)))?;
    set_executable(link)
}

#[cfg(unix)]
fn write_c_compiler_toolchain_alias(target: &Path, unwind_archive: &Path, link: &Path) -> Result<(), RunError> {
    let runtime_dir = link
        .parent()
        .ok_or_else(|| internal(format!("{} has no parent", link.display())))?
        .join(TOOLCHAIN_ALIAS_RUNTIME_DIR);
    fs::create_dir_all(&runtime_dir)
        .map_err(|err| internal(format!("create toolchain alias runtime dir {}: {err}", runtime_dir.display())))?;
    let runtime_unwind = runtime_dir.join(TOOLCHAIN_ALIAS_UNWIND_ARCHIVE);
    fs::copy(unwind_archive, &runtime_unwind).map_err(|err| {
        internal(format!(
            "copy declared unwind archive {} -> {}: {err}",
            unwind_archive.display(),
            runtime_unwind.display()
        ))
    })?;
    let script = format!(
        "#!/bin/sh\nremaining=$#\nwhile [ \"$remaining\" -gt 0 ]; do\n  arg=$1\n  shift\n  case \"$arg\" in\n    -static-pie) set -- \"$@\" -static ;;\n    *) set -- \"$@\" \"$arg\" ;;\n  esac\n  remaining=$((remaining - 1))\ndone\nexec {} -L{} \"$@\"\n",
        shell_quote(target),
        shell_quote(&runtime_dir)
    );
    write_text(link, &script)?;
    set_executable(link)
}

#[cfg(not(unix))]
fn write_toolchain_alias(target: &Path, link: &Path) -> Result<(), RunError> {
    fs::copy(target, link)
        .map_err(|err| internal(format!("copy {} -> {}: {err}", target.display(), link.display())))?;
    set_executable(link)
}

#[cfg(not(unix))]
fn write_c_compiler_toolchain_alias(target: &Path, _unwind_archive: &Path, link: &Path) -> Result<(), RunError> {
    write_toolchain_alias(target, link)
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
        "This build does not claim Crunch bootstrap or release reproducibility.",
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
        "This proof does not claim Crunch bootstrap or release reproducibility.",
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

fn load_rust_source_provider(provider_dir: Option<&Path>) -> Result<LoadedRustSourceProvider, RunError> {
    let Some(provider_dir) = provider_dir else {
        return Ok(LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: None,
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
    Ok(LoadedRustSourceProvider {
        status: validated_rust_source_provider_binding(&provider_dir, &validation, &rustc_path),
        rustc: Some(rustc_path),
        toolchain_closure_status: Some(toolchain_closure_status),
    })
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
        "bundle_dir": plan.bundle_dir,
        "shared_execution_dir": plan.shared_execution_dir,
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
    use super::*;

    const FIXED_POINT_TEST_DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const FIXED_POINT_TEST_DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
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

        assert!(script.contains(LINK_SELF_CONTAINED_PROBE_ARG));
        assert!(script.contains(LINK_SELF_CONTAINED_JOINED_ARG));
        assert!(script.contains(RUSTC_BOOTSTRAP_ENV));
        assert!(script.contains(REAL_RUSTC_ENV));
        assert!(script.contains("exec"));
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

        let loaded = load_rust_source_provider(Some(&provider_dir)).unwrap();
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

    #[test]
    fn rust_source_provider_selection_prefers_validated_provider_and_keeps_fallback() {
        let fallback = PathBuf::from("/fallback/rustc");
        let provider_rustc = PathBuf::from("/provider/bin/rustc");
        let absent = LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: None,
            toolchain_closure_status: None,
        };
        let validated = LoadedRustSourceProvider {
            status: absent_rust_source_provider_binding(),
            rustc: Some(provider_rustc.clone()),
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

        let err = load_rust_source_provider(Some(&provider_dir)).unwrap_err();
        let message = err.message();

        assert!(message.contains("source-built Rust provider blocked"));
        assert!(message.contains("failed validation"));
        assert!(message.contains("not source-built"));
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
            blocker: None,
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
                "This proof does not claim Crunch bootstrap or release reproducibility.\nThis proof does not claim full Cargo compatibility.\n"
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
    fn receipt_bound_c_compiler_alias_materializes_declared_unwind_archive() {
        let dir = tempfile::TempDir::new().unwrap();
        let tools = fake_toolchain(&dir, false);
        let manifest = fake_toolchain_manifest(&tools, None);
        let guard_dir = dir.path().join("guard-bin");
        fs::create_dir_all(&guard_dir).unwrap();

        write_toolchain_path_aliases(&guard_dir, &manifest).unwrap();

        let cc_alias = fs::read_to_string(guard_dir.join(C_COMPILER_ALIAS)).unwrap();
        let runtime_unwind = guard_dir.join(TOOLCHAIN_ALIAS_RUNTIME_DIR).join(TOOLCHAIN_ALIAS_UNWIND_ARCHIVE);
        assert!(cc_alias.contains("-L"));
        assert!(cc_alias.contains(TOOLCHAIN_ALIAS_RUNTIME_DIR));
        assert!(cc_alias.contains("remaining=$#"));
        assert!(cc_alias.contains("-static-pie) set -- \"$@\" -static ;;"));
        assert_eq!(fs::read(&runtime_unwind).unwrap(), fs::read(&tools.unwind_archive).unwrap());
        assert!(!guard_dir.join(CARGO_SHIM_NAME).exists());
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

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("Sysroot"));
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
        assert!(err.message().contains("CCompiler"));
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

        assert!(err.message().contains("host-tool-leakage"));
        assert!(err.message().contains("PkgConfig"));
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
        write_fake_executable(&rustc, &rustc_sysroot_script(&sysroot));
        write_fake_executable(&linker, "#!/bin/sh\nexit 0\n");
        write_fake_executable(&c_compiler, "#!/bin/sh\nexit 0\n");
        write_fake_executable(&archiver, "#!/bin/sh\nexit 0\n");
        write_fake_executable(&ranlib, "#!/bin/sh\nexit 0\n");
        write_text(&unwind_archive, "fake unwind archive\n").unwrap();
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
