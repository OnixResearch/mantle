use std::collections::BTreeMap;

use serde::Deserialize;

use super::super::super::*;
use super::fixtures::for_each_matrix_case;
use super::fixtures::matrix_case;
use super::fixtures::unchecked_projection;
use crate::distributed::RemoteAttemptApplyDisposition;
use crate::distributed::RemoteAttemptApplyPlan;
use crate::distributed::RemoteAttemptReportPayload;
use crate::distributed::plan_remote_attempt_report;

const ORACLE_BYTES: &[u8] = include_bytes!("../../../../../../fixtures/trellis-remote-admission/oracle.v1.bin");
const ORACLE_MANIFEST: &str = include_str!("../../../../../../fixtures/trellis-remote-admission/oracle.v1.json");
const NO_REJECTION_CODE: u8 = u8::MAX;
const DISPOSITION_INDEX: usize = 0;
const REJECTION_INDEX: usize = 1;
const PHASE_INDEX: usize = 2;
const PROGRESS_DELTA_INDEX: usize = 3;
const EVENT_DELTA_INDEX: usize = 4;
const RESULT_EFFECT_INDEX: usize = 5;
const STATE_PRESERVED_INDEX: usize = 6;
const DISPOSITION_APPLIED: u8 = 0;
const DISPOSITION_REPLAYED: u8 = 1;
const DISPOSITION_REJECTED: u8 = 2;
const REJECT_STALE_EPOCH: u8 = 0;
const REJECT_FUTURE_EPOCH: u8 = 1;
const REJECT_WRONG_JOB: u8 = 2;
const REJECT_WRONG_RUN: u8 = 3;
const REJECT_EVENT_CONFLICT: u8 = 4;
const REJECT_TERMINAL_PHASE: u8 = 5;
const REJECT_INVALID_TRANSITION: u8 = 6;
const REJECT_WORKER_DENIED: u8 = 7;
const REJECT_OUTPUT_DENIED: u8 = 8;
const REJECT_MISSING_RESULT: u8 = 9;
const REJECT_RESULT_MISMATCH: u8 = 10;
const REJECT_EPOCH_EXHAUSTED: u8 = 11;
const REJECT_HISTORY_FULL: u8 = 12;
const PHASE_QUEUED: u8 = 0;
const PHASE_RUNNING: u8 = 1;
const PHASE_TRANSFERRING: u8 = 2;
const PHASE_RESULT_READY: u8 = 3;
const PHASE_COMPLETED: u8 = 4;
const PHASE_FAILED: u8 = 5;
const PHASE_SUPERSEDED: u8 = 6;
const RESULT_PRESERVED: u8 = 0;
const RESULT_SET_FROM_REPORT: u8 = 1;
const RESULT_CLEARED: u8 = 2;

#[derive(Debug, Deserialize)]
struct OracleManifest {
    schema: String,
    trellis_revision: String,
    trellis_source_archive_blake3: String,
    record_bytes: usize,
    case_count: usize,
    matrix_blake3: String,
    projection_counts: BTreeMap<String, u32>,
}

#[test]
// r[verify remote_builds.trellis_admission_projection]
// r[verify remote_builds.trellis_admission_safety]
fn complete_finite_matrix_matches_the_pinned_trellis_oracle_or_rejects() {
    let manifest: OracleManifest = serde_json::from_str(ORACLE_MANIFEST).unwrap();
    validate_manifest(&manifest);
    let mut offset = 0_usize;
    let mut projection_counts = BTreeMap::<String, u32>::new();
    for_each_matrix_case(|phase, variant, identity, event, history, worker, output| {
        let (state, report, authorization) = matrix_case(phase, variant, identity, event, history, worker, output);
        let expected = decode_oracle_record(&ORACLE_BYTES[offset..offset + manifest.record_bytes]);
        offset += manifest.record_bytes;
        let plan = plan_remote_attempt_report(&state, &report, authorization);
        match project_remote_attempt_to_trellis(&state, &report, authorization) {
            Ok(projection) => {
                increment(&mut projection_counts, "supported");
                let actual = normalize_mantle_trellis_outcome(&state, projection, &plan).unwrap();
                assert_eq!(actual, expected, "supported case drifted: {:?}", projection.case);
                assert_output_authority(&report.payload, &plan);
            }
            Err(error) => {
                increment(&mut projection_counts, error.as_str());
                let projection = unchecked_projection(&state, &report, authorization);
                let actual = normalize_mantle_trellis_outcome(&state, projection, &plan).unwrap();
                assert_ne!(
                    actual, expected,
                    "projection rejected a case that already agrees: {:?} error={error:?}",
                    projection.case
                );
                assert!(!plan.output_admission_allowed || plan.disposition == RemoteAttemptApplyDisposition::Applied);
            }
        }
    });
    assert_eq!(offset, ORACLE_BYTES.len());
    assert_eq!(u32::try_from(offset / manifest.record_bytes).unwrap(), TRELLIS_REMOTE_ADMISSION_MATRIX_CASES);
    assert_eq!(projection_counts, manifest.projection_counts);
}

fn validate_manifest(manifest: &OracleManifest) {
    assert_eq!(manifest.schema, TRELLIS_REMOTE_ADMISSION_ORACLE_SCHEMA);
    assert_eq!(manifest.trellis_revision, TRELLIS_FENCED_ATTEMPT_REVISION);
    assert_eq!(manifest.trellis_source_archive_blake3, TRELLIS_FENCED_ATTEMPT_SOURCE_ARCHIVE_BLAKE3);
    assert_eq!(manifest.record_bytes, TRELLIS_REMOTE_ADMISSION_ORACLE_RECORD_BYTES);
    assert_eq!(u32::try_from(manifest.case_count).unwrap(), TRELLIS_REMOTE_ADMISSION_MATRIX_CASES);
    assert_eq!(ORACLE_BYTES.len(), manifest.case_count * manifest.record_bytes);
    assert_eq!(blake3::hash(ORACLE_BYTES).to_hex().as_str(), manifest.matrix_blake3);
}

fn decode_oracle_record(bytes: &[u8]) -> TrellisNormalizedOutcome {
    assert_eq!(bytes.len(), TRELLIS_REMOTE_ADMISSION_ORACLE_RECORD_BYTES);
    TrellisNormalizedOutcome {
        disposition: decode_disposition(bytes[DISPOSITION_INDEX]),
        reject_class: decode_reject_class(bytes[REJECTION_INDEX]),
        next_phase: decode_phase(bytes[PHASE_INDEX]),
        progress_delta: bytes[PROGRESS_DELTA_INDEX],
        event_delta: bytes[EVENT_DELTA_INDEX],
        result_effect: decode_result_effect(bytes[RESULT_EFFECT_INDEX]),
        state_preserved: decode_bool(bytes[STATE_PRESERVED_INDEX]),
    }
}

fn decode_disposition(value: u8) -> TrellisOutcomeDisposition {
    match value {
        DISPOSITION_APPLIED => TrellisOutcomeDisposition::Applied,
        DISPOSITION_REPLAYED => TrellisOutcomeDisposition::Replayed,
        DISPOSITION_REJECTED => TrellisOutcomeDisposition::Rejected,
        _ => panic!("unknown oracle disposition {value}"),
    }
}

fn decode_reject_class(value: u8) -> Option<TrellisRejectClass> {
    if value == NO_REJECTION_CODE {
        return None;
    }
    Some(match value {
        REJECT_STALE_EPOCH => TrellisRejectClass::StaleEpoch,
        REJECT_FUTURE_EPOCH => TrellisRejectClass::FutureEpoch,
        REJECT_WRONG_JOB => TrellisRejectClass::WrongJob,
        REJECT_WRONG_RUN => TrellisRejectClass::WrongRun,
        REJECT_EVENT_CONFLICT => TrellisRejectClass::EventConflict,
        REJECT_TERMINAL_PHASE => TrellisRejectClass::TerminalPhase,
        REJECT_INVALID_TRANSITION => TrellisRejectClass::InvalidTransition,
        REJECT_WORKER_DENIED => TrellisRejectClass::WorkerDenied,
        REJECT_OUTPUT_DENIED => TrellisRejectClass::OutputDenied,
        REJECT_MISSING_RESULT => TrellisRejectClass::MissingResult,
        REJECT_RESULT_MISMATCH => TrellisRejectClass::ResultMismatch,
        REJECT_EPOCH_EXHAUSTED => TrellisRejectClass::EpochExhausted,
        REJECT_HISTORY_FULL => TrellisRejectClass::HistoryFull,
        _ => panic!("unknown oracle rejection {value}"),
    })
}

fn decode_phase(value: u8) -> TrellisProjectionPhase {
    match value {
        PHASE_QUEUED => TrellisProjectionPhase::Queued,
        PHASE_RUNNING => TrellisProjectionPhase::Running,
        PHASE_TRANSFERRING => TrellisProjectionPhase::Transferring,
        PHASE_RESULT_READY => TrellisProjectionPhase::ResultReady,
        PHASE_COMPLETED => TrellisProjectionPhase::Completed,
        PHASE_FAILED => TrellisProjectionPhase::Failed,
        PHASE_SUPERSEDED => TrellisProjectionPhase::Superseded,
        _ => panic!("unknown oracle phase {value}"),
    }
}

fn decode_result_effect(value: u8) -> TrellisResultEffect {
    match value {
        RESULT_PRESERVED => TrellisResultEffect::Preserved,
        RESULT_SET_FROM_REPORT => TrellisResultEffect::SetFromReport,
        RESULT_CLEARED => TrellisResultEffect::Cleared,
        _ => panic!("unknown oracle result effect {value}"),
    }
}

fn decode_bool(value: u8) -> bool {
    match value {
        0 => false,
        1 => true,
        _ => panic!("unknown oracle Boolean {value}"),
    }
}

fn assert_output_authority(payload: &RemoteAttemptReportPayload, plan: &RemoteAttemptApplyPlan) {
    let requires_output = matches!(
        payload,
        RemoteAttemptReportPayload::ResultReady { .. } | RemoteAttemptReportPayload::Completion { .. }
    );
    let expected = plan.disposition == RemoteAttemptApplyDisposition::Applied && requires_output;
    assert_eq!(plan.output_admission_allowed, expected);
    assert!(!plan.output_admission_allowed || plan.disposition == RemoteAttemptApplyDisposition::Applied);
}

fn increment(counts: &mut BTreeMap<String, u32>, key: &str) {
    let value = counts.entry(key.to_string()).or_default();
    *value = value.checked_add(1).unwrap();
}
