//! Imperative shell for the source-built bootstrap trust report.

use std::fs;
use std::io::Read as _;
use std::path::Path;

use crunch_release_core::DeterministicBuildProofReceipt;
use crunch_release_core::DeterministicBuildProofVerdict;
use serde::Deserialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::validate_source_built_fixed_point_plan;
use crate::source_built_fixed_point_receipt::EXTENSION_FIELD;
use crate::source_built_fixed_point_receipt::EXTENSION_SCHEMA;
use crate::source_built_fixed_point_receipt::SourceBuiltReceiptExtension;
use crate::source_built_fixed_point_receipt::SourceBuiltStageEvidence;
use crate::source_built_fixed_point_receipt::StageExecutionOrigin;
use crate::source_built_trust_report::ROOT_ACTION_RECONCILIATION_FILE;
use crate::source_built_trust_report::ROOT_ACTION_RECONCILIATION_SCHEMA;
use crate::source_built_trust_report::ROOT_ACTION_TRUST_PLAN_FILE;
use crate::source_built_trust_report::ROOT_ACTION_TRUST_PLAN_SCHEMA;
use crate::source_built_trust_report::RootActionTrustFacts;
use crate::source_built_trust_report::VerifiedFixedPointTrustFacts;
use crate::source_built_trust_report::build_bootstrap_trust_report;
use crate::source_built_trust_report::render_bootstrap_trust_report;

const PLAN_FILE: &str = "source-built-fixed-point-plan.json";
const STAGE_EVIDENCE_FILE: &str = "source-built-stage-evidence.json";
const HASH_BUFFER_KIBIBYTES: usize = 64;
const BYTES_PER_KIBIBYTE: usize = 1_024;
const HASH_BUFFER_BYTES: usize = HASH_BUFFER_KIBIBYTES * BYTES_PER_KIBIBYTE;
const BLAKE3_HEX_LENGTH: usize = 64;
const ACTION_COUNT_MAX: u32 = 1_048_576;

#[derive(Debug, Deserialize)]
struct RootActionTrustPlanSummary {
    schema: String,
    proof_plan_digest_blake3: String,
    adapter_count: u32,
    action_count: u32,
    local_only: bool,
    cache_only_completion_allowed: bool,
    blockers: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RootActionReconciliationSummary {
    schema: String,
    action_plan_digest_blake3: String,
    planned_action_count: u32,
    matched_action_count: u32,
    observed_event_count: u32,
    matched_event_count: u32,
    unknown_event_count: u32,
    missing_action_count: u32,
    authority_violation_count: u32,
    fallback_event_count: u32,
    remote_event_count: u32,
    cache_only_completion_count: u32,
    local_only: bool,
    blockers: Vec<String>,
}

pub(crate) fn cmd_bootstrap_trust_report(proof_root: &Path, json: bool) -> Result<(), RunError> {
    let canonical_root = canonical_proof_root(proof_root)?;
    let facts = observe_verified_fixed_point(&canonical_root)?;
    let action_trust = observe_optional_action_trust(&canonical_root, &facts)?;
    let report = build_bootstrap_trust_report(facts, action_trust)
        .map_err(|error| trust_error(format!("classifying bounded trust: {error}")))?;
    if json {
        let rendered = serde_json::to_string_pretty(&report)
            .map_err(|error| trust_error(format!("serializing bootstrap trust report: {error}")))?;
        println!("{rendered}");
    } else {
        print!("{}", render_bootstrap_trust_report(&report));
    }
    assert!(report.fixed_point_verified);
    debug_assert_eq!(report.schema, crate::source_built_trust_report::BOOTSTRAP_TRUST_REPORT_SCHEMA);
    Ok(())
}

fn canonical_proof_root(proof_root: &Path) -> Result<std::path::PathBuf, RunError> {
    let canonical = fs::canonicalize(proof_root)
        .map_err(|error| trust_error(format!("resolving proof root {}: {error}", proof_root.display())))?;
    let metadata = fs::metadata(&canonical)
        .map_err(|error| trust_error(format!("reading proof root {}: {error}", canonical.display())))?;
    if !metadata.is_dir() {
        return Err(trust_error(format!("proof root is not a directory: {}", canonical.display())));
    }
    assert!(canonical.is_absolute());
    debug_assert!(metadata.is_dir());
    Ok(canonical)
}

fn observe_verified_fixed_point(proof_root: &Path) -> Result<VerifiedFixedPointTrustFacts, RunError> {
    let receipt_path = proof_root.join(crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE);
    crate::source_built_fixed_point_receipt::verify_source_built_fixed_point_receipt(proof_root, &receipt_path)?;
    let receipt_value: serde_json::Value = read_json(&receipt_path, "deterministic proof receipt")?;
    let core: DeterministicBuildProofReceipt = serde_json::from_value(receipt_value.clone())
        .map_err(|error| trust_error(format!("parsing deterministic proof receipt core: {error}")))?;
    let canonical = crunch_release_core::canonical_deterministic_build_proof_receipt(core)
        .map_err(|error| trust_error(format!("validating deterministic proof receipt core: {error}")))?;
    if canonical.verdict != DeterministicBuildProofVerdict::SelfRebuildMatch {
        return Err(trust_error("deterministic proof verdict is not self-rebuild-match".to_string()));
    }
    let extension: SourceBuiltReceiptExtension = serde_json::from_value(
        receipt_value
            .get(EXTENSION_FIELD)
            .cloned()
            .ok_or_else(|| trust_error(format!("receipt is missing {EXTENSION_FIELD}")))?,
    )
    .map_err(|error| trust_error(format!("parsing source-built receipt extension: {error}")))?;
    let plan: SourceBuiltFixedPointPlan = read_json(&proof_root.join(PLAN_FILE), "source-built fixed-point plan")?;
    validate_report_linkage(&canonical, &extension, &plan)?;
    let stages: Vec<SourceBuiltStageEvidence> =
        read_json(&proof_root.join(STAGE_EVIDENCE_FILE), "source-built stage evidence")?;
    fixed_point_facts(canonical, extension, &plan, &stages)
}

fn validate_report_linkage(
    receipt: &DeterministicBuildProofReceipt,
    extension: &SourceBuiltReceiptExtension,
    plan: &SourceBuiltFixedPointPlan,
) -> Result<(), RunError> {
    validate_source_built_fixed_point_plan(plan)
        .map_err(|error| trust_error(format!("validating source-built fixed-point plan: {error}")))?;
    if extension.schema != EXTENSION_SCHEMA
        || extension.stage_evidence_path != STAGE_EVIDENCE_FILE
        || extension.plan_digest_blake3 != plan.plan_digest_blake3
        || extension.source_authority_digest_blake3 != plan.source_authority_digest_blake3
        || receipt.workflow_version != plan.receipt_contract.workflow_version
        || receipt.selected_provider_kind != plan.receipt_contract.selected_provider_kind
    {
        return Err(trust_error("receipt, extension, plan, or stage-evidence linkage does not match".to_string()));
    }
    validate_digest("extension proof bundle", &extension.final_proof_bundle_digest_blake3)?;
    validate_digest("receipt plan", &extension.plan_digest_blake3)?;
    assert_eq!(extension.plan_digest_blake3, plan.plan_digest_blake3);
    debug_assert_eq!(receipt.workflow_version, plan.receipt_contract.workflow_version);
    Ok(())
}

fn fixed_point_facts(
    receipt: DeterministicBuildProofReceipt,
    extension: SourceBuiltReceiptExtension,
    plan: &SourceBuiltFixedPointPlan,
    stages: &[SourceBuiltStageEvidence],
) -> Result<VerifiedFixedPointTrustFacts, RunError> {
    let planned_stage_count = bounded_count("planned stage", plan.stages.len())?;
    let observed_stage_count = bounded_count("observed stage", stages.len())?;
    let executed_stage_count = bounded_count(
        "executed stage",
        stages.iter().filter(|stage| stage.execution_origin == StageExecutionOrigin::Executed).count(),
    )?;
    let restored_stage_count = bounded_count(
        "restored stage",
        stages
            .iter()
            .filter(|stage| stage.execution_origin == StageExecutionOrigin::RestoredCheckpoint)
            .count(),
    )?;
    let authority_violation_count =
        bounded_sum("stage authority violation", stages.iter().map(|stage| stage.authority_violations.len()))?;
    let fallback_event_count =
        bounded_sum("stage fallback event", stages.iter().map(|stage| stage.fallback_events.len()))?;
    let substitution_count =
        bounded_sum("substitution", receipt.runs.iter().map(|run| run.substituted_dependency_identities.len()))?;
    let receipt_digest_blake3 = receipt
        .receipt_blake3
        .clone()
        .ok_or_else(|| trust_error("deterministic proof receipt has no receipt digest".to_string()))?;
    validate_digest("receipt", &receipt_digest_blake3)?;
    Ok(VerifiedFixedPointTrustFacts {
        workflow_version: receipt.workflow_version,
        verdict: "self-rebuild-match".to_string(),
        receipt_digest_blake3,
        plan_digest_blake3: extension.plan_digest_blake3,
        source_authority_digest_blake3: extension.source_authority_digest_blake3,
        proof_bundle_digest_blake3: extension.final_proof_bundle_digest_blake3,
        provider_kind: extension.provider_kind,
        hermeticity_mode: receipt.hermeticity_mode,
        planned_stage_count,
        observed_stage_count,
        executed_stage_count,
        restored_stage_count,
        authority_violation_count,
        fallback_event_count,
        substitution_count,
    })
}

fn observe_optional_action_trust(
    proof_root: &Path,
    fixed_point: &VerifiedFixedPointTrustFacts,
) -> Result<Option<RootActionTrustFacts>, RunError> {
    let receipt_path = proof_root.join(crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE);
    let receipt_value: serde_json::Value = read_json(&receipt_path, "deterministic proof receipt")?;
    let extension: SourceBuiltReceiptExtension = serde_json::from_value(
        receipt_value
            .get(EXTENSION_FIELD)
            .cloned()
            .ok_or_else(|| trust_error(format!("receipt is missing {EXTENSION_FIELD}")))?,
    )
    .map_err(|error| trust_error(format!("parsing source-built receipt extension: {error}")))?;
    match (extension.action_trust_plan_digest_blake3, extension.action_trust_reconciliation_digest_blake3) {
        (None, None) => Ok(None),
        (Some(plan_digest), Some(reconciliation_digest)) => {
            observe_action_trust(proof_root, &fixed_point.plan_digest_blake3, &plan_digest, &reconciliation_digest)
                .map(Some)
        }
        _ => Err(trust_error(
            "receipt binds only one of the action-trust plan and reconciliation digests".to_string(),
        )),
    }
}

fn observe_action_trust(
    proof_root: &Path,
    proof_plan_digest: &str,
    expected_plan_digest: &str,
    expected_reconciliation_digest: &str,
) -> Result<RootActionTrustFacts, RunError> {
    let plan_path = proof_root.join(ROOT_ACTION_TRUST_PLAN_FILE);
    let reconciliation_path = proof_root.join(ROOT_ACTION_RECONCILIATION_FILE);
    let observed_plan_digest = hash_regular_file(&plan_path)?;
    let observed_reconciliation_digest = hash_regular_file(&reconciliation_path)?;
    if observed_plan_digest != expected_plan_digest || observed_reconciliation_digest != expected_reconciliation_digest
    {
        return Err(trust_error("action-trust file digest does not match the receipt binding".to_string()));
    }
    let plan: RootActionTrustPlanSummary = read_json(&plan_path, "root action-trust plan")?;
    let reconciliation: RootActionReconciliationSummary =
        read_json(&reconciliation_path, "root action-trust reconciliation")?;
    validate_action_documents(&plan, &reconciliation, proof_plan_digest, &observed_plan_digest)?;
    Ok(RootActionTrustFacts {
        plan_path: ROOT_ACTION_TRUST_PLAN_FILE.to_string(),
        plan_digest_blake3: observed_plan_digest,
        reconciliation_path: ROOT_ACTION_RECONCILIATION_FILE.to_string(),
        reconciliation_digest_blake3: observed_reconciliation_digest,
        adapter_count: plan.adapter_count,
        planned_action_count: plan.action_count,
        matched_action_count: reconciliation.matched_action_count,
        observed_event_count: reconciliation.observed_event_count,
        matched_event_count: reconciliation.matched_event_count,
        unknown_event_count: reconciliation.unknown_event_count,
        missing_action_count: reconciliation.missing_action_count,
        authority_violation_count: reconciliation.authority_violation_count,
        fallback_event_count: reconciliation.fallback_event_count,
        remote_event_count: reconciliation.remote_event_count,
        cache_only_completion_count: reconciliation.cache_only_completion_count,
        local_only: plan.local_only && reconciliation.local_only,
    })
}

fn validate_action_documents(
    plan: &RootActionTrustPlanSummary,
    reconciliation: &RootActionReconciliationSummary,
    proof_plan_digest: &str,
    action_plan_digest: &str,
) -> Result<(), RunError> {
    let identity_valid = plan.schema == ROOT_ACTION_TRUST_PLAN_SCHEMA
        && reconciliation.schema == ROOT_ACTION_RECONCILIATION_SCHEMA
        && plan.proof_plan_digest_blake3 == proof_plan_digest
        && reconciliation.action_plan_digest_blake3 == action_plan_digest;
    let policy_valid = plan.local_only
        && !plan.cache_only_completion_allowed
        && plan.blockers.is_empty()
        && reconciliation.local_only
        && reconciliation.blockers.is_empty();
    let count_valid = plan.adapter_count > 0
        && plan.action_count > 0
        && plan.action_count <= ACTION_COUNT_MAX
        && reconciliation.planned_action_count == plan.action_count;
    if !identity_valid || !policy_valid || !count_valid {
        return Err(trust_error(
            "action-trust plan or reconciliation is incomplete, mismatched, or permits forbidden execution".to_string(),
        ));
    }
    assert_eq!(plan.action_count, reconciliation.planned_action_count);
    debug_assert!(plan.local_only && reconciliation.local_only);
    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, label: &str) -> Result<T, RunError> {
    let bytes = fs::read(path).map_err(|error| trust_error(format!("reading {label} {}: {error}", path.display())))?;
    if bytes.is_empty() {
        return Err(trust_error(format!("{label} is empty: {}", path.display())));
    }
    serde_json::from_slice(&bytes).map_err(|error| trust_error(format!("parsing {label} {}: {error}", path.display())))
}

fn hash_regular_file(path: &Path) -> Result<String, RunError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| trust_error(format!("reading action-trust file metadata {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() == 0 {
        return Err(trust_error(format!("action-trust evidence must be a nonempty regular file: {}", path.display())));
    }
    let mut file = fs::File::open(path)
        .map_err(|error| trust_error(format!("opening action-trust file {}: {error}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_BYTES];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| trust_error(format!("reading action-trust file {}: {error}", path.display())))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(metadata.len() > 0);
    Ok(digest)
}

fn validate_digest(label: &str, digest: &str) -> Result<(), RunError> {
    let valid = digest.len() == BLAKE3_HEX_LENGTH
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(trust_error(format!("{label} is not a lowercase BLAKE3 digest")));
    }
    Ok(())
}

fn bounded_count(label: &str, count: usize) -> Result<u32, RunError> {
    let count = u32::try_from(count).map_err(|_| trust_error(format!("{label} count exceeds u32")))?;
    if count > ACTION_COUNT_MAX {
        return Err(trust_error(format!("{label} count {count} exceeds {ACTION_COUNT_MAX}")));
    }
    Ok(count)
}

fn bounded_sum(label: &str, mut counts: impl Iterator<Item = usize>) -> Result<u32, RunError> {
    let total = counts.try_fold(0_usize, |total, count| {
        total.checked_add(count).ok_or_else(|| trust_error(format!("{label} count overflows usize")))
    })?;
    let bounded = bounded_count(label, total)?;
    assert!(bounded <= ACTION_COUNT_MAX);
    debug_assert_eq!(usize::try_from(bounded).ok(), Some(total));
    Ok(bounded)
}

fn trust_error(message: String) -> RunError {
    RunError::Build(format!("bootstrap trust report failed closed: {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TEST_ADAPTER_COUNT: u32 = 2;
    const TEST_ACTION_COUNT: u32 = 3;
    const TEST_EVENT_COUNT: u32 = 5;

    #[test]
    fn verified_v48_fixture_reports_fixed_point_without_action_trust() {
        let fixture = v48_fixture();

        let facts = observe_verified_fixed_point(fixture.path()).unwrap();
        let action_trust = observe_optional_action_trust(fixture.path(), &facts).unwrap();
        let report = build_bootstrap_trust_report(facts, action_trust).unwrap();

        assert!(report.fixed_point_verified);
        assert!(!report.root_action_trust_complete);
        assert_eq!(report.status, crate::source_built_trust_report::BootstrapTrustStatus::FixedPointOnly);
        assert_eq!(report.proof.verdict, "self-rebuild-match");
    }

    #[test]
    fn trust_report_rejects_tampered_stage_evidence() {
        let fixture = v48_fixture();
        fs::write(fixture.path().join(STAGE_EVIDENCE_FILE), "[]\n").unwrap();

        let error = observe_verified_fixed_point(fixture.path()).unwrap_err();

        assert!(error.to_string().contains("stage evidence digest mismatch"));
        assert!(fixture.path().join(crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE).is_file());
    }

    #[test]
    fn bounded_sum_rejects_count_overflow() {
        let error = bounded_sum("event", [usize::MAX, 1].into_iter()).unwrap_err();

        assert!(error.to_string().contains("overflows usize"));
        assert!(error.to_string().contains("failed closed"));
    }

    #[test]
    fn action_documents_bind_plan_and_complete_local_observations() {
        let plan = action_plan();
        let reconciliation = action_reconciliation();

        validate_action_documents(&plan, &reconciliation, DIGEST_A, DIGEST_B).unwrap();

        assert_eq!(plan.action_count, TEST_ACTION_COUNT);
        assert_eq!(reconciliation.matched_action_count, TEST_ACTION_COUNT);
        assert_eq!(reconciliation.matched_event_count, TEST_EVENT_COUNT);
    }

    #[test]
    fn action_documents_reject_cache_only_and_plan_digest_drift() {
        let mut plan = action_plan();
        let reconciliation = action_reconciliation();
        plan.cache_only_completion_allowed = true;
        plan.proof_plan_digest_blake3 = "c".repeat(BLAKE3_HEX_LENGTH);

        let error = validate_action_documents(&plan, &reconciliation, DIGEST_A, DIGEST_B).unwrap_err();

        assert!(error.to_string().contains("permits forbidden execution"));
        assert!(error.to_string().contains("failed closed"));
    }

    #[test]
    fn action_file_hash_rejects_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target.json");
        let link = temp.path().join("link.json");
        fs::write(&target, "{}").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let error = hash_regular_file(&link).unwrap_err();

        assert!(error.to_string().contains("regular file"));
        assert!(target.is_file());
    }

    fn v48_fixture() -> tempfile::TempDir {
        let fixture = tempfile::tempdir().unwrap();
        fs::write(
            fixture.path().join(PLAN_FILE),
            include_bytes!("../cairn/changes/prove-source-built-mantle-fixed-point/evidence/v48-promoted-fixed-point-success-2026-08-23/source-built-fixed-point-plan.json"),
        )
        .unwrap();
        fs::write(
            fixture.path().join(STAGE_EVIDENCE_FILE),
            include_bytes!("../cairn/changes/prove-source-built-mantle-fixed-point/evidence/v48-promoted-fixed-point-success-2026-08-23/source-built-stage-evidence.json"),
        )
        .unwrap();
        let bundle_digest =
            crate::source_built_fixed_point_receipt::proof_bundle_digest_for_test(fixture.path()).unwrap();
        let mut receipt: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../cairn/changes/prove-source-built-mantle-fixed-point/evidence/v48-promoted-fixed-point-success-2026-08-23/deterministic-build-proof.json"
        ))
        .unwrap();
        receipt[EXTENSION_FIELD]["final_proof_bundle_digest_blake3"] = serde_json::Value::String(bundle_digest);
        fs::write(
            fixture.path().join(crate::source_built_fixed_point_shell::FINAL_RECEIPT_FILE),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
        assert!(fixture.path().join(PLAN_FILE).is_file());
        assert!(fixture.path().join(STAGE_EVIDENCE_FILE).is_file());
        fixture
    }

    fn action_plan() -> RootActionTrustPlanSummary {
        RootActionTrustPlanSummary {
            schema: ROOT_ACTION_TRUST_PLAN_SCHEMA.to_string(),
            proof_plan_digest_blake3: DIGEST_A.to_string(),
            adapter_count: TEST_ADAPTER_COUNT,
            action_count: TEST_ACTION_COUNT,
            local_only: true,
            cache_only_completion_allowed: false,
            blockers: Vec::new(),
        }
    }

    fn action_reconciliation() -> RootActionReconciliationSummary {
        RootActionReconciliationSummary {
            schema: ROOT_ACTION_RECONCILIATION_SCHEMA.to_string(),
            action_plan_digest_blake3: DIGEST_B.to_string(),
            planned_action_count: TEST_ACTION_COUNT,
            matched_action_count: TEST_ACTION_COUNT,
            observed_event_count: TEST_EVENT_COUNT,
            matched_event_count: TEST_EVENT_COUNT,
            unknown_event_count: 0,
            missing_action_count: 0,
            authority_violation_count: 0,
            fallback_event_count: 0,
            remote_event_count: 0,
            cache_only_completion_count: 0,
            local_only: true,
            blockers: Vec::new(),
        }
    }
}
