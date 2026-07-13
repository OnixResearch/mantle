use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use base64::Engine;
use serde_json::Value;
use sha2::Digest;

const STORE_PREFIX: &str = "/mantle/store";
const INPUT_STORE_PATH: &str = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-production-input";
const INPUT_BYTES: usize = 196_608;
const OUTPUT_BYTES: usize = 262_144;
const PRODUCTION_SCALE_OUTPUT_BYTES: usize = 8_388_608;
const MIN_PRODUCTION_SCALE_OUTPUT_CHUNKS: usize = 100;
const PATTERN_MODULUS: usize = 251;
const INTERRUPT_AFTER_CHUNKS: &str = "1";
const TICKET_ID: &str = "production-ticket";
const TICKET_SECRET: &str = "production-secret";
const BUILDER_ID: &str = "production-builder";
const MAX_BUILD_TIME_SECS: u64 = 600;
const MAX_UPLOAD_BYTES: u64 = 1_073_741_824;
const REJECTING_UPLOAD_BYTES: u64 = 1;
const TICKET_USES: u32 = 2;
const TEST_CREATED_UNIX_S: u64 = 1;
const TEST_EXPIRES_UNIX_S: u64 = u64::MAX;
const INPUT_INTERRUPT_ENV: &str = "MANTLE_TEST_REMOTE_INTERRUPT_AFTER_INPUT_CHUNKS";
const INTERRUPT_ENV: &str = "MANTLE_TEST_REMOTE_INTERRUPT_AFTER_OUTPUT_CHUNKS";
const MAX_TEST_REMOTE_WORKERS: usize = 16;
const VALID_TRACEPARENT: &str = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
const VALID_TRACESTATE: &str = "vendor=value";
const TRACEPARENT_ENV: &str = "TRACEPARENT";
const TRACESTATE_ENV: &str = "TRACESTATE";
const OTLP_OUTAGE_ENDPOINT: &str = "http://127.0.0.1:9/v1/metrics";
const OTLP_TEST_TIMEOUT_MS: u32 = 100;
const MALFORMED_TRACEPARENT_BYTES: usize = 4_096;
const REMOTE_ATTEMPT_LOG_DIR: &str = "remote-attempt-logs";
const MAX_DIAGNOSTIC_SCAN_FILES: usize = 4_096;
const MAX_DIAGNOSTIC_SCAN_BYTES: usize = 1_048_576;
const MAX_DIAGNOSTIC_JSON_NODES: usize = 65_536;
const PRIORITY_LIFECYCLE_PREFIX_COUNT: usize = 5;

#[test]
fn production_stdio_resumes_missing_chunks_and_imports_output() {
    let root = tempfile::tempdir().expect("production transfer tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let output_source = root.path().join("output-source.bin");
    let build_file = root.path().join("remote-production.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&store_dir).expect("store dir");
    fs::write(&output_source, patterned_bytes(OUTPUT_BYTES)).expect("output source");
    let input_host_path = physical_store_path(&store_dir, INPUT_STORE_PATH);
    fs::write(&input_host_path, patterned_bytes(INPUT_BYTES)).expect("input source");
    write_ticket_state(&state_dir);
    write_fetch_derivation(&build_file, &output_source);

    let interrupted = remote_build_command(&state_dir, &store_dir, &build_file)
        .env(INTERRUPT_ENV, INTERRUPT_AFTER_CHUNKS)
        .output()
        .expect("run interrupted production transfer");
    assert!(!interrupted.status.success(), "first run must interrupt after a durable chunk");
    let interrupted_stderr = String::from_utf8_lossy(&interrupted.stderr);
    assert!(
        interrupted_stderr.contains("remote-production-transfer-interrupted-after-checkpoint"),
        "unexpected interruption stderr: {interrupted_stderr}"
    );
    assert!(state_diagnostics_contain(&state_dir, "transfer-cutoff"));
    assert!(!state_diagnostics_contain(&state_dir, "execution-failed"));
    assert!(!state_diagnostics_contain(&state_dir, "output-admitted"));
    let first_run_checkpoints = transfer_checkpoints(&state_dir);
    let completed_upload = first_run_checkpoints
        .iter()
        .filter(|summary| summary.acknowledged_chunks > 1)
        .max_by_key(|summary| summary.transferred_bytes)
        .expect("completed multi-chunk production input upload checkpoint");
    assert!(completed_upload.transferred_bytes > u64::try_from(INPUT_BYTES).unwrap());
    let partial = partial_output_checkpoint(&state_dir);
    assert_eq!(partial.acknowledged_chunks, 1, "one output chunk must be durable before restart");
    assert!(partial.transferred_bytes > 0, "partial checkpoint records transferred bytes");

    let resumed = remote_build_command(&state_dir, &store_dir, &build_file)
        .output()
        .expect("run resumed production transfer");
    assert!(resumed.status.success(), "resume stderr={}", String::from_utf8_lossy(&resumed.stderr));
    let report: Value = serde_json::from_slice(&resumed.stdout).expect("production build JSON report");
    let substitution = &report["outcomes"][0]["outputs"][0]["substitution"];
    assert_eq!(substitution["mode"], "streaming");
    assert_production_lifecycle_sequence(&report, true);
    assert!(substitution["reused_bytes"].as_u64().unwrap_or(0) >= partial.transferred_bytes);
    let output_path =
        PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("imported output path"));
    assert_eq!(fs::read(output_path).expect("imported output bytes"), patterned_bytes(OUTPUT_BYTES));
    let completed = output_checkpoint_with_most_acknowledgements(&state_dir);
    assert!(completed.acknowledged_chunks > partial.acknowledged_chunks);
    assert!(completed.transferred_bytes > partial.transferred_bytes);
}

#[test]
fn production_stdio_resumes_interrupted_multi_chunk_input_upload() {
    let root = tempfile::tempdir().expect("production input resume tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let output_source = root.path().join("output-source.bin");
    let build_file = root.path().join("remote-production.ncl");
    fs::create_dir_all(&state_dir).expect("state dir");
    fs::create_dir_all(&store_dir).expect("store dir");
    fs::write(&output_source, patterned_bytes(OUTPUT_BYTES)).expect("output source");
    fs::write(physical_store_path(&store_dir, INPUT_STORE_PATH), patterned_bytes(INPUT_BYTES)).expect("input source");
    write_ticket_state(&state_dir);
    write_fetch_derivation(&build_file, &output_source);

    let interrupted = remote_build_command(&state_dir, &store_dir, &build_file)
        .env(INPUT_INTERRUPT_ENV, INTERRUPT_AFTER_CHUNKS)
        .output()
        .expect("run interrupted production input transfer");
    assert!(!interrupted.status.success());
    let interrupted_stderr = String::from_utf8_lossy(&interrupted.stderr);
    assert!(
        interrupted_stderr.contains("remote-production-input-transfer-interrupted-after-checkpoint"),
        "unexpected input interruption stderr: {interrupted_stderr}"
    );
    let partial = partial_output_checkpoint(&state_dir);
    assert_eq!(partial.acknowledged_chunks, 1);
    assert!(partial.transferred_bytes > 0);

    let resumed = remote_build_command(&state_dir, &store_dir, &build_file)
        .output()
        .expect("resume production input transfer");
    assert!(resumed.status.success(), "resume stderr={}", String::from_utf8_lossy(&resumed.stderr));
    let completed = read_checkpoint_summary(&partial.path);
    assert!(completed.acknowledged_chunks > partial.acknowledged_chunks);
    assert!(completed.transferred_bytes > u64::try_from(INPUT_BYTES).unwrap());
    let report: Value = serde_json::from_slice(&resumed.stdout).expect("resumed production JSON report");
    let imported_path =
        PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("imported output path"));
    assert_eq!(fs::read(imported_path).unwrap(), patterned_bytes(OUTPUT_BYTES));
}

#[test]
fn production_stdio_rejects_ticket_upload_quota_before_checkpoint_or_admission() {
    let root = tempfile::tempdir().expect("production quota tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let output_source = root.path().join("output-source.bin");
    let build_file = root.path().join("remote-production.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::write(&output_source, patterned_bytes(OUTPUT_BYTES)).unwrap();
    fs::write(physical_store_path(&store_dir, INPUT_STORE_PATH), patterned_bytes(INPUT_BYTES)).unwrap();
    write_ticket_state_with_upload_limit(&state_dir, REJECTING_UPLOAD_BYTES);
    write_fetch_derivation(&build_file, &output_source);

    let rejected = remote_build_command(&state_dir, &store_dir, &build_file)
        .output()
        .expect("run quota-rejected production transfer");
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("upload-byte-limit-exceeded"));
    assert!(transfer_checkpoints(&state_dir).is_empty());
    assert_eq!(fs::read_dir(&store_dir).unwrap().count(), 1);
    assert_eq!(fs::read(physical_store_path(&store_dir, INPUT_STORE_PATH)).unwrap(), patterned_bytes(INPUT_BYTES));
}

#[test]
fn production_stdio_delta_unavailable_falls_back_to_bounded_full_nar_and_admits() {
    let root = tempfile::tempdir().expect("production delta fallback tempdir");
    let fixture = setup_production_fixture(root.path(), OUTPUT_BYTES, MAX_UPLOAD_BYTES);
    let completed = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .arg("--remote-delta")
        .output()
        .expect("run production delta-unavailable fallback");
    assert!(completed.status.success(), "fallback stderr={}", String::from_utf8_lossy(&completed.stderr));
    let report: Value = serde_json::from_slice(&completed.stdout).expect("fallback production JSON report");
    let substitution = &report["outcomes"][0]["outputs"][0]["substitution"];
    assert_eq!(substitution["mode"], "full");
    assert_eq!(substitution["fallback_reason"], "delta-unavailable");
    let imported_path =
        PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("fallback imported output path"));
    assert_eq!(fs::read(imported_path).unwrap(), patterned_bytes(OUTPUT_BYTES));
    let output_checkpoint = output_checkpoint_with_most_acknowledgements(&fixture.state_dir);
    assert!(output_checkpoint.acknowledged_chunks > 1);
    assert!(output_checkpoint.transferred_bytes > u64::try_from(OUTPUT_BYTES).unwrap());
}

#[test]
fn production_stdio_streams_and_admits_8_mib_output() {
    let root = tempfile::tempdir().expect("production 8 MiB tempdir");
    let fixture = setup_production_fixture(root.path(), PRODUCTION_SCALE_OUTPUT_BYTES, MAX_UPLOAD_BYTES);
    let completed = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .output()
        .expect("run production 8 MiB transfer");
    assert!(completed.status.success(), "8 MiB stderr={}", String::from_utf8_lossy(&completed.stderr));
    let report: Value = serde_json::from_slice(&completed.stdout).expect("8 MiB production JSON report");
    let substitution = &report["outcomes"][0]["outputs"][0]["substitution"];
    assert_eq!(substitution["mode"], "streaming");
    assert_eq!(substitution["fallback_reason"], Value::Null);
    let imported_path =
        PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("8 MiB imported output path"));
    assert_eq!(fs::read(imported_path).unwrap(), patterned_bytes(PRODUCTION_SCALE_OUTPUT_BYTES));
    let output_checkpoint = output_checkpoint_with_most_acknowledgements(&fixture.state_dir);
    assert!(output_checkpoint.acknowledged_chunks > MIN_PRODUCTION_SCALE_OUTPUT_CHUNKS);
    assert!(output_checkpoint.transferred_bytes > u64::try_from(PRODUCTION_SCALE_OUTPUT_BYTES).unwrap());
}

#[test]
fn production_stdio_exports_prometheus_and_propagates_bounded_trace_context() {
    let root = tempfile::Builder::new()
        .prefix("observability-positive")
        .tempdir()
        .expect("production fixture tempdir");
    let fixture = setup_production_fixture(root.path(), OUTPUT_BYTES, MAX_UPLOAD_BYTES);
    let config = root.path().join("observability.ncl");
    let prometheus = root.path().join("mantle-remote.prom");
    write_observability_config(&config, Some(&prometheus), None);

    let output = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .env(TRACEPARENT_ENV, VALID_TRACEPARENT)
        .env(TRACESTATE_ENV, VALID_TRACESTATE)
        .output()
        .expect("run trace-enabled production transfer");
    assert!(output.status.success(), "trace-enabled stderr={}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("trace-enabled production report");
    let observability = &report["remote_observability"][0];
    assert_eq!(observability["health"]["trace_context"]["status"], "accepted");
    assert_eq!(observability["health"]["exporters"]["prometheus"]["status"], "succeeded");
    assert!(observability["immutable_log"]["head_record_blake3"].as_str().is_some());
    assert!(observability["immutable_log"]["next_cursor"].as_u64().unwrap_or(0) > 0);
    assert_production_lifecycle_sequence(&report, false);
    let rendered = String::from_utf8(output.stdout).unwrap();
    let metrics = fs::read_to_string(prometheus).expect("Prometheus textfile");
    assert!(!rendered.contains(VALID_TRACEPARENT));
    assert!(!metrics.contains(VALID_TRACEPARENT));
    assert!(metrics.contains("mantle_remote_admission_total"));
}

#[test]
fn production_stdio_drops_malformed_trace_and_survives_otlp_outage() {
    let root = tempfile::Builder::new()
        .prefix("observability-outage")
        .tempdir()
        .expect("production fixture tempdir");
    let fixture = setup_production_fixture(root.path(), OUTPUT_BYTES, MAX_UPLOAD_BYTES);
    let config = root.path().join("observability.ncl");
    write_observability_config(&config, None, Some(OTLP_OUTAGE_ENDPOINT));
    let malformed_traceparent = "x".repeat(MALFORMED_TRACEPARENT_BYTES);

    let output = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .env(TRACEPARENT_ENV, &malformed_traceparent)
        .env(TRACESTATE_ENV, VALID_TRACESTATE)
        .output()
        .expect("run outage-isolated production transfer");
    assert!(output.status.success(), "outage stderr={}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("outage production report");
    let health = &report["remote_observability"][0]["health"];
    assert_eq!(health["trace_context"]["status"], "dropped");
    assert_eq!(health["exporters"]["otlp"]["status"], "failed");
    assert_eq!(health["immutable_log"]["status"], "succeeded");
    assert_production_lifecycle_sequence(&report, false);
    let output_path = PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().unwrap());
    assert_eq!(fs::read(output_path).unwrap(), patterned_bytes(OUTPUT_BYTES));
    assert!(!String::from_utf8(output.stdout).unwrap().contains(&malformed_traceparent));
}

struct ProductionFixture {
    state_dir: PathBuf,
    store_dir: PathBuf,
    build_file: PathBuf,
}

fn setup_production_fixture(root: &Path, output_bytes: usize, max_upload_bytes: u64) -> ProductionFixture {
    let state_dir = root.join("state");
    let store_dir = root.join("store");
    let output_source = root.join("output-source.bin");
    let build_file = root.join("remote-production.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::write(&output_source, patterned_bytes(output_bytes)).unwrap();
    fs::write(physical_store_path(&store_dir, INPUT_STORE_PATH), patterned_bytes(INPUT_BYTES)).unwrap();
    write_ticket_state_with_upload_limit(&state_dir, max_upload_bytes);
    write_fetch_derivation(&build_file, &output_source);
    assert!(output_bytes > 0);
    assert!(max_upload_bytes > 0);
    ProductionFixture {
        state_dir,
        store_dir,
        build_file,
    }
}

fn write_observability_config(path: &Path, prometheus: Option<&Path>, otlp_endpoint: Option<&str>) {
    let contract = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/remote-builders.ncl");
    let prometheus_config = prometheus.map_or_else(
        || "{ enabled = false }".to_string(),
        |path| format!("{{ enabled = true, textfile_path = \"{}\" }}", path.display()),
    );
    let otlp_config = otlp_endpoint.map_or_else(
        || "{ enabled = false }".to_string(),
        |endpoint| format!("{{ enabled = true, endpoint = \"{endpoint}\", timeout_ms = {OTLP_TEST_TIMEOUT_MS} }}"),
    );
    let source = format!(
        "let remote = import \"{}\" in {{ pools = [], telemetry = ({{ prometheus = {prometheus_config}, otlp = {otlp_config} }} | remote.RemoteTelemetry), trace_context = {{ enabled = true }} }}",
        contract.display(),
    );
    fs::write(path, source).expect("observability config");
}

fn assert_production_lifecycle_sequence(report: &Value, expect_resume: bool) {
    let reasons = report["remote_telemetry_events"]
        .as_array()
        .expect("remote telemetry array")
        .iter()
        .map(|event| event["reason"].as_str().expect("telemetry reason"))
        .collect::<Vec<_>>();
    assert_eq!(&reasons[..PRIORITY_LIFECYCLE_PREFIX_COUNT], [
        "priority-selected",
        "route-selected",
        "queue-admitted",
        "worker-assigned",
        "fence-accepted"
    ]);
    assert_eq!(reasons.last(), Some(&"output-admitted"));
    assert!(reasons.contains(&"transfer-demand"));
    assert!(reasons.contains(&"transfer-credit"));
    assert!(reasons.contains(&"transfer-completed"));
    assert!(reasons.contains(&"execution-completed"));
    assert_eq!(reasons.contains(&"transfer-resumed"), expect_resume);
    let priority_event = &report["remote_telemetry_events"][0];
    assert_eq!(priority_event["measurement"], "candidate-count");
    assert_eq!(priority_event["value"], 1);
    let decisions = report["scheduler_priority_decisions"].as_array().expect("priority decisions");
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0]["competing_goal_count"], 1);
    assert_eq!(decisions[0]["known_critical_path_nodes"], 0);
    assert_eq!(decisions[0]["known_critical_path_work_units"], 0);
    assert_eq!(decisions[0]["blocked_root_count"], 0);
    assert_eq!(decisions[0]["history_basis"], "structural-fallback-missing");
    let execution = reasons.iter().position(|reason| *reason == "execution-completed").unwrap();
    let admission = reasons.iter().position(|reason| *reason == "output-admitted").unwrap();
    assert!(execution < admission);
}

#[derive(Debug)]
struct CheckpointSummary {
    path: PathBuf,
    acknowledged_chunks: usize,
    transferred_bytes: u64,
}

fn partial_output_checkpoint(state_dir: &Path) -> CheckpointSummary {
    let checkpoints = transfer_checkpoints(state_dir);
    checkpoints
        .into_iter()
        .filter(|summary| summary.acknowledged_chunks == 1)
        .max_by_key(|summary| summary.transferred_bytes)
        .expect("partial output checkpoint")
}

fn output_checkpoint_with_most_acknowledgements(state_dir: &Path) -> CheckpointSummary {
    transfer_checkpoints(state_dir)
        .into_iter()
        .max_by_key(|summary| summary.acknowledged_chunks)
        .expect("completed output checkpoint")
}

fn transfer_checkpoints(state_dir: &Path) -> Vec<CheckpointSummary> {
    let mut transfer_dirs = vec![state_dir.join("remote-transfers")];
    let workers_dir = state_dir.join("remote-workers");
    if let Ok(workers) = fs::read_dir(workers_dir) {
        for worker in workers.filter_map(Result::ok).take(MAX_TEST_REMOTE_WORKERS) {
            transfer_dirs.push(worker.path().join("state/remote-transfers"));
        }
    }
    let mut checkpoints = Vec::new();
    for transfer_dir in transfer_dirs {
        let Ok(entries) = fs::read_dir(transfer_dir) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if !entry.path().extension().is_some_and(|extension| extension == "json") {
                continue;
            }
            checkpoints.push(read_checkpoint_summary(&entry.path()));
        }
    }
    checkpoints
}

fn state_diagnostics_contain(state_dir: &Path, needle: &str) -> bool {
    assert!(!needle.is_empty());
    let mut pending = vec![state_dir.join(REMOTE_ATTEMPT_LOG_DIR)];
    let mut scanned_files = 0_usize;
    while let Some(path) = pending.pop() {
        if scanned_files >= MAX_DIAGNOSTIC_SCAN_FILES {
            return false;
        }
        if path.is_dir() {
            let Ok(entries) = fs::read_dir(path) else {
                continue;
            };
            pending.extend(entries.filter_map(Result::ok).map(|entry| entry.path()));
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        scanned_files = scanned_files.saturating_add(1);
        let retained = &bytes[..bytes.len().min(MAX_DIAGNOSTIC_SCAN_BYTES)];
        if let Ok(value) = serde_json::from_slice::<Value>(retained)
            && json_diagnostic_contains(&value, needle)
        {
            return true;
        }
    }
    assert!(scanned_files <= MAX_DIAGNOSTIC_SCAN_FILES);
    false
}

fn json_diagnostic_contains(root: &Value, needle: &str) -> bool {
    assert!(!needle.is_empty());
    let mut pending = vec![root];
    let mut visited = 0_usize;
    while let Some(value) = pending.pop() {
        if visited >= MAX_DIAGNOSTIC_JSON_NODES {
            return false;
        }
        visited = visited.saturating_add(1);
        match value {
            Value::String(text) => {
                if text.contains(needle) {
                    return true;
                }
            }
            Value::Array(values) => {
                let payload = values
                    .iter()
                    .map(|value| value.as_u64().and_then(|byte| u8::try_from(byte).ok()))
                    .collect::<Option<Vec<_>>>();
                if payload.is_some_and(|bytes| String::from_utf8_lossy(&bytes).contains(needle)) {
                    return true;
                }
                pending.extend(values);
            }
            Value::Object(fields) => pending.extend(fields.values()),
            Value::Null | Value::Bool(_) | Value::Number(_) => {}
        }
    }
    assert!(visited <= MAX_DIAGNOSTIC_JSON_NODES);
    false
}

fn read_checkpoint_summary(path: &Path) -> CheckpointSummary {
    let value: Value = serde_json::from_slice(&fs::read(path).expect("checkpoint bytes")).expect("checkpoint JSON");
    CheckpointSummary {
        path: path.to_path_buf(),
        acknowledged_chunks: value["checkpoint"]["acknowledged_chunk_digests"]
            .as_array()
            .expect("acknowledged chunk array")
            .len(),
        transferred_bytes: value["checkpoint"]["transferred_bytes"].as_u64().expect("transferred byte count"),
    }
}

fn remote_build_command(state_dir: &Path, store_dir: &Path, build_file: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin("mantle"));
    command.args([
        "--json",
        "--state-dir",
        state_dir.to_str().expect("UTF-8 state dir"),
        "--store",
        store_dir.to_str().expect("UTF-8 store dir"),
        "--store-prefix",
        STORE_PREFIX,
        "build",
        build_file.to_str().expect("UTF-8 build file"),
        "--no-substitute",
        "--builder",
        BUILDER_ID,
        "--ticket",
        &format!("{TICKET_ID}:{TICKET_SECRET}"),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
    ]);
    command
}

fn write_ticket_state(state_dir: &Path) {
    write_ticket_state_with_upload_limit(state_dir, MAX_UPLOAD_BYTES);
}

fn write_ticket_state_with_upload_limit(state_dir: &Path, max_upload_bytes: u64) {
    let ticket_dir = state_dir.join("remote-builders");
    fs::create_dir_all(&ticket_dir).expect("ticket state dir");
    let state = serde_json::json!({
        "tickets": {
            TICKET_ID: {
                "id": TICKET_ID,
                "display_name": "production transfer test",
                "secret": TICKET_SECRET,
                "created_unix_s": TEST_CREATED_UNIX_S,
                "expires_unix_s": TEST_EXPIRES_UNIX_S,
                "uses_remaining": TICKET_USES,
                "max_build_time_secs": MAX_BUILD_TIME_SECS,
                "max_upload_bytes": max_upload_bytes,
                "bound_client_endpoint": null,
                "revoked": false
            }
        }
    });
    fs::write(ticket_dir.join("tickets.json"), serde_json::to_vec_pretty(&state).expect("ticket state JSON"))
        .expect("ticket state");
}

fn write_fetch_derivation(path: &Path, output_source: &Path) {
    let bytes = fs::read(output_source).expect("output source bytes");
    let digest = sha2::Sha256::digest(&bytes);
    let hash = format!("sha256-{}", base64::engine::general_purpose::STANDARD.encode(digest));
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-output",
  builder = "builtin:fetchurl",
  system = 'x86_64-linux,
  args = [],
  outputs = ["out"],
  env = {{ url = "file://{}" }},
  inputs = ["{}"],
  fixed_output = {{ hash = "{}", algo = 'sha256, mode = 'flat }},
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
        output_source.display(),
        INPUT_STORE_PATH,
        hash,
    );
    fs::write(path, source).expect("production transfer derivation");
}

fn physical_store_path(store_dir: &Path, logical_path: &str) -> PathBuf {
    let name = logical_path.rsplit('/').next().expect("logical store basename");
    store_dir.join(name)
}

fn patterned_bytes(size: usize) -> Vec<u8> {
    (0..size).map(|index| u8::try_from(index % PATTERN_MODULUS).expect("pattern byte")).collect()
}
