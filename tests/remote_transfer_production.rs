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
    assert!(substitution["reused_bytes"].as_u64().unwrap_or(0) >= partial.transferred_bytes);
    let output_path = PathBuf::from(
        report["outcomes"][0]["outputs"][0]["path"]
            .as_str()
            .expect("imported output path"),
    );
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
    fs::write(
        physical_store_path(&store_dir, INPUT_STORE_PATH),
        patterned_bytes(INPUT_BYTES),
    )
    .expect("input source");
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
    let imported_path = PathBuf::from(
        report["outcomes"][0]["outputs"][0]["path"]
            .as_str()
            .expect("imported output path"),
    );
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
    fs::write(
        physical_store_path(&store_dir, INPUT_STORE_PATH),
        patterned_bytes(INPUT_BYTES),
    )
    .unwrap();
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

fn read_checkpoint_summary(path: &Path) -> CheckpointSummary {
    let value: Value = serde_json::from_slice(&fs::read(path).expect("checkpoint bytes"))
        .expect("checkpoint JSON");
    CheckpointSummary {
        path: path.to_path_buf(),
        acknowledged_chunks: value["checkpoint"]["acknowledged_chunk_digests"]
            .as_array()
            .expect("acknowledged chunk array")
            .len(),
        transferred_bytes: value["checkpoint"]["transferred_bytes"]
            .as_u64()
            .expect("transferred byte count"),
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
    fs::write(
        ticket_dir.join("tickets.json"),
        serde_json::to_vec_pretty(&state).expect("ticket state JSON"),
    )
    .expect("ticket state");
}

fn write_fetch_derivation(path: &Path, output_source: &Path) {
    let bytes = fs::read(output_source).expect("output source bytes");
    let digest = sha2::Sha256::digest(&bytes);
    let hash = format!(
        "sha256-{}",
        base64::engine::general_purpose::STANDARD.encode(digest)
    );
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
    (0..size)
        .map(|index| u8::try_from(index % PATTERN_MODULUS).expect("pattern byte"))
        .collect()
}
