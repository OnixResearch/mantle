use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

pub const DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA: &str = "mantle-deterministic-proof-receipt-v1";
pub const BUILD_EFFECT_POLICY_VERSION: &str = "mantle-build-effects-v1";
pub const DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA: &str = "mantle-deterministic-sandbox-isolation-evidence-v1";
const REQUIRED_RUN_COUNT: usize = 2;
pub const SUPPORTED_SANDBOX_PROFILE_FAMILY: &str = "mantle-proof-sandbox-v1";
pub const REQUIRED_ISOLATION_CHECKS: &[&str] = &[
    "denies-undeclared-host-access",
    "denies-host-network-by-default",
    "denies-main-output-and-proof-store-reuse",
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
const DETERMINISTIC_PROOF_AUDIT_WORKFLOW: &str = "deterministic-release";
const DETERMINISTIC_PROOF_AUDIT_CLAIM: &str = "deterministic-release";
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
    receipt.schema = DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string();
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
        if let Some(observed_effects) = &mut run.observed_effects {
            observed_effects.sort();
            observed_effects.dedup();
        }
        run.output_digests.sort_by(|left, right| left.name.cmp(&right.name));
    }
    receipt.runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    validate_receipt_evidence(&receipt)?;
    let (expected_verdict, expected_reasons) = classify_deterministic_build_proof(&receipt);
    if receipt.verdict != expected_verdict {
        return Err(validation_error(format!(
            "deterministic build proof receipt verdict {:?} does not match classified verdict {:?}",
            receipt.verdict, expected_verdict
        )));
    }
    let mut sorted_expected_reasons = expected_reasons;
    sorted_expected_reasons.sort();
    if receipt.blocking_reasons != sorted_expected_reasons {
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

fn validate_receipt_header(receipt: &DeterministicBuildProofReceipt) -> Result<(), ReleaseEvidenceError> {
    if receipt.schema != DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA {
        return Err(validation_error(format!(
            "deterministic build proof receipt schema must be {DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA}, got {}",
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
    let mut run_ids = BTreeSet::new();
    for run in &receipt.runs {
        validate_run(run)?;
        if !run_ids.insert(run.run_id.clone()) {
            return Err(validation_error(format!("deterministic build proof receipt duplicate run_id {}", run.run_id)));
        }
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
    if receipt.hermeticity_mode == "impure" {
        reasons.push("impure hermeticity mode".to_string());
        return (DeterministicBuildProofVerdict::ImpureMode, reasons);
    }
    if receipt.hermeticity_mode != "strict" {
        reasons.push(format!("non-strict hermeticity mode {}", receipt.hermeticity_mode));
    }
    if receipt.runs.len() < REQUIRED_RUN_COUNT {
        reasons.push("fewer than two clean proof runs".to_string());
    }
    for required in REQUIRED_PERTURBATIONS {
        if !receipt.ambient_host_perturbations.iter().any(|actual| actual == required) {
            reasons.push(format!("missing ambient host perturbation {required}"));
        }
    }
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
    reasons.extend(proof_audit_gate_reasons(receipt));
    if !reasons.is_empty() {
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
        (DeterministicBuildProofVerdict::Mismatch, vec!["BLAKE3 output digest set mismatch".to_string()])
    }
}

fn proof_audit_gate_reasons(receipt: &DeterministicBuildProofReceipt) -> Vec<String> {
    let event_classes =
        receipt.runs.iter().flat_map(|run| run.hermeticity_audit_events.iter().cloned()).collect::<Vec<_>>();
    let report = crate::proof_audit::strict_proof_audit_gate(
        DETERMINISTIC_PROOF_AUDIT_WORKFLOW.to_string(),
        DETERMINISTIC_PROOF_AUDIT_CLAIM.to_string(),
        event_classes,
    );
    if report.strict_claim_satisfied {
        return Vec::new();
    }
    crate::proof_audit::proof_audit_gate_blocking_reasons(report)
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

    fn run(id: &str, case: &str, seed: u8) -> DeterministicBuildRunReceipt {
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
        }
    }

    fn perturbations() -> Vec<String> {
        REQUIRED_PERTURBATIONS.iter().map(|value| value.to_string()).collect()
    }

    fn receipt() -> DeterministicBuildProofReceipt {
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
            normalized_execution_envelope: vec!["sandbox=bwrap".to_string(), "network=none".to_string()],
            ambient_host_perturbations: perturbations(),
            sandbox_profile_identities: vec!["mantle-proof-sandbox-v1:demo".to_string()],
            runs: vec![run("run-b", "case-b", 1), run("run-a", "case-a", 1)],
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
        let mut receipt = DeterministicBuildProofReceipt::new(DeterministicBuildProofReceiptInit {
            runs: vec![run("run-a", "case-a", 1), run("run-b", "case-b", 2)],
            ..DeterministicBuildProofReceiptInit {
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
                normalized_execution_envelope: vec!["sandbox=bwrap".to_string()],
                ambient_host_perturbations: perturbations(),
                sandbox_profile_identities: vec!["mantle-proof-sandbox-v1:demo".to_string()],
                runs: Vec::new(),
            }
        });
        assert_eq!(receipt.verdict, DeterministicBuildProofVerdict::Mismatch);
        receipt.blocking_reasons.sort();
        assert!(receipt.blocking_reasons.contains(&"BLAKE3 output digest set mismatch".to_string()));
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
        let mut reused = run("run-b", "case-b", 1);
        reused.output_store_paths = vec!["/mantle/store/reused-demo".to_string()];
        let mut first = run("run-a", "case-a", 1);
        first.output_store_paths = vec!["/mantle/store/reused-demo".to_string()];
        let receipt = DeterministicBuildProofReceipt::new(DeterministicBuildProofReceiptInit {
            runs: vec![first, reused],
            ..DeterministicBuildProofReceiptInit {
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
                normalized_execution_envelope: vec!["sandbox=bwrap".to_string()],
                ambient_host_perturbations: perturbations(),
                sandbox_profile_identities: vec!["mantle-proof-sandbox-v1:demo".to_string()],
                runs: Vec::new(),
            }
        });
        assert_eq!(receipt.verdict, DeterministicBuildProofVerdict::ReusedStore);
        assert!(
            receipt
                .blocking_reasons
                .iter()
                .any(|reason| reason.contains("reused derivation-under-test output store identity")
                    || reason.contains("reused derivation-under-test output root identity"))
        );
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
        let mut evidence = isolation_evidence();
        evidence.checks.retain(|check| check != "denies-host-network-by-default");
        let err = deterministic_release_claim_eligible(&[digest(1)], &[receipt()], Some(&evidence)).unwrap_err();
        assert!(err.to_string().contains("missing required check denies-host-network-by-default"));
    }
}
