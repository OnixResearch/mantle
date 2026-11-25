mod store_crash_reopen;

use proptest::prelude::*;
use store_crash_reopen::artifacts::build_artifact;
use store_crash_reopen::artifacts::persist_counterexample_artifact;
use store_crash_reopen::artifacts::shrink_history_preserving_failure;
use store_crash_reopen::language::FaultInjectionMode;
use store_crash_reopen::language::GcExecutionKind;
use store_crash_reopen::language::HISTORY_SCHEMA_VERSION;
use store_crash_reopen::language::HistoryProgramV1;
use store_crash_reopen::language::ReopenKind;
use store_crash_reopen::language::StoreOp;
use store_crash_reopen::language::StoreProfile;
use store_crash_reopen::language::TrustAdmissionMode;
use store_crash_reopen::language::arb_history_program;
use store_crash_reopen::model::ViolationKind;
use store_crash_reopen::runner::NegativeMutant;
use store_crash_reopen::runner::assert_history_passes_or_retain_artifact;
use store_crash_reopen::runner::execute_history;
use store_crash_reopen::runner::replay_artifact_from_json;
use store_crash_reopen::runner::run_child_entry_if_configured;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn store_crash_reopen_child_entry() {
    run_child_entry_if_configured().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_snix_publish_import_root_gc_and_fresh_process_reopen() {
    let program = HistoryProgramV1 {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        profile: StoreProfile::SnixSingle,
        seed: 101,
        steps: vec![
            StoreOp::PublishSignedOutput {
                object_idx: 0,
                is_publish_action_result: true,
                is_register_root: true,
                fault: FaultInjectionMode::None,
            },
            StoreOp::PublishSignedOutput {
                object_idx: 1,
                is_publish_action_result: true,
                is_register_root: false,
                fault: FaultInjectionMode::RecordOnlyInterruptedPublish,
            },
            StoreOp::LookupAndProbeActionResult {
                object_idx: 0,
                is_verify_export: true,
            },
            StoreOp::RunGc {
                kind: GcExecutionKind::DryRunPlanOnly,
            },
            StoreOp::RunGc {
                kind: GcExecutionKind::ExecuteLatestPlan,
            },
            StoreOp::Reopen {
                kind: ReopenKind::FreshChildProcessReopen,
            },
        ],
    };
    assert_history_passes_or_retain_artifact(&program).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_snix_unsigned_import_closure_gc_and_unroot_reclaim() {
    let program = HistoryProgramV1 {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        profile: StoreProfile::SnixSingle,
        seed: 202,
        steps: vec![
            StoreOp::ImportCacheDir {
                object_idx: 0,
                dep_indices: vec![],
                trust_mode: TrustAdmissionMode::Unsigned,
            },
            StoreOp::ImportCacheDir {
                object_idx: 1,
                dep_indices: vec![0],
                trust_mode: TrustAdmissionMode::Unsigned,
            },
            StoreOp::PinRoot {
                object_idx: 1,
                owner_idx: 0,
                reason_idx: 0,
            },
            StoreOp::RunGc {
                kind: GcExecutionKind::ExecuteLatestPlan,
            },
            StoreOp::Reopen {
                kind: ReopenKind::FreshChildProcessReopen,
            },
            StoreOp::UnpinRoot {
                object_idx: 1,
                owner_idx: 0,
                reason_idx: 0,
            },
            StoreOp::RunGc {
                kind: GcExecutionKind::ExecuteLatestPlan,
            },
            StoreOp::Reopen {
                kind: ReopenKind::InProcessReopen,
            },
        ],
    };
    assert_history_passes_or_retain_artifact(&program).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_casita_publish_root_gc_stale_plan_and_fence_recovery() {
    let program = HistoryProgramV1 {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        profile: StoreProfile::CasitaSingle,
        seed: 303,
        steps: vec![
            StoreOp::PublishSignedOutput {
                object_idx: 0,
                is_publish_action_result: true,
                is_register_root: true,
                fault: FaultInjectionMode::None,
            },
            StoreOp::ImportCacheDir {
                object_idx: 1,
                dep_indices: vec![],
                trust_mode: TrustAdmissionMode::Unsigned,
            },
            StoreOp::RunGc {
                kind: GcExecutionKind::ExecuteUnguarded,
            },
            StoreOp::RunGc {
                kind: GcExecutionKind::DryRunPlanOnly,
            },
            StoreOp::RunGc {
                kind: GcExecutionKind::ExecuteStalePlanAfterRootChange,
            },
            StoreOp::RecoverCasitaGc,
            StoreOp::Reopen {
                kind: ReopenKind::FreshChildProcessReopen,
            },
        ],
    };
    assert_history_passes_or_retain_artifact(&program).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_competing_process_mutation_guard_and_casita_root_conflict() {
    let program = HistoryProgramV1 {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        profile: StoreProfile::CasitaSingle,
        seed: 404,
        steps: vec![
            StoreOp::PublishSignedOutput {
                object_idx: 0,
                is_publish_action_result: true,
                is_register_root: true,
                fault: FaultInjectionMode::ConflictingRecordBytes,
            },
            StoreOp::CompetingProcessMutation {
                is_wait_for_lock: false,
            },
            StoreOp::CompetingProcessMutation { is_wait_for_lock: true },
            StoreOp::PublishSignedOutput {
                object_idx: 1,
                is_publish_action_result: true,
                is_register_root: true,
                fault: FaultInjectionMode::HttpIndexPut500,
            },
            StoreOp::Reopen {
                kind: ReopenKind::FreshChildProcessReopen,
            },
        ],
    };
    assert_history_passes_or_retain_artifact(&program).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_abrupt_process_abort_and_fresh_process_reconciliation() {
    for profile in [StoreProfile::SnixSingle, StoreProfile::CasitaSingle] {
        let program = HistoryProgramV1 {
            schema_version: HISTORY_SCHEMA_VERSION.to_string(),
            profile,
            seed: 505,
            steps: vec![
                StoreOp::PublishSignedOutput {
                    object_idx: 0,
                    is_publish_action_result: true,
                    is_register_root: true,
                    fault: FaultInjectionMode::AbruptChildAbort,
                },
                StoreOp::PublishSignedOutput {
                    object_idx: 1,
                    is_publish_action_result: true,
                    is_register_root: true,
                    fault: FaultInjectionMode::DanglingIndexWithoutRecord,
                },
                StoreOp::Reopen {
                    kind: ReopenKind::FreshChildProcessReopen,
                },
            ],
        };
        assert_history_passes_or_retain_artifact(&program).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_semantic_negative_controls_and_healthy_controls() {
    let index_witness = HistoryProgramV1 {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        profile: StoreProfile::SnixSingle,
        seed: 606,
        steps: vec![StoreOp::PublishSignedOutput {
            object_idx: 0,
            is_publish_action_result: true,
            is_register_root: true,
            fault: FaultInjectionMode::RecordOnlyInterruptedPublish,
        }],
    };
    let healthy_index = execute_history(&index_witness, NegativeMutant::None).await;
    assert!(healthy_index.violation.is_none(), "{:?}", healthy_index.violation);
    let mutated_index = execute_history(&index_witness, NegativeMutant::ExposeIncompleteIndex).await;
    assert!(mutated_index.is_mutant_reached);
    assert!(matches!(
        mutated_index.violation,
        Some((ViolationKind::InvalidCacheHitDiscovered | ViolationKind::IncompleteIndexExposed, _))
    ));

    let root_witness = HistoryProgramV1 {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        profile: StoreProfile::SnixSingle,
        seed: 707,
        steps: vec![
            StoreOp::PublishSignedOutput {
                object_idx: 0,
                is_publish_action_result: false,
                is_register_root: true,
                fault: FaultInjectionMode::None,
            },
            StoreOp::RunGc {
                kind: GcExecutionKind::ExecuteLatestPlan,
            },
        ],
    };
    let healthy_root = execute_history(&root_witness, NegativeMutant::None).await;
    assert!(healthy_root.violation.is_none(), "{:?}", healthy_root.violation);
    let mutated_root = execute_history(&root_witness, NegativeMutant::LoseProtectedReference).await;
    assert!(mutated_root.is_mutant_reached);
    assert!(matches!(mutated_root.violation, Some((ViolationKind::ProtectedRootLost, _))));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn smoke_counterexample_artifact_shrink_and_replay() {
    let witness = HistoryProgramV1 {
        schema_version: HISTORY_SCHEMA_VERSION.to_string(),
        profile: StoreProfile::SnixSingle,
        seed: 808,
        steps: vec![
            StoreOp::PublishSignedOutput {
                object_idx: 2,
                is_publish_action_result: false,
                is_register_root: true,
                fault: FaultInjectionMode::None,
            },
            StoreOp::PublishSignedOutput {
                object_idx: 0,
                is_publish_action_result: true,
                is_register_root: true,
                fault: FaultInjectionMode::RecordOnlyInterruptedPublish,
            },
            StoreOp::LookupAndProbeActionResult {
                object_idx: 2,
                is_verify_export: false,
            },
        ],
    };
    let outcome = execute_history(&witness, NegativeMutant::ExposeIncompleteIndex).await;
    let (kind, detail) = outcome.violation.expect("mutant must fail");
    let rt = tokio::runtime::Handle::current();
    let minimized = shrink_history_preserving_failure(&witness, kind, |cand| {
        tokio::task::block_in_place(|| rt.block_on(execute_history(cand, NegativeMutant::ExposeIncompleteIndex)))
            .violation
            .map(|(k, _)| k)
    });
    assert!(minimized.steps.len() < witness.steps.len(), "shrinking should reduce witness");
    let artifact = build_artifact(
        witness.clone(),
        minimized,
        outcome.requested_vs_applied_faults,
        outcome.executed_schedule,
        kind,
        detail,
    );
    let artifact_path = persist_counterexample_artifact(&artifact).expect("persist artifact");
    let json = std::fs::read_to_string(&artifact_path).expect("read artifact");
    let _ = std::fs::remove_file(&artifact_path);
    // Replaying the same witness under healthy semantics passes.
    replay_artifact_from_json(&json).await.expect("healthy replay passes");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn replay_retained_counterexample_if_configured() {
    let Ok(path) = std::env::var("MANTLE_STORE_HISTORY_REPLAY_PATH") else {
        return;
    };
    let json = std::fs::read_to_string(&path).expect("read replay artifact");
    replay_artifact_from_json(&json).await.expect("replay artifact");
}

fn parse_positive_cases(raw: Option<&str>) -> Option<u32> {
    raw.and_then(|value| value.trim().parse::<u32>().ok()).filter(|&count| count > 0)
}

fn resolve_proptest_cases(mantle_cases: Option<&str>, proptest_cases: Option<&str>) -> u32 {
    const DEFAULT_HISTORY_CASES: u32 = 12;
    parse_positive_cases(mantle_cases)
        .or_else(|| parse_positive_cases(proptest_cases))
        .unwrap_or(DEFAULT_HISTORY_CASES)
}

fn configured_proptest_cases() -> u32 {
    let mantle_cases = std::env::var("MANTLE_STORE_HISTORY_CASES").ok();
    let proptest_cases = std::env::var("PROPTEST_CASES").ok();
    resolve_proptest_cases(mantle_cases.as_deref(), proptest_cases.as_deref())
}

#[test]
fn configured_proptest_cases_respects_precedence_and_positive_validation() {
    assert_eq!(resolve_proptest_cases(Some("64"), Some("32")), 64);
    assert_eq!(resolve_proptest_cases(None, Some("64")), 64);
    assert_eq!(resolve_proptest_cases(Some("0"), Some("48")), 48);
    assert_eq!(resolve_proptest_cases(Some("invalid"), Some(" 24 ")), 24);
    assert_eq!(resolve_proptest_cases(Some("0"), Some("0")), 12);
    assert_eq!(resolve_proptest_cases(Some(""), Some("nope")), 12);
    assert_eq!(resolve_proptest_cases(None, None), 12);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: configured_proptest_cases(),
        max_shrink_iters: 16,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn prop_snix_bounded_store_histories(program in arb_history_program(StoreProfile::SnixSingle)) {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(assert_history_passes_or_retain_artifact(&program));
    }

    #[test]
    fn prop_casita_bounded_store_histories(program in arb_history_program(StoreProfile::CasitaSingle)) {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(assert_history_passes_or_retain_artifact(&program));
    }
}
