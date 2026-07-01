use std::fs;

use assert_cmd::Command;
use serde_json::Value;

const FRAME_HEADER_BYTES: usize = 4;
const PROTOCOL_ALPN: &str = "mantle-remote-build/1";
const PROTOCOL_VERSION: u32 = 1;
const TEST_NOW_UNIX_S: u64 = 1;
const TEST_EXPIRY_UNIX_S: u64 = 100;
const TEST_BUILD_TIME_LIMIT_SECS: u64 = 60;
const TEST_TICKET_MAX_BUILD_TIME_SECS: u64 = 600;
const TEST_UPLOAD_BYTES: u64 = 10;
const TEST_MAX_UPLOAD_BYTES: u64 = 1_000;
const BLAKE3_HEX_LENGTH_CHARS: usize = 64;

#[test]
fn remote_serve_stdio_once_exchanges_frames_and_redeems_ticket() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_ticket_state(temp.path());
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
            "builder-key",
        ])
        .write_stdin(input)
        .output()
        .expect("run stdio-once serve");

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty(), "stdio protocol diagnostics stayed quiet");

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
    assert!(frames.iter().any(|frame| frame_kind(frame) == "output-transfer-done"));
    assert_eq!(build_finished["result"]["outputs"][0]["name"], "out");
    assert_eq!(build_finished["result"]["outputs"][0]["path_info_signing_key_id"], "builder-key");
    assert_eq!(
        build_finished["result"]["outputs"][0]["artifact_attestation_digest_blake3"]
            .as_str()
            .expect("artifact digest string")
            .len(),
        BLAKE3_HEX_LENGTH_CHARS
    );
    assert_eq!(ticket_state["tickets"]["ticket-1"]["uses_remaining"], 0);
}

#[test]
fn remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames() {
    let temp = tempfile::tempdir().expect("tempdir");
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
            "builder-key",
        ])
        .write_stdin(input)
        .output()
        .expect("run stdio-once serve");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown-remote-ticket-ticket-1"));
}

fn write_ticket_state(state_dir: &std::path::Path) {
    let ticket_dir = state_dir.join("remote-builders");
    fs::create_dir_all(&ticket_dir).expect("ticket dir");
    let ticket_state = serde_json::json!({
        "tickets": {
            "ticket-1": {
                "id": "ticket-1",
                "display_name": "ticket",
                "secret": "secret-1",
                "created_unix_s": TEST_NOW_UNIX_S,
                "expires_unix_s": TEST_EXPIRY_UNIX_S,
                "uses_remaining": 1,
                "max_build_time_secs": TEST_TICKET_MAX_BUILD_TIME_SECS,
                "max_upload_bytes": TEST_MAX_UPLOAD_BYTES,
                "bound_client_endpoint": null,
                "revoked": false
            }
        }
    });
    fs::write(
        ticket_dir.join("tickets.json"),
        serde_json::to_vec_pretty(&ticket_state).expect("ticket state serializes"),
    )
    .expect("write ticket state");
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
                "ticket_id": "ticket-1",
                "secret": "secret-1",
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
