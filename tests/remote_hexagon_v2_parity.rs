use std::collections::BTreeMap;
use std::fs;
use std::io::{Read as _, Seek as _, Write as _};
use std::os::fd::AsRawFd as _;
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::process::CommandExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use base64::Engine as _;
use crunch_remote_core::protocol::{self, Direction, FrameKind, Phase};
use crunch_remote_core::receipt::{
    ExecutablePlanFacts, ExecutableSourceFacts, ExpectedOutputFacts, OrderedOutputReceipt, OutputReceiptFields,
};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};

const TICKET_KEY: &str = "ticket-key-1:QUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUE";
const SIGNING_KEY: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
const SIGNING_KEY_ID: &str = "cache.example.com-1";
const ORIGINAL_WIRE: &str =
    include_str!("../.cairn/changes/separate-remote-build-hexagon/evidence/origin-main-accepted-wire.base64");
const ORIGINAL_RECEIPT: &[u8] =
    include_bytes!("../.cairn/changes/separate-remote-build-hexagon/evidence/origin-main-receipt.json");
const ORIGINAL_PLAN: &[u8] =
    include_bytes!("../.cairn/changes/separate-remote-build-hexagon/evidence/origin-main-executable-plan.json");
const ORIGINAL_UNKNOWN_TICKET_STDERR: &[u8] =
    include_bytes!("../.cairn/changes/separate-remote-build-hexagon/evidence/origin-main-rejected-stderr.txt");
const ORIGINAL_MALFORMED_STDERR: &[u8] = include_bytes!("fixtures/remote_hexagon_v2/origin-malformed-stderr.txt");
const ORIGINAL_SESSION: &[u8] = include_bytes!("fixtures/remote_hexagon_v2/origin-session.json");
const ORIGINAL_PRODUCTION: &[u8] = include_bytes!("fixtures/remote_hexagon_v2/origin-production.json");
const ORIGINAL_SIGNED_SUCCESSOR: &[u8] = include_bytes!("fixtures/remote_hexagon_v2/origin-signed-successor.json");
const ORIGINAL_FAILED_ADAPTER: &[u8] = include_bytes!("fixtures/remote_hexagon_v2/origin-failed-adapter.json");

fn original_session() -> Value {
    serde_json::from_slice(ORIGINAL_SESSION).expect("independently captured origin session facts")
}

fn credential_directory(root: &Path) -> std::path::PathBuf {
    let credentials = root.join("credentials");
    fs::create_dir(&credentials).expect("private credentials directory");
    fs::set_permissions(&credentials, fs::Permissions::from_mode(0o700)).expect("private credentials permissions");
    for (name, value) in [("TICKET_VERIFIER_KEY", TICKET_KEY), ("RESULT_SIGNING_KEY", SIGNING_KEY)] {
        let file = credentials.join(name);
        fs::write(&file, value).expect("write test-only service credential");
        fs::set_permissions(file, fs::Permissions::from_mode(0o600)).expect("private credential permissions");
    }
    credentials
}

fn ticket_state_path(root: &Path) -> std::path::PathBuf {
    root.join("remote-builders/tickets.json")
}

fn inherit_private_fd(command: &mut Command, source_fd: i32) {
    // dup2 happens only in the child. The test process must not leak FD 9 to
    // unrelated concurrent tests or put the bearer in argv/environment.
    unsafe {
        command.pre_exec(move || {
            if libc::dup2(source_fd, 9) < 0 || libc::fcntl(9, libc::F_SETFD, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

fn issue_one_use_ticket(root: &Path, credentials: &Path) -> (String, String) {
    issue_ticket(root, credentials, "300", None)
}

fn issue_ticket(root: &Path, credentials: &Path, ttl_secs: &str, max_upload_bytes: Option<u64>) -> (String, String) {
    let mut delivery = tempfile::tempfile_in(root).expect("private ticket delivery file");
    let mut command = Command::new(env!("CARGO_BIN_EXE_mantle"));
    command
        .args(["--json", "--state-dir"])
        .arg(root)
        .args([
            "remote", "ticket", "create", "--display-name", "hexagon-v2-origin-issued",
            "--ttl-secs", ttl_secs, "--uses", "1", "--ticket-fd", "9", "--secret-manifest",
        ])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml"))
        .args(["--secret-profile", "production", "--secret-provider", "systemd-credential://"])
        .env("CREDENTIALS_DIRECTORY", credentials);
    if let Some(limit) = max_upload_bytes {
        command.args(["--max-build-time-secs", "600", "--max-upload-bytes"]).arg(limit.to_string());
    }
    inherit_private_fd(&mut command, delivery.as_raw_fd());
    let result = command.output().expect("issue actual one-use ticket");
    assert!(result.status.success(), "ticket issuance failed: {}", String::from_utf8_lossy(&result.stderr));
    delivery.rewind().expect("read issued credential");
    let mut credential = String::new();
    delivery.read_to_string(&mut credential).expect("read private ticket delivery");
    let (ticket_id, token) = credential.trim_end().split_once(':').expect("issued ID:bearer");
    assert!(!token.is_empty(), "issued bearer empty");
    (ticket_id.to_string(), token.to_string())
}

fn original_request_frames(ticket_id: &str, token: &str) -> Vec<Value> {
    let action_spec = json!({
        "schema": "mantle-remote-action-v1",
        "action_id": "action-1",
        "builder": "builtin:fixture",
        "args": ["--emit"],
        "outputs": ["out"]
    });
    vec![
        json!({"kind": "hello", "hello": {
            "alpn": "mantle-remote-build/1", "version": 1, "endpoint_id": "builder-1",
            "capabilities": ["delta", "full"]
        }}),
        json!({"kind": "auth-ticket", "auth": {
            "ticket_id": ticket_id, "secret": token, "client_endpoint": null, "now_unix_s": 1
        }}),
        json!({"kind": "build-request", "request": {
            "request_id": "request-1", "store_prefix": "/mantle/store", "input_refs": ["input-a"],
            "upload_bytes": 10, "build_time_limit_secs": 60, "contains_raw_frontend_eval": false,
            "payload": {"kind": "action", "action_id": "action-1", "spec_json": action_spec.to_string()},
            "expected_outputs": [{"name": "out", "logical_path": "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-fixture"}]
        }}),
        json!({"kind": "input-manifest", "manifest": {
            "request_id": "request-1", "store_prefix": "/mantle/store", "input_refs": ["input-a"], "closure_refs": []
        }}),
        json!({"kind": "input-upload", "upload": {
            "request_id": "request-1", "refs": ["input-a"], "byte_count": 10
        }})
    ]
}

fn encode_frames(frames: &[Value]) -> Vec<u8> {
    let mut wire = Vec::new();
    for frame in frames {
        let payload = serde_json::to_vec(frame).expect("encode client frame");
        wire.extend_from_slice(&u32::try_from(payload.len()).expect("bounded frame").to_be_bytes());
        wire.extend_from_slice(&payload);
    }
    wire
}

fn decode_frames(wire: &[u8]) -> Vec<Value> {
    let mut frames = Vec::new();
    let mut offset = 0;
    while offset < wire.len() {
        let end_header = offset + 4;
        let header: [u8; 4] = wire[offset..end_header].try_into().expect("complete frame header");
        let end_frame = end_header + usize::try_from(u32::from_be_bytes(header)).expect("frame length fits usize");
        frames.push(serde_json::from_slice(&wire[end_header..end_frame]).expect("complete JSON frame"));
        offset = end_frame;
    }
    frames
}

fn decode_production_response_frames(wire: &[u8]) -> Vec<Value> {
    assert!(wire.len() <= 512 * 1024, "physical Worker response exceeds private capture bound");
    let mut frames = Vec::new();
    let mut offset = 0_usize;
    let mut envelope_count = 0_usize;
    let mut data_chunks = 0_usize;
    let mut output_transfer: Option<Value> = None;
    while offset < wire.len() {
        envelope_count += 1;
        assert!(envelope_count <= 4_096, "physical Worker response frame count exceeded");
        let header_end = offset.checked_add(4).expect("bounded response header offset");
        let header: [u8; 4] = wire.get(offset..header_end).expect("complete response header")
            .try_into().expect("four response header bytes");
        let payload_bytes = usize::try_from(u32::from_be_bytes(header)).expect("bounded response envelope length");
        assert!(payload_bytes > 0, "empty physical response envelope");
        let payload_end = header_end.checked_add(payload_bytes).expect("bounded response envelope offset");
        let payload = wire.get(header_end..payload_end).expect("complete physical response JSON envelope");
        let envelope: Value = serde_json::from_slice(payload).expect("physical response JSON envelope");
        offset = payload_end;

        if envelope["schema"] == "mantle-remote-transfer-data-frame-v1" {
            let transfer = output_transfer.as_ref().expect("raw output chunk needs preceding DOWNLOAD manifest");
            let manifest = &transfer["manifest"];
            let header = &envelope["chunk"];
            let scope = &header["scope"];
            for field in ["session_id", "job_id", "attempt_id", "fence_generation", "policy_digest_blake3"] {
                assert_eq!(scope[field], manifest[field], "raw chunk scope differs from output manifest");
            }
            assert_eq!(scope["manifest_digest_blake3"], transfer["manifest_digest_blake3"]);
            let descriptor = &header["chunk"];
            let artifact = manifest["artifacts"].as_array().expect("actual output manifest artifacts").iter()
                .find(|artifact| artifact["artifact_id"] == header["artifact_id"]
                    && artifact["artifact_kind"] == header["artifact_kind"])
                .expect("raw chunk artifact belongs to output manifest");
            assert!(artifact["chunks"].as_array().expect("manifest chunk descriptors")
                .iter().any(|chunk| chunk == descriptor), "raw chunk descriptor not in output manifest");
            let raw_bytes = usize::try_from(descriptor["size_bytes"].as_u64().expect("raw chunk byte count"))
                .expect("bounded raw chunk length");
            assert!(raw_bytes > 0, "empty output chunk");
            let raw_end = offset.checked_add(raw_bytes).expect("bounded raw chunk offset");
            let raw = wire.get(offset..raw_end).expect("complete raw output chunk");
            assert_eq!(blake3::hash(raw).to_hex().to_string(),
                descriptor["digest_blake3"].as_str().expect("manifest chunk BLAKE3"));
            offset = raw_end;
            data_chunks += 1;
            continue;
        }

        let kind = envelope["kind"].as_str().expect("known control frame or bounded data header");
        match kind {
            "transfer-demand" | "transfer-credit" | "transfer-acknowledgement" => {
                assert!(output_transfer.is_none(), "input transfer sideband appeared during output");
            }
            "auth-ok" | "missing-inputs" | "transfer-complete" | "build-queued" | "build-started"
            | "build-finished" | "output-transfer-done" | "done" => frames.push(envelope),
            "transfer-manifest" => {
                assert_eq!(envelope["transfer"]["direction"], "download", "Worker output manifest direction");
                assert!(output_transfer.replace(envelope["transfer"].clone()).is_none(),
                    "duplicate Worker output manifest");
                frames.push(envelope);
            }
            other => panic!("unrecognized physical Worker control frame: {other}"),
        }
    }
    assert!(data_chunks > 0, "signed output transferred no authenticated raw chunks");
    assert_eq!(offset, wire.len(), "physical response ended before consuming all bytes");
    frames
}

fn serve_once(root: &Path, credentials: &Path, input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mantle"))
        .arg("--state-dir")
        .arg(root)
        .args([
            "remote", "serve", "--endpoint-id", "builder-1", "--binding", "stdio-once",
            "--signing-key-id", SIGNING_KEY_ID, "--secret-manifest",
        ])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml"))
        .env("CREDENTIALS_DIRECTORY", credentials)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch actual mantle stdio server");
    child.stdin.take().expect("stdio child input").write_all(input).expect("write framed client request");
    child.wait_with_output().expect("wait for actual stdio server")
}

fn assert_origin_receipt_digests(receipt: &Value) {
    let original: Value = serde_json::from_slice(ORIGINAL_RECEIPT).expect("original receipt JSON");
    assert_eq!(receipt, &original, "original and extracted results differ");
    let outputs = original["outputs"].as_array().expect("original output list");
    let mut extracted_digest = OrderedOutputReceipt::new();
    for output in outputs {
        extracted_digest.append(OutputReceiptFields {
            name: output["name"].as_str().expect("original output name"),
            logical_path: output["logical_path"].as_str().expect("original output path"),
            content_digest_blake3: output["content_digest_blake3"].as_str().expect("original content digest"),
            artifact_attestation_digest_blake3: output["artifact_attestation_digest_blake3"]
                .as_str().expect("original artifact digest"),
            size_bytes: output["size_bytes"].as_u64().expect("original output size"),
            nar_payload_digest_blake3: output["nar_payload_digest_blake3"].as_str(),
            nar_payload_size_bytes: output["nar_payload_size_bytes"].as_u64(),
        });
    }
    let extracted_digest = blake3::Hash::from_bytes(extracted_digest.finish()).to_hex().to_string();
    assert_eq!(extracted_digest, original["output_digest_blake3"].as_str().expect("original receipt digest"),
        "origin receipt preimage changed");
}

#[test]
fn accepted_stdio_matches_independently_observed_origin_wire_receipt_and_durable_ticket() {
    let root = tempfile::tempdir().expect("private accepted fixture root");
    let credentials = credential_directory(root.path());
    let (ticket_id, token) = issue_one_use_ticket(root.path(), &credentials);
    let before: Value = serde_json::from_slice(&fs::read(ticket_state_path(root.path())).expect("issued ticket input"))
        .expect("before ticket JSON");
    let old = original_session();
    assert_eq!(before["tickets"][&ticket_id]["uses_remaining"], old["accepted"]["ticket_uses_before"]);

    let client_frames = original_request_frames(&ticket_id, &token);
    let observed = serve_once(root.path(), &credentials, &encode_frames(&client_frames));
    assert_eq!(observed.status.code(), old["accepted"]["exit_code"].as_i64()
        .map(|code| i32::try_from(code).expect("bounded original exit code")),
        "accepted original input rejected: {}", String::from_utf8_lossy(&observed.stderr));
    assert_eq!(observed.stderr.len(),
        usize::try_from(old["accepted"]["stderr_bytes"].as_u64().expect("original stderr length")).expect("bounded stderr"));
    let origin_wire = base64::engine::general_purpose::STANDARD.decode(ORIGINAL_WIRE.trim()).expect("old wire bytes");
    assert_eq!(observed.stdout, origin_wire, "actual current output differs from original process bytes");
    assert_eq!(observed.stdout.len(),
        usize::try_from(old["accepted"]["wire_bytes"].as_u64().expect("original wire length")).expect("bounded wire"));
    assert_eq!(format!("{:x}", Sha256::digest(&observed.stdout)),
        old["accepted"]["wire_sha256"].as_str().expect("original wire SHA-256"));

    let frames = decode_frames(&observed.stdout);
    let kinds = frames.iter().map(|frame| frame["kind"].as_str().expect("response frame kind")).collect::<Vec<_>>();
    let old_kinds = old["accepted"]["frame_kinds"].as_array().expect("old frame kinds");
    assert_eq!(kinds, old_kinds.iter().map(|kind| kind.as_str().expect("old kind")).collect::<Vec<_>>());
    let mut phase = Phase::Open;
    phase = protocol::transition(phase, Direction::ClientToBuilder, FrameKind::Hello)
        .expect("original client hello admitted");
    assert_eq!(phase, Phase::AwaitAuth);
    phase = protocol::transition(phase, Direction::ClientToBuilder, FrameKind::AuthTicket)
        .expect("original client ticket admitted");
    assert_eq!(phase, Phase::AwaitAuthOk);
    for (frame, expected) in frames.iter().zip(old["accepted"]["server_phases_after_response"].as_array().expect("old phases")) {
        let kind = match frame["kind"].as_str().expect("response kind") {
            "auth-ok" => FrameKind::AuthOk,
            "missing-inputs" => {
                phase = protocol::transition(phase, Direction::ClientToBuilder, FrameKind::BuildRequest)
                    .expect("original client build request admitted");
                phase = protocol::transition(phase, Direction::ClientToBuilder, FrameKind::InputManifest)
                    .expect("original client input manifest admitted");
                FrameKind::MissingInputs
            }
            "build-queued" => {
                phase = protocol::transition(phase, Direction::ClientToBuilder, FrameKind::InputUpload)
                    .expect("original client upload admitted");
                FrameKind::BuildQueued
            }
            "build-started" => FrameKind::BuildStarted,
            "build-finished" => FrameKind::BuildFinished,
            "output-transfer-done" => FrameKind::OutputTransferDone,
            "done" => FrameKind::Done,
            other => panic!("unrecognized original response kind: {other}"),
        };
        phase = protocol::transition(phase, Direction::BuilderToClient, kind).expect("extracted protocol accepts old frame");
        assert_eq!(phase.as_str(), expected.as_str().expect("old phase"));
    }
    assert_eq!(phase, Phase::Done, "original accepted session completed");
    let receipt = &frames.iter().find(|frame| frame["kind"] == "build-finished").expect("original result frame")["result"];
    assert_origin_receipt_digests(receipt);
    assert_eq!(receipt["output_digest_blake3"], old["accepted"]["output_digest_blake3"]);
    let after: Value = serde_json::from_slice(&fs::read(ticket_state_path(root.path())).expect("persisted ticket state"))
        .expect("persisted ticket JSON");
    assert_eq!(after["tickets"][&ticket_id]["uses_remaining"], old["accepted"]["ticket_uses_after"]);
    assert_eq!(after["tickets"][&ticket_id]["verifier"], before["tickets"][&ticket_id]["verifier"]);
}

#[test]
fn original_executable_plan_preimage_matches_extracted_receipt_core() {
    let old: Value = serde_json::from_slice(ORIGINAL_PLAN).expect("independent original plan capture");
    let args: Vec<String> = serde_json::from_value(old["command_args"].clone()).expect("original builder args");
    let env: BTreeMap<String, String> = serde_json::from_value(old["command_env"].clone()).expect("original builder environment");
    let outputs = old["expected_outputs"].as_array().expect("original expected outputs");
    let extracted = crunch_remote_core::receipt::executable_plan_digest(ExecutablePlanFacts {
        request_id: old["request_id"].as_str().expect("original request id"),
        store_prefix: old["store_prefix"].as_str().expect("original store prefix"),
        source: ExecutableSourceFacts::Action {
            action_id: old["source"]["action_id"].as_str().expect("original action id"),
            schema: old["source"]["schema"].as_str().expect("original action schema"),
            spec_digest_blake3: old["source"]["spec_digest_blake3"].as_str().expect("original action digest"),
        },
        system: old["system"].as_str().expect("original system"),
        command_args: &args,
        command_env: &env,
        expected_outputs: outputs.iter().map(|output| ExpectedOutputFacts {
            name: output["name"].as_str().expect("original output name"),
            logical_path: output["logical_path"].as_str(),
        }),
    }).expect("extracted plan accepts original action");
    assert_eq!(blake3::Hash::from_bytes(extracted).to_hex().to_string(),
        old["plan_digest_blake3"].as_str().expect("original digest"));
}

#[test]
fn malformed_frame_rejects_with_distinct_original_diagnostic_before_ticket_state() {
    let root = tempfile::tempdir().expect("private malformed fixture root");
    let credentials = credential_directory(root.path());
    let old = original_session();
    let byte = u8::from_str_radix(old["malformed_header"]["input_bytes_hex"].as_str().expect("old malformed input"), 16)
        .expect("one malformed byte");
    let observed = serve_once(root.path(), &credentials, &[byte]);
    assert_eq!(observed.status.code(), old["malformed_header"]["exit_code"].as_i64()
        .map(|code| i32::try_from(code).expect("bounded original exit code")));
    assert_eq!(observed.stdout.len(), usize::try_from(
        old["malformed_header"]["wire_bytes"].as_u64().expect("old empty wire")
    ).expect("bounded original wire length"));
    assert_eq!(observed.stderr, ORIGINAL_MALFORMED_STDERR, "wrong original malformed-frame reason");
    assert_ne!(observed.stderr, ORIGINAL_UNKNOWN_TICKET_STDERR, "distinct rejected authority");
    assert_eq!(format!("{:x}", Sha256::digest(&observed.stderr)),
        old["malformed_header"]["stderr_sha256"].as_str().expect("original malformed SHA-256"));
    assert_eq!(ticket_state_path(root.path()).exists(),
        old["malformed_header"]["ticket_state_created"].as_bool().expect("original durable state"));
}

const PRODUCTION_INPUT: &str = "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-production-input";
const PRODUCTION_PAYLOAD_BYTES: usize = 4096;

struct ProductionFixture {
    state: PathBuf,
    store: PathBuf,
    build_file: PathBuf,
    credentials: PathBuf,
    payload: Vec<u8>,
}

fn production_fixture(root: &Path) -> ProductionFixture {
    let state = root.join("state");
    fs::create_dir(&state).expect("private production state directory");
    let store = root.join("store");
    let credentials = credential_directory(&state);
    fs::create_dir(&store).expect("private client store");
    let payload = (0..PRODUCTION_PAYLOAD_BYTES)
        .map(|index| u8::try_from(index % 251).expect("patterned source byte"))
        .collect::<Vec<_>>();
    let source = root.join("output-source.bin");
    fs::write(&source, &payload).expect("write local file URL source");
    fs::write(store.join(PRODUCTION_INPUT.rsplit('/').next().expect("input basename")), &payload)
        .expect("write physical input store object");
    let digest = base64::engine::general_purpose::STANDARD.encode(Sha256::digest(&payload));
    let build_file = root.join("remote-production.ncl");
    let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/lib.ncl");
    let expression = format!(
        r#"let mantle = import "{}" in
{{
  name = "production-remote-output",
  builder = "builtin:fetchurl",
  system = 'x86_64-linux,
  args = [],
  outputs = ["out"],
  env = {{ url = "file://{}" }},
  inputs = ["{}"],
  fixed_output = {{ hash = "sha256-{}", algo = 'sha256, mode = 'flat }},
  addressing_mode = 'input-addressed,
  sandbox = 'native,
}} | mantle.Derivation
"#,
        library.display(), source.display(), PRODUCTION_INPUT, digest
    );
    fs::write(&build_file, expression).expect("write physical production derivation");
    ProductionFixture { state, store, build_file, credentials, payload }
}

fn production_ticket(fixture: &ProductionFixture, max_upload_bytes: u64) -> (String, String) {
    issue_ticket(&fixture.state, &fixture.credentials, "3600", Some(max_upload_bytes))
}

fn run_production(
    root: &Path,
    fixture: &ProductionFixture,
    ticket_id: &str,
    bearer: &str,
    trusted_key_override: Option<&str>,
    builder_program: Option<&Path>,
) -> Output {
    let mut delivery = tempfile::tempfile_in(root).expect("private ticket input file");
    writeln!(delivery, "{ticket_id}:{bearer}").expect("write bearer to caller-owned FD only");
    delivery.rewind().expect("rewind private ticket input");
    let mut command = Command::new(env!("CARGO_BIN_EXE_mantle"));
    command
        .args(["--json", "--state-dir"])
        .arg(&fixture.state)
        .arg("--store")
        .arg(&fixture.store)
        .args(["--store-prefix", "/mantle/store", "build"])
        .arg(&fixture.build_file)
        .args([
            "--no-substitute", "--builder", "production-builder",
            "--ticket-fd", "9", "--remote-build-time-secs", "600", "--remote-secret-manifest",
        ])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml"))
        .env("CREDENTIALS_DIRECTORY", &fixture.credentials)
        .env("CRUNCH_NO_FUSE", "1")
        .env("TMPDIR", root);
    if let Some(key) = trusted_key_override {
        command.args(["--trusted-builder-key", key]);
    }
    if let Some(program) = builder_program {
        command.arg("--builder-program").arg(program);
        let worker_root = fixture.state.join("remote-workers")
            .join(blake3::hash(b"production-builder").to_hex().to_string());
        let args = [
            "--state-dir".to_string(),
            fixture.state.display().to_string(),
            "--store".to_string(),
            worker_root.join("store").display().to_string(),
            "--store-prefix".to_string(),
            "/mantle/store".to_string(),
            "--store-backend".to_string(),
            "snix".to_string(),
            "remote".to_string(),
            "serve".to_string(),
            "--endpoint-id".to_string(),
            "production-builder".to_string(),
            "--binding".to_string(),
            "stdio-once".to_string(),
            "--executor".to_string(),
            "local-build".to_string(),
            "--execution-state-dir".to_string(),
            worker_root.join("state").display().to_string(),
            "--secret-manifest".to_string(),
            Path::new(env!("CARGO_MANIFEST_DIR")).join("secretspec.toml").display().to_string(),
            "--secret-profile".to_string(),
            "production".to_string(),
            "--secret-provider".to_string(),
            "systemd-credential://".to_string(),
        ];
        for arg in args {
            command.arg(format!("--builder-arg={arg}"));
        }
        command.env("MANTLE_TEST_BIN", env!("CARGO_BIN_EXE_mantle"));
        command.env("MANTLE_TEST_WIRE_OUTPUT", root.join("worker-response.wire"));
    }
    inherit_private_fd(&mut command, delivery.as_raw_fd());
    command.output().expect("run real remote production child and import")
}

fn store_command(fixture: &ProductionFixture, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mantle"))
        .args(["--json", "--state-dir"])
        .arg(&fixture.state)
        .arg("--store")
        .arg(&fixture.store)
        .args(["--store-prefix", "/mantle/store", "store"])
        .args(args)
        .output()
        .expect("run actual store inspection")
}

fn production_ticket_uses(fixture: &ProductionFixture, ticket_id: &str) -> Value {
    let state: Value = serde_json::from_slice(&fs::read(ticket_state_path(&fixture.state)).expect("issued ticket state"))
        .expect("durable ticket JSON");
    state["tickets"][ticket_id]["uses_remaining"].clone()
}

fn coordinator_state(fixture: &ProductionFixture) -> Value {
    serde_json::from_slice(&fs::read(fixture.state.join("remote-coordinator-state.json")).expect("coordinator state"))
        .expect("durable coordinator JSON")
}

fn single_coordinator_job(coordinator: &Value) -> &Value {
    let jobs = coordinator["jobs"].as_object().expect("coordinator jobs");
    assert_eq!(jobs.len(), 1, "one physical job");
    jobs.values().next().expect("physical job")
}

fn telemetry_events(events: &Value) -> Value {
    Value::Array(events.as_array().expect("ordered telemetry events").iter().map(|event| json!({
        "category": event["category"],
        "phase": event["phase"],
        "reason": event["reason"],
        "result": event["result"],
        "retry": event["retry"],
        "route": event["route"],
        "transfer": event["transfer"],
        "measurement": event["measurement"],
        "value": event["value"],
    })).collect())
}

fn physical_observability_log(fixture: &ProductionFixture, attempt: &Value) -> Value {
    let mut matching = Vec::new();
    let root = fixture.state.join("remote-attempt-logs");
    for directory in fs::read_dir(root).expect("durable attempt log directories") {
        let directory = directory.expect("durable attempt log directory").path();
        let manifest: Value = serde_json::from_slice(&fs::read(directory.join("manifest.json"))
            .expect("durable attempt log manifest")).expect("attempt log manifest JSON");
        if manifest["scope"]["attempt_id"] != attempt["attempt_id"] {
            continue;
        }
        assert_eq!(manifest["scope"]["job_id"], attempt["job_id"]);
        assert_eq!(manifest["scope"]["fence_generation"], attempt["fence_generation"]);
        for reference in manifest["segments"].as_array().expect("immutable log segment references") {
            let digest = reference["segment_blake3"].as_str().expect("segment digest");
            let segment: Value = serde_json::from_slice(&fs::read(directory.join("segments").join(format!("{digest}.json")))
                .expect("immutable log segment")).expect("immutable log segment JSON");
            assert_eq!(segment["segment_blake3"], reference["segment_blake3"]);
            assert_eq!(segment["scope"], manifest["scope"]);
            for record in segment["records"].as_array().expect("immutable log records") {
                assert_eq!(record["flags"]["secret_redacted"], false, "event was redacted");
                assert_eq!(record["flags"]["payload_truncated"], false, "event was truncated");
                let bytes: Vec<u8> = serde_json::from_value(record["payload"].clone()).expect("real log payload bytes");
                let payload: Value = serde_json::from_slice(&bytes).expect("real log payload JSON");
                if payload["schema"] == "mantle-remote-observability-log-v1" {
                    matching.push(json!({
                        "accepted_events": payload["accepted_events"],
                        "dropped_events": payload["dropped_events"],
                        "events": telemetry_events(&payload["events"]),
                    }));
                }
            }
        }
    }
    assert_eq!(matching.len(), 1, "exactly one matching, untruncated durable observability record");
    matching.pop().expect("matched original-shaped immutable event")
}

fn production_checkpoints(fixture: &ProductionFixture) -> Vec<(bool, Value)> {
    let mut directories = vec![(false, fixture.state.join("remote-transfers"))];
    let worker_root = fixture.state.join("remote-workers");
    if worker_root.exists() {
        for entry in fs::read_dir(worker_root).expect("worker state directory") {
            directories.push((true, entry.expect("worker entry").path().join("state/remote-transfers")));
        }
    }
    let mut checkpoints = Vec::new();
    for (worker, directory) in directories {
        if !directory.exists() {
            continue;
        }
        for entry in fs::read_dir(directory).expect("transfer state directory") {
            let file = entry.expect("transfer entry").path();
            if file.extension().is_some_and(|extension| extension == "json") {
                let state: Value = serde_json::from_slice(&fs::read(file).expect("transfer checkpoint"))
                    .expect("durable transfer JSON");
                checkpoints.push((worker, state["checkpoint"].clone()));
            }
        }
    }
    checkpoints
}

fn assert_physical_attempt(fixture: &ProductionFixture, expected: &Value, expected_claim: Option<&str>) -> Value {
    let coordinator = coordinator_state(fixture);
    let job = single_coordinator_job(&coordinator);
    let attempt = &job["current_attempt"];
    assert_eq!(job["phase"], expected["coordinator_phase"]);
    assert_eq!(attempt["phase"], expected["attempt_phase"]);
    assert_eq!(attempt["fence_generation"], expected["fence_generation"]);
    assert_eq!(attempt["attempts_started"], expected["attempts_started"]);
    assert_eq!(job["output_admission_completed"], expected["output_admission_completed"]);
    assert_eq!(job["resource_fit"], expected["resource_fit"]);
    assert_eq!(coordinator["live_output_claims"].as_object().expect("coordinator claims").len(),
        usize::try_from(expected["live_output_claims"].as_u64().expect("old claim count")).expect("bounded claims"));
    if let Some(path) = expected_claim {
        assert!(coordinator["live_output_claims"].get(path).is_some(), "admitted output must have a live claim");
    }
    assert_eq!(job["last_attempt_reason_code"], expected["last_attempt_reason_code"]);
    let checkpoints = production_checkpoints(fixture);
    assert_eq!(checkpoints.len(),
        usize::try_from(expected["checkpoint_count"].as_u64().expect("old checkpoint count")).expect("bounded checkpoints"));
    for (_, checkpoint) in &checkpoints {
        let scope = &checkpoint["scope"];
        assert_eq!(scope["job_id"], attempt["job_id"]);
        assert_eq!(scope["attempt_id"], attempt["attempt_id"]);
        assert_eq!(scope["fence_generation"], attempt["fence_generation"]);
    }
    for (worker, label) in [(false, "client_transfer"), (true, "worker_transfer")] {
        if let Some(old_transfer) = expected.get(label) {
            let (_, observed) = checkpoints.iter().find(|(side, _)| *side == worker)
                .expect("old physically observed transfer side");
            assert_eq!(observed["acknowledged_chunk_digests"].as_array().expect("acknowledged chunks").len(),
                usize::try_from(old_transfer["acknowledged_chunks"].as_u64().expect("old chunks"))
                    .expect("bounded chunks"));
            assert_eq!(observed["transferred_bytes"], old_transfer["transferred_bytes"]);
        }
    }
    coordinator
}

fn assert_rejected_physical_production(case: &str, max_upload_bytes: u64, trusted_override: Option<&str>) {
    let old: Value = serde_json::from_slice(ORIGINAL_PRODUCTION).expect("independent original production outcomes");
    let expected = &old[case];
    let root = tempfile::tempdir().expect("private physical production root");
    let fixture = production_fixture(root.path());
    assert_eq!(format!("{:x}", Sha256::digest(&fixture.payload)),
        old["payload_sha256"].as_str().expect("old physical payload SHA-256"));
    let (ticket_id, bearer) = production_ticket(&fixture, max_upload_bytes);
    assert_eq!(production_ticket_uses(&fixture, &ticket_id), expected["ticket_uses_before"]);
    let output = run_production(root.path(), &fixture, &ticket_id, &bearer, trusted_override, None);
    assert_eq!(output.status.code(), expected["exit_code"].as_i64().map(|code| i32::try_from(code).expect("exit code")),
        "rejected physical production unexpectedly returned success");
    assert_eq!(output.stdout.len(),
        usize::try_from(expected["stdout_bytes"].as_u64().expect("old rejected stdout")).expect("bounded stdout"));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let reason = expected["reason_code"].as_str().expect("old rejection reason");
    let anchor = expected["diagnostic_anchor"].as_str().expect("independently observed diagnostic");
    assert!(stderr.contains(anchor), "expected old {reason} rejection point, got: {stderr}");
    assert_eq!(production_ticket_uses(&fixture, &ticket_id), expected["ticket_uses_after"]);
    assert_physical_attempt(&fixture, expected, None);
    let store_paths = fs::read_dir(&fixture.store).expect("client store").count();
    assert_eq!(store_paths,
        usize::try_from(expected["client_store_paths"].as_u64().expect("old client store count")).expect("bounded paths"));
    assert_eq!(fs::read(fixture.store.join(PRODUCTION_INPUT.rsplit('/').next().unwrap())).expect("client input"),
        fixture.payload);
}

#[test]
fn physical_production_signed_output_and_fenced_transfer_match_original_semantics() {
    let old: Value = serde_json::from_slice(ORIGINAL_PRODUCTION).expect("independent original production outcome");
    let expected = &old["accepted"];
    // The independent da00f receipt covers two signed Workers and same-job
    // fence 1→2/backoff/stale-before-import. This CLI run checks the shared
    // signed successor surface, not a second CLI retry.
    let signed: Value = serde_json::from_slice(ORIGINAL_SIGNED_SUCCESSOR).expect("independent original signed successor");
    let root = tempfile::tempdir().expect("private accepted production root");
    let fixture = production_fixture(root.path());
    assert_eq!(format!("{:x}", Sha256::digest(&fixture.payload)),
        old["payload_sha256"].as_str().expect("old physical payload SHA-256"));
    let (ticket_id, bearer) = production_ticket(&fixture, 16 * 1024 * 1024);
    assert_eq!(production_ticket_uses(&fixture, &ticket_id), expected["ticket_uses_before"]);
    let wrapper = root.path().join("capture-worker-response.sh");
    fs::write(&wrapper, b"#!/usr/bin/env bash\nset -euo pipefail\n\"$MANTLE_TEST_BIN\" \"$@\" | (ulimit -f 512; umask 077; tee \"$MANTLE_TEST_WIRE_OUTPUT\")\n")
        .expect("private bounded response-only worker wrapper");
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700)).expect("private worker wrapper mode");
    // Dispatch matches the Worker's signer name; full-public-key store verification follows import.
    let trusted_builder_name = crunch_build::load_keypair(SIGNING_KEY)
        .expect("test signing keypair").verifying_key.name().to_string();
    let output = run_production(root.path(), &fixture, &ticket_id, &bearer, Some(&trusted_builder_name), Some(&wrapper));
    assert_eq!(output.status.code(), expected["exit_code"].as_i64().map(|code| i32::try_from(code).expect("exit code")),
        "physical build failed: {}", String::from_utf8_lossy(&output.stderr));
    let response = root.path().join("worker-response.wire");
    assert!(fs::metadata(&response).expect("real Worker response capture").len() <= 512 * 1024,
        "response-only frame capture exceeded bound");
    let frames = decode_production_response_frames(&fs::read(&response).expect("actual Worker response bytes"));
    fs::remove_file(response).expect("remove private response-only frame capture");
    let kinds = frames.iter().map(|frame| match frame["kind"].as_str().expect("real Worker frame kind") {
        "auth-ok" => "AuthOk",
        "missing-inputs" => "MissingInputs",
        "transfer-complete" => "TransferComplete",
        "build-queued" => "BuildQueued",
        "build-started" => "BuildStarted",
        "build-finished" => "BuildFinished",
        "transfer-manifest" => "TransferManifest",
        "output-transfer-done" => "OutputTransferDone",
        "done" => "Done",
        kind => panic!("unrecognized actual Worker response frame: {kind}"),
    }).collect::<Vec<_>>();
    for label in ["original_worker_frame_kinds", "successor_worker_frame_kinds"] {
        assert_eq!(kinds, signed[label].as_array().expect("original signed Worker frame sequence")
            .iter().map(|kind| kind.as_str().expect("original response frame kind")).collect::<Vec<_>>());
    }
    let report: Value = serde_json::from_slice(&output.stdout).expect("physical build JSON");
    let imported = PathBuf::from(report["outcomes"][0]["outputs"][0]["path"].as_str().expect("imported store path"));
    assert_eq!(imported.parent(), Some(fixture.store.as_path()), "physical output must be client-imported");
    let logical = format!("/mantle/store/{}", imported.file_name().expect("store basename").to_string_lossy());
    assert_eq!(fs::read(&imported).expect("physical imported output"), fixture.payload);
    assert_eq!(logical, signed["logical_output_path"].as_str().expect("original signed output path"));
    assert_eq!(fs::metadata(&imported).expect("imported signed bytes").len(),
        signed["output_bytes"].as_u64().expect("original signed physical byte count"));
    assert_eq!(format!("{:x}", Sha256::digest(&fixture.payload)),
        signed["output_sha256"].as_str().expect("original signed physical output SHA-256"));
    assert_eq!(format!("{:x}", Sha256::digest(&fixture.payload)),
        expected["output_sha256"].as_str().expect("old signed output SHA-256"));
    assert_eq!(fs::metadata(&imported).expect("imported output metadata").len(),
        expected["output_bytes"].as_u64().expect("old physical output bytes"));
    assert_eq!(production_ticket_uses(&fixture, &ticket_id), expected["ticket_uses_after"]);
    let coordinator = assert_physical_attempt(&fixture, expected, Some(&logical));
    let job = single_coordinator_job(&coordinator);
    assert_eq!(job["observability_health"]["telemetry"]["accepted_events"], expected["telemetry_accepted_events"]);
    assert_eq!(job["observability_health"]["telemetry"]["dropped_events"], expected["telemetry_dropped_events"]);
    assert_eq!(job["phase"], signed["terminal_job_phase"]);
    assert_eq!(job["current_attempt"]["phase"], signed["terminal_attempt_phase"]);
    assert_eq!(job["last_attempt_reason_code"], signed["terminal_reason"]);
    assert_eq!(job["output_admission_completed"], signed["terminal_output_admission_completed"]);
    let telemetry = physical_observability_log(&fixture, &job["current_attempt"]);
    assert_eq!(telemetry["events"], signed["events_after_success"],
        "real current accepted phases and admission diverged from independent signed original");
    let selector = imported.file_name().expect("imported store basename").to_str().expect("UTF-8 store path");
    let info_output = store_command(&fixture, &["info", selector]);
    assert!(info_output.status.success(), "store info failed: {}", String::from_utf8_lossy(&info_output.stderr));
    let info: Value = serde_json::from_slice(&info_output.stdout).expect("signed PathInfo JSON");
    let path_info = &info["paths"][0];
    assert_eq!(path_info["nar_sha256"], expected["nar_sha256"]);
    assert_eq!(path_info["nar_size"], expected["nar_bytes"]);
    let signatures = path_info["signatures"].as_array().expect("signed PathInfo signatures");
    assert_eq!(signatures.len(), 1);
    assert!(signatures[0].as_str().expect("PathInfo signature")
        .starts_with(&format!("{}:", expected["signing_key_id"].as_str().expect("original builder key"))));
    assert!(signatures[0].as_str().expect("signed output PathInfo signature").starts_with(&format!("{}:",
        signed["imported_pathinfo_signer"].as_str().expect("original signed output signer"))));
    let verifying_key = crunch_build::load_keypair(SIGNING_KEY).expect("test signing keypair").verifying_key.to_string();
    let verify = store_command(&fixture, &["verify", "--trusted-public-keys", &verifying_key]);
    assert_eq!(verify.status.code(), expected["store_verify_exit_code"].as_i64()
        .map(|code| i32::try_from(code).expect("bounded verification exit code")),
        "imported NAR or signature rejected: {}", String::from_utf8_lossy(&verify.stderr));
    assert_eq!(verify.stdout.len(), usize::try_from(expected["store_verify_stdout_bytes"].as_u64()
        .expect("old matched verifier output bytes")).expect("bounded verifier stdout"));
    let verified = std::str::from_utf8(&verify.stdout).expect("human-readable signed store verification");
    let trusted = expected["store_verify_trusted_signatures"].as_u64().expect("old trusted signature count");
    let expected_row = format!("OK {selector}  trusted_signatures={trusted}/{}", signatures.len());
    let mut rows = verified.lines();
    assert_eq!(rows.next(), Some(expected_row.as_str()), "the imported output was not cryptographically checked");
    assert_eq!(expected_row, signed["signed_worker_verify_row"].as_str()
        .expect("independently verified original Worker signed output"));
    assert_eq!(rows.next(), None, "unexpected extra verification rows");
    // The original verifier's signing key already existed after a prior filtered no-op;
    // a fresh client's first verification can print a local key-creation notice first.
    let verify_diagnostic = std::str::from_utf8(&verify.stderr).expect("store verification diagnostics");
    assert_eq!(verify_diagnostic.lines().last(),
        Some(expected["store_verify_stderr_summary"].as_str().expect("old matched-path summary")));
    assert_eq!(fs::read_dir(&fixture.store).expect("physical client store").count(),
        usize::try_from(expected["client_store_paths"].as_u64().expect("old store paths")).expect("bounded paths"));
}

#[test]
fn physical_production_upload_quota_rejects_before_ticket_or_transfer() {
    assert_rejected_physical_production("quota", 1, None);
}

#[test]
fn physical_production_untrusted_key_rejects_after_input_without_client_output_claim() {
    let old: Value = serde_json::from_slice(ORIGINAL_PRODUCTION).expect("independent untrusted result");
    let key = old["untrusted"]["trusted_key_override"].as_str().expect("original untrusted key");
    assert_rejected_physical_production("untrusted", 16 * 1024 * 1024, Some(key));
}

#[test]
fn physical_failed_adapter_preserves_original_post_dispatch_failure_semantics() {
    let old: Value = serde_json::from_slice(ORIGINAL_FAILED_ADAPTER).expect("independent original spawn failure");
    let signed: Value = serde_json::from_slice(ORIGINAL_SIGNED_SUCCESSOR).expect("independent signed production output");
    let root = tempfile::tempdir().expect("private failed adapter root");
    let fixture = production_fixture(root.path());
    assert_eq!(format!("{:x}", Sha256::digest(&fixture.payload)),
        old["payload_sha256"].as_str().expect("old physical failure payload SHA-256"));
    let (ticket_id, bearer) = production_ticket(&fixture, 16 * 1024 * 1024);
    assert_eq!(production_ticket_uses(&fixture, &ticket_id), old["ticket_uses_before"]);
    let absent = root.path().join("absent-worker");
    assert!(!absent.exists(), "failure must reach a genuinely absent program");
    let verifying_key = crunch_build::load_keypair(SIGNING_KEY).expect("test signing keypair").verifying_key.to_string();
    let output = run_production(root.path(), &fixture, &ticket_id, &bearer, Some(&verifying_key), Some(&absent));
    assert_eq!(output.status.code(), old["exit_code"].as_i64().map(|code| i32::try_from(code).expect("failure exit code")));
    assert_eq!(output.stdout.len(), usize::try_from(old["stdout_bytes"].as_u64().expect("old failure stdout"))
        .expect("bounded failure stdout"));
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("spawning stdio remote child")
        && diagnostic.contains("absent-worker")
        && (diagnostic.contains("No such file or directory") || diagnostic.contains("os error 2")),
        "real failure must be the transport child spawn ENOENT, not earlier admission");
    assert_eq!(production_ticket_uses(&fixture, &ticket_id), old["ticket_uses_after"]);

    let coordinator = coordinator_state(&fixture);
    let job = single_coordinator_job(&coordinator);
    let attempt = &job["current_attempt"];
    assert_eq!(job["phase"], old["coordinator_phase"]);
    assert_eq!(attempt["phase"], old["attempt_phase"]);
    assert_eq!(attempt["fence_generation"], old["fence_generation"]);
    assert_eq!(attempt["attempts_started"], old["attempts_started"]);
    assert_eq!(job["last_attempt_reason_code"], old["last_attempt_reason_code"]);
    assert_eq!(job["output_admission_completed"], old["output_admission_completed"]);
    assert_eq!(coordinator["live_output_claims"].as_object().expect("live claims").len(),
        usize::try_from(old["live_output_claims"].as_u64().expect("old live claims")).expect("bounded claims"));
    let event = physical_observability_log(&fixture, attempt);
    let old_logs = old["durable_logs"].as_array().expect("independent original durable log");
    assert_eq!(old_logs.len(), 1);
    assert_eq!(event, old_logs[0], "real failed transport observation lost original event ordering");

    assert_eq!(fs::read_dir(&fixture.store).expect("client store").count(),
        usize::try_from(old["client_store_paths"].as_u64().expect("old client store count"))
            .expect("bounded store count"));
    assert_eq!(fs::read(fixture.store.join(PRODUCTION_INPUT.rsplit('/').next().unwrap())).expect("client input"),
        fixture.payload);
    let logical = signed["logical_output_path"].as_str().expect("original physical signed output logical path");
    let selector = logical.rsplit('/').next().expect("output selector");
    assert!(!fixture.store.join(selector).exists(), "failed attempt exported an output");
    let info = store_command(&fixture, &["info", selector]);
    assert_eq!(info.status.code(), old["store_info_exit_code"].as_i64()
        .map(|code| i32::try_from(code).expect("bounded missing PathInfo exit")));
    assert_eq!(info.stdout.len(), usize::try_from(old["store_info_stdout_bytes"].as_u64()
        .expect("old absent PathInfo stdout")).expect("bounded PathInfo stdout"));
    assert!(!fixture.state.join("remote-workers").exists(), "missing adapter launched a Worker");
    assert!(!fixture.state.join("remote-exports").exists(), "failed adapter exported an output");
}
