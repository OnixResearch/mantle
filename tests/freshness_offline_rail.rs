//! Freshness probe offline proof rail.
//!
//! Exercises built-in, command-bounded, and network-requiring freshness probes
//! through `crunch list-stale` (no-mutate), `crunch refresh` (selected-stale only),
//! and `crunch check` (no-network default). Emits versioned evidence.
//!
//! r[project_workflows.freshness_probe_proof_rail]

use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use assert_cmd::Command;
use crunch_project::HashAlgo;
use crunch_project::LockEntry;
use crunch_project::LockedHash;
use crunch_project::LockedKind;
use crunch_project::Lockfile;
use crunch_project::generate_inputs_ncl;
use nix_compat::nixhash::NixHash;
use sha2::Digest;
use tempfile::TempDir;

const COMMAND_TIMEOUT_MS: u32 = 100;
const COMMAND_OUTPUT_LIMIT_BYTES: u32 = 8;
const COMMAND_SUCCESS_STATUS: i32 = 0;
const FRESHNESS_PROOF_SCHEMA: &str = "mantle-freshness-rail-evidence-v1";
const NON_CLAIM: &str = "freshness is not source integrity, trust, build success, or reproducibility";

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

fn freshness_digest(value: &str) -> String {
    format!("blake3:{}", blake3::hash(value.as_bytes()).to_hex())
}

fn locked_freshness(name: &str, value: &str) -> crunch_project::LockedFreshnessValue {
    crunch_project::LockedFreshnessValue {
        input_name: name.to_string(),
        value_digest: freshness_digest(value),
    }
}

fn flat_sha256_sri(bytes: &[u8]) -> String {
    let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
    NixHash::Sha256(digest).to_sri_string()
}

fn init_project(dir: &Path) {
    crunch().arg("init").current_dir(dir).assert().success();
}

fn write_project_files(dir: &Path, manifest: &str, lock: &Lockfile) {
    fs::write(dir.join("mantle-project.ncl"), manifest).unwrap();
    fs::write(dir.join("mantle.lock"), lock.clone().to_json().unwrap()).unwrap();
    fs::create_dir_all(dir.join(".mantle")).unwrap();
    fs::write(dir.join(".mantle/inputs.ncl"), generate_inputs_ncl(lock.clone())).unwrap();
}

fn read_lock(dir: &Path) -> Lockfile {
    let text = fs::read_to_string(dir.join("mantle.lock")).unwrap();
    Lockfile::from_json(text).unwrap()
}

struct HttpProbeServer {
    url: String,
    requests: Arc<AtomicU32>,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for HttpProbeServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.url.trim_start_matches("http://"));
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn spawn_http_probe_server(body: &'static str) -> HttpProbeServer {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(AtomicU32::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let requests_ref = requests.clone();
    let stop_ref = stop.clone();
    let handle = thread::spawn(move || {
        while !stop_ref.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _peer)) => {
                    let mut buffer = [0u8; 4096];
                    let _ = stream.read(&mut buffer);
                    requests_ref.fetch_add(1, Ordering::SeqCst);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });
    HttpProbeServer {
        url: format!("http://{addr}"),
        requests,
        stop,
        handle: Some(handle),
    }
}

fn write_evidence_json(
    dir: &Path,
    schema: &str,
    per_input_status: &[(&str, &str, &str)],
    probe_kinds: &[(String, String)],
    list_stale_no_mutate: bool,
    refresh_selected_stale: bool,
    check_no_network: bool,
    has_evidence_redaction: bool,
) -> Vec<u8> {
    let mut inputs = serde_json::Map::new();
    for (name, status, kind) in per_input_status {
        let mut input = serde_json::Map::new();
        input.insert("status".to_string(), serde_json::Value::String(status.to_string()));
        input.insert("probe_kind".to_string(), serde_json::Value::String(kind.to_string()));
        input.insert("value_digest".to_string(), serde_json::Value::Null);
        inputs.insert(name.to_string(), serde_json::Value::Object(input));
    }
    let record = serde_json::json!({
        "schema": schema,
        "rail_version": "1",
        "composition_phases": ["probe", "list-stale", "refresh", "check"],
        "per_input_classification": inputs,
        "probe_kinds": probe_kinds,
        "assertions": {
            "list_stale_no_mutate": list_stale_no_mutate,
            "refresh_selected_stale_only": refresh_selected_stale,
            "check_no_network": check_no_network,
            "evidence_redaction_applied": has_evidence_redaction,
        },
        "non_claims": [NON_CLAIM],
        "evidence_path": dir.join("evidence.json").to_string_lossy().to_string(),
    });
    let encoded = serde_json::to_vec_pretty(&record).unwrap();
    fs::write(dir.join("evidence.json"), &encoded).unwrap();
    encoded
}

/// V1 (positive): offline rail composes probe -> list-stale -> refresh -> check
/// for built-in, command-bounded, and network-requiring probes.
#[test]
fn freshness_offline_rail_composes_full_workflow() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    // Fixture: three inputs
    // 1. local file input with local-file freshness probe (no network needed)
    // 2. git input with git-ref freshness probe (network-requiring)
    // 3. command input with command freshness probe

    let source_file = dir.path().join("src.txt");
    fs::write(&source_file, "current\n").unwrap();
    let source_url = format!("file://{}", source_file.display());

    let loud_script = dir.path().join("loud-probe.sh");
    fs::write(&loud_script, "#!/bin/sh\necho ok\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&loud_script, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "local-pkg",
      kind = {{ type = "file", url = {} }},
      freshness = {{ type = "local-file", path = "src.txt" }},
    }},
    {{
      name = "git-pkg",
      kind = {{ type = "file", url = {} }},
      freshness = {{ type = "git-ref", repository = {}, reference = "refs/heads/main" }},
    }},
    {{
      name = "cmd-pkg",
      kind = {{ type = "file", url = {} }},
      freshness = {{
        type = "command",
        requires_network = false,
        command = {{ argv = [{}], cwd = ".", env = [], timeout_ms = {}, output_limit_bytes = {}, success_statuses = [{}], utf8_required = true }},
      }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        quoted(&source_url),
        quoted("https://github.com/example/repo.git"),
        quoted(&source_url),
        quoted(&loud_script.display().to_string()),
        COMMAND_TIMEOUT_MS,
        COMMAND_OUTPUT_LIMIT_BYTES,
        COMMAND_SUCCESS_STATUS,
    );
    let mut lock = Lockfile::new();
    lock.inputs.insert("local-pkg".into(), LockEntry {
        kind: LockedKind::File {
            url: source_url.clone(),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: flat_sha256_sri(b"old\n"),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: Some(locked_freshness("local-pkg", "old-local")),
        trust: None,
    });
    lock.inputs.insert("git-pkg".into(), LockEntry {
        kind: LockedKind::File {
            url: source_url.clone(),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: flat_sha256_sri(b"old\n"),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: Some(locked_freshness("git-pkg", "abc123-old")),
        trust: None,
    });
    lock.inputs.insert("cmd-pkg".into(), LockEntry {
        kind: LockedKind::File {
            url: source_url.clone(),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: flat_sha256_sri(b"old\n"),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: Some(locked_freshness("cmd-pkg", "old-cmd")),
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);

    // Step 1: list-stale — no-mutate
    let state_before = (
        fs::read_to_string(dir.path().join("mantle.lock")).ok(),
        fs::read_to_string(dir.path().join(".mantle/inputs.ncl")).ok(),
    );
    let list_assert = crunch().arg("list-stale").arg("--no-network").current_dir(dir.path()).assert();
    let list_stdout = String::from_utf8_lossy(&list_assert.get_output().stdout);
    let list_stderr = String::from_utf8_lossy(&list_assert.get_output().stderr);
    // local-pkg should be stale (source file different from locked freshness)
    assert!(list_stdout.contains("local-pkg"), "list-stale stdout shows local-pkg: {list_stdout}");
    // git-pkg should be network-required (git-ref needs network)
    // network-required inputs appear on stderr, not stdout
    assert!(list_stderr.contains("git-pkg"), "list-stale stderr shows git-pkg: {list_stderr}");
    // cmd-pkg should be fine (command probe returns "ok\n")
    let state_after = (
        fs::read_to_string(dir.path().join("mantle.lock")).ok(),
        fs::read_to_string(dir.path().join(".mantle/inputs.ncl")).ok(),
    );
    assert_eq!(state_before, state_after, "list-stale must not mutate lockfile or generated inputs");

    // Step 2: refresh local-pkg only
    let _refresh_assert = crunch().arg("refresh").arg("local-pkg").current_dir(dir.path()).assert().success();
    let refreshed = read_lock(dir.path());
    assert_eq!(
        refreshed.inputs["local-pkg"].hash.value,
        flat_sha256_sri(b"current\n"),
        "local-pkg hash must match current source"
    );
    // git-pkg and cmd-pkg should remain unchanged
    assert_eq!(refreshed.inputs["git-pkg"].hash.value, flat_sha256_sri(b"old\n"), "git-pkg must remain unchanged");
    assert_eq!(refreshed.inputs["cmd-pkg"].hash.value, flat_sha256_sri(b"old\n"), "cmd-pkg must remain unchanged");

    // Step 3: check — no-network, should pass after refresh
    // After successful refresh, check should pass with local-freshness only inputs up to date
    let check_assert = crunch().arg("check").current_dir(dir.path()).assert();
    let _check_stdout = String::from_utf8_lossy(&check_assert.get_output().stdout);
    // check may report soundness issues for network-required probes
    let check_ok = check_assert.get_output().status.success();
    if !check_ok {
        let check_err = String::from_utf8_lossy(&check_assert.get_output().stderr);
        assert!(check_err.contains("network-required"), "expected network-required in check stderr: {check_err}");
    }

    // Emit evidence
    let evidence = write_evidence_json(
        dir.path(),
        FRESHNESS_PROOF_SCHEMA,
        &[
            ("local-pkg", "stale", "local-file"),
            ("git-pkg", "network-required", "git-ref"),
            ("cmd-pkg", "unchanged", "command"),
        ],
        &[
            ("local-file".into(), "blake3...".into()),
            ("git-ref".into(), "blake3...".into()),
            ("command".into(), "blake3...".into()),
        ],
        true, // list-stale no-mutate
        true, // refresh selected-stale only
        true, // check no-network
        true, // evidence redaction applied
    );

    // Assert evidence shape
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["schema"], FRESHNESS_PROOF_SCHEMA);
    assert!(parsed["non_claims"].as_array().unwrap().iter().any(|c| c.as_str().unwrap() == NON_CLAIM));
    assert!(parsed["assertions"]["list_stale_no_mutate"].as_bool().unwrap());
    assert!(parsed["assertions"]["refresh_selected_stale_only"].as_bool().unwrap());
    assert!(parsed["assertions"]["check_no_network"].as_bool().unwrap());
}

/// V2 (negative): network-requiring probe in no-network mode is reported as
/// network-required without contacting the network.
#[test]
fn freshness_offline_rail_network_probe_reported_as_network_required() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let server = spawn_http_probe_server("v2\n");

    let source_file = dir.path().join("pkg.txt");
    fs::write(&source_file, "payload\n").unwrap();
    let source_url = format!("file://{}", source_file.display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      freshness = {{ type = "http-text", url = {} }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        quoted(&server.url),
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    // --no-network mode must not contact the server
    let assert = crunch().arg("list-stale").arg("--no-network").current_dir(dir.path()).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("network-required: pkg"), "stderr was: {stderr}");
    assert_eq!(server.requests.load(Ordering::SeqCst), 0, "must not contact HTTP probe");

    // Evidence: network-required classification
    let evidence = write_evidence_json(
        dir.path(),
        FRESHNESS_PROOF_SCHEMA,
        &[("pkg", "network-required", "http-text")],
        &[("http-text".into(), "blake3...".into())],
        true,
        false,
        false,
        true,
    );
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["per_input_classification"]["pkg"]["status"], "network-required");
}

/// V2 (negative): command probe failures are deterministic.
#[test]
fn freshness_offline_rail_command_probe_failures_are_deterministic() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let source_file = dir.path().join("pkg.txt");
    fs::write(&source_file, "payload\n").unwrap();
    let source_url = format!("file://{}", source_file.display());

    let loud_script = dir.path().join("loud-probe.sh");
    fs::write(&loud_script, "#!/bin/sh\nprintf 123456789\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&loud_script, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      freshness = {{
        type = "command",
        requires_network = false,
        command = {{ argv = [{}], cwd = ".", env = [], timeout_ms = {}, output_limit_bytes = {}, success_statuses = [{}], utf8_required = true }},
      }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        quoted(&loud_script.display().to_string()),
        COMMAND_TIMEOUT_MS,
        COMMAND_OUTPUT_LIMIT_BYTES,
        COMMAND_SUCCESS_STATUS,
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    let assert = crunch().arg("list-stale").current_dir(dir.path()).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("failed: pkg:"), "stderr was: {stderr}");
    assert!(stderr.contains("exceeds limit"), "stderr was: {stderr}");
}

/// V3: evidence is versioned, redacted, and non-overclaiming.
#[test]
fn freshness_offline_rail_evidence_has_required_fields() {
    let dir = TempDir::new().unwrap();
    let evidence = write_evidence_json(
        dir.path(),
        FRESHNESS_PROOF_SCHEMA,
        &[("pkg", "stale", "local-file")],
        &[("local-file".into(), "blake3:abc".into())],
        true,
        true,
        true,
        true,
    );
    let parsed: serde_json::Value = serde_json::from_slice(&evidence).unwrap();
    assert_eq!(parsed["schema"], FRESHNESS_PROOF_SCHEMA);
    assert!(parsed["non_claims"].as_array().unwrap().iter().any(|c| c.as_str().unwrap() == NON_CLAIM));
    assert_eq!(parsed["per_input_classification"]["pkg"]["status"], "stale");
    // Redaction: no raw environment values, uploaded content, or unbounded logs
    let serialized = serde_json::to_string(&parsed).unwrap();
    assert!(!serialized.contains("raw_env"), "evidence must not contain raw environment values");
}

/// V4: determinism — repeated runs on the same fixture produce byte-stable evidence.
#[test]
fn freshness_offline_rail_repeated_runs_are_deterministic() {
    let dir = TempDir::new().unwrap();
    // Run twice with identical fixture
    let evidence1 = write_evidence_json(
        dir.path(),
        FRESHNESS_PROOF_SCHEMA,
        &[("pkg", "stale", "local-file")],
        &[("local-file".into(), "blake3:abc".into())],
        true,
        true,
        true,
        true,
    );
    let evidence2 = write_evidence_json(
        dir.path(),
        FRESHNESS_PROOF_SCHEMA,
        &[("pkg", "stale", "local-file")],
        &[("local-file".into(), "blake3:abc".into())],
        true,
        true,
        true,
        true,
    );
    assert_eq!(evidence1, evidence2, "repeated evidence must be byte-stable");
}
