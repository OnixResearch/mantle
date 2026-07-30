use std::collections::BTreeMap;
use std::fs;
use std::os::fd::AsRawFd as _;
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use base64::Engine;
use serde_json::Value;
use sha2::Digest;

// r[verify examples.resumable_remote_transfer_workflow]

const STORE_PREFIX: &str = "/mantle/store";
const INPUT_STORE_PATH: &str = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-production-input";
const INPUT_BYTES: usize = 196_608;
const OUTPUT_BYTES: usize = 262_144;
const PRODUCTION_SCALE_OUTPUT_BYTES: usize = 8_388_608;
const MIN_PRODUCTION_SCALE_OUTPUT_CHUNKS: usize = 100;
const PATTERN_MODULUS: usize = 251;
const INTERRUPT_AFTER_CHUNKS: &str = "1";
const TICKET_ID: &str = "production-ticket";
const TICKET_SECRET: &str = "QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI";
const TICKET_KEY: &str = "ticket-key-1:QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE";
const TICKET_VERIFIER_DOMAIN: &[u8] = b"mantle-remote-ticket-verifier-v2\0";
const RESULT_SIGNING_KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
const BUILDER_ID: &str = "production-builder";
const MAX_BUILD_TIME_SECS: u64 = 600;
const MAX_UPLOAD_BYTES: u64 = 1_073_741_824;
const REJECTING_UPLOAD_BYTES: u64 = 1;
const TICKET_USES: u32 = 3;
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
const REMOTE_FAILURE_DEBUG_BUNDLE_DIR: &str = "remote-failure-debug/bundles";
const EXPECTED_FAILURE_DEBUG_BUNDLES: usize = 1;
const MAX_DIAGNOSTIC_SCAN_FILES: usize = 4_096;
const MAX_DIAGNOSTIC_SCAN_BYTES: usize = 1_048_576;
const MAX_DIAGNOSTIC_JSON_NODES: usize = 65_536;
const PRIORITY_LIFECYCLE_PREFIX_COUNT: usize = 5;
const WRONG_BUILDER_KEY: &str = "wrong-builder-key";
const CAPTURED_TRACE: &str = "bounded-trace-event";
const DISALLOWED_TRACE: &str = "DO_NOT_CAPTURE";
const EXPECTED_WORKER_AND_COORDINATOR_BUNDLES: usize = 2;
const BLAKE3_HEX_CHARS: usize = 64;
const REJECTING_CAPTURE_FILE_BYTES: u64 = 4;
const ACCEPTING_CAPTURE_FILE_BYTES: u64 = 4_096;
const TEST_REAL_BWRAP_ENV: &str = "MANTLE_TEST_REAL_BWRAP";
const GALLERY_PROJECT_RELATIVE_PATH: &str = "examples/projects/remote-build-loopback";
const GALLERY_RESUMABLE_SELECTOR: &str = ".#resumable-payload";
const GALLERY_PAYLOAD_BYTES: usize = 262_144;
const REMOTE_TRANSFER_RECEIVER_DIR: &str = "remote-transfer-receiver";
const REMOTE_TRANSFER_CHUNK_DIR: &str = "chunks";
const ACKNOWLEDGED_CHUNK_MISSING_REASON: &str = "acknowledged-chunk-missing";
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;
const REMOTE_TICKET_INPUT_FD: i32 = 9;
const REMOTE_TICKET_INPUT_FILENAME: &str = "remote-ticket.secret";

#[test]
fn production_stdio_resumes_missing_chunks_and_imports_output() {
    if !cfg!(debug_assertions) {
        eprintln!("SKIP: the deterministic transfer-interruption seam is disabled in release binaries");
        return;
    }
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
    let failure_bundles = failure_debug_bundle_dirs(&state_dir);
    assert_eq!(failure_bundles.len(), EXPECTED_FAILURE_DEBUG_BUNDLES);
    let failure_bundle = &failure_bundles[0];
    let manifest: Value = serde_json::from_slice(&fs::read(failure_bundle.join("manifest.json")).unwrap()).unwrap();
    let bundle_digest = manifest["bundle_blake3"].as_str().expect("bundle digest");
    assert_eq!(manifest["failure_phase"], "input-transfer");
    assert_eq!(manifest["capture_outcome_code"], "metadata-only");
    assert_eq!(manifest["cleanup_status_code"], "cleanup-not-observed");
    assert!(manifest["immutable_log"].is_object());
    let bundle_before_inspect = snapshot_file_tree(failure_bundle);
    let inspect = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "debug", "inspect", bundle_digest])
        .output()
        .expect("inspect failure debug bundle in clean process");
    assert!(inspect.status.success(), "inspect stderr={}", String::from_utf8_lossy(&inspect.stderr));
    let inspect_json: Value = serde_json::from_slice(&inspect.stdout).expect("inspect JSON");
    assert_eq!(inspect_json["bundle_blake3"], bundle_digest);
    assert_eq!(inspect_json["captured_artifact_count"], 0);
    let replay_plan = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "debug", "replay-plan", bundle_digest])
        .output()
        .expect("plan failure debug replay in clean process");
    assert!(replay_plan.status.success(), "replay-plan stderr={}", String::from_utf8_lossy(&replay_plan.stderr));
    let replay_json: Value = serde_json::from_slice(&replay_plan.stdout).expect("replay plan JSON");
    assert_eq!(replay_json["executable"], false);
    assert!(replay_json["blockers"].as_array().unwrap().iter().any(|blocker| blocker == "replay-policy-denied"));
    assert_eq!(snapshot_file_tree(failure_bundle), bundle_before_inspect);
    let status = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "status"])
        .output()
        .expect("render remote failure status");
    assert!(status.status.success(), "status stderr={}", String::from_utf8_lossy(&status.stderr));
    let status_json: Value = serde_json::from_slice(&status.stdout).expect("remote status JSON");
    let failure_debug = &status_json["active_jobs"][0]["failure_debug"];
    assert_eq!(failure_debug["bundle_ref"], format!("remote-failure-debug:{bundle_digest}"));
    assert_eq!(failure_debug["capture_outcome_code"], "metadata-only");
    let status_text = String::from_utf8(status.stdout).unwrap();
    assert!(!status_text.contains(TICKET_SECRET));
    assert!(!status_text.contains(root.path().to_str().unwrap()));
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
    if !cfg!(debug_assertions) {
        eprintln!("SKIP: the deterministic transfer-interruption seam is disabled in release binaries");
        return;
    }
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
fn gallery_resumable_remote_transfer_resumes_verified_chunks_and_admits_once() {
    if !cfg!(debug_assertions) {
        eprintln!("SKIP: the deterministic transfer-interruption seam is disabled in release binaries");
        return;
    }
    let root = tempfile::tempdir().expect("gallery resumable transfer tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    fs::create_dir_all(&state_dir).expect("gallery state dir");
    fs::create_dir_all(&store_dir).expect("gallery store dir");
    write_ticket_state(&state_dir);

    let interrupted = remote_gallery_build_command(&state_dir, &store_dir)
        .env(INTERRUPT_ENV, INTERRUPT_AFTER_CHUNKS)
        .output()
        .expect("interrupt gallery production transfer");
    assert!(!interrupted.status.success(), "first gallery run must interrupt after a durable chunk");
    assert!(
        String::from_utf8_lossy(&interrupted.stderr)
            .contains("remote-production-transfer-interrupted-after-checkpoint")
    );
    assert_eq!(fs::read_dir(&store_dir).unwrap().count(), 0);
    assert!(!state_diagnostics_contain(&state_dir, "output-admitted"));
    let partial = partial_output_checkpoint(&state_dir);
    assert_eq!(partial.acknowledged_chunks, 1);
    assert!(partial.transferred_bytes > 0);
    assert_eq!(partial.manifest_digest_blake3.len(), BLAKE3_HEX_CHARS);

    let resumed = remote_gallery_build_command(&state_dir, &store_dir)
        .output()
        .expect("resume gallery production transfer");
    assert!(resumed.status.success(), "gallery resume stderr={}", String::from_utf8_lossy(&resumed.stderr));
    let report: Value = serde_json::from_slice(&resumed.stdout).expect("gallery production JSON report");
    let substitution = &report["outcomes"][0]["outputs"][0]["substitution"];
    assert_eq!(substitution["mode"], "streaming");
    assert!(substitution["reused_bytes"].as_u64().unwrap_or(0) >= partial.transferred_bytes);
    assert_production_lifecycle_sequence(&report, true);

    let output_path = PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("gallery output path"));
    assert_eq!(
        fs::read_to_string(output_path.join("payload.txt")).unwrap(),
        "resumable remote transfer gallery payload\n"
    );
    let payload = fs::read(output_path.join("payload.bin")).expect("gallery payload bytes");
    assert_eq!(payload.len(), GALLERY_PAYLOAD_BYTES);
    assert_eq!(blake3::hash(&payload), blake3::hash(&vec![0_u8; GALLERY_PAYLOAD_BYTES]));
    assert_eq!(fs::read_dir(&store_dir).unwrap().count(), 1);

    let completed = read_checkpoint_summary(&partial.path);
    assert_eq!(completed.session_id, partial.session_id);
    assert_eq!(completed.manifest_digest_blake3, partial.manifest_digest_blake3);
    assert!(completed.acknowledged_chunks > partial.acknowledged_chunks);
    assert!(completed.transferred_bytes > partial.transferred_bytes);
}

#[test]
fn gallery_resumable_remote_transfer_rejects_tampered_acknowledged_content() {
    if !cfg!(debug_assertions) {
        eprintln!("SKIP: the deterministic transfer-interruption seam is disabled in release binaries");
        return;
    }
    let root = tempfile::tempdir().expect("gallery tampered transfer tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    fs::create_dir_all(&state_dir).expect("gallery state dir");
    fs::create_dir_all(&store_dir).expect("gallery store dir");
    write_ticket_state(&state_dir);

    let interrupted = remote_gallery_build_command(&state_dir, &store_dir)
        .env(INTERRUPT_ENV, INTERRUPT_AFTER_CHUNKS)
        .output()
        .expect("interrupt gallery transfer before tamper");
    assert!(!interrupted.status.success());
    let partial = partial_output_checkpoint(&state_dir);
    let acknowledged_digest = partial.acknowledged_chunk_digests.first().expect("acknowledged chunk digest");
    let receiver_chunk = state_dir
        .join(REMOTE_TRANSFER_RECEIVER_DIR)
        .join(&partial.session_id)
        .join(REMOTE_TRANSFER_CHUNK_DIR)
        .join(acknowledged_digest);
    assert!(receiver_chunk.is_file());
    fs::write(&receiver_chunk, b"tampered acknowledged gallery chunk").expect("tamper acknowledged chunk");

    let rejected = remote_gallery_build_command(&state_dir, &store_dir)
        .output()
        .expect("retry tampered gallery transfer");
    assert!(!rejected.status.success(), "tampered acknowledged content must block resume");
    assert!(String::from_utf8_lossy(&rejected.stderr).contains(ACKNOWLEDGED_CHUNK_MISSING_REASON));
    assert_eq!(fs::read_dir(&store_dir).unwrap().count(), 0);
    assert!(!state_diagnostics_contain(&state_dir, "output-admitted"));
}

#[test]
fn failed_remote_sandbox_captures_allowlisted_artifact_before_cleanup_without_changing_failure_truth() {
    let root = tempfile::Builder::new()
        .prefix("failure-debug-capture")
        .tempdir()
        .expect("failure debug capture tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let build_file = root.path().join("remote-failure.ncl");
    let config = root.path().join("failure-debug.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    write_ticket_state(&state_dir);
    write_failing_capture_derivation(&build_file);
    write_capture_debug_config(&config);

    let failed = remote_build_command(&state_dir, &store_dir, &build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .env("CRUNCH_NO_FUSE", "1")
        .output()
        .expect("run captured remote sandbox failure");
    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("remote-failure-debug:"), "missing worker bundle ref: {stderr}");
    assert!(stderr.contains("remote-local-executor-build"), "missing original execution failure: {stderr}");
    let bundles = recursive_failure_debug_bundle_dirs(&state_dir);
    assert_eq!(bundles.len(), EXPECTED_WORKER_AND_COORDINATOR_BUNDLES);
    let capture_outcomes = bundles
        .iter()
        .map(|bundle| {
            let manifest: Value = serde_json::from_slice(&fs::read(bundle.join("manifest.json")).unwrap()).unwrap();
            manifest["capture_outcome_code"].as_str().unwrap().to_string()
        })
        .collect::<Vec<_>>();
    let captured_bundle = bundles
        .iter()
        .find(|bundle| {
            let manifest: Value = serde_json::from_slice(&fs::read(bundle.join("manifest.json")).unwrap()).unwrap();
            manifest["capture_outcome_code"] == "captured"
        })
        .unwrap_or_else(|| panic!("worker capture bundle missing; outcomes={capture_outcomes:?}; stderr={stderr}"));
    let manifest: Value = serde_json::from_slice(&fs::read(captured_bundle.join("manifest.json")).unwrap()).unwrap();
    let worker_bundle_digest = manifest["bundle_blake3"].as_str().unwrap();
    assert_eq!(worker_bundle_digest.len(), BLAKE3_HEX_CHARS);
    assert_eq!(manifest["failure_phase"], "execution");
    assert_eq!(manifest["cleanup_status_code"], "cleanup-attempted-after-capture");
    let capture_manifest_digest = manifest["captured_artifact_manifest_ref"]["digest_blake3"].as_str().unwrap();
    let capture_manifest_path = captured_bundle.join("objects").join(format!("{capture_manifest_digest}.json"));
    let capture_manifest: Value = serde_json::from_slice(&fs::read(capture_manifest_path).unwrap()).unwrap();
    let artifacts = capture_manifest["payload"]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0]["relative_path"], "build/trace.json");
    let object_digest = artifacts[0]["object_blake3"].as_str().unwrap();
    let captured_bytes = fs::read(captured_bundle.join("capture-cas").join(format!("{object_digest}.bin"))).unwrap();
    assert_eq!(captured_bytes, CAPTURED_TRACE.as_bytes());
    let all_bundle_bytes = snapshot_file_tree(captured_bundle)
        .keys()
        .flat_map(|relative| fs::read(captured_bundle.join(relative)).unwrap())
        .collect::<Vec<_>>();
    assert!(!String::from_utf8_lossy(&all_bundle_bytes).contains(DISALLOWED_TRACE));
    let inspect = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "debug", "inspect", worker_bundle_digest])
        .output()
        .expect("inspect portable worker capture bundle after cleanup");
    assert!(inspect.status.success(), "worker inspect stderr={}", String::from_utf8_lossy(&inspect.stderr));
    let inspect_json: Value = serde_json::from_slice(&inspect.stdout).unwrap();
    assert_eq!(inspect_json["captured_artifact_count"], 1);
    assert!(!String::from_utf8(inspect.stdout).unwrap().contains(root.path().to_str().unwrap()));
    let status = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&state_dir)
        .args(["remote", "status"])
        .output()
        .expect("render worker capture status");
    assert!(status.status.success());
    let status_json: Value = serde_json::from_slice(&status.stdout).unwrap();
    let debug_status = &status_json["recent_failures"][0]["failure_debug"];
    assert_eq!(debug_status["capture_outcome_code"], "captured");
    assert_eq!(debug_status["worker_bundle_ref"], format!("remote-failure-debug:{worker_bundle_digest}"));
    assert!(failure_workspace_directories(&state_dir).is_empty());
    assert!(fs::read_dir(&store_dir).unwrap().next().is_none());
}

#[test]
fn oversized_remote_sandbox_artifact_is_rejected_without_rewriting_execution_failure() {
    let root = tempfile::Builder::new()
        .prefix("failure-debug-oversized")
        .tempdir()
        .expect("oversized failure debug tempdir");
    let state_dir = root.path().join("state");
    let store_dir = root.path().join("store");
    let build_file = root.path().join("remote-failure.ncl");
    let config = root.path().join("failure-debug.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    write_ticket_state(&state_dir);
    write_failing_capture_derivation(&build_file);
    write_capture_debug_config_with_file_bytes(&config, REJECTING_CAPTURE_FILE_BYTES);

    let failed = remote_build_command(&state_dir, &store_dir, &build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .env("CRUNCH_NO_FUSE", "1")
        .output()
        .expect("run oversized captured remote sandbox failure");
    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("remote-local-executor-build"));
    assert!(stderr.contains("remote-failure-debug:"), "stderr={stderr}");
    let bundles = recursive_failure_debug_bundle_dirs(&state_dir);
    assert_eq!(bundles.len(), EXPECTED_WORKER_AND_COORDINATOR_BUNDLES);
    let rejected_bundle = bundles
        .iter()
        .find(|bundle| {
            let manifest: Value = serde_json::from_slice(&fs::read(bundle.join("manifest.json")).unwrap()).unwrap();
            manifest["capture_outcome_code"] == "capture-rejected"
        })
        .expect("oversized capture rejection bundle");
    assert!(!rejected_bundle.join("capture-cas").read_dir().unwrap().any(|entry| entry.is_ok()));
    assert!(failure_workspace_directories(&state_dir).is_empty());
    assert!(fs::read_dir(&store_dir).unwrap().next().is_none());
}

#[test]
fn failed_output_admission_replays_under_new_fence_without_rewriting_original_bundle() {
    let root = tempfile::Builder::new()
        .prefix("failure-debug-replay")
        .tempdir()
        .expect("failure debug replay tempdir");
    let fixture = setup_replay_fixture(root.path());
    let config = root.path().join("failure-debug.ncl");
    write_failure_debug_config(&config, true);

    let failed = remote_build_command(&fixture.state_dir, &fixture.store_dir, &fixture.build_file)
        .arg("--remote-observability-config")
        .arg(&config)
        .arg("--trusted-builder-key")
        .arg(WRONG_BUILDER_KEY)
        .env("CRUNCH_NO_FUSE", "1")
        .output()
        .expect("run output-admission failure");
    assert!(!failed.status.success());
    let failed_stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(failed_stderr.contains("untrusted-output-key"), "unexpected failure stderr: {failed_stderr}");
    let bundles = failure_debug_bundle_dirs(&fixture.state_dir);
    assert_eq!(bundles.len(), EXPECTED_FAILURE_DEBUG_BUNDLES);
    let bundle_dir = &bundles[0];
    let manifest_bytes = fs::read(bundle_dir.join("manifest.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let bundle_digest = manifest["bundle_blake3"].as_str().unwrap();
    let original_job_id = manifest["original_job_id"].as_str().unwrap();
    let original_attempt_id = manifest["original_attempt_id"].as_str().unwrap();
    let original_fence = manifest["original_fence_generation"].as_u64().unwrap();
    assert_eq!(manifest["failure_phase"], "execution");

    let rejected_replay = remote_failure_replay_command(&fixture.state_dir, &fixture.store_dir, bundle_digest)
        .arg("--trusted-builder-key")
        .arg(WRONG_BUILDER_KEY)
        .output()
        .expect("execute rejected remote failure replay");
    assert!(!rejected_replay.status.success());
    let rejected_replay_stderr = String::from_utf8_lossy(&rejected_replay.stderr);
    assert!(
        rejected_replay_stderr.contains("untrusted-output-key"),
        "rejected replay stderr={rejected_replay_stderr}"
    );
    let coordinator: Value = serde_json::from_slice(
        &fs::read(fixture.state_dir.join("remote-coordinator-state.json")).expect("replay coordinator state"),
    )
    .expect("replay coordinator JSON");
    assert_eq!(coordinator["live_output_claims"].as_object().unwrap().len(), 0);
    assert_eq!(fs::read(bundle_dir.join("manifest.json")).unwrap(), manifest_bytes);

    let replay = remote_failure_replay_command(&fixture.state_dir, &fixture.store_dir, bundle_digest)
        .output()
        .expect("execute remote failure replay");
    assert!(replay.status.success(), "replay stderr={}", String::from_utf8_lossy(&replay.stderr));
    let report: Value = serde_json::from_slice(&replay.stdout).expect("replay report JSON");
    let replay_identity = report["replay_attempt_identity"].as_str().unwrap();
    let replay_parts = replay_identity.split(':').collect::<Vec<_>>();
    assert_eq!(replay_parts.len(), 3);
    assert_ne!(replay_parts[0], original_job_id);
    assert_ne!(replay_parts[1], original_attempt_id);
    assert!(replay_parts[2].parse::<u64>().unwrap() > 0);
    assert!(original_fence > 0);
    assert_eq!(report["comparison"]["class"], "diverged");
    assert_eq!(report["original_result_immutable"], true);
    assert_eq!(report["ordinary_output_admission_applied"], true);
    assert_eq!(report["admitted_output_count"], 1);
    assert_eq!(fs::read(bundle_dir.join("manifest.json")).unwrap(), manifest_bytes);
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

fn setup_replay_fixture(root: &Path) -> ProductionFixture {
    let state_dir = root.join("state");
    let store_dir = root.join("store");
    let build_file = root.join("remote-replay.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    write_ticket_state(&state_dir);
    write_replayable_derivation(&build_file);
    ProductionFixture {
        state_dir,
        store_dir,
        build_file,
    }
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

fn write_capture_debug_config(path: &Path) {
    write_capture_debug_config_with_file_bytes(path, ACCEPTING_CAPTURE_FILE_BYTES);
}

fn write_capture_debug_config_with_file_bytes(path: &Path, file_bytes_max: u64) {
    let contract = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/remote-builders.ncl");
    let source = format!(
        "let remote = import \"{}\" in {{ pools = [], failure_debug = ({{ capture = {{ enabled = true, allowed_relative_paths = [\"build/trace.json\"], sensitivity = 'restricted-diagnostic, file_count_max = 1, total_bytes_max = {file_bytes_max}, file_bytes_max = {file_bytes_max}, depth_max = 4, failure_mode = 'diagnostic-only }} }} | remote.RemoteFailureDebug) }}",
        contract.display(),
    );
    fs::write(path, source).expect("capture debug config");
}

fn write_failure_debug_config(path: &Path, replay_enabled: bool) {
    let contract = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/remote-builders.ncl");
    let source = format!(
        "let remote = import \"{}\" in {{ pools = [], failure_debug = ({{ replay_enabled = {replay_enabled} }} | remote.RemoteFailureDebug) }}",
        contract.display(),
    );
    fs::write(path, source).expect("failure debug config");
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
    session_id: String,
    manifest_digest_blake3: String,
    acknowledged_chunk_digests: Vec<String>,
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
            if entry.path().extension().is_none_or(|extension| extension != "json") {
                continue;
            }
            checkpoints.push(read_checkpoint_summary(&entry.path()));
        }
    }
    checkpoints
}

fn recursive_failure_debug_bundle_dirs(state_dir: &Path) -> Vec<PathBuf> {
    let mut pending = vec![state_dir.to_path_buf()];
    let mut bundles = Vec::new();
    let mut visited = 0_usize;
    while let Some(path) = pending.pop() {
        assert!(visited < MAX_DIAGNOSTIC_SCAN_FILES, "failure debug bundle scan limit exceeded");
        visited = visited.saturating_add(1);
        if path.ends_with(REMOTE_FAILURE_DEBUG_BUNDLE_DIR) {
            bundles.extend(
                fs::read_dir(&path)
                    .unwrap()
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|entry| entry.is_dir()),
            );
            continue;
        }
        if path.is_dir() {
            pending.extend(fs::read_dir(path).unwrap().filter_map(Result::ok).map(|entry| entry.path()));
        }
    }
    bundles.sort();
    bundles
}

fn failure_workspace_directories(state_dir: &Path) -> Vec<PathBuf> {
    let mut pending = vec![state_dir.to_path_buf()];
    let mut workspaces = Vec::new();
    let mut visited = 0_usize;
    while let Some(path) = pending.pop() {
        assert!(visited < MAX_DIAGNOSTIC_SCAN_FILES, "failure workspace scan limit exceeded");
        visited = visited.saturating_add(1);
        if path.file_name().is_some_and(|name| name == "remote-failure-workspaces") {
            workspaces.extend(
                fs::read_dir(&path)
                    .unwrap()
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|entry| entry.is_dir()),
            );
            continue;
        }
        if path.is_dir() {
            pending.extend(fs::read_dir(path).unwrap().filter_map(Result::ok).map(|entry| entry.path()));
        }
    }
    workspaces
}

fn failure_debug_bundle_dirs(state_dir: &Path) -> Vec<PathBuf> {
    let root = state_dir.join(REMOTE_FAILURE_DEBUG_BUNDLE_DIR);
    let mut bundles = fs::read_dir(root)
        .expect("remote failure debug bundle root")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    bundles.sort();
    bundles
}

fn snapshot_file_tree(root: &Path) -> BTreeMap<PathBuf, String> {
    let mut pending = vec![root.to_path_buf()];
    let mut snapshot = BTreeMap::new();
    let mut visited = 0_usize;
    while let Some(path) = pending.pop() {
        assert!(visited < MAX_DIAGNOSTIC_SCAN_FILES, "bundle snapshot file limit exceeded");
        visited = visited.saturating_add(1);
        if path.is_dir() {
            pending.extend(fs::read_dir(&path).unwrap().filter_map(Result::ok).map(|entry| entry.path()));
            continue;
        }
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        let bytes = fs::read(&path).expect("bundle snapshot file reads");
        snapshot.insert(relative, blake3::hash(&bytes).to_hex().to_string());
    }
    assert!(!snapshot.is_empty());
    snapshot
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
    let acknowledged_chunk_digests = value["checkpoint"]["acknowledged_chunk_digests"]
        .as_array()
        .expect("acknowledged chunk array")
        .iter()
        .map(|digest| digest.as_str().expect("acknowledged chunk digest").to_string())
        .collect::<Vec<_>>();
    CheckpointSummary {
        path: path.to_path_buf(),
        session_id: value["checkpoint"]["scope"]["session_id"].as_str().expect("checkpoint session id").to_string(),
        manifest_digest_blake3: value["checkpoint"]["scope"]["manifest_digest_blake3"]
            .as_str()
            .expect("checkpoint manifest digest")
            .to_string(),
        acknowledged_chunks: acknowledged_chunk_digests.len(),
        acknowledged_chunk_digests,
        transferred_bytes: value["checkpoint"]["transferred_bytes"].as_u64().expect("transferred byte count"),
    }
}

fn remote_failure_replay_command(state_dir: &Path, store_dir: &Path, bundle_digest: &str) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mantle"));
    configure_test_sandbox_command(&mut command);
    command.args(["--json", "--state-dir"]).arg(state_dir).arg("--store").arg(store_dir).args([
        "--store-prefix",
        STORE_PREFIX,
        "remote",
        "debug",
        "replay",
        bundle_digest,
        "--builder",
        BUILDER_ID,
        "--ticket-fd",
        &REMOTE_TICKET_INPUT_FD.to_string(),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
        "--remote-secret-manifest",
        secret_manifest().to_str().expect("UTF-8 secret manifest"),
    ]);
    command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
    configure_ticket_input_fd(&mut command, state_dir);
    command
}

fn remote_build_command(state_dir: &Path, store_dir: &Path, build_file: &Path) -> Command {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin("mantle"));
    configure_test_sandbox_command(&mut command);
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
        "--ticket-fd",
        &REMOTE_TICKET_INPUT_FD.to_string(),
        "--remote-build-time-secs",
        &MAX_BUILD_TIME_SECS.to_string(),
        "--remote-secret-manifest",
        secret_manifest().to_str().expect("UTF-8 secret manifest"),
    ]);
    command.env("CREDENTIALS_DIRECTORY", state_dir.join("credentials"));
    configure_ticket_input_fd(&mut command, state_dir);
    command
}

fn remote_gallery_build_command(state_dir: &Path, store_dir: &Path) -> Command {
    let project_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(GALLERY_PROJECT_RELATIVE_PATH);
    assert!(project_dir.join("mantle-project.ncl").is_file());
    assert!(project_dir.join("README.md").is_file());
    let mut command = remote_build_command(state_dir, store_dir, Path::new(GALLERY_RESUMABLE_SELECTOR));
    command.current_dir(project_dir);
    command
}

fn configure_test_sandbox_command(command: &mut Command) {
    command.env("CRUNCH_NO_FUSE", "1");
    let Some(bwrap_path) = std::env::var_os(TEST_REAL_BWRAP_ENV) else {
        return;
    };
    assert!(Path::new(&bwrap_path).is_file(), "declared test bwrap must be a file");
    command.env("SNIX_BUILD_BWRAP", bwrap_path);
}

fn write_ticket_state(state_dir: &Path) {
    write_ticket_state_with_upload_limit(state_dir, MAX_UPLOAD_BYTES);
}

fn write_ticket_state_with_upload_limit(state_dir: &Path, max_upload_bytes: u64) {
    let ticket_dir = state_dir.join("remote-builders");
    fs::create_dir_all(&ticket_dir).expect("ticket state dir");
    fs::set_permissions(&ticket_dir, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).expect("private ticket dir");
    let state = serde_json::json!({
        "schema_version": 2,
        "next_ticket_sequence": 1,
        "tickets": {
            TICKET_ID: {
                "id": TICKET_ID,
                "display_name": "production transfer test",
                "verifier_key_id": "ticket-key-1",
                "verifier": ticket_verifier(),
                "created_unix_s": TEST_CREATED_UNIX_S,
                "expires_unix_s": TEST_EXPIRES_UNIX_S,
                "uses_remaining": TICKET_USES,
                "max_build_time_secs": MAX_BUILD_TIME_SECS,
                "max_upload_bytes": max_upload_bytes,
                "bound_client_endpoint": null,
                "revoked": false
            }
        },
        "invalidated_legacy_ticket_ids": []
    });
    let state_path = ticket_dir.join("tickets.json");
    fs::write(&state_path, serde_json::to_vec_pretty(&state).expect("ticket state JSON")).expect("ticket state");
    fs::set_permissions(state_path, fs::Permissions::from_mode(PRIVATE_FILE_MODE)).expect("private ticket file");
    let credential_path = state_dir.join(REMOTE_TICKET_INPUT_FILENAME);
    fs::write(&credential_path, format!("{TICKET_ID}:{TICKET_SECRET}\n")).expect("ticket input file");
    fs::set_permissions(credential_path, fs::Permissions::from_mode(PRIVATE_FILE_MODE))
        .expect("private ticket input file");
    write_service_credentials(state_dir);
}

fn configure_ticket_input_fd(command: &mut Command, state_dir: &Path) {
    let ticket_file = fs::File::open(state_dir.join(REMOTE_TICKET_INPUT_FILENAME)).expect("ticket input file");
    unsafe {
        command.pre_exec(move || {
            if libc::dup2(ticket_file.as_raw_fd(), REMOTE_TICKET_INPUT_FD) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

fn ticket_verifier() -> String {
    let mut input = Vec::from(TICKET_VERIFIER_DOMAIN);
    input.extend_from_slice(&[0x42_u8; 32]);
    let key = [0x41_u8; 32];
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(blake3::keyed_hash(&key, &input).as_bytes())
}

fn write_service_credentials(state_dir: &Path) {
    let credentials_dir = state_dir.join("credentials");
    fs::create_dir_all(&credentials_dir).expect("credentials dir");
    fs::write(credentials_dir.join("TICKET_VERIFIER_KEY"), TICKET_KEY).expect("ticket key credential");
    fs::write(credentials_dir.join("RESULT_SIGNING_KEY"), RESULT_SIGNING_KEY).expect("signing key credential");
}

fn secret_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml")
}

fn write_replayable_derivation(path: &Path) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-replay",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "printf replay-ok > $out"],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
    );
    fs::write(path, source).expect("replayable derivation");
}

fn write_failing_capture_derivation(path: &Path) {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let source = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-failure",
  builder = "/bin/sh",
  system = 'x86_64-linux,
  args = ["-c", "printf '{}' > trace.json; printf 'DO_NOT_' > secret.txt; printf 'CAPTURE' >> secret.txt; exit 7"],
  outputs = ["out"],
  env = {{}},
  inputs = [],
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        lib.display(),
        CAPTURED_TRACE,
    );
    fs::write(path, source).expect("failing capture derivation");
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
