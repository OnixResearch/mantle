// machine-artifact-public: build.evaluation-stream
// r[impl evaluation_streaming.versioned_event_stream]
// r[impl evaluation_streaming.partial_run_disposition]
// r[impl evaluation_streaming.cancellation_and_output]
// r[impl evaluation_streaming.core_shell_boundary]
use std::io::Write;

use crunch_evaluation_stream_core::FailureScope;
use crunch_evaluation_stream_core::RootCounts;
use crunch_evaluation_stream_core::RootOutcome;
use crunch_evaluation_stream_core::RunDisposition;
use crunch_evaluation_stream_core::StreamRecordValue;
use crunch_evaluation_stream_core::TerminalPhase;
use crunch_evaluation_stream_core::TerminalState;
use serde::Serialize;

pub const STREAM_EVENT_CHANNEL_CAPACITY: usize = 64;
const MAX_STREAM_RECORD_BYTES: usize = 134_217_728;
const RECORD_NEWLINE_BYTES: usize = 1;
const COMPATIBILITY_MODE: &str = "stream-v1-introduction-plus-one-subsequent-minor-release";

#[derive(Debug)]
pub enum EvaluationStreamOutputError {
    MissingRunStart,
    DuplicateRunStart,
    RecordAfterSummary,
    RunIdentityMismatch,
    MissingSummary,
    RecordTooLarge,
    Serialize(serde_json::Error),
    Write(std::io::Error),
    Flush(std::io::Error),
}

impl std::fmt::Display for EvaluationStreamOutputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRunStart => formatter.write_str("evaluation stream must start with run-start"),
            Self::DuplicateRunStart => formatter.write_str("evaluation stream contains a second run-start"),
            Self::RecordAfterSummary => formatter.write_str("evaluation stream contains a record after run-summary"),
            Self::RunIdentityMismatch => {
                formatter.write_str("evaluation stream run identity changed within one stream")
            }
            Self::MissingSummary => formatter.write_str("evaluation stream ended before run-summary"),
            Self::RecordTooLarge => {
                write!(formatter, "evaluation stream record exceeds {MAX_STREAM_RECORD_BYTES} bytes")
            }
            Self::Serialize(error) => write!(formatter, "serializing evaluation stream record: {error}"),
            Self::Write(error) => write!(formatter, "writing evaluation stream record: {error}"),
            Self::Flush(error) => write!(formatter, "flushing evaluation stream record: {error}"),
        }
    }
}

impl std::error::Error for EvaluationStreamOutputError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WriterState {
    StartExpected,
    Active,
    Complete,
}

pub struct EvaluationStreamWriter<W> {
    output: W,
    state: WriterState,
    run_id: Option<String>,
    disposition: Option<RunDisposition>,
}

impl<W: Write> EvaluationStreamWriter<W> {
    pub fn new(output: W) -> Self {
        Self {
            output,
            state: WriterState::StartExpected,
            run_id: None,
            disposition: None,
        }
    }

    pub fn write_record(&mut self, record: &StreamRecordValue) -> Result<(), EvaluationStreamOutputError> {
        let next_state = next_writer_state(self.state, record)?;
        let projected = project_record(record, self.run_id.as_deref())?;
        let bytes = encode_record(&projected)?;
        self.output.write_all(&bytes).map_err(EvaluationStreamOutputError::Write)?;
        self.output.write_all(b"\n").map_err(EvaluationStreamOutputError::Write)?;
        self.output.flush().map_err(EvaluationStreamOutputError::Flush)?;

        if let StreamRecordValue::RunStart(start) = record {
            self.run_id = Some(start.run_id());
        }
        if let StreamRecordValue::RunSummary(summary) = record {
            self.disposition = Some(summary.disposition());
        }
        self.state = next_state;
        debug_assert!(self.run_id.is_some());
        debug_assert!(self.state != WriterState::Complete || self.disposition.is_some());
        Ok(())
    }

    pub fn finish(&self) -> Result<RunDisposition, EvaluationStreamOutputError> {
        if self.state != WriterState::Complete {
            return Err(EvaluationStreamOutputError::MissingSummary);
        }
        self.disposition.ok_or(EvaluationStreamOutputError::MissingSummary)
    }

    #[cfg(test)]
    fn into_inner(self) -> W {
        self.output
    }
}

fn next_writer_state(
    current: WriterState,
    record: &StreamRecordValue,
) -> Result<WriterState, EvaluationStreamOutputError> {
    match (current, record) {
        (WriterState::StartExpected, StreamRecordValue::RunStart(_)) => Ok(WriterState::Active),
        (WriterState::StartExpected, _) => Err(EvaluationStreamOutputError::MissingRunStart),
        (WriterState::Active, StreamRecordValue::RunStart(_)) => Err(EvaluationStreamOutputError::DuplicateRunStart),
        (WriterState::Active, StreamRecordValue::RunSummary(_)) => Ok(WriterState::Complete),
        (WriterState::Active, StreamRecordValue::RootDiscovered(_) | StreamRecordValue::RootTerminal(_)) => {
            Ok(WriterState::Active)
        }
        (WriterState::Complete, _) => Err(EvaluationStreamOutputError::RecordAfterSummary),
    }
}

fn encode_record(record: &WireRecord) -> Result<Vec<u8>, EvaluationStreamOutputError> {
    let bytes = serde_json::to_vec(record).map_err(EvaluationStreamOutputError::Serialize)?;
    let framed_bytes =
        bytes.len().checked_add(RECORD_NEWLINE_BYTES).ok_or(EvaluationStreamOutputError::RecordTooLarge)?;
    if framed_bytes > MAX_STREAM_RECORD_BYTES {
        return Err(EvaluationStreamOutputError::RecordTooLarge);
    }
    debug_assert!(!bytes.is_empty());
    debug_assert!(framed_bytes <= MAX_STREAM_RECORD_BYTES);
    Ok(bytes)
}

fn project_record(
    record: &StreamRecordValue,
    active_run_id: Option<&str>,
) -> Result<WireRecord, EvaluationStreamOutputError> {
    match record {
        StreamRecordValue::RunStart(start) => Ok(WireRecord::RunStart(RunStartRecord {
            schema: start.schema(),
            kind: "run-start",
            run_id: start.run_id(),
            source_blake3: start.source_blake3(),
            evaluator_cohort: start.evaluator_cohort(),
            selector: start.selector(),
            selected_root_count: start.selected_root_count(),
            compatibility_mode: COMPATIBILITY_MODE,
        })),
        StreamRecordValue::RootDiscovered(root) => {
            let run_id = required_run_id(active_run_id)?;
            Ok(WireRecord::RootDiscovered(RootDiscoveredRecord {
                schema: crunch_evaluation_stream_core::STREAM_SCHEMA,
                kind: "root-discovered",
                run_id,
                sequence: root.sequence().value(),
                root_id: root.root_id(),
                root_label: root.label(),
            }))
        }
        StreamRecordValue::RootTerminal(outcome) => {
            let run_id = required_run_id(active_run_id)?;
            Ok(WireRecord::RootTerminal(RootTerminalRecord::from_outcome(run_id, outcome)))
        }
        StreamRecordValue::RunSummary(summary) => {
            let run_id = required_run_id(active_run_id)?;
            if run_id != summary.run_id() {
                return Err(EvaluationStreamOutputError::RunIdentityMismatch);
            }
            let roots = summary.roots().iter().map(RootProjection::from_outcome).collect::<Vec<_>>();
            let selected_root_count =
                u32::try_from(roots.len()).map_err(|_| EvaluationStreamOutputError::RecordTooLarge)?;
            Ok(WireRecord::RunSummary(RunSummaryRecord {
                schema: summary.schema(),
                kind: "run-summary",
                run_id,
                disposition: disposition_name(summary.disposition()),
                selected_root_count,
                terminal_root_count: selected_root_count,
                counts: CountsRecord::from(summary.counts()),
                roots,
            }))
        }
    }
}

fn required_run_id(active_run_id: Option<&str>) -> Result<String, EvaluationStreamOutputError> {
    active_run_id.map(str::to_string).ok_or(EvaluationStreamOutputError::MissingRunStart)
}

#[derive(Serialize)]
#[serde(untagged)]
enum WireRecord {
    RunStart(RunStartRecord),
    RootDiscovered(RootDiscoveredRecord),
    RootTerminal(RootTerminalRecord),
    RunSummary(RunSummaryRecord),
}

#[derive(Serialize)]
struct RunStartRecord {
    schema: String,
    kind: &'static str,
    run_id: String,
    source_blake3: String,
    evaluator_cohort: String,
    selector: String,
    selected_root_count: u32,
    compatibility_mode: &'static str,
}

#[derive(Serialize)]
struct RootDiscoveredRecord {
    schema: &'static str,
    kind: &'static str,
    run_id: String,
    sequence: u32,
    root_id: String,
    root_label: String,
}

#[derive(Serialize)]
struct RootTerminalRecord {
    schema: &'static str,
    kind: &'static str,
    run_id: String,
    #[serde(flatten)]
    root: RootProjection,
}

impl RootTerminalRecord {
    fn from_outcome(run_id: String, outcome: &RootOutcome) -> Self {
        Self {
            schema: crunch_evaluation_stream_core::STREAM_SCHEMA,
            kind: "root-terminal",
            run_id,
            root: RootProjection::from_outcome(outcome),
        }
    }
}

#[derive(Serialize)]
struct RunSummaryRecord {
    schema: String,
    kind: &'static str,
    run_id: String,
    disposition: &'static str,
    selected_root_count: u32,
    terminal_root_count: u32,
    counts: CountsRecord,
    roots: Vec<RootProjection>,
}

#[derive(Serialize)]
struct CountsRecord {
    succeeded: u32,
    failed: u32,
    worker_lost: u32,
    cancelled: u32,
    not_started: u32,
}

impl From<RootCounts> for CountsRecord {
    fn from(counts: RootCounts) -> Self {
        Self {
            succeeded: counts.succeeded,
            failed: counts.failed,
            worker_lost: counts.worker_lost,
            cancelled: counts.cancelled,
            not_started: counts.not_started,
        }
    }
}

#[derive(Serialize)]
struct RootProjection {
    sequence: u32,
    root_id: String,
    root_label: String,
    terminal_state: &'static str,
    terminal_phase: &'static str,
    failure_scope: Option<&'static str>,
    diagnostic: Option<String>,
    diagnostic_truncated: bool,
    result_ref: Option<String>,
    cache_ref: Option<String>,
    build_ref: Option<String>,
}

impl RootProjection {
    fn from_outcome(outcome: &RootOutcome) -> Self {
        let root = outcome.root();
        let diagnostic = outcome.diagnostic();
        let references = outcome.references();
        Self {
            sequence: root.sequence().value(),
            root_id: root.root_id(),
            root_label: root.label(),
            terminal_state: terminal_state_name(outcome.terminal_state()),
            terminal_phase: terminal_phase_name(outcome.terminal_phase()),
            failure_scope: outcome.failure_scope().map(failure_scope_name),
            diagnostic: diagnostic.as_ref().map(|value| value.text()),
            diagnostic_truncated: diagnostic.as_ref().is_some_and(|value| value.truncated()),
            result_ref: references.as_ref().and_then(|value| value.result_ref()),
            cache_ref: references.as_ref().and_then(|value| value.cache_ref()),
            build_ref: references.as_ref().and_then(|value| value.build_ref()),
        }
    }
}

fn terminal_state_name(state: TerminalState) -> &'static str {
    match state {
        TerminalState::Succeeded => "succeeded",
        TerminalState::Failed => "failed",
        TerminalState::WorkerLost => "worker-lost",
        TerminalState::Cancelled => "cancelled",
        TerminalState::NotStarted => "not-started",
    }
}

fn terminal_phase_name(phase: TerminalPhase) -> &'static str {
    match phase {
        TerminalPhase::Evaluation => "evaluation",
        TerminalPhase::Conversion => "conversion",
        TerminalPhase::Build => "build",
        TerminalPhase::Coordination => "coordination",
    }
}

fn failure_scope_name(scope: FailureScope) -> &'static str {
    match scope {
        FailureScope::RootScoped => "root-scoped",
        FailureScope::SharedFatal => "shared-fatal",
        FailureScope::Cancellation => "cancellation",
        FailureScope::CoordinatorFailure => "coordinator-failure",
    }
}

fn disposition_name(disposition: RunDisposition) -> &'static str {
    match disposition {
        RunDisposition::Success => "success",
        RunDisposition::Partial => "partial",
        RunDisposition::Failed => "failed",
        RunDisposition::Cancelled => "cancelled",
    }
}

#[cfg(test)]
mod tests {
    use crunch_evaluation_stream_core::BoundedDiagnostic;
    use crunch_evaluation_stream_core::FailureFact;
    use crunch_evaluation_stream_core::IdentityContext;
    use crunch_evaluation_stream_core::OutcomeLedger;
    use crunch_evaluation_stream_core::RootReferences;
    use crunch_evaluation_stream_core::RootSet;
    use crunch_evaluation_stream_core::StreamRecordValue;
    use crunch_evaluation_stream_core::TerminalPhase;

    use super::*;

    const SOURCE_BLAKE3: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn root_set() -> RootSet {
        RootSet::admit(
            IdentityContext::new("nickel@2".to_string(), SOURCE_BLAKE3.to_string(), "all-roots".to_string()).unwrap(),
            vec!["alpha".to_string(), "beta".to_string()],
        )
        .unwrap()
    }

    fn complete_records() -> Vec<StreamRecordValue> {
        let root_set = root_set();
        let roots = root_set.roots();
        let mut ledger = OutcomeLedger::new(root_set.clone());
        ledger = ledger.start(roots[0].sequence()).unwrap();
        ledger = ledger.start(roots[1].sequence()).unwrap();
        ledger = ledger
            .record_success_at_phase(
                roots[1].sequence(),
                TerminalPhase::Conversion,
                RootReferences::new(Some("/mantle/store/beta.drv".to_string()), None, None).unwrap(),
            )
            .unwrap()
            .into_ledger();
        ledger = ledger
            .record_failure(
                roots[0].sequence(),
                FailureFact::RootEvaluation,
                BoundedDiagnostic::new("bad alpha".to_string()),
            )
            .unwrap()
            .into_ledger();
        let summary = ledger.finish().unwrap();
        vec![
            StreamRecordValue::run_start(&root_set).unwrap(),
            StreamRecordValue::root_discovered(roots[0].clone()),
            StreamRecordValue::root_discovered(roots[1].clone()),
            StreamRecordValue::root_terminal(summary.roots()[1].clone()),
            StreamRecordValue::root_terminal(summary.roots()[0].clone()),
            StreamRecordValue::run_summary(summary),
        ]
    }

    // r[verify evaluation_streaming.versioned_event_stream]
    // r[verify evaluation_streaming.deterministic_identity_and_order]
    #[test]
    fn writer_emits_complete_ndjson_and_canonical_summary() {
        let mut writer = EvaluationStreamWriter::new(Vec::new());
        for record in complete_records() {
            writer.write_record(&record).unwrap();
        }
        assert_eq!(writer.finish().unwrap(), RunDisposition::Partial);
        let output = String::from_utf8(writer.into_inner()).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        let values = lines
            .iter()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(lines.len(), complete_records().len());
        assert_eq!(values[0]["kind"], "run-start");
        assert_eq!(values[3]["root_label"], "beta");
        assert_eq!(values[3]["terminal_phase"], "conversion");
        assert_eq!(values[5]["kind"], "run-summary");
        assert_eq!(values[5]["disposition"], "partial");
        assert_eq!(values[5]["roots"][0]["root_label"], "alpha");
        assert_eq!(values[5]["roots"][1]["root_label"], "beta");
    }

    #[test]
    fn writer_rejects_missing_start_and_missing_summary() {
        let records = complete_records();
        let mut missing_start = EvaluationStreamWriter::new(Vec::new());
        let error = missing_start.write_record(&records[1]).unwrap_err();
        assert!(matches!(error, EvaluationStreamOutputError::MissingRunStart));

        let mut missing_summary = EvaluationStreamWriter::new(Vec::new());
        missing_summary.write_record(&records[0]).unwrap();
        let error = missing_summary.finish().unwrap_err();
        assert!(matches!(error, EvaluationStreamOutputError::MissingSummary));
    }

    struct WriteFailure;

    impl std::io::Write for WriteFailure {
        fn write(&mut self, _buffer: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "closed consumer"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    struct FlushFailure;

    impl std::io::Write for FlushFailure {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            Ok(buffer.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "closed consumer"))
        }
    }

    // r[verify evaluation_streaming.cancellation_and_output]
    #[test]
    fn writer_reports_write_and_flush_failures_without_completion() {
        let run_start = complete_records().remove(0);
        let mut write_failure = EvaluationStreamWriter::new(WriteFailure);
        let write_error = write_failure.write_record(&run_start).unwrap_err();
        assert!(matches!(write_error, EvaluationStreamOutputError::Write(_)));
        assert!(matches!(write_failure.finish(), Err(EvaluationStreamOutputError::MissingSummary)));

        let mut flush_failure = EvaluationStreamWriter::new(FlushFailure);
        let flush_error = flush_failure.write_record(&run_start).unwrap_err();
        assert!(matches!(flush_error, EvaluationStreamOutputError::Flush(_)));
        assert!(matches!(flush_failure.finish(), Err(EvaluationStreamOutputError::MissingSummary)));
    }

    #[test]
    fn writer_rejects_records_after_summary() {
        let records = complete_records();
        let mut writer = EvaluationStreamWriter::new(Vec::new());
        for record in &records {
            writer.write_record(record).unwrap();
        }
        let error = writer.write_record(&records[1]).unwrap_err();
        assert!(matches!(error, EvaluationStreamOutputError::RecordAfterSummary));
    }
}
