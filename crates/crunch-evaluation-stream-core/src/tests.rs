use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use proptest::prelude::*;

use crate::BoundedDiagnostic;
use crate::CANCELLED_EXIT_CODE;
use crate::CONTEXT_FIELD_BYTES_MAX;
use crate::DIAGNOSTIC_BYTES_MAX;
use crate::DispatchDecision;
use crate::EVALUATION_NON_SUCCESS_EXIT_CODE;
use crate::FailureFact;
use crate::FailureScope;
use crate::INTERNAL_EXIT_CODE;
use crate::IdentityContext;
use crate::OutcomeError;
use crate::OutcomeLedger;
use crate::PIPELINE_NON_SUCCESS_EXIT_CODE;
use crate::ProcessMode;
use crate::ProcessOutcome;
use crate::REFERENCE_BYTES_MAX;
use crate::ROOT_LABEL_BYTES_MAX;
use crate::ROOT_LABEL_BYTES_MAX_USIZE;
use crate::RootReferences;
use crate::RootSet;
use crate::RunDisposition;
use crate::SUCCESS_EXIT_CODE;
use crate::SourceSequence;
use crate::StreamRecordValue;
use crate::TerminalPhase;
use crate::TerminalState;
use crate::process_status;

const SOURCE_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ROOT_A_ID: &str = "db4c0be35af35d0f235bf3c6160783fa17ae089620fbc4f6a9cf93caf29b28de";
const RUN_ID: &str = "c88eadb275c442621cc18a93d40896a1a94cc341292bcc3fa3c1d4a21036a14d";
const ROOT_A_SEQUENCE: u32 = 0;
const ROOT_B_SEQUENCE: u32 = 1;
const ROOT_C_SEQUENCE: u32 = 2;
const THREE_ROOT_COUNT: u32 = 3;
const TWO_ROOT_COUNT: u32 = 2;
const PROPERTY_ROOT_COUNT_MAX: u32 = 32;
const FAILURE_INTERVAL: u32 = 2;
const ASCII_CHAR_BYTES: u32 = 1;
const BOUND_GENERATOR_END_PADDING: usize = 2;

fn context() -> IdentityContext {
    IdentityContext::new("nickel-2".to_string(), SOURCE_BLAKE3.to_string(), "all-roots".to_string())
        .expect("valid identity context")
}

fn root_set(labels: Vec<String>) -> RootSet {
    RootSet::admit(context(), labels).expect("valid root set")
}

fn three_roots() -> RootSet {
    root_set(vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()])
}

fn sequence(value: u32) -> SourceSequence {
    SourceSequence::new(value).expect("valid source sequence")
}

fn references(value: &str) -> RootReferences {
    RootReferences::new(Some(value.to_string()), None, None).expect("valid result reference")
}

fn started_all(root_set: RootSet) -> OutcomeLedger {
    let mut ledger = OutcomeLedger::new(root_set.clone());
    for root in root_set.roots() {
        ledger = ledger.start(root.sequence()).expect("start admitted root");
    }
    ledger
}

fn finish_in_order(ledger: OutcomeLedger, order: Vec<u32>) -> OutcomeLedger {
    let mut current = ledger;
    for value in order {
        let source_sequence = sequence(value);
        let transition = if value % FAILURE_INTERVAL == 0 {
            current.record_failure(
                source_sequence,
                FailureFact::RootEvaluation,
                BoundedDiagnostic::new(format!("root-{value}-failed")),
            )
        } else {
            current.record_success(source_sequence, references(&format!("result:{value}")))
        }
        .expect("terminal transition");
        current = transition.into_ledger();
    }
    current
}

#[test]
fn admission_assigns_source_order_and_stable_identities() {
    let first = three_roots();
    let second = three_roots();
    let roots = first.roots();

    assert_eq!(first.root_count(), THREE_ROOT_COUNT);
    assert_eq!(roots[ROOT_A_SEQUENCE as usize].sequence().value(), ROOT_A_SEQUENCE);
    assert_eq!(roots[ROOT_B_SEQUENCE as usize].sequence().value(), ROOT_B_SEQUENCE);
    assert_eq!(roots[ROOT_C_SEQUENCE as usize].sequence().value(), ROOT_C_SEQUENCE);
    assert_eq!(roots[ROOT_A_SEQUENCE as usize].root_id(), ROOT_A_ID);
    assert_eq!(first.run_id().expect("first run identity"), RUN_ID);
    assert_eq!(first.run_id().expect("first run identity"), second.run_id().expect("second run identity"));
    assert_ne!(roots[ROOT_A_SEQUENCE as usize].root_id(), roots[ROOT_B_SEQUENCE as usize].root_id());
}

#[test]
fn root_scoped_failure_keeps_dispatch_open_and_preserves_success() {
    let ledger = started_all(three_roots());
    let failed = ledger
        .record_failure(
            sequence(ROOT_B_SEQUENCE),
            FailureFact::RootEvaluation,
            BoundedDiagnostic::new("invalid beta".to_string()),
        )
        .expect("root failure");
    assert_eq!(failed.decision(), DispatchDecision::Continue);

    let after_failure = failed.into_ledger();
    let first = after_failure
        .record_success(sequence(ROOT_A_SEQUENCE), references("result:alpha"))
        .expect("alpha success")
        .into_ledger();
    let complete = first
        .record_success(sequence(ROOT_C_SEQUENCE), references("result:gamma"))
        .expect("gamma success")
        .into_ledger();
    let summary = complete.finish().expect("complete summary");

    assert_eq!(summary.disposition(), RunDisposition::Partial);
    assert_eq!(summary.counts().succeeded, TWO_ROOT_COUNT);
    assert_eq!(summary.counts().failed, 1);
    assert!(summary.roots().iter().all(|root| root.terminal_phase() == TerminalPhase::Evaluation));
    assert_eq!(summary.counts().total().expect("bounded total"), THREE_ROOT_COUNT);
}

#[test]
fn shared_fatal_classifies_started_and_pending_roots() {
    let base = OutcomeLedger::new(three_roots());
    let started_a = base.start(sequence(ROOT_A_SEQUENCE)).expect("start alpha");
    let started_b = started_a.start(sequence(ROOT_B_SEQUENCE)).expect("start beta");
    let transition = started_b
        .record_failure(
            sequence(ROOT_A_SEQUENCE),
            FailureFact::SharedImport,
            BoundedDiagnostic::new("shared import failed".to_string()),
        )
        .expect("shared failure");
    let summary = transition.into_ledger().finish().expect("complete shared-fatal summary");
    let roots = summary.roots();

    assert_eq!(summary.disposition(), RunDisposition::Failed);
    assert_eq!(roots[ROOT_A_SEQUENCE as usize].terminal_state(), TerminalState::Failed);
    assert_eq!(roots[ROOT_B_SEQUENCE as usize].terminal_state(), TerminalState::Failed);
    assert_eq!(roots[ROOT_C_SEQUENCE as usize].terminal_state(), TerminalState::NotStarted);
    assert!(roots.iter().all(|root| root.failure_scope() == Some(FailureScope::SharedFatal)));
    assert!(roots.iter().all(|root| root.terminal_phase() == TerminalPhase::Evaluation));
}

#[test]
fn cancellation_is_terminal_and_rejects_late_success() {
    let base = OutcomeLedger::new(three_roots());
    let started = base.start(sequence(ROOT_A_SEQUENCE)).expect("start alpha");
    let stopped = started
        .stop_remaining(FailureScope::Cancellation, BoundedDiagnostic::new("operator cancelled".to_string()))
        .expect("cancel remaining roots");
    let ledger = stopped.into_ledger();
    let summary = ledger.finish().expect("complete cancellation summary");
    let roots = summary.roots();

    assert_eq!(summary.disposition(), RunDisposition::Cancelled);
    assert_eq!(roots[ROOT_A_SEQUENCE as usize].terminal_state(), TerminalState::Cancelled);
    assert_eq!(roots[ROOT_B_SEQUENCE as usize].terminal_state(), TerminalState::NotStarted);
    assert_eq!(roots[ROOT_C_SEQUENCE as usize].terminal_state(), TerminalState::NotStarted);
    assert!(roots.iter().all(|root| root.terminal_phase() == TerminalPhase::Coordination));
    assert_eq!(summary.counts().total().expect("bounded total"), THREE_ROOT_COUNT);

    let error = ledger
        .record_success(sequence(ROOT_A_SEQUENCE), references("late:alpha"))
        .expect_err("late success must fail");
    assert_eq!(error, OutcomeError::DuplicateTerminal(ROOT_A_SEQUENCE));
}

#[test]
fn coordinator_failure_marks_started_work_as_lost() {
    let base = OutcomeLedger::new(three_roots());
    let started = base.start(sequence(ROOT_A_SEQUENCE)).expect("start alpha");
    let stopped = started
        .stop_remaining(FailureScope::CoordinatorFailure, BoundedDiagnostic::new("coordinator lost".to_string()))
        .expect("coordinator stop");
    let summary = stopped.into_ledger().finish().expect("complete coordinator summary");
    let roots = summary.roots();

    assert_eq!(roots[ROOT_A_SEQUENCE as usize].terminal_state(), TerminalState::WorkerLost);
    assert_eq!(roots[ROOT_B_SEQUENCE as usize].terminal_state(), TerminalState::NotStarted);
    assert_eq!(roots[ROOT_C_SEQUENCE as usize].terminal_state(), TerminalState::NotStarted);
    assert!(roots.iter().all(|root| root.terminal_phase() == TerminalPhase::Coordination));
    assert_eq!(summary.counts().worker_lost, 1);
    assert_eq!(summary.counts().not_started, TWO_ROOT_COUNT);
}

#[test]
fn invalid_admission_and_transition_inputs_fail_closed() {
    let empty = RootSet::admit(context(), Vec::new()).expect_err("empty roots must fail");
    assert_eq!(empty, OutcomeError::EmptyRootSet);

    let duplicate =
        RootSet::admit(context(), vec!["same".to_string(), "same".to_string()]).expect_err("duplicate roots must fail");
    assert_eq!(duplicate, OutcomeError::DuplicateRootLabel("same".to_string()));

    let ledger = OutcomeLedger::new(three_roots());
    let not_started = ledger
        .record_success(sequence(ROOT_A_SEQUENCE), references("result:alpha"))
        .expect_err("terminal before start must fail");
    assert_eq!(not_started, OutcomeError::RootNotStarted(ROOT_A_SEQUENCE));
    assert_eq!(ledger.finish().expect_err("incomplete ledger must fail"), OutcomeError::IncompleteSummary);

    let stop_error = ledger
        .stop_remaining(FailureScope::RootScoped, BoundedDiagnostic::new("not a stop scope".to_string()))
        .expect_err("root-scoped stop must fail");
    assert_eq!(stop_error, OutcomeError::StopScopeRequired);

    let sequence_error = SourceSequence::new(crate::ROOTS_MAX).expect_err("large sequence must fail");
    assert_eq!(sequence_error, OutcomeError::SequenceOutOfRange(crate::ROOTS_MAX));

    let references_error = RootReferences::new(None, None, None).expect_err("missing references must fail");
    assert_eq!(references_error, OutcomeError::MissingSuccessReference);
}

#[test]
fn bounded_values_reject_or_truncate_at_utf8_boundaries() {
    let maximum_label = "x".repeat(ROOT_LABEL_BYTES_MAX as usize);
    let admitted = RootSet::admit(context(), vec![maximum_label.clone()]).expect("maximum label must pass");
    assert_eq!(admitted.roots().into_iter().next().expect("one root").label(), maximum_label);

    let maximum_reference = "r".repeat(REFERENCE_BYTES_MAX as usize);
    let accepted_reference =
        RootReferences::new(Some(maximum_reference.clone()), None, None).expect("maximum reference must pass");
    assert_eq!(accepted_reference.result_ref(), Some(maximum_reference));

    let long_label = "x".repeat((ROOT_LABEL_BYTES_MAX + ASCII_CHAR_BYTES) as usize);
    let label_error = RootSet::admit(context(), vec![long_label]).expect_err("long label must fail");
    assert_eq!(label_error, OutcomeError::FieldTooLong("root label"));

    let long_reference = "r".repeat((REFERENCE_BYTES_MAX + ASCII_CHAR_BYTES) as usize);
    let reference_error = RootReferences::new(Some(long_reference), None, None).expect_err("long reference must fail");
    assert_eq!(reference_error, OutcomeError::FieldTooLong("root reference"));

    let utf8 = "é".repeat(DIAGNOSTIC_BYTES_MAX as usize);
    let diagnostic = BoundedDiagnostic::new(utf8);
    assert!(diagnostic.truncated());
    assert!(diagnostic.text().len() <= DIAGNOSTIC_BYTES_MAX as usize);
    assert!(diagnostic.text().is_char_boundary(diagnostic.text().len()));
}

#[test]
fn bounded_diagnostic_redacts_sensitive_tokens_paths_and_digests() {
    let digest = "a".repeat(SOURCE_BLAKE3.len());
    let diagnostic = BoundedDiagnostic::new(format!(
        "failed at /home/operator/private token=secret-value digest {digest} ordinary-detail"
    ));
    let text = diagnostic.text();

    assert!(text.contains("<redacted-path>"));
    assert!(text.contains("<redacted>"));
    assert!(text.contains("<redacted-digest>"));
    assert!(text.contains("ordinary-detail"));
    assert!(!text.contains("/home/operator/private"));
    assert!(!text.contains("secret-value"));
    assert!(!text.contains(&digest));
}

#[test]
fn invalid_identity_and_context_bounds_fail_closed() {
    let all_roots = IdentityContext::new("nickel-2".to_string(), SOURCE_BLAKE3.to_string(), String::new())
        .expect("empty selector denotes all roots");
    assert_eq!(all_roots.selector(), String::new());

    let uppercase_digest = SOURCE_BLAKE3.to_uppercase();
    let digest_error = IdentityContext::new("nickel-2".to_string(), uppercase_digest, "all-roots".to_string())
        .expect_err("uppercase digest must fail");
    assert_eq!(digest_error, OutcomeError::InvalidSourceIdentity);

    let long_context = "c".repeat((CONTEXT_FIELD_BYTES_MAX + ASCII_CHAR_BYTES) as usize);
    let context_error = IdentityContext::new(long_context, SOURCE_BLAKE3.to_string(), "all-roots".to_string())
        .expect_err("long context must fail");
    assert_eq!(context_error, OutcomeError::FieldTooLong("evaluator cohort"));
}

#[test]
fn failure_fact_classification_is_explicit() {
    let root_facts = [FailureFact::RootEvaluation, FailureFact::RootConversion];
    let shared_facts = [
        FailureFact::SharedSource,
        FailureFact::SharedImport,
        FailureFact::SharedEvaluatorCohort,
        FailureFact::SharedProtocol,
    ];

    assert!(root_facts.into_iter().all(|fact| fact.scope() == FailureScope::RootScoped));
    assert!(shared_facts.into_iter().all(|fact| fact.scope() == FailureScope::SharedFatal));
    assert_eq!(FailureFact::Coordinator.scope(), FailureScope::CoordinatorFailure);
    assert_eq!(FailureFact::OperatorCancellation.scope(), FailureScope::Cancellation);
}

#[test]
fn pure_stream_projection_keeps_json_and_io_outside_core() {
    let roots = root_set(vec!["alpha".to_string()]);
    let started = OutcomeLedger::new(roots.clone()).start(sequence(ROOT_A_SEQUENCE)).expect("start alpha");
    let complete = started
        .record_success(sequence(ROOT_A_SEQUENCE), references("result:alpha"))
        .expect("complete alpha")
        .into_ledger();
    let outcome = complete.outcomes().into_iter().next().expect("one terminal outcome");
    let summary = complete.finish().expect("complete summary");
    let start = StreamRecordValue::run_start(&roots).expect("run-start projection");
    let selected = roots.roots().into_iter().next().expect("one selected root");
    let discovered = StreamRecordValue::root_discovered(selected);
    let terminal = StreamRecordValue::root_terminal(outcome);
    let finished = StreamRecordValue::run_summary(summary);

    assert!(matches!(start, StreamRecordValue::RunStart(_)));
    assert!(matches!(discovered, StreamRecordValue::RootDiscovered(_)));
    assert!(matches!(terminal, StreamRecordValue::RootTerminal(_)));
    assert!(matches!(finished, StreamRecordValue::RunSummary(_)));
}

#[test]
fn process_status_mapping_matches_contract() {
    assert_eq!(
        process_status(ProcessOutcome::Completed(RunDisposition::Success), ProcessMode::Evaluation),
        SUCCESS_EXIT_CODE
    );
    assert_eq!(
        process_status(ProcessOutcome::Completed(RunDisposition::Partial), ProcessMode::Evaluation),
        EVALUATION_NON_SUCCESS_EXIT_CODE
    );
    assert_eq!(
        process_status(ProcessOutcome::Completed(RunDisposition::Failed), ProcessMode::Pipeline),
        PIPELINE_NON_SUCCESS_EXIT_CODE
    );
    assert_eq!(
        process_status(ProcessOutcome::Completed(RunDisposition::Cancelled), ProcessMode::Pipeline),
        CANCELLED_EXIT_CODE
    );
    assert_eq!(process_status(ProcessOutcome::InternalFailure, ProcessMode::Pipeline), INTERNAL_EXIT_CODE);
}

proptest! {
    #[test]
    fn property_schedule_independence_and_canonical_order(
        count in 1u32..PROPERTY_ROOT_COUNT_MAX,
    ) {
        let labels = (0..count).map(|value| format!("root-{value}")).collect::<Vec<_>>();
        let roots = root_set(labels);
        let ledger = started_all(roots);
        let ascending = (0..count).collect::<Vec<_>>();
        let descending = (0..count).rev().collect::<Vec<_>>();

        let first = finish_in_order(ledger.clone(), ascending).finish().expect("ascending summary");
        let second = finish_in_order(ledger, descending).finish().expect("descending summary");
        let observed = first.roots().into_iter().map(|root| root.root().sequence().value()).collect::<Vec<_>>();
        let expected = (0..count).collect::<Vec<_>>();

        prop_assert_eq!(first.counts().total().expect("bounded total"), count);
        prop_assert_eq!(observed, expected);
        prop_assert_eq!(first, second);
    }

    #[test]
    fn property_complete_accounting(
        count in 1u32..PROPERTY_ROOT_COUNT_MAX,
    ) {
        let labels = (0..count).map(|value| format!("root-{value}")).collect::<Vec<_>>();
        let roots = root_set(labels);
        let mut ledger = started_all(roots);
        for value in 0..count {
            ledger = ledger
                .record_success(sequence(value), references(&format!("result:{value}")))
                .expect("terminal success")
                .into_ledger();
        }
        let summary = ledger.finish().expect("complete accounting summary");
        let expected_count = usize::try_from(count).expect("property count fits usize");

        prop_assert_eq!(summary.counts().total().expect("bounded total"), count);
        prop_assert_eq!(summary.roots().len(), expected_count);
        prop_assert_eq!(summary.disposition(), RunDisposition::Success);
    }

    #[test]
    fn property_checked_label_bounds(
        label_bytes in 0usize..(ROOT_LABEL_BYTES_MAX_USIZE + BOUND_GENERATOR_END_PADDING),
    ) {
        let label = "x".repeat(label_bytes);
        let result = RootSet::admit(context(), vec![label]);
        if label_bytes == 0 {
            prop_assert_eq!(result.expect_err("empty label must fail"), OutcomeError::EmptyField("root label"));
        } else if label_bytes <= ROOT_LABEL_BYTES_MAX_USIZE {
            prop_assert!(result.is_ok());
        } else {
            prop_assert_eq!(result.expect_err("large label must fail"), OutcomeError::FieldTooLong("root label"));
        }
    }

    #[test]
    fn property_terminal_exclusivity(
        count in 1u32..PROPERTY_ROOT_COUNT_MAX,
        selected in 0u32..PROPERTY_ROOT_COUNT_MAX,
    ) {
        let labels = (0..count).map(|value| format!("root-{value}")).collect::<Vec<_>>();
        let roots = root_set(labels);
        let ledger = started_all(roots);
        let selected = selected % count;
        let first = ledger
            .record_success(sequence(selected), references("result:first"))
            .expect("first terminal")
            .into_ledger();
        let duplicate = first.record_failure(
            sequence(selected),
            FailureFact::RootEvaluation,
            BoundedDiagnostic::new("duplicate".to_string()),
        );

        prop_assert_eq!(duplicate.expect_err("duplicate must fail"), OutcomeError::DuplicateTerminal(selected));
        prop_assert_eq!(first.outcomes().len(), 1);
    }

    #[test]
    fn property_equivalent_fact_replay(
        count in 1u32..PROPERTY_ROOT_COUNT_MAX,
    ) {
        let labels = (0..count).map(|value| format!("root-{value}")).collect::<Vec<_>>();
        let roots = root_set(labels);
        let ledger = started_all(roots);
        let order = (0..count).rev().collect::<Vec<_>>();

        let first = finish_in_order(ledger.clone(), order.clone()).finish().expect("first replay");
        let second = finish_in_order(ledger, order).finish().expect("second replay");

        prop_assert_eq!(first.run_id(), second.run_id());
        prop_assert_eq!(first, second);
    }
}
