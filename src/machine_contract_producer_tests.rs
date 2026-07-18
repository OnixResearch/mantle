use std::fs;
use std::path::PathBuf;

use mantle::fresh_clone_fixed_point::HydratedFreshCloneFixedPointReport;
use mantle::fresh_clone_fixed_point::HydratedFreshCloneReportInput;
use mantle::fresh_clone_fixed_point::HydratedFreshCloneStageReport;
use serde::Serialize;
use serde_json::Value;

use crate::build_plan::BuildPlanReport;
use crate::build_report::BuildJsonCounts;
use crate::build_report::BuildJsonReport;
use crate::nickel_export::ExportSourceRef;
use crate::nickel_export::NickelEvaluatorDescriptor;
use crate::nickel_export::NickelExportReceipt;
use crate::nickel_export::NickelExportReport;
use crate::operator_diagnostics::DoctorProfile;
use crate::operator_diagnostics::PreflightCheck;
use crate::operator_diagnostics::PreflightReport;
use crate::operator_diagnostics::PreflightStatus;
use crate::portable_receipt::ClaimStrengthRequest;
use crate::portable_receipt::ReceiptBundleReport;
use crate::portable_receipt::ReceiptImportReport;
use crate::portable_receipt::ReceiptRecordKind;
use crate::portable_receipt::ReceiptRecordSummary;
use crate::portable_receipt::ReceiptVerifyReport;
use crate::realization_routing::ClaimStrength;
use crate::realization_routing::NetworkPolicy;
use crate::realization_routing::RouteClass;
use crate::realization_routing::RoutePlanReport;
use crate::realization_routing::RoutePolicyReport;
use crate::realization_routing::RouteRejection;
use crate::source_bundle::SelfBuildHydrationReport;
use crate::source_bundle::SourceBundlePlanReport;
use crate::source_bundle::SourceBundleVerifyReport;
use crate::source_bundle::SourceOfflinePreflightReport;
use crate::source_bundle::SourceReadiness;
use crate::source_bundle::SourceRecordKind;
use crate::source_bundle::SourceRecordSummary;

const FIXTURE_ROOT: &str = "schemas/machine-contracts/fixtures";
const RECEIPT_NON_CLAIM: &str =
    "receipt bundle evidence proves only matched receipt, attestation, graph, and trust-basis facts";
const SOURCE_NON_CLAIM: &str = "source bundle evidence proves declared source/input availability and identity only";
const NICKEL_NON_CLAIM: &str = "Nickel export success proves only the declared evaluation output digest under the recorded evaluator descriptor; it does not prove deployability, frontend correctness, or build success";
const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const SOURCE_PAYLOAD_BYTES: u64 = 12;
const FRESH_SOURCE_OVERRIDE_COUNT: u32 = 12;
const STAGE0_FALLBACK_EVENT_COUNT: u32 = 2;
const SCHEDULER_FIXTURE_SELECTED_GOAL: &str = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-selected.drv";
const SCHEDULER_FIXTURE_RUNNER_UP_GOAL: &str = "/mantle/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-runner.drv";
const SCHEDULER_FIXTURE_EPOCH: u32 = 1;
const SCHEDULER_FIXTURE_SELECTED_PATH_NODES: u32 = 2;
const SCHEDULER_FIXTURE_RUNNER_UP_PATH_NODES: u32 = 1;

fn scheduler_fixture_decision() -> crunch_pipeline::PriorityDecisionEvidence {
    let policy = crunch_pipeline::SchedulingPolicy::default();
    let ready = [
        crunch_build::ReadyGoalFacts::ordinary(SCHEDULER_FIXTURE_RUNNER_UP_GOAL.to_string(), 0),
        crunch_build::ReadyGoalFacts::ordinary(SCHEDULER_FIXTURE_SELECTED_GOAL.to_string(), 0),
    ];
    let pressures = std::collections::BTreeMap::from([
        (SCHEDULER_FIXTURE_SELECTED_GOAL.to_string(), crunch_build::KnownGraphPressure {
            known_critical_path_nodes: SCHEDULER_FIXTURE_SELECTED_PATH_NODES,
            known_critical_path_work_units: SCHEDULER_FIXTURE_SELECTED_PATH_NODES,
            blocked_root_count: SCHEDULER_FIXTURE_EPOCH,
            blocked_root_count_saturated: false,
        }),
        (SCHEDULER_FIXTURE_RUNNER_UP_GOAL.to_string(), crunch_build::KnownGraphPressure {
            known_critical_path_nodes: SCHEDULER_FIXTURE_RUNNER_UP_PATH_NODES,
            known_critical_path_work_units: SCHEDULER_FIXTURE_RUNNER_UP_PATH_NODES,
            blocked_root_count: SCHEDULER_FIXTURE_EPOCH,
            blocked_root_count_saturated: false,
        }),
    ]);
    let ranked = crunch_build::rank_ready_goals(&policy, SCHEDULER_FIXTURE_EPOCH, &ready, &pressures)
        .expect("rank scheduler fixture candidates");
    crunch_build::priority_decision_evidence(
        &policy,
        SCHEDULER_FIXTURE_EPOCH,
        &ranked,
        crunch_build::HistoryBasis::StructuralFallbackMissing,
        None,
    )
    .expect("construct scheduler fixture evidence")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_ROOT).join(name)
}

fn assert_fixture<T: Serialize>(name: &str, value: &T) {
    let expected: Value =
        serde_json::from_str(&fs::read_to_string(fixture_path(name)).expect("read fixture")).expect("parse fixture");
    let actual = serde_json::to_value(value).expect("serialize Rust producer value");
    assert_eq!(actual, expected, "Rust producer fixture drift: {name}");
}

#[test]
fn doctor_and_build_reports_serialize_to_registered_positive_fixtures() {
    let doctor = PreflightReport {
        schema: "crunch-doctor-report-v1",
        command: "doctor",
        profile: DoctorProfile::Build,
        ok: true,
        checks: vec![PreflightCheck {
            id: "bwrap",
            status: PreflightStatus::Ok,
            summary: "bwrap visible".to_string(),
            detail: None,
        }],
    };
    assert_fixture("doctor-report-valid.json", &doctor);

    let build = BuildJsonReport {
        schema: "crunch-build-report-v1",
        file: "examples/hello.ncl".to_string(),
        output_dir: "/mantle/store".to_string(),
        state_dir: ".mantle/state".to_string(),
        store_dir: "/mantle/store".to_string(),
        scheduler_policy: crunch_pipeline::SchedulingPolicy::default(),
        hermeticity_mode: "strict".to_string(),
        hermeticity_audit_events: Vec::new(),
        build_environment_reports: Vec::new(),
        network_policy_reports: Vec::new(),
        workspace_reports: Vec::new(),
        action_result_reports: vec![crunch_build::ActionResultRuntimeReport {
            schema: "mantle-action-result-runtime-report-v1".to_string(),
            phase: "discovery".to_string(),
            action_ref: "mantle-action://blake3/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            disposition: "reused".to_string(),
            selected_result_ref: Some(
                "mantle-action-result://blake3/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                    .to_string(),
            ),
            selected_source_id: Some("local-action-results".to_string()),
            selected_source_class: Some("local".to_string()),
            trust_basis: vec!["record-signature-verified:fixture-key-1".to_string()],
            conflict_class: None,
            candidate_decisions: vec![crunch_action_result_core::CandidateDecision {
                result_ref:
                    "mantle-action-result://blake3/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
                        .to_string(),
                source_id: "local-action-results".to_string(),
                source_class: "local".to_string(),
                admitted: true,
                diagnostics: Vec::new(),
                trust_basis: vec!["record-signature-verified:fixture-key-1".to_string()],
                output_set_digest_blake3: Some(
                    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
                ),
            }],
            publication_result_refs: Vec::new(),
            transfer: None,
            diagnostics: Vec::new(),
            non_claims: vec!["index-presence-is-not-output-trust".to_string()],
        }],
        native_dynamic_plans: Vec::new(),
        scheduler_priority_decisions: vec![scheduler_fixture_decision()],
        remote_telemetry_events: Vec::new(),
        remote_observability: None,
        frontend_artifact_attestations: Vec::new(),
        cargo_build_evidence: Vec::new(),
        cargo_build_evidence_diagnostics: Vec::new(),
        ast_grep_structural_evidence: Vec::new(),
        ast_grep_structural_evidence_diagnostics: Vec::new(),
        diagnostic_persistence_failures: Vec::new(),
        counts: BuildJsonCounts {
            succeeded_total: 0,
            built_total: 0,
            cached_total: 0,
            failed_total: 0,
        },
        outcomes: Vec::new(),
        failed: Vec::new(),
        fod_mismatches: Vec::new(),
    };
    assert_fixture("build-json-report.valid.json", &build);

    let plan = BuildPlanReport {
        schema: "crunch-build-plan-v1",
        file: "examples/hello.ncl".to_string(),
        output_dir: "/mantle/store".to_string(),
        state_dir: ".mantle/state".to_string(),
        store_dir: "/mantle/store".to_string(),
        entries: Vec::new(),
    };
    assert_fixture("build-plan-report.valid.json", &plan);
}

#[test]
fn route_report_serializes_to_registered_positive_fixture() {
    let route = RoutePlanReport {
        schema: "mantle-realization-route-plan-v1",
        selected_route: RouteClass::LocalBuild,
        selected_reason_code: "local-build-eligible".to_string(),
        selected_detail: Some("local sandbox is available".to_string()),
        policy: RoutePolicyReport {
            network: NetworkPolicy::Offline,
            requested_claim_strength: ClaimStrength::Practical,
        },
        tie_breaker: "local-cache,trusted-substitute,archive-import,source-bundle,p2p-remote-builder,local-build,preflight-error",
        rejected_routes: vec![RouteRejection {
            route: RouteClass::TrustedSubstitute,
            reason_code: "network-disabled".to_string(),
            detail: None,
        }],
        upload_summary: None,
        non_claim: "route planning is advisory and does not prove transport execution or artifact correctness",
    };
    assert_fixture("route-plan-report.valid.json", &route);
}

#[test]
fn portable_receipt_reports_serialize_to_registered_positive_fixtures() {
    let bundle = ReceiptBundleReport {
        format: "mantle-build-receipt-bundle-v1",
        store_prefix: "/mantle/store".to_string(),
        claim_strength: ClaimStrengthRequest::Diagnostic,
        evidence_complete: false,
        missing_evidence: vec![ReceiptRecordKind::ActionRef],
        output_identities: Vec::new(),
        records: vec![ReceiptRecordSummary {
            kind: ReceiptRecordKind::SourceRef,
            identity: "source/example".to_string(),
            digest: format!("blake3:{DIGEST_A}"),
        }],
        record_count: 1,
        bundle_blake3: DIGEST_B.to_string(),
        non_claim: RECEIPT_NON_CLAIM,
    };
    assert_fixture("receipt-bundle-report.valid.json", &bundle);

    let verify = ReceiptVerifyReport {
        format: "mantle-build-receipt-bundle-v1",
        store_prefix: "/mantle/store".to_string(),
        policy_hash: format!("blake3:{DIGEST_A}"),
        expected_policy_hash: format!("blake3:{DIGEST_A}"),
        policy_hash_matched: true,
        valid_at_unix_s: 1,
        trust_window_valid: true,
        public_key_digests: vec![DIGEST_B.to_string()],
        revocation_ref: None,
        bundle_blake3: DIGEST_C.to_string(),
        evidence_complete: true,
        missing_evidence: Vec::new(),
        output_matches: Vec::new(),
        source_matches: Vec::new(),
        signature_matches: Vec::new(),
        non_claim: RECEIPT_NON_CLAIM,
    };
    assert_fixture("receipt-verify-report.valid.json", &verify);

    let import = ReceiptImportReport {
        imported: true,
        idempotent: false,
        bundle_imported: true,
        graph_imported: true,
        graph_nodes_imported: 1,
        graph_edges_imported: 0,
        attestation_sidecars_imported: 0,
        bundle_blake3: DIGEST_A.to_string(),
        non_claim: RECEIPT_NON_CLAIM,
    };
    assert_fixture("receipt-import-report.valid.json", &import);
}

#[test]
fn source_reports_serialize_to_registered_positive_fixtures() {
    let record = SourceRecordSummary {
        kind: SourceRecordKind::LocalPath,
        identity: "source/example".to_string(),
        payload_bytes: SOURCE_PAYLOAD_BYTES,
        content_blake3: DIGEST_A.to_string(),
        file_count: 1,
    };
    let plan = SourceBundlePlanReport {
        format: "mantle-source-bundle-v1",
        store_prefix: "/mantle/store".to_string(),
        record_count: 1,
        payload_bytes: SOURCE_PAYLOAD_BYTES,
        ready_class: SourceReadiness::Ready,
        records: vec![record],
        non_claim: SOURCE_NON_CLAIM,
    };
    assert_fixture("source-bundle-plan-report.valid.json", &plan);

    let verify = SourceBundleVerifyReport {
        manifest_blake3: DIGEST_A.to_string(),
        ready_class: SourceReadiness::Ready,
        missing_records: Vec::new(),
        stale_records: Vec::new(),
        unsupported_records: Vec::new(),
        untrusted_records: Vec::new(),
        non_claim: SOURCE_NON_CLAIM,
    };
    assert_fixture("source-bundle-verify-report.valid.json", &verify);

    let hydration = SelfBuildHydrationReport {
        format: "mantle-self-build-source-hydration-v1",
        manifest_blake3: DIGEST_A.to_string(),
        vendor_content_blake3: DIGEST_B.to_string(),
        provider_archive_content_blake3: DIGEST_C.to_string(),
        imported_record_count: 3,
        existing_record_count: 0,
        pinned: true,
        non_claim: SOURCE_NON_CLAIM,
    };
    assert_fixture("self-build-source-hydration-report.valid.json", &hydration);

    let stage0 = HydratedFreshCloneStageReport {
        source_policy: "require-override".to_string(),
        source_override_count: FRESH_SOURCE_OVERRIDE_COUNT,
        live_fetch_events: 0,
        hermeticity_mode: "practical".to_string(),
        fallback_event_count: STAGE0_FALLBACK_EVENT_COUNT,
    };
    let stage2 = HydratedFreshCloneStageReport {
        source_policy: "require-override".to_string(),
        source_override_count: FRESH_SOURCE_OVERRIDE_COUNT,
        live_fetch_events: 0,
        hermeticity_mode: "strict".to_string(),
        fallback_event_count: 0,
    };
    let fixed_point = HydratedFreshCloneFixedPointReport::from_input(HydratedFreshCloneReportInput {
        expected_manifest_blake3: DIGEST_A.to_string(),
        source_state_blake3: DIGEST_B.to_string(),
        hydration_report_blake3: DIGEST_C.to_string(),
        staged_source_store_name: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-mantle-src".to_string(),
        provider_kind: "musl.cc-native-reduced-v1".to_string(),
        platform: "x86_64-linux".to_string(),
        proof_mode: "fixed-point".to_string(),
        stage0,
        stage2,
        stage1_binary_blake3: DIGEST_D.to_string(),
        stage2_binary_blake3: DIGEST_D.to_string(),
    })
    .unwrap();
    assert_fixture("hydrated-fresh-clone-fixed-point-report.valid.json", &fixed_point);

    let preflight = SourceOfflinePreflightReport {
        format: "mantle-source-offline-preflight-v1",
        manifest_blake3: Some(DIGEST_A.to_string()),
        source_state_blake3: DIGEST_B.to_string(),
        ready_class: SourceReadiness::Ready,
        record_count: 1,
        missing_records: Vec::new(),
        stale_records: Vec::new(),
        unsupported_records: Vec::new(),
        untrusted_records: Vec::new(),
        network_required_records: Vec::new(),
        unpinned_records: Vec::new(),
        next_actions: Vec::new(),
        records: vec![SourceRecordSummary {
            kind: SourceRecordKind::LocalPath,
            identity: "source/example".to_string(),
            payload_bytes: SOURCE_PAYLOAD_BYTES,
            content_blake3: DIGEST_C.to_string(),
            file_count: 1,
        }],
        non_claim: SOURCE_NON_CLAIM,
    };
    assert_fixture("source-offline-preflight-report.valid.json", &preflight);
}

#[test]
fn nickel_export_reports_serialize_to_registered_positive_fixtures() {
    let receipt = NickelExportReceipt {
        schema: "mantle-nickel-export-receipt-v1".to_string(),
        root_source: ExportSourceRef {
            path: "config/root.ncl".to_string(),
            digest_blake3: DIGEST_A.to_string(),
        },
        deps: Vec::new(),
        import_paths: vec!["lib".to_string()],
        format: "json".to_string(),
        output_target: "stdout".to_string(),
        output_digest_blake3: DIGEST_B.to_string(),
        evaluator: NickelEvaluatorDescriptor {
            identity: "mantle-embedded-crunch-eval".to_string(),
            version: "1".to_string(),
            options: vec!["format=json".to_string(), "import-paths=1".to_string()],
        },
        non_claim: NICKEL_NON_CLAIM.to_string(),
    };
    assert_fixture("nickel-export-receipt.valid.json", &receipt);
    let report = NickelExportReport {
        schema: "mantle-nickel-export-report-v1".to_string(),
        success: true,
        format: "json".to_string(),
        output_target: "stdout".to_string(),
        receipt_digest_blake3: Some(DIGEST_C.to_string()),
        output_digest_blake3: Some(DIGEST_B.to_string()),
        failure_class: None,
        diagnostics: Vec::new(),
        receipt: Some(receipt),
        output_text: Some("{\"answer\":42}".to_string()),
    };
    assert_fixture("nickel-export-report.valid.json", &report);
}
