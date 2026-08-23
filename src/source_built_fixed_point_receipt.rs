use std::collections::BTreeSet;
use std::fs;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;

use crunch_release_core::BuildEffect;
use crunch_release_core::ContentBoundRebuildDescriptor;
use crunch_release_core::DeterministicBuildProofReceipt;
use crunch_release_core::DeterministicBuildProofReceiptInit;
use crunch_release_core::DeterministicBuildProofVerdict;
use crunch_release_core::DeterministicBuildRunReceipt;
use crunch_release_core::DeterministicOutputDigest;
use crunch_release_core::DeterministicProofUnit;
use crunch_release_core::RebuildAuthorityPlan;
use crunch_release_core::RebuildContentIdentity;
use crunch_release_core::RebuildContentKind;
use crunch_release_core::RebuildInputRole;
use crunch_release_core::RebuildPolicyIdentities;
use crunch_release_core::RebuildRunRootIdentity;
use crunch_release_core::content_bound_rebuild_descriptor_digest_blake3;
use crunch_release_core::deterministic_build_proof_receipt_digest_blake3;
use crunch_release_core::rebuild_arguments_digest_blake3;
use crunch_release_core::rebuild_authority_plan_digest_blake3;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::ProofOutputRole;
use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_REQUIRED_SOURCE_ROLE_COUNT;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_shell::ConstructedProviders;
use crate::source_built_fixed_point_shell::STAGEX_TRANSITION_AUDIT_FILE;
use crate::source_built_fixed_point_shell::STAGEX_TRANSITION_REPORT_FILE;

const EXTENSION_FIELD: &str = "source_built_fixed_point";
const EXTENSION_SCHEMA: &str = "mantle-source-built-fixed-point-receipt-extension-v1";
const STAGE_EVIDENCE_FILE: &str = "source-built-stage-evidence.json";
const FIXED_POINT_META_RELATIVE_PATH: &str = "cargo-free-fixed-point/meta.json";
const FIXED_POINT_STAGE1_RECEIPT: &str = "cargo-free-fixed-point/stage1/receipt.json";
const FIXED_POINT_STAGE2_RECEIPT: &str = "cargo-free-fixed-point/stage2/receipt.json";
const FIXED_POINT_STAGE1_STDERR: &str = "cargo-free-fixed-point/stage1/stderr.txt";
const FIXED_POINT_STAGE2_STDERR: &str = "cargo-free-fixed-point/stage2/stderr.txt";
const FIXED_POINT_STAGE1_BINARY: &str = "cargo-free-fixed-point/stage1/mantle";
const FIXED_POINT_STAGE2_BINARY: &str = "cargo-free-fixed-point/stage2/mantle";
const RUST_PROVIDER_BUILD_RECEIPT: &str = "rust-provider/share/mantle-rust-provider/receipts/build.json";
const RUST_PROVIDER_BINDING_RECEIPT: &str =
    "rust-provider/share/mantle-rust-provider/receipts/full-source-binding.json";
const RUST_PROVIDER_RUSTC: &str = "rust-provider/bin/rustc";
const TOOLCHAIN_CLOSURE_FILE: &str = "source-built-toolchain-closure.json";
const PLAN_FILE: &str = "source-built-fixed-point-plan.json";
const PROVIDER_LINKAGE_SCHEMA: &str = "mantle-self-build-provider-kind-linkage-v1";
const RELEASE_INPUTS_SCHEMA: &str = "mantle-source-built-fixed-point-release-inputs-v1";
const PROOF_PROVIDER_KIND: &str = "full-source";
const HERMETICITY_MODE: &str = "strict";
const PHYSICAL_STORE_ISOLATION: &str = "clean-namespace-per-run";
const PROOF_TARGET_IDENTITY: &str = "mantle-source-built-fixed-point";
const MANTLE_OUTPUT_NAME: &str = "mantle";
const STAGE1_RUN_ID: &str = "stage1";
const STAGE2_RUN_ID: &str = "stage2";
const STAGE1_PERTURBATION: &str = "normalized-host-control-a";
const STAGE2_PERTURBATION: &str = "normalized-host-control-b";
const STAGE1_OUTPUT_ROOT: &str = "proof-output:stage1";
const STAGE2_OUTPUT_ROOT: &str = "proof-output:stage2";
const STAGE1_STORE_ROOT: &str = "proof-store:stage1-clean-namespace";
const STAGE2_STORE_ROOT: &str = "proof-store:stage2-clean-namespace";
const STAGE1_OUTPUT_PATH: &str = "proof://cargo-free-fixed-point/stage1/mantle";
const STAGE2_OUTPUT_PATH: &str = "proof://cargo-free-fixed-point/stage2/mantle";
const STAGE1_SANDBOX: &str = "mantle-proof-sandbox-v1:source-built-stage1";
const STAGE2_SANDBOX: &str = "mantle-proof-sandbox-v1:source-built-stage2";
const EFFECT_POLICY_VERSION: &str = "mantle-build-effects-v1";
const REBUILD_DESCRIPTOR_SCHEMA: &str = "mantle-content-bound-rebuild-descriptor-v1";
const REBUILD_AUTHORITY_SCHEMA: &str = "mantle-rebuild-authority-plan-v1";
const PROOF_WORKFLOW: &str = "mantle-deterministic-proof-receipt-v2";
const STAGE_STATUS_COMPLETE: &str = "complete";
const STAGE_STATUS_SUCCESS: &str = "success";
const RECEIPT_BUNDLE_DIGEST_DOMAIN: &[u8] = b"mantle-source-built-fixed-point-proof-bundle-v2\0";
const APPROVED_READ_DIGEST_DOMAIN: &[u8] = b"mantle-source-built-fixed-point-approved-reads-v1\0";
const PROVIDER_IDENTITY_DIGEST_DOMAIN: &[u8] = b"mantle-source-built-fixed-point-provider-identity-v1\0";
const BLAKE3_HEX_LENGTH: usize = 64;
const HASH_BUFFER_BYTES: usize = 64 * 1_024;
const PROOF_ENTRY_COUNT_MAX: usize = 2_000_000;
const NON_DURABLE_PROOF_DIRECTORIES: &[&str] = &[
    "cargo-free-fixed-point/execution",
    "home",
    "native-state",
    "rust-provider-scratch",
    "tmp",
];
const STAGE_EVIDENCE_COUNT: usize = 6;
const REQUIRED_RUN_COUNT: usize = 2;
const REQUIRED_PERTURBATIONS: &[&str] = &[
    "HOME",
    "PATH",
    "USER",
    "LOGNAME",
    "TZ",
    "LANG",
    "LC_ALL",
    "TMPDIR",
    "cwd",
    "umask",
    "env-noise",
];
const NORMALIZATION_ENVELOPE: &[&str] = &[
    "normalization:time",
    "normalization:timezone=UTC",
    "normalization:locale=C",
    "normalization:temp-roots=proof-local",
    "normalization:host-user-metadata=mantle-proof",
    "normalization:umask=0022",
    "normalization:modeled-randomness=none",
    "normalization:order-sensitive-output-processing=canonical-sort",
];
const NON_CLAIMS: &[&str] = &[
    "This proof does not prove compiler correctness or seed correctness.",
    "This proof does not prove kernel isolation or complete Cargo compatibility.",
    "This proof does not prove independent rebuild agreement or bit-for-bit release reproducibility.",
    "This proof does not prove deployment success or release eligibility.",
    "The stage1 Mantle binary is an explicit fixed-point predecessor, not ambient or published-target path authority.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum StageExecutionOrigin {
    Executed,
    RestoredCheckpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SourceBuiltStageEvidence {
    pub(crate) stage_id: String,
    pub(crate) output_role: ProofOutputRole,
    pub(crate) status: String,
    pub(crate) execution_origin: StageExecutionOrigin,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) checkpoint_digest_blake3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) original_execution_evidence_digest_blake3: Option<String>,
    pub(crate) orchestrator: String,
    pub(crate) executable_identity: String,
    pub(crate) transcript_path: String,
    pub(crate) transcript_digest_blake3: String,
    pub(crate) audit_paths: Vec<String>,
    pub(crate) audit_digests_blake3: Vec<String>,
    pub(crate) output_path: String,
    pub(crate) output_digest_blake3: String,
    pub(crate) authority_violations: Vec<String>,
    pub(crate) fallback_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceBuiltReceiptExtension {
    schema: String,
    plan_digest_blake3: String,
    source_authority_digest_blake3: String,
    provider_kind: String,
    native_provider_digest_blake3: String,
    rust_provider_digest_blake3: String,
    toolchain_closure_digest_blake3: String,
    stage_evidence_path: String,
    stage_evidence_digest_blake3: String,
    final_proof_bundle_digest_blake3: String,
    protected_execution_policy_digest_blake3: String,
    effect_policy_digest_blake3: String,
    normalization_policy_digest_blake3: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_checkpoint_digest_blake3: Option<String>,
    non_claims: Vec<String>,
}

#[derive(Debug, Clone)]
struct FixedPointObservation {
    stage1: FixedPointStageObservation,
    stage2: FixedPointStageObservation,
    closure_policy_digest_blake3: String,
}

#[derive(Debug, Clone)]
struct FixedPointStageObservation {
    binary_path: PathBuf,
    binary_digest_blake3: String,
    receipt_digest_blake3: String,
    stderr_digest_blake3: String,
}

#[derive(Debug, Deserialize)]
struct FixedPointMeta {
    status: String,
    fixed_point: bool,
    hermeticity_mode: String,
    stage1: FixedPointMetaStage,
    stage2: Option<FixedPointMetaStage>,
    source_built_toolchain_closure: FixedPointClosureStatus,
    blocker: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FixedPointMetaStage {
    success: bool,
    cargo_marker_absent: bool,
    binary_blake3: Option<String>,
    smoke_status_code: Option<i32>,
    source_built_toolchain_closure_policy_digest_blake3: Option<String>,
    blocker: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FixedPointClosureStatus {
    claim: bool,
    status: String,
    policy_digest_blake3: Option<String>,
    seed_exception_count: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ProviderKindLinkage<'a> {
    schema: &'static str,
    proof_identity: ProviderKindValue<'a>,
    proof_linkage: ProviderKindValue<'a>,
    prerequisites: ProviderKindValue<'a>,
    source_built_plan_digest_blake3: &'a str,
}

#[derive(Debug, Serialize)]
struct ProviderKindValue<'a> {
    selected_provider_kind: &'a str,
}

#[derive(Debug, Serialize)]
struct ReleaseEvidenceInputs<'a> {
    schema: &'static str,
    deterministic_proof: &'a str,
    provider_kind_linkage: &'a str,
    source_built_plan: &'a str,
    toolchain_closure: &'a str,
    proof_bundle_digest: &'a str,
}

#[derive(Debug)]
struct ProofBundleObservation {
    digest_blake3: String,
    entry_count: usize,
}

pub(crate) fn write_source_built_fixed_point_receipt(
    proof_root: &Path,
    plan: &SourceBuiltFixedPointPlan,
    providers: &ConstructedProviders,
) -> Result<(), RunError> {
    let fixed_point = observe_fixed_point(proof_root)?;
    let rust_provider_digest_blake3 = hash_tree(&providers.rust_provider.output_path)?.1;
    let toolchain_closure_digest_blake3 = hash_file(&providers.toolchain_closure_path)?;
    let stage_evidence = stage_evidence(
        proof_root,
        plan,
        providers,
        &fixed_point,
        &rust_provider_digest_blake3,
        &toolchain_closure_digest_blake3,
    )?;
    let stage_evidence_path = proof_root.join(STAGE_EVIDENCE_FILE);
    write_json_create_new(&stage_evidence_path, &stage_evidence)?;
    let stage_evidence_digest_blake3 = hash_file(&stage_evidence_path)?;
    write_provider_kind_linkage(proof_root, plan)?;
    write_release_inputs(proof_root)?;
    let bundle = proof_bundle_digest(proof_root)?;
    write_text_create_new(
        &proof_root.join(crate::source_built_fixed_point_shell::FINAL_BUNDLE_DIGEST_FILE),
        &format!("{}  .\n", bundle.digest_blake3),
    )?;
    let provider_identity_digest_blake3 = provider_identity_digest(
        &providers.native_admission.output_digest_blake3,
        &rust_provider_digest_blake3,
        &toolchain_closure_digest_blake3,
    );
    let (descriptor, descriptor_digest, authority_plan, authority_plan_digest) = rebuild_authority(
        plan,
        &fixed_point,
        providers,
        &rust_provider_digest_blake3,
        &provider_identity_digest_blake3,
    )?;
    let core_receipt = deterministic_receipt(
        plan,
        &fixed_point,
        &provider_identity_digest_blake3,
        descriptor,
        descriptor_digest,
        authority_plan,
        authority_plan_digest,
    )?;
    let extension = SourceBuiltReceiptExtension {
        schema: EXTENSION_SCHEMA.to_string(),
        plan_digest_blake3: plan.plan_digest_blake3.clone(),
        source_authority_digest_blake3: plan.source_authority_digest_blake3.clone(),
        provider_kind: PROOF_PROVIDER_KIND.to_string(),
        native_provider_digest_blake3: providers.native_admission.output_digest_blake3.clone(),
        rust_provider_digest_blake3,
        toolchain_closure_digest_blake3,
        stage_evidence_path: STAGE_EVIDENCE_FILE.to_string(),
        stage_evidence_digest_blake3,
        final_proof_bundle_digest_blake3: bundle.digest_blake3.clone(),
        protected_execution_policy_digest_blake3: plan.policies.protected_execution_policy_digest_blake3.clone(),
        effect_policy_digest_blake3: plan.policies.effect_policy_digest_blake3.clone(),
        normalization_policy_digest_blake3: plan.policies.normalization_policy_digest_blake3.clone(),
        provider_checkpoint_digest_blake3: providers
            .provider_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.admission.checkpoint_digest_blake3.clone()),
        non_claims: NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    let receipt_path = proof_root.join(crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE);
    write_extended_receipt(&receipt_path, &core_receipt, &extension)?;
    verify_source_built_fixed_point_receipt(proof_root, &receipt_path)?;
    assert_eq!(stage_evidence.len(), STAGE_EVIDENCE_COUNT);
    debug_assert!(bundle.entry_count > 0);
    Ok(())
}

pub(crate) fn verify_source_built_fixed_point_receipt(proof_root: &Path, receipt_path: &Path) -> Result<(), RunError> {
    let bytes = fs::read(receipt_path)
        .map_err(|error| receipt_error(format!("reading receipt {}: {error}", receipt_path.display())))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| receipt_error(format!("parsing receipt {}: {error}", receipt_path.display())))?;
    let core: DeterministicBuildProofReceipt = serde_json::from_value(value.clone())
        .map_err(|error| receipt_error(format!("parsing deterministic receipt core: {error}")))?;
    let canonical = crunch_release_core::canonical_deterministic_build_proof_receipt(core)
        .map_err(|error| receipt_error(format!("validating deterministic receipt core: {error}")))?;
    if canonical.verdict != DeterministicBuildProofVerdict::SelfRebuildMatch || !canonical.blocking_reasons.is_empty() {
        return Err(receipt_error(format!(
            "deterministic receipt verdict is {:?}: {}",
            canonical.verdict,
            canonical.blocking_reasons.join("; ")
        )));
    }
    let extension: SourceBuiltReceiptExtension = serde_json::from_value(
        value
            .get(EXTENSION_FIELD)
            .cloned()
            .ok_or_else(|| receipt_error(format!("receipt missing {EXTENSION_FIELD}")))?,
    )
    .map_err(|error| receipt_error(format!("parsing source-built receipt extension: {error}")))?;
    if extension.schema != EXTENSION_SCHEMA || extension.provider_kind != PROOF_PROVIDER_KIND {
        return Err(receipt_error("source-built receipt extension identity mismatch".to_string()));
    }
    let stage_evidence_path = proof_root.join(&extension.stage_evidence_path);
    let observed_stage_evidence_digest = hash_file(&stage_evidence_path)?;
    if observed_stage_evidence_digest != extension.stage_evidence_digest_blake3 {
        return Err(receipt_error("source-built stage evidence digest mismatch".to_string()));
    }
    let stage_evidence: Vec<SourceBuiltStageEvidence> = serde_json::from_slice(
        &fs::read(&stage_evidence_path).map_err(|error| receipt_error(format!("reading stage evidence: {error}")))?,
    )
    .map_err(|error| receipt_error(format!("parsing stage evidence: {error}")))?;
    validate_stage_evidence(&stage_evidence)?;
    let observed_bundle = proof_bundle_digest(proof_root)?;
    if observed_bundle.digest_blake3 != extension.final_proof_bundle_digest_blake3 {
        return Err(receipt_error(format!(
            "proof bundle digest mismatch: expected {}, observed {}",
            extension.final_proof_bundle_digest_blake3, observed_bundle.digest_blake3
        )));
    }
    assert_eq!(canonical.runs.len(), REQUIRED_RUN_COUNT);
    debug_assert_eq!(stage_evidence.len(), STAGE_EVIDENCE_COUNT);
    Ok(())
}

fn observe_fixed_point(proof_root: &Path) -> Result<FixedPointObservation, RunError> {
    let meta_path = proof_root.join(FIXED_POINT_META_RELATIVE_PATH);
    let meta: FixedPointMeta =
        serde_json::from_slice(&fs::read(&meta_path).map_err(|error| {
            receipt_error(format!("reading fixed-point metadata {}: {error}", meta_path.display()))
        })?)
        .map_err(|error| receipt_error(format!("parsing fixed-point metadata {}: {error}", meta_path.display())))?;
    let stage2_meta = meta
        .stage2
        .as_ref()
        .ok_or_else(|| receipt_error("fixed-point metadata has no stage2".to_string()))?;
    validate_fixed_point_meta(&meta, stage2_meta)?;
    let stage1 = observe_fixed_point_stage(
        proof_root,
        &meta.stage1,
        FIXED_POINT_STAGE1_BINARY,
        FIXED_POINT_STAGE1_RECEIPT,
        FIXED_POINT_STAGE1_STDERR,
    )?;
    let stage2 = observe_fixed_point_stage(
        proof_root,
        stage2_meta,
        FIXED_POINT_STAGE2_BINARY,
        FIXED_POINT_STAGE2_RECEIPT,
        FIXED_POINT_STAGE2_STDERR,
    )?;
    if stage1.binary_digest_blake3 != stage2.binary_digest_blake3 {
        return Err(receipt_error("stage1 and stage2 Mantle binary digests differ".to_string()));
    }
    let closure_policy_digest_blake3 = meta
        .source_built_toolchain_closure
        .policy_digest_blake3
        .ok_or_else(|| receipt_error("fixed-point closure policy digest is missing".to_string()))?;
    assert_eq!(closure_policy_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert_eq!(stage1.binary_digest_blake3, stage2.binary_digest_blake3);
    Ok(FixedPointObservation {
        stage1,
        stage2,
        closure_policy_digest_blake3,
    })
}

fn validate_fixed_point_meta(meta: &FixedPointMeta, stage2: &FixedPointMetaStage) -> Result<(), RunError> {
    if meta.status != STAGE_STATUS_SUCCESS || !meta.fixed_point || meta.hermeticity_mode != HERMETICITY_MODE {
        return Err(receipt_error("fixed-point metadata does not record strict success".to_string()));
    }
    if meta.blocker.is_some() || meta.stage1.blocker.is_some() || stage2.blocker.is_some() {
        return Err(receipt_error("fixed-point metadata contains a blocker".to_string()));
    }
    for (label, stage) in [(STAGE1_RUN_ID, &meta.stage1), (STAGE2_RUN_ID, stage2)] {
        if !stage.success || !stage.cargo_marker_absent || stage.smoke_status_code != Some(0) {
            return Err(receipt_error(format!("{label} is not a successful Cargo-free smoke-checked stage")));
        }
        if stage.binary_blake3.is_none() || stage.source_built_toolchain_closure_policy_digest_blake3.is_none() {
            return Err(receipt_error(format!("{label} lacks binary or closure-policy identity")));
        }
    }
    if !meta.source_built_toolchain_closure.claim
        || meta.source_built_toolchain_closure.status != "enforced-source-built"
        || meta.source_built_toolchain_closure.seed_exception_count != Some(0)
    {
        return Err(receipt_error("fixed-point closure is not zero-seed and source-built".to_string()));
    }
    assert_eq!(meta.stage1.binary_blake3, stage2.binary_blake3);
    debug_assert_eq!(meta.stage1.smoke_status_code, Some(0));
    Ok(())
}

fn observe_fixed_point_stage(
    proof_root: &Path,
    meta: &FixedPointMetaStage,
    binary_relative_path: &str,
    receipt_relative_path: &str,
    stderr_relative_path: &str,
) -> Result<FixedPointStageObservation, RunError> {
    let binary_path = proof_root.join(binary_relative_path);
    let receipt_path = proof_root.join(receipt_relative_path);
    let stderr_path = proof_root.join(stderr_relative_path);
    let binary_digest_blake3 = hash_file(&binary_path)?;
    if meta.binary_blake3.as_deref() != Some(binary_digest_blake3.as_str()) {
        return Err(receipt_error(format!("fixed-point stage binary digest mismatch: {binary_relative_path}")));
    }
    let receipt_digest_blake3 = hash_file(&receipt_path)?;
    let stderr_digest_blake3 = hash_file_allow_empty(&stderr_path)?;
    assert_eq!(binary_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(receipt_path.is_file());
    Ok(FixedPointStageObservation {
        binary_path,
        binary_digest_blake3,
        receipt_digest_blake3,
        stderr_digest_blake3,
    })
}

struct StageExecutionBinding {
    origin: StageExecutionOrigin,
    checkpoint_digest_blake3: Option<String>,
    execution_evidence_digest_blake3: Option<String>,
    producer_executable_digest_blake3: Option<String>,
}

impl StageExecutionBinding {
    fn executable_identity(&self, executed_digest_blake3: &str) -> String {
        let digest = self.producer_executable_digest_blake3.as_deref().unwrap_or(executed_digest_blake3);
        format!("blake3:{digest}")
    }
}

fn stage_execution_binding(
    providers: &ConstructedProviders,
    output_role: ProofOutputRole,
) -> Result<StageExecutionBinding, RunError> {
    let Some(checkpoint) = providers.provider_checkpoint.as_ref() else {
        return Ok(executed_stage_binding());
    };
    if !is_provider_checkpoint_role(output_role) {
        return Ok(executed_stage_binding());
    }
    let mut matches = checkpoint.manifest.stages.iter().filter(|stage| stage.output_role == output_role);
    let record = matches
        .next()
        .ok_or_else(|| receipt_error(format!("provider checkpoint is missing stage role {output_role:?}")))?;
    if matches.next().is_some() {
        return Err(receipt_error(format!("provider checkpoint duplicates stage role {output_role:?}")));
    }
    Ok(StageExecutionBinding {
        origin: StageExecutionOrigin::RestoredCheckpoint,
        checkpoint_digest_blake3: Some(checkpoint.admission.checkpoint_digest_blake3.clone()),
        execution_evidence_digest_blake3: Some(record.execution_evidence_digest_blake3.clone()),
        producer_executable_digest_blake3: Some(record.producer_executable_digest_blake3.clone()),
    })
}

fn executed_stage_binding() -> StageExecutionBinding {
    StageExecutionBinding {
        origin: StageExecutionOrigin::Executed,
        checkpoint_digest_blake3: None,
        execution_evidence_digest_blake3: None,
        producer_executable_digest_blake3: None,
    }
}

fn is_provider_checkpoint_role(role: ProofOutputRole) -> bool {
    matches!(
        role,
        ProofOutputRole::StagexTransition
            | ProofOutputRole::StagexProvider
            | ProofOutputRole::FullSourceNativeProvider
            | ProofOutputRole::FullSourceRustProvider
    )
}

fn canonical_stage_id(role: ProofOutputRole) -> &'static str {
    match role {
        ProofOutputRole::StagexTransition => crate::source_built_fixed_point::STAGEX_TRANSITION_STAGE_ID,
        ProofOutputRole::StagexProvider => crate::source_built_fixed_point::STAGEX_PROVIDER_STAGE_ID,
        ProofOutputRole::FullSourceNativeProvider => crate::source_built_fixed_point::FULL_SOURCE_NATIVE_STAGE_ID,
        ProofOutputRole::FullSourceRustProvider => crate::source_built_fixed_point::FULL_SOURCE_RUST_STAGE_ID,
        ProofOutputRole::MantleStage1 => crate::source_built_fixed_point::MANTLE_STAGE1_STAGE_ID,
        ProofOutputRole::MantleStage2 => crate::source_built_fixed_point::MANTLE_STAGE2_STAGE_ID,
    }
}

fn stage_evidence(
    proof_root: &Path,
    plan: &SourceBuiltFixedPointPlan,
    providers: &ConstructedProviders,
    fixed_point: &FixedPointObservation,
    rust_provider_digest_blake3: &str,
    toolchain_closure_digest_blake3: &str,
) -> Result<Vec<SourceBuiltStageEvidence>, RunError> {
    let current_executable =
        std::env::current_exe().map_err(|error| receipt_error(format!("resolving proof executable: {error}")))?;
    let current_executable_digest = hash_file(&current_executable)?;
    let transition_root = &providers.stagex_transition_execution_dir;
    let transition_report = transition_root.join(STAGEX_TRANSITION_REPORT_FILE);
    let transition_audit = transition_root.join(STAGEX_TRANSITION_AUDIT_FILE);
    let transition_identity =
        hash_preserved_stagex_transition_tree(transition_root, plan.resource_bounds.disk_bytes_max)?;
    let stagex_receipt = providers.stagex_provider_report.receipt_path.clone();
    let stagex_validation = providers
        .stagex_provider_report
        .output_path
        .join(crate::stagex_provider::PROVIDER_VALIDATION_RELATIVE_PATH);
    let rust_build_receipt = proof_root.join(RUST_PROVIDER_BUILD_RECEIPT);
    let rust_binding_receipt = proof_root.join(RUST_PROVIDER_BINDING_RECEIPT);
    let rustc_path = proof_root.join(RUST_PROVIDER_RUSTC);
    let rustc_digest = hash_file(&rustc_path)?;
    let closure_path = proof_root.join(TOOLCHAIN_CLOSURE_FILE);
    if hash_file(&closure_path)? != toolchain_closure_digest_blake3 {
        return Err(receipt_error("toolchain closure digest changed during receipt construction".to_string()));
    }
    let transition_binding = stage_execution_binding(providers, ProofOutputRole::StagexTransition)?;
    let stagex_binding = stage_execution_binding(providers, ProofOutputRole::StagexProvider)?;
    let native_binding = stage_execution_binding(providers, ProofOutputRole::FullSourceNativeProvider)?;
    let rust_binding = stage_execution_binding(providers, ProofOutputRole::FullSourceRustProvider)?;
    let stage1_binding = stage_execution_binding(providers, ProofOutputRole::MantleStage1)?;
    let stage2_binding = stage_execution_binding(providers, ProofOutputRole::MantleStage2)?;
    let transition_executable = transition_binding.executable_identity(&current_executable_digest);
    let stagex_executable = stagex_binding.executable_identity(&current_executable_digest);
    let native_executable = native_binding.executable_identity(&current_executable_digest);
    let rust_executable = rust_binding.executable_identity(&rustc_digest);
    let evidence = vec![
        SourceBuiltStageEvidence {
            stage_id: canonical_stage_id(ProofOutputRole::StagexTransition).to_string(),
            output_role: ProofOutputRole::StagexTransition,
            status: STAGE_STATUS_COMPLETE.to_string(),
            execution_origin: transition_binding.origin,
            checkpoint_digest_blake3: transition_binding.checkpoint_digest_blake3,
            original_execution_evidence_digest_blake3: transition_binding.execution_evidence_digest_blake3,
            orchestrator: "host-mantle".to_string(),
            executable_identity: transition_executable,
            transcript_path: relative_path(proof_root, &transition_report)?,
            transcript_digest_blake3: hash_file(&transition_report)?,
            audit_paths: vec![relative_path(proof_root, &transition_audit)?],
            audit_digests_blake3: vec![hash_file(&transition_audit)?],
            output_path: relative_path(proof_root, transition_root)?,
            output_digest_blake3: transition_identity.digest_blake3,
            authority_violations: Vec::new(),
            fallback_events: Vec::new(),
        },
        SourceBuiltStageEvidence {
            stage_id: canonical_stage_id(ProofOutputRole::StagexProvider).to_string(),
            output_role: ProofOutputRole::StagexProvider,
            status: STAGE_STATUS_COMPLETE.to_string(),
            execution_origin: stagex_binding.origin,
            checkpoint_digest_blake3: stagex_binding.checkpoint_digest_blake3,
            original_execution_evidence_digest_blake3: stagex_binding.execution_evidence_digest_blake3,
            orchestrator: "host-mantle".to_string(),
            executable_identity: stagex_executable,
            transcript_path: relative_path(proof_root, &stagex_receipt)?,
            transcript_digest_blake3: hash_file(&stagex_receipt)?,
            audit_paths: vec![relative_path(proof_root, &stagex_validation)?],
            audit_digests_blake3: vec![hash_file(&stagex_validation)?],
            output_path: relative_path(proof_root, &providers.stagex_provider_report.output_path)?,
            output_digest_blake3: providers.stagex_provider_report.final_bundle_digest_blake3.clone(),
            authority_violations: Vec::new(),
            fallback_events: Vec::new(),
        },
        SourceBuiltStageEvidence {
            stage_id: canonical_stage_id(ProofOutputRole::FullSourceNativeProvider).to_string(),
            output_role: ProofOutputRole::FullSourceNativeProvider,
            status: STAGE_STATUS_COMPLETE.to_string(),
            execution_origin: native_binding.origin,
            checkpoint_digest_blake3: native_binding.checkpoint_digest_blake3,
            original_execution_evidence_digest_blake3: native_binding.execution_evidence_digest_blake3,
            orchestrator: "host-mantle".to_string(),
            executable_identity: native_executable,
            transcript_path: relative_path(proof_root, &providers.native_provider.transcript_path)?,
            transcript_digest_blake3: providers.native_provider.transcript_digest_blake3.clone(),
            audit_paths: vec![relative_path(proof_root, &providers.native_admission_report_path)?],
            audit_digests_blake3: vec![hash_file(&providers.native_admission_report_path)?],
            output_path: relative_path(proof_root, &providers.native_provider.output.path)?,
            output_digest_blake3: providers.native_admission.output_digest_blake3.clone(),
            authority_violations: Vec::new(),
            fallback_events: Vec::new(),
        },
        SourceBuiltStageEvidence {
            stage_id: canonical_stage_id(ProofOutputRole::FullSourceRustProvider).to_string(),
            output_role: ProofOutputRole::FullSourceRustProvider,
            status: STAGE_STATUS_COMPLETE.to_string(),
            execution_origin: rust_binding.origin,
            checkpoint_digest_blake3: rust_binding.checkpoint_digest_blake3,
            original_execution_evidence_digest_blake3: rust_binding.execution_evidence_digest_blake3,
            orchestrator: "host-mantle".to_string(),
            executable_identity: rust_executable,
            transcript_path: RUST_PROVIDER_BUILD_RECEIPT.to_string(),
            transcript_digest_blake3: hash_file(&rust_build_receipt)?,
            audit_paths: vec![RUST_PROVIDER_BINDING_RECEIPT.to_string()],
            audit_digests_blake3: vec![hash_file(&rust_binding_receipt)?],
            output_path: "rust-provider".to_string(),
            output_digest_blake3: rust_provider_digest_blake3.to_string(),
            authority_violations: Vec::new(),
            fallback_events: Vec::new(),
        },
        SourceBuiltStageEvidence {
            stage_id: canonical_stage_id(ProofOutputRole::MantleStage1).to_string(),
            output_role: ProofOutputRole::MantleStage1,
            status: STAGE_STATUS_COMPLETE.to_string(),
            execution_origin: stage1_binding.origin,
            checkpoint_digest_blake3: stage1_binding.checkpoint_digest_blake3,
            original_execution_evidence_digest_blake3: stage1_binding.execution_evidence_digest_blake3,
            orchestrator: "host-mantle".to_string(),
            executable_identity: format!("blake3:{rustc_digest}"),
            transcript_path: FIXED_POINT_STAGE1_RECEIPT.to_string(),
            transcript_digest_blake3: fixed_point.stage1.receipt_digest_blake3.clone(),
            audit_paths: vec![
                FIXED_POINT_STAGE1_STDERR.to_string(),
                TOOLCHAIN_CLOSURE_FILE.to_string(),
            ],
            audit_digests_blake3: vec![
                fixed_point.stage1.stderr_digest_blake3.clone(),
                toolchain_closure_digest_blake3.to_string(),
            ],
            output_path: FIXED_POINT_STAGE1_BINARY.to_string(),
            output_digest_blake3: fixed_point.stage1.binary_digest_blake3.clone(),
            authority_violations: Vec::new(),
            fallback_events: Vec::new(),
        },
        SourceBuiltStageEvidence {
            stage_id: canonical_stage_id(ProofOutputRole::MantleStage2).to_string(),
            output_role: ProofOutputRole::MantleStage2,
            status: STAGE_STATUS_COMPLETE.to_string(),
            execution_origin: stage2_binding.origin,
            checkpoint_digest_blake3: stage2_binding.checkpoint_digest_blake3,
            original_execution_evidence_digest_blake3: stage2_binding.execution_evidence_digest_blake3,
            orchestrator: "stage1-mantle".to_string(),
            executable_identity: format!("blake3:{}", fixed_point.stage1.binary_digest_blake3),
            transcript_path: FIXED_POINT_STAGE2_RECEIPT.to_string(),
            transcript_digest_blake3: fixed_point.stage2.receipt_digest_blake3.clone(),
            audit_paths: vec![
                FIXED_POINT_STAGE2_STDERR.to_string(),
                TOOLCHAIN_CLOSURE_FILE.to_string(),
            ],
            audit_digests_blake3: vec![
                fixed_point.stage2.stderr_digest_blake3.clone(),
                toolchain_closure_digest_blake3.to_string(),
            ],
            output_path: FIXED_POINT_STAGE2_BINARY.to_string(),
            output_digest_blake3: fixed_point.stage2.binary_digest_blake3.clone(),
            authority_violations: Vec::new(),
            fallback_events: Vec::new(),
        },
    ];
    validate_stage_evidence(&evidence)?;
    validate_stage_plan_alignment(plan, &evidence)?;
    assert_eq!(evidence.len(), STAGE_EVIDENCE_COUNT);
    debug_assert!(evidence.iter().all(|stage| stage.fallback_events.is_empty()));
    Ok(evidence)
}

fn validate_stage_evidence(evidence: &[SourceBuiltStageEvidence]) -> Result<(), RunError> {
    if evidence.len() != STAGE_EVIDENCE_COUNT {
        return Err(receipt_error(format!(
            "stage evidence must contain {STAGE_EVIDENCE_COUNT} stages, got {}",
            evidence.len()
        )));
    }
    let mut ids = BTreeSet::new();
    for stage in evidence {
        if !ids.insert(stage.stage_id.as_str()) {
            return Err(receipt_error(format!("duplicate stage evidence id {}", stage.stage_id)));
        }
        for (label, digest) in [
            ("transcript", stage.transcript_digest_blake3.as_str()),
            ("output", stage.output_digest_blake3.as_str()),
        ] {
            validate_digest(&format!("{} {label}", stage.stage_id), digest)?;
        }
        if stage.status != STAGE_STATUS_COMPLETE
            || !stage.authority_violations.is_empty()
            || !stage.fallback_events.is_empty()
            || stage.audit_paths.len() != stage.audit_digests_blake3.len()
        {
            return Err(receipt_error(format!("stage evidence {} is incomplete", stage.stage_id)));
        }
        validate_stage_execution_origin(stage)?;
        for digest in &stage.audit_digests_blake3 {
            validate_digest("stage audit", digest)?;
        }
    }
    assert_eq!(ids.len(), STAGE_EVIDENCE_COUNT);
    debug_assert!(evidence.iter().all(|stage| stage.status == STAGE_STATUS_COMPLETE));
    Ok(())
}

fn validate_stage_execution_origin(stage: &SourceBuiltStageEvidence) -> Result<(), RunError> {
    match stage.execution_origin {
        StageExecutionOrigin::Executed => {
            if stage.checkpoint_digest_blake3.is_some() || stage.original_execution_evidence_digest_blake3.is_some() {
                return Err(receipt_error(format!(
                    "executed stage {} carries checkpoint-only evidence",
                    stage.stage_id
                )));
            }
        }
        StageExecutionOrigin::RestoredCheckpoint => {
            if !is_provider_checkpoint_role(stage.output_role) {
                return Err(receipt_error(format!(
                    "non-provider stage {} cannot be restored from the provider checkpoint",
                    stage.stage_id
                )));
            }
            let checkpoint = stage
                .checkpoint_digest_blake3
                .as_deref()
                .ok_or_else(|| receipt_error(format!("restored stage {} has no checkpoint digest", stage.stage_id)))?;
            let execution = stage
                .original_execution_evidence_digest_blake3
                .as_deref()
                .ok_or_else(|| receipt_error(format!("restored stage {} has no execution evidence", stage.stage_id)))?;
            validate_digest("restored checkpoint", checkpoint)?;
            validate_digest("restored execution evidence", execution)?;
        }
    }
    Ok(())
}

fn validate_stage_plan_alignment(
    plan: &SourceBuiltFixedPointPlan,
    evidence: &[SourceBuiltStageEvidence],
) -> Result<(), RunError> {
    validate_stage_plan_entries(&plan.stages, evidence)
}

fn validate_stage_plan_entries(
    planned_stages: &[crate::source_built_fixed_point::SourceBuiltFixedPointStagePlan],
    evidence: &[SourceBuiltStageEvidence],
) -> Result<(), RunError> {
    if planned_stages.len() != evidence.len() {
        return Err(receipt_error("plan and stage evidence counts differ".to_string()));
    }
    for (planned, observed) in planned_stages.iter().zip(evidence) {
        if planned.stage_id != observed.stage_id || planned.output != observed.output_role {
            return Err(receipt_error(format!(
                "stage evidence does not match plan: planned {} {:?}, observed {} {:?}",
                planned.stage_id, planned.output, observed.stage_id, observed.output_role
            )));
        }
    }
    assert_eq!(planned_stages.len(), STAGE_EVIDENCE_COUNT);
    debug_assert_eq!(evidence.len(), STAGE_EVIDENCE_COUNT);
    Ok(())
}

fn rebuild_authority(
    plan: &SourceBuiltFixedPointPlan,
    fixed_point: &FixedPointObservation,
    providers: &ConstructedProviders,
    rust_provider_digest_blake3: &str,
    provider_identity_digest_blake3: &str,
) -> Result<(ContentBoundRebuildDescriptor, String, RebuildAuthorityPlan, String), RunError> {
    let arguments = vec![
        "workflow:source-built-fixed-point".to_string(),
        format!("plan-blake3:{}", plan.plan_digest_blake3),
        format!("source-authority-blake3:{}", plan.source_authority_digest_blake3),
        format!("closure-policy-blake3:{}", fixed_point.closure_policy_digest_blake3),
    ];
    let arguments_blake3 = rebuild_arguments_digest_blake3(arguments.clone())
        .map_err(|error| receipt_error(format!("digesting rebuild arguments: {error}")))?;
    let source_inputs = plan.source_inputs.iter().map(rebuild_source_identity).collect::<Vec<_>>();
    validate_rebuild_source_input_count(source_inputs.len())?;
    let target = RebuildContentIdentity {
        name: MANTLE_OUTPUT_NAME.to_string(),
        role: RebuildInputRole::PublishedTarget,
        kind: RebuildContentKind::RegularFile,
        digest_blake3: fixed_point.stage1.binary_digest_blake3.clone(),
        size_bytes: file_size(&fixed_point.stage1.binary_path)?,
    };
    let recipe = RebuildContentIdentity {
        name: PLAN_FILE.to_string(),
        role: RebuildInputRole::Recipe,
        kind: RebuildContentKind::RegularFile,
        digest_blake3: plan.plan_digest_blake3.clone(),
        size_bytes: 1,
    };
    let executable = RebuildContentIdentity {
        name: "source-built-rustc".to_string(),
        role: RebuildInputRole::Executable,
        kind: RebuildContentKind::RegularFile,
        digest_blake3: hash_file(&providers.rust_provider.output_path.join("bin/rustc"))?,
        size_bytes: file_size(&providers.rust_provider.output_path.join("bin/rustc"))?,
    };
    let stage1_orchestrator = RebuildContentIdentity {
        name: "stage1-mantle-orchestrator".to_string(),
        role: RebuildInputRole::Tool,
        kind: RebuildContentKind::RegularFile,
        digest_blake3: fixed_point.stage1.binary_digest_blake3.clone(),
        size_bytes: file_size(&fixed_point.stage1.binary_path)?,
    };
    let provider = RebuildContentIdentity {
        name: "source-built-native-and-rust-provider-closure".to_string(),
        role: RebuildInputRole::Provider,
        kind: RebuildContentKind::Directory,
        digest_blake3: provider_identity_digest_blake3.to_string(),
        size_bytes: file_size(&providers.native_admission_report_path)?
            .saturating_add(file_size(&providers.rust_provider.metadata_path)?),
    };
    let run_roots = vec![
        RebuildRunRootIdentity {
            run_id: STAGE1_RUN_ID.to_string(),
            output_root_identity: STAGE1_OUTPUT_ROOT.to_string(),
            store_root_identity: STAGE1_STORE_ROOT.to_string(),
        },
        RebuildRunRootIdentity {
            run_id: STAGE2_RUN_ID.to_string(),
            output_root_identity: STAGE2_OUTPUT_ROOT.to_string(),
            store_root_identity: STAGE2_STORE_ROOT.to_string(),
        },
    ];
    let descriptor = ContentBoundRebuildDescriptor {
        schema: REBUILD_DESCRIPTOR_SCHEMA.to_string(),
        target_artifacts: vec![target],
        recipe,
        executable,
        tools: vec![stage1_orchestrator],
        ordered_arguments: arguments,
        arguments_blake3,
        source_inputs,
        provider,
        policies: RebuildPolicyIdentities {
            sandbox_policy_blake3: plan.policies.protected_execution_policy_digest_blake3.clone(),
            effect_policy_blake3: plan.policies.effect_policy_digest_blake3.clone(),
            normalization_policy_blake3: plan.policies.normalization_policy_digest_blake3.clone(),
        },
        run_roots,
    };
    let descriptor_digest = content_bound_rebuild_descriptor_digest_blake3(descriptor.clone())
        .map_err(|error| receipt_error(format!("validating rebuild descriptor: {error}")))?;
    let approved_read_identities = approved_read_identities(plan, &descriptor, rust_provider_digest_blake3);
    let approved_read_paths_blake3 = digest_string_set(APPROVED_READ_DIGEST_DOMAIN, &approved_read_identities);
    let authority_plan = RebuildAuthorityPlan {
        schema: REBUILD_AUTHORITY_SCHEMA.to_string(),
        descriptor_blake3: descriptor_digest.clone(),
        approved_read_identities,
        approved_read_paths_blake3,
        fresh_write_root_identities: vec![STAGE1_OUTPUT_ROOT.to_string(), STAGE2_OUTPUT_ROOT.to_string()],
        target_authority_excluded: true,
        blockers: Vec::new(),
    };
    let authority_plan_digest = rebuild_authority_plan_digest_blake3(authority_plan.clone())
        .map_err(|error| receipt_error(format!("validating rebuild authority plan: {error}")))?;
    assert!(authority_plan.target_authority_excluded);
    debug_assert!(authority_plan.blockers.is_empty());
    Ok((descriptor, descriptor_digest, authority_plan, authority_plan_digest))
}

fn deterministic_receipt(
    plan: &SourceBuiltFixedPointPlan,
    fixed_point: &FixedPointObservation,
    provider_identity_digest_blake3: &str,
    descriptor: ContentBoundRebuildDescriptor,
    descriptor_digest: String,
    authority_plan: RebuildAuthorityPlan,
    authority_plan_digest: String,
) -> Result<DeterministicBuildProofReceipt, RunError> {
    let approved_reads = authority_plan.approved_read_identities.clone();
    let output_digest = fixed_point.stage1.binary_digest_blake3.clone();
    let runs = vec![
        deterministic_run(
            STAGE1_RUN_ID,
            STAGE1_PERTURBATION,
            STAGE1_OUTPUT_PATH,
            STAGE1_OUTPUT_ROOT,
            STAGE1_SANDBOX,
            &output_digest,
            &descriptor_digest,
            &authority_plan_digest,
            &approved_reads,
        ),
        deterministic_run(
            STAGE2_RUN_ID,
            STAGE2_PERTURBATION,
            STAGE2_OUTPUT_PATH,
            STAGE2_OUTPUT_ROOT,
            STAGE2_SANDBOX,
            &output_digest,
            &descriptor_digest,
            &authority_plan_digest,
            &approved_reads,
        ),
    ];
    let toolchain_provider_identity = format!(
        "provider-kind={PROOF_PROVIDER_KIND};identity-blake3={provider_identity_digest_blake3};closure-policy-blake3={}",
        fixed_point.closure_policy_digest_blake3
    );
    let init = DeterministicBuildProofReceiptInit {
        proof_unit: DeterministicProofUnit {
            target_artifact_identity: PROOF_TARGET_IDENTITY.to_string(),
            output_identities: vec![format!("{MANTLE_OUTPUT_NAME}:{output_digest}")],
        },
        derivation_identity: format!("source-built-plan:{}", plan.plan_digest_blake3),
        hermeticity_mode: HERMETICITY_MODE.to_string(),
        workflow_version: PROOF_WORKFLOW.to_string(),
        selected_provider_kind: PROOF_PROVIDER_KIND.to_string(),
        source_blake3: plan.source_authority_digest_blake3.clone(),
        vendor_blake3: plan.receipt_contract.vendor_blake3.clone(),
        toolchain_provider_identity,
        toolchain_stage_roots: vec![
            format!("stagex-provider:{}", plan.policies.protected_execution_policy_digest_blake3),
            format!("native-provider:{}", plan.policies.expected_native_provider_digest_blake3),
            format!("closure-policy:{}", fixed_point.closure_policy_digest_blake3),
        ],
        logical_store_prefix: plan.logical_store_prefix.clone(),
        physical_store_isolation: PHYSICAL_STORE_ISOLATION.to_string(),
        effect_policy_version: EFFECT_POLICY_VERSION.to_string(),
        declared_effects: vec![
            BuildEffect::ReadStore,
            BuildEffect::WriteOutput,
            BuildEffect::Environment,
        ],
        observed_effects: Some(vec![
            BuildEffect::ReadStore,
            BuildEffect::WriteOutput,
            BuildEffect::Environment,
        ]),
        normalized_execution_envelope: NORMALIZATION_ENVELOPE.iter().map(|value| (*value).to_string()).collect(),
        ambient_host_perturbations: REQUIRED_PERTURBATIONS.iter().map(|value| (*value).to_string()).collect(),
        sandbox_profile_identities: vec![STAGE1_SANDBOX.to_string(), STAGE2_SANDBOX.to_string()],
        runs,
        rebuild_descriptor: descriptor,
        rebuild_descriptor_blake3: descriptor_digest,
        rebuild_authority_plan: authority_plan,
        rebuild_authority_plan_blake3: authority_plan_digest,
    };
    let receipt = DeterministicBuildProofReceipt::new(init);
    if receipt.verdict != DeterministicBuildProofVerdict::SelfRebuildMatch || !receipt.blocking_reasons.is_empty() {
        return Err(receipt_error(format!(
            "source-built deterministic receipt classification failed: {:?}: {}",
            receipt.verdict,
            receipt.blocking_reasons.join("; ")
        )));
    }
    let digest = deterministic_build_proof_receipt_digest_blake3(receipt.clone())
        .map_err(|error| receipt_error(format!("digesting deterministic receipt: {error}")))?;
    let mut receipt = receipt;
    receipt.receipt_blake3 = Some(digest);
    let canonical = crunch_release_core::canonical_deterministic_build_proof_receipt(receipt)
        .map_err(|error| receipt_error(format!("canonicalizing deterministic receipt: {error}")))?;
    assert_eq!(canonical.runs.len(), REQUIRED_RUN_COUNT);
    debug_assert_eq!(canonical.verdict, DeterministicBuildProofVerdict::SelfRebuildMatch);
    Ok(canonical)
}

fn deterministic_run(
    run_id: &str,
    perturbation_case: &str,
    output_store_path: &str,
    output_root_identity: &str,
    sandbox_profile_identity: &str,
    output_digest: &str,
    descriptor_digest: &str,
    authority_plan_digest: &str,
    approved_reads: &[String],
) -> DeterministicBuildRunReceipt {
    assert!(!approved_reads.is_empty());
    assert_eq!(output_digest.len(), BLAKE3_HEX_LENGTH);
    DeterministicBuildRunReceipt {
        run_id: run_id.to_string(),
        perturbation_case: perturbation_case.to_string(),
        output_store_paths: vec![output_store_path.to_string()],
        output_root_identity: output_root_identity.to_string(),
        sandbox_profile_identity: sandbox_profile_identity.to_string(),
        output_digests: vec![DeterministicOutputDigest {
            name: MANTLE_OUTPUT_NAME.to_string(),
            digest_blake3: output_digest.to_string(),
        }],
        substituted_dependency_identities: Vec::new(),
        hermeticity_audit_events: Vec::new(),
        observed_effects: Some(vec![
            BuildEffect::ReadStore,
            BuildEffect::WriteOutput,
            BuildEffect::Environment,
        ]),
        rebuild_descriptor_blake3: Some(descriptor_digest.to_string()),
        rebuild_authority_plan_blake3: Some(authority_plan_digest.to_string()),
        observed_read_identities: approved_reads.to_vec(),
        authority_violations: Vec::new(),
    }
}

fn rebuild_source_identity(input: &crate::source_built_fixed_point::SourceAuthorityInput) -> RebuildContentIdentity {
    let kind = match input.kind {
        crate::source_built_fixed_point::SourceContentKind::RegularFile => RebuildContentKind::RegularFile,
        crate::source_built_fixed_point::SourceContentKind::Directory => RebuildContentKind::Directory,
    };
    RebuildContentIdentity {
        name: format!("{:?}:{}", input.role, input.id),
        role: RebuildInputRole::Source,
        kind,
        digest_blake3: input.digest_blake3.clone(),
        size_bytes: input.size_bytes,
    }
}

fn validate_rebuild_source_input_count(source_input_count: usize) -> Result<(), RunError> {
    let source_input_count = u32::try_from(source_input_count)
        .map_err(|_| receipt_error("rebuild descriptor source input count exceeds u32".to_string()))?;
    if source_input_count != SOURCE_BUILT_FIXED_POINT_REQUIRED_SOURCE_ROLE_COUNT {
        return Err(receipt_error(format!(
            "rebuild descriptor source input count does not match the plan contract: expected {SOURCE_BUILT_FIXED_POINT_REQUIRED_SOURCE_ROLE_COUNT}, got {source_input_count}"
        )));
    }
    Ok(())
}

fn approved_read_identities(
    plan: &SourceBuiltFixedPointPlan,
    descriptor: &ContentBoundRebuildDescriptor,
    rust_provider_digest_blake3: &str,
) -> Vec<String> {
    let mut reads = plan
        .source_inputs
        .iter()
        .map(|input| format!("source:{:?}:{}:{}", input.role, input.id, input.digest_blake3))
        .collect::<Vec<_>>();
    reads.extend([
        format!("recipe:{}", plan.plan_digest_blake3),
        format!("provider:{}", descriptor.provider.digest_blake3),
        format!("rust-provider:{rust_provider_digest_blake3}"),
        format!("policy:sandbox:{}", descriptor.policies.sandbox_policy_blake3),
        format!("policy:effect:{}", descriptor.policies.effect_policy_blake3),
        format!("policy:normalization:{}", descriptor.policies.normalization_policy_blake3),
        format!("stage1-orchestrator:{}", descriptor.tools[0].digest_blake3),
    ]);
    reads.sort();
    reads.dedup();
    assert!(reads.len() > plan.source_inputs.len());
    debug_assert!(reads.windows(2).all(|pair| pair[0] < pair[1]));
    reads
}

fn provider_identity_digest(native: &str, rust: &str, closure: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PROVIDER_IDENTITY_DIGEST_DOMAIN);
    for (role, digest) in [("native", native), ("rust", rust), ("closure", closure)] {
        hasher.update(role.as_bytes());
        hasher.update(&[0]);
        hasher.update(digest.as_bytes());
        hasher.update(&[0]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(native != rust || rust != closure);
    digest
}

fn write_provider_kind_linkage(proof_root: &Path, plan: &SourceBuiltFixedPointPlan) -> Result<(), RunError> {
    let linkage = ProviderKindLinkage {
        schema: PROVIDER_LINKAGE_SCHEMA,
        proof_identity: ProviderKindValue {
            selected_provider_kind: PROOF_PROVIDER_KIND,
        },
        proof_linkage: ProviderKindValue {
            selected_provider_kind: PROOF_PROVIDER_KIND,
        },
        prerequisites: ProviderKindValue {
            selected_provider_kind: PROOF_PROVIDER_KIND,
        },
        source_built_plan_digest_blake3: &plan.plan_digest_blake3,
    };
    write_json_create_new(&proof_root.join(crate::source_built_fixed_point_shell::PROVIDER_KIND_LINKAGE_FILE), &linkage)
}

fn write_release_inputs(proof_root: &Path) -> Result<(), RunError> {
    let inputs = ReleaseEvidenceInputs {
        schema: RELEASE_INPUTS_SCHEMA,
        deterministic_proof: crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE,
        provider_kind_linkage: crate::source_built_fixed_point_shell::PROVIDER_KIND_LINKAGE_FILE,
        source_built_plan: PLAN_FILE,
        toolchain_closure: TOOLCHAIN_CLOSURE_FILE,
        proof_bundle_digest: crate::source_built_fixed_point_shell::FINAL_BUNDLE_DIGEST_FILE,
    };
    write_json_create_new(&proof_root.join(crate::source_built_fixed_point_shell::RELEASE_INPUTS_FILE), &inputs)
}

fn write_extended_receipt(
    path: &Path,
    core: &DeterministicBuildProofReceipt,
    extension: &SourceBuiltReceiptExtension,
) -> Result<(), RunError> {
    let mut value = serde_json::to_value(core)
        .map_err(|error| receipt_error(format!("serializing deterministic receipt: {error}")))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| receipt_error("deterministic receipt is not a JSON object".to_string()))?;
    object.insert(
        EXTENSION_FIELD.to_string(),
        serde_json::to_value(extension)
            .map_err(|error| receipt_error(format!("serializing source-built extension: {error}")))?,
    );
    write_json_create_new(path, &value)
}

fn proof_bundle_digest(proof_root: &Path) -> Result<ProofBundleObservation, RunError> {
    proof_bundle_digest_with_limit(proof_root, PROOF_ENTRY_COUNT_MAX)
}

fn proof_bundle_digest_with_limit(
    proof_root: &Path,
    entries_count_max: usize,
) -> Result<ProofBundleObservation, RunError> {
    if entries_count_max == 0 {
        return Err(receipt_error("proof bundle entry limit must be positive".to_string()));
    }
    let mut entries = Vec::new();
    collect_bundle_files(proof_root, proof_root, entries_count_max, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.is_empty() || entries.len() > entries_count_max {
        return Err(receipt_error(format!(
            "proof bundle entry count must be within 1..={entries_count_max}, got {}",
            entries.len()
        )));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(RECEIPT_BUNDLE_DIGEST_DOMAIN);
    for (relative, path, symlink_target) in &entries {
        hasher.update(relative.as_bytes());
        hasher.update(&[0]);
        if let Some(target) = symlink_target {
            hasher.update(b"symlink\0");
            hasher.update(target.as_os_str().as_encoded_bytes());
        } else {
            hasher.update(b"file\0");
            hasher.update(hash_file_allow_empty(path)?.as_bytes());
        }
        hasher.update(&[0]);
    }
    let digest_blake3 = hasher.finalize().to_hex().to_string();
    assert_eq!(digest_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!entries.is_empty());
    Ok(ProofBundleObservation {
        digest_blake3,
        entry_count: entries.len(),
    })
}

fn collect_bundle_files(
    proof_root: &Path,
    current: &Path,
    entries_count_max: usize,
    entries: &mut Vec<(String, PathBuf, Option<PathBuf>)>,
) -> Result<(), RunError> {
    let mut children = fs::read_dir(current)
        .map_err(|error| receipt_error(format!("reading proof bundle directory {}: {error}", current.display())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| receipt_error(format!("reading proof bundle entry: {error}")))?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| receipt_error(format!("reading proof bundle metadata {}: {error}", path.display())))?;
        if metadata.file_type().is_symlink() {
            let relative = relative_path(proof_root, &path)?;
            if !excluded_from_bundle_digest(&relative) {
                let target = fs::read_link(&path).map_err(|error| {
                    receipt_error(format!("reading proof bundle symlink {}: {error}", path.display()))
                })?;
                push_bundle_entry(entries, entries_count_max, (relative, path, Some(target)))?;
            }
            continue;
        }
        if metadata.is_dir() {
            let relative = relative_path(proof_root, &path)?;
            if excluded_directory_from_bundle_digest(&relative) {
                continue;
            }
            collect_bundle_files(proof_root, &path, entries_count_max, entries)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(receipt_error(format!("proof bundle contains unsupported entry: {}", path.display())));
        }
        let relative = relative_path(proof_root, &path)?;
        if excluded_from_bundle_digest(&relative) {
            continue;
        }
        push_bundle_entry(entries, entries_count_max, (relative, path, None))?;
    }
    assert!(entries.len() <= entries_count_max);
    debug_assert!(current.starts_with(proof_root));
    Ok(())
}

fn push_bundle_entry(
    entries: &mut Vec<(String, PathBuf, Option<PathBuf>)>,
    entries_count_max: usize,
    entry: (String, PathBuf, Option<PathBuf>),
) -> Result<(), RunError> {
    if entries.len() >= entries_count_max {
        return Err(receipt_error(format!("proof bundle entry count exceeds {entries_count_max}")));
    }
    entries.push(entry);
    assert!(entries.len() <= entries_count_max);
    debug_assert!(!entries.is_empty());
    Ok(())
}

fn excluded_directory_from_bundle_digest(relative: &str) -> bool {
    NON_DURABLE_PROOF_DIRECTORIES.contains(&relative)
}

fn excluded_from_bundle_digest(relative: &str) -> bool {
    matches!(
        relative,
        crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE
            | crate::source_built_fixed_point_shell::FINAL_BUNDLE_DIGEST_FILE
            | "attempt-status.json"
    )
}

fn hash_preserved_stagex_transition_tree(
    root: &Path,
    total_file_bytes_max: u64,
) -> Result<crate::preserved_evidence_tree::PreservedEvidenceTreeIdentity, RunError> {
    let entries_count_max = u32::try_from(PROOF_ENTRY_COUNT_MAX)
        .map_err(|_| receipt_error("proof entry limit does not fit u32".to_string()))?;
    let release_limits = crunch_release_core::TreeCopyLimits::RELEASE_BUNDLE;
    let limits = crate::preserved_evidence_tree::PreservedEvidenceTreeLimits {
        entries_count_max,
        depth_count_max: release_limits.depth_count_max,
        path_bytes_max: release_limits.path_bytes_max,
        symlink_target_bytes_max: release_limits.path_bytes_max,
        total_file_bytes_max,
    };
    let identity = crate::preserved_evidence_tree::hash_preserved_evidence_tree(root, limits).map_err(|error| {
        receipt_error(format!("hashing preserved StageX transition tree {}: {error}", root.display()))
    })?;
    assert!(identity.entry_count <= entries_count_max);
    assert!(identity.total_file_bytes <= total_file_bytes_max);
    Ok(identity)
}

fn hash_tree(root: &Path) -> Result<(u64, String), RunError> {
    crate::release_tree_copy::hash_directory_tree(root)
        .map_err(|error| receipt_error(format!("hashing directory tree {}: {error}", root.display())))
}

fn hash_file(path: &Path) -> Result<String, RunError> {
    let metadata = fs::metadata(path)
        .map_err(|error| receipt_error(format!("reading file metadata {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(receipt_error(format!("required receipt file is missing or empty: {}", path.display())));
    }
    hash_file_allow_empty(path)
}

fn hash_file_allow_empty(path: &Path) -> Result<String, RunError> {
    let mut file =
        fs::File::open(path).map_err(|error| receipt_error(format!("opening file {}: {error}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; HASH_BUFFER_BYTES];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| receipt_error(format!("reading file {}: {error}", path.display())))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(path.is_file());
    Ok(digest)
}

fn digest_string_set(domain: &[u8], values: &[String]) -> String {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted.dedup();
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for value in sorted {
        hasher.update(value.as_bytes());
        hasher.update(&[0]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!values.is_empty());
    digest
}

fn validate_digest(label: &str, digest: &str) -> Result<(), RunError> {
    let valid = digest.len() == BLAKE3_HEX_LENGTH
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(receipt_error(format!("{label} is not a lowercase BLAKE3 digest")));
    }
    assert!(!label.is_empty());
    debug_assert!(valid);
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, RunError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| receipt_error(format!("path {} is outside proof root {}", path.display(), root.display())))?;
    let text = relative
        .to_str()
        .ok_or_else(|| receipt_error(format!("proof-relative path is not UTF-8: {}", relative.display())))?;
    if text.is_empty() || text.starts_with('/') || text.split('/').any(|component| component == "..") {
        return Err(receipt_error(format!("invalid proof-relative path: {text}")));
    }
    assert!(!text.is_empty());
    debug_assert!(!Path::new(text).is_absolute());
    Ok(text.to_string())
}

fn file_size(path: &Path) -> Result<u64, RunError> {
    let size = fs::metadata(path)
        .map_err(|error| receipt_error(format!("reading file size {}: {error}", path.display())))?
        .len();
    if size == 0 {
        return Err(receipt_error(format!("required file is empty: {}", path.display())));
    }
    assert!(size > 0);
    debug_assert!(path.is_file());
    Ok(size)
}

fn write_json_create_new<T: Serialize>(path: &Path, value: &T) -> Result<(), RunError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| receipt_error(format!("serializing {}: {error}", path.display())))?;
    bytes.push(b'\n');
    write_bytes_create_new(path, &bytes)
}

fn write_text_create_new(path: &Path, text: &str) -> Result<(), RunError> {
    write_bytes_create_new(path, text.as_bytes())
}

fn write_bytes_create_new(path: &Path, bytes: &[u8]) -> Result<(), RunError> {
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|error| receipt_error(format!("creating {}: {error}", path.display())))?;
    file.write_all(bytes)
        .map_err(|error| receipt_error(format!("writing {}: {error}", path.display())))?;
    file.sync_all().map_err(|error| receipt_error(format!("syncing {}: {error}", path.display())))?;
    assert!(path.is_file());
    debug_assert_eq!(fs::metadata(path).ok().map(|metadata| metadata.len()), Some(bytes.len() as u64));
    Ok(())
}

fn receipt_error(message: String) -> RunError {
    RunError::Build(format!("source-built fixed-point receipt blocked: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TOO_SMALL_BUNDLE_ENTRY_LIMIT: usize = 1;
    #[cfg(unix)]
    const NO_ACCESS_DIRECTORY_MODE: u32 = 0o000;
    #[cfg(unix)]
    const RESTORED_DIRECTORY_MODE: u32 = 0o700;

    #[test]
    fn rebuild_source_count_tracks_the_plan_contract_and_rejects_drift() {
        let expected = usize::try_from(SOURCE_BUILT_FIXED_POINT_REQUIRED_SOURCE_ROLE_COUNT).unwrap();
        let missing = expected.checked_sub(1).unwrap();
        let extra = expected.checked_add(1).unwrap();

        validate_rebuild_source_input_count(expected).unwrap();
        let missing_error = validate_rebuild_source_input_count(missing).unwrap_err();
        let extra_error = validate_rebuild_source_input_count(extra).unwrap_err();

        assert!(missing_error.to_string().contains(&format!("expected {expected}, got {missing}")));
        assert!(extra_error.to_string().contains(&format!("expected {expected}, got {extra}")));
        assert_ne!(missing, expected);
        assert_ne!(extra, expected);
    }

    #[test]
    fn stage_evidence_ids_match_the_plan_and_reject_stale_aliases() {
        let planned = crate::source_built_fixed_point::expected_stage_plans();
        let mut evidence = planned
            .iter()
            .map(|stage| {
                let mut evidence = test_stage_evidence(stage.output);
                evidence.stage_id = canonical_stage_id(stage.output).to_string();
                evidence
            })
            .collect::<Vec<_>>();

        validate_stage_plan_entries(&planned, &evidence).unwrap();
        evidence[1].stage_id = "stagex-provider".to_string();
        let stagex_error = validate_stage_plan_entries(&planned, &evidence).unwrap_err();
        evidence[1].stage_id = canonical_stage_id(ProofOutputRole::StagexProvider).to_string();
        evidence[4].stage_id = STAGE1_RUN_ID.to_string();
        let stage1_error = validate_stage_plan_entries(&planned, &evidence).unwrap_err();

        assert!(stagex_error.to_string().contains("planned stagex-provider-publication"));
        assert!(stage1_error.to_string().contains("planned mantle-stage1"));
        assert_ne!(canonical_stage_id(ProofOutputRole::StagexProvider), "stagex-provider");
        assert_ne!(canonical_stage_id(ProofOutputRole::MantleStage1), STAGE1_RUN_ID);
    }

    #[test]
    fn stage_evidence_rejects_fallbacks_and_duplicate_ids() {
        let mut evidence = (0..STAGE_EVIDENCE_COUNT)
            .map(|index| SourceBuiltStageEvidence {
                stage_id: format!("stage-{index}"),
                output_role: ProofOutputRole::StagexTransition,
                status: STAGE_STATUS_COMPLETE.to_string(),
                execution_origin: StageExecutionOrigin::Executed,
                checkpoint_digest_blake3: None,
                original_execution_evidence_digest_blake3: None,
                orchestrator: "host-mantle".to_string(),
                executable_identity: format!("blake3:{DIGEST_A}"),
                transcript_path: format!("stage-{index}.json"),
                transcript_digest_blake3: DIGEST_A.to_string(),
                audit_paths: vec![format!("audit-{index}.json")],
                audit_digests_blake3: vec![DIGEST_B.to_string()],
                output_path: format!("output-{index}"),
                output_digest_blake3: DIGEST_B.to_string(),
                authority_violations: Vec::new(),
                fallback_events: Vec::new(),
            })
            .collect::<Vec<_>>();
        validate_stage_evidence(&evidence).unwrap();
        evidence[0].fallback_events.push("host-tool-fallback".to_string());
        let fallback = validate_stage_evidence(&evidence).unwrap_err();
        evidence[0].fallback_events.clear();
        evidence[1].stage_id = evidence[0].stage_id.clone();
        let duplicate = validate_stage_evidence(&evidence).unwrap_err();

        assert!(fallback.to_string().contains("incomplete"));
        assert!(duplicate.to_string().contains("duplicate stage evidence id"));
    }

    // r[verify bootstrap_inventory.source_built_mantle_checkpoint_reuse]
    #[test]
    fn restored_stage_origin_requires_provider_role_and_complete_checkpoint_link() {
        let mut restored = test_stage_evidence(ProofOutputRole::StagexProvider);
        restored.execution_origin = StageExecutionOrigin::RestoredCheckpoint;
        restored.checkpoint_digest_blake3 = Some(DIGEST_A.to_string());
        restored.original_execution_evidence_digest_blake3 = Some(DIGEST_B.to_string());
        validate_stage_execution_origin(&restored).unwrap();

        restored.checkpoint_digest_blake3 = None;
        let missing = validate_stage_execution_origin(&restored).unwrap_err();
        restored.checkpoint_digest_blake3 = Some(DIGEST_A.to_string());
        restored.output_role = ProofOutputRole::MantleStage1;
        let wrong_stage = validate_stage_execution_origin(&restored).unwrap_err();
        restored.execution_origin = StageExecutionOrigin::Executed;
        let false_execution = validate_stage_execution_origin(&restored).unwrap_err();

        assert!(missing.to_string().contains("no checkpoint digest"));
        assert!(wrong_stage.to_string().contains("non-provider stage"));
        assert!(false_execution.to_string().contains("checkpoint-only evidence"));
    }

    fn test_stage_evidence(output_role: ProofOutputRole) -> SourceBuiltStageEvidence {
        SourceBuiltStageEvidence {
            stage_id: "checkpoint-stage".to_string(),
            output_role,
            status: STAGE_STATUS_COMPLETE.to_string(),
            execution_origin: StageExecutionOrigin::Executed,
            checkpoint_digest_blake3: None,
            original_execution_evidence_digest_blake3: None,
            orchestrator: "host-mantle".to_string(),
            executable_identity: format!("blake3:{DIGEST_A}"),
            transcript_path: "transcript.json".to_string(),
            transcript_digest_blake3: DIGEST_A.to_string(),
            audit_paths: vec!["audit.json".to_string()],
            audit_digests_blake3: vec![DIGEST_B.to_string()],
            output_path: "output".to_string(),
            output_digest_blake3: DIGEST_B.to_string(),
            authority_violations: Vec::new(),
            fallback_events: Vec::new(),
        }
    }

    // r[verify bootstrap_inventory.source_built_mantle_fixed_point]
    #[test]
    fn bundle_digest_excludes_only_self_referential_receipt_files() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("evidence.txt"), "evidence").unwrap();
        fs::write(temp.path().join("second-evidence.txt"), "second").unwrap();
        fs::write(temp.path().join("empty-evidence.txt"), "").unwrap();
        fs::create_dir(temp.path().join("rust-provider-scratch")).unwrap();
        fs::write(temp.path().join("rust-provider-scratch/ignored.txt"), "ignored").unwrap();
        fs::write(temp.path().join(crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE), "one").unwrap();
        fs::write(temp.path().join(crate::source_built_fixed_point_shell::FINAL_BUNDLE_DIGEST_FILE), "two").unwrap();
        fs::write(temp.path().join("attempt-status.json"), "running").unwrap();
        let first = proof_bundle_digest(temp.path()).unwrap();
        fs::write(temp.path().join(crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE), "changed").unwrap();
        fs::write(temp.path().join("attempt-status.json"), "complete").unwrap();
        fs::write(temp.path().join("rust-provider-scratch/ignored.txt"), "changed ignored scratch").unwrap();
        let second = proof_bundle_digest(temp.path()).unwrap();
        fs::write(temp.path().join("evidence.txt"), "changed evidence").unwrap();
        let changed = proof_bundle_digest(temp.path()).unwrap();
        let limit_error = proof_bundle_digest_with_limit(temp.path(), TOO_SMALL_BUNDLE_ENTRY_LIMIT).unwrap_err();
        let limit_message = format!("entry count exceeds {TOO_SMALL_BUNDLE_ENTRY_LIMIT}");

        assert_eq!(first.digest_blake3, second.digest_blake3);
        assert_ne!(second.digest_blake3, changed.digest_blake3);
        assert!(limit_error.to_string().contains(&limit_message));
    }

    // r[verify bootstrap_inventory.source_built_mantle_fixed_point]
    #[test]
    #[cfg(unix)]
    fn bundle_digest_skips_declared_unreadable_scratch_but_rejects_unknown_unreadable_content() {
        use std::os::unix::fs::PermissionsExt as _;

        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("evidence.txt"), "evidence").unwrap();
        let declared_scratch = temp.path().join("tmp");
        let unknown_content = temp.path().join("unknown-content");
        fs::create_dir(&declared_scratch).unwrap();
        fs::create_dir(&unknown_content).unwrap();
        fs::set_permissions(&declared_scratch, fs::Permissions::from_mode(NO_ACCESS_DIRECTORY_MODE)).unwrap();

        let accepted = proof_bundle_digest(temp.path()).unwrap();
        fs::set_permissions(&unknown_content, fs::Permissions::from_mode(NO_ACCESS_DIRECTORY_MODE)).unwrap();
        let rejected = proof_bundle_digest(temp.path()).unwrap_err();
        fs::set_permissions(&declared_scratch, fs::Permissions::from_mode(RESTORED_DIRECTORY_MODE)).unwrap();
        fs::set_permissions(&unknown_content, fs::Permissions::from_mode(RESTORED_DIRECTORY_MODE)).unwrap();

        assert_eq!(accepted.entry_count, TOO_SMALL_BUNDLE_ENTRY_LIMIT);
        assert_eq!(accepted.digest_blake3.len(), BLAKE3_HEX_LENGTH);
        assert!(rejected.to_string().contains("unknown-content"));
    }

    #[test]
    fn provider_identity_digest_binds_each_declared_role() {
        let first = provider_identity_digest(DIGEST_A, DIGEST_B, DIGEST_A);
        let repeated = provider_identity_digest(DIGEST_A, DIGEST_B, DIGEST_A);
        let swapped = provider_identity_digest(DIGEST_B, DIGEST_A, DIGEST_A);
        let substituted = provider_identity_digest(DIGEST_A, DIGEST_B, DIGEST_B);

        assert_eq!(first, repeated);
        assert_ne!(first, swapped);
        assert_ne!(first, substituted);
        assert_eq!(first.len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    #[ignore = "requires a preserved source-built proof bundle"]
    fn preserved_proof_bundle_fixture_fits_receipt_entry_policy() {
        const PROOF_ROOT_ENV: &str = "MANTLE_TEST_PRESERVED_PROOF_ROOT";

        let root = PathBuf::from(std::env::var_os(PROOF_ROOT_ENV).expect("preserved proof root must be set"));
        let observation = proof_bundle_digest(&root).unwrap();
        println!("preserved-proof-bundle: entries={} blake3={}", observation.entry_count, observation.digest_blake3);

        let release_entry_limit = usize::try_from(crunch_release_core::RELEASE_TREE_COPY_MAX_ENTRIES_COUNT).unwrap();
        assert!(root.is_dir());
        assert!(root.join("rust-provider-scratch").is_dir());
        assert!(observation.entry_count > release_entry_limit);
        assert!(observation.entry_count <= PROOF_ENTRY_COUNT_MAX);
        assert_eq!(observation.digest_blake3.len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    #[ignore = "requires a preserved source-built StageX transition tree"]
    fn preserved_stagex_transition_tree_fixture_hashes_under_receipt_policy() {
        const TREE_PATH_ENV: &str = "MANTLE_TEST_PRESERVED_STAGEX_TREE";
        const TOTAL_FILE_BYTES_MAX_ENV: &str = "MANTLE_TEST_PRESERVED_STAGEX_TREE_BYTES_MAX";

        let root = PathBuf::from(std::env::var_os(TREE_PATH_ENV).expect("preserved StageX tree path must be set"));
        let total_file_bytes_max = std::env::var(TOTAL_FILE_BYTES_MAX_ENV)
            .expect("preserved StageX tree byte bound must be set")
            .parse::<u64>()
            .expect("preserved StageX tree byte bound must be a u64");
        let identity = hash_preserved_stagex_transition_tree(&root, total_file_bytes_max).unwrap();
        println!(
            "preserved-stagex-tree: entries={} bytes={} blake3={}",
            identity.entry_count, identity.total_file_bytes, identity.digest_blake3
        );

        assert!(root.is_dir());
        assert!(identity.entry_count > crunch_release_core::RELEASE_TREE_COPY_MAX_ENTRIES_COUNT);
        assert!(identity.total_file_bytes <= total_file_bytes_max);
        assert_eq!(identity.digest_blake3.len(), BLAKE3_HEX_LENGTH);
    }
}
