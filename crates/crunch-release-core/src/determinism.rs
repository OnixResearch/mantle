use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ContentBoundRebuildDescriptor;
use crate::RebuildAuthorityPlan;
use crate::ReleaseEvidenceError;
use crate::content_bound_rebuild_descriptor_digest_blake3;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;
use crate::rebuild_authority_plan_digest_blake3;

pub const LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA: &str = "mantle-deterministic-proof-receipt-v1";
pub const DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA: &str = "mantle-deterministic-proof-receipt-v2";
pub const BUILD_EFFECT_POLICY_VERSION: &str = "mantle-build-effects-v1";
pub const DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA: &str = "mantle-deterministic-sandbox-isolation-evidence-v1";
const REQUIRED_RUN_COUNT: usize = 2;
pub const SUPPORTED_SANDBOX_PROFILE_FAMILY: &str = "mantle-proof-sandbox-v1";
pub const REQUIRED_ISOLATION_CHECKS: &[&str] = &[
    "denies-undeclared-host-access",
    "denies-host-network-by-default",
    "denies-main-output-and-proof-store-reuse",
    "denies-clock-syscalls",
];
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
const REQUIRED_NORMALIZATION_CONTROLS: &[&str] = &[
    "time",
    "timezone",
    "locale",
    "temp-roots",
    "host-user-metadata",
    "umask",
    "modeled-randomness",
    "order-sensitive-output-processing",
];
const NORMALIZATION_CONTROL_PREFIX: &str = "normalization:";
const UNSUPPORTED_NORMALIZATION_PREFIX: &str = "normalization-unsupported:";
const DETERMINISTIC_PROOF_WORKFLOW: &str = "deterministic-release";
const DETERMINISTIC_PROOF_CLAIM: &str = "deterministic-release";
const SUPPORTED_SANDBOX_PROFILE_PREFIX: &str = "mantle-proof-sandbox-v1:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildEffect {
    ReadStore,
    WriteOutput,
    Network,
    Clock,
    Random,
    Environment,
    Secret,
    HostTool,
    RemoteBuild,
}

pub const PURE_LOCAL_BUILD_EFFECTS: &[BuildEffect] = &[
    BuildEffect::ReadStore,
    BuildEffect::WriteOutput,
    BuildEffect::Environment,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeterministicBuildProofVerdict {
    NotAttempted,
    SelfRebuildMatch,
    Mismatch,
    MissingEvidence,
    ReusedStore,
    ImpureMode,
    UnsupportedWorkflow,
    UnsupportedSandbox,
    ProviderKindMismatch,
    MissingGenuineRebuildEvidence,
    TargetAuthorityViolation,
    MalformedReceipt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeterministicSandboxIsolationEvidenceStatus {
    Passed,
    Failed,
    Bypassed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicSandboxIsolationEvidence {
    pub schema: String,
    pub profile_family: String,
    pub evidence_version: String,
    pub status: DeterministicSandboxIsolationEvidenceStatus,
    pub checks: Vec<String>,
    pub evidence_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicProofUnit {
    pub target_artifact_identity: String,
    pub output_identities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicOutputDigest {
    pub name: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicBuildRunReceipt {
    pub run_id: String,
    pub perturbation_case: String,
    pub output_store_paths: Vec<String>,
    pub output_root_identity: String,
    pub sandbox_profile_identity: String,
    pub output_digests: Vec<DeterministicOutputDigest>,
    pub substituted_dependency_identities: Vec<String>,
    pub hermeticity_audit_events: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_effects: Option<Vec<BuildEffect>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuild_descriptor_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuild_authority_plan_blake3: Option<String>,
    #[serde(default)]
    pub observed_read_identities: Vec<String>,
    #[serde(default)]
    pub authority_violations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicBuildProofReceipt {
    pub schema: String,
    pub proof_unit: DeterministicProofUnit,
    pub derivation_identity: String,
    pub hermeticity_mode: String,
    pub workflow_version: String,
    pub selected_provider_kind: String,
    pub source_blake3: String,
    pub vendor_blake3: String,
    pub toolchain_provider_identity: String,
    pub toolchain_stage_roots: Vec<String>,
    pub logical_store_prefix: String,
    pub physical_store_isolation: String,
    pub effect_policy_version: String,
    pub declared_effects: Vec<BuildEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_effects: Option<Vec<BuildEffect>>,
    pub normalized_execution_envelope: Vec<String>,
    pub ambient_host_perturbations: Vec<String>,
    pub sandbox_profile_identities: Vec<String>,
    pub runs: Vec<DeterministicBuildRunReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuild_descriptor: Option<ContentBoundRebuildDescriptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuild_descriptor_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuild_authority_plan: Option<RebuildAuthorityPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rebuild_authority_plan_blake3: Option<String>,
    pub verdict: DeterministicBuildProofVerdict,
    pub blocking_reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicBuildProofReceiptInit {
    pub proof_unit: DeterministicProofUnit,
    pub derivation_identity: String,
    pub hermeticity_mode: String,
    pub workflow_version: String,
    pub selected_provider_kind: String,
    pub source_blake3: String,
    pub vendor_blake3: String,
    pub toolchain_provider_identity: String,
    pub toolchain_stage_roots: Vec<String>,
    pub logical_store_prefix: String,
    pub physical_store_isolation: String,
    pub effect_policy_version: String,
    pub declared_effects: Vec<BuildEffect>,
    pub observed_effects: Option<Vec<BuildEffect>>,
    pub normalized_execution_envelope: Vec<String>,
    pub ambient_host_perturbations: Vec<String>,
    pub sandbox_profile_identities: Vec<String>,
    pub runs: Vec<DeterministicBuildRunReceipt>,
    pub rebuild_descriptor: ContentBoundRebuildDescriptor,
    pub rebuild_descriptor_blake3: String,
    pub rebuild_authority_plan: RebuildAuthorityPlan,
    pub rebuild_authority_plan_blake3: String,
}

impl DeterministicBuildProofReceipt {
    pub fn new(init: DeterministicBuildProofReceiptInit) -> Self {
        let mut receipt = Self {
            schema: DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string(),
            proof_unit: init.proof_unit,
            derivation_identity: init.derivation_identity,
            hermeticity_mode: init.hermeticity_mode,
            workflow_version: init.workflow_version,
            selected_provider_kind: init.selected_provider_kind,
            source_blake3: init.source_blake3,
            vendor_blake3: init.vendor_blake3,
            toolchain_provider_identity: init.toolchain_provider_identity,
            toolchain_stage_roots: init.toolchain_stage_roots,
            logical_store_prefix: init.logical_store_prefix,
            physical_store_isolation: init.physical_store_isolation,
            effect_policy_version: init.effect_policy_version,
            declared_effects: init.declared_effects,
            observed_effects: init.observed_effects,
            normalized_execution_envelope: init.normalized_execution_envelope,
            ambient_host_perturbations: init.ambient_host_perturbations,
            sandbox_profile_identities: init.sandbox_profile_identities,
            runs: init.runs,
            rebuild_descriptor: Some(init.rebuild_descriptor),
            rebuild_descriptor_blake3: Some(init.rebuild_descriptor_blake3),
            rebuild_authority_plan: Some(init.rebuild_authority_plan),
            rebuild_authority_plan_blake3: Some(init.rebuild_authority_plan_blake3),
            verdict: DeterministicBuildProofVerdict::NotAttempted,
            blocking_reasons: Vec::new(),
            receipt_blake3: None,
        };
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
        receipt.verdict = verdict;
        receipt.blocking_reasons = reasons;
        receipt
    }
}

pub fn canonical_deterministic_build_proof_receipt(
    mut receipt: DeterministicBuildProofReceipt,
) -> Result<DeterministicBuildProofReceipt, ReleaseEvidenceError> {
    let provided_receipt_blake3 = receipt.receipt_blake3.take();
    validate_receipt_header(&receipt)?;
    let legacy_receipt = receipt.schema == LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA;
    if !legacy_receipt {
        receipt.schema = DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string();
    }
    receipt.declared_effects.sort();
    receipt.declared_effects.dedup();
    if let Some(observed_effects) = &mut receipt.observed_effects {
        observed_effects.sort();
        observed_effects.dedup();
    }
    receipt.normalized_execution_envelope.sort();
    receipt.ambient_host_perturbations.sort();
    receipt.sandbox_profile_identities.sort();
    receipt.toolchain_stage_roots.sort();
    receipt.proof_unit.output_identities.sort();
    receipt.blocking_reasons.sort();
    for run in &mut receipt.runs {
        run.output_store_paths.sort();
        run.substituted_dependency_identities.sort();
        run.hermeticity_audit_events.sort();
        run.observed_read_identities.sort();
        run.observed_read_identities.dedup();
        run.authority_violations.sort();
        run.authority_violations.dedup();
        if let Some(observed_effects) = &mut run.observed_effects {
            observed_effects.sort();
            observed_effects.dedup();
        }
        run.output_digests.sort_by(|left, right| left.name.cmp(&right.name));
    }
    receipt.runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    validate_receipt_evidence(&receipt)?;
    let (expected_verdict, expected_reasons) = classify_deterministic_build_proof(&receipt);
    if !legacy_receipt && receipt.verdict != expected_verdict {
        return Err(validation_error(format!(
            "deterministic build proof receipt verdict {:?} does not match classified verdict {:?}",
            receipt.verdict, expected_verdict
        )));
    }
    let mut sorted_expected_reasons = expected_reasons;
    sorted_expected_reasons.sort();
    if !legacy_receipt && receipt.blocking_reasons != sorted_expected_reasons {
        return Err(validation_error(
            "deterministic build proof receipt blocking_reasons do not match classified reasons".to_string(),
        ));
    }
    if let Some(provided) = provided_receipt_blake3 {
        let digest = blake3::hash(&serde_json::to_vec(&receipt).map_err(|err| {
            ReleaseEvidenceError::Parse(format!("serializing deterministic build proof receipt: {err}"))
        })?)
        .to_hex()
        .to_string();
        if provided != digest {
            return Err(validation_error(
                "deterministic build proof receipt receipt_blake3 does not match canonical receipt bytes".to_string(),
            ));
        }
        receipt.receipt_blake3 = Some(provided);
    }
    Ok(receipt)
}

pub fn deterministic_build_proof_receipt_canonical_bytes(
    receipt: DeterministicBuildProofReceipt,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let mut canonical = canonical_deterministic_build_proof_receipt(receipt)?;
    canonical.receipt_blake3 = None;
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing deterministic build proof receipt: {err}")))
}

pub fn deterministic_build_proof_receipt_digest_blake3(
    receipt: DeterministicBuildProofReceipt,
) -> Result<String, ReleaseEvidenceError> {
    let mut receipt = canonical_deterministic_build_proof_receipt(receipt)?;
    receipt.receipt_blake3 = None;
    let bytes = serde_json::to_vec(&receipt)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing deterministic build proof receipt: {err}")))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

pub fn validate_deterministic_sandbox_isolation_evidence(
    evidence: &DeterministicSandboxIsolationEvidence,
) -> Result<(), ReleaseEvidenceError> {
    if evidence.schema != DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA {
        return Err(validation_error(format!(
            "deterministic sandbox isolation evidence schema must be {DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA}, got {}",
            evidence.schema
        )));
    }
    if evidence.profile_family != SUPPORTED_SANDBOX_PROFILE_FAMILY {
        return Err(validation_error(format!(
            "deterministic sandbox isolation evidence profile_family must be {SUPPORTED_SANDBOX_PROFILE_FAMILY}, got {}",
            evidence.profile_family
        )));
    }
    if evidence.evidence_version.trim().is_empty() {
        return Err(validation_error(
            "deterministic sandbox isolation evidence evidence_version must not be empty".to_string(),
        ));
    }
    if evidence.status != DeterministicSandboxIsolationEvidenceStatus::Passed {
        return Err(validation_error(format!(
            "deterministic sandbox isolation evidence status must be passed, got {:?}",
            evidence.status
        )));
    }
    validate_string_set(&evidence.checks, "isolation_evidence.checks")?;
    for required in REQUIRED_ISOLATION_CHECKS {
        if !evidence.checks.iter().any(|actual| actual == required) {
            return Err(validation_error(format!(
                "deterministic sandbox isolation evidence missing required check {required}"
            )));
        }
    }
    validate_blake3_hex(&evidence.evidence_digest_blake3, "deterministic sandbox isolation evidence digest_blake3")?;
    Ok(())
}

pub fn deterministic_sandbox_isolation_evidence_canonical_bytes(
    mut evidence: DeterministicSandboxIsolationEvidence,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    evidence.checks.sort();
    validate_deterministic_sandbox_isolation_evidence(&evidence)?;
    serde_json::to_vec(&evidence).map_err(|err| {
        ReleaseEvidenceError::Parse(format!("serializing deterministic sandbox isolation evidence: {err}"))
    })
}

pub fn deterministic_sandbox_isolation_evidence_digest_blake3(
    evidence: DeterministicSandboxIsolationEvidence,
) -> Result<String, ReleaseEvidenceError> {
    let bytes = deterministic_sandbox_isolation_evidence_canonical_bytes(evidence)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

pub fn deterministic_release_claim_eligible(
    release_digest_set: &[String],
    receipts: &[DeterministicBuildProofReceipt],
    isolation_evidence: Option<&DeterministicSandboxIsolationEvidence>,
) -> Result<bool, ReleaseEvidenceError> {
    let Some(isolation_evidence) = isolation_evidence else {
        return Ok(false);
    };
    validate_deterministic_sandbox_isolation_evidence(isolation_evidence)?;
    let mut release_digests = canonical_digest_set(release_digest_set, "release_digest_set")?;
    release_digests.sort();
    let mut receipt_digests = Vec::new();
    for receipt in receipts {
        let receipt = canonical_deterministic_build_proof_receipt(receipt.clone())?;
        if !deterministic_build_proof_has_genuine_rebuild_authority(receipt.clone())? {
            return Ok(false);
        }
        for profile in &receipt.sandbox_profile_identities {
            if !profile.starts_with(SUPPORTED_SANDBOX_PROFILE_PREFIX) {
                return Err(validation_error(format!(
                    "deterministic build proof receipt sandbox profile {profile} must use supported family {SUPPORTED_SANDBOX_PROFILE_FAMILY}"
                )));
            }
        }
        for run in &receipt.runs {
            if !run.sandbox_profile_identity.starts_with(SUPPORTED_SANDBOX_PROFILE_PREFIX) {
                return Err(validation_error(format!(
                    "deterministic build proof run {} sandbox profile {} must use supported family {SUPPORTED_SANDBOX_PROFILE_FAMILY}",
                    run.run_id, run.sandbox_profile_identity
                )));
            }
        }
        if receipt.verdict != DeterministicBuildProofVerdict::SelfRebuildMatch {
            return Ok(false);
        }
        for digest in receipt.runs.first().into_iter().flat_map(|run| &run.output_digests) {
            receipt_digests.push(digest.digest_blake3.clone());
        }
    }
    receipt_digests.sort();
    receipt_digests.dedup();
    Ok(!release_digests.is_empty() && release_digests == receipt_digests)
}

// r[impl mantle.release_provenance.deterministic_rebuild_admission.contract]
// r[impl mantle.release_provenance.deterministic_rebuild_admission.validation]
pub fn deterministic_build_proof_has_genuine_rebuild_authority(
    receipt: DeterministicBuildProofReceipt,
) -> Result<bool, ReleaseEvidenceError> {
    validate_receipt_header(&receipt)?;
    if receipt.schema != DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA {
        return Ok(false);
    }
    Ok(genuine_rebuild_admission_reasons(&receipt).is_empty())
}

fn validate_receipt_header(receipt: &DeterministicBuildProofReceipt) -> Result<(), ReleaseEvidenceError> {
    let supported_schema = receipt.schema == DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA
        || receipt.schema == LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA;
    if !supported_schema {
        return Err(validation_error(format!(
            "deterministic build proof receipt schema must be {DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA} or diagnostic-only {LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA}, got {}",
            receipt.schema
        )));
    }
    for (field, value) in [
        ("proof_unit.target_artifact_identity", &receipt.proof_unit.target_artifact_identity),
        ("derivation_identity", &receipt.derivation_identity),
        ("hermeticity_mode", &receipt.hermeticity_mode),
        ("workflow_version", &receipt.workflow_version),
        ("selected_provider_kind", &receipt.selected_provider_kind),
        ("source_blake3", &receipt.source_blake3),
        ("vendor_blake3", &receipt.vendor_blake3),
        ("toolchain_provider_identity", &receipt.toolchain_provider_identity),
        ("logical_store_prefix", &receipt.logical_store_prefix),
        ("physical_store_isolation", &receipt.physical_store_isolation),
    ] {
        if value.trim().is_empty() {
            return Err(validation_error(format!("deterministic build proof receipt {field} must not be empty")));
        }
    }
    validate_blake3_hex(&receipt.source_blake3, "deterministic build proof receipt source_blake3")?;
    validate_blake3_hex(&receipt.vendor_blake3, "deterministic build proof receipt vendor_blake3")?;
    Ok(())
}

fn validate_receipt_evidence(receipt: &DeterministicBuildProofReceipt) -> Result<(), ReleaseEvidenceError> {
    validate_string_set(&receipt.proof_unit.output_identities, "proof_unit.output_identities")?;
    validate_string_set(&receipt.toolchain_stage_roots, "toolchain_stage_roots")?;
    validate_build_effect_policy(receipt)?;
    validate_string_set(&receipt.normalized_execution_envelope, "normalized_execution_envelope")?;
    validate_string_set(&receipt.ambient_host_perturbations, "ambient_host_perturbations")?;
    validate_string_set(&receipt.sandbox_profile_identities, "sandbox_profile_identities")?;
    validate_string_set(&receipt.blocking_reasons, "blocking_reasons")?;
    validate_genuine_rebuild_evidence_shape(receipt)?;
    let mut run_ids = BTreeSet::new();
    for run in &receipt.runs {
        validate_run(run)?;
        if !run_ids.insert(run.run_id.clone()) {
            return Err(validation_error(format!("deterministic build proof receipt duplicate run_id {}", run.run_id)));
        }
    }
    Ok(())
}

fn validate_genuine_rebuild_evidence_shape(
    receipt: &DeterministicBuildProofReceipt,
) -> Result<(), ReleaseEvidenceError> {
    if receipt.schema == LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA {
        return Ok(());
    }
    if let Some(digest) = &receipt.rebuild_descriptor_blake3 {
        validate_blake3_hex(digest, "rebuild_descriptor_blake3")?;
    }
    if let Some(digest) = &receipt.rebuild_authority_plan_blake3 {
        validate_blake3_hex(digest, "rebuild_authority_plan_blake3")?;
    }
    Ok(())
}

fn validate_run(run: &DeterministicBuildRunReceipt) -> Result<(), ReleaseEvidenceError> {
    for (field, value) in [
        ("run_id", &run.run_id),
        ("perturbation_case", &run.perturbation_case),
        ("output_root_identity", &run.output_root_identity),
        ("sandbox_profile_identity", &run.sandbox_profile_identity),
    ] {
        if value.trim().is_empty() {
            return Err(validation_error(format!("deterministic build proof receipt run {field} must not be empty")));
        }
    }
    validate_string_set(&run.output_store_paths, "runs.output_store_paths")?;
    validate_string_set(&run.substituted_dependency_identities, "runs.substituted_dependency_identities")?;
    validate_string_set(&run.hermeticity_audit_events, "runs.hermeticity_audit_events")?;
    validate_string_set(&run.observed_read_identities, "runs.observed_read_identities")?;
    validate_string_set(&run.authority_violations, "runs.authority_violations")?;
    if let Some(digest) = &run.rebuild_descriptor_blake3 {
        validate_blake3_hex(digest, "runs.rebuild_descriptor_blake3")?;
    }
    if let Some(digest) = &run.rebuild_authority_plan_blake3 {
        validate_blake3_hex(digest, "runs.rebuild_authority_plan_blake3")?;
    }
    if run.output_digests.is_empty() {
        return Err(validation_error(
            "deterministic build proof receipt run output_digests must not be empty".to_string(),
        ));
    }
    let mut names = BTreeSet::new();
    for output in &run.output_digests {
        if output.name.trim().is_empty() {
            return Err(validation_error(
                "deterministic build proof receipt output name must not be empty".to_string(),
            ));
        }
        if !names.insert(output.name.clone()) {
            return Err(validation_error(format!(
                "deterministic build proof receipt duplicate output name {}",
                output.name
            )));
        }
        validate_blake3_hex(&output.digest_blake3, "deterministic output digest_blake3")?;
    }
    Ok(())
}

fn validate_build_effect_policy(receipt: &DeterministicBuildProofReceipt) -> Result<(), ReleaseEvidenceError> {
    if receipt.effect_policy_version != BUILD_EFFECT_POLICY_VERSION {
        return Err(validation_error(format!(
            "deterministic build proof receipt effect_policy_version must be {BUILD_EFFECT_POLICY_VERSION}, got {}",
            receipt.effect_policy_version
        )));
    }
    validate_effect_set(&receipt.declared_effects, "declared_effects")?;
    if let Some(observed) = &receipt.observed_effects {
        validate_effect_set(observed, "observed_effects")?;
    }
    for run in &receipt.runs {
        if let Some(observed) = &run.observed_effects {
            validate_effect_set(observed, "runs.observed_effects")?;
        }
    }
    Ok(())
}

fn validate_effect_set(values: &[BuildEffect], field: &str) -> Result<(), ReleaseEvidenceError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(validation_error(format!(
                "deterministic build proof receipt {field} contains duplicate build effect {:?}",
                value
            )));
        }
    }
    Ok(())
}

fn receipt_observed_effects(receipt: &DeterministicBuildProofReceipt) -> Option<Vec<BuildEffect>> {
    if let Some(observed) = &receipt.observed_effects {
        return Some(observed.clone());
    }
    let mut effects = BTreeSet::new();
    for run in &receipt.runs {
        let observed = run.observed_effects.as_ref()?;
        effects.extend(observed.iter().copied());
        effects.extend(run.hermeticity_audit_events.iter().filter_map(|event| audit_event_effect(event)));
    }
    Some(effects.into_iter().collect())
}

fn audit_event_effect(event: &str) -> Option<BuildEffect> {
    match event {
        "network-access" | "host-network-access" => Some(BuildEffect::Network),
        "clock-access" | "time-access" => Some(BuildEffect::Clock),
        "random-access" | "entropy-access" => Some(BuildEffect::Random),
        "environment-access" | "host-env-access" | "host-state-leak" => Some(BuildEffect::Environment),
        "secret-access" => Some(BuildEffect::Secret),
        "host-tool-access" | "host-tool-fallback" => Some(BuildEffect::HostTool),
        "remote-build-dispatch" => Some(BuildEffect::RemoteBuild),
        "store-read" => Some(BuildEffect::ReadStore),
        "output-write" => Some(BuildEffect::WriteOutput),
        _ => None,
    }
}

fn undeclared_effect_reasons(receipt: &DeterministicBuildProofReceipt) -> Vec<String> {
    let Some(observed) = receipt_observed_effects(receipt) else {
        return vec!["missing deterministic build effect observations".to_string()];
    };
    if observed.is_empty() {
        return vec!["missing deterministic build effect observations".to_string()];
    }
    let declared = receipt.declared_effects.iter().copied().collect::<BTreeSet<_>>();
    observed
        .into_iter()
        .filter(|effect| !declared.contains(effect))
        .map(|effect| format!("undeclared observed build effect {:?}", effect))
        .collect()
}

fn validate_string_set(values: &[String], field: &str) -> Result<(), ReleaseEvidenceError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(validation_error(format!(
                "deterministic build proof receipt {field} entries must not be empty"
            )));
        }
        if !seen.insert(value) {
            return Err(validation_error(format!(
                "deterministic build proof receipt {field} contains duplicate {value}"
            )));
        }
    }
    Ok(())
}

fn classify_deterministic_build_proof(
    receipt: &DeterministicBuildProofReceipt,
) -> (DeterministicBuildProofVerdict, Vec<String>) {
    let mut reasons = Vec::new();
    if receipt.schema == LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA {
        reasons.push("missing genuine rebuild evidence: legacy path-bound v1 receipt".to_string());
        return (DeterministicBuildProofVerdict::MissingGenuineRebuildEvidence, reasons);
    }
    if receipt.workflow_version != DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA {
        reasons.push(format!("unsupported workflow_version {}", receipt.workflow_version));
        return (DeterministicBuildProofVerdict::UnsupportedWorkflow, reasons);
    }
    if receipt.source_blake3.len() != crate::manifest::BLAKE3_HEX_LENGTH_CHARS
        || receipt.vendor_blake3.len() != crate::manifest::BLAKE3_HEX_LENGTH_CHARS
    {
        reasons.push("malformed source/vendor BLAKE3 input identity".to_string());
        return (DeterministicBuildProofVerdict::MalformedReceipt, reasons);
    }
    if !receipt
        .toolchain_provider_identity
        .contains(&format!("provider-kind={}", receipt.selected_provider_kind))
    {
        reasons.push("provider kind mismatch between proof unit and provider identity".to_string());
        return (DeterministicBuildProofVerdict::ProviderKindMismatch, reasons);
    }
    if receipt.effect_policy_version != BUILD_EFFECT_POLICY_VERSION {
        reasons.push(format!("unsupported build effect policy {}", receipt.effect_policy_version));
        return (DeterministicBuildProofVerdict::MissingEvidence, reasons);
    }
    let effect_reasons = undeclared_effect_reasons(receipt);
    if !effect_reasons.is_empty() {
        reasons.extend(effect_reasons);
        return (DeterministicBuildProofVerdict::MissingEvidence, reasons);
    }
    let genuine_rebuild_reasons = genuine_rebuild_admission_reasons(receipt);
    if !genuine_rebuild_reasons.is_empty() {
        let target_authority_violation =
            genuine_rebuild_reasons.iter().any(|reason| reason.contains("target authority"));
        reasons.extend(genuine_rebuild_reasons);
        return if target_authority_violation {
            (DeterministicBuildProofVerdict::TargetAuthorityViolation, reasons)
        } else {
            (DeterministicBuildProofVerdict::MissingGenuineRebuildEvidence, reasons)
        };
    }
    reasons.extend(strict_proof_eligibility_reasons(receipt));
    if receipt.runs.len() < REQUIRED_RUN_COUNT {
        reasons.push("fewer than two clean proof runs".to_string());
    }
    for required in REQUIRED_PERTURBATIONS {
        if !receipt.ambient_host_perturbations.iter().any(|actual| actual == required) {
            reasons.push(format!("missing ambient host perturbation {required}"));
        }
    }
    reasons.extend(normalization_policy_reasons(&receipt.normalized_execution_envelope));
    if receipt.physical_store_isolation != "fresh-store-per-run"
        && receipt.physical_store_isolation != "clean-namespace-per-run"
    {
        reasons.push(format!("unsupported physical store isolation {}", receipt.physical_store_isolation));
    }
    if receipt.sandbox_profile_identities.is_empty() {
        reasons.push("missing deterministic proof sandbox profile identity".to_string());
    }
    for profile in &receipt.sandbox_profile_identities {
        if !profile.starts_with(SUPPORTED_SANDBOX_PROFILE_PREFIX) {
            reasons.push(format!("unsupported deterministic proof sandbox profile {profile}"));
        }
    }
    reasons.extend(reused_output_store_path_reasons(&receipt.runs));
    for run in &receipt.runs {
        if !receipt.sandbox_profile_identities.iter().any(|profile| profile == &run.sandbox_profile_identity) {
            reasons.push(format!(
                "run {} uses unlisted sandbox profile identity {}",
                run.run_id, run.sandbox_profile_identity
            ));
        }
    }
    if !reasons.is_empty() {
        if receipt.hermeticity_mode == crate::proof_eligibility::IMPURE_HERMETICITY_MODE {
            return (DeterministicBuildProofVerdict::ImpureMode, reasons);
        }
        if reasons.iter().any(|reason| {
            reason.contains("reused derivation-under-test output store identity")
                || reason.contains("reused derivation-under-test output root identity")
        }) {
            return (DeterministicBuildProofVerdict::ReusedStore, reasons);
        }
        if reasons.iter().any(|reason| reason.contains("sandbox")) {
            return (DeterministicBuildProofVerdict::UnsupportedSandbox, reasons);
        }
        return (DeterministicBuildProofVerdict::MissingEvidence, reasons);
    }
    if digest_sets_match(&receipt.runs) {
        (DeterministicBuildProofVerdict::SelfRebuildMatch, Vec::new())
    } else {
        (DeterministicBuildProofVerdict::Mismatch, output_divergence_reasons(&receipt.runs))
    }
}

fn genuine_rebuild_admission_reasons(receipt: &DeterministicBuildProofReceipt) -> Vec<String> {
    let mut reasons = Vec::new();
    let Some(descriptor) = &receipt.rebuild_descriptor else {
        return vec!["missing genuine rebuild evidence: content-bound rebuild descriptor".to_string()];
    };
    let Some(descriptor_blake3) = &receipt.rebuild_descriptor_blake3 else {
        return vec!["missing genuine rebuild evidence: rebuild descriptor BLAKE3".to_string()];
    };
    let Some(plan) = &receipt.rebuild_authority_plan else {
        return vec!["missing genuine rebuild evidence: rebuild authority plan".to_string()];
    };
    let Some(plan_blake3) = &receipt.rebuild_authority_plan_blake3 else {
        return vec!["missing genuine rebuild evidence: rebuild authority plan BLAKE3".to_string()];
    };
    match content_bound_rebuild_descriptor_digest_blake3(descriptor.clone()) {
        Ok(actual) if &actual != descriptor_blake3 => {
            reasons.push("stale genuine rebuild descriptor BLAKE3".to_string())
        }
        Err(err) => reasons.push(format!("invalid genuine rebuild descriptor: {err}")),
        Ok(_) => {}
    }
    match rebuild_authority_plan_digest_blake3(plan.clone()) {
        Ok(actual) if &actual != plan_blake3 => reasons.push("stale genuine rebuild authority plan BLAKE3".to_string()),
        Err(err) => reasons.push(format!("invalid genuine rebuild authority plan: {err}")),
        Ok(_) => {}
    }
    if plan.descriptor_blake3 != *descriptor_blake3 {
        reasons.push("rebuild authority plan cites a different descriptor".to_string());
    }
    if !plan.target_authority_excluded {
        reasons.push("target authority was not excluded".to_string());
    }
    for blocker in &plan.blockers {
        reasons.push(format!("target authority or declared-input blocker {:?}:{}", blocker.code, blocker.subject));
    }
    let mut expected_reads = plan.approved_read_identities.clone();
    expected_reads.sort();
    let descriptor_run_ids = descriptor.run_roots.iter().map(|root| root.run_id.clone()).collect::<BTreeSet<_>>();
    for run in &receipt.runs {
        if run.rebuild_descriptor_blake3.as_ref() != Some(descriptor_blake3) {
            reasons.push(format!("run {} does not cite accepted rebuild descriptor", run.run_id));
        }
        if run.rebuild_authority_plan_blake3.as_ref() != Some(plan_blake3) {
            reasons.push(format!("run {} does not cite accepted rebuild authority plan", run.run_id));
        }
        let mut observed_reads = run.observed_read_identities.clone();
        observed_reads.sort();
        if observed_reads != expected_reads {
            reasons.push(format!("run {} observed read identities differ from authority plan", run.run_id));
        }
        if !run.authority_violations.is_empty() {
            reasons.push(format!(
                "run {} target authority violations: {}",
                run.run_id,
                run.authority_violations.join(",")
            ));
        }
        if !descriptor_run_ids.contains(&run.run_id) {
            reasons.push(format!("run {} is absent from rebuild descriptor roots", run.run_id));
        }
    }
    let run_ids = receipt.runs.iter().map(|run| run.run_id.clone()).collect::<BTreeSet<_>>();
    if run_ids != descriptor_run_ids {
        reasons.push("rebuild descriptor run roots do not match executed runs".to_string());
    }
    let target_pairs = descriptor
        .target_artifacts
        .iter()
        .map(|target| (target.name.clone(), target.digest_blake3.clone()))
        .collect::<BTreeMap<_, _>>();
    let output_pairs = receipt.runs.first().map(canonical_run_digest_map).unwrap_or_default();
    if target_pairs != output_pairs {
        reasons.push("rebuild descriptor target identities do not match selected outputs".to_string());
    }
    if !descriptor.source_inputs.iter().any(|source| source.digest_blake3 == receipt.source_blake3) {
        reasons.push("rebuild descriptor source closure does not bind receipt source".to_string());
    }
    reasons
}

fn normalization_policy_reasons(envelope: &[String]) -> Vec<String> {
    let mut reasons = Vec::new();
    for entry in envelope {
        if let Some(control) = entry.strip_prefix(UNSUPPORTED_NORMALIZATION_PREFIX) {
            reasons.push(format!("unsupported determinism normalization control {control}"));
        }
    }
    for required in REQUIRED_NORMALIZATION_CONTROLS {
        if !envelope_has_normalization_control(envelope, required) {
            reasons.push(format!("missing determinism normalization control {required}"));
        }
    }
    reasons
}

fn envelope_has_normalization_control(envelope: &[String], control: &str) -> bool {
    let exact = format!("{NORMALIZATION_CONTROL_PREFIX}{control}");
    let assigned = format!("{NORMALIZATION_CONTROL_PREFIX}{control}=");
    envelope.iter().any(|entry| entry == &exact || entry.starts_with(&assigned))
}

fn output_divergence_reasons(runs: &[DeterministicBuildRunReceipt]) -> Vec<String> {
    let mut iter = runs.iter().map(canonical_run_digest_map);
    let Some(first) = iter.next() else {
        return vec!["deterministic output divergence:no-runs".to_string()];
    };
    for other in iter {
        let names = first.keys().chain(other.keys()).cloned().collect::<BTreeSet<_>>();
        for name in names {
            if first.get(&name) != other.get(&name) {
                return vec![format!("deterministic output divergence:{name}")];
            }
        }
    }
    vec!["BLAKE3 output digest set mismatch".to_string()]
}

fn strict_proof_eligibility_reasons(receipt: &DeterministicBuildProofReceipt) -> Vec<String> {
    let event_classes =
        receipt.runs.iter().flat_map(|run| run.hermeticity_audit_events.iter().cloned()).collect::<Vec<_>>();
    let report = crate::proof_eligibility::strict_proof_eligibility_gate(
        crate::proof_eligibility::StrictProofEligibilityInput {
            workflow: DETERMINISTIC_PROOF_WORKFLOW.to_string(),
            requested_claim: DETERMINISTIC_PROOF_CLAIM.to_string(),
            hermeticity_mode: receipt.hermeticity_mode.clone(),
            hermeticity_audit_events: event_classes,
            closure_status: crate::proof_eligibility::ProofFactStatus::Satisfied,
            protected_environment_status: crate::proof_eligibility::ProofFactStatus::Satisfied,
            host_tool_status: crate::proof_eligibility::ProofFactStatus::Satisfied,
        },
    );
    if report.strict_claim_satisfied {
        return Vec::new();
    }
    crate::proof_eligibility::proof_eligibility_blocking_reasons(report)
}

fn reused_output_store_path_reasons(runs: &[DeterministicBuildRunReceipt]) -> Vec<String> {
    let mut seen_store_paths = BTreeSet::new();
    let mut seen_output_roots = BTreeSet::new();
    let mut reasons = Vec::new();
    for run in runs {
        if run.output_store_paths.is_empty() {
            reasons.push(format!("run {} has no clean output store identity", run.run_id));
        }
        for path in &run.output_store_paths {
            if !seen_store_paths.insert(path.clone()) {
                reasons.push(format!("reused derivation-under-test output store identity {path}"));
            }
            if run.output_root_identity == *path {
                reasons.push(format!("run {} output root reuses proof store identity {path}", run.run_id));
            }
        }
        if !seen_output_roots.insert(run.output_root_identity.clone()) {
            reasons.push(format!("reused derivation-under-test output root identity {}", run.output_root_identity));
        }
    }
    reasons
}

fn digest_sets_match(runs: &[DeterministicBuildRunReceipt]) -> bool {
    let mut iter = runs.iter().map(canonical_run_digest_map);
    let Some(first) = iter.next() else {
        return false;
    };
    iter.all(|other| other == first)
}

fn canonical_run_digest_map(run: &DeterministicBuildRunReceipt) -> BTreeMap<String, String> {
    run.output_digests
        .iter()
        .map(|output| (output.name.clone(), output.digest_blake3.clone()))
        .collect()
}

fn canonical_digest_set(values: &[String], field: &str) -> Result<Vec<String>, ReleaseEvidenceError> {
    let mut seen = BTreeSet::new();
    for value in values {
        validate_blake3_hex(value, field)?;
        if !seen.insert(value.clone()) {
            return Err(validation_error(format!("{field} contains duplicate digest {value}")));
        }
    }
    Ok(seen.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::BLAKE3_HEX_LENGTH_CHARS;

    fn digest(seed: u8) -> String {
        format!("{:x}", seed % 16).repeat(BLAKE3_HEX_LENGTH_CHARS)
    }

    fn genuine_evidence() -> (ContentBoundRebuildDescriptor, String, RebuildAuthorityPlan, String) {
        let content = |name: &str, role: crate::RebuildInputRole, seed: u8| crate::RebuildContentIdentity {
            name: name.to_string(),
            role,
            kind: crate::RebuildContentKind::RegularFile,
            digest_blake3: digest(seed),
            size_bytes: u64::from(seed).saturating_add(1),
        };
        let arguments = vec!["sh".to_string(), "input:recipe".to_string()];
        let run_roots = ["run-a", "run-b"]
            .into_iter()
            .map(|run_id| crate::RebuildRunRootIdentity {
                run_id: run_id.to_string(),
                output_root_identity: format!("{run_id}:output"),
                store_root_identity: format!("{run_id}:store"),
            })
            .collect::<Vec<_>>();
        let descriptor = ContentBoundRebuildDescriptor {
            schema: crate::CONTENT_BOUND_REBUILD_DESCRIPTOR_SCHEMA.to_string(),
            target_artifacts: vec![content("out", crate::RebuildInputRole::PublishedTarget, 1)],
            recipe: content("recipe", crate::RebuildInputRole::Recipe, 2),
            executable: content("executable", crate::RebuildInputRole::Executable, 3),
            tools: vec![content("tool", crate::RebuildInputRole::Tool, 4)],
            ordered_arguments: arguments.clone(),
            arguments_blake3: crate::rebuild_arguments_digest_blake3(arguments).unwrap(),
            source_inputs: vec![content("source", crate::RebuildInputRole::Source, 10)],
            provider: content("provider", crate::RebuildInputRole::Provider, 6),
            policies: crate::RebuildPolicyIdentities {
                sandbox_policy_blake3: digest(7),
                effect_policy_blake3: digest(8),
                normalization_policy_blake3: digest(9),
            },
            run_roots,
        };
        let descriptor_blake3 = crate::content_bound_rebuild_descriptor_digest_blake3(descriptor.clone()).unwrap();
        let observed = |identity: crate::RebuildContentIdentity, path: &str| crate::RebuildInputObservation {
            identity,
            normalized_path: path.to_string(),
            filesystem_object_identity: None,
        };
        let policy = |role: crate::RebuildInputRole, seed: u8| crate::RebuildInputObservation {
            identity: crate::RebuildContentIdentity {
                name: "policy".to_string(),
                role,
                kind: crate::RebuildContentKind::SyntheticPolicy,
                digest_blake3: digest(seed),
                size_bytes: 1,
            },
            normalized_path: format!("policy:{role:?}"),
            filesystem_object_identity: None,
        };
        let candidate_inputs = vec![
            observed(descriptor.recipe.clone(), "/inputs/recipe"),
            observed(descriptor.executable.clone(), "/inputs/executable"),
            observed(descriptor.tools[0].clone(), "/inputs/tool"),
            observed(descriptor.source_inputs[0].clone(), "/inputs/source"),
            observed(descriptor.provider.clone(), "/inputs/provider"),
            policy(crate::RebuildInputRole::SandboxPolicy, 7),
            policy(crate::RebuildInputRole::EffectPolicy, 8),
            policy(crate::RebuildInputRole::NormalizationPolicy, 9),
        ];
        let run_roots = descriptor
            .run_roots
            .iter()
            .map(|root| crate::RebuildRunRootObservation {
                identity: root.clone(),
                normalized_output_path: format!("/tmp/proof/{}/outputs", root.run_id),
                normalized_store_path: format!("/tmp/proof/{}/store", root.run_id),
            })
            .collect();
        let plan = crate::plan_rebuild_authority(crate::RebuildAuthorityInput {
            descriptor: descriptor.clone(),
            descriptor_blake3: descriptor_blake3.clone(),
            published_targets: vec![observed(descriptor.target_artifacts[0].clone(), "/bundle/out")],
            candidate_inputs,
            run_roots,
            ordinary_output_path: "/tmp/ordinary".to_string(),
            proof_root_path: "/tmp/proof".to_string(),
        });
        assert!(plan.eligible(), "{:?}", plan.blockers);
        let plan_blake3 = crate::rebuild_authority_plan_digest_blake3(plan.clone()).unwrap();
        (descriptor, descriptor_blake3, plan, plan_blake3)
    }

    fn run(id: &str, case: &str, seed: u8) -> DeterministicBuildRunReceipt {
        let (_, descriptor_blake3, plan, plan_blake3) = genuine_evidence();
        DeterministicBuildRunReceipt {
            run_id: id.to_string(),
            perturbation_case: case.to_string(),
            output_store_paths: vec![format!("/mantle/store/{seed:02x}-{id}-demo")],
            output_root_identity: format!("/tmp/proof/{id}/outputs"),
            sandbox_profile_identity: "mantle-proof-sandbox-v1:demo".to_string(),
            output_digests: vec![DeterministicOutputDigest {
                name: "out".to_string(),
                digest_blake3: digest(seed),
            }],
            substituted_dependency_identities: vec!["dep=toolchain-v1".to_string()],
            hermeticity_audit_events: Vec::new(),
            observed_effects: Some(PURE_LOCAL_BUILD_EFFECTS.to_vec()),
            rebuild_descriptor_blake3: Some(descriptor_blake3),
            rebuild_authority_plan_blake3: Some(plan_blake3),
            observed_read_identities: plan.approved_read_identities,
            authority_violations: Vec::new(),
        }
    }

    fn perturbations() -> Vec<String> {
        REQUIRED_PERTURBATIONS.iter().map(|value| value.to_string()).collect()
    }

    fn normalization_envelope() -> Vec<String> {
        let mut envelope = vec!["sandbox=bwrap".to_string(), "network=none".to_string()];
        envelope.extend(
            REQUIRED_NORMALIZATION_CONTROLS
                .iter()
                .map(|control| format!("{NORMALIZATION_CONTROL_PREFIX}{control}=enforced")),
        );
        envelope
    }

    fn receipt() -> DeterministicBuildProofReceipt {
        let (rebuild_descriptor, rebuild_descriptor_blake3, rebuild_authority_plan, rebuild_authority_plan_blake3) =
            genuine_evidence();
        DeterministicBuildProofReceipt::new(DeterministicBuildProofReceiptInit {
            proof_unit: DeterministicProofUnit {
                target_artifact_identity: "release:demo".to_string(),
                output_identities: vec!["out".to_string()],
            },
            selected_provider_kind: "source-root".to_string(),
            source_blake3: digest(10),
            vendor_blake3: digest(11),
            toolchain_stage_roots: vec!["stage-root=/mantle/store/stage".to_string()],
            derivation_identity: "/mantle/store/demo.drv".to_string(),
            hermeticity_mode: "strict".to_string(),
            workflow_version: DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string(),
            toolchain_provider_identity: "provider-kind=source-root;toolchain=gcc10-provider-contract".to_string(),
            logical_store_prefix: "/mantle/store".to_string(),
            physical_store_isolation: "fresh-store-per-run".to_string(),
            effect_policy_version: BUILD_EFFECT_POLICY_VERSION.to_string(),
            declared_effects: PURE_LOCAL_BUILD_EFFECTS.to_vec(),
            observed_effects: None,
            normalized_execution_envelope: normalization_envelope(),
            ambient_host_perturbations: perturbations(),
            sandbox_profile_identities: vec!["mantle-proof-sandbox-v1:demo".to_string()],
            runs: vec![run("run-b", "case-b", 1), run("run-a", "case-a", 1)],
            rebuild_descriptor,
            rebuild_descriptor_blake3,
            rebuild_authority_plan,
            rebuild_authority_plan_blake3,
        })
    }

    fn isolation_evidence() -> DeterministicSandboxIsolationEvidence {
        DeterministicSandboxIsolationEvidence {
            schema: DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA.to_string(),
            profile_family: SUPPORTED_SANDBOX_PROFILE_FAMILY.to_string(),
            evidence_version: "mantle-deterministic-proof-sandbox-isolation-v1".to_string(),
            status: DeterministicSandboxIsolationEvidenceStatus::Passed,
            checks: REQUIRED_ISOLATION_CHECKS.iter().map(|check| check.to_string()).collect(),
            evidence_digest_blake3: digest(15),
        }
    }

    #[test]
    fn deterministic_receipt_canonical_bytes_are_stable() {
        let mut first = receipt();
        first.runs.reverse();
        first.ambient_host_perturbations.reverse();
        let second = receipt();
        assert_eq!(
            deterministic_build_proof_receipt_canonical_bytes(first).unwrap(),
            deterministic_build_proof_receipt_canonical_bytes(second).unwrap()
        );
    }

    #[test]
    fn deterministic_receipt_accepts_strict_matching_runs() {
        let canonical = canonical_deterministic_build_proof_receipt(receipt()).unwrap();
        assert_eq!(canonical.verdict, DeterministicBuildProofVerdict::SelfRebuildMatch);
        assert_eq!(canonical.effect_policy_version, BUILD_EFFECT_POLICY_VERSION);
        assert_eq!(canonical.declared_effects, PURE_LOCAL_BUILD_EFFECTS);
    }

    #[test]
    fn deterministic_receipt_rejects_missing_effect_observations() {
        for observed_effects in [None, Some(Vec::new())] {
            let mut receipt = receipt();
            receipt.runs[0].observed_effects = observed_effects;
            if let Some(effects) = &mut receipt.runs[1].observed_effects {
                effects.clear();
            }
            let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
            assert_eq!(verdict, DeterministicBuildProofVerdict::MissingEvidence);
            assert!(
                reasons.iter().any(|reason| reason.contains("missing deterministic build effect observations")),
                "{reasons:?}"
            );
        }
    }

    #[test]
    fn deterministic_receipt_rejects_unsupported_effect_policy() {
        let mut receipt = receipt();
        receipt.effect_policy_version = "mantle-build-effects-v0".to_string();
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
        assert_eq!(verdict, DeterministicBuildProofVerdict::MissingEvidence);
        assert!(reasons.iter().any(|reason| reason.contains("unsupported build effect policy")));
    }

    #[test]
    fn deterministic_receipt_rejects_undeclared_observed_effects() {
        for (event, expected) in [
            ("network-access", "Network"),
            ("clock-access", "Clock"),
            ("host-env-access", "Environment"),
            ("host-tool-access", "HostTool"),
        ] {
            let mut receipt = receipt();
            receipt.declared_effects = vec![BuildEffect::ReadStore, BuildEffect::WriteOutput];
            receipt.runs[0].hermeticity_audit_events.push(event.to_string());
            let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
            assert_eq!(verdict, DeterministicBuildProofVerdict::MissingEvidence, "{event}");
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason.contains("undeclared observed build effect") && reason.contains(expected)),
                "{event}: {reasons:?}"
            );
        }
    }

    #[test]
    fn deterministic_receipt_blocks_unknown_audit_events_closed_by_default() {
        let mut receipt = receipt();
        receipt.runs[0].hermeticity_audit_events.push("future-benign-event".to_string());
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);

        assert_eq!(verdict, DeterministicBuildProofVerdict::MissingEvidence);
        assert!(reasons.iter().any(|reason| reason.contains("future-benign-event")), "{reasons:?}");
        assert!(
            reasons.iter().any(|reason| reason.contains(crate::proof_audit::STRICT_PROOF_POLICY_BASIS)),
            "{reasons:?}"
        );
    }

    #[test]
    fn deterministic_receipt_blocks_degraded_audit_even_when_effect_is_declared() {
        let mut receipt = receipt();
        receipt.declared_effects.push(BuildEffect::HostTool);
        receipt.runs[0].observed_effects = Some(vec![
            BuildEffect::ReadStore,
            BuildEffect::WriteOutput,
            BuildEffect::Environment,
            BuildEffect::HostTool,
        ]);
        receipt.runs[0].hermeticity_audit_events.push("host-tool-fallback".to_string());
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);

        assert_eq!(verdict, DeterministicBuildProofVerdict::MissingEvidence);
        assert!(reasons.iter().any(|reason| reason.contains("host-tool-fallback")), "{reasons:?}");
        assert!(reasons.iter().all(|reason| !reason.contains("undeclared observed build effect")), "{reasons:?}");
    }

    #[test]
    fn deterministic_receipt_accepts_policy_informational_audit_events() {
        let mut receipt = receipt();
        receipt.runs[0].hermeticity_audit_events.push("store-read".to_string());
        receipt.runs[0].hermeticity_audit_events.push("output-write".to_string());
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);

        assert_eq!(verdict, DeterministicBuildProofVerdict::SelfRebuildMatch);
        assert!(reasons.is_empty(), "{reasons:?}");
    }

    #[test]
    fn deterministic_receipt_rejects_digest_drift() {
        let mut receipt = receipt();
        receipt.runs[1].output_digests[0].digest_blake3 = digest(2);
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);

        assert_eq!(verdict, DeterministicBuildProofVerdict::Mismatch);
        assert!(reasons.contains(&"deterministic output divergence:out".to_string()));
    }

    #[test]
    fn deterministic_receipt_blocks_missing_or_unsupported_normalization_controls() {
        let mut missing = receipt();
        let missing_umask_prefix = format!("{NORMALIZATION_CONTROL_PREFIX}umask=");
        missing.normalized_execution_envelope.retain(|entry| !entry.starts_with(&missing_umask_prefix));
        let (missing_verdict, missing_reasons) = classify_deterministic_build_proof(&missing);
        assert_eq!(missing_verdict, DeterministicBuildProofVerdict::MissingEvidence);
        assert!(missing_reasons.iter().any(|reason| reason == "missing determinism normalization control umask"));

        let mut unsupported = receipt();
        unsupported
            .normalized_execution_envelope
            .push(format!("{UNSUPPORTED_NORMALIZATION_PREFIX}modeled-randomness=executor-lacks-control"));
        let (unsupported_verdict, unsupported_reasons) = classify_deterministic_build_proof(&unsupported);
        assert_eq!(unsupported_verdict, DeterministicBuildProofVerdict::MissingEvidence);
        assert!(
            unsupported_reasons
                .iter()
                .any(|reason| reason.contains("unsupported determinism normalization control modeled-randomness"))
        );
    }

    #[test]
    fn deterministic_receipt_blocks_impure_mode() {
        let mut receipt = receipt();
        receipt.hermeticity_mode = "impure".to_string();
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
        receipt.verdict = verdict;
        receipt.blocking_reasons = reasons;
        let canonical = canonical_deterministic_build_proof_receipt(receipt).unwrap();
        assert_eq!(canonical.verdict, DeterministicBuildProofVerdict::ImpureMode);
    }

    #[test]
    fn deterministic_receipt_blocks_reused_output_store_identity() {
        let mut receipt = receipt();
        receipt.runs[1].output_store_paths = receipt.runs[0].output_store_paths.clone();
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);

        assert_eq!(verdict, DeterministicBuildProofVerdict::ReusedStore);
        assert!(reasons.iter().any(|reason| reason.contains("reused derivation-under-test output store identity")));
    }

    #[test]
    fn deterministic_receipt_rejects_unsupported_sandbox_profile() {
        let mut receipt = receipt();
        receipt.sandbox_profile_identities = vec!["direct-host:demo".to_string()];
        receipt.runs[0].sandbox_profile_identity = "direct-host:demo".to_string();
        receipt.runs[1].sandbox_profile_identity = "direct-host:demo".to_string();
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
        assert_eq!(verdict, DeterministicBuildProofVerdict::UnsupportedSandbox);
        assert!(reasons.iter().any(|reason| reason.contains("unsupported deterministic proof sandbox profile")));
    }

    #[test]
    fn deterministic_receipt_rejects_unsupported_workflow_version() {
        let mut receipt = receipt();
        receipt.workflow_version = "mantle-deterministic-proof-receipt-v0".to_string();
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
        assert_eq!(verdict, DeterministicBuildProofVerdict::UnsupportedWorkflow);
        assert!(reasons.iter().any(|reason| reason.contains("unsupported workflow_version")));
    }

    #[test]
    fn deterministic_receipt_rejects_provider_kind_mismatch() {
        let mut receipt = receipt();
        receipt.selected_provider_kind = "binary-cache".to_string();
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
        assert_eq!(verdict, DeterministicBuildProofVerdict::ProviderKindMismatch);
        assert!(reasons.iter().any(|reason| reason.contains("provider kind mismatch")));
    }

    #[test]
    fn deterministic_sandbox_isolation_evidence_canonical_bytes_sort_checks() {
        let mut first = isolation_evidence();
        first.checks.reverse();
        let second = isolation_evidence();
        assert_eq!(
            deterministic_sandbox_isolation_evidence_canonical_bytes(first).unwrap(),
            deterministic_sandbox_isolation_evidence_canonical_bytes(second).unwrap()
        );
        assert_eq!(
            deterministic_sandbox_isolation_evidence_digest_blake3(isolation_evidence()).unwrap().len(),
            BLAKE3_HEX_LENGTH_CHARS
        );
    }

    #[test]
    fn legacy_path_bound_receipt_remains_parseable_but_non_promoting() {
        let mut legacy = receipt();
        legacy.schema = LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string();
        legacy.workflow_version = LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string();
        legacy.rebuild_descriptor = None;
        legacy.rebuild_descriptor_blake3 = None;
        legacy.rebuild_authority_plan = None;
        legacy.rebuild_authority_plan_blake3 = None;
        for run in &mut legacy.runs {
            run.rebuild_descriptor_blake3 = None;
            run.rebuild_authority_plan_blake3 = None;
            run.observed_read_identities.clear();
            run.authority_violations.clear();
        }
        legacy.verdict = DeterministicBuildProofVerdict::SelfRebuildMatch;
        legacy.blocking_reasons.clear();
        let bytes = deterministic_build_proof_receipt_canonical_bytes(legacy.clone()).unwrap();
        let parsed: DeterministicBuildProofReceipt = serde_json::from_slice(&bytes).unwrap();
        let evidence = isolation_evidence();

        assert_eq!(parsed.schema, LEGACY_DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA);
        assert!(!deterministic_build_proof_has_genuine_rebuild_authority(parsed.clone()).unwrap());
        assert!(!deterministic_release_claim_eligible(&[digest(1)], &[parsed], Some(&evidence)).unwrap());
    }

    #[test]
    fn deterministic_release_claim_requires_exact_digest_set() {
        let receipt = receipt();
        let evidence = isolation_evidence();
        assert!(deterministic_release_claim_eligible(&[digest(1)], &[receipt], Some(&evidence)).unwrap());
        assert!(!deterministic_release_claim_eligible(&[digest(2)], &[self::receipt()], Some(&evidence)).unwrap());
    }

    #[test]
    fn deterministic_release_claim_requires_isolation_evidence() {
        assert!(!deterministic_release_claim_eligible(&[digest(1)], &[receipt()], None).unwrap());
    }

    #[test]
    fn deterministic_release_claim_rejects_failed_isolation_evidence() {
        let mut evidence = isolation_evidence();
        evidence.status = DeterministicSandboxIsolationEvidenceStatus::Failed;
        let err = deterministic_release_claim_eligible(&[digest(1)], &[receipt()], Some(&evidence)).unwrap_err();
        assert!(err.to_string().contains("status must be passed"));
    }

    #[test]
    fn deterministic_release_claim_rejects_profile_family_mismatch() {
        let mut evidence = isolation_evidence();
        evidence.profile_family = "mantle-proof-sandbox-v2".to_string();
        let err = deterministic_release_claim_eligible(&[digest(1)], &[receipt()], Some(&evidence)).unwrap_err();
        assert!(err.to_string().contains("profile_family"));
    }

    #[test]
    fn deterministic_release_claim_rejects_missing_isolation_check() {
        for missing_check in ["denies-host-network-by-default", "denies-clock-syscalls"] {
            let mut evidence = isolation_evidence();
            evidence.checks.retain(|check| check != missing_check);
            let err = deterministic_release_claim_eligible(&[digest(1)], &[receipt()], Some(&evidence)).unwrap_err();

            assert!(err.to_string().contains(&format!("missing required check {missing_check}")));
            assert!(!evidence.checks.iter().any(|check| check == missing_check));
        }
    }
}
