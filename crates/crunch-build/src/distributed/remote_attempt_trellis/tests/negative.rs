use super::super::super::*;
use super::fixtures::MatrixReportVariant;
use super::fixtures::TEST_CONFLICT_DIGEST;
use super::fixtures::matrix_case;
use crate::distributed::RemoteAttemptPhase;
use crate::distributed::RemoteAttemptReasonCode;
use crate::distributed::RemoteFenceGeneration;
use crate::distributed::RemotePayloadDigest;
use crate::distributed::plan_remote_attempt_report;

#[test]
// r[verify remote_builds.trellis_admission_projection]
fn malformed_report_and_state_facts_fail_before_proof_coverage() {
    let (state, mut report, authorization) = matrix_case(
        RemoteAttemptPhase::Running,
        MatrixReportVariant::Heartbeat,
        TrellisIdentityClass::Current,
        TrellisEventClass::New,
        TrellisHistoryClass::Available,
        TrellisScopeClass::Granted,
        TrellisScopeClass::Granted,
    );
    report.identity.payload_digest = RemotePayloadDigest::new(TEST_CONFLICT_DIGEST).unwrap();
    assert_eq!(
        project_remote_attempt_to_trellis(&state, &report, authorization),
        Err(TrellisProjectionError::ReportNotAdmitted)
    );

    let mut malformed_state = state;
    malformed_state.progress_events_applied = u32::MAX;
    assert_eq!(
        project_remote_attempt_to_trellis(&malformed_state, &report, authorization),
        Err(TrellisProjectionError::StateNotAdmitted)
    );
}

#[test]
// r[verify remote_builds.trellis_admission_safety]
fn representable_fence_advance_and_exhaustion_match_the_proved_boundary() {
    let current = RemoteFenceGeneration::new(1).unwrap();
    let next = current.advance().unwrap();
    let exhausted = RemoteFenceGeneration::new(u64::MAX).unwrap().advance();

    assert!(next > current);
    assert_ne!(next.get(), 0);
    assert_eq!(exhausted, Err(RemoteAttemptReasonCode::FenceExhausted));
}

#[test]
// r[verify remote_builds.trellis_admission_claim_boundary]
fn claim_guard_accepts_only_the_named_abstract_claim() {
    assert!(trellis_claim_text_is_bounded(TRELLIS_REMOTE_ADMISSION_CLAIM));
    for overclaim in TRELLIS_REMOTE_ADMISSION_NON_CLAIMS {
        assert!(!trellis_claim_text_is_bounded(overclaim), "accepted overclaim: {overclaim}");
    }
}

#[test]
// r[verify remote_builds.trellis_admission_projection]
fn known_semantic_differences_reject_with_stable_classes() {
    let cases = [
        (
            RemoteAttemptPhase::Transferring,
            MatrixReportVariant::Heartbeat,
            TrellisScopeClass::Granted,
            TrellisScopeClass::Granted,
            TrellisProjectionError::PhaseReportMismatch,
        ),
        (
            RemoteAttemptPhase::Queued,
            MatrixReportVariant::Failure,
            TrellisScopeClass::Granted,
            TrellisScopeClass::Granted,
            TrellisProjectionError::PhaseReportMismatch,
        ),
        (
            RemoteAttemptPhase::Running,
            MatrixReportVariant::Failure,
            TrellisScopeClass::Denied,
            TrellisScopeClass::Granted,
            TrellisProjectionError::AuthorityMeaningMismatch,
        ),
        (
            RemoteAttemptPhase::Running,
            MatrixReportVariant::ResultReady,
            TrellisScopeClass::Granted,
            TrellisScopeClass::Denied,
            TrellisProjectionError::AuthorityMeaningMismatch,
        ),
        (
            RemoteAttemptPhase::FinishedUndelivered,
            MatrixReportVariant::Failure,
            TrellisScopeClass::Granted,
            TrellisScopeClass::Granted,
            TrellisProjectionError::ResultRetentionMismatch,
        ),
    ];
    for (phase, variant, worker, output, expected) in cases {
        let (state, report, authorization) = matrix_case(
            phase,
            variant,
            TrellisIdentityClass::Current,
            TrellisEventClass::New,
            TrellisHistoryClass::Available,
            worker,
            output,
        );
        assert_eq!(project_remote_attempt_to_trellis(&state, &report, authorization), Err(expected));
    }
}

#[test]
// r[verify remote_builds.trellis_admission_projection]
fn projection_anchor_mutation_is_rejected() {
    let (state, report, authorization) = matrix_case(
        RemoteAttemptPhase::Running,
        MatrixReportVariant::Heartbeat,
        TrellisIdentityClass::Current,
        TrellisEventClass::New,
        TrellisHistoryClass::Available,
        TrellisScopeClass::Granted,
        TrellisScopeClass::Granted,
    );
    let mut projection = project_remote_attempt_to_trellis(&state, &report, authorization).unwrap();
    projection.progress_before = projection.progress_before.checked_add(1).unwrap();
    let plan = plan_remote_attempt_report(&state, &report, authorization);

    assert_eq!(
        normalize_mantle_trellis_outcome(&state, projection, &plan),
        Err(TrellisProjectionError::OutcomeMutation)
    );
    assert_ne!(projection.progress_before, state.progress_events_applied);
}

#[test]
// r[verify remote_builds.trellis_admission_evidence_boundary]
fn runtime_dependency_and_authority_surfaces_remain_separate() {
    let workspace_manifest = include_str!("../../../../../../Cargo.toml");
    let crate_manifest = include_str!("../../../../Cargo.toml");
    let runtime_adapter = include_str!("../../../../../../src/remote_build.rs");

    assert!(!workspace_manifest.contains("verified-logic"));
    assert!(!crate_manifest.contains("verified-logic"));
    assert!(runtime_adapter.contains("plan_remote_attempt_report"));
    assert!(!runtime_adapter.contains("project_remote_attempt_to_trellis"));
    assert!(!runtime_adapter.contains("trellis-proof-envelope"));
}
