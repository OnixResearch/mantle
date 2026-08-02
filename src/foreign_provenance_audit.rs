//! Receipt-bound foreign output provenance audit.
// machine-artifact-public: import.command-reports
//!
//! The admission and receipt cores are deterministic. The shell opens the
//! ordinary store and asks crunch-store to observe signed castore facts.
// r[impl foreign_derivation_import.provenance_audit_receipt]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use crunch_store::CastoreProvenanceRequest;
use crunch_store::CastoreProvenanceScan;
use crunch_store::ForeignProvenancePolicy;
use crunch_store::ProvenanceExpectedPathInfo;
use crunch_store::ProvenanceFinding;
use crunch_store::StoreConfig;
use crunch_store::StoreFallbackMode;
use crunch_store::StoreHandle;
use nix_compat::narinfo::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;

use crate::RunError;
use crate::build_correctness::OutputReferenceScanReport;
use crate::build_correctness::ReferenceObservation;
use crate::build_correctness::ReferenceScanInput;
use crate::build_correctness::validate_reference_scan;
use crate::foreign_executable_plan::ExecutableNativeUnit;
use crate::foreign_executable_plan::ForeignExecutablePlan;
use crate::foreign_executable_plan::validate_foreign_executable_plan;
use crate::foreign_realization_receipt::FOREIGN_REALIZATION_COMPLETE_STATUS;
use crate::foreign_realization_receipt::FOREIGN_REALIZATION_REALIZED_STATE;
use crate::foreign_realization_receipt::FOREIGN_REALIZATION_RECEIPT_SCHEMA;
use crate::foreign_realization_receipt::ForeignRealizationReceipt;
use crate::foreign_realization_receipt::foreign_realization_receipt_digest;

pub(crate) const FOREIGN_PROVENANCE_AUDIT_SCHEMA: &str = "mantle-foreign-provenance-audit-v1";
const FOREIGN_PROVENANCE_AUDIT_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-provenance-audit-v1\0";
const FOREIGN_PROVENANCE_OBSERVATION_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-provenance-observations-v1\0";
const FOREIGN_PROVENANCE_PATH_MAP_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-provenance-path-map-v1\0";
const FOREIGN_PROVENANCE_PROFILE_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-provenance-profiles-v1\0";
const FOREIGN_PROVENANCE_POLICY_DIGEST_DOMAIN: &[u8] = b"mantle-foreign-provenance-policy-v1\0";
const PASS_STATUS: &str = "pass";
const FAIL_STATUS: &str = "fail";
const PROVENANCE_AUDITED_STATE: &str = "provenance-audited";
const CASTORE_SCANNER_KIND: &str = "signed-castore-foreign-provenance-v1";
const AUDIT_NON_CLAIMS: [&str; 10] = [
    "This audit does not prove compiler correctness.",
    "This audit does not prove source correctness.",
    "This audit does not prove package correctness.",
    "This audit does not prove runtime behavior.",
    "This audit does not prove bootstrap parity.",
    "This audit does not prove reproducibility.",
    "This audit does not prove OS bootability.",
    "This audit does not prove deployment safety.",
    "This audit does not prove release eligibility.",
    "Static bounded classification does not prove complete dynamic dependency behavior.",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ForeignProvenanceAuditReceipt {
    pub(crate) schema: String,
    pub(crate) status: String,
    pub(crate) strongest_state: String,
    pub(crate) audit_blake3: String,
    pub(crate) realization_receipt_blake3: String,
    pub(crate) build_report_blake3: String,
    pub(crate) plan_blake3: String,
    pub(crate) path_map_blake3: String,
    pub(crate) profile_set_blake3: String,
    pub(crate) policy_blake3: String,
    pub(crate) selected_root_node_ids: Vec<String>,
    pub(crate) selected_root_paths: Vec<String>,
    pub(crate) closure_paths: Vec<String>,
    pub(crate) observation_blake3: String,
    pub(crate) policy: ForeignProvenancePolicy,
    pub(crate) scan: CastoreProvenanceScan,
    pub(crate) reference_scans: Vec<OutputReferenceScanReport>,
    pub(crate) findings: Vec<ProvenanceFinding>,
    pub(crate) disposition: String,
    pub(crate) non_claims: Vec<String>,
}

pub(crate) struct ForeignProvenanceAuditRequest<'a> {
    pub(crate) plan: &'a ForeignExecutablePlan,
    pub(crate) realization_receipt: &'a ForeignRealizationReceipt,
    pub(crate) policy: &'a ForeignProvenancePolicy,
    pub(crate) selected_root_node_ids: &'a [String],
    pub(crate) output_dir: &'a Path,
    pub(crate) state_dir: &'a Path,
    pub(crate) base_state_dirs: &'a [PathBuf],
    pub(crate) trusted_keys: &'a [VerifyingKey],
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AdmittedForeignAudit {
    selected_root_node_ids: Vec<String>,
    selected_root_paths: Vec<String>,
    closure_paths: Vec<String>,
    admitted_closure_paths: BTreeSet<String>,
    expected_path_infos: BTreeMap<String, ProvenanceExpectedPathInfo>,
    declared_references_by_output: BTreeMap<String, BTreeSet<String>>,
    foreign_to_target_paths: BTreeMap<String, String>,
}

pub(crate) async fn audit_foreign_realization(
    request: ForeignProvenanceAuditRequest<'_>,
) -> Result<ForeignProvenanceAuditReceipt, RunError> {
    let admitted =
        admit_foreign_audit(request.plan, request.realization_receipt, request.policy, request.selected_root_node_ids)?;
    let store = StoreHandle::open(StoreConfig {
        state_dir: request.state_dir.to_path_buf(),
        output_dir: request.output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        fallback_mode: StoreFallbackMode::Strict,
        store_dir: request.plan.target_store_prefix.clone(),
        base_state_dirs: request.base_state_dirs.to_vec(),
    })
    .await
    .map_err(|error| RunError::Internal(format!("opening foreign provenance audit store: {error}")))?;
    let scan = crunch_store::scan_castore_provenance(&store, CastoreProvenanceRequest {
        selected_root_paths: &admitted.closure_paths,
        admitted_closure_paths: &admitted.admitted_closure_paths,
        expected_path_infos: &admitted.expected_path_infos,
        declared_references_by_output: &admitted.declared_references_by_output,
        foreign_to_target_paths: &admitted.foreign_to_target_paths,
        trusted_keys: request.trusted_keys,
        policy: request.policy,
    })
    .await
    .map_err(|error| RunError::Internal(format!("scanning foreign castore provenance: {error}")))?;
    build_foreign_provenance_receipt(request.plan, request.realization_receipt, request.policy, admitted, scan)
}

fn admit_foreign_audit(
    plan: &ForeignExecutablePlan,
    receipt: &ForeignRealizationReceipt,
    policy: &ForeignProvenancePolicy,
    selected_root_node_ids: &[String],
) -> Result<AdmittedForeignAudit, RunError> {
    validate_foreign_executable_plan(plan).map_err(|diagnostic| {
        RunError::Internal(format!("foreign provenance plan is invalid: {}", diagnostic.class))
    })?;
    crunch_store::validate_foreign_provenance_policy(policy)
        .map_err(|error| RunError::Internal(format!("foreign provenance policy is invalid: {error}")))?;
    validate_realization_receipt(plan, receipt)?;
    let selected_roots = canonical_selected_roots(plan, receipt, selected_root_node_ids)?;
    let units = selected_unit_closure(plan, &selected_roots)?;
    let selected_root_paths = plan
        .selected_roots
        .iter()
        .filter(|root| selected_roots.binary_search(&root.node_id).is_ok())
        .flat_map(|root| root.target_outputs.values().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut admitted_closure_paths = BTreeSet::new();
    let mut declared_references_by_output = BTreeMap::new();
    for unit in &units {
        let declared = unit.declared_references.iter().cloned().collect::<BTreeSet<_>>();
        for output in unit.outputs.values() {
            admitted_closure_paths.insert(output.clone());
            declared_references_by_output.insert(output.clone(), declared.clone());
        }
        for source in &unit.input_sources {
            admitted_closure_paths.insert(source.clone());
        }
    }
    let expected_path_infos = expected_path_info_facts(receipt);
    for path in &admitted_closure_paths {
        if !expected_path_infos.contains_key(path) {
            return Err(RunError::Internal(format!(
                "foreign provenance closure path has no realization NAR fact: {path}"
            )));
        }
    }
    let mut foreign_to_target_paths = plan.exact_path_maps.outputs.clone();
    for (foreign, target) in &plan.exact_path_maps.sources {
        if foreign_to_target_paths.insert(foreign.clone(), target.clone()).is_some() {
            return Err(RunError::Internal(format!(
                "foreign provenance path map repeats a foreign output or source: {foreign}"
            )));
        }
    }
    let closure_paths = admitted_closure_paths.iter().cloned().collect::<Vec<_>>();
    assert!(!selected_root_paths.is_empty());
    assert!(!closure_paths.is_empty());
    Ok(AdmittedForeignAudit {
        selected_root_node_ids: selected_roots,
        selected_root_paths,
        closure_paths,
        admitted_closure_paths,
        expected_path_infos,
        declared_references_by_output,
        foreign_to_target_paths,
    })
}

fn validate_realization_receipt(
    plan: &ForeignExecutablePlan,
    receipt: &ForeignRealizationReceipt,
) -> Result<(), RunError> {
    if receipt.schema != FOREIGN_REALIZATION_RECEIPT_SCHEMA {
        return Err(RunError::Internal(format!(
            "foreign provenance receipt schema must be {FOREIGN_REALIZATION_RECEIPT_SCHEMA}"
        )));
    }
    if receipt.status != FOREIGN_REALIZATION_COMPLETE_STATUS
        || receipt.strongest_state != FOREIGN_REALIZATION_REALIZED_STATE
        || receipt.failure.is_some()
    {
        return Err(RunError::Internal("foreign provenance audit requires one complete realized receipt".to_string()));
    }
    if foreign_realization_receipt_digest(receipt)? != receipt.receipt_blake3 {
        return Err(RunError::Internal("foreign provenance realization receipt self-digest is stale".to_string()));
    }
    if receipt.plan_blake3 != plan.plan_identity.value
        || receipt.import_receipt_blake3 != plan.accepted_import.receipt_digest_blake3
    {
        return Err(RunError::Internal(
            "foreign provenance realization receipt does not bind the executable plan".to_string(),
        ));
    }
    Ok(())
}

fn canonical_selected_roots(
    plan: &ForeignExecutablePlan,
    receipt: &ForeignRealizationReceipt,
    selected_root_node_ids: &[String],
) -> Result<Vec<String>, RunError> {
    if selected_root_node_ids.is_empty() {
        return Err(RunError::Internal("foreign provenance audit requires explicit selected-root IDs".to_string()));
    }
    let mut selected = selected_root_node_ids.to_vec();
    let original_count = selected.len();
    selected.sort();
    selected.dedup();
    if selected.len() != original_count {
        return Err(RunError::Internal("foreign provenance selected-root set contains a duplicate".to_string()));
    }
    let plan_roots = plan.selected_roots.iter().map(|root| root.node_id.as_str()).collect::<BTreeSet<_>>();
    let receipt_roots = receipt.selected_root_node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for root in &selected {
        if !plan_roots.contains(root.as_str()) || !receipt_roots.contains(root.as_str()) {
            return Err(RunError::Internal(format!(
                "foreign provenance selected root is not plan-and-receipt admitted: {root}"
            )));
        }
    }
    Ok(selected)
}

fn selected_unit_closure<'a>(
    plan: &'a ForeignExecutablePlan,
    selected_roots: &[String],
) -> Result<Vec<&'a ExecutableNativeUnit>, RunError> {
    let by_derivation = plan
        .native_units
        .iter()
        .map(|unit| (unit.target_derivation.as_str(), unit))
        .collect::<BTreeMap<_, _>>();
    let mut pending = plan
        .selected_roots
        .iter()
        .filter(|root| selected_roots.binary_search(&root.node_id).is_ok())
        .map(|root| root.target_derivation.clone())
        .collect::<BTreeSet<_>>();
    let mut selected_derivations = BTreeSet::new();
    while let Some(derivation) = pending.pop_first() {
        if !selected_derivations.insert(derivation.clone()) {
            continue;
        }
        let unit = by_derivation.get(derivation.as_str()).ok_or_else(|| {
            RunError::Internal(format!("foreign provenance selected derivation has no plan unit: {derivation}"))
        })?;
        for dependency in unit.input_derivations.keys() {
            pending.insert(dependency.clone());
        }
    }
    let units = plan
        .native_units
        .iter()
        .filter(|unit| selected_derivations.contains(&unit.target_derivation))
        .collect::<Vec<_>>();
    if units.is_empty() {
        return Err(RunError::Internal("foreign provenance selected unit closure is empty".to_string()));
    }
    Ok(units)
}

fn expected_path_info_facts(receipt: &ForeignRealizationReceipt) -> BTreeMap<String, ProvenanceExpectedPathInfo> {
    let mut facts = BTreeMap::new();
    for source in &receipt.sources {
        facts.insert(source.target_path.clone(), ProvenanceExpectedPathInfo {
            nar_sha256: source.nar_sha256.clone(),
            nar_size: source.nar_size,
        });
    }
    for unit in &receipt.units {
        for output in &unit.outputs {
            facts.insert(output.target_path.clone(), ProvenanceExpectedPathInfo {
                nar_sha256: output.nar_sha256.clone(),
                nar_size: output.nar_size,
            });
        }
    }
    facts
}

fn build_foreign_provenance_receipt(
    plan: &ForeignExecutablePlan,
    realization_receipt: &ForeignRealizationReceipt,
    policy: &ForeignProvenancePolicy,
    admitted: AdmittedForeignAudit,
    scan: CastoreProvenanceScan,
) -> Result<ForeignProvenanceAuditReceipt, RunError> {
    let reference_scans = build_correctness_reference_scans(policy, &admitted, &scan);
    let mut findings = scan.findings.clone();
    for report in &reference_scans {
        for diagnostic in &report.diagnostics {
            findings.push(ProvenanceFinding {
                code: "build-correctness-reference-rejection".to_string(),
                path: report.output_object_ref.clone(),
                detail: diagnostic.clone(),
            });
        }
    }
    findings.sort();
    findings.dedup();
    let passed = scan.preflight_complete
        && scan.traversal_complete
        && findings.is_empty()
        && reference_scans.iter().all(|report| report.status == "accepted");
    let disposition = if passed { PASS_STATUS } else { FAIL_STATUS };
    let observation_blake3 = domain_digest(
        FOREIGN_PROVENANCE_OBSERVATION_DIGEST_DOMAIN,
        &(scan.clone(), reference_scans.clone()),
        "foreign provenance observations",
    )?;
    let path_map_blake3 =
        domain_digest(FOREIGN_PROVENANCE_PATH_MAP_DIGEST_DOMAIN, &plan.exact_path_maps, "foreign provenance path map")?;
    let profile_set_blake3 = domain_digest(
        FOREIGN_PROVENANCE_PROFILE_DIGEST_DOMAIN,
        &realization_receipt.execution_profiles,
        "foreign provenance profiles",
    )?;
    let policy_blake3 = domain_digest(FOREIGN_PROVENANCE_POLICY_DIGEST_DOMAIN, policy, "foreign provenance policy")?;
    let mut receipt = ForeignProvenanceAuditReceipt {
        schema: FOREIGN_PROVENANCE_AUDIT_SCHEMA.to_string(),
        status: disposition.to_string(),
        strongest_state: if passed {
            PROVENANCE_AUDITED_STATE.to_string()
        } else {
            FOREIGN_REALIZATION_REALIZED_STATE.to_string()
        },
        audit_blake3: String::new(),
        realization_receipt_blake3: realization_receipt.receipt_blake3.clone(),
        build_report_blake3: realization_receipt.build_report_blake3.clone(),
        plan_blake3: plan.plan_identity.value.clone(),
        path_map_blake3,
        profile_set_blake3,
        policy_blake3,
        selected_root_node_ids: admitted.selected_root_node_ids,
        selected_root_paths: admitted.selected_root_paths,
        closure_paths: admitted.closure_paths,
        observation_blake3,
        policy: policy.clone(),
        scan,
        reference_scans,
        findings,
        disposition: disposition.to_string(),
        non_claims: AUDIT_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    receipt.audit_blake3 = foreign_provenance_audit_digest(&receipt)?;
    Ok(receipt)
}

fn build_correctness_reference_scans(
    policy: &ForeignProvenancePolicy,
    admitted: &AdmittedForeignAudit,
    scan: &CastoreProvenanceScan,
) -> Vec<OutputReferenceScanReport> {
    let mut reports = Vec::new();
    for (output, declared) in &admitted.declared_references_by_output {
        let mut declared = declared.iter().cloned().collect::<Vec<_>>();
        declared.extend(policy.allowed_profile_paths.iter().cloned());
        declared.push(output.clone());
        declared.sort();
        declared.dedup();
        let observations = scan
            .references
            .iter()
            .filter(|observation| observation.owner_store_path == *output)
            .map(|observation| ReferenceObservation {
                ref_value: observation.reference.clone(),
                view: observation.view.clone(),
            })
            .collect::<Vec<_>>();
        let report = validate_reference_scan(ReferenceScanInput {
            output_object_ref: output.clone(),
            scan_root_ref: output.clone(),
            scanner_kind: CASTORE_SCANNER_KIND.to_string(),
            declared_refs: declared,
            forbidden_refs: admitted.foreign_to_target_paths.keys().cloned().collect(),
            observations,
        })
        .unwrap_or_else(|report| *report);
        reports.push(report);
    }
    reports.sort_by(|left, right| left.output_object_ref.cmp(&right.output_object_ref));
    reports
}

pub(crate) fn foreign_provenance_audit_digest(receipt: &ForeignProvenanceAuditReceipt) -> Result<String, RunError> {
    let mut material = receipt.clone();
    material.audit_blake3.clear();
    domain_digest(FOREIGN_PROVENANCE_AUDIT_DIGEST_DOMAIN, &material, "foreign provenance audit receipt")
}

fn domain_digest<T: Serialize>(domain: &[u8], value: &T, label: &str) -> Result<String, RunError> {
    let canonical =
        serde_json::to_vec(value).map_err(|error| RunError::Internal(format!("serializing {label}: {error}")))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&canonical);
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foreign_derivation_import::ForeignDerivationGraph;
    use crate::foreign_derivation_import::PackageIndex;
    use crate::foreign_derivation_import::TranslationPolicy;
    use crate::foreign_executable_plan::compile_foreign_executable_plan;
    use crate::foreign_realization_receipt::ForeignRealizedOutput;
    use crate::foreign_realization_receipt::ForeignRealizedUnit;
    use crate::foreign_realization_receipt::RealizedCachePolicy;
    use crate::foreign_realization_receipt::RealizedExecutionProfile;
    use crate::foreign_realization_shell::ForeignSourceStoreFact;

    const GRAPH: &str = include_str!("../tests/fixtures/foreign-import/nix-hello.graph.json");
    const INDEX: &str = include_str!("../tests/fixtures/foreign-import/nix-hello.index.json");
    const TRANSLATION_POLICY: &str = include_str!("../tests/fixtures/foreign-import/policy.json");
    const TEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn fixture_plan() -> ForeignExecutablePlan {
        let graph: ForeignDerivationGraph = serde_json::from_str(GRAPH).unwrap();
        let index: PackageIndex = serde_json::from_str(INDEX).unwrap();
        let policy: TranslationPolicy = serde_json::from_str(TRANSLATION_POLICY).unwrap();
        compile_foreign_executable_plan(&graph, &index, &policy, "hello", "x86_64-linux").unwrap().0
    }

    fn fixture_receipt(plan: &ForeignExecutablePlan) -> ForeignRealizationReceipt {
        let profiles = plan
            .native_units
            .iter()
            .map(|unit| (unit.execution_profile_id.clone(), unit.execution_profile_digest_blake3.clone()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|(profile_id, digest_blake3)| RealizedExecutionProfile {
                profile_id,
                digest_blake3,
            })
            .collect();
        let sources = plan
            .source_requirements
            .iter()
            .map(|source| ForeignSourceStoreFact {
                payload_id: source.payload_id.clone(),
                foreign_path: source.foreign_path.clone(),
                target_path: source.target_path.clone(),
                descriptor_blake3: source.descriptor_digest.clone(),
                source_record_identity: TEST_DIGEST.to_string(),
                source_content_blake3: TEST_DIGEST.to_string(),
                nar_sha256: TEST_DIGEST.to_string(),
                nar_size: 1,
            })
            .collect();
        let units = plan
            .native_units
            .iter()
            .map(|unit| ForeignRealizedUnit {
                sequence: unit.sequence,
                node_id: unit.node_id.clone(),
                target_derivation: unit.target_derivation.clone(),
                execution_profile_id: unit.execution_profile_id.clone(),
                execution_profile_digest_blake3: unit.execution_profile_digest_blake3.clone(),
                execution_class: "built".to_string(),
                fetch_attempts: Vec::new(),
                outputs: unit
                    .outputs
                    .iter()
                    .map(|(output_name, target_path)| ForeignRealizedOutput {
                        output_name: output_name.clone(),
                        target_path: target_path.clone(),
                        nar_sha256: TEST_DIGEST.to_string(),
                        nar_size: 1,
                        substitution_mode: None,
                        substitution_transferred_bytes: None,
                        substitution_reused_bytes: None,
                    })
                    .collect(),
                failure: None,
            })
            .collect();
        let mut receipt = ForeignRealizationReceipt {
            schema: FOREIGN_REALIZATION_RECEIPT_SCHEMA.to_string(),
            status: FOREIGN_REALIZATION_COMPLETE_STATUS.to_string(),
            strongest_state: FOREIGN_REALIZATION_REALIZED_STATE.to_string(),
            receipt_blake3: String::new(),
            plan_blake3: plan.plan_identity.value.clone(),
            import_receipt_blake3: plan.accepted_import.receipt_digest_blake3.clone(),
            source_bundle_manifest_blake3: TEST_DIGEST.to_string(),
            build_report_blake3: TEST_DIGEST.to_string(),
            selected_root_node_ids: plan.selected_roots.iter().map(|root| root.node_id.clone()).collect(),
            selected_root_paths: plan
                .selected_roots
                .iter()
                .flat_map(|root| root.target_outputs.values().cloned())
                .collect(),
            execution_profiles: profiles,
            cache_policy: RealizedCachePolicy {
                substitution_enabled: false,
                offline: true,
                ordered_cache_urls: Vec::new(),
            },
            sources,
            units,
            failure: None,
            non_claims: Vec::new(),
        };
        receipt.receipt_blake3 = foreign_realization_receipt_digest(&receipt).unwrap();
        receipt
    }

    fn policy() -> ForeignProvenancePolicy {
        ForeignProvenancePolicy {
            schema: crunch_store::FOREIGN_PROVENANCE_POLICY_SCHEMA.to_string(),
            max_path_infos: 32,
            max_nodes: 128,
            max_blobs: 64,
            max_blob_bytes: 1_048_576,
            max_total_bytes: 4_194_304,
            max_depth: 32,
            max_findings: 64,
            max_duplicates: 32,
            max_container_entries: 64,
            max_container_expanded_bytes: 1_048_576,
            max_container_depth: 4,
            max_path_bytes: 4096,
            max_shebang_bytes: 256,
            allowed_profile_paths: vec!["/bin/sh".to_string()],
        }
    }

    #[test]
    fn audit_admission_rejects_stale_receipt_and_unknown_root() {
        let plan = fixture_plan();
        let mut receipt = fixture_receipt(&plan);
        receipt.receipt_blake3 = "0".repeat(TEST_DIGEST.len());
        let root = vec![plan.selected_roots[0].node_id.clone()];
        assert!(admit_foreign_audit(&plan, &receipt, &policy(), &root).is_err());

        let receipt = fixture_receipt(&plan);
        assert!(admit_foreign_audit(&plan, &receipt, &policy(), &["unknown".to_string()]).is_err());
    }

    #[test]
    fn audit_receipt_digest_is_deterministic_and_tamper_evident() {
        let plan = fixture_plan();
        let receipt = fixture_receipt(&plan);
        let root = vec![plan.selected_roots[0].node_id.clone()];
        let admitted = admit_foreign_audit(&plan, &receipt, &policy(), &root).unwrap();
        let scan = CastoreProvenanceScan {
            preflight_complete: true,
            traversal_complete: true,
            path_infos: Vec::new(),
            payloads: Vec::new(),
            references: Vec::new(),
            findings: Vec::new(),
            visited_node_count: 1,
            visited_blob_count: 1,
            read_byte_count: 1,
            duplicate_node_count: 0,
        };
        let first =
            build_foreign_provenance_receipt(&plan, &receipt, &policy(), admitted.clone(), scan.clone()).unwrap();
        let second = build_foreign_provenance_receipt(&plan, &receipt, &policy(), admitted, scan).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.status, PASS_STATUS);
        assert_eq!(first.strongest_state, PROVENANCE_AUDITED_STATE);
        let mut tampered = first.clone();
        tampered.status = FAIL_STATUS.to_string();
        assert_ne!(foreign_provenance_audit_digest(&tampered).unwrap(), first.audit_blake3);
    }

    #[test]
    fn failed_audit_preserves_realized_state_and_non_claims() {
        let plan = fixture_plan();
        let receipt = fixture_receipt(&plan);
        let root = vec![plan.selected_roots[0].node_id.clone()];
        let admitted = admit_foreign_audit(&plan, &receipt, &policy(), &root).unwrap();
        let finding = ProvenanceFinding {
            code: "untranslated-foreign-path".to_string(),
            path: "fixture".to_string(),
            detail: "/gnu/store/fixture".to_string(),
        };
        let scan = CastoreProvenanceScan {
            preflight_complete: true,
            traversal_complete: true,
            path_infos: Vec::new(),
            payloads: Vec::new(),
            references: Vec::new(),
            findings: vec![finding],
            visited_node_count: 1,
            visited_blob_count: 1,
            read_byte_count: 1,
            duplicate_node_count: 0,
        };
        let audit = build_foreign_provenance_receipt(&plan, &receipt, &policy(), admitted, scan).unwrap();
        assert_eq!(audit.status, FAIL_STATUS);
        assert_eq!(audit.strongest_state, FOREIGN_REALIZATION_REALIZED_STATE);
        assert_eq!(audit.build_report_blake3, receipt.build_report_blake3);
        assert_eq!(audit.non_claims.len(), AUDIT_NON_CLAIMS.len());
    }
}
