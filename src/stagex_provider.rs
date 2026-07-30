use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::path::PathBuf;

use crunch_bootstrap_core::STAGEX_RECEIPT_SCHEMA_V1;
use crunch_bootstrap_core::STAGEX_RECEIPT_STATUS_COMPLETE;
use crunch_bootstrap_core::STAGEX_STAGE_STATUS_COMPLETE;
use crunch_bootstrap_core::StagexLineageReceipt;
use crunch_bootstrap_core::StagexMaterializationPlan;
use crunch_bootstrap_core::StagexOutputObservation;
use crunch_bootstrap_core::StagexPredecessorReport;
use crunch_bootstrap_core::StagexStageReport;
use crunch_bootstrap_core::stagex_plan_digest_blake3;
use crunch_bootstrap_core::stagex_stage_plan_digest_blake3;
use crunch_bootstrap_core::stagex_stage_report_digest_blake3;
use crunch_bootstrap_core::validate_stagex_plan;
use crunch_bootstrap_core::validate_stagex_receipt;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::protected_exec::PlannedExecutable;
use crate::protected_exec::PromotedExecutable;
use crate::protected_exec::ProtectedExecPolicy;
use crate::protected_exec::ProtectedSeccompAuditEvent;
use crate::protected_exec::blake3_file_hex;
use crate::protected_exec_seccomp::install_current_thread_exec_supervisor;
use crate::protected_exec_seccomp::reap_adopted_exec_descendants;

pub(crate) const PROVIDER_METADATA_RELATIVE_PATH: &str = "share/crunch-bootstrap/provider.json";
pub(crate) const PROVIDER_RECEIPT_RELATIVE_PATH: &str = "share/crunch-bootstrap/stagex-lineage-receipt.json";
pub(crate) const PROVIDER_VALIDATION_RELATIVE_PATH: &str = "share/crunch-bootstrap/stagex-provider-validation.json";

const PROVIDER_SCHEMA: &str = "mantle-stagex-intermediate-provider-v1";
const PROVIDER_VALIDATION_SCHEMA: &str = "mantle-stagex-provider-validation-v1";
const PROVIDER_KIND: &str = "stagex-lineage";
const PROVIDER_NAME: &str = "mantle-stagex-intermediate-toolchain";
const PROVIDER_TARGET: &str = "x86_64-linux-musl";
const PROVIDER_COMPILER_KIND: &str = "tinycc-0.9.27-selfhost";
const PROVIDER_BOUNDED_CLAIM: &str =
    "Intermediate StageX provider publication from the protected TinyCC/native-musl/binutils closure.";
const STAGE_REPORT_OUTPUT_BINDING: &str = "Live artifacts use observed BLAKE3 identities. Exact allowlisted report-only observations use a domain-separated BLAKE3 projection over the artifact ID, plan digest, and complete transition-report digest.";
const EXECUTABLE_AUTHORIZATION_BINDING: &str = "Stage reports bind the complete declared executable-authorization set only after every exact path-plus-digest identity appears in a kernel-intercepted allowed protected execve or execveat audit decision. Identity-equivalent IDs may share one event.";
const PROVIDER_NON_CLAIMS: &[&str] = &[
    "This receipt does not claim final native GCC provider admission.",
    "This receipt does not prove compiler correctness or complete musl/binutils behavior.",
    "This receipt does not prove Mantle self-build completion, release reproducibility, or release eligibility.",
    "A protected exec decision proves a kernel-intercepted executable identity, not successful process completion or behavior.",
    "The original v81 pueue command log was unavailable; retained plan, report, audit, inventories, and artifacts are the publication evidence.",
];
const TRANSITION_REPORT_SCHEMA: &str = "mantle-stagex-protected-transition-report-v1";
const TRANSITION_PLAN_FILE_NAME: &str = "transition-plan.json";
const TRANSITION_REPORT_FILE_NAME: &str = "transition-report.json";
const TRANSITION_AUDIT_FILE_NAME: &str = "protected-exec-audit.json";
const TRANSITION_STATUS_COMPLETE: &str = "complete";
const PROVIDER_STAGE_SUFFIX: &str = ".stagex-provider-staging";
const PROVIDER_TCC_RELATIVE_PATH: &str = "bin/x86_64-linux-musl-tcc";
const PROVIDER_TCC_RUNTIME_RELATIVE_PATH: &str = "lib/tcc";
const PROVIDER_TCC_SOURCE_RELATIVE_PATH: &str = "share/mantle-stagex/tcc-source-patched";
const PROVIDER_MUSL_INCLUDE_RELATIVE_PATH: &str = "x86_64-linux-musl/include";
const PROVIDER_MUSL_LIB_RELATIVE_PATH: &str = "x86_64-linux-musl/lib";
const PROVIDER_BINUTILS_INCLUDE_RELATIVE_PATH: &str = "x86_64-linux-musl/include-binutils";
const PROVIDER_BINUTILS_LD_RELATIVE_PATH: &str = "x86_64-linux-musl/lib/ldscripts";
const TRANSITION_TCC_RELATIVE_PATH: &str = "tcc-musl-selfhost-stage/runtime/output/bin/tcc-0.9.27-musl-selfhost";
const TRANSITION_TCC_RUNTIME_RELATIVE_PATH: &str = "tcc-musl-selfhost-stage/runtime/output/lib/tcc";
const TRANSITION_TCC_SOURCE_RELATIVE_PATH: &str = "tcc-musl-selfhost-stage/runtime/output/share/tcc-source-patched";
const TRANSITION_MUSL_INCLUDE_RELATIVE_PATH: &str = "musl-native-stage/runtime/output/include";
const TRANSITION_MUSL_LIB_RELATIVE_PATH: &str = "musl-native-stage/runtime/output/lib";
const TRANSITION_BINUTILS_ROOT_RELATIVE_PATH: &str =
    "binutils-stage/runtime/install-destdir/mantle/stagex/binutils-probe-output";
const TRANSITION_BINUTILS_RUNTIME_RELATIVE_PATH: &str = "binutils-stage/runtime";
const TRANSITION_SEED_RELATIVE_PATH: &str = "hex0-seed";
const STAGE_REPORT_OUTPUT_PROOF_DOMAIN: &[u8] = b"mantle-stagex-output-proof-v1\0";
const STAGE_GRAPH_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-stage-graph-v1\0";
const NORMALIZED_PROVIDER_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-normalized-provider-v1\0";
const PROVIDER_OUTPUT_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-provider-outputs-v1\0";
const FINAL_BUNDLE_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-provider-bundle-v1\0";
const BINUTILS_GENERATED_EXECUTABLE_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-binutils-generated-executables-v1\0";
const BINUTILS_GENERATED_SOURCE_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-binutils-generated-sources-v1\0";
const PROVIDER_VALIDATION_AUDIT_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-provider-validation-audit-v1\0";
const PROVIDER_VALIDATION_REPORT_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-provider-validation-report-v1\0";
const PROVIDER_RECEIPT_PAYLOAD_DIGEST_DOMAIN: &[u8] = b"mantle-stagex-provider-receipt-payload-v1\0";
const JSON_PLAN_BYTES_MAX: u64 = 16 * 1_024 * 1_024;
const JSON_REPORT_BYTES_MAX: u64 = 256 * 1_024 * 1_024;
const JSON_AUDIT_BYTES_MAX: u64 = 256 * 1_024 * 1_024;
const JSON_MANIFEST_BYTES_MAX: u64 = 16 * 1_024 * 1_024;
const PROVIDER_FILE_BYTES_MAX: u64 = 64 * 1_024 * 1_024;
const PROVIDER_OUTPUT_COUNT_MAX: usize = 512;
const TRANSITION_AUDIT_EVENT_COUNT_MAX: usize = 131_072;
const PROVIDER_VALIDATION_EVENT_COUNT_MAX: usize = 64;
const PROVIDER_VALIDATION_EVENT_COUNT_EXPECTED: usize = 7;
const PROTECTED_EXEC_SYSCALL_COUNT: usize = 2;
const PROTECTED_EXEC_SYSCALLS: [&str; PROTECTED_EXEC_SYSCALL_COUNT] = ["execve", "execveat"];
const PROVIDER_VALIDATION_PLANNED_EXECUTABLE_COUNT: usize = 3;
const PROVIDER_VALIDATION_EXECUTABLE_SEQUENCE: [&str; PROVIDER_VALIDATION_EVENT_COUNT_EXPECTED] = [
    "bin/x86_64-linux-musl-tcc",
    ".validation/tcc-positive",
    "bin/x86_64-linux-musl-tcc",
    "bin/x86_64-linux-musl-as",
    "bin/x86_64-linux-musl-ld",
    "bin/x86_64-linux-musl-as",
    ".validation/binutils-positive",
];
const PROVIDER_VALIDATION_AUTHORIZATION_SEQUENCE: [&str; PROVIDER_VALIDATION_EVENT_COUNT_EXPECTED] = [
    "planned:stagex-provider-normalization:exec:provider-validation:tcc",
    "promoted:provider-relocation-smoke:.validation/tcc-positive",
    "planned:stagex-provider-normalization:exec:provider-validation:tcc",
    "planned:stagex-provider-normalization:exec:provider-validation:as",
    "planned:stagex-provider-normalization:exec:provider-validation:ld",
    "planned:stagex-provider-normalization:exec:provider-validation:as",
    "promoted:provider-relocation-smoke:.validation/binutils-positive",
];
const HEX_CHARS_PER_BYTE: usize = 2;
const ADJACENT_PAIR_WINDOW_SIZE: usize = 2;
const ELF_MAGIC: &[u8] = b"\x7fELF";
const ELF_MAGIC_BYTES: usize = 4;
const EXECUTABLE_MODE_BITS: u32 = 0o111;
const PROVIDER_EXECUTABLE_MODE: u32 = 0o755;
const PROVIDER_DATA_MODE: u32 = 0o644;
const PROVIDER_DIRECTORY_MODE: u32 = 0o755;
const PROVIDER_PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const FILE_MODE_MASK: u32 = 0o777;
const RUNTIME_SMOKE_EXPECTED_EXIT: i32 = 42;
const PROVIDER_ROLE_COUNT: usize = 4;
const PROJECTED_BINUTILS_OUTPUT_COUNT: usize = 3;
const BINUTILS_GENERATED_SOURCE_OUTPUT_COUNT: usize = 22;
const PROVIDER_PAYLOAD_BASE_COUNT: usize = 7;
const TCC_SMOKE_SOURCE: &[u8] = b"int main(void) { return 42; }\n";
const BINUTILS_SMOKE_SOURCE: &[u8] = b".global _start\n.text\n_start:\n  mov $60, %rax\n  mov $42, %rdi\n  syscall\n";
const BINUTILS_NEGATIVE_SOURCE: &[u8] = b".mantle-invalid-directive\n";
const EMPTY_INPUT: &[u8] = b"";
const NATIVE_LIBRARY_NAMES: &[&str] = &[
    "crt1.o",
    "crti.o",
    "crtn.o",
    "libc.a",
    "libcrypt.a",
    "libdl.a",
    "libm.a",
    "libpthread.a",
    "libresolv.a",
    "librt.a",
    "libutil.a",
    "libxnet.a",
];
const BINUTILS_TOOL_NAMES: &[&str] = &[
    "as", "ld", "ar", "ranlib", "nm", "objcopy", "objdump", "readelf", "size", "strings", "strip",
];
const PROJECTED_BINUTILS_OUTPUT_IDS: &[&str] = &[
    "binutils-configure-probe-executables",
    "binutils-generated-sources",
    "binutils-runtime-smoke",
];
const REPORT_BOUND_OUTPUT_IDS: &[&str] = &[
    "stage0-full-sha256-verification",
    "stage0-full-after-observation",
    "mes-m2-smoke-observation",
    "nyacc-generated-tables",
    "tcc-boot0-smoke-observation",
    "tinycc-final-smoke-observation",
    "tinycc27-version-observation",
    "tinycc27-negative-observation",
    "make-version-observation",
    "make-malformed-observation",
    "gnu-patch-version-observation",
    "gnu-patch-negative-observation",
    "gzip-help-observation",
    "gzip-negative-observation",
    "tar-version-observation",
    "tar-negative-observation",
    "sed-version-observation",
    "sed-negative-observation",
    "bzip2-help-observation",
    "bzip2-negative-observation",
    "coreutils-negative-observation",
    "oyacc-negative-observation",
    "bash-negative-observation",
    "tcc-musl-prep-negative-observation",
    "musl-negative-observation",
    "tcc-musl-negative-observation",
    "musl-pass2-negative-observation",
    "tcc-musl-v2-negative-observation",
];

#[derive(Debug, Clone)]
pub(crate) struct StagexProviderRequest<'a> {
    pub lineage_manifest_path: &'a Path,
    pub transition_root: &'a Path,
    pub output_path: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StagexProviderPublicationReport {
    pub schema: &'static str,
    pub provider_kind: &'static str,
    pub output_path: PathBuf,
    pub receipt_path: PathBuf,
    pub normalized_provider_digest_blake3: String,
    pub output_digest_blake3: String,
    pub final_bundle_digest_blake3: String,
    pub provider_validation_audit_digest_blake3: String,
    pub provider_validation_report_digest_blake3: String,
}

#[derive(Debug)]
pub(crate) enum StagexProviderError {
    InvalidInput(String),
    Io(String),
    Transition(String),
    Provider(String),
    Receipt(String),
    Runtime(String),
    Publication(String),
}

impl std::fmt::Display for StagexProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(formatter, "invalid StageX provider input: {message}"),
            Self::Io(message) => write!(formatter, "StageX provider I/O failed: {message}"),
            Self::Transition(message) => write!(formatter, "StageX transition evidence failed validation: {message}"),
            Self::Provider(message) => write!(formatter, "StageX provider contract failed validation: {message}"),
            Self::Receipt(message) => write!(formatter, "StageX provider receipt failed validation: {message}"),
            Self::Runtime(message) => write!(formatter, "StageX provider runtime validation failed: {message}"),
            Self::Publication(message) => write!(formatter, "StageX provider publication failed: {message}"),
        }
    }
}

impl std::error::Error for StagexProviderError {}

#[derive(Debug)]
struct TransitionEvidence {
    manifest_digest_blake3: String,
    plan: StagexMaterializationPlan,
    plan_digest_blake3: String,
    report_digest_blake3: String,
    audit_digest_blake3: String,
    report: Value,
    audit_events: Vec<ProtectedSeccompAuditEvent>,
    observed_outputs: BTreeMap<String, ObservedTransitionOutput>,
    manifest_outputs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ObservedTransitionOutput {
    artifact_id: String,
    path: PathBuf,
    bytes_len: u64,
    digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProviderPayloadObservation {
    artifact_id: String,
    relative_path: String,
    kind: String,
    bytes_len: u64,
    digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProviderRoleObservation {
    role: String,
    artifact_ids: Vec<String>,
    digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProviderValidationEvent {
    executable_relative_path: String,
    digest_blake3: String,
    authorization_id: String,
    policy_decision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProviderValidationReport {
    schema: String,
    status: String,
    audit_digest_blake3: String,
    events: Vec<ProviderValidationEvent>,
    positive_tcc_compile: bool,
    negative_tcc_rejection: bool,
    positive_binutils_execution_status: i32,
    negative_assembler_rejection: bool,
    fallback_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ProviderMetadata {
    schema: String,
    provider_id: String,
    provider_kind: String,
    name: String,
    target: String,
    compiler_kind: String,
    lineage_manifest_digest_blake3: String,
    transition_plan_digest_blake3: String,
    transition_report_digest_blake3: String,
    protected_exec_audit_digest_blake3: String,
    provider_validation_audit_digest_blake3: String,
    provider_validation_report_digest_blake3: String,
    normalized_provider_digest_blake3: String,
    payload: Vec<ProviderPayloadObservation>,
    retained_tools: Vec<String>,
    provider_roles: Vec<String>,
    notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct StagexProviderReceipt {
    schema: String,
    provider_kind: String,
    audited_seed_digest: String,
    lineage_manifest_digest: String,
    stage_graph_digest: String,
    normalized_provider_digest: String,
    transition_report_digest_blake3: String,
    provider_validation_audit_digest_blake3: String,
    provider_validation_report_digest_blake3: String,
    receipt_payload_digest_blake3: String,
    provider_outputs: Vec<ProviderRoleObservation>,
    bounded_claim: String,
    stage_report_output_binding: String,
    executable_authorization_binding: String,
    non_claims: Vec<String>,
    #[serde(flatten)]
    lineage: StagexLineageReceipt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FinalBundleIdentity<'a> {
    schema: &'static str,
    provider_kind: &'static str,
    plan_digest_blake3: &'a str,
    lineage_manifest_digest_blake3: &'a str,
    stage_graph_digest_blake3: &'a str,
    source_state_digest_blake3: &'a str,
    normalized_provider_digest_blake3: &'a str,
    output_digest_blake3: &'a str,
    protected_exec_audit_digest_blake3: &'a str,
    provider_validation_audit_digest_blake3: &'a str,
    provider_validation_report_digest_blake3: &'a str,
    transition_report_digest_blake3: &'a str,
    stage_report_digests: Vec<String>,
    provider_outputs: &'a [ProviderRoleObservation],
    bounded_claim: &'static str,
    stage_report_output_binding: &'static str,
    executable_authorization_binding: &'static str,
    non_claims: &'static [&'static str],
}

#[derive(Debug)]
struct ProviderDigests {
    normalized_provider_digest_blake3: String,
    output_digest_blake3: String,
    stage_graph_digest_blake3: String,
    provider_validation_report_digest_blake3: String,
    final_bundle_digest_blake3: String,
}

pub(crate) fn materialize_stagex_provider(
    request: StagexProviderRequest<'_>,
) -> Result<StagexProviderPublicationReport, StagexProviderError> {
    validate_request_paths(&request)?;
    let evidence = load_transition_evidence(&request)?;
    let staging_path = provider_staging_path(request.output_path)?;
    create_private_staging(&staging_path)?;
    let result = materialize_and_validate_staging(&request, &evidence, &staging_path);
    let report = match result {
        Ok(report) => report,
        Err(error) => {
            remove_failed_staging(&staging_path);
            return Err(error);
        }
    };
    publish_no_replace(&staging_path, request.output_path)?;
    independently_validate_published_provider(request.output_path, &evidence)?;
    assert!(request.output_path.is_dir());
    assert!(!staging_path.exists());
    Ok(StagexProviderPublicationReport {
        output_path: request.output_path.to_path_buf(),
        receipt_path: request.output_path.join(PROVIDER_RECEIPT_RELATIVE_PATH),
        ..report
    })
}

fn materialize_and_validate_staging(
    request: &StagexProviderRequest<'_>,
    evidence: &TransitionEvidence,
    staging_path: &Path,
) -> Result<StagexProviderPublicationReport, StagexProviderError> {
    copy_provider_payload(request.transition_root, staging_path, evidence)?;
    let validation = validate_relocated_provider_runtime(staging_path, request.transition_root)?;
    let payload = observe_provider_payload(staging_path)?;
    let normalized_provider_digest_blake3 = digest_serialized(NORMALIZED_PROVIDER_DIGEST_DOMAIN, &payload)?;
    let provider_validation_report_digest_blake3 =
        digest_serialized(PROVIDER_VALIDATION_REPORT_DIGEST_DOMAIN, &validation)?;
    let metadata = provider_metadata(
        evidence,
        &payload,
        &validation,
        &provider_validation_report_digest_blake3,
        &normalized_provider_digest_blake3,
    );
    write_json_create_new(&staging_path.join(PROVIDER_METADATA_RELATIVE_PATH), &metadata)?;
    let roles = observe_provider_roles(staging_path, &payload)?;
    let output_digest_blake3 = digest_serialized(PROVIDER_OUTPUT_DIGEST_DOMAIN, &roles)?;
    let stage_reports = build_stage_reports(evidence, request.transition_root)?;
    let stage_graph_digest_blake3 = digest_serialized(STAGE_GRAPH_DIGEST_DOMAIN, &evidence.plan.stages)?;
    let final_bundle_digest_blake3 = final_bundle_digest(FinalBundleInput {
        evidence,
        validation: &validation,
        stage_reports: &stage_reports,
        stage_graph_digest_blake3: &stage_graph_digest_blake3,
        normalized_provider_digest_blake3: &normalized_provider_digest_blake3,
        output_digest_blake3: &output_digest_blake3,
        provider_validation_report_digest_blake3: &provider_validation_report_digest_blake3,
        provider_outputs: &roles,
    })?;
    let digests = ProviderDigests {
        normalized_provider_digest_blake3,
        output_digest_blake3,
        stage_graph_digest_blake3,
        provider_validation_report_digest_blake3,
        final_bundle_digest_blake3,
    };
    let receipt = build_provider_receipt(evidence, stage_reports, roles, &validation, &digests)?;
    write_json_create_new(&staging_path.join(PROVIDER_VALIDATION_RELATIVE_PATH), &validation)?;
    write_json_create_new(&staging_path.join(PROVIDER_RECEIPT_RELATIVE_PATH), &receipt)?;
    independently_validate_staging(staging_path, evidence, &receipt)?;
    Ok(StagexProviderPublicationReport {
        schema: PROVIDER_SCHEMA,
        provider_kind: PROVIDER_KIND,
        output_path: PathBuf::new(),
        receipt_path: PathBuf::new(),
        normalized_provider_digest_blake3: digests.normalized_provider_digest_blake3,
        output_digest_blake3: digests.output_digest_blake3,
        final_bundle_digest_blake3: digests.final_bundle_digest_blake3,
        provider_validation_audit_digest_blake3: validation.audit_digest_blake3,
        provider_validation_report_digest_blake3: digests.provider_validation_report_digest_blake3,
    })
}

fn validate_request_paths(request: &StagexProviderRequest<'_>) -> Result<(), StagexProviderError> {
    require_absolute_regular_file(request.lineage_manifest_path, "lineage manifest")?;
    require_absolute_directory(request.transition_root, "transition root")?;
    if !request.output_path.is_absolute() || request.output_path.exists() {
        return Err(StagexProviderError::InvalidInput(format!(
            "provider output must be an absent absolute path: {}",
            request.output_path.display()
        )));
    }
    let parent = request.output_path.parent().ok_or_else(|| {
        StagexProviderError::InvalidInput(format!("provider output has no parent: {}", request.output_path.display()))
    })?;
    if !parent.is_dir() {
        return Err(StagexProviderError::InvalidInput(format!(
            "provider output parent is not a directory: {}",
            parent.display()
        )));
    }
    assert!(request.lineage_manifest_path.is_absolute());
    assert!(request.transition_root.is_absolute());
    Ok(())
}

fn load_transition_evidence(request: &StagexProviderRequest<'_>) -> Result<TransitionEvidence, StagexProviderError> {
    let manifest_bytes = read_bounded_file(request.lineage_manifest_path, JSON_MANIFEST_BYTES_MAX, "lineage manifest")?;
    let manifest = crate::bootstrap_source_root::validate_stagex_lineage_manifest(&manifest_bytes)
        .map_err(|errors| StagexProviderError::Transition(crate::bootstrap_source_root::format_diagnostics(&errors)))?;
    let provider_boundary = crunch_bootstrap_core::validate_provider_boundary(&manifest, &[]);
    if !provider_boundary.is_valid() {
        let missing_roles =
            provider_boundary.missing_roles.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
        return Err(StagexProviderError::Transition(format!(
            "lineage manifest is missing provider roles: {missing_roles}"
        )));
    }
    let manifest_digest_blake3 = blake3::hash(&manifest_bytes).to_hex().to_string();
    let manifest_value: Value = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| StagexProviderError::Transition(format!("parsing lineage manifest: {error}")))?;
    let manifest_outputs = manifest_output_digests(&manifest_value)?;
    let plan_path = request.transition_root.join(TRANSITION_PLAN_FILE_NAME);
    let report_path = request.transition_root.join(TRANSITION_REPORT_FILE_NAME);
    let audit_path = request.transition_root.join(TRANSITION_AUDIT_FILE_NAME);
    let plan_bytes = read_bounded_file(&plan_path, JSON_PLAN_BYTES_MAX, "transition plan")?;
    let report_bytes = read_bounded_file(&report_path, JSON_REPORT_BYTES_MAX, "transition report")?;
    let audit_bytes = read_bounded_file(&audit_path, JSON_AUDIT_BYTES_MAX, "protected execution audit")?;
    let plan: StagexMaterializationPlan = serde_json::from_slice(&plan_bytes)
        .map_err(|error| StagexProviderError::Transition(format!("parsing transition plan: {error}")))?;
    let report: Value = serde_json::from_slice(&report_bytes)
        .map_err(|error| StagexProviderError::Transition(format!("parsing transition report: {error}")))?;
    let audit_events: Vec<ProtectedSeccompAuditEvent> = serde_json::from_slice(&audit_bytes)
        .map_err(|error| StagexProviderError::Transition(format!("parsing protected execution audit: {error}")))?;
    let plan_digest_blake3 =
        stagex_plan_digest_blake3(&plan).map_err(|error| StagexProviderError::Transition(error.to_string()))?;
    validate_transition_headers(&plan, &report, &audit_events, &manifest_digest_blake3, &plan_digest_blake3)?;
    validate_report_audit_binding(&report, &audit_events)?;
    let observed_outputs = collect_observed_transition_outputs(&report)?;
    validate_manifest_provider_authority(&manifest, &observed_outputs, &manifest_outputs)?;
    assert!(!plan.stages.is_empty());
    assert!(!audit_events.is_empty());
    Ok(TransitionEvidence {
        manifest_digest_blake3,
        plan,
        plan_digest_blake3,
        report_digest_blake3: blake3::hash(&report_bytes).to_hex().to_string(),
        audit_digest_blake3: blake3::hash(&audit_bytes).to_hex().to_string(),
        report,
        audit_events,
        observed_outputs,
        manifest_outputs,
    })
}

fn validate_manifest_provider_authority(
    manifest: &crunch_bootstrap_core::LineageManifest,
    observed_outputs: &BTreeMap<String, ObservedTransitionOutput>,
    manifest_outputs: &BTreeMap<String, String>,
) -> Result<(), StagexProviderError> {
    for provider_output in &manifest.provider_outputs {
        let artifact_id = &provider_output.producing_artifact_id;
        let expected_digest = manifest_outputs.get(artifact_id).ok_or_else(|| {
            StagexProviderError::Transition(format!(
                "provider role {} lacks manifest artifact authority for {artifact_id}",
                provider_output.role
            ))
        })?;
        let observed = observed_outputs.get(artifact_id).ok_or_else(|| {
            StagexProviderError::Transition(format!(
                "provider role {} artifact {artifact_id} lacks a transition observation",
                provider_output.role
            ))
        })?;
        if &observed.digest_blake3 != expected_digest {
            return Err(StagexProviderError::Transition(format!(
                "provider role {} artifact {artifact_id} digest mismatch",
                provider_output.role
            )));
        }
    }
    assert_eq!(manifest.provider_outputs.len(), PROVIDER_ROLE_COUNT);
    assert!(
        manifest
            .provider_outputs
            .iter()
            .all(|output| observed_outputs.contains_key(&output.producing_artifact_id))
    );
    Ok(())
}

fn validate_transition_headers(
    plan: &StagexMaterializationPlan,
    report: &Value,
    audit_events: &[ProtectedSeccompAuditEvent],
    manifest_digest_blake3: &str,
    plan_digest_blake3: &str,
) -> Result<(), StagexProviderError> {
    let plan_validation = validate_stagex_plan(plan);
    if !plan_validation.is_valid() {
        return Err(StagexProviderError::Transition(format!(
            "plan failed core validation: {}",
            plan_validation.errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
        )));
    }
    require_json_string(report, "schema_version", TRANSITION_REPORT_SCHEMA)?;
    require_json_string(report, "status", TRANSITION_STATUS_COMPLETE)?;
    require_json_string(report, "plan_digest_blake3", plan_digest_blake3)?;
    require_json_string(report, "source_state_digest_blake3", &plan.source_state_digest_blake3)?;
    if plan.lineage_manifest_digest_blake3 != manifest_digest_blake3 {
        return Err(StagexProviderError::Transition(format!(
            "plan lineage manifest digest mismatch: expected {manifest_digest_blake3}, observed {}",
            plan.lineage_manifest_digest_blake3
        )));
    }
    if audit_events.is_empty() || audit_events.len() > TRANSITION_AUDIT_EVENT_COUNT_MAX {
        return Err(StagexProviderError::Transition(format!(
            "protected audit event count {} is outside 1..={TRANSITION_AUDIT_EVENT_COUNT_MAX}",
            audit_events.len()
        )));
    }
    reject_report_fallback_events(report)?;
    validate_allowed_audit_events(audit_events)?;
    assert!(plan_validation.errors.is_empty());
    assert!(audit_events.len() <= TRANSITION_AUDIT_EVENT_COUNT_MAX);
    Ok(())
}

fn validate_report_audit_binding(
    report: &Value,
    audit_events: &[ProtectedSeccompAuditEvent],
) -> Result<(), StagexProviderError> {
    let report_events = report
        .get("protected_exec_events")
        .and_then(Value::as_array)
        .ok_or_else(|| StagexProviderError::Transition("transition report lacks protected_exec_events".to_string()))?;
    if report_events.len() != audit_events.len() {
        return Err(StagexProviderError::Transition(format!(
            "report/audit event count mismatch: report {}, audit {}",
            report_events.len(),
            audit_events.len()
        )));
    }
    for (index, (reported, audited)) in report_events.iter().zip(audit_events).enumerate() {
        let parsed: ProtectedSeccompAuditEvent = serde_json::from_value(reported.clone()).map_err(|error| {
            StagexProviderError::Transition(format!("parsing transition report event {index}: {error}"))
        })?;
        if &parsed != audited {
            return Err(StagexProviderError::Transition(format!("report/audit event mismatch at index {index}")));
        }
    }
    assert_eq!(report_events.len(), audit_events.len());
    assert!(!report_events.is_empty());
    Ok(())
}

fn validate_allowed_audit_events(events: &[ProtectedSeccompAuditEvent]) -> Result<(), StagexProviderError> {
    for (index, event) in events.iter().enumerate() {
        if !is_allowed_protected_exec_event(event) {
            return Err(StagexProviderError::Transition(format!(
                "audit event {index} is not an allowed protected exec decision"
            )));
        }
        if event.inventory_entry_id.as_deref().is_none_or(str::is_empty) {
            return Err(StagexProviderError::Transition(format!("audit event {index} lacks authorization identity")));
        }
        validate_blake3(&event.digest_hex, "audit event digest")?;
    }
    assert!(events.iter().all(is_allowed_protected_exec_event));
    assert!(events.iter().all(|event| event.phase == "protected"));
    Ok(())
}

fn is_allowed_protected_exec_event(event: &ProtectedSeccompAuditEvent) -> bool {
    let allowed = event.policy_decision == "allowed";
    let protected = event.phase == "protected";
    let exec_syscall = PROTECTED_EXEC_SYSCALLS.contains(&event.syscall.as_str());
    assert_eq!(PROTECTED_EXEC_SYSCALLS.len(), PROTECTED_EXEC_SYSCALL_COUNT);
    assert!(!PROTECTED_EXEC_SYSCALLS.iter().any(|syscall| syscall.is_empty()));
    allowed && protected && exec_syscall
}

fn reject_report_fallback_events(report: &Value) -> Result<(), StagexProviderError> {
    let mut pending = vec![report];
    let mut checked = 0_usize;
    while let Some(value) = pending.pop() {
        checked = checked.saturating_add(1);
        if checked > PROVIDER_OUTPUT_COUNT_MAX.saturating_mul(TRANSITION_AUDIT_EVENT_COUNT_MAX) {
            return Err(StagexProviderError::Transition("transition report value count exceeds bound".to_string()));
        }
        match value {
            Value::Object(object) => {
                if let Some(fallbacks) = object.get("fallback_events") {
                    let entries = fallbacks.as_array().ok_or_else(|| {
                        StagexProviderError::Transition("fallback_events is not an array".to_string())
                    })?;
                    if !entries.is_empty() {
                        return Err(StagexProviderError::Transition(
                            "transition report contains fallback events".to_string(),
                        ));
                    }
                }
                pending.extend(object.values());
            }
            Value::Array(values) => pending.extend(values),
            _ => {}
        }
    }
    assert!(checked > 0);
    assert!(pending.is_empty());
    Ok(())
}

fn collect_observed_transition_outputs(
    report: &Value,
) -> Result<BTreeMap<String, ObservedTransitionOutput>, StagexProviderError> {
    let mut outputs = BTreeMap::new();
    let mut pending = vec![report];
    while let Some(value) = pending.pop() {
        match value {
            Value::Object(object) => {
                if object.contains_key("artifact_id")
                    && object.contains_key("path")
                    && object.contains_key("bytes_len")
                    && object.contains_key("digest_blake3")
                {
                    let output: ObservedTransitionOutput = serde_json::from_value(value.clone()).map_err(|error| {
                        StagexProviderError::Transition(format!("parsing observed transition output: {error}"))
                    })?;
                    insert_observed_output(&mut outputs, output)?;
                }
                pending.extend(object.values());
            }
            Value::Array(values) => pending.extend(values),
            _ => {}
        }
        if outputs.len() > PROVIDER_OUTPUT_COUNT_MAX {
            return Err(StagexProviderError::Transition(format!(
                "transition output observations exceed {PROVIDER_OUTPUT_COUNT_MAX}"
            )));
        }
    }
    if outputs.is_empty() {
        return Err(StagexProviderError::Transition("transition report contains no output observations".to_string()));
    }
    assert!(outputs.len() <= PROVIDER_OUTPUT_COUNT_MAX);
    assert!(outputs.values().all(|output| !output.artifact_id.is_empty()));
    Ok(outputs)
}

fn insert_observed_output(
    outputs: &mut BTreeMap<String, ObservedTransitionOutput>,
    output: ObservedTransitionOutput,
) -> Result<(), StagexProviderError> {
    validate_blake3(&output.digest_blake3, "transition output digest")?;
    if !output.path.is_absolute() {
        return Err(StagexProviderError::Transition(format!(
            "transition output {} has a relative path",
            output.artifact_id
        )));
    }
    if let Some(existing) = outputs.get(&output.artifact_id) {
        if existing.digest_blake3 != output.digest_blake3 || existing.bytes_len != output.bytes_len {
            return Err(StagexProviderError::Transition(format!(
                "conflicting transition output observation for {}",
                output.artifact_id
            )));
        }
        if output.path < existing.path {
            outputs.insert(output.artifact_id.clone(), output);
        }
        return Ok(());
    }
    outputs.insert(output.artifact_id.clone(), output);
    Ok(())
}

fn manifest_output_digests(manifest: &Value) -> Result<BTreeMap<String, String>, StagexProviderError> {
    let entries = manifest
        .get("generated_artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| StagexProviderError::Transition("lineage manifest lacks generated_artifacts".to_string()))?;
    let mut outputs = BTreeMap::new();
    for entry in entries {
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| StagexProviderError::Transition("generated artifact lacks id".to_string()))?;
        let digest = entry
            .get("digest")
            .and_then(|value| value.get("hex_value"))
            .and_then(Value::as_str)
            .ok_or_else(|| StagexProviderError::Transition(format!("generated artifact {id} lacks digest")))?;
        validate_blake3(digest, "manifest generated artifact digest")?;
        if outputs.insert(id.to_string(), digest.to_string()).is_some() {
            return Err(StagexProviderError::Transition(format!("duplicate generated artifact {id}")));
        }
    }
    assert_eq!(outputs.len(), entries.len());
    assert!(!outputs.is_empty());
    Ok(outputs)
}

fn copy_provider_payload(
    transition_root: &Path,
    staging: &Path,
    evidence: &TransitionEvidence,
) -> Result<(), StagexProviderError> {
    let tcc =
        require_observed_component(evidence, "tcc-musl-selfhost", &transition_root.join(TRANSITION_TCC_RELATIVE_PATH))?;
    copy_file_create_new(&tcc.path, &staging.join(PROVIDER_TCC_RELATIVE_PATH), true)?;
    copy_tree_create_new(
        &transition_root.join(TRANSITION_TCC_RUNTIME_RELATIVE_PATH),
        &staging.join(PROVIDER_TCC_RUNTIME_RELATIVE_PATH),
    )?;
    copy_tree_create_new(
        &transition_root.join(TRANSITION_TCC_SOURCE_RELATIVE_PATH),
        &staging.join(PROVIDER_TCC_SOURCE_RELATIVE_PATH),
    )?;
    copy_tree_create_new(
        &transition_root.join(TRANSITION_MUSL_INCLUDE_RELATIVE_PATH),
        &staging.join(PROVIDER_MUSL_INCLUDE_RELATIVE_PATH),
    )?;
    validate_native_library_set(&transition_root.join(TRANSITION_MUSL_LIB_RELATIVE_PATH))?;
    copy_tree_create_new(
        &transition_root.join(TRANSITION_MUSL_LIB_RELATIVE_PATH),
        &staging.join(PROVIDER_MUSL_LIB_RELATIVE_PATH),
    )?;
    copy_binutils_payload(transition_root, staging, evidence)?;
    validate_static_provider_payload(staging, evidence)?;
    assert!(staging.join(PROVIDER_TCC_RELATIVE_PATH).is_file());
    assert!(staging.join(PROVIDER_MUSL_INCLUDE_RELATIVE_PATH).is_dir());
    Ok(())
}

fn copy_binutils_payload(
    transition_root: &Path,
    staging: &Path,
    evidence: &TransitionEvidence,
) -> Result<(), StagexProviderError> {
    let source_root = transition_root.join(TRANSITION_BINUTILS_ROOT_RELATIVE_PATH);
    for tool in BINUTILS_TOOL_NAMES {
        let artifact_id = format!("binutils-installed-{tool}");
        let source = source_root.join("bin").join(tool);
        require_observed_component(evidence, &artifact_id, &source)?;
        let destination = staging.join("bin").join(format!("{PROVIDER_TARGET}-{tool}"));
        copy_file_create_new(&source, &destination, true)?;
    }
    copy_tree_create_new(&source_root.join("include"), &staging.join(PROVIDER_BINUTILS_INCLUDE_RELATIVE_PATH))?;
    copy_tree_create_new(&source_root.join("lib/ldscripts"), &staging.join(PROVIDER_BINUTILS_LD_RELATIVE_PATH))?;
    assert_eq!(BINUTILS_TOOL_NAMES.len(), crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS.len());
    assert!(staging.join(PROVIDER_BINUTILS_LD_RELATIVE_PATH).is_dir());
    Ok(())
}

fn require_observed_component<'a>(
    evidence: &'a TransitionEvidence,
    artifact_id: &str,
    expected_path: &Path,
) -> Result<&'a ObservedTransitionOutput, StagexProviderError> {
    let output = evidence
        .observed_outputs
        .get(artifact_id)
        .ok_or_else(|| StagexProviderError::Transition(format!("missing observed provider component {artifact_id}")))?;
    if output.path != expected_path {
        return Err(StagexProviderError::Transition(format!(
            "provider component {artifact_id} path mismatch: expected {}, observed {}",
            expected_path.display(),
            output.path.display()
        )));
    }
    validate_observed_path(output)?;
    assert_eq!(output.path, expected_path);
    assert!(!output.digest_blake3.is_empty());
    Ok(output)
}

fn validate_observed_path(output: &ObservedTransitionOutput) -> Result<(), StagexProviderError> {
    let (bytes_len, digest_blake3) = observe_path_identity(&output.path)?;
    if bytes_len != output.bytes_len || digest_blake3 != output.digest_blake3 {
        return Err(StagexProviderError::Transition(format!(
            "provider component {} content mismatch: expected {} bytes {}, observed {bytes_len} bytes {digest_blake3}",
            output.artifact_id, output.bytes_len, output.digest_blake3
        )));
    }
    assert_eq!(bytes_len, output.bytes_len);
    assert_eq!(digest_blake3, output.digest_blake3);
    Ok(())
}

fn validate_native_library_set(root: &Path) -> Result<(), StagexProviderError> {
    let observed = sorted_child_names(root)?;
    let expected = NATIVE_LIBRARY_NAMES.iter().map(|name| (*name).to_string()).collect::<Vec<_>>();
    if observed != expected {
        return Err(StagexProviderError::Provider(format!(
            "native musl library set mismatch: expected {expected:?}, observed {observed:?}"
        )));
    }
    assert_eq!(observed.len(), NATIVE_LIBRARY_NAMES.len());
    assert!(observed.iter().all(|name| !name.is_empty()));
    Ok(())
}

fn validate_static_provider_payload(staging: &Path, evidence: &TransitionEvidence) -> Result<(), StagexProviderError> {
    let tcc_digest = require_regular_executable_elf(&staging.join(PROVIDER_TCC_RELATIVE_PATH))?;
    let expected_tcc = evidence.observed_outputs.get("tcc-musl-selfhost").expect("copied TCC has prior observation");
    if tcc_digest != expected_tcc.digest_blake3 {
        return Err(StagexProviderError::Provider("relocated TinyCC digest changed".to_string()));
    }
    for tool in BINUTILS_TOOL_NAMES {
        let path = staging.join("bin").join(format!("{PROVIDER_TARGET}-{tool}"));
        let digest = require_regular_executable_elf(&path)?;
        let expected = evidence
            .observed_outputs
            .get(&format!("binutils-installed-{tool}"))
            .expect("copied binutils tool has prior observation");
        if digest != expected.digest_blake3 || digest == tcc_digest {
            return Err(StagexProviderError::Provider(format!(
                "relocated binutils tool {tool} changed or delegates to TinyCC"
            )));
        }
    }
    validate_required_header(&staging.join(PROVIDER_MUSL_INCLUDE_RELATIVE_PATH).join("stddef.h"))?;
    validate_required_header(&staging.join(PROVIDER_MUSL_INCLUDE_RELATIVE_PATH).join("sys/types.h"))?;
    validate_required_file(&staging.join(PROVIDER_MUSL_LIB_RELATIVE_PATH).join("libc.a"), "native libc")?;
    assert_eq!(BINUTILS_TOOL_NAMES.len(), crate::stagex_binutils::BINUTILS_REQUIRED_TOOLS.len());
    assert_eq!(tcc_digest, expected_tcc.digest_blake3);
    Ok(())
}

fn validate_required_header(path: &Path) -> Result<(), StagexProviderError> {
    validate_required_file(path, "provider header")?;
    let bytes = read_bounded_file(path, PROVIDER_FILE_BYTES_MAX, "provider header")?;
    if bytes.is_empty() {
        return Err(StagexProviderError::Provider(format!("provider header is empty: {}", path.display())));
    }
    assert!(path.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn validate_required_file(path: &Path, label: &str) -> Result<(), StagexProviderError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| StagexProviderError::Provider(format!("reading {label} {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() == 0 {
        return Err(StagexProviderError::Provider(format!(
            "{label} is not a non-empty regular file: {}",
            path.display()
        )));
    }
    assert!(metadata.is_file());
    assert!(metadata.len() > 0);
    Ok(())
}

fn require_regular_executable_elf(path: &Path) -> Result<String, StagexProviderError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| StagexProviderError::Provider(format!("reading provider tool {}: {error}", path.display())))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.permissions().mode() & EXECUTABLE_MODE_BITS == 0
    {
        return Err(StagexProviderError::Provider(format!(
            "provider tool is not a regular executable file: {}",
            path.display()
        )));
    }
    let bytes = read_bounded_file(path, PROVIDER_FILE_BYTES_MAX, "provider tool")?;
    if bytes.get(..ELF_MAGIC_BYTES) != Some(ELF_MAGIC) {
        return Err(StagexProviderError::Provider(format!("provider tool is not ELF: {}", path.display())));
    }
    let digest = blake3::hash(&bytes).to_hex().to_string();
    assert!(bytes.len() >= ELF_MAGIC_BYTES);
    assert_eq!(digest.len(), blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
    Ok(digest)
}

fn validate_relocated_provider_runtime(
    staging: &Path,
    transition_root: &Path,
) -> Result<ProviderValidationReport, StagexProviderError> {
    let validation_root = prepare_provider_validation_root(staging)?;
    let supervisor = install_provider_validation_supervisor(staging, transition_root)?;
    run_tcc_positive_smoke(staging, &validation_root, &supervisor)?;
    let negative_tcc_rejection = run_tcc_negative_smoke(staging, &validation_root)?;
    run_binutils_positive_smoke(staging, &validation_root)?;
    let negative_assembler_rejection = run_binutils_negative_smoke(staging, &validation_root)?;
    let positive = validation_root.join("binutils-positive");
    let positive_digest = blake3_file_hex(&positive)
        .map_err(|error| StagexProviderError::Runtime(format!("hashing binutils positive smoke: {error}")))?;
    supervisor
        .promote_verified_output("provider-relocation-smoke", &[], &[PromotedExecutable {
            path: positive.clone(),
            digest_hex: positive_digest,
        }])
        .map_err(|error| StagexProviderError::Runtime(format!("promoting provider smoke output: {error}")))?;
    let positive_status = run_expected_status(&positive, &[] as &[&str], &validation_root, "positive-execution")?;
    reap_adopted_exec_descendants()
        .map_err(|error| StagexProviderError::Runtime(format!("reaping provider validation descendants: {error}")))?;
    supervisor
        .wait_for_audit_quiescence()
        .map_err(|error| StagexProviderError::Runtime(format!("waiting for provider validation audit: {error}")))?;
    let projected = project_provider_validation_events(staging, &supervisor.audit_events())?;
    validate_provider_validation_event_sequence(&projected)?;
    let audit_digest_blake3 = digest_serialized(PROVIDER_VALIDATION_AUDIT_DIGEST_DOMAIN, &projected)?;
    validate_runtime_smoke_results(positive_status, negative_tcc_rejection, negative_assembler_rejection)?;
    remove_provider_validation_root(&validation_root)?;
    assert!(!validation_root.exists());
    assert!(!projected.is_empty());
    let report = ProviderValidationReport {
        schema: PROVIDER_VALIDATION_SCHEMA.to_string(),
        status: TRANSITION_STATUS_COMPLETE.to_string(),
        audit_digest_blake3,
        events: projected,
        positive_tcc_compile: true,
        negative_tcc_rejection,
        positive_binutils_execution_status: positive_status,
        negative_assembler_rejection,
        fallback_events: Vec::new(),
    };
    validate_provider_validation_report(&report)?;
    assert!(report.positive_tcc_compile);
    assert!(report.fallback_events.is_empty());
    Ok(report)
}

fn prepare_provider_validation_root(staging: &Path) -> Result<PathBuf, StagexProviderError> {
    let validation_root = staging.join(".validation");
    fs::create_dir(&validation_root).map_err(|error| {
        StagexProviderError::Runtime(format!(
            "creating provider validation root {}: {error}",
            validation_root.display()
        ))
    })?;
    write_file_create_new(&validation_root.join("empty.stdin"), EMPTY_INPUT, PROVIDER_DATA_MODE)?;
    write_file_create_new(&validation_root.join("positive.c"), TCC_SMOKE_SOURCE, PROVIDER_DATA_MODE)?;
    write_file_create_new(&validation_root.join("positive.s"), BINUTILS_SMOKE_SOURCE, PROVIDER_DATA_MODE)?;
    write_file_create_new(&validation_root.join("negative.s"), BINUTILS_NEGATIVE_SOURCE, PROVIDER_DATA_MODE)?;
    assert!(validation_root.is_dir());
    assert!(!validation_root.join("binutils-positive").exists());
    Ok(validation_root)
}

fn validate_runtime_smoke_results(
    positive_status: i32,
    negative_tcc_rejection: bool,
    negative_assembler_rejection: bool,
) -> Result<(), StagexProviderError> {
    if positive_status != RUNTIME_SMOKE_EXPECTED_EXIT {
        return Err(StagexProviderError::Runtime(format!("provider binutils smoke exited {positive_status}")));
    }
    if !negative_tcc_rejection {
        return Err(StagexProviderError::Runtime(
            "provider TinyCC negative smoke did not reject its input".to_string(),
        ));
    }
    if !negative_assembler_rejection {
        return Err(StagexProviderError::Runtime(
            "provider assembler negative smoke did not reject its input".to_string(),
        ));
    }
    assert_eq!(positive_status, RUNTIME_SMOKE_EXPECTED_EXIT);
    assert!(negative_tcc_rejection);
    assert!(negative_assembler_rejection);
    Ok(())
}

fn remove_provider_validation_root(validation_root: &Path) -> Result<(), StagexProviderError> {
    fs::remove_dir_all(validation_root).map_err(|error| {
        StagexProviderError::Runtime(format!(
            "removing provider validation root {}: {error}",
            validation_root.display()
        ))
    })?;
    assert!(!validation_root.exists());
    assert!(validation_root.is_absolute());
    Ok(())
}

fn install_provider_validation_supervisor(
    staging: &Path,
    transition_root: &Path,
) -> Result<crate::protected_exec_seccomp::ProtectedSeccompSupervisor, StagexProviderError> {
    let source_stage_id = "stagex-provider-normalization".to_string();
    let promotion_stage_id = "provider-relocation-smoke".to_string();
    let allowed_source_stage_ids = vec![source_stage_id.clone(), promotion_stage_id];
    let mut planned = Vec::with_capacity(PROVIDER_VALIDATION_PLANNED_EXECUTABLE_COUNT);
    for (authorization_id, relative_path) in [
        ("exec:provider-validation:tcc", PROVIDER_TCC_RELATIVE_PATH.to_string()),
        ("exec:provider-validation:as", format!("bin/{PROVIDER_TARGET}-as")),
        ("exec:provider-validation:ld", format!("bin/{PROVIDER_TARGET}-ld")),
    ] {
        let path = staging.join(&relative_path);
        planned.push(PlannedExecutable {
            authorization_id: authorization_id.to_string(),
            source_stage_id: source_stage_id.clone(),
            digest_hex: blake3_file_hex(&path)
                .map_err(|error| StagexProviderError::Runtime(format!("hashing provider executable: {error}")))?,
            path,
        });
    }
    let seed = transition_root.join(TRANSITION_SEED_RELATIVE_PATH);
    let policy = ProtectedExecPolicy::from_stagex_plan(
        seed.clone(),
        crate::stagex_transition::HEX0_SEED_BLAKE3.to_string(),
        &allowed_source_stage_ids,
        &planned,
    )
    .map_err(|error| StagexProviderError::Runtime(format!("creating provider validation policy: {error}")))?;
    let supervisor = install_current_thread_exec_supervisor(policy)
        .map_err(|error| StagexProviderError::Runtime(format!("installing provider validation supervisor: {error}")))?;
    assert_eq!(planned.len(), PROVIDER_VALIDATION_PLANNED_EXECUTABLE_COUNT);
    assert!(seed.is_file());
    Ok(supervisor)
}

fn run_tcc_positive_smoke(
    staging: &Path,
    validation: &Path,
    supervisor: &crate::protected_exec_seccomp::ProtectedSeccompSupervisor,
) -> Result<(), StagexProviderError> {
    let tcc = staging.join(PROVIDER_TCC_RELATIVE_PATH);
    let tcc_runtime = staging.join(PROVIDER_TCC_RUNTIME_RELATIVE_PATH);
    let musl_include = staging.join(PROVIDER_MUSL_INCLUDE_RELATIVE_PATH);
    let musl_lib = staging.join(PROVIDER_MUSL_LIB_RELATIVE_PATH);
    let arguments = vec![
        format!("-B{}", utf8_path(&tcc_runtime, "TCC runtime")?),
        "-nostdinc".to_string(),
        format!("-I{}/include", utf8_path(&tcc_runtime, "TCC runtime")?),
        format!("-I{}", utf8_path(&musl_include, "musl include")?),
        "-nostdlib".to_string(),
        "-static".to_string(),
        utf8_path(&musl_lib.join("crt1.o"), "crt1")?.to_string(),
        utf8_path(&musl_lib.join("crti.o"), "crti")?.to_string(),
        "positive.c".to_string(),
        utf8_path(&musl_lib.join("libc.a"), "libc")?.to_string(),
        utf8_path(&tcc_runtime.join("libtcc1.a"), "libtcc1")?.to_string(),
        utf8_path(&musl_lib.join("crtn.o"), "crtn")?.to_string(),
        "-o".to_string(),
        "tcc-positive".to_string(),
    ];
    run_success(&tcc, &arguments, validation, "tcc-positive")?;
    let positive = validation.join("tcc-positive");
    fs::set_permissions(&positive, fs::Permissions::from_mode(PROVIDER_EXECUTABLE_MODE))
        .map_err(|error| StagexProviderError::Runtime(format!("setting TCC smoke mode: {error}")))?;
    let positive_digest = blake3_file_hex(&positive)
        .map_err(|error| StagexProviderError::Runtime(format!("hashing TCC positive smoke: {error}")))?;
    supervisor
        .promote_verified_output("provider-relocation-smoke", &[], &[PromotedExecutable {
            path: positive.clone(),
            digest_hex: positive_digest,
        }])
        .map_err(|error| StagexProviderError::Runtime(format!("promoting TCC smoke output: {error}")))?;
    let status = run_expected_status(&positive, &[] as &[&str], validation, "tcc-execution")?;
    if status != RUNTIME_SMOKE_EXPECTED_EXIT {
        return Err(StagexProviderError::Runtime(format!("relocated TCC smoke exited {status}")));
    }
    assert!(validation.join("tcc-positive").is_file());
    assert_eq!(status, RUNTIME_SMOKE_EXPECTED_EXIT);
    Ok(())
}

fn run_tcc_negative_smoke(staging: &Path, validation: &Path) -> Result<bool, StagexProviderError> {
    let tcc = staging.join(PROVIDER_TCC_RELATIVE_PATH);
    let arguments = ["-c", "missing-source.c", "-o", "negative.o"];
    let status = run_expected_status(&tcc, &arguments, validation, "tcc-negative")?;
    if status == 0 {
        return Err(StagexProviderError::Runtime("relocated TinyCC accepted a missing source input".to_string()));
    }
    if validation.join("negative.o").exists() {
        return Err(StagexProviderError::Runtime(
            "relocated TinyCC left output after rejecting a missing source input".to_string(),
        ));
    }
    assert_ne!(status, 0);
    assert!(!validation.join("negative.o").exists());
    Ok(true)
}

fn run_binutils_positive_smoke(staging: &Path, validation: &Path) -> Result<(), StagexProviderError> {
    let assembler = staging.join("bin").join(format!("{PROVIDER_TARGET}-as"));
    let linker = staging.join("bin").join(format!("{PROVIDER_TARGET}-ld"));
    run_success(&assembler, &["positive.s", "-o", "positive.o"], validation, "as-positive")?;
    run_success(&linker, &["positive.o", "-o", "binutils-positive"], validation, "ld-positive")?;
    let output = validation.join("binutils-positive");
    fs::set_permissions(&output, fs::Permissions::from_mode(PROVIDER_EXECUTABLE_MODE))
        .map_err(|error| StagexProviderError::Runtime(format!("setting binutils smoke mode: {error}")))?;
    require_regular_executable_elf(&output)?;
    assert!(validation.join("positive.o").is_file());
    assert!(output.is_file());
    Ok(())
}

fn run_binutils_negative_smoke(staging: &Path, validation: &Path) -> Result<bool, StagexProviderError> {
    let assembler = staging.join("bin").join(format!("{PROVIDER_TARGET}-as"));
    let status = run_expected_status(&assembler, &["negative.s", "-o", "negative-as.o"], validation, "as-negative")?;
    if status == 0 {
        return Err(StagexProviderError::Runtime("relocated assembler accepted malformed input".to_string()));
    }
    if validation.join("negative-as.o").exists() {
        return Err(StagexProviderError::Runtime(
            "relocated assembler left output after rejecting malformed input".to_string(),
        ));
    }
    assert_ne!(status, 0);
    assert!(!validation.join("negative-as.o").exists());
    Ok(true)
}

fn run_success<S: AsRef<std::ffi::OsStr>>(
    executable: &Path,
    arguments: &[S],
    current_dir: &Path,
    label: &str,
) -> Result<(), StagexProviderError> {
    crate::stagex_mes_lib::run_bounded_process(
        executable,
        arguments,
        current_dir,
        &BTreeMap::new(),
        &current_dir.join(format!("{label}.stderr.txt")),
    )
    .map_err(|error| StagexProviderError::Runtime(error.to_string()))?;
    assert!(executable.is_absolute());
    assert!(current_dir.is_absolute());
    Ok(())
}

fn run_expected_status<S: AsRef<std::ffi::OsStr>>(
    executable: &Path,
    arguments: &[S],
    current_dir: &Path,
    label: &str,
) -> Result<i32, StagexProviderError> {
    let stdin = current_dir.join("empty.stdin");
    let stdout = current_dir.join(format!("{label}.stdout.txt"));
    let stderr = current_dir.join(format!("{label}.stderr.txt"));
    let status = crate::stagex_mes_lib::run_bounded_process_with_stdin_capturing_stdout_status(
        executable,
        arguments,
        current_dir,
        &BTreeMap::new(),
        &stdin,
        PROVIDER_FILE_BYTES_MAX,
        &stdout,
        PROVIDER_FILE_BYTES_MAX,
        &stderr,
    )
    .map_err(|error| StagexProviderError::Runtime(error.to_string()))?;
    assert!(stdout.is_file());
    assert!(stderr.is_file());
    Ok(status)
}

fn project_provider_validation_events(
    staging: &Path,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<Vec<ProviderValidationEvent>, StagexProviderError> {
    if events.is_empty() || events.len() > PROVIDER_VALIDATION_EVENT_COUNT_MAX {
        return Err(StagexProviderError::Runtime(format!(
            "provider validation audit count {} is outside 1..={PROVIDER_VALIDATION_EVENT_COUNT_MAX}",
            events.len()
        )));
    }
    let mut projected = Vec::with_capacity(events.len());
    for event in events {
        if event.policy_decision != "allowed" || event.phase != "protected" {
            return Err(StagexProviderError::Runtime("provider validation audit contains a denied event".to_string()));
        }
        let relative = event.resolved_host_path.strip_prefix(staging).map_err(|_| {
            StagexProviderError::Runtime(format!(
                "provider validation executed outside staging: {}",
                event.resolved_host_path.display()
            ))
        })?;
        let relative_path = utf8_path(relative, "provider validation executable")?;
        let authorization_id = portable_validation_authorization_id(event, relative_path)?;
        projected.push(ProviderValidationEvent {
            executable_relative_path: relative_path.to_string(),
            digest_blake3: event.digest_hex.clone(),
            authorization_id,
            policy_decision: event.policy_decision.clone(),
        });
    }
    assert_eq!(projected.len(), events.len());
    assert!(projected.iter().all(|event| event.policy_decision == "allowed"));
    Ok(projected)
}

fn validate_provider_validation_event_sequence(events: &[ProviderValidationEvent]) -> Result<(), StagexProviderError> {
    if events.len() != PROVIDER_VALIDATION_EVENT_COUNT_EXPECTED {
        return Err(StagexProviderError::Runtime(format!(
            "provider validation recorded {} events; expected {PROVIDER_VALIDATION_EVENT_COUNT_EXPECTED}",
            events.len()
        )));
    }
    for (index, event) in events.iter().enumerate() {
        if event.policy_decision != "allowed" {
            return Err(StagexProviderError::Runtime(format!(
                "provider validation event {} was not allowed",
                event.executable_relative_path
            )));
        }
        let expected_path = PROVIDER_VALIDATION_EXECUTABLE_SEQUENCE[index];
        if event.executable_relative_path != expected_path {
            return Err(StagexProviderError::Runtime(format!(
                "provider validation event order mismatch: expected {expected_path}, observed {}",
                event.executable_relative_path
            )));
        }
        let expected_authorization = PROVIDER_VALIDATION_AUTHORIZATION_SEQUENCE[index];
        if event.authorization_id != expected_authorization {
            return Err(StagexProviderError::Runtime(format!(
                "provider validation authorization mismatch: expected {expected_authorization}, observed {}",
                event.authorization_id
            )));
        }
    }
    assert_eq!(events.len(), PROVIDER_VALIDATION_EXECUTABLE_SEQUENCE.len());
    assert!(events.iter().all(|event| event.policy_decision == "allowed"));
    Ok(())
}

fn validate_provider_validation_report(report: &ProviderValidationReport) -> Result<(), StagexProviderError> {
    if report.schema != PROVIDER_VALIDATION_SCHEMA || report.status != TRANSITION_STATUS_COMPLETE {
        return Err(StagexProviderError::Receipt(
            "provider validation report has the wrong schema or status".to_string(),
        ));
    }
    validate_provider_validation_event_sequence(&report.events)?;
    let audit_digest = digest_serialized(PROVIDER_VALIDATION_AUDIT_DIGEST_DOMAIN, &report.events)?;
    if audit_digest != report.audit_digest_blake3 {
        return Err(StagexProviderError::Receipt("provider validation report audit digest mismatch".to_string()));
    }
    if !report.positive_tcc_compile || !report.negative_tcc_rejection || !report.negative_assembler_rejection {
        return Err(StagexProviderError::Receipt(
            "provider validation report lacks a required positive or negative result".to_string(),
        ));
    }
    if report.positive_binutils_execution_status != RUNTIME_SMOKE_EXPECTED_EXIT {
        return Err(StagexProviderError::Receipt(format!(
            "provider validation report has execution status {}; expected {RUNTIME_SMOKE_EXPECTED_EXIT}",
            report.positive_binutils_execution_status
        )));
    }
    if !report.fallback_events.is_empty() {
        return Err(StagexProviderError::Receipt("provider validation report contains fallback events".to_string()));
    }
    assert_eq!(audit_digest, report.audit_digest_blake3);
    assert!(report.fallback_events.is_empty());
    Ok(())
}

fn portable_validation_authorization_id(
    event: &ProtectedSeccompAuditEvent,
    relative_path: &str,
) -> Result<String, StagexProviderError> {
    let authorization_id = event
        .inventory_entry_id
        .as_deref()
        .ok_or_else(|| StagexProviderError::Runtime("provider validation event lacks authorization ID".to_string()))?;
    if authorization_id.starts_with("promoted:provider-relocation-smoke:") {
        return Ok(format!("promoted:provider-relocation-smoke:{relative_path}"));
    }
    Ok(authorization_id.to_string())
}

fn observe_provider_payload(staging: &Path) -> Result<Vec<ProviderPayloadObservation>, StagexProviderError> {
    let mut specifications = vec![
        ("stagex-selfhosted-tinycc".to_string(), PROVIDER_TCC_RELATIVE_PATH.to_string()),
        ("stagex-tinycc-runtime".to_string(), PROVIDER_TCC_RUNTIME_RELATIVE_PATH.to_string()),
        ("stagex-tinycc-patched-source".to_string(), PROVIDER_TCC_SOURCE_RELATIVE_PATH.to_string()),
        ("stagex-native-musl-headers".to_string(), PROVIDER_MUSL_INCLUDE_RELATIVE_PATH.to_string()),
        ("stagex-native-musl-libraries".to_string(), PROVIDER_MUSL_LIB_RELATIVE_PATH.to_string()),
        ("stagex-binutils-headers".to_string(), PROVIDER_BINUTILS_INCLUDE_RELATIVE_PATH.to_string()),
        ("stagex-binutils-ldscripts".to_string(), PROVIDER_BINUTILS_LD_RELATIVE_PATH.to_string()),
    ];
    for tool in BINUTILS_TOOL_NAMES {
        specifications.push((format!("stagex-binutils-{tool}"), format!("bin/{PROVIDER_TARGET}-{tool}")));
    }
    let mut observations = Vec::with_capacity(specifications.len());
    for (artifact_id, relative_path) in specifications {
        observations.push(observe_provider_payload_path(staging, artifact_id, relative_path)?);
    }
    observations.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    if observations.len() != PROVIDER_PAYLOAD_BASE_COUNT.saturating_add(BINUTILS_TOOL_NAMES.len()) {
        return Err(StagexProviderError::Provider("provider payload inventory count drifted".to_string()));
    }
    assert!(
        observations
            .windows(ADJACENT_PAIR_WINDOW_SIZE)
            .all(|pair| pair[0].artifact_id < pair[1].artifact_id)
    );
    assert!(observations.iter().all(|entry| !entry.digest_blake3.is_empty()));
    Ok(observations)
}

fn observe_provider_payload_path(
    staging: &Path,
    artifact_id: String,
    relative_path: String,
) -> Result<ProviderPayloadObservation, StagexProviderError> {
    let path = staging.join(&relative_path);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        StagexProviderError::Provider(format!("reading provider payload {}: {error}", path.display()))
    })?;
    if metadata.file_type().is_symlink() {
        return Err(StagexProviderError::Provider(format!("provider payload contains symlink: {}", path.display())));
    }
    let (bytes_len, digest_blake3) = observe_path_identity(&path)?;
    let kind = if metadata.is_file() {
        "file"
    } else if metadata.is_dir() {
        "directory"
    } else {
        return Err(StagexProviderError::Provider(format!("unsupported provider payload: {}", path.display())));
    };
    assert!(bytes_len > 0);
    assert!(!artifact_id.is_empty());
    Ok(ProviderPayloadObservation {
        artifact_id,
        relative_path,
        kind: kind.to_string(),
        bytes_len,
        digest_blake3,
    })
}

fn provider_metadata(
    evidence: &TransitionEvidence,
    payload: &[ProviderPayloadObservation],
    validation: &ProviderValidationReport,
    provider_validation_report_digest_blake3: &str,
    normalized_provider_digest_blake3: &str,
) -> ProviderMetadata {
    let retained_tools = std::iter::once(format!("{PROVIDER_TARGET}-tcc"))
        .chain(BINUTILS_TOOL_NAMES.iter().map(|tool| format!("{PROVIDER_TARGET}-{tool}")))
        .collect::<Vec<_>>();
    let provider_roles = provider_role_names();
    assert_eq!(provider_roles.len(), PROVIDER_ROLE_COUNT);
    assert_eq!(retained_tools.len(), BINUTILS_TOOL_NAMES.len().saturating_add(1));
    ProviderMetadata {
        schema: PROVIDER_SCHEMA.to_string(),
        provider_id: "stagex-lineage-intermediate-v1".to_string(),
        provider_kind: PROVIDER_KIND.to_string(),
        name: PROVIDER_NAME.to_string(),
        target: PROVIDER_TARGET.to_string(),
        compiler_kind: PROVIDER_COMPILER_KIND.to_string(),
        lineage_manifest_digest_blake3: evidence.manifest_digest_blake3.clone(),
        transition_plan_digest_blake3: evidence.plan_digest_blake3.clone(),
        transition_report_digest_blake3: evidence.report_digest_blake3.clone(),
        protected_exec_audit_digest_blake3: evidence.audit_digest_blake3.clone(),
        provider_validation_audit_digest_blake3: validation.audit_digest_blake3.clone(),
        provider_validation_report_digest_blake3: provider_validation_report_digest_blake3.to_string(),
        normalized_provider_digest_blake3: normalized_provider_digest_blake3.to_string(),
        payload: payload.to_vec(),
        retained_tools,
        provider_roles,
        notes: vec![
            "Intermediate protected StageX provider built from self-hosted TinyCC, native musl 1.1.24, and binutils 2.30.".to_string(),
            "This provider is not the final native GCC provider and does not claim compiler correctness or Mantle self-build completion.".to_string(),
            "Binutils were configured for x86_64-unknown-linux-gnu and are exposed under the StageX musl target prefix only for the validated static bootstrap boundary.".to_string(),
        ],
    }
}

fn provider_role_names() -> Vec<String> {
    vec![
        "target_prefixed_tools".to_string(),
        "headers".to_string(),
        "libraries".to_string(),
        "provider_metadata".to_string(),
    ]
}

fn observe_provider_roles(
    staging: &Path,
    payload: &[ProviderPayloadObservation],
) -> Result<Vec<ProviderRoleObservation>, StagexProviderError> {
    let payload_by_id = payload.iter().map(|item| (item.artifact_id.as_str(), item)).collect::<BTreeMap<_, _>>();
    let tool_ids = std::iter::once("stagex-selfhosted-tinycc".to_string())
        .chain(BINUTILS_TOOL_NAMES.iter().map(|tool| format!("stagex-binutils-{tool}")))
        .collect::<Vec<_>>();
    let header_ids = vec![
        "stagex-native-musl-headers".to_string(),
        "stagex-binutils-headers".to_string(),
    ];
    let library_ids = vec![
        "stagex-native-musl-libraries".to_string(),
        "stagex-tinycc-runtime".to_string(),
        "stagex-binutils-ldscripts".to_string(),
    ];
    let metadata_path = staging.join(PROVIDER_METADATA_RELATIVE_PATH);
    let metadata_digest = blake3_file_hex(&metadata_path)
        .map_err(|error| StagexProviderError::Provider(format!("hashing provider metadata: {error}")))?;
    let mut roles = vec![
        role_observation("target_prefixed_tools", &tool_ids, &payload_by_id)?,
        role_observation("headers", &header_ids, &payload_by_id)?,
        role_observation("libraries", &library_ids, &payload_by_id)?,
        ProviderRoleObservation {
            role: "provider_metadata".to_string(),
            artifact_ids: vec![PROVIDER_METADATA_RELATIVE_PATH.to_string()],
            digest_blake3: metadata_digest,
        },
    ];
    roles.sort_by(|left, right| left.role.cmp(&right.role));
    if roles.iter().map(|role| role.role.as_str()).collect::<BTreeSet<_>>().len() != PROVIDER_ROLE_COUNT {
        return Err(StagexProviderError::Provider("provider roles are incomplete or duplicated".to_string()));
    }
    assert_eq!(roles.len(), PROVIDER_ROLE_COUNT);
    assert!(roles.iter().all(|role| !role.artifact_ids.is_empty()));
    Ok(roles)
}

fn role_observation(
    role: &str,
    artifact_ids: &[String],
    payload: &BTreeMap<&str, &ProviderPayloadObservation>,
) -> Result<ProviderRoleObservation, StagexProviderError> {
    let mut observations = Vec::with_capacity(artifact_ids.len());
    for artifact_id in artifact_ids {
        let observation = payload.get(artifact_id.as_str()).ok_or_else(|| {
            StagexProviderError::Provider(format!("provider role {role} lacks artifact {artifact_id}"))
        })?;
        observations.push((*observation).clone());
    }
    let digest_blake3 = digest_serialized(PROVIDER_OUTPUT_DIGEST_DOMAIN, &observations)?;
    assert_eq!(observations.len(), artifact_ids.len());
    assert!(!observations.is_empty());
    Ok(ProviderRoleObservation {
        role: role.to_string(),
        artifact_ids: artifact_ids.to_vec(),
        digest_blake3,
    })
}

fn build_stage_reports(
    evidence: &TransitionEvidence,
    transition_root: &Path,
) -> Result<Vec<StagexStageReport>, StagexProviderError> {
    let output_digests = stage_output_digests(evidence, transition_root)?;
    let mut reports = Vec::with_capacity(evidence.plan.stages.len());
    for stage in &evidence.plan.stages {
        let prior = reports
            .iter()
            .map(|report: &StagexStageReport| (report.stage_id.as_str(), report))
            .collect::<BTreeMap<_, _>>();
        let predecessor_reports = stage
            .immediate_predecessor_stage_ids
            .iter()
            .map(|stage_id| predecessor_report(stage_id, &prior))
            .collect::<Result<Vec<_>, _>>()?;
        let executable_event_ids = bind_stage_authorization_ids(stage, &evidence.audit_events)?;
        let output_observations = stage
            .output_artifact_ids
            .iter()
            .map(|artifact_id| {
                output_digests
                    .get(artifact_id)
                    .cloned()
                    .map(|digest_blake3| StagexOutputObservation {
                        artifact_id: artifact_id.clone(),
                        digest_blake3,
                    })
                    .ok_or_else(|| StagexProviderError::Receipt(format!("missing output digest for {artifact_id}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        reports.push(StagexStageReport {
            stage_id: stage.id.clone(),
            stage_plan_digest_blake3: stagex_stage_plan_digest_blake3(stage)
                .map_err(|error| StagexProviderError::Receipt(error.to_string()))?,
            predecessor_reports,
            executable_event_ids,
            output_observations,
            status: STAGEX_STAGE_STATUS_COMPLETE.to_string(),
        });
    }
    assert_eq!(reports.len(), evidence.plan.stages.len());
    assert!(reports.iter().all(|report| report.status == STAGEX_STAGE_STATUS_COMPLETE));
    Ok(reports)
}

fn predecessor_report(
    stage_id: &str,
    prior: &BTreeMap<&str, &StagexStageReport>,
) -> Result<StagexPredecessorReport, StagexProviderError> {
    let report = prior.get(stage_id).ok_or_else(|| {
        StagexProviderError::Receipt(format!("predecessor report {stage_id} is not earlier in plan order"))
    })?;
    let report_digest_blake3 =
        stagex_stage_report_digest_blake3(report).map_err(|error| StagexProviderError::Receipt(error.to_string()))?;
    assert_eq!(report.stage_id, stage_id);
    assert!(!report_digest_blake3.is_empty());
    Ok(StagexPredecessorReport {
        stage_id: stage_id.to_string(),
        report_digest_blake3,
    })
}

fn bind_stage_authorization_ids(
    stage: &crunch_bootstrap_core::StagexStagePlan,
    events: &[ProtectedSeccompAuditEvent],
) -> Result<Vec<String>, StagexProviderError> {
    if events.is_empty() {
        return Err(StagexProviderError::Receipt(
            "stage authorization binding requires a non-empty protected audit".to_string(),
        ));
    }
    if stage.executable_authorizations.is_empty() {
        return Err(StagexProviderError::Receipt(format!(
            "protected stage {} has no executable authorization IDs",
            stage.id
        )));
    }
    let missing = stage
        .executable_authorizations
        .iter()
        .filter(|authorization| {
            !events.iter().any(|event| {
                is_allowed_protected_exec_event(event)
                    && event.resolved_host_path == Path::new(&authorization.absolute_path)
                    && event.digest_hex == authorization.digest_blake3
            })
        })
        .map(|authorization| authorization.id.as_str())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(StagexProviderError::Receipt(format!(
            "stage {} authorization identities lack protected audit observations: {}",
            stage.id,
            missing.join(", ")
        )));
    }
    let authorization_ids = stage
        .executable_authorizations
        .iter()
        .map(|authorization| authorization.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(authorization_ids.len(), stage.executable_authorizations.len());
    assert!(authorization_ids.iter().all(|id| !id.is_empty()));
    Ok(authorization_ids)
}

fn stage_output_digests(
    evidence: &TransitionEvidence,
    transition_root: &Path,
) -> Result<BTreeMap<String, String>, StagexProviderError> {
    let mut digests = evidence.manifest_outputs.clone();
    for output in evidence.observed_outputs.values() {
        match digests.get(&output.artifact_id) {
            Some(expected) if expected != &output.digest_blake3 => {
                return Err(StagexProviderError::Receipt(format!(
                    "observed output {} disagrees with lineage manifest",
                    output.artifact_id
                )));
            }
            _ => {}
        }
        digests.insert(output.artifact_id.clone(), output.digest_blake3.clone());
    }
    insert_transition_root_output_digests(&mut digests, evidence)?;
    insert_report_bound_output_digests(&mut digests, evidence)?;
    insert_binutils_output_digests(&mut digests, evidence, transition_root)?;
    let missing = evidence
        .plan
        .stages
        .iter()
        .flat_map(|stage| {
            stage
                .output_artifact_ids
                .iter()
                .filter(|artifact_id| !digests.contains_key(*artifact_id))
                .map(|artifact_id| format!("{}:{artifact_id}", stage.id))
        })
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(StagexProviderError::Receipt(format!(
            "stage outputs lack observed or manifest-bound identities: {}",
            missing.join(", ")
        )));
    }
    assert!(digests.len() >= evidence.manifest_outputs.len());
    assert!(digests.values().all(|digest| digest.len() == blake3::OUT_LEN * HEX_CHARS_PER_BYTE));
    Ok(digests)
}

fn insert_transition_root_output_digests(
    digests: &mut BTreeMap<String, String>,
    evidence: &TransitionEvidence,
) -> Result<(), StagexProviderError> {
    for (artifact_id, field) in [
        ("hex0-reproduced", "reproduced_hex0_digest_blake3"),
        ("kaem-0", "kaem_digest_blake3"),
        ("kaem-smoke-observation", "kaem_smoke_digest_blake3"),
    ] {
        let digest = evidence
            .report
            .get(field)
            .and_then(Value::as_str)
            .ok_or_else(|| StagexProviderError::Receipt(format!("transition report lacks {field}")))?;
        validate_blake3(digest, field)?;
        match digests.get(artifact_id) {
            Some(expected) if expected != digest => {
                return Err(StagexProviderError::Receipt(format!("transition root output {artifact_id} drifted")));
            }
            _ => {}
        }
        digests.insert(artifact_id.to_string(), digest.to_string());
    }
    assert!(digests.contains_key("hex0-reproduced"));
    assert!(digests.contains_key("kaem-0"));
    Ok(())
}

fn insert_report_bound_output_digests(
    digests: &mut BTreeMap<String, String>,
    evidence: &TransitionEvidence,
) -> Result<(), StagexProviderError> {
    digests.insert("stage0-hex0-copy".to_string(), crate::stagex_transition::HEX0_SEED_BLAKE3.to_string());
    for artifact_id in REPORT_BOUND_OUTPUT_IDS {
        let mut hasher = blake3::Hasher::new();
        hasher.update(STAGE_REPORT_OUTPUT_PROOF_DOMAIN);
        hash_length_prefixed(&mut hasher, artifact_id.as_bytes())?;
        hash_length_prefixed(&mut hasher, evidence.plan_digest_blake3.as_bytes())?;
        hash_length_prefixed(&mut hasher, evidence.report_digest_blake3.as_bytes())?;
        digests.insert((*artifact_id).to_string(), hasher.finalize().to_hex().to_string());
    }
    assert!(digests.contains_key("stage0-hex0-copy"));
    assert!(REPORT_BOUND_OUTPUT_IDS.iter().all(|id| digests.contains_key(*id)));
    Ok(())
}

fn insert_binutils_output_digests(
    digests: &mut BTreeMap<String, String>,
    evidence: &TransitionEvidence,
    transition_root: &Path,
) -> Result<(), StagexProviderError> {
    let runtime = transition_root.join(TRANSITION_BINUTILS_RUNTIME_RELATIVE_PATH);
    for (artifact_id, relative_path, expected) in [
        (
            "binutils-elf-symbol-canonicalizer",
            "stagex-elf-local-symbol-canonicalizer",
            crate::stagex_binutils::ELF_SYMBOL_CANONICALIZER_BLAKE3,
        ),
        (
            "binutils-sed-bridge-launcher",
            "stagex-sed-bridge-launcher",
            crate::stagex_binutils::SED_BRIDGE_LAUNCHER_BLAKE3,
        ),
        (
            "binutils-configure-utility",
            "stagex-configure-utility",
            crate::stagex_binutils::CONFIGURE_UTILITY_BLAKE3,
        ),
        (
            "binutils-ylwrap-runner",
            "stagex-ylwrap-sed-runner",
            crate::stagex_binutils::YLWRAP_SED_RUNNER_BLAKE3,
        ),
        ("binutils-bfd-chew", "source/bfd/doc/chew", crate::stagex_binutils::BINUTILS_BFD_CHEW_BLAKE3),
        (
            "binutils-positive-smoke-executable",
            "binutils-runtime-smoke/positive",
            crate::stagex_binutils::BINUTILS_POSITIVE_SMOKE_BLAKE3,
        ),
    ] {
        require_file_digest(&runtime.join(relative_path), expected, artifact_id)?;
        digests.insert(artifact_id.to_string(), expected.to_string());
    }
    let generated_executable_digest = validate_binutils_generated_executable_history(evidence, &runtime)?;
    digests.insert("binutils-configure-probe-executables".to_string(), generated_executable_digest);
    let generated_source_digest = digest_binutils_generated_sources(&runtime.join("source"))?;
    digests.insert("binutils-generated-sources".to_string(), generated_source_digest);
    let (_, runtime_smoke_digest) = observe_path_identity(&runtime.join("binutils-runtime-smoke"))?;
    digests.insert("binutils-runtime-smoke".to_string(), runtime_smoke_digest);
    assert!(PROJECTED_BINUTILS_OUTPUT_IDS.iter().all(|id| digests.contains_key(*id)));
    assert!(digests.contains_key("binutils-positive-smoke-executable"));
    Ok(())
}

fn validate_binutils_generated_executable_history(
    evidence: &TransitionEvidence,
    runtime: &Path,
) -> Result<String, StagexProviderError> {
    let mut facts = Vec::with_capacity(crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLES.len());
    for (label, relative_path, digest) in crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLES {
        let authorization_id = format!(
            "planned:{}:exec:binutils-generated:{label}",
            crate::stagex_transition::BINUTILS_GENERATOR_BUILD_STAGE_ID
        );
        let path = runtime.join(relative_path);
        let observed = evidence.audit_events.iter().any(|event| {
            event.inventory_entry_id.as_deref() == Some(authorization_id.as_str())
                && event.resolved_host_path == path
                && event.digest_hex == digest
                && event.policy_decision == "allowed"
        });
        if !observed {
            return Err(StagexProviderError::Receipt(format!(
                "binutils generated executable {label} lacks exact audit evidence"
            )));
        }
        facts.push((label, relative_path, digest));
    }
    assert_eq!(facts.len(), crate::stagex_binutils::BINUTILS_GENERATED_EXECUTABLE_COUNT);
    assert!(!facts.is_empty());
    digest_serialized(BINUTILS_GENERATED_EXECUTABLE_DIGEST_DOMAIN, &facts)
}

fn digest_binutils_generated_sources(source_root: &Path) -> Result<String, StagexProviderError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(BINUTILS_GENERATED_SOURCE_DIGEST_DOMAIN);
    for relative_path in crate::stagex_binutils::BINUTILS_GENERATED_SOURCE_OUTPUTS {
        let path = source_root.join(relative_path);
        let bytes = read_bounded_file(&path, PROVIDER_FILE_BYTES_MAX, "binutils generated source")?;
        hash_length_prefixed(&mut hasher, relative_path.as_bytes())?;
        hash_length_prefixed(&mut hasher, &bytes)?;
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(
        crate::stagex_binutils::BINUTILS_GENERATED_SOURCE_OUTPUTS.len(),
        BINUTILS_GENERATED_SOURCE_OUTPUT_COUNT
    );
    assert_eq!(digest.len(), blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
    Ok(digest)
}

fn require_file_digest(path: &Path, expected: &str, label: &str) -> Result<(), StagexProviderError> {
    let observed = blake3_file_hex(path)
        .map_err(|error| StagexProviderError::Receipt(format!("hashing {label} {}: {error}", path.display())))?;
    if observed != expected {
        return Err(StagexProviderError::Receipt(format!(
            "{label} digest mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert!(path.is_file());
    assert_eq!(observed, expected);
    Ok(())
}

struct FinalBundleInput<'a> {
    evidence: &'a TransitionEvidence,
    validation: &'a ProviderValidationReport,
    stage_reports: &'a [StagexStageReport],
    stage_graph_digest_blake3: &'a str,
    normalized_provider_digest_blake3: &'a str,
    output_digest_blake3: &'a str,
    provider_validation_report_digest_blake3: &'a str,
    provider_outputs: &'a [ProviderRoleObservation],
}

fn final_bundle_digest(input: FinalBundleInput<'_>) -> Result<String, StagexProviderError> {
    let stage_report_digests = input
        .stage_reports
        .iter()
        .map(stagex_stage_report_digest_blake3)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| StagexProviderError::Receipt(error.to_string()))?;
    let identity = FinalBundleIdentity {
        schema: STAGEX_RECEIPT_SCHEMA_V1,
        provider_kind: PROVIDER_KIND,
        plan_digest_blake3: &input.evidence.plan_digest_blake3,
        lineage_manifest_digest_blake3: &input.evidence.manifest_digest_blake3,
        stage_graph_digest_blake3: input.stage_graph_digest_blake3,
        source_state_digest_blake3: &input.evidence.plan.source_state_digest_blake3,
        normalized_provider_digest_blake3: input.normalized_provider_digest_blake3,
        output_digest_blake3: input.output_digest_blake3,
        protected_exec_audit_digest_blake3: &input.evidence.audit_digest_blake3,
        provider_validation_audit_digest_blake3: &input.validation.audit_digest_blake3,
        provider_validation_report_digest_blake3: input.provider_validation_report_digest_blake3,
        transition_report_digest_blake3: &input.evidence.report_digest_blake3,
        stage_report_digests,
        provider_outputs: input.provider_outputs,
        bounded_claim: PROVIDER_BOUNDED_CLAIM,
        stage_report_output_binding: STAGE_REPORT_OUTPUT_BINDING,
        executable_authorization_binding: EXECUTABLE_AUTHORIZATION_BINDING,
        non_claims: PROVIDER_NON_CLAIMS,
    };
    assert_eq!(identity.stage_report_digests.len(), input.stage_reports.len());
    assert!(!identity.stage_report_digests.is_empty());
    digest_serialized(FINAL_BUNDLE_DIGEST_DOMAIN, &identity)
}

fn build_provider_receipt(
    evidence: &TransitionEvidence,
    stage_reports: Vec<StagexStageReport>,
    provider_outputs: Vec<ProviderRoleObservation>,
    validation: &ProviderValidationReport,
    digests: &ProviderDigests,
) -> Result<StagexProviderReceipt, StagexProviderError> {
    let lineage = StagexLineageReceipt {
        schema_version: STAGEX_RECEIPT_SCHEMA_V1.to_string(),
        lineage_receipt_status: STAGEX_RECEIPT_STATUS_COMPLETE.to_string(),
        plan_digest_blake3: evidence.plan_digest_blake3.clone(),
        lineage_manifest_digest_blake3: evidence.manifest_digest_blake3.clone(),
        stage_graph_digest_blake3: digests.stage_graph_digest_blake3.clone(),
        source_state_digest_blake3: evidence.plan.source_state_digest_blake3.clone(),
        normalized_provider_digest_blake3: digests.normalized_provider_digest_blake3.clone(),
        output_digest_blake3: digests.output_digest_blake3.clone(),
        protected_exec_audit_digest_blake3: evidence.audit_digest_blake3.clone(),
        final_bundle_digest_blake3: digests.final_bundle_digest_blake3.clone(),
        fallback_events: Vec::new(),
        stage_reports,
    };
    let validation_result = validate_stagex_receipt(&evidence.plan, &lineage);
    if !validation_result.is_valid() {
        return Err(StagexProviderError::Receipt(format!(
            "core receipt validation failed: {}",
            validation_result.errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
        )));
    }
    assert_eq!(provider_outputs.len(), PROVIDER_ROLE_COUNT);
    assert!(lineage.fallback_events.is_empty());
    let mut receipt = StagexProviderReceipt {
        schema: STAGEX_RECEIPT_SCHEMA_V1.to_string(),
        provider_kind: PROVIDER_KIND.to_string(),
        audited_seed_digest: crate::stagex_transition::HEX0_SEED_BLAKE3.to_string(),
        lineage_manifest_digest: evidence.manifest_digest_blake3.clone(),
        stage_graph_digest: digests.stage_graph_digest_blake3.clone(),
        normalized_provider_digest: digests.normalized_provider_digest_blake3.clone(),
        transition_report_digest_blake3: evidence.report_digest_blake3.clone(),
        provider_validation_audit_digest_blake3: validation.audit_digest_blake3.clone(),
        provider_validation_report_digest_blake3: digests.provider_validation_report_digest_blake3.clone(),
        receipt_payload_digest_blake3: String::new(),
        provider_outputs,
        bounded_claim: PROVIDER_BOUNDED_CLAIM.to_string(),
        stage_report_output_binding: STAGE_REPORT_OUTPUT_BINDING.to_string(),
        executable_authorization_binding: EXECUTABLE_AUTHORIZATION_BINDING.to_string(),
        non_claims: PROVIDER_NON_CLAIMS.iter().map(ToString::to_string).collect(),
        lineage,
    };
    receipt.receipt_payload_digest_blake3 = compute_receipt_payload_digest(&receipt)?;
    assert!(!receipt.receipt_payload_digest_blake3.is_empty());
    Ok(receipt)
}

fn compute_receipt_payload_digest(receipt: &StagexProviderReceipt) -> Result<String, StagexProviderError> {
    let mut payload = receipt.clone();
    payload.receipt_payload_digest_blake3.clear();
    digest_serialized(PROVIDER_RECEIPT_PAYLOAD_DIGEST_DOMAIN, &payload)
}

pub(crate) fn compute_lineage_receipt_payload_digest(bytes: &[u8]) -> Result<String, String> {
    let receipt = serde_json::from_slice::<StagexProviderReceipt>(bytes)
        .map_err(|error| format!("parse StageX lineage provider receipt: {error}"))?;
    compute_receipt_payload_digest(&receipt).map_err(|error| error.to_string())
}

pub(crate) fn validate_lineage_receipt_payload_digest(bytes: &[u8]) -> Result<(), String> {
    let receipt = serde_json::from_slice::<StagexProviderReceipt>(bytes)
        .map_err(|error| format!("parse StageX lineage provider receipt: {error}"))?;
    let observed = receipt.receipt_payload_digest_blake3.clone();
    let expected = compute_receipt_payload_digest(&receipt).map_err(|error| error.to_string())?;
    if observed != expected {
        return Err(format!(
            "StageX lineage provider receipt payload digest mismatch: observed `{observed}`, expected `{expected}`"
        ));
    }
    assert_eq!(observed, expected);
    assert!(!observed.is_empty());
    Ok(())
}

fn independently_validate_published_provider(
    output: &Path,
    evidence: &TransitionEvidence,
) -> Result<(), StagexProviderError> {
    let receipt_bytes = read_bounded_file(
        &output.join(PROVIDER_RECEIPT_RELATIVE_PATH),
        JSON_REPORT_BYTES_MAX,
        "published provider receipt",
    )?;
    let receipt = serde_json::from_slice::<StagexProviderReceipt>(&receipt_bytes)
        .map_err(|error| StagexProviderError::Receipt(format!("parsing emitted provider receipt: {error}")))?;
    independently_validate_staging(output, evidence, &receipt)?;
    assert!(output.is_dir());
    assert_eq!(receipt.lineage.lineage_receipt_status, STAGEX_RECEIPT_STATUS_COMPLETE);
    Ok(())
}

fn independently_validate_staging(
    staging: &Path,
    evidence: &TransitionEvidence,
    expected_receipt: &StagexProviderReceipt,
) -> Result<(), StagexProviderError> {
    validate_static_provider_payload(staging, evidence)?;
    let payload = observe_provider_payload(staging)?;
    let normalized = digest_serialized(NORMALIZED_PROVIDER_DIGEST_DOMAIN, &payload)?;
    if normalized != expected_receipt.lineage.normalized_provider_digest_blake3 {
        return Err(StagexProviderError::Provider("independent provider payload digest mismatch".to_string()));
    }
    let metadata_bytes = read_bounded_file(
        &staging.join(PROVIDER_METADATA_RELATIVE_PATH),
        PROVIDER_FILE_BYTES_MAX,
        "provider metadata",
    )?;
    let metadata: ProviderMetadata = serde_json::from_slice(&metadata_bytes)
        .map_err(|error| StagexProviderError::Provider(format!("parsing provider metadata: {error}")))?;
    if metadata.normalized_provider_digest_blake3 != normalized
        || metadata.provider_roles != provider_role_names()
        || metadata.transition_report_digest_blake3 != expected_receipt.transition_report_digest_blake3
        || metadata.provider_validation_report_digest_blake3
            != expected_receipt.provider_validation_report_digest_blake3
    {
        return Err(StagexProviderError::Provider(
            "provider metadata does not bind payload, reports, and roles".to_string(),
        ));
    }
    let roles = observe_provider_roles(staging, &payload)?;
    let output_digest = digest_serialized(PROVIDER_OUTPUT_DIGEST_DOMAIN, &roles)?;
    if roles != expected_receipt.provider_outputs || output_digest != expected_receipt.lineage.output_digest_blake3 {
        return Err(StagexProviderError::Provider("independent provider role digest mismatch".to_string()));
    }
    independently_validate_receipt_file(staging, evidence, expected_receipt)?;
    assert_eq!(metadata.provider_kind, PROVIDER_KIND);
    assert_eq!(roles.len(), PROVIDER_ROLE_COUNT);
    Ok(())
}

fn independently_validate_receipt_file(
    staging: &Path,
    evidence: &TransitionEvidence,
    expected_receipt: &StagexProviderReceipt,
) -> Result<(), StagexProviderError> {
    let receipt_bytes =
        read_bounded_file(&staging.join(PROVIDER_RECEIPT_RELATIVE_PATH), JSON_REPORT_BYTES_MAX, "provider receipt")?;
    let receipt: StagexProviderReceipt = serde_json::from_slice(&receipt_bytes)
        .map_err(|error| StagexProviderError::Receipt(format!("parsing published provider receipt: {error}")))?;
    if &receipt != expected_receipt {
        return Err(StagexProviderError::Receipt(
            "published receipt bytes do not match constructed receipt".to_string(),
        ));
    }
    validate_lineage_receipt_payload_digest(&receipt_bytes).map_err(StagexProviderError::Receipt)?;
    let result = validate_stagex_receipt(&evidence.plan, &receipt.lineage);
    if !result.is_valid() || receipt.provider_kind != PROVIDER_KIND || !receipt.lineage.fallback_events.is_empty() {
        return Err(StagexProviderError::Receipt(format!(
            "independent receipt validation failed: {}",
            result.errors.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
        )));
    }
    let validation_bytes = read_bounded_file(
        &staging.join(PROVIDER_VALIDATION_RELATIVE_PATH),
        JSON_REPORT_BYTES_MAX,
        "provider validation report",
    )?;
    let validation: ProviderValidationReport = serde_json::from_slice(&validation_bytes)
        .map_err(|error| StagexProviderError::Receipt(format!("parsing provider validation report: {error}")))?;
    validate_provider_validation_report(&validation)?;
    let observed_audit_digest = digest_serialized(PROVIDER_VALIDATION_AUDIT_DIGEST_DOMAIN, &validation.events)?;
    let observed_report_digest = digest_serialized(PROVIDER_VALIDATION_REPORT_DIGEST_DOMAIN, &validation)?;
    if observed_audit_digest != receipt.provider_validation_audit_digest_blake3
        || observed_report_digest != receipt.provider_validation_report_digest_blake3
    {
        return Err(StagexProviderError::Receipt(
            "provider validation report or its audit digest is stale".to_string(),
        ));
    }
    validate_independent_receipt_identity(evidence, &receipt, &validation)?;
    assert_eq!(receipt.schema, STAGEX_RECEIPT_SCHEMA_V1);
    assert_eq!(receipt.lineage.lineage_receipt_status, STAGEX_RECEIPT_STATUS_COMPLETE);
    Ok(())
}

fn validate_independent_receipt_identity(
    evidence: &TransitionEvidence,
    receipt: &StagexProviderReceipt,
    validation: &ProviderValidationReport,
) -> Result<(), StagexProviderError> {
    let stage_graph_digest_blake3 = digest_serialized(STAGE_GRAPH_DIGEST_DOMAIN, &evidence.plan.stages)?;
    let final_bundle_digest_blake3 = final_bundle_digest(FinalBundleInput {
        evidence,
        validation,
        stage_reports: &receipt.lineage.stage_reports,
        stage_graph_digest_blake3: &stage_graph_digest_blake3,
        normalized_provider_digest_blake3: &receipt.lineage.normalized_provider_digest_blake3,
        output_digest_blake3: &receipt.lineage.output_digest_blake3,
        provider_validation_report_digest_blake3: &receipt.provider_validation_report_digest_blake3,
        provider_outputs: &receipt.provider_outputs,
    })?;
    require_receipt_field(
        &receipt.audited_seed_digest,
        crate::stagex_transition::HEX0_SEED_BLAKE3,
        "audited_seed_digest",
    )?;
    require_receipt_field(
        &receipt.lineage_manifest_digest,
        &receipt.lineage.lineage_manifest_digest_blake3,
        "lineage_manifest_digest",
    )?;
    require_receipt_field(
        &receipt.stage_graph_digest,
        &receipt.lineage.stage_graph_digest_blake3,
        "stage_graph_digest",
    )?;
    require_receipt_field(
        &receipt.normalized_provider_digest,
        &receipt.lineage.normalized_provider_digest_blake3,
        "normalized_provider_digest",
    )?;
    require_receipt_field(
        &receipt.transition_report_digest_blake3,
        &evidence.report_digest_blake3,
        "transition_report_digest_blake3",
    )?;
    require_receipt_field(
        &receipt.provider_validation_report_digest_blake3,
        &digest_serialized(PROVIDER_VALIDATION_REPORT_DIGEST_DOMAIN, validation)?,
        "provider_validation_report_digest_blake3",
    )?;
    require_receipt_field(&receipt.bounded_claim, PROVIDER_BOUNDED_CLAIM, "bounded_claim")?;
    require_receipt_field(
        &receipt.stage_report_output_binding,
        STAGE_REPORT_OUTPUT_BINDING,
        "stage_report_output_binding",
    )?;
    require_receipt_field(
        &receipt.executable_authorization_binding,
        EXECUTABLE_AUTHORIZATION_BINDING,
        "executable_authorization_binding",
    )?;
    require_receipt_field(
        &stage_graph_digest_blake3,
        &receipt.lineage.stage_graph_digest_blake3,
        "recomputed stage graph digest",
    )?;
    require_receipt_field(
        &final_bundle_digest_blake3,
        &receipt.lineage.final_bundle_digest_blake3,
        "recomputed final bundle digest",
    )?;
    let observed_non_claims = receipt.non_claims.iter().map(String::as_str).collect::<Vec<_>>();
    if observed_non_claims != PROVIDER_NON_CLAIMS {
        return Err(StagexProviderError::Receipt("provider receipt non-claims changed".to_string()));
    }
    assert_eq!(observed_non_claims.len(), PROVIDER_NON_CLAIMS.len());
    assert_eq!(final_bundle_digest_blake3, receipt.lineage.final_bundle_digest_blake3);
    Ok(())
}

fn require_receipt_field(actual: &str, expected: &str, field: &str) -> Result<(), StagexProviderError> {
    if actual != expected {
        return Err(StagexProviderError::Receipt(format!(
            "provider receipt {field} mismatch: expected {expected}, observed {actual}"
        )));
    }
    assert_eq!(actual, expected);
    assert!(!actual.is_empty());
    Ok(())
}

fn create_private_staging(path: &Path) -> Result<(), StagexProviderError> {
    if path.exists() {
        return Err(StagexProviderError::Publication(format!(
            "provider staging path already exists: {}",
            path.display()
        )));
    }
    fs::create_dir(path).map_err(|error| {
        StagexProviderError::Publication(format!("creating provider staging {}: {error}", path.display()))
    })?;
    fs::set_permissions(path, fs::Permissions::from_mode(PROVIDER_PRIVATE_DIRECTORY_MODE)).map_err(|error| {
        StagexProviderError::Publication(format!("setting provider staging mode {}: {error}", path.display()))
    })?;
    assert!(path.is_dir());
    assert_eq!(
        fs::metadata(path).map(|metadata| metadata.permissions().mode() & FILE_MODE_MASK).ok(),
        Some(PROVIDER_PRIVATE_DIRECTORY_MODE)
    );
    Ok(())
}

fn provider_staging_path(output: &Path) -> Result<PathBuf, StagexProviderError> {
    let file_name = output.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        StagexProviderError::InvalidInput(format!("provider output name is not UTF-8: {}", output.display()))
    })?;
    let parent = output.parent().ok_or_else(|| {
        StagexProviderError::InvalidInput(format!("provider output has no parent: {}", output.display()))
    })?;
    let staging = parent.join(format!("{file_name}{PROVIDER_STAGE_SUFFIX}"));
    assert_ne!(staging, output);
    assert_eq!(staging.parent(), output.parent());
    Ok(staging)
}

fn publish_no_replace(staging: &Path, output: &Path) -> Result<(), StagexProviderError> {
    crate::linux_rename::rename_path_no_replace(staging, output).map_err(|error| {
        StagexProviderError::Publication(format!(
            "atomically publishing provider {} -> {} without replacement: {error}",
            staging.display(),
            output.display()
        ))
    })?;
    assert!(!staging.exists());
    assert!(output.is_dir());
    Ok(())
}

fn remove_failed_staging(path: &Path) {
    if path.is_dir() {
        let _ = fs::remove_dir_all(path);
    }
}

fn copy_tree_create_new(source: &Path, destination: &Path) -> Result<(), StagexProviderError> {
    require_absolute_directory(source, "provider source tree")?;
    if destination.exists() {
        return Err(StagexProviderError::Provider(format!(
            "create-new provider tree destination exists: {}",
            destination.display()
        )));
    }
    reject_tree_symlinks(source)?;
    crate::release_tree_copy::copy_directory_tree(source, destination)
        .map_err(|error| StagexProviderError::Provider(format!("copying provider tree: {error}")))?;
    reject_tree_symlinks(destination)?;
    let source_identity = observe_path_identity(source)?;
    let destination_identity = observe_path_identity(destination)?;
    if source_identity != destination_identity {
        return Err(StagexProviderError::Provider(format!(
            "provider tree copy identity mismatch: {} -> {}",
            source.display(),
            destination.display()
        )));
    }
    assert!(destination.is_dir());
    assert_eq!(source_identity, destination_identity);
    Ok(())
}

fn copy_file_create_new(source: &Path, destination: &Path, executable: bool) -> Result<(), StagexProviderError> {
    require_absolute_regular_file(source, "provider source file")?;
    let bytes = read_bounded_file(source, PROVIDER_FILE_BYTES_MAX, "provider source file")?;
    let parent = destination.parent().ok_or_else(|| {
        StagexProviderError::Provider(format!("provider destination has no parent: {}", destination.display()))
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| StagexProviderError::Provider(format!("creating provider destination parent: {error}")))?;
    let mode = if executable {
        PROVIDER_EXECUTABLE_MODE
    } else {
        PROVIDER_DATA_MODE
    };
    write_file_create_new(destination, &bytes, mode)?;
    let source_digest = blake3::hash(&bytes).to_hex().to_string();
    let destination_digest = blake3_file_hex(destination)
        .map_err(|error| StagexProviderError::Provider(format!("hashing copied provider file: {error}")))?;
    if source_digest != destination_digest {
        return Err(StagexProviderError::Provider(format!("provider file copy changed: {}", destination.display())));
    }
    assert_eq!(source_digest, destination_digest);
    assert!(destination.is_file());
    Ok(())
}

fn write_file_create_new(path: &Path, bytes: &[u8], mode: u32) -> Result<(), StagexProviderError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| StagexProviderError::Io(format!("creating {}: {error}", parent.display())))?;
    }
    let mut file = File::options()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| StagexProviderError::Io(format!("creating {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| StagexProviderError::Io(format!("writing {}: {error}", path.display())))?;
    file.sync_all()
        .map_err(|error| StagexProviderError::Io(format!("syncing {}: {error}", path.display())))?;
    file.set_permissions(fs::Permissions::from_mode(mode))
        .map_err(|error| StagexProviderError::Io(format!("setting mode on {}: {error}", path.display())))?;
    assert!(path.is_file());
    assert_eq!(fs::metadata(path).map(|metadata| metadata.len()).ok(), u64::try_from(bytes.len()).ok());
    Ok(())
}

fn write_json_create_new(path: &Path, value: &impl Serialize) -> Result<(), StagexProviderError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| StagexProviderError::Io(format!("serializing {}: {error}", path.display())))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > JSON_REPORT_BYTES_MAX {
        return Err(StagexProviderError::Io(format!("JSON output exceeds bound: {}", path.display())));
    }
    write_file_create_new(path, &bytes, PROVIDER_DATA_MODE)?;
    assert!(!bytes.is_empty());
    assert!(path.is_file());
    Ok(())
}

fn observe_path_identity(path: &Path) -> Result<(u64, String), StagexProviderError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| StagexProviderError::Io(format!("reading identity path {}: {error}", path.display())))?;
    if metadata.file_type().is_symlink() {
        return Err(StagexProviderError::Provider(format!("identity path is a symlink: {}", path.display())));
    }
    if metadata.is_file() {
        let bytes = read_bounded_file(path, PROVIDER_FILE_BYTES_MAX, "identity file")?;
        return Ok((u64::try_from(bytes.len()).unwrap_or(u64::MAX), blake3::hash(&bytes).to_hex().to_string()));
    }
    if metadata.is_dir() {
        return crate::release_tree_copy::hash_directory_tree(path).map_err(|error| {
            StagexProviderError::Provider(format!("hashing identity tree {}: {error}", path.display()))
        });
    }
    Err(StagexProviderError::Provider(format!("identity path has unsupported type: {}", path.display())))
}

fn reject_tree_symlinks(root: &Path) -> Result<(), StagexProviderError> {
    let mut pending = vec![root.to_path_buf()];
    let mut count = 0_usize;
    while let Some(path) = pending.pop() {
        count = count.saturating_add(1);
        if count > crunch_release_core::TreeCopyLimits::RELEASE_BUNDLE.entries_count_max as usize {
            return Err(StagexProviderError::Provider("provider tree exceeds entry bound".to_string()));
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            StagexProviderError::Provider(format!("reading provider tree {}: {error}", path.display()))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(StagexProviderError::Provider(format!("provider tree contains symlink: {}", path.display())));
        }
        if metadata.is_dir() {
            let mut entries = fs::read_dir(&path)
                .map_err(|error| {
                    StagexProviderError::Provider(format!("reading provider tree {}: {error}", path.display()))
                })?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| StagexProviderError::Provider(format!("reading provider tree entry: {error}")))?;
            entries.sort_by_key(fs::DirEntry::file_name);
            pending.extend(entries.into_iter().rev().map(|entry| entry.path()));
        } else if !metadata.is_file() {
            return Err(StagexProviderError::Provider(format!(
                "provider tree has unsupported entry: {}",
                path.display()
            )));
        }
    }
    assert!(count > 0);
    assert!(pending.is_empty());
    Ok(())
}

fn sorted_child_names(root: &Path) -> Result<Vec<String>, StagexProviderError> {
    let mut names = fs::read_dir(root)
        .map_err(|error| StagexProviderError::Provider(format!("reading {}: {error}", root.display())))?
        .map(|entry| {
            entry
                .map_err(|error| StagexProviderError::Provider(format!("reading {} entry: {error}", root.display())))?
                .file_name()
                .into_string()
                .map_err(|_| StagexProviderError::Provider(format!("non-UTF-8 child in {}", root.display())))
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    assert!(names.windows(ADJACENT_PAIR_WINDOW_SIZE).all(|pair| pair[0] < pair[1]));
    assert!(names.iter().all(|name| !name.is_empty()));
    Ok(names)
}

fn read_bounded_file(path: &Path, bytes_max: u64, label: &str) -> Result<Vec<u8>, StagexProviderError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| StagexProviderError::Io(format!("reading {label} metadata {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > bytes_max {
        return Err(StagexProviderError::InvalidInput(format!(
            "{label} must be a no-follow regular file of at most {bytes_max} bytes: {}",
            path.display()
        )));
    }
    let bytes = fs::read(path)
        .map_err(|error| StagexProviderError::Io(format!("reading {label} {}: {error}", path.display())))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) != metadata.len() {
        return Err(StagexProviderError::Io(format!("{label} changed while reading: {}", path.display())));
    }
    assert!(metadata.len() <= bytes_max);
    assert_eq!(u64::try_from(bytes.len()).ok(), Some(metadata.len()));
    Ok(bytes)
}

fn require_absolute_regular_file(path: &Path, label: &str) -> Result<(), StagexProviderError> {
    if !path.is_absolute() {
        return Err(StagexProviderError::InvalidInput(format!("{label} path is relative: {}", path.display())));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| StagexProviderError::InvalidInput(format!("reading {label} {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(StagexProviderError::InvalidInput(format!(
            "{label} is not a no-follow regular file: {}",
            path.display()
        )));
    }
    assert!(path.is_absolute());
    assert!(metadata.is_file());
    Ok(())
}

fn require_absolute_directory(path: &Path, label: &str) -> Result<(), StagexProviderError> {
    if !path.is_absolute() {
        return Err(StagexProviderError::InvalidInput(format!("{label} path is relative: {}", path.display())));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| StagexProviderError::InvalidInput(format!("reading {label} {}: {error}", path.display())))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(StagexProviderError::InvalidInput(format!(
            "{label} is not a no-follow directory: {}",
            path.display()
        )));
    }
    assert!(path.is_absolute());
    assert!(metadata.is_dir());
    Ok(())
}

fn require_json_string(value: &Value, field: &str, expected: &str) -> Result<(), StagexProviderError> {
    let observed = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| StagexProviderError::Transition(format!("transition report lacks string field {field}")))?;
    if observed != expected {
        return Err(StagexProviderError::Transition(format!(
            "transition report {field} mismatch: expected {expected}, observed {observed}"
        )));
    }
    assert_eq!(observed, expected);
    assert!(!observed.is_empty());
    Ok(())
}

fn validate_blake3(value: &str, label: &str) -> Result<(), StagexProviderError> {
    let valid = value.len() == blake3::OUT_LEN * HEX_CHARS_PER_BYTE
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(StagexProviderError::Transition(format!("{label} is not lowercase BLAKE3: {value}")));
    }
    assert_eq!(value.len(), blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
    assert!(valid);
    Ok(())
}

fn digest_serialized(domain: &[u8], value: &impl Serialize) -> Result<String, StagexProviderError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| StagexProviderError::Receipt(format!("serializing digest input: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hash_length_prefixed(&mut hasher, &bytes)?;
    let digest = hasher.finalize().to_hex().to_string();
    assert!(!domain.is_empty());
    assert_eq!(digest.len(), blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
    Ok(digest)
}

fn hash_length_prefixed(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), StagexProviderError> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| StagexProviderError::Receipt("digest input length exceeds u64".to_string()))?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    assert_eq!(length, u64::try_from(bytes.len()).unwrap_or(u64::MAX));
    assert_eq!(length.to_le_bytes().len(), std::mem::size_of::<u64>());
    Ok(())
}

fn utf8_path<'a>(path: &'a Path, label: &str) -> Result<&'a str, StagexProviderError> {
    let value = path
        .to_str()
        .ok_or_else(|| StagexProviderError::InvalidInput(format!("{label} path is not UTF-8: {}", path.display())))?;
    if value.is_empty() {
        return Err(StagexProviderError::InvalidInput(format!("{label} path is empty")));
    }
    assert!(!value.is_empty());
    assert_eq!(Path::new(value), path);
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const NON_UTF8_FIXTURE_BYTE: u8 = 0xff;

    fn validation_event(path: &str, authorization_id: &str) -> ProviderValidationEvent {
        ProviderValidationEvent {
            executable_relative_path: path.to_string(),
            digest_blake3: DIGEST_A.to_string(),
            authorization_id: authorization_id.to_string(),
            policy_decision: "allowed".to_string(),
        }
    }

    fn complete_validation_report() -> ProviderValidationReport {
        let events = PROVIDER_VALIDATION_EXECUTABLE_SEQUENCE
            .iter()
            .zip(PROVIDER_VALIDATION_AUTHORIZATION_SEQUENCE)
            .map(|(path, authorization_id)| validation_event(path, authorization_id))
            .collect::<Vec<_>>();
        let audit_digest_blake3 = digest_serialized(PROVIDER_VALIDATION_AUDIT_DIGEST_DOMAIN, &events).unwrap();
        assert_eq!(events.len(), PROVIDER_VALIDATION_EVENT_COUNT_EXPECTED);
        assert!(!audit_digest_blake3.is_empty());
        ProviderValidationReport {
            schema: PROVIDER_VALIDATION_SCHEMA.to_string(),
            status: TRANSITION_STATUS_COMPLETE.to_string(),
            audit_digest_blake3,
            events,
            positive_tcc_compile: true,
            negative_tcc_rejection: true,
            positive_binutils_execution_status: RUNTIME_SMOKE_EXPECTED_EXIT,
            negative_assembler_rejection: true,
            fallback_events: Vec::new(),
        }
    }

    #[test]
    fn runtime_smoke_results_accept_only_the_complete_boundary() {
        assert!(validate_runtime_smoke_results(RUNTIME_SMOKE_EXPECTED_EXIT, true, true).is_ok());
        assert!(validate_runtime_smoke_results(RUNTIME_SMOKE_EXPECTED_EXIT + 1, true, true).is_err());
        assert!(validate_runtime_smoke_results(RUNTIME_SMOKE_EXPECTED_EXIT, false, true).is_err());
        assert!(validate_runtime_smoke_results(RUNTIME_SMOKE_EXPECTED_EXIT, true, false).is_err());
    }

    #[test]
    fn provider_validation_report_accepts_complete_results() {
        let report = complete_validation_report();
        assert!(validate_provider_validation_report(&report).is_ok());
        assert!(report.fallback_events.is_empty());
    }

    #[test]
    fn provider_validation_report_rejects_false_negative_result() {
        let mut report = complete_validation_report();
        report.negative_tcc_rejection = false;
        let error = validate_provider_validation_report(&report).unwrap_err();
        assert!(matches!(error, StagexProviderError::Receipt(_)));
        assert!(error.to_string().contains("lacks a required"));
    }

    #[test]
    fn provider_role_names_are_exact_and_complete() {
        let roles = provider_role_names();
        let unique = roles.iter().collect::<BTreeSet<_>>();
        assert_eq!(roles.len(), PROVIDER_ROLE_COUNT);
        assert_eq!(unique.len(), PROVIDER_ROLE_COUNT);
        assert_eq!(roles, vec!["target_prefixed_tools", "headers", "libraries", "provider_metadata"]);
        assert!(!roles.contains(&"retained_tool_metadata".to_string()));
    }

    #[test]
    fn provider_staging_path_is_deterministic_and_separate() {
        let output = Path::new("/tmp/provider");
        let first = provider_staging_path(output).unwrap();
        let second = provider_staging_path(output).unwrap();
        assert_eq!(first, second);
        assert_ne!(first, output);
        assert_eq!(first, Path::new("/tmp/provider.stagex-provider-staging"));
        assert_eq!(first.parent(), output.parent());
    }

    #[test]
    fn provider_staging_path_rejects_non_utf8_name() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt as _;

        let output = PathBuf::from("/tmp").join(OsString::from_vec(vec![NON_UTF8_FIXTURE_BYTE]));
        let error = provider_staging_path(&output).unwrap_err();
        assert!(matches!(error, StagexProviderError::InvalidInput(_)));
        assert!(error.to_string().contains("not UTF-8"));
    }

    #[test]
    fn payload_observation_rejects_symlink() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("target"), b"payload").unwrap();
        symlink("target", temp.path().join("link")).unwrap();
        let error = observe_provider_payload_path(temp.path(), "link".to_string(), "link".to_string()).unwrap_err();
        assert!(matches!(error, StagexProviderError::Provider(_)));
        assert!(error.to_string().contains("symlink"));
    }

    #[test]
    fn native_library_set_rejects_missing_library() {
        let temp = tempfile::tempdir().unwrap();
        for name in NATIVE_LIBRARY_NAMES.iter().skip(1) {
            fs::write(temp.path().join(name), b"archive").unwrap();
        }
        let error = validate_native_library_set(temp.path()).unwrap_err();
        assert!(matches!(error, StagexProviderError::Provider(_)));
        assert!(error.to_string().contains("library set mismatch"));
    }

    #[test]
    fn provider_role_observation_rejects_missing_artifact() {
        let payload = BTreeMap::new();
        let error = role_observation("headers", &["missing".to_string()], &payload).unwrap_err();
        assert!(matches!(error, StagexProviderError::Provider(_)));
        assert!(error.to_string().contains("lacks artifact"));
    }

    #[test]
    fn digest_validation_accepts_blake3_and_rejects_placeholder() {
        assert!(validate_blake3(DIGEST_A, "test digest").is_ok());
        let error = validate_blake3("placeholder", "test digest").unwrap_err();
        assert!(matches!(error, StagexProviderError::Transition(_)));
        assert!(error.to_string().contains("not lowercase BLAKE3"));
    }

    #[test]
    fn report_fallback_scan_rejects_nested_event() {
        let report = serde_json::json!({"stage": {"fallback_events": ["ambient-shell"]}});
        let error = reject_report_fallback_events(&report).unwrap_err();
        assert!(matches!(error, StagexProviderError::Transition(_)));
        assert!(error.to_string().contains("fallback"));
    }

    #[test]
    fn static_tool_validation_rejects_non_elf_executable() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("tool");
        fs::write(&path, b"#!/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(PROVIDER_EXECUTABLE_MODE)).unwrap();
        let error = require_regular_executable_elf(&path).unwrap_err();
        assert!(matches!(error, StagexProviderError::Provider(_)));
        assert!(error.to_string().contains("not ELF"));
    }

    #[test]
    fn projected_binutils_outputs_are_exact() {
        assert_eq!(PROJECTED_BINUTILS_OUTPUT_IDS.len(), PROJECTED_BINUTILS_OUTPUT_COUNT);
        assert!(PROJECTED_BINUTILS_OUTPUT_IDS.contains(&"binutils-generated-sources"));
        assert!(!PROJECTED_BINUTILS_OUTPUT_IDS.contains(&"binutils-installed-as"));
    }

    #[test]
    fn request_validation_rejects_existing_destination() {
        let temp = tempfile::tempdir().unwrap();
        let manifest = temp.path().join("lineage.json");
        let transition_root = temp.path().join("transition");
        let output = temp.path().join("provider");
        fs::write(&manifest, b"{}").unwrap();
        fs::create_dir(&transition_root).unwrap();
        fs::create_dir(&output).unwrap();
        let error = validate_request_paths(&StagexProviderRequest {
            lineage_manifest_path: &manifest,
            transition_root: &transition_root,
            output_path: &output,
        })
        .unwrap_err();
        assert!(matches!(error, StagexProviderError::InvalidInput(_)));
        assert!(error.to_string().contains("absent absolute path"));
    }

    #[test]
    fn stage_authorization_binding_requires_an_observed_path_digest_identity() {
        let authorization = crunch_bootstrap_core::StagexExecutableAuthorization {
            id: "exec:test:tool".to_string(),
            absolute_path: "/tmp/stagex-tool".to_string(),
            role: "compiler".to_string(),
            source_stage_id: "predecessor".to_string(),
            digest_blake3: DIGEST_A.to_string(),
        };
        let stage = crunch_bootstrap_core::StagexStagePlan {
            id: "consumer".to_string(),
            immediate_predecessor_stage_ids: vec!["predecessor".to_string()],
            source_artifact_ids: vec!["source".to_string()],
            input_artifact_ids: vec!["input".to_string()],
            output_artifact_ids: vec!["output".to_string()],
            executable_authorizations: vec![authorization],
            timeout_ms_max: 1,
            output_bytes_max: 1,
            parallel_job_count_max: 1,
        };
        let mut event = sample_allowed_event("other-equivalent-id", "/tmp/stagex-tool", DIGEST_A);
        assert_eq!(bind_stage_authorization_ids(&stage, &[event.clone()]).unwrap(), vec!["exec:test:tool"]);
        event.digest_hex = DIGEST_B.to_string();
        let error = bind_stage_authorization_ids(&stage, &[event.clone()]).unwrap_err();
        assert!(matches!(error, StagexProviderError::Receipt(_)));
        assert!(error.to_string().contains("lack protected audit observations"));
        event.digest_hex = DIGEST_A.to_string();
        event.policy_decision = "denied".to_string();
        assert!(bind_stage_authorization_ids(&stage, &[event.clone()]).is_err());
        event.policy_decision = "allowed".to_string();
        event.syscall = "clone3".to_string();
        assert!(bind_stage_authorization_ids(&stage, &[event]).is_err());
    }

    fn sample_allowed_event(authorization_id: &str, path: &str, digest: &str) -> ProtectedSeccompAuditEvent {
        ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from(path),
            tracee_path: PathBuf::from(path),
            resolved_host_path: PathBuf::from(path),
            digest_hex: digest.to_string(),
            reason: "test".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some(authorization_id.to_string()),
            policy_decision: "allowed".to_string(),
        }
    }

    #[test]
    fn promoted_validation_authorization_is_path_independent() {
        let event = ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from("/tmp/staging/.validation/smoke"),
            tracee_path: PathBuf::from("/tmp/staging/.validation/smoke"),
            resolved_host_path: PathBuf::from("/tmp/staging/.validation/smoke"),
            digest_hex: DIGEST_A.to_string(),
            reason: "allowed".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some("promoted:provider-relocation-smoke:/tmp/staging/.validation/smoke".to_string()),
            policy_decision: "allowed".to_string(),
        };
        let projected = portable_validation_authorization_id(&event, ".validation/smoke").unwrap();
        assert_eq!(projected, "promoted:provider-relocation-smoke:.validation/smoke");
        assert!(!projected.contains("/tmp/staging"));
    }

    #[test]
    fn report_bound_outputs_do_not_include_provider_component_artifacts() {
        assert!(REPORT_BOUND_OUTPUT_IDS.contains(&"stage0-full-sha256-verification"));
        assert!(!REPORT_BOUND_OUTPUT_IDS.contains(&"tcc-musl-selfhost"));
        assert!(!REPORT_BOUND_OUTPUT_IDS.contains(&"musl-native-libc"));
    }

    #[test]
    fn provider_role_authority_rejects_missing_transition_observation() {
        let bytes = include_bytes!("../bootstrap/stagex-transition-lineage.json");
        let manifest = serde_json::from_slice::<crunch_bootstrap_core::LineageManifest>(bytes).unwrap();
        let value = serde_json::from_slice::<Value>(bytes).unwrap();
        let manifest_outputs = manifest_output_digests(&value).unwrap();
        let error = validate_manifest_provider_authority(&manifest, &BTreeMap::new(), &manifest_outputs).unwrap_err();
        assert!(matches!(error, StagexProviderError::Transition(_)));
        assert!(error.to_string().contains("lacks a transition observation"));
    }

    #[test]
    fn provider_validation_sequence_accepts_closed_trace() {
        let events = PROVIDER_VALIDATION_EXECUTABLE_SEQUENCE
            .iter()
            .zip(PROVIDER_VALIDATION_AUTHORIZATION_SEQUENCE)
            .map(|(path, authorization_id)| validation_event(path, authorization_id))
            .collect::<Vec<_>>();
        assert!(validate_provider_validation_event_sequence(&events).is_ok());
        assert_eq!(events.len(), PROVIDER_VALIDATION_EVENT_COUNT_EXPECTED);
    }

    #[test]
    fn provider_validation_sequence_rejects_reordered_trace() {
        let mut events = PROVIDER_VALIDATION_EXECUTABLE_SEQUENCE
            .iter()
            .zip(PROVIDER_VALIDATION_AUTHORIZATION_SEQUENCE)
            .map(|(path, authorization_id)| validation_event(path, authorization_id))
            .collect::<Vec<_>>();
        events.swap(0, 1);
        let error = validate_provider_validation_event_sequence(&events).unwrap_err();
        assert!(matches!(error, StagexProviderError::Runtime(_)));
        assert!(error.to_string().contains("order mismatch"));
    }

    #[test]
    fn provider_validation_sequence_rejects_substituted_authorization() {
        let mut events = PROVIDER_VALIDATION_EXECUTABLE_SEQUENCE
            .iter()
            .zip(PROVIDER_VALIDATION_AUTHORIZATION_SEQUENCE)
            .map(|(path, authorization_id)| validation_event(path, authorization_id))
            .collect::<Vec<_>>();
        events[0].authorization_id = "planned:substituted".to_string();
        let error = validate_provider_validation_event_sequence(&events).unwrap_err();
        assert!(matches!(error, StagexProviderError::Runtime(_)));
        assert!(error.to_string().contains("authorization mismatch"));
    }

    #[test]
    fn private_staging_rejects_partial_existing_directory() {
        let temp = tempfile::tempdir().unwrap();
        let staging = temp.path().join("provider.stagex-provider-staging");
        fs::create_dir(&staging).unwrap();
        let error = create_private_staging(&staging).unwrap_err();
        assert!(matches!(error, StagexProviderError::Publication(_)));
        assert!(error.to_string().contains("already exists"));
    }

    #[test]
    fn receipt_field_check_accepts_match_and_rejects_substitution() {
        assert!(require_receipt_field(DIGEST_A, DIGEST_A, "digest").is_ok());
        let error = require_receipt_field(DIGEST_A, "substituted", "digest").unwrap_err();
        assert!(matches!(error, StagexProviderError::Receipt(_)));
        assert!(error.to_string().contains("mismatch"));
    }
}
