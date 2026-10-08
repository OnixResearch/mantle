use std::fs;
use std::io::BufRead as _;
use std::io::BufReader;
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::Child;
use std::process::Command as ProcessCommand;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use assert_cmd::Command;
use base64::Engine as _;
use serde_json::Value;

const FRAME_HEADER_BYTES: usize = 4;
const PROTOCOL_ALPN: &str = "mantle-remote-build/1";
const PROTOCOL_VERSION: u32 = 1;
const TEST_NOW_UNIX_S: u64 = 1;
const TEST_TICKET_TTL_SECS: u64 = 3_600;
const TEST_CLOCK_ROLLBACK_SECS: u64 = 60;
const TEST_TICKET_ID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TEST_BUILD_TIME_LIMIT_SECS: u64 = 60;
const TEST_TICKET_MAX_BUILD_TIME_SECS: u64 = 600;
const TEST_UPLOAD_BYTES: u64 = 10;
const TEST_MAX_UPLOAD_BYTES: u64 = 1_000;
const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const TICKET_TOKEN: &str = "QkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkI";
const TICKET_KEY: &str = "ticket-key-1:QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE";
const RESULT_SIGNING_KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
const RESULT_SIGNING_KEY_ID: &str = "cache.example.com-1";
const TICKET_VERIFIER_DOMAIN: &[u8] = b"mantle-remote-ticket-verifier-v2\0";
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const PRIVATE_FILE_MODE: u32 = 0o600;

#[test]
fn live_subscriber_pipe_flushes_snapshot_and_later_real_goal_before_daemon_stops() {
    struct Reap(Child);

    impl Drop for Reap {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let temp = tempfile::tempdir().expect("live CLI tempdir");
    let socket = temp.path().join("live.sock");
    let binary = env!("CARGO_BIN_EXE_mantle");
    let mut daemon = Reap(
        ProcessCommand::new(binary)
            .args(["remote", "live", "serve", "--socket"])
            .arg(&socket)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn real live daemon"),
    );
    let listening = (0..50).any(|_| {
        if UnixStream::connect(&socket).is_ok() {
            return true;
        }
        assert!(daemon.0.try_wait().expect("poll daemon").is_none(), "live daemon exited before listening");
        thread::sleep(Duration::from_millis(100));
        false
    });
    assert!(listening, "live daemon never accepted connections");

    let mut subscriber = Reap(
        ProcessCommand::new(binary)
            .args(["remote", "live", "subscribe", "--socket"])
            .arg(&socket)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn real piped live subscriber"),
    );
    let stdout = subscriber.0.stdout.take().expect("subscriber stdout pipe");
    let (events_tx, events_rx) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if events_tx.send(line).is_err() {
                break;
            }
        }
    });
    let next_event = |timeout| -> Value {
        let line = events_rx
            .recv_timeout(timeout)
            .expect("subscriber event arrived before daemon closed")
            .expect("read subscriber stdout line");
        serde_json::from_str(&line).expect("valid live protocol event")
    };
    assert_eq!(next_event(Duration::from_secs(5))["op"], "snapshot-start");
    assert_eq!(next_event(Duration::from_secs(5))["op"], "snapshot-end");
    assert!(daemon.0.try_wait().expect("poll daemon after snapshot").is_none());
    assert!(subscriber.0.try_wait().expect("poll subscriber after snapshot").is_none());

    let store = temp.path().join("store");
    let state = temp.path().join("state");
    let config = temp.path().join("config");
    for path in [&store, &state, &config] {
        fs::create_dir(path).expect("create private build path");
        fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).expect("private build path");
    }
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/simple.ncl");
    let mut build = Reap(
        ProcessCommand::new(binary)
            .arg("--store")
            .arg(&store)
            .arg("--state-dir")
            .arg(&state)
            .args(["build", "--no-substitute", "--evaluation-stream"])
            .arg(&source)
            .env("CRUNCH_CONFIG_DIR", &config)
            .env("MANTLE_LIVE_STATE_SOCKET", &socket)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn actual observed build"),
    );
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut observed_goal = false;
    for _ in 0..32 {
        let event = next_event(deadline.saturating_duration_since(Instant::now()));
        if event["op"] == "publish" && event["fact"]["value"]["kind"] == "goal" {
            observed_goal = true;
            break;
        }
    }
    assert!(observed_goal, "actual build goal reached the same piped subscriber");
    assert!(daemon.0.try_wait().expect("poll daemon after goal").is_none());
    let build_deadline = Instant::now() + Duration::from_secs(30);
    while build.0.try_wait().expect("poll actual build").is_none() {
        assert!(Instant::now() < build_deadline, "observed build did not finish");
        thread::sleep(Duration::from_millis(50));
    }
    drop(build);
    drop(subscriber);
    reader.join().expect("join subscriber stdout reader");
}

#[test]
fn remote_serve_stdio_once_exchanges_frames_and_redeems_ticket() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_ticket_state(temp.path());
    let credentials_dir = write_service_credentials(temp.path());
    let input = encode_frames(&client_request_frames());
    let output = Command::cargo_bin("mantle")
        .expect("mantle binary")
        .args([
            "--state-dir",
            temp.path().to_str().expect("utf8 temp path"),
            "remote",
            "serve",
            "--endpoint-id",
            "builder-1",
            "--binding",
            "stdio-once",
            "--signing-key-id",
            RESULT_SIGNING_KEY_ID,
            "--secret-manifest",
            secret_manifest().to_str().expect("UTF-8 manifest path"),
        ])
        .env("CREDENTIALS_DIRECTORY", &credentials_dir)
        .write_stdin(input)
        .output()
        .expect("run stdio-once serve");

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "stdio protocol diagnostics stayed quiet");
    let accepted_wire = base64::engine::general_purpose::STANDARD
        .decode(
            include_str!("../.cairn/changes/separate-remote-build-hexagon/evidence/origin-main-accepted-wire.base64")
                .trim(),
        )
        .expect("origin/main accepted wire fixture");
    assert_eq!(output.stdout, accepted_wire, "accepted framed bytes changed from origin/main");

    let frames = decode_frames(&output.stdout);
    let ticket_state: Value = serde_json::from_slice(
        &fs::read(temp.path().join("remote-builders/tickets.json")).expect("ticket state after run"),
    )
    .expect("ticket state json");
    assert_eq!(frame_kind(&frames[0]), "auth-ok");
    assert_eq!(frame_kind(&frames[1]), "missing-inputs");
    assert_eq!(frame_kind(&frames[frames.len() - 1]), "done");
    let build_finished = frames
        .iter()
        .find(|frame| frame_kind(frame) == "build-finished")
        .expect("build-finished frame present");
    let accepted_receipt: Value = serde_json::from_slice(include_bytes!(
        "../.cairn/changes/separate-remote-build-hexagon/evidence/origin-main-receipt.json"
    ))
    .expect("origin/main receipt fixture");
    assert_eq!(build_finished["result"], accepted_receipt, "accepted receipt changed from origin/main");
    assert!(frames.iter().any(|frame| frame_kind(frame) == "output-transfer-done"));
    assert_eq!(build_finished["result"]["outputs"][0]["name"], "out");
    assert_eq!(build_finished["result"]["outputs"][0]["path_info_signing_key_id"], RESULT_SIGNING_KEY_ID);
    assert_eq!(
        build_finished["result"]["outputs"][0]["artifact_attestation_digest_blake3"]
            .as_str()
            .expect("artifact digest string")
            .len(),
        BLAKE3_HEX_LENGTH_CHARS
    );
    assert_eq!(ticket_state["tickets"][TEST_TICKET_ID]["uses_remaining"], 0);
}

#[test]
fn selected_casita_remote_child_rejects_snix_worker_state_before_loading_secrets() {
    let root = tempfile::tempdir().unwrap();
    let worker_state = root.path().join("worker-state");
    let worker_store = root.path().join("worker-store");
    let coordinator_state = root.path().join("coordinator-state");
    let missing_manifest = root.path().join("missing-secretspec.toml");
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        crunch_store::StoreHandle::open(crunch_store::StoreConfig::new(
            crunch_store::StoreBackend::Snix,
            worker_state.clone(),
            worker_store.clone(),
            "/mantle/store".to_string(),
        ))
        .await
        .unwrap();
    });
    let identity_path = worker_state.join("store-identity.json");
    let identity_before = fs::read(&identity_path).unwrap();
    let state_entries_before = fs::read_dir(&worker_state).unwrap().count();

    let failure = Command::cargo_bin("mantle")
        .unwrap()
        .args(["--state-dir", coordinator_state.to_str().unwrap()])
        .args(["--store", worker_store.to_str().unwrap()])
        .args(["--store-backend", "casita", "remote", "serve"])
        .args(["--binding", "stdio-once", "--executor", "local-build"])
        .args(["--execution-state-dir", worker_state.to_str().unwrap()])
        .args(["--secret-manifest", missing_manifest.to_str().unwrap()])
        .assert()
        .failure();
    let stderr = String::from_utf8_lossy(&failure.get_output().stderr);
    assert!(stderr.contains("store-backend-mismatch"), "{stderr}");
    assert!(stderr.contains("requested casita"), "{stderr}");
    assert!(stderr.contains("state declares snix"), "{stderr}");

    assert_eq!(fs::read(identity_path).unwrap(), identity_before);
    assert_eq!(fs::read_dir(worker_state).unwrap().count(), state_entries_before);
    assert!(!missing_manifest.exists());
    assert!(!coordinator_state.exists());
}

#[test]
fn concurrent_one_use_redemption_commits_exactly_once() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_ticket_state(temp.path());
    let credentials_dir = write_service_credentials(temp.path());
    let state_a = temp.path().to_path_buf();
    let state_b = state_a.clone();
    let credentials_a = credentials_dir.clone();
    let credentials_b = credentials_dir;
    let input_a = encode_frames(&client_request_frames());
    let input_b = input_a.clone();

    let contender_a = std::thread::spawn(move || run_stdio_contender(&state_a, &credentials_a, input_a));
    let contender_b = std::thread::spawn(move || run_stdio_contender(&state_b, &credentials_b, input_b));
    let outputs = [contender_a.join().unwrap(), contender_b.join().unwrap()];
    let success_count = outputs.iter().filter(|output| output.status.success()).count();
    let failure_count = outputs.len().saturating_sub(success_count);
    let ticket_state: Value = serde_json::from_slice(
        &fs::read(temp.path().join("remote-builders/tickets.json")).expect("ticket state after contenders"),
    )
    .expect("ticket state json");

    assert_eq!(success_count, 1);
    assert_eq!(failure_count, 1);
    assert_eq!(ticket_state["tickets"][TEST_TICKET_ID]["uses_remaining"], 0);
    assert!(outputs.iter().all(|output| !String::from_utf8_lossy(&output.stderr).contains(TICKET_TOKEN)));
}

#[test]
fn stdio_rejects_clock_rollback_before_any_success_frame() {
    let temp = tempfile::tempdir().expect("tempdir");
    let now_unix_s = test_unix_time_now_s();
    let created_unix_s = now_unix_s.checked_add(TEST_CLOCK_ROLLBACK_SECS).unwrap();
    let expires_unix_s = created_unix_s.checked_add(TEST_TICKET_TTL_SECS).unwrap();
    write_ticket_state_with_policy(temp.path(), created_unix_s, expires_unix_s, None);
    let credentials_dir = write_service_credentials(temp.path());
    let output = run_stdio_contender(temp.path(), &credentials_dir, encode_frames(&client_request_frames()));

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ticket-clock-before-issuance"));
    assert_eq!(read_ticket_uses(temp.path()), 1);
}

#[test]
fn stdio_rejects_endpoint_bound_ticket_without_authenticated_peer() {
    let temp = tempfile::tempdir().expect("tempdir");
    let created_unix_s = test_unix_time_now_s();
    let expires_unix_s = created_unix_s.checked_add(TEST_TICKET_TTL_SECS).unwrap();
    write_ticket_state_with_policy(temp.path(), created_unix_s, expires_unix_s, Some("client-a"));
    let credentials_dir = write_service_credentials(temp.path());
    let output = run_stdio_contender(temp.path(), &credentials_dir, encode_frames(&client_request_frames()));

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ticket-client-endpoint-mismatch"));
    assert_eq!(read_ticket_uses(temp.path()), 1);
}

#[test]
fn remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames() {
    let temp = tempfile::tempdir().expect("tempdir");
    let credentials_dir = write_service_credentials(temp.path());
    let input = encode_frames(&client_request_frames());
    let output = Command::cargo_bin("mantle")
        .expect("mantle binary")
        .args([
            "--state-dir",
            temp.path().to_str().expect("utf8 temp path"),
            "remote",
            "serve",
            "--endpoint-id",
            "builder-1",
            "--binding",
            "stdio-once",
            "--signing-key-id",
            RESULT_SIGNING_KEY_ID,
            "--secret-manifest",
            secret_manifest().to_str().expect("UTF-8 manifest path"),
        ])
        .env("CREDENTIALS_DIRECTORY", &credentials_dir)
        .write_stdin(input)
        .output()
        .expect("run stdio-once serve");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        include_bytes!("../.cairn/changes/separate-remote-build-hexagon/evidence/origin-main-rejected-stderr.txt"),
        "rejected diagnostic bytes changed from origin/main",
    );
}

fn run_stdio_contender(state_dir: &Path, credentials_dir: &Path, input: Vec<u8>) -> std::process::Output {
    Command::cargo_bin("mantle")
        .expect("mantle binary")
        .args([
            "--state-dir",
            state_dir.to_str().expect("UTF-8 state dir"),
            "remote",
            "serve",
            "--endpoint-id",
            "builder-1",
            "--binding",
            "stdio-once",
            "--signing-key-id",
            RESULT_SIGNING_KEY_ID,
            "--secret-manifest",
            secret_manifest().to_str().expect("UTF-8 manifest path"),
        ])
        .env("CREDENTIALS_DIRECTORY", credentials_dir)
        .write_stdin(input)
        .output()
        .expect("run stdio contender")
}

fn write_ticket_state(state_dir: &std::path::Path) {
    let created_unix_s = test_unix_time_now_s();
    let expires_unix_s = created_unix_s.checked_add(TEST_TICKET_TTL_SECS).unwrap();
    write_ticket_state_with_policy(state_dir, created_unix_s, expires_unix_s, None);
}

fn write_ticket_state_with_policy(
    state_dir: &std::path::Path,
    created_unix_s: u64,
    expires_unix_s: u64,
    bound_client_endpoint: Option<&str>,
) {
    let ticket_dir = state_dir.join("remote-builders");
    fs::create_dir_all(&ticket_dir).expect("ticket dir");
    fs::set_permissions(&ticket_dir, fs::Permissions::from_mode(PRIVATE_DIRECTORY_MODE)).expect("private ticket dir");
    let ticket_state = serde_json::json!({
        "schema_version": 2,
        "next_ticket_sequence": 1,
        "tickets": {
            TEST_TICKET_ID: {
                "id": TEST_TICKET_ID,
                "display_name": "ticket",
                "verifier_key_id": "ticket-key-1",
                "verifier": ticket_verifier(),
                "created_unix_s": created_unix_s,
                "expires_unix_s": expires_unix_s,
                "uses_remaining": 1,
                "max_build_time_secs": TEST_TICKET_MAX_BUILD_TIME_SECS,
                "max_upload_bytes": TEST_MAX_UPLOAD_BYTES,
                "bound_client_endpoint": bound_client_endpoint,
                "revoked": false
            }
        },
        "invalidated_legacy_ticket_ids": []
    });
    let path = ticket_dir.join("tickets.json");
    fs::write(&path, serde_json::to_vec_pretty(&ticket_state).expect("ticket state serializes"))
        .expect("write ticket state");
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_FILE_MODE)).expect("private ticket file");
}

fn read_ticket_uses(state_dir: &Path) -> u64 {
    let state: Value =
        serde_json::from_slice(&fs::read(state_dir.join("remote-builders/tickets.json")).expect("ticket state"))
            .expect("ticket state JSON");
    state["tickets"][TEST_TICKET_ID]["uses_remaining"].as_u64().expect("ticket uses")
}

fn test_unix_time_now_s() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

fn ticket_verifier() -> String {
    let mut input = Vec::from(TICKET_VERIFIER_DOMAIN);
    input.extend_from_slice(&[0x42_u8; 32]);
    let key = [0x41_u8; 32];
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(blake3::keyed_hash(&key, &input).as_bytes())
}

fn write_service_credentials(root: &Path) -> std::path::PathBuf {
    let credentials_dir = root.join("credentials");
    fs::create_dir_all(&credentials_dir).expect("credentials dir");
    fs::write(credentials_dir.join("TICKET_VERIFIER_KEY"), TICKET_KEY).expect("ticket key credential");
    fs::write(credentials_dir.join("RESULT_SIGNING_KEY"), RESULT_SIGNING_KEY).expect("signing key credential");
    credentials_dir
}

fn secret_manifest() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml")
}

fn client_request_frames() -> Vec<Value> {
    vec![
        serde_json::json!({
            "kind": "hello",
            "hello": {
                "alpn": PROTOCOL_ALPN,
                "version": PROTOCOL_VERSION,
                "endpoint_id": "builder-1",
                "capabilities": ["delta", "full"]
            }
        }),
        serde_json::json!({
            "kind": "auth-ticket",
            "auth": {
                "ticket_id": TEST_TICKET_ID,
                "secret": TICKET_TOKEN,
                "client_endpoint": null,
                "now_unix_s": TEST_NOW_UNIX_S
            }
        }),
        serde_json::json!({
            "kind": "build-request",
            "request": {
                "request_id": "request-1",
                "store_prefix": "/mantle/store",
                "input_refs": ["input-a"],
                "upload_bytes": TEST_UPLOAD_BYTES,
                "build_time_limit_secs": TEST_BUILD_TIME_LIMIT_SECS,
                "contains_raw_frontend_eval": false,
                "payload": {
                    "kind": "action",
                    "action_id": "action-1",
                    "spec_json": action_spec_json()
                },
                "expected_outputs": [{
                    "name": "out",
                    "logical_path": "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-fixture"
                }]
            }
        }),
        serde_json::json!({
            "kind": "input-manifest",
            "manifest": {
                "request_id": "request-1",
                "store_prefix": "/mantle/store",
                "input_refs": ["input-a"],
                "closure_refs": []
            }
        }),
        serde_json::json!({
            "kind": "input-upload",
            "upload": {
                "request_id": "request-1",
                "refs": ["input-a"],
                "byte_count": TEST_UPLOAD_BYTES
            }
        }),
    ]
}

fn action_spec_json() -> String {
    serde_json::json!({
        "schema": "mantle-remote-action-v1",
        "action_id": "action-1",
        "builder": "builtin:fixture",
        "args": ["--emit"],
        "outputs": ["out"]
    })
    .to_string()
}

fn encode_frames(frames: &[Value]) -> Vec<u8> {
    let mut encoded = Vec::new();
    for frame in frames {
        let payload = serde_json::to_vec(frame).expect("frame serializes");
        let payload_len = u32::try_from(payload.len()).expect("payload len fits u32");
        encoded.extend_from_slice(&payload_len.to_be_bytes());
        encoded.extend_from_slice(&payload);
    }
    encoded
}

fn decode_frames(encoded: &[u8]) -> Vec<Value> {
    let mut frames = Vec::new();
    let mut offset = 0_usize;
    while offset < encoded.len() {
        assert!(encoded.len().saturating_sub(offset) >= FRAME_HEADER_BYTES);
        let mut header = [0_u8; FRAME_HEADER_BYTES];
        header.copy_from_slice(&encoded[offset..offset + FRAME_HEADER_BYTES]);
        offset += FRAME_HEADER_BYTES;
        let payload_len = usize::try_from(u32::from_be_bytes(header)).expect("payload length fits usize");
        let end = offset.checked_add(payload_len).expect("payload end does not overflow");
        assert!(end <= encoded.len());
        frames.push(serde_json::from_slice(&encoded[offset..end]).expect("frame json"));
        offset = end;
    }
    frames
}

fn frame_kind(frame: &Value) -> &str {
    frame["kind"].as_str().expect("frame kind string")
}
