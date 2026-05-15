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

pub const DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA: &str = "mantle-deterministic-build-proof-v1";
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
const PROOF_BLOCKING_AUDIT_EVENTS: &[&str] = &["host-tool-fallback", "host-state-leak", "impure-mode-selected"];
const SUPPORTED_SANDBOX_PROFILE_PREFIX: &str = "mantle-proof-sandbox-v1:";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeterministicBuildProofVerdict {
    NotAttempted,
    DeterministicMatch,
    Mismatch,
    MissingEvidence,
    ImpureMode,
    UnsupportedWorkflow,
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
    pub sandbox_profile_identity: String,
    pub output_digests: Vec<DeterministicOutputDigest>,
    pub substituted_dependency_identities: Vec<String>,
    pub hermeticity_audit_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeterministicBuildProofReceipt {
    pub schema: String,
    pub derivation_identity: String,
    pub hermeticity_mode: String,
    pub workflow_version: String,
    pub toolchain_provider_identity: String,
    pub logical_store_prefix: String,
    pub physical_store_isolation: String,
    pub normalized_execution_envelope: Vec<String>,
    pub ambient_host_perturbations: Vec<String>,
    pub sandbox_profile_identities: Vec<String>,
    pub runs: Vec<DeterministicBuildRunReceipt>,
    pub verdict: DeterministicBuildProofVerdict,
    pub blocking_reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicBuildProofReceiptInit {
    pub derivation_identity: String,
    pub hermeticity_mode: String,
    pub workflow_version: String,
    pub toolchain_provider_identity: String,
    pub logical_store_prefix: String,
    pub physical_store_isolation: String,
    pub normalized_execution_envelope: Vec<String>,
    pub ambient_host_perturbations: Vec<String>,
    pub sandbox_profile_identities: Vec<String>,
    pub runs: Vec<DeterministicBuildRunReceipt>,
}

impl DeterministicBuildProofReceipt {
    pub fn new(init: DeterministicBuildProofReceiptInit) -> Self {
        let mut receipt = Self {
            schema: DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string(),
            derivation_identity: init.derivation_identity,
            hermeticity_mode: init.hermeticity_mode,
            workflow_version: init.workflow_version,
            toolchain_provider_identity: init.toolchain_provider_identity,
            logical_store_prefix: init.logical_store_prefix,
            physical_store_isolation: init.physical_store_isolation,
            normalized_execution_envelope: init.normalized_execution_envelope,
            ambient_host_perturbations: init.ambient_host_perturbations,
            sandbox_profile_identities: init.sandbox_profile_identities,
            runs: init.runs,
            verdict: DeterministicBuildProofVerdict::NotAttempted,
            blocking_reasons: Vec::new(),
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
    validate_receipt_header(&receipt)?;
    receipt.schema = DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string();
    receipt.normalized_execution_envelope.sort();
    receipt.ambient_host_perturbations.sort();
    receipt.sandbox_profile_identities.sort();
    receipt.blocking_reasons.sort();
    for run in &mut receipt.runs {
        run.output_store_paths.sort();
        run.substituted_dependency_identities.sort();
        run.hermeticity_audit_events.sort();
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
    Ok(receipt)
}

pub fn deterministic_build_proof_receipt_canonical_bytes(
    receipt: DeterministicBuildProofReceipt,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_deterministic_build_proof_receipt(receipt)?;
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing deterministic build proof receipt: {err}")))
}

pub fn deterministic_build_proof_receipt_digest_blake3(
    receipt: DeterministicBuildProofReceipt,
) -> Result<String, ReleaseEvidenceError> {
    let bytes = deterministic_build_proof_receipt_canonical_bytes(receipt)?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

pub fn deterministic_release_claim_eligible(
    release_digest_set: &[String],
    receipts: &[DeterministicBuildProofReceipt],
) -> Result<bool, ReleaseEvidenceError> {
    let mut release_digests = canonical_digest_set(release_digest_set, "release_digest_set")?;
    release_digests.sort();
    let mut receipt_digests = Vec::new();
    for receipt in receipts {
        let receipt = canonical_deterministic_build_proof_receipt(receipt.clone())?;
        if receipt.verdict != DeterministicBuildProofVerdict::DeterministicMatch {
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
        ("derivation_identity", &receipt.derivation_identity),
        ("hermeticity_mode", &receipt.hermeticity_mode),
        ("workflow_version", &receipt.workflow_version),
        ("toolchain_provider_identity", &receipt.toolchain_provider_identity),
        ("logical_store_prefix", &receipt.logical_store_prefix),
        ("physical_store_isolation", &receipt.physical_store_isolation),
    ] {
        if value.trim().is_empty() {
            return Err(validation_error(format!("deterministic build proof receipt {field} must not be empty")));
        }
    }
    Ok(())
}

fn validate_receipt_evidence(receipt: &DeterministicBuildProofReceipt) -> Result<(), ReleaseEvidenceError> {
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
        for event in &run.hermeticity_audit_events {
            if PROOF_BLOCKING_AUDIT_EVENTS.iter().any(|blocking| blocking == event) {
                reasons.push(format!("proof-blocking audit event {event}"));
            }
        }
    }
    if !reasons.is_empty() {
        return (DeterministicBuildProofVerdict::MissingEvidence, reasons);
    }
    if digest_sets_match(&receipt.runs) {
        (DeterministicBuildProofVerdict::DeterministicMatch, Vec::new())
    } else {
        (DeterministicBuildProofVerdict::Mismatch, vec!["BLAKE3 output digest set mismatch".to_string()])
    }
}

fn reused_output_store_path_reasons(runs: &[DeterministicBuildRunReceipt]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut reasons = Vec::new();
    for run in runs {
        if run.output_store_paths.is_empty() {
            reasons.push(format!("run {} has no clean output store identity", run.run_id));
        }
        for path in &run.output_store_paths {
            if !seen.insert(path.clone()) {
                reasons.push(format!("reused derivation-under-test output store identity {path}"));
            }
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
            sandbox_profile_identity: "mantle-proof-sandbox-v1:demo".to_string(),
            output_digests: vec![DeterministicOutputDigest {
                name: "out".to_string(),
                digest_blake3: digest(seed),
            }],
            substituted_dependency_identities: vec!["dep=toolchain-v1".to_string()],
            hermeticity_audit_events: Vec::new(),
        }
    }

    fn perturbations() -> Vec<String> {
        REQUIRED_PERTURBATIONS.iter().map(|value| value.to_string()).collect()
    }

    fn receipt() -> DeterministicBuildProofReceipt {
        DeterministicBuildProofReceipt::new(DeterministicBuildProofReceiptInit {
            derivation_identity: "/mantle/store/demo.drv".to_string(),
            hermeticity_mode: "strict".to_string(),
            workflow_version: DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string(),
            toolchain_provider_identity: "toolchain=gcc10-provider-contract".to_string(),
            logical_store_prefix: "/mantle/store".to_string(),
            physical_store_isolation: "fresh-store-per-run".to_string(),
            normalized_execution_envelope: vec!["sandbox=bwrap".to_string(), "network=none".to_string()],
            ambient_host_perturbations: perturbations(),
            sandbox_profile_identities: vec!["mantle-proof-sandbox-v1:demo".to_string()],
            runs: vec![run("run-b", "case-b", 1), run("run-a", "case-a", 1)],
        })
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
        assert_eq!(canonical.verdict, DeterministicBuildProofVerdict::DeterministicMatch);
    }

    #[test]
    fn deterministic_receipt_rejects_digest_drift() {
        let mut receipt = DeterministicBuildProofReceipt::new(DeterministicBuildProofReceiptInit {
            runs: vec![run("run-a", "case-a", 1), run("run-b", "case-b", 2)],
            ..DeterministicBuildProofReceiptInit {
                derivation_identity: "/mantle/store/demo.drv".to_string(),
                hermeticity_mode: "strict".to_string(),
                workflow_version: DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string(),
                toolchain_provider_identity: "toolchain=gcc10-provider-contract".to_string(),
                logical_store_prefix: "/mantle/store".to_string(),
                physical_store_isolation: "fresh-store-per-run".to_string(),
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
                derivation_identity: "/mantle/store/demo.drv".to_string(),
                hermeticity_mode: "strict".to_string(),
                workflow_version: DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string(),
                toolchain_provider_identity: "toolchain=gcc10-provider-contract".to_string(),
                logical_store_prefix: "/mantle/store".to_string(),
                physical_store_isolation: "fresh-store-per-run".to_string(),
                normalized_execution_envelope: vec!["sandbox=bwrap".to_string()],
                ambient_host_perturbations: perturbations(),
                sandbox_profile_identities: vec!["mantle-proof-sandbox-v1:demo".to_string()],
                runs: Vec::new(),
            }
        });
        assert_eq!(receipt.verdict, DeterministicBuildProofVerdict::MissingEvidence);
        assert!(
            receipt
                .blocking_reasons
                .iter()
                .any(|reason| reason.contains("reused derivation-under-test output store identity"))
        );
    }

    #[test]
    fn deterministic_receipt_rejects_unsupported_sandbox_profile() {
        let mut receipt = receipt();
        receipt.sandbox_profile_identities = vec!["direct-host:demo".to_string()];
        receipt.runs[0].sandbox_profile_identity = "direct-host:demo".to_string();
        receipt.runs[1].sandbox_profile_identity = "direct-host:demo".to_string();
        let (verdict, reasons) = classify_deterministic_build_proof(&receipt);
        assert_eq!(verdict, DeterministicBuildProofVerdict::MissingEvidence);
        assert!(reasons.iter().any(|reason| reason.contains("unsupported deterministic proof sandbox profile")));
    }

    #[test]
    fn deterministic_release_claim_requires_exact_digest_set() {
        let receipt = receipt();
        assert!(deterministic_release_claim_eligible(&[digest(1)], &[receipt]).unwrap());
        assert!(!deterministic_release_claim_eligible(&[digest(2)], &[self::receipt()]).unwrap());
    }
}
