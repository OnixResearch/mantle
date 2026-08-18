// r[impl distributed_evaluation.feasibility_assessment]
//
// Probe shell for the distributed-evaluation feasibility assessment.
//
// This shell owns file reads, argument parsing, the bounded stdio transport,
// worker spawning, and report writes. All decision logic lives in the pure
// assessment core (`src/distributed_eval_assessment.rs`).
//
// The probe is a real transport round trip. The client renders an isolated
// eval-worker request (source text, declared import paths, source name, root
// label) and sends it over a framed stdio pipe to a fresh worker process of
// this same binary. The worker rebuilds an `EvaluationSession` purely from
// the request facts via `EvaluationSession::open_named_str`, forces the named
// root, and returns a canonical response. The client compares the worker
// response with its own in-process baseline.
//
// Usage:
//   distributed_eval_assess --self-test
//   distributed_eval_assess --probe-worker          (worker side, stdio framed)
//   distributed_eval_assess --probe-run \
//     --fixture <target.ncl> --root <label> \
//     [--import-path <dir>]... [--source-name <name>] \
//     --facts <declared-facts.json> --out <report-dir>

use std::ffi::OsString;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

#[path = "../src/distributed_eval_assessment.rs"]
mod distributed_eval_assessment;

use distributed_eval_assessment::classify;
use distributed_eval_assessment::validate_facts;
use distributed_eval_assessment::AssessmentFacts;
use distributed_eval_assessment::AssessmentOutcome;
use distributed_eval_assessment::InventoryFacts;
use distributed_eval_assessment::Route;
use distributed_eval_assessment::RouteFacts;

const PROBE_REQUEST_SCHEMA: &str = "mantle-distributed-evaluation-probe-request-v1";
const PROBE_RESPONSE_SCHEMA: &str = "mantle-distributed-evaluation-probe-response-v1";
const DECLARED_FACTS_SCHEMA: &str = "mantle-distributed-evaluation-declared-facts-v1";
const PROBE_EVIDENCE_SCHEMA: &str = "mantle-distributed-evaluation-probe-evidence-v1";

const FRAME_HEADER_BYTES: usize = 8;
const MAX_FRAME_BYTES: u64 = 8 * 1024 * 1024;
const MAX_FIXTURE_BYTES: u64 = 1_048_576;
const MAX_FACTS_BYTES: u64 = 4_194_304;
const MAX_IMPORT_PATHS: usize = 64;
const MAX_SOURCE_CHARS: usize = 1_048_576;
const MAX_SOURCE_NAME_CHARS: usize = 4_096;
const MAX_EVAL_MESSAGE_CHARS: usize = 16_384;
const SELF_TEST_DIR_PREFIX: &str = "distributed-eval-assess-self-test";

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ProbeRequest {
    schema: String,
    source: String,
    import_paths: Vec<String>,
    source_name: String,
    root_label: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct ProbeResponse {
    schema: String,
    ok: bool,
    result_json: Option<String>,
    error_class: Option<String>,
    error_message: Option<String>,
    source_blake3: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct DeclaredFacts {
    schema: String,
    inventory: InventoryFacts,
    eval_service: EvalServiceDeclaredFacts,
    evaluate_once: EvaluateOnceDeclaredFacts,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct EvalServiceDeclaredFacts {
    transport_bounded: bool,
    source_staged_by_digest: bool,
    streaming_overlap_evidence: bool,
    dynamic_goal_evidence: bool,
    suspension_evidence: bool,
    authority_blocker: bool,
    hidden_host_local_reads: bool,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct EvaluateOnceDeclaredFacts {
    source_staged_by_digest: bool,
    exported_graph_evidence: bool,
    producer_bound_evidence: bool,
    artifacts_digest_bound: bool,
    authority_blocker: bool,
}

#[derive(Clone, Debug, serde::Serialize)]
struct ProbeEvidence {
    schema: String,
    request_source_blake3: String,
    response_source_blake3: String,
    client_result_json: String,
    worker_result_json: String,
    response_matches: bool,
    worker_error_observed: bool,
    worker_error_class: Option<String>,
    worker_error_message: Option<String>,
    framed_transport_used: bool,
}

#[derive(Debug)]
enum ShellError {
    Usage(String),
    Io(String),
    Facts(String),
    Probe(String),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(detail) => write!(formatter, "usage error: {detail}"),
            Self::Io(detail) => write!(formatter, "io error: {detail}"),
            Self::Facts(detail) => write!(formatter, "facts error: {detail}"),
            Self::Probe(detail) => write!(formatter, "probe error: {detail}"),
        }
    }
}

fn usage() -> ShellError {
    ShellError::Usage(
        "see header comment: --self-test, --probe-worker, or --probe-run with \
         --fixture --root [--import-path ...] [--source-name ...] --facts --out"
            .to_string(),
    )
}

fn read_bounded(path: &Path, max_bytes: u64) -> Result<String, ShellError> {
    debug_assert!(max_bytes > 0, "max bytes must be positive");
    let metadata =
        std::fs::metadata(path).map_err(|error| ShellError::Io(format!("stat {}: {error}", path.display())))?;
    if metadata.len() > max_bytes {
        return Err(ShellError::Io(format!("{} exceeds {max_bytes} bytes", path.display())));
    }
    std::fs::read_to_string(path).map_err(|error| ShellError::Io(format!("read {}: {error}", path.display())))
}

fn blake3_hex(bytes: &[u8]) -> String {
    debug_assert!(!bytes.is_empty(), "digest input must not be empty");
    blake3::hash(bytes).to_hex().to_string()
}

fn write_frame(writer: &mut impl Write, payload: &[u8]) -> Result<(), ShellError> {
    debug_assert!(payload.len() as u64 <= MAX_FRAME_BYTES, "frame must fit the bound");
    let header = (payload.len() as u64).to_le_bytes();
    debug_assert_eq!(header.len(), FRAME_HEADER_BYTES, "frame header width");
    writer
        .write_all(&header)
        .and_then(|_| writer.write_all(payload))
        .map_err(|error| ShellError::Io(format!("write frame: {error}")))
}

fn read_frame(reader: &mut impl Read) -> Result<Vec<u8>, ShellError> {
    let mut header = [0u8; FRAME_HEADER_BYTES];
    reader
        .read_exact(&mut header)
        .map_err(|error| ShellError::Io(format!("read frame header: {error}")))?;
    let declared = u64::from_le_bytes(header);
    if declared > MAX_FRAME_BYTES {
        return Err(ShellError::Probe(format!("frame length {declared} exceeds bound {MAX_FRAME_BYTES}")));
    }
    let mut payload = vec![0u8; declared as usize];
    reader
        .read_exact(&mut payload)
        .map_err(|error| ShellError::Io(format!("read frame payload: {error}")))?;
    Ok(payload)
}

fn import_paths(request: &ProbeRequest) -> Result<Vec<OsString>, ShellError> {
    if request.import_paths.len() > MAX_IMPORT_PATHS {
        return Err(ShellError::Probe(format!(
            "request declares {} import paths, exceeding {MAX_IMPORT_PATHS}",
            request.import_paths.len()
        )));
    }
    if request.source.chars().count() > MAX_SOURCE_CHARS {
        return Err(ShellError::Probe(format!("request source exceeds {MAX_SOURCE_CHARS} chars")));
    }
    if request.source_name.chars().count() > MAX_SOURCE_NAME_CHARS {
        return Err(ShellError::Probe(format!("request source name exceeds {MAX_SOURCE_NAME_CHARS} chars")));
    }
    Ok(request.import_paths.iter().map(OsString::from).collect())
}

/// Evaluate one root in a fresh session opened purely from declared facts.
/// This is the worker-side transportability test: no fixture path, no ambient
/// cwd, no host shell state.
fn evaluate_from_request(request: &ProbeRequest) -> Result<ProbeResponse, ShellError> {
    let import_paths = import_paths(request)?;
    let mut session =
        crunch_eval::session::EvaluationSession::open_named_str(&request.source, &import_paths, &request.source_name)
            .map_err(|error| ShellError::Probe(format!("open worker session: {error}")))?;
    let source_blake3 = session.source_blake3();
    match session.force_root::<serde_json::Value>(&request.root_label) {
        Ok(value) => {
            let result_json = serde_json::to_string(&value)
                .map_err(|error| ShellError::Probe(format!("serialize result: {error}")))?;
            Ok(ProbeResponse {
                schema: PROBE_RESPONSE_SCHEMA.to_string(),
                ok: true,
                result_json: Some(result_json),
                error_class: None,
                error_message: None,
                source_blake3,
            })
        }
        Err(error) => {
            let message = format!("{error}");
            let message = if message.chars().count() > MAX_EVAL_MESSAGE_CHARS {
                message.chars().take(MAX_EVAL_MESSAGE_CHARS).collect()
            } else {
                message
            };
            Ok(ProbeResponse {
                schema: PROBE_RESPONSE_SCHEMA.to_string(),
                ok: false,
                result_json: None,
                error_class: Some("eval".to_string()),
                error_message: Some(message),
                source_blake3,
            })
        }
    }
}

fn run_probe_worker() -> Result<(), ShellError> {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();
    let payload = read_frame(&mut stdin)?;
    let request: ProbeRequest =
        serde_json::from_slice(&payload).map_err(|error| ShellError::Probe(format!("parse request: {error}")))?;
    if request.schema != PROBE_REQUEST_SCHEMA {
        return Err(ShellError::Probe(format!("unexpected request schema: {}", request.schema)));
    }
    let response = evaluate_from_request(&request)?;
    let response_bytes =
        serde_json::to_vec(&response).map_err(|error| ShellError::Probe(format!("serialize response: {error}")))?;
    write_frame(&mut stdout, &response_bytes)?;
    stdout.flush().map_err(|error| ShellError::Io(format!("flush worker stdout: {error}")))
}

fn baseline(client_request: &ProbeRequest) -> Result<ProbeResponse, ShellError> {
    evaluate_from_request(client_request)
}

fn spawn_worker(client_request: &ProbeRequest) -> Result<ProbeResponse, ShellError> {
    let executable =
        std::env::current_exe().map_err(|error| ShellError::Probe(format!("locate current exe: {error}")))?;
    let mut child = std::process::Command::new(&executable)
        .arg("--probe-worker")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|error| ShellError::Probe(format!("spawn worker: {error}")))?;

    let request_bytes =
        serde_json::to_vec(client_request).map_err(|error| ShellError::Probe(format!("serialize request: {error}")))?;
    {
        let mut stdin = child.stdin.take().expect("spawned worker stdin");
        write_frame(&mut stdin, &request_bytes).map_err(|error| ShellError::Probe(format!("send request: {error}")))?;
    }
    let mut stdout = child.stdout.take().expect("spawned worker stdout");
    let response_payload =
        read_frame(&mut stdout).map_err(|error| ShellError::Probe(format!("receive response: {error}")))?;
    let status = child.wait().map_err(|error| ShellError::Probe(format!("wait worker: {error}")))?;
    if !status.success() {
        return Err(ShellError::Probe(format!("worker exited with {status}")));
    }
    let response: ProbeResponse = serde_json::from_slice(&response_payload)
        .map_err(|error| ShellError::Probe(format!("parse response: {error}")))?;
    if response.schema != PROBE_RESPONSE_SCHEMA {
        return Err(ShellError::Probe(format!("unexpected response schema: {}", response.schema)));
    }
    Ok(response)
}

fn run_probe(request: &ProbeRequest) -> Result<ProbeEvidence, ShellError> {
    let client_baseline = baseline(request)?;
    let worker_response = spawn_worker(request)?;
    let client_result = client_baseline.result_json.clone().unwrap_or_default();
    let worker_result = worker_response.result_json.clone().unwrap_or_default();
    let response_matches = client_baseline.ok
        && worker_response.ok
        && client_result == worker_result
        && client_baseline.source_blake3 == worker_response.source_blake3;
    let evidence = ProbeEvidence {
        schema: PROBE_EVIDENCE_SCHEMA.to_string(),
        request_source_blake3: client_baseline.source_blake3.clone(),
        response_source_blake3: worker_response.source_blake3.clone(),
        client_result_json: client_result,
        worker_result_json: worker_result,
        response_matches,
        worker_error_observed: !worker_response.ok,
        worker_error_class: worker_response.error_class.clone(),
        worker_error_message: worker_response.error_message.clone(),
        framed_transport_used: true,
    };
    Ok(evidence)
}

fn run_probe_run() -> Result<AssessmentOutcome, ShellError> {
    let args: Vec<String> = std::env::args().collect();
    let mut fixture: Option<PathBuf> = None;
    let mut root_label: Option<String> = None;
    let mut import_paths: Vec<String> = Vec::new();
    let mut source_name: Option<String> = None;
    let mut facts_path: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    // argv[1] is the `--probe-run` mode token; options start at index 2.
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--fixture" => {
                fixture = Some(PathBuf::from(args.get(index + 1).ok_or_else(usage)?));
                index += 2;
            }
            "--root" => {
                root_label = Some(args.get(index + 1).ok_or_else(usage)?.clone());
                index += 2;
            }
            "--import-path" => {
                import_paths.push(args.get(index + 1).ok_or_else(usage)?.clone());
                index += 2;
            }
            "--source-name" => {
                source_name = Some(args.get(index + 1).ok_or_else(usage)?.clone());
                index += 2;
            }
            "--facts" => {
                facts_path = Some(PathBuf::from(args.get(index + 1).ok_or_else(usage)?));
                index += 2;
            }
            "--out" => {
                out_dir = Some(PathBuf::from(args.get(index + 1).ok_or_else(usage)?));
                index += 2;
            }
            _ => return Err(usage()),
        }
    }

    let fixture = fixture.ok_or_else(usage)?;
    let root_label = root_label.ok_or_else(usage)?;
    let facts_path = facts_path.ok_or_else(usage)?;
    let out_dir = out_dir.ok_or_else(usage)?;

    let source = read_bounded(&fixture, MAX_FIXTURE_BYTES)?;
    let source_name = source_name.unwrap_or_else(|| fixture.display().to_string());
    let declared_facts_json = read_bounded(&facts_path, MAX_FACTS_BYTES)?;
    let declared: DeclaredFacts = serde_json::from_str(&declared_facts_json)
        .map_err(|error| ShellError::Facts(format!("parse declared facts: {error}")))?;
    if declared.schema != DECLARED_FACTS_SCHEMA {
        return Err(ShellError::Facts(format!("unexpected declared facts schema: {}", declared.schema)));
    }

    let request = ProbeRequest {
        schema: PROBE_REQUEST_SCHEMA.to_string(),
        source,
        import_paths,
        source_name,
        root_label,
    };

    std::fs::create_dir_all(&out_dir)
        .map_err(|error| ShellError::Io(format!("create {}: {error}", out_dir.display())))?;

    let probe_evidence = run_probe(&request)?;
    let probe_digest = blake3_hex(serde_json::to_vec(&probe_evidence).expect("serializable evidence").as_slice());
    let inventory_digest = blake3_hex(declared_facts_json.as_bytes());

    let eval_service = RouteFacts {
        route: Route::EvalService,
        response_matches: probe_evidence.response_matches,
        worker_error_observed: probe_evidence.worker_error_observed,
        hidden_host_local_reads: declared.eval_service.hidden_host_local_reads,
        transport_bounded: declared.eval_service.transport_bounded,
        source_staged_by_digest: declared.eval_service.source_staged_by_digest,
        streaming_overlap_evidence: declared.eval_service.streaming_overlap_evidence,
        dynamic_goal_evidence: declared.eval_service.dynamic_goal_evidence,
        suspension_evidence: declared.eval_service.suspension_evidence,
        exported_graph_evidence: false,
        producer_bound_evidence: false,
        artifacts_digest_bound: false,
        authority_blocker: declared.eval_service.authority_blocker,
        fresh_nickel_served: true,
        evidence_digests: vec![probe_digest, inventory_digest.clone()],
    };
    let evaluate_once = RouteFacts {
        route: Route::EvaluateOnce,
        response_matches: false,
        worker_error_observed: false,
        hidden_host_local_reads: false,
        transport_bounded: false,
        source_staged_by_digest: declared.evaluate_once.source_staged_by_digest,
        streaming_overlap_evidence: false,
        dynamic_goal_evidence: false,
        suspension_evidence: false,
        exported_graph_evidence: declared.evaluate_once.exported_graph_evidence,
        producer_bound_evidence: declared.evaluate_once.producer_bound_evidence,
        artifacts_digest_bound: declared.evaluate_once.artifacts_digest_bound,
        authority_blocker: declared.evaluate_once.authority_blocker,
        fresh_nickel_served: false,
        evidence_digests: vec![inventory_digest.clone()],
    };

    let facts = AssessmentFacts {
        inventory: declared.inventory,
        routes: vec![eval_service, evaluate_once],
    };
    validate_facts(&facts).map_err(ShellError::Facts)?;
    let mut report = classify(&facts);
    report.inventory_source_digest = inventory_digest;

    let probe_path = out_dir.join("probe-evidence.json");
    let report_path = out_dir.join("assessment-report.json");
    let evidence_json = serde_json::to_string_pretty(&probe_evidence)
        .map_err(|error| ShellError::Facts(format!("probe evidence serialization: {error}")))?;
    let report_json = serde_json::to_string_pretty(&report)
        .map_err(|error| ShellError::Facts(format!("report serialization: {error}")))?;
    std::fs::write(&probe_path, evidence_json)
        .map_err(|error| ShellError::Io(format!("write {}: {error}", probe_path.display())))?;
    std::fs::write(&report_path, report_json)
        .map_err(|error| ShellError::Io(format!("write {}: {error}", report_path.display())))?;

    let outcome = report.outcomes[0].outcome;
    for route_outcome in &report.outcomes {
        println!("route={} outcome={}", route_outcome.route, route_outcome.outcome);
        for reason in &route_outcome.reasons {
            println!("reason={reason}");
        }
    }
    println!("probe-evidence={}", probe_path.display());
    println!("assessment-report={}", report_path.display());
    Ok(outcome)
}

fn run_self_test() -> Result<(), ShellError> {
    let work = std::env::temp_dir().join(SELF_TEST_DIR_PREFIX);
    std::fs::create_dir_all(&work).map_err(|error| ShellError::Io(format!("self-test dir: {error}")))?;

    // The shortest honest self-contained fixture: a static top-level record.
    let source = "{ greeting = \"hello-from-worker\" }";
    let import_paths: Vec<String> = Vec::new();
    let request = ProbeRequest {
        schema: PROBE_REQUEST_SCHEMA.to_string(),
        source: source.to_string(),
        import_paths,
        source_name: "<self-test>".to_string(),
        root_label: "greeting".to_string(),
    };
    let response = evaluate_from_request(&request)?;
    assert!(response.ok, "self-test worker evaluation must succeed");
    assert_eq!(response.result_json.as_deref(), Some("\"hello-from-worker\""), "self-test result");

    // Framing round trip must be lossless and bounded.
    let mut encoded: Vec<u8> = Vec::new();
    write_frame(&mut encoded, b"probe-frame-payload")?;
    assert_eq!(encoded.len(), FRAME_HEADER_BYTES + "probe-frame-payload".len(), "frame length");
    let mut cursor = std::io::Cursor::new(encoded);
    let decoded = read_frame(&mut cursor)?;
    assert_eq!(decoded, b"probe-frame-payload", "frame round trip");

    // Oversized frame must fail closed before allocation.
    let mut oversized: Vec<u8> = Vec::new();
    oversized.extend_from_slice(&(MAX_FRAME_BYTES + 1).to_le_bytes());
    let mut oversized_cursor = std::io::Cursor::new(oversized);
    assert!(read_frame(&mut oversized_cursor).is_err(), "oversized frame must fail");

    // Core classification triples: candidate, blocked, rejected.
    let candidate_facts = assessment_self_test_facts();
    let candidate_report = classify(&candidate_facts);
    assert!(candidate_report.outcomes.iter().all(|o| o.outcome == AssessmentOutcome::Candidate));
    let mut blocked_facts = candidate_facts.clone();
    blocked_facts.routes[0].streaming_overlap_evidence = false;
    let blocked_report = classify(&blocked_facts);
    assert_eq!(blocked_report.outcomes[0].outcome, AssessmentOutcome::Blocked);
    let mut rejected_facts = candidate_facts.clone();
    rejected_facts.routes[0].response_matches = false;
    let rejected_report = classify(&rejected_facts);
    assert_eq!(rejected_report.outcomes[0].outcome, AssessmentOutcome::Rejected);

    println!("self-test-ok");
    Ok(())
}

fn required_seams_with_bindings() -> (Vec<String>, Vec<String>) {
    let seams: Vec<String> =
        distributed_eval_assessment::required_seam_names().iter().map(ToString::to_string).collect();
    let bindings: Vec<String> = seams.iter().map(|seam| format!("{seam}=#crates/crunch-eval/src/session.rs")).collect();
    debug_assert_eq!(seams.len(), bindings.len(), "one binding per seam");
    (seams, bindings)
}

fn assessment_self_test_facts() -> AssessmentFacts {
    let (seams, bindings) = required_seams_with_bindings();
    let inventory = InventoryFacts {
        seams: seams.into_iter().collect(),
        bindings: bindings.into_iter().collect(),
    };
    let eval_service = RouteFacts {
        route: Route::EvalService,
        response_matches: true,
        worker_error_observed: false,
        hidden_host_local_reads: false,
        transport_bounded: true,
        source_staged_by_digest: true,
        streaming_overlap_evidence: true,
        dynamic_goal_evidence: true,
        suspension_evidence: true,
        exported_graph_evidence: false,
        producer_bound_evidence: false,
        artifacts_digest_bound: false,
        authority_blocker: false,
        fresh_nickel_served: true,
        evidence_digests: vec!["a".repeat(64)],
    };
    let evaluate_once = RouteFacts {
        route: Route::EvaluateOnce,
        response_matches: false,
        worker_error_observed: false,
        hidden_host_local_reads: false,
        transport_bounded: false,
        source_staged_by_digest: true,
        streaming_overlap_evidence: false,
        dynamic_goal_evidence: false,
        suspension_evidence: false,
        exported_graph_evidence: true,
        producer_bound_evidence: true,
        artifacts_digest_bound: true,
        authority_blocker: false,
        fresh_nickel_served: false,
        evidence_digests: vec!["b".repeat(64)],
    };
    AssessmentFacts {
        inventory,
        routes: vec![eval_service, evaluate_once],
    }
}

fn run() -> Result<(), ShellError> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--self-test") => run_self_test(),
        Some("--probe-worker") => run_probe_worker(),
        Some("--probe-run") => {
            run_probe_run()?;
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
