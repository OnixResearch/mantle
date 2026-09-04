use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use verified_logic::fenced_attempt_admission as trellis;

const SCHEMA: &str = "mantle-trellis-remote-admission-oracle-v1";
const TRELLIS_REVISION: &str = "8de4b24aa2d66cc2e6ec966d686df023492265d3";
const TRELLIS_SOURCE_ARCHIVE_BLAKE3: &str =
    "e13e9f71da4964ab4d4f04d9c29525b20f0e778997da5ada7b6caa0f1711f56c";
const RECORD_BYTES: usize = 7;
const EXPECTED_CASE_COUNT: usize = 6_720;
const BASE_PROGRESS: u32 = 3;
const AVAILABLE_EVENT_COUNT: usize = 3;
const JOB_CURRENT: u64 = 1;
const JOB_OTHER: u64 = 2;
const RUN_CURRENT: u64 = 10;
const RUN_OTHER: u64 = 11;
const EPOCH_CURRENT: u64 = 100;
const EPOCH_STALE: u64 = EPOCH_CURRENT - 1;
const EPOCH_FUTURE: u64 = EPOCH_CURRENT + 1;
const EVENT_INCOMING: u64 = 200;
const EVENT_OTHER_BASE: u64 = 1_000;
const PAYLOAD_INCOMING: u64 = 300;
const PAYLOAD_CONFLICT: u64 = 301;
const PAYLOAD_OTHER_BASE: u64 = 2_000;
const RESULT_PRIMARY: u64 = 777;
const RESULT_ALTERNATE: u64 = 778;
const NO_REJECTION_CODE: u8 = u8::MAX;

#[derive(Clone, Copy)]
enum ReportVariant {
    Start,
    Heartbeat,
    LogAppend,
    Checkpoint,
    ResultReady,
    Failure,
    CompletionPrimary,
    CompletionAlternate,
}

#[derive(Clone, Copy)]
enum IdentityClass {
    Current,
    Stale,
    Future,
    WrongJob,
    WrongRun,
}

#[derive(Clone, Copy)]
enum EventClass {
    New,
    Replayed,
    Conflict,
}

#[derive(Clone, Copy)]
enum HistoryClass {
    Available,
    Full,
}

fn main() {
    let mut arguments = env::args_os().skip(1);
    let oracle_path = PathBuf::from(arguments.next().expect("oracle output path"));
    let manifest_path = PathBuf::from(arguments.next().expect("manifest output path"));
    assert!(arguments.next().is_none(), "generator accepts exactly two paths");

    let mut bytes = Vec::with_capacity(EXPECTED_CASE_COUNT * RECORD_BYTES);
    let mut dispositions = BTreeMap::<String, u32>::new();
    let mut rejections = BTreeMap::<String, u32>::new();
    for_each_case(|state, report| {
        let record = encode_outcome(&state, &report, trellis::apply_report(&state, &report));
        increment(&mut dispositions, disposition_name(record[0]));
        if record[1] != NO_REJECTION_CODE {
            increment(&mut rejections, rejection_name(record[1]));
        }
        bytes.extend_from_slice(&record);
    });
    assert_eq!(bytes.len(), EXPECTED_CASE_COUNT * RECORD_BYTES);

    let matrix_blake3 = blake3::hash(&bytes).to_hex().to_string();
    let manifest = serde_json::json!({
        "schema": SCHEMA,
        "trellis_revision": TRELLIS_REVISION,
        "trellis_source_archive_blake3": TRELLIS_SOURCE_ARCHIVE_BLAKE3,
        "record_bytes": RECORD_BYTES,
        "case_count": EXPECTED_CASE_COUNT,
        "matrix_blake3": matrix_blake3,
        "axis_order": [
            "phase",
            "mantle-report-variant",
            "identity-class",
            "event-class",
            "history-class",
            "worker-scope",
            "output-scope"
        ],
        "axis_cardinality": {
            "phase": 7,
            "mantle-report-variant": 8,
            "identity-class": 5,
            "event-class": 3,
            "history-class": 2,
            "worker-scope": 2,
            "output-scope": 2
        },
        "disposition_counts": dispositions,
        "rejection_counts": rejections,
        "record_layout": [
            "disposition",
            "reject-class-or-255",
            "next-phase",
            "progress-delta",
            "event-delta",
            "result-effect",
            "state-preserved"
        ]
    });
    fs::write(&oracle_path, &bytes).expect("write oracle");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize manifest"),
    )
    .expect("write manifest");
    println!(
        "trellis oracle generated: cases={} bytes={} blake3={}",
        EXPECTED_CASE_COUNT,
        bytes.len(),
        matrix_blake3
    );
}

fn for_each_case(mut visit: impl FnMut(trellis::State, trellis::Report)) {
    for phase in phases() {
        for report_variant in report_variants() {
            for identity in identity_classes() {
                for event in event_classes() {
                    for history in history_classes() {
                        for worker in scopes() {
                            for output in scopes() {
                                let state = state(phase, event, history);
                                let report = report(report_variant, identity, event, worker, output);
                                visit(state, report);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn state(
    phase: trellis::Phase,
    event_class: EventClass,
    history_class: HistoryClass,
) -> trellis::State {
    let event_count = match history_class {
        HistoryClass::Available => AVAILABLE_EVENT_COUNT,
        HistoryClass::Full => usize::try_from(trellis::MAX_EVENTS).expect("event bound fits usize"),
    };
    let mut events = Vec::with_capacity(event_count);
    if matches!(event_class, EventClass::Replayed | EventClass::Conflict) {
        events.push((trellis::EventId(EVENT_INCOMING), retained_payload(event_class)));
    }
    while events.len() < event_count {
        let index = u64::try_from(events.len()).expect("bounded event index fits u64");
        events.push((
            trellis::EventId(EVENT_OTHER_BASE + index),
            trellis::PayloadId(PAYLOAD_OTHER_BASE + index),
        ));
    }
    let result = matches!(phase, trellis::Phase::ResultReady | trellis::Phase::Completed)
        .then(primary_result);
    trellis::State {
        job: trellis::JobId(JOB_CURRENT),
        run: trellis::RunId(RUN_CURRENT),
        epoch: trellis::Epoch(EPOCH_CURRENT),
        phase,
        progress: BASE_PROGRESS,
        events,
        result,
    }
}

fn report(
    variant: ReportVariant,
    identity: IdentityClass,
    event: EventClass,
    worker: trellis::Scope,
    output: trellis::Scope,
) -> trellis::Report {
    let (job, run, epoch) = report_identity(identity);
    trellis::Report {
        job,
        run,
        epoch,
        event: trellis::EventId(EVENT_INCOMING),
        payload: incoming_payload(event),
        kind: report_kind(variant),
        worker,
        output,
        result: report_result(variant),
    }
}

fn report_identity(
    identity: IdentityClass,
) -> (trellis::JobId, trellis::RunId, trellis::Epoch) {
    match identity {
        IdentityClass::Current => current_identity(),
        IdentityClass::Stale => (
            trellis::JobId(JOB_CURRENT),
            trellis::RunId(RUN_CURRENT),
            trellis::Epoch(EPOCH_STALE),
        ),
        IdentityClass::Future => (
            trellis::JobId(JOB_CURRENT),
            trellis::RunId(RUN_CURRENT),
            trellis::Epoch(EPOCH_FUTURE),
        ),
        IdentityClass::WrongJob => (
            trellis::JobId(JOB_OTHER),
            trellis::RunId(RUN_CURRENT),
            trellis::Epoch(EPOCH_CURRENT),
        ),
        IdentityClass::WrongRun => (
            trellis::JobId(JOB_CURRENT),
            trellis::RunId(RUN_OTHER),
            trellis::Epoch(EPOCH_CURRENT),
        ),
    }
}

fn current_identity() -> (trellis::JobId, trellis::RunId, trellis::Epoch) {
    (
        trellis::JobId(JOB_CURRENT),
        trellis::RunId(RUN_CURRENT),
        trellis::Epoch(EPOCH_CURRENT),
    )
}

fn retained_payload(event: EventClass) -> trellis::PayloadId {
    match event {
        EventClass::Replayed => trellis::PayloadId(PAYLOAD_INCOMING),
        EventClass::Conflict => trellis::PayloadId(PAYLOAD_CONFLICT),
        EventClass::New => unreachable!("new events are not retained"),
    }
}

fn incoming_payload(_event: EventClass) -> trellis::PayloadId {
    trellis::PayloadId(PAYLOAD_INCOMING)
}

fn report_kind(variant: ReportVariant) -> trellis::ReportKind {
    match variant {
        ReportVariant::Start => trellis::ReportKind::Start,
        ReportVariant::Heartbeat | ReportVariant::LogAppend => trellis::ReportKind::Progress,
        ReportVariant::Checkpoint => trellis::ReportKind::Checkpoint,
        ReportVariant::ResultReady => trellis::ReportKind::ResultReady,
        ReportVariant::Failure => trellis::ReportKind::Failure,
        ReportVariant::CompletionPrimary | ReportVariant::CompletionAlternate => {
            trellis::ReportKind::Completion
        }
    }
}

fn report_result(variant: ReportVariant) -> Option<trellis::ResultLink> {
    match variant {
        ReportVariant::ResultReady | ReportVariant::CompletionPrimary => Some(primary_result()),
        ReportVariant::CompletionAlternate => Some(trellis::ResultLink {
            run: trellis::RunId(RUN_CURRENT),
            payload: trellis::PayloadId(RESULT_ALTERNATE),
        }),
        ReportVariant::Start
        | ReportVariant::Heartbeat
        | ReportVariant::LogAppend
        | ReportVariant::Checkpoint
        | ReportVariant::Failure => None,
    }
}

fn primary_result() -> trellis::ResultLink {
    trellis::ResultLink {
        run: trellis::RunId(RUN_CURRENT),
        payload: trellis::PayloadId(RESULT_PRIMARY),
    }
}

fn encode_outcome(
    before: &trellis::State,
    report: &trellis::Report,
    outcome: Result<trellis::Disposition, trellis::Rejection>,
) -> [u8; RECORD_BYTES] {
    match outcome {
        Ok(trellis::Disposition::Applied(after)) => [
            0,
            NO_REJECTION_CODE,
            after.phase.tag(),
            delta_u32(before.progress, after.progress),
            delta_usize(before.events.len(), after.events.len()),
            result_effect(before.result, after.result, report.result),
            u8::from(before == &after),
        ],
        Ok(trellis::Disposition::Replayed) => [
            1,
            NO_REJECTION_CODE,
            before.phase.tag(),
            0,
            0,
            0,
            1,
        ],
        Err(rejection) => [
            2,
            rejection.class.tag(),
            before.phase.tag(),
            0,
            0,
            0,
            1,
        ],
    }
}

fn delta_u32(before: u32, after: u32) -> u8 {
    u8::try_from(after.checked_sub(before).expect("progress does not decrease"))
        .expect("progress delta is bounded")
}

fn delta_usize(before: usize, after: usize) -> u8 {
    u8::try_from(after.checked_sub(before).expect("event count does not decrease"))
        .expect("event delta is bounded")
}

fn result_effect(
    before: Option<trellis::ResultLink>,
    after: Option<trellis::ResultLink>,
    report: Option<trellis::ResultLink>,
) -> u8 {
    if before == after {
        0
    } else if after == report {
        1
    } else if before.is_some() && after.is_none() {
        2
    } else {
        panic!("unrecognized result effect")
    }
}

fn increment(counts: &mut BTreeMap<String, u32>, key: &str) {
    let count = counts.entry(key.to_string()).or_default();
    *count = count.checked_add(1).expect("matrix count fits u32");
}

fn disposition_name(code: u8) -> &'static str {
    match code {
        0 => "applied",
        1 => "replayed",
        2 => "rejected",
        _ => panic!("unknown disposition code"),
    }
}

fn rejection_name(code: u8) -> &'static str {
    match code {
        0 => "stale-epoch",
        1 => "future-epoch",
        2 => "wrong-job",
        3 => "wrong-run",
        4 => "event-conflict",
        5 => "terminal-phase",
        6 => "invalid-transition",
        7 => "worker-denied",
        8 => "output-denied",
        9 => "missing-result",
        10 => "result-mismatch",
        11 => "epoch-exhausted",
        12 => "history-full",
        _ => panic!("unknown rejection code"),
    }
}

fn phases() -> [trellis::Phase; 7] {
    [
        trellis::Phase::Queued,
        trellis::Phase::Running,
        trellis::Phase::Transferring,
        trellis::Phase::ResultReady,
        trellis::Phase::Completed,
        trellis::Phase::Failed,
        trellis::Phase::Superseded,
    ]
}

fn report_variants() -> [ReportVariant; 8] {
    [
        ReportVariant::Start,
        ReportVariant::Heartbeat,
        ReportVariant::LogAppend,
        ReportVariant::Checkpoint,
        ReportVariant::ResultReady,
        ReportVariant::Failure,
        ReportVariant::CompletionPrimary,
        ReportVariant::CompletionAlternate,
    ]
}

fn identity_classes() -> [IdentityClass; 5] {
    [
        IdentityClass::Current,
        IdentityClass::Stale,
        IdentityClass::Future,
        IdentityClass::WrongJob,
        IdentityClass::WrongRun,
    ]
}

fn event_classes() -> [EventClass; 3] {
    [EventClass::New, EventClass::Replayed, EventClass::Conflict]
}

fn history_classes() -> [HistoryClass; 2] {
    [HistoryClass::Available, HistoryClass::Full]
}

fn scopes() -> [trellis::Scope; 2] {
    [trellis::Scope::Granted, trellis::Scope::Denied]
}
