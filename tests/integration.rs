//! End-to-end integration tests for the crunch binary.
//!
//! These tests invoke the compiled `crunch` binary via `assert_cmd` and
//! check stdout, stderr, and exit codes.

use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::JoinHandle;
use std::thread::{self};
use std::time::Duration;

use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should be built")
}

fn crunch_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const RUNTIME_FINGERPRINT_PREFIX: &str = "mantle-runtime-fingerprint";
const CLAP_USAGE_ERROR_CODE: i32 = 2;

fn runtime_fingerprint_payload(stderr: &str) -> serde_json::Value {
    let line = stderr
        .lines()
        .find(|line| line.starts_with(RUNTIME_FINGERPRINT_PREFIX))
        .unwrap_or_else(|| panic!("stderr should include runtime fingerprint: {stderr}"));
    let payload = line
        .strip_prefix(&format!("{RUNTIME_FINGERPRINT_PREFIX} "))
        .expect("fingerprint line should include JSON payload");
    serde_json::from_str(payload).unwrap_or_else(|err| panic!("fingerprint should be JSON: {err}\npayload: {payload}"))
}

#[derive(Debug, Clone)]
enum HttpFixtureResponse {
    Fixed {
        status_line: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    },
}

struct HttpFixtureServer {
    base_url: String,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl HttpFixtureServer {
    fn serve_cache_dir(cache_dir: &Path) -> Self {
        let mut routes = BTreeMap::<String, HttpFixtureResponse>::new();
        routes.insert("/nix-cache-info".to_string(), HttpFixtureResponse::Fixed {
            status_line: "HTTP/1.1 200 OK".to_string(),
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: std::fs::read(cache_dir.join("nix-cache-info")).unwrap(),
        });
        for entry in std::fs::read_dir(cache_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "narinfo") {
                routes.insert(format!("/{}", entry.file_name().to_string_lossy()), HttpFixtureResponse::Fixed {
                    status_line: "HTTP/1.1 200 OK".to_string(),
                    headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                    body: std::fs::read(path).unwrap(),
                });
            }
        }
        for entry in std::fs::read_dir(cache_dir.join("nar")).unwrap() {
            let entry = entry.unwrap();
            routes.insert(format!("/nar/{}", entry.file_name().to_string_lossy()), HttpFixtureResponse::Fixed {
                status_line: "HTTP/1.1 200 OK".to_string(),
                headers: vec![("Content-Type".to_string(), "application/octet-stream".to_string())],
                body: std::fs::read(entry.path()).unwrap(),
            });
        }

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let routes = Arc::new(routes);
        let routes_thread = Arc::clone(&routes);
        let handle = thread::spawn(move || {
            while !stop_thread.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => serve_http_fixture_request(&mut stream, &routes_thread),
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => panic!("accept failed: {err}"),
                }
            }
        });

        Self {
            base_url: format!("http://{addr}"),
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for HttpFixtureServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let url: url::Url = self.base_url.parse().unwrap();
        let host = url.host_str().unwrap();
        let port = url.port_or_known_default().unwrap();
        let _ = TcpStream::connect((host, port));
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

fn serve_http_fixture_request(stream: &mut TcpStream, routes: &BTreeMap<String, HttpFixtureResponse>) {
    let mut buffer = [0u8; 4096];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let path = request.lines().next().and_then(|line| line.split_whitespace().nth(1)).unwrap_or("/").to_string();
    let response = routes.get(&path).cloned().unwrap_or(HttpFixtureResponse::Fixed {
        status_line: "HTTP/1.1 404 Not Found".to_string(),
        headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
        body: b"not found".to_vec(),
    });

    let HttpFixtureResponse::Fixed {
        status_line,
        headers,
        body,
    } = response;
    let mut response_bytes =
        format!("{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n", body.len()).into_bytes();
    for (name, value) in headers {
        response_bytes.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
    }
    response_bytes.extend_from_slice(b"\r\n");
    response_bytes.extend_from_slice(&body);
    stream.write_all(&response_bytes).unwrap();
    stream.flush().unwrap();
}

async fn open_http_pull_test_store(dir: &Path) -> crunch_store::StoreHandle {
    let state_dir = dir.join("state");
    let output_dir = dir.join("output");
    std::fs::create_dir_all(&state_dir).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir,
        output_dir,
        remote_cache_url: None,
        fallback_mode: crunch_store::StoreFallbackMode::Practical,
        store_dir: "/mantle/store".to_string(),
    })
    .await
    .unwrap()
}

async fn render_http_pull_nar_bytes(handle: &crunch_store::StoreHandle, node: &snix_castore::Node) -> Vec<u8> {
    use tokio::io::AsyncReadExt;

    let (mut reader, writer) = tokio::io::duplex(64 * 1024);
    let node = node.clone();
    let blob_service = handle.blob_service();
    let directory_service = handle.directory_service();
    let write_task =
        tokio::spawn(async move { snix_store::nar::write_nar(writer, &node, blob_service, directory_service).await });
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await.unwrap();
    write_task.await.unwrap().unwrap();
    bytes
}

async fn make_http_pull_cache_fixture(work_dir: &Path) -> (PathBuf, String, String) {
    use nix_compat::store_path::StorePath;
    use snix_castore::Node;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::PathInfoService;
    use tokio::io::AsyncWriteExt;

    let store = open_http_pull_test_store(&work_dir.join("seed-store")).await;
    let mut writer = store.blob_service().open_write().await;
    let content = b"http pull integration content";
    writer.write_all(content).await.unwrap();
    let blob_digest = writer.close().await.unwrap();
    let node = Node::File {
        digest: blob_digest,
        size: content.len() as u64,
        executable: false,
    };
    let nar_bytes = render_http_pull_nar_bytes(&store, &node).await;
    let nar_sha256: [u8; 32] = {
        use sha2::Digest;
        sha2::Sha256::digest(&nar_bytes).into()
    };
    let store_path = StorePath::from_name_and_digest_fixed("http-pull-cli", [7u8; 20]).unwrap();
    let mut path_info = PathInfo {
        store_path: store_path.clone(),
        node,
        references: vec![],
        nar_sha256,
        nar_size: nar_bytes.len() as u64,
        signatures: vec![],
        deriver: None,
        ca: None,
    };
    let (keypair, _) = crunch_build::signing::generate_keypair();
    let store_path_ref: nix_compat::store_path::StorePathRef = path_info.store_path.as_ref();
    let fingerprint = nix_compat::narinfo::fingerprint_with_store_dir(
        &store_path_ref,
        &path_info.nar_sha256,
        path_info.nar_size,
        std::iter::empty::<&nix_compat::store_path::StorePathRef>(),
        "/mantle/store",
    );
    path_info.signatures.push(keypair.signing_key.sign(fingerprint.as_bytes()).to_owned());
    store.pathinfo_service().put(path_info.clone()).await.unwrap();

    let cache_dir = work_dir.join("cache");
    crunch_store::export_paths_to_cache_dir(&store, &[path_info.clone()], &cache_dir, &crunch_store::PushOptions {
        trust_unsigned: false,
    })
    .await
    .unwrap();

    (
        cache_dir,
        path_info.store_path.to_absolute_path_with_prefix("/mantle/store"),
        keypair.verifying_key.to_string(),
    )
}

// ── Phase 2: Eval tests ─────────────────────────────────────────

#[test]
fn eval_simple_derivation_prints_json() {
    crunch_cmd()
        .arg("eval")
        .arg(fixture("simple.ncl"))
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\""))
        .stdout(predicate::str::contains("simple-test"));
}

#[test]
fn eval_multi_derivation_prints_both() {
    crunch_cmd()
        .arg("eval")
        .arg(fixture("multi.ncl"))
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn eval_output_is_valid_json() {
    let output = crunch_cmd().arg("eval").arg(fixture("simple.ncl")).output().expect("should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "stdout should be valid JSON: {stdout}");
}

#[test]
fn eval_still_uses_json_export_after_build_path_switch() {
    let output = crunch_cmd().arg("eval").arg(fixture("simple.ncl")).output().expect("should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should stay JSON");
    assert_eq!(parsed["name"], "simple-test");
    assert_eq!(parsed["builder"], "/bin/sh");
    assert_eq!(parsed["args"][0], "-c");
    assert_eq!(parsed["args"][1], "echo hello > $out");
}

#[test]
fn eval_invalid_nickel_exits_2() {
    crunch_cmd()
        .arg("eval")
        .arg(fixture("invalid.ncl"))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error"));
}

#[test]
fn eval_nonexistent_file_exits_2() {
    crunch_cmd()
        .arg("eval")
        .arg("/nonexistent/path/to/file.ncl")
        .assert()
        .code(2)
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn eval_json_flag_emits_json_error() {
    let output = crunch_cmd().arg("--json").arg("eval").arg(fixture("invalid.ncl")).output().expect("should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(stderr.trim());
    assert!(parsed.is_ok(), "stderr should be valid JSON: {stderr}");

    let obj = parsed.unwrap();
    assert_eq!(obj["code"], 2);
    assert_eq!(obj["kind"], "eval");
    assert!(obj["error"].is_string());
}

#[test]
fn eval_with_import_path_flag() {
    let lib_dir = tempfile::tempdir().unwrap();
    std::fs::write(lib_dir.path().join("extra.ncl"), r#"{ val = 42 }"#).unwrap();

    let main_dir = tempfile::tempdir().unwrap();
    std::fs::write(
        main_dir.path().join("app.ncl"),
        r#"let crunch = import "lib.ncl" in
let extra = import "extra.ncl" in
{
  name = "with-import",
  builder = "/bin/sh",
  args = ["-c", "echo > $out"],
  inputs = [],
  env = { EXTRA = std.string.from_number extra.val },
} | crunch.Derivation"#,
    )
    .unwrap();

    crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(lib_dir.path())
        .arg(main_dir.path().join("app.ncl"))
        .assert()
        .success()
        .stdout(predicate::str::contains("with-import"));
}

// ── Phase 3: Bootstrap tests ────────────────────────────────────

#[test]
fn bootstrap_creates_seed_file() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.ncl");

    let result = crunch_cmd().arg("bootstrap").arg("-o").arg(&seed).arg("bash").output().expect("should run");

    // Bootstrap might fail if nix isn't installed — skip gracefully
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        if stderr.contains("failed to run nix") || stderr.contains("No such file") {
            eprintln!("skipping bootstrap test: nix not available");
            return;
        }
        panic!("bootstrap failed unexpectedly: {}", String::from_utf8_lossy(&result.stderr));
    }

    assert!(seed.exists(), "seed.ncl should be created");
    let content = std::fs::read_to_string(&seed).unwrap();
    assert!(content.contains("bash"), "seed should contain bash entry");
    assert!(content.contains("/nix/store/"), "seed should contain store paths");
}

#[test]
fn bootstrap_seed_is_importable() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.ncl");

    let result = crunch_cmd().arg("bootstrap").arg("-o").arg(&seed).arg("bash").output().expect("should run");

    if !result.status.success() {
        eprintln!("skipping: bootstrap failed (nix not available?)");
        return;
    }

    // Try to eval a file that imports the generated seed
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in
{
  name = "seed-test",
  builder = "%{seed.bash}/bin/bash",
  args = ["-c", "echo > $out"],
  inputs = [seed.bash],
} | crunch.Derivation"#,
    )
    .unwrap();

    crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg(dir.path().join("test.ncl"))
        .assert()
        .success()
        .stdout(predicate::str::contains("seed-test"));
}

// ── Phase 4: Build tests (Linux-only) ──────────────────────────

/// Check if we can actually build (need bwrap + sandbox shell + nix store).
/// Shared by build_tests and shell_build_tests.
#[cfg(target_os = "linux")]
fn can_build() -> bool {
    if !std::path::Path::new("/nix/store").exists() {
        return false;
    }
    if std::process::Command::new("bwrap").arg("--version").output().is_err() {
        return false;
    }
    // Verify the sandbox shell is available. The crunch binary at runtime
    // checks SNIX_BUILD_SANDBOX_SHELL env, then compile-time default, then
    // discovers busybox-static in /nix/store. If all fail, builds break.
    // Mirror that discovery here so we skip instead of failing cryptically.
    if let Ok(shell) = std::env::var("SNIX_BUILD_SANDBOX_SHELL")
        && shell != "/bin/sh"
    {
        return std::path::Path::new(&shell).is_file();
    }
    // Check common static busybox locations
    for candidate in ["/run/current-system/sw/bin/busybox-static", "/bin/busybox.static"] {
        if std::path::Path::new(candidate).is_file() {
            return true;
        }
    }
    // Scan /nix/store for busybox-static (limited)
    if let Ok(entries) = std::fs::read_dir("/nix/store") {
        let mut count = 0u32;
        for entry in entries.flatten() {
            count = count.saturating_add(1);
            if count > 50_000 {
                break;
            }
            let name = entry.file_name();
            if name.to_string_lossy().contains("busybox-static") {
                let bin = entry.path().join("bin/busybox");
                if bin.is_file() {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
mod build_tests {
    use super::*;

    #[test]
    fn build_trivial_derivation() {
        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        // Use /bin/sh as builder (available in bwrap via --bind).
        // Store-path builders (e.g., seed.bash) require the full closure
        // in inputs, which bootstrap doesn't resolve yet.
        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("hello.ncl"),
            r#"let crunch = import "lib.ncl" in
{
  name = "hello-e2e",
  builder = "/bin/sh",
  args = ["-c", "echo hello > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        crunch_cmd()
            .arg("--store")
            .arg(store.path())
            .arg("build")
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("hello.ncl"))
            .assert()
            .success()
            .stdout(predicate::str::contains("hello-e2e"));
    }

    #[test]
    fn build_writes_log_file() {
        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();

        std::fs::write(
            dir.path().join("logged.ncl"),
            r#"let crunch = import "lib.ncl" in
{
  name = "logged-build",
  builder = "/bin/sh",
  args = ["-c", "echo 'log test output' > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        crunch_cmd()
            .env("CRUNCH_LOG_DIR", log_dir.path())
            .arg("--store")
            .arg(store.path())
            .arg("build")
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("logged.ncl"))
            .assert()
            .success();

        // Check that a log file was written
        let logs: Vec<_> = std::fs::read_dir(log_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
            .collect();
        assert!(!logs.is_empty(), "should have written at least one log file");

        let content = std::fs::read_to_string(logs[0].path()).unwrap();
        assert!(content.contains("# crunch build log"), "log should have header");
        assert!(content.contains("# status: success"), "log should show success");
        assert!(content.contains("logged-build"), "log should name the derivation");
    }

    #[test]
    fn build_json_success_emits_machine_readable_report() {
        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("json-success.ncl"),
            r#"let crunch = import "lib.ncl" in
{
  name = "json-success",
  builder = "/bin/sh",
  args = ["-c", "echo ok > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        let output = crunch_cmd()
            .arg("--json")
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .arg("build")
            .arg("--strict-hermetic")
            .arg("--no-substitute")
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("json-success.ncl"))
            .output()
            .unwrap();

        assert!(output.status.success(), "build should succeed: {}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8(output.stdout).unwrap();
        let report: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON report");
        assert_eq!(report["schema"], "crunch-build-report-v1");
        assert_eq!(report["hermeticity_mode"], "strict");
        assert_eq!(report["hermeticity_audit_events"], serde_json::json!([]));
        assert_eq!(report["counts"]["succeeded_total"], 1);
        assert_eq!(report["counts"]["failed_total"], 0);
        assert_eq!(report["outcomes"][0]["label"], "json-success");
        assert_eq!(report["outcomes"][0]["outputs"][0]["name"], "out");
        assert!(report["outcomes"][0]["outputs"][0]["path"].as_str().unwrap().contains("json-success"));
        assert!(
            report["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"]
                .as_str()
                .unwrap()
                .contains("json-success")
        );
        let attestation_path = report["outcomes"][0]["outputs"][0]["artifact_attestation"]["path"].as_str().unwrap();
        assert!(std::path::Path::new(attestation_path).exists(), "attestation should exist: {attestation_path}");
        assert!(
            String::from_utf8_lossy(&output.stderr).trim().is_empty(),
            "stderr should stay empty on JSON success"
        );
    }

    #[test]
    fn build_json_failure_keeps_report_and_json_error_separate() {
        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("json-fail.ncl"),
            r#"let crunch = import "lib.ncl" in
{
  name = "json-fail",
  builder = "/bin/sh",
  args = ["-c", "echo boom >&2; exit 7"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        let output = crunch_cmd()
            .arg("--json")
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .arg("build")
            .arg("--no-substitute")
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("json-fail.ncl"))
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(1));
        let stdout = String::from_utf8(output.stdout).unwrap();
        let report: serde_json::Value = serde_json::from_str(&stdout).expect("stdout should be valid JSON report");
        assert_eq!(report["schema"], "crunch-build-report-v1");
        assert_eq!(report["hermeticity_mode"], "practical");
        assert_eq!(report["hermeticity_audit_events"], serde_json::json!([]));
        assert_eq!(report["counts"]["failed_total"], 1);
        assert_eq!(report["failed"][0]["label"], "json-fail");
        assert!(report["failed"][0]["error"].as_str().unwrap().contains("exit code 7"));

        let stderr = String::from_utf8_lossy(&output.stderr);
        let error: serde_json::Value =
            serde_json::from_str(stderr.trim()).expect("stderr should be a JSON error object");
        assert_eq!(error["kind"], "build");
        assert_eq!(error["code"], 1);
        assert!(error["error"].as_str().unwrap().contains("root build(s) failed"));
    }

    #[test]
    fn build_failing_builder_exits_1() {
        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();

        std::fs::write(
            dir.path().join("fail.ncl"),
            r#"let crunch = import "lib.ncl" in
{
  name = "will-fail",
  builder = "/bin/sh",
  args = ["-c", "exit 1"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        crunch_cmd()
            .arg("build")
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("fail.ncl"))
            .assert()
            .code(1);
    }

    #[test]
    fn build_persists_signed_pathinfo() {
        use futures::StreamExt;
        use snix_store::pathinfoservice::PathInfoService;
        use snix_store::pathinfoservice::RedbPathInfoService;
        use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let key_file = dir.path().join("cache.key");
        std::fs::write(
            &key_file,
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==\n",
        )
        .unwrap();

        std::fs::write(
            dir.path().join("signed.ncl"),
            r#"let crunch = import "lib.ncl" in
{
  name = "signed-cli-build",
  builder = "/bin/sh",
  args = ["-c", "echo signed > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        crunch_cmd()
            .env("CRUNCH_STATE_DIR", state.path())
            .arg("--store")
            .arg(store.path())
            .arg("build")
            .arg("--signing-key")
            .arg(&key_file)
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("signed.ncl"))
            .assert()
            .success()
            .stdout(predicate::str::contains("signed-cli-build"));

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let svc = RedbPathInfoService::new(
                "verify".to_string(),
                RedbPathInfoServiceConfig {
                    path: Some(state.path().join("pathinfo.redb")),
                    read_only: true,
                    cache_size: None,
                },
            )
            .await
            .unwrap();

            let mut stream = svc.list();
            let pi = stream.next().await.unwrap().unwrap();
            assert_eq!(pi.signatures.len(), 1, "built PathInfo should be signed");

            let keypair = crunch_build::load_keypair(
                "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
            )
            .unwrap();
            let verify = crunch_build::verify_pathinfo_signatures(&pi, &[keypair.verifying_key]);
            assert!(verify.is_trusted(), "persisted build signature should verify");
        });
    }
}

#[test]
fn store_sign_all_signs_existing_unsigned_entries() {
    use futures::StreamExt;
    use nix_compat::store_path::StorePath;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::PathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

    let state = tempfile::tempdir().unwrap();
    let key_file = state.path().join("cache.key");
    std::fs::write(
        &key_file,
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==\n",
    )
    .unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let svc = RedbPathInfoService::new("seed".to_string(), RedbPathInfoServiceConfig {
            path: Some(state.path().join("pathinfo.redb")),
            read_only: false,
            cache_size: None,
        })
        .await
        .unwrap();

        for (name, digest_byte) in [("unsigned-a", 1u8), ("unsigned-b", 2u8)] {
            let pi = PathInfo {
                store_path: StorePath::from_name_and_digest_fixed(name, [digest_byte; 20]).unwrap(),
                node: Node::Symlink {
                    target: SymlinkTarget::try_from("target").unwrap(),
                },
                references: vec![],
                nar_size: 5,
                nar_sha256: [digest_byte; 32],
                signatures: vec![],
                deriver: None,
                ca: None,
            };
            svc.put(pi).await.unwrap();
        }

        let (other_keypair, _line) = crunch_build::generate_keypair();
        let mut signed_pi = PathInfo {
            store_path: StorePath::from_name_and_digest_fixed("already-signed", [3u8; 20]).unwrap(),
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 5,
            nar_sha256: [3u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        crunch_build::sign_pathinfo(&mut signed_pi, &other_keypair.signing_key);
        svc.put(signed_pi).await.unwrap();
    });

    crunch_cmd()
        .env("CRUNCH_STATE_DIR", state.path())
        .arg("store")
        .arg("sign")
        .arg("--all")
        .arg("--signing-key")
        .arg(&key_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("SIGNED"))
        .stderr(predicate::str::contains("2 signed, 0 appended, 0 replaced, 2 total"));

    rt.block_on(async {
        let svc = RedbPathInfoService::new("verify".to_string(), RedbPathInfoServiceConfig {
            path: Some(state.path().join("pathinfo.redb")),
            read_only: true,
            cache_size: None,
        })
        .await
        .unwrap();

        let mut unsigned_count = 0u32;
        let mut already_signed_count = 0u32;
        let mut stream = svc.list();
        while let Some(result) = stream.next().await {
            let pi = result.unwrap();
            if pi.store_path.name() == "already-signed" {
                assert_eq!(pi.signatures.len(), 1, "--all should skip entries that are already signed");
                already_signed_count = already_signed_count.saturating_add(1);
                continue;
            }

            assert_eq!(pi.signatures.len(), 1, "unsigned entries should gain one signature");
            unsigned_count = unsigned_count.saturating_add(1);
        }
        assert_eq!(unsigned_count, 2);
        assert_eq!(already_signed_count, 1);
    });
}

// ── Phase 5: Error and edge cases ───────────────────────────────

#[test]
fn store_pull_http_requires_explicit_paths() {
    let store_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    crunch_cmd()
        .arg("--store")
        .arg(store_dir.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("store")
        .arg("pull")
        .arg("--from")
        .arg("https://cache.example.com")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("HTTP pull requires explicit store path selectors"));
}

#[test]
fn store_pull_http_rejects_all() {
    let store_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    crunch_cmd()
        .arg("--store")
        .arg(store_dir.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("store")
        .arg("pull")
        .arg("--from")
        .arg("https://cache.example.com")
        .arg("--all")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("--all is not supported for HTTP caches; specify paths explicitly"));
}

#[test]
fn store_pull_rejects_http_url_with_userinfo() {
    let store_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    crunch_cmd()
        .arg("--store")
        .arg(store_dir.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("store")
        .arg("pull")
        .arg("--from")
        .arg("https://user@cache.example.com")
        .arg("/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-test")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("HTTP pull source must not include URL credentials"));
}

#[test]
fn store_pull_rejects_unsupported_url_scheme() {
    let store_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    crunch_cmd()
        .arg("--store")
        .arg(store_dir.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("store")
        .arg("pull")
        .arg("--from")
        .arg("file://cache.example.com")
        .arg("/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-test")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("unsupported pull source URL scheme"));
}

#[test]
fn store_pull_http_round_trip_imports_path() {
    let work = tempfile::tempdir().unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (cache_dir, logical_store_path, trusted_public_key) =
        runtime.block_on(make_http_pull_cache_fixture(work.path()));
    let server = HttpFixtureServer::serve_cache_dir(&cache_dir);
    let store_dir = work.path().join("pull-store");
    let state_dir = work.path().join("pull-state");
    std::fs::create_dir_all(&store_dir).unwrap();
    std::fs::create_dir_all(&state_dir).unwrap();

    let output = crunch_cmd()
        .arg("--store")
        .arg(&store_dir)
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("store")
        .arg("pull")
        .arg("--from")
        .arg(&server.base_url)
        .arg("--trusted-public-keys")
        .arg(&trusted_public_key)
        .arg(&logical_store_path)
        .output()
        .expect("HTTP pull should execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "HTTP pull failed (exit {}):\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code().unwrap_or(-1),
    );
    assert!(stdout.contains("PULL "), "stdout: {stdout}");
    assert!(stderr.contains("imported=1"), "stderr: {stderr}");

    let pulled_name = logical_store_path.rsplit('/').next().unwrap();
    assert!(store_dir.join(pulled_name).exists(), "expected pulled output on disk");
}

#[test]
fn build_missing_store_exits_3() {
    let state_dir = tempfile::tempdir().unwrap();

    crunch_cmd()
        .arg("--store")
        .arg("/nonexistent/store/path")
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("build")
        .arg(fixture("simple.ncl"))
        .assert()
        .code(3)
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn verbose_eval_emits_runtime_fingerprint_to_stderr() {
    let store = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let output = crunch_cmd()
        .arg("--verbose")
        .arg("--store")
        .arg(store.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("eval")
        .arg(fixture("simple.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let parsed_stdout: serde_json::Value = serde_json::from_str(&stdout).expect("eval stdout should stay JSON");
    let fingerprint = runtime_fingerprint_payload(&stderr);

    assert_eq!(parsed_stdout["name"], "simple-test");
    assert_eq!(fingerprint["schema"], "mantle-runtime-fingerprint-v1");
    assert_eq!(fingerprint["command"], "eval");
    assert_eq!(fingerprint["logical_store_prefix"], "/mantle/store");
    assert_eq!(fingerprint["physical_store_dir"], store.path().display().to_string());
    assert_eq!(fingerprint["state_dir"], state_dir.path().display().to_string());
    assert_eq!(fingerprint["json_mode"], false);
    assert_eq!(fingerprint["verbosity_source"], "verbose-flag");
    assert_eq!(fingerprint["diagnostic_scope"], "selected-runtime-context-only");
    assert!(fingerprint["mantle_version"].as_str().is_some());
}

#[test]
fn default_eval_does_not_emit_runtime_fingerprint() {
    let output = crunch_cmd().arg("eval").arg(fixture("simple.ncl")).output().expect("should run");

    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains(RUNTIME_FINGERPRINT_PREFIX), "default stderr should stay quiet: {stderr}");
}

#[test]
fn json_verbose_eval_keeps_stdout_parseable_and_diagnostics_on_stderr() {
    let store = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let output = crunch_cmd()
        .arg("--json")
        .arg("--verbose")
        .arg("--store")
        .arg(store.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("eval")
        .arg(fixture("simple.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let parsed_stdout: serde_json::Value = serde_json::from_str(&stdout).expect("JSON stdout should remain parseable");
    let fingerprint = runtime_fingerprint_payload(&stderr);

    assert_eq!(parsed_stdout["name"], "simple-test");
    assert_eq!(fingerprint["command"], "eval");
    assert_eq!(fingerprint["json_mode"], true);
    assert!(!stdout.contains(RUNTIME_FINGERPRINT_PREFIX), "fingerprint must not pollute stdout: {stdout}");
}

#[test]
fn verbose_build_plan_emits_build_mode_fingerprint_before_preflight_result() {
    let store = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let output = crunch_cmd()
        .arg("--verbose")
        .arg("--nix-compat")
        .arg("--store")
        .arg(store.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("build")
        .arg("--plan")
        .arg("--strict-hermetic")
        .arg("--no-substitute")
        .arg(fixture("simple.ncl"))
        .output()
        .expect("build plan should run");

    assert_ne!(output.status.code(), Some(CLAP_USAGE_ERROR_CODE));
    let stderr = String::from_utf8(output.stderr).unwrap();
    let fingerprint = runtime_fingerprint_payload(&stderr);

    assert_eq!(fingerprint["command"], "build");
    assert_eq!(fingerprint["logical_store_prefix"], "/nix/store");
    assert_eq!(fingerprint["hermeticity_mode"], "strict");
    assert_eq!(fingerprint["substitution_mode"], "disabled");
    assert_eq!(fingerprint["substituter_count"], 0);
    assert_eq!(fingerprint["nix_compat_mode"], true);
}

#[test]
fn verbose_store_roots_emits_store_fingerprint() {
    let store = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let output = crunch_cmd()
        .arg("--verbose")
        .arg("--store")
        .arg(store.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("store")
        .arg("roots")
        .output()
        .expect("store roots should run");

    assert!(output.status.success(), "store roots should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stderr = String::from_utf8(output.stderr).unwrap();
    let fingerprint = runtime_fingerprint_payload(&stderr);

    assert_eq!(fingerprint["command"], "store.roots");
    assert_eq!(fingerprint["physical_store_dir"], store.path().display().to_string());
    assert_eq!(fingerprint["state_dir"], state_dir.path().display().to_string());
}

// ── Build log tests ────────────────────────────────────────

#[test]
fn log_subcommand_no_logs() {
    // Point to an empty log dir
    let dir = tempfile::tempdir().unwrap();
    crunch_cmd().env("CRUNCH_LOG_DIR", dir.path()).arg("log").arg("--list").assert().success();
}

#[test]
fn log_subcommand_lists_logs() {
    let dir = tempfile::tempdir().unwrap();
    let log_file = dir.path().join("abc123-test.drv.log");
    std::fs::write(
        &log_file,
        "# crunch build log\n# derivation: mytest\n# drv_path: abc123-test.drv\n# status: success\n# timestamp: 0\n\nhello world\n",
    ).unwrap();

    crunch_cmd()
        .env("CRUNCH_LOG_DIR", dir.path())
        .arg("log")
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("mytest"))
        .stdout(predicate::str::contains("success"));
}

#[test]
fn log_subcommand_shows_log_by_query() {
    let dir = tempfile::tempdir().unwrap();
    let log_file = dir.path().join("abc123-test.drv.log");
    std::fs::write(
        &log_file,
        "# crunch build log\n# derivation: mytest\n# status: success\n# timestamp: 0\n\nbuild output here\n",
    )
    .unwrap();

    crunch_cmd()
        .env("CRUNCH_LOG_DIR", dir.path())
        .arg("log")
        .arg("abc123")
        .assert()
        .success()
        .stdout(predicate::str::contains("build output here"));
}

#[test]
fn log_subcommand_query_not_found() {
    let dir = tempfile::tempdir().unwrap();
    crunch_cmd()
        .env("CRUNCH_LOG_DIR", dir.path())
        .arg("log")
        .arg("nonexistent")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("no log matching"));
}

// ── Phase: Fetcher hash mismatch + --fix ─────────────────────────

#[test]
fn fetchurl_wrong_hash_shows_correct_hash() {
    // Spin up a local HTTP server serving known content.
    let content = b"auto-fix test content";
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        use std::io::Write;
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            let resp = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", content.len());
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.write_all(content);
        }
    });

    // Write a .ncl file with a wrong hash
    let store_dir = tempfile::tempdir().unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let ncl_file = work_dir.path().join("fetch-test.ncl");
    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
               crunch.fetchurl {{
                 url = "http://{addr}/test.txt",
                 hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
               }}"#
        ),
    )
    .unwrap();

    let output = crunch_cmd().arg("--store").arg(store_dir.path()).arg("build").arg(&ncl_file).output().unwrap();

    handle.join().unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should contain both the wrong and correct hash
    assert!(stderr.contains("hash mismatch"), "stderr should mention hash mismatch: {stderr}");
    assert!(stderr.contains("sha256-"), "stderr should contain SRI hash: {stderr}");
    // Should suggest the update
    assert!(
        stderr.contains("update") || stderr.contains("got:") || stderr.contains("got sha256-"),
        "stderr should suggest the correct hash: {stderr}"
    );
    assert!(!output.status.success(), "build should fail on hash mismatch");
}

#[test]
fn fix_flag_rewrites_hash() {
    // Spin up a local HTTP server
    let content = b"fix-flag test content";
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        use std::io::Write;
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            let resp = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", content.len());
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.write_all(content);
        }
    });

    let store_dir = tempfile::tempdir().unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let ncl_file = work_dir.path().join("fix-test.ncl");
    let wrong_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    std::fs::write(
        &ncl_file,
        format!(
            r#"let crunch = import "lib.ncl" in
               crunch.fetchurl {{
                 url = "http://{addr}/test.txt",
                 hash = "{wrong_hash}",
               }}"#
        ),
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("--store")
        .arg(store_dir.path())
        .arg("build")
        .arg("--fix")
        .arg(&ncl_file)
        .output()
        .unwrap();

    handle.join().unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("fixed:"), "stderr should confirm the fix: {stderr}");

    // The .ncl file should have been rewritten
    let updated = std::fs::read_to_string(&ncl_file).unwrap();
    assert!(!updated.contains(wrong_hash), "old hash should be gone from the file");
    assert!(updated.contains("sha256-"), "new SRI hash should be in the file: {updated}");
}

// ── CLI flag tests ────────────────────────────────────────────────

#[test]
fn build_accepts_jobs_flag() {
    // --jobs should be accepted without error (even if the build itself
    // fails due to missing store, bwrap, etc.).
    crunch_cmd()
        .args(["build", "--jobs", "2"])
        .arg(fixture("simple.ncl"))
        .assert()
        // We don't assert success — the build may fail (no bwrap, read-only
        // store, etc.). We just verify clap accepts the flag.
        .stderr(predicate::str::contains("unrecognized").not());
}

#[test]
fn build_accepts_short_j_flag() {
    crunch_cmd()
        .args(["build", "-j", "1"])
        .arg(fixture("simple.ncl"))
        .assert()
        .stderr(predicate::str::contains("unrecognized").not());
}

#[test]
fn build_accepts_no_substitute_flag() {
    crunch_cmd()
        .args(["build", "--no-substitute"])
        .arg(fixture("simple.ncl"))
        .assert()
        .stderr(predicate::str::contains("unrecognized").not());
}

#[test]
fn build_accepts_substituters_flag() {
    crunch_cmd()
        .args(["build", "--substituters", "https://example.com"])
        .arg(fixture("simple.ncl"))
        .assert()
        .stderr(predicate::str::contains("unrecognized").not());
}

// ── mkDerivation eval tests ───────────────────────────────────

#[test]
fn eval_mkderivation_has_default_phases() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let builders = import "builders/lib.ncl" in
let stdenv = builders.mkStdenv {
  bash = "/nix/store/00000000000000000000000000000000-bash",
  coreutils = "/nix/store/00000000000000000000000000000001-coreutils",
} in
stdenv.mkDerivation {
  pname = "defaults-test",
  version = "0",
}"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg("-I")
        .arg(crunch_root())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success(), "eval should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Default phases should appear in the build script.
    assert!(stdout.contains("make install"), "should have default installPhase: {stdout}");
    assert!(stdout.contains("configure"), "should have default configurePhase: {stdout}");
    assert!(stdout.contains("NIX_BUILD_CORES"), "should have NIX_BUILD_CORES: {stdout}");
}

#[test]
fn eval_mkderivation_custom_phase_overrides_default() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let builders = import "builders/lib.ncl" in
let stdenv = builders.mkStdenv {
  bash = "/nix/store/00000000000000000000000000000000-bash",
  coreutils = "/nix/store/00000000000000000000000000000001-coreutils",
} in
stdenv.mkDerivation {
  pname = "custom-phase",
  version = "1",
  buildPhase = "cmake --build .",
  installPhase = "cmake --install . --prefix $out",
}"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg("-I")
        .arg(crunch_root())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cmake --build"), "custom buildPhase: {stdout}");
    assert!(stdout.contains("cmake --install"), "custom installPhase: {stdout}");
    // Default make should NOT appear.
    assert!(!stdout.contains("make -j"), "default buildPhase should be overridden: {stdout}");
    assert!(!stdout.contains("make install"), "default installPhase should be overridden: {stdout}");
}

#[test]
fn eval_mkderivation_empty_phase_skips() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let builders = import "builders/lib.ncl" in
let stdenv = builders.mkStdenv {
  bash = "/nix/store/00000000000000000000000000000000-bash",
  coreutils = "/nix/store/00000000000000000000000000000001-coreutils",
} in
stdenv.mkDerivation {
  pname = "skip-phase",
  version = "0",
  configurePhase = "",
  buildPhase = "gcc -o out main.c",
  installPhase = "cp out $out",
}"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg("-I")
        .arg(crunch_root())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Empty configurePhase should not emit ./configure default.
    assert!(!stdout.contains("./configure"), "empty configurePhase should be skipped: {stdout}");
    assert!(stdout.contains("gcc -o out"), "custom buildPhase present: {stdout}");
}

#[test]
fn eval_mkshell_produces_valid_derivation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let builders = import "builders/lib.ncl" in
builders.mkShell {
  bash = "/nix/store/00000000000000000000000000000000-bash",
  name = "test-shell",
  buildInputs = ["/nix/store/00000000000000000000000000000001-gcc"],
  env = { CC = "gcc" },
}"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg("-I")
        .arg(crunch_root())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success(), "mkShell eval should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-shell"), "name: {stdout}");
    assert!(stdout.contains("gcc"), "PATH should include gcc: {stdout}");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed["env"]["CC"].as_str() == Some("gcc"), "env.CC: {stdout}");

    // Sidecar JSON is embedded in the derivation's env
    let sidecar_raw = parsed["env"]["CRUNCH_SIDECAR_JSON"]
        .as_str()
        .expect("CRUNCH_SIDECAR_JSON should be a string in derivation env");
    let sidecar: serde_json::Value =
        serde_json::from_str(sidecar_raw).expect("CRUNCH_SIDECAR_JSON should be valid JSON");
    assert_eq!(sidecar["version"], 1, "sidecar version");
    assert_eq!(sidecar["env"]["CC"].as_str(), Some("gcc"), "sidecar env.CC");
    assert!(sidecar["hook"].is_null(), "no hook declared");
    let path_entries = sidecar["path_entries"].as_array().expect("path_entries is array");
    assert_eq!(path_entries.len(), 1);
    assert!(path_entries[0].as_str().unwrap().ends_with("/bin"), "path entry ends with /bin");
}

#[test]
fn eval_mkshell_sidecar_with_hook() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let builders = import "builders/lib.ncl" in
builders.mkShell {
  bash = "/nix/store/00000000000000000000000000000000-bash",
  name = "hook-shell",
  env = { GREETING = "hi" },
  hook = "echo welcome",
}"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg("-I")
        .arg(crunch_root())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(
        output.status.success(),
        "mkShell+hook eval should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let sidecar_raw = parsed["env"]["CRUNCH_SIDECAR_JSON"].as_str().unwrap();
    let sidecar: serde_json::Value = serde_json::from_str(sidecar_raw).unwrap();
    assert_eq!(sidecar["version"], 1);
    assert_eq!(sidecar["hook"].as_str(), Some("echo welcome"));
    assert_eq!(sidecar["env"]["GREETING"].as_str(), Some("hi"));
}

#[test]
fn eval_mkshell_sidecar_empty_inputs() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let builders = import "builders/lib.ncl" in
builders.mkShell {
  name = "bare-shell",
}"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg("-I")
        .arg(crunch_root())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(
        output.status.success(),
        "bare mkShell eval should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let sidecar_raw = parsed["env"]["CRUNCH_SIDECAR_JSON"].as_str().unwrap();
    let sidecar: serde_json::Value = serde_json::from_str(sidecar_raw).unwrap();
    assert_eq!(sidecar["version"], 1);
    assert_eq!(sidecar["env"], serde_json::json!({}));
    assert_eq!(sidecar["path_entries"], serde_json::json!([]));
    assert!(sidecar["hook"].is_null());
}

#[test]
fn eval_mkderivation_provenance_exported_from_builder_layer() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let builders = import "builders/lib.ncl" in
let stdenv = builders.mkStdenv {
  bash = "/nix/store/00000000000000000000000000000000-bash",
  coreutils = "/nix/store/00000000000000000000000000000001-coreutils",
} in
stdenv.mkDerivation {
  pname = "prov-test",
  version = "1",
  provenance = {
    supplier = "Example Supplier",
    homepage = "https://example.invalid/prov",
    source_aliases = ["origin", "mirror"],
  },
}"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg("-I")
        .arg(crunch_root())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success(), "eval should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let parsed: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    assert_eq!(parsed["provenance"]["supplier"].as_str(), Some("Example Supplier"));
    assert_eq!(parsed["provenance"]["homepage"].as_str(), Some("https://example.invalid/prov"));
    assert_eq!(parsed["provenance"]["source_aliases"][0].as_str(), Some("origin"));
}

#[test]
fn eval_core_derivation_rejects_builder_only_provenance() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let crunch = import "lib.ncl" in
{
  name = "core-provenance",
  builder = "/bin/sh",
  provenance = { supplier = "nope" },
} | crunch.Derivation"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(
        !output.status.success(),
        "core contract should reject provenance field: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn eval_mkderivation_src_wired_to_env() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let builders = import "builders/lib.ncl" in
let stdenv = builders.mkStdenv {
  bash = "/nix/store/00000000000000000000000000000000-bash",
  coreutils = "/nix/store/00000000000000000000000000000001-coreutils",
} in
stdenv.mkDerivation {
  pname = "src-test",
  version = "0",
  src = "/nix/store/00000000000000000000000000000002-source",
  buildPhase = "echo building",
  installPhase = "mkdir -p $out",
}"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg("-I")
        .arg(crunch_root())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success(), "eval should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // src should be in env
    assert_eq!(
        parsed["env"]["src"].as_str(),
        Some("/nix/store/00000000000000000000000000000002-source"),
        "$src env var: {stdout}"
    );
    // src should be in inputs
    let inputs = parsed["inputs"].as_array().unwrap();
    assert!(inputs.iter().any(|v| v.as_str().unwrap().contains("source")), "src should be in inputs: {stdout}");
}

#[test]
fn eval_multi_output_derivation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let crunch = import "lib.ncl" in
{
  name = "multi-out",
  builder = "/bin/sh",
  args = ["-c", "mkdir -p $out $dev"],
  outputs = ["out", "dev"],
} | crunch.Derivation"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    let outputs = parsed["outputs"].as_array().unwrap();
    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0], "out");
    assert_eq!(outputs[1], "dev");
}

#[test]
fn eval_output_selection_with_select() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let crunch = import "lib.ncl" in
let lib = {
  name = "mylib",
  builder = "/bin/sh",
  args = ["-c", "mkdir -p $out $dev"],
  outputs = ["out", "dev"],
} | crunch.Derivation in
{
  name = "consumer",
  builder = "/bin/sh",
  args = ["-c", "echo > $out"],
  inputs = [crunch.select lib "dev"],
} | crunch.Derivation"#,
    )
    .unwrap();

    let output = crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg(dir.path().join("test.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success(), "select eval: {}", String::from_utf8_lossy(&output.stderr));
    let parsed: serde_json::Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap();
    // inputs should contain the output selection record
    let inputs = parsed["inputs"].as_array().unwrap();
    assert!(
        inputs.iter().any(|v| v.is_object() && v.get("output").is_some()),
        "inputs should contain an output selection: {inputs:?}"
    );
}

// ── Shell build tests (Linux-only, require bwrap) ───────────────

#[cfg(target_os = "linux")]
mod shell_build_tests {
    use super::*;

    /// Write a crunch.ncl project whose default devShell produces a sidecar.
    fn write_shell_project(
        dir: &std::path::Path,
        env_entries: &[(&str, &str)],
        path_entries: &[&str],
        hook: Option<&str>,
    ) {
        let env_record = if env_entries.is_empty() {
            "{}".to_string()
        } else {
            let fields: Vec<String> = env_entries.iter().map(|(k, v)| format!("    {k} = \"{v}\",")).collect();
            format!("{{\n{}\n  }}", fields.join("\n"))
        };

        let path_array = if path_entries.is_empty() {
            "[]".to_string()
        } else {
            let items: Vec<String> = path_entries.iter().map(|p| format!("    \"{p}\",")).collect();
            format!("[\n{}\n  ]", items.join("\n"))
        };

        let hook_value = match hook {
            Some(h) => format!("\"{h}\""),
            None => "null".to_string(),
        };

        let ncl = format!(
            r#"let crunch = import "lib.ncl" in
{{
  devShells = {{
    default = {{
      name = "test-shell",
      builder = "/bin/sh",
      args = ["-c", "/bin/busybox mkdir -p \"$out\"\nprintf '%s\\n' \"$CRUNCH_SIDECAR_JSON\" > \"$out/.crunch-shell.json\""],
      addressing_mode = 'input-addressed,
      env = {{
        CRUNCH_SIDECAR_JSON = std.serialize 'Json {{
          version = 1,
          env = {env_record},
          path_entries = {path_array},
          hook = {hook_value},
        }},
      }},
    }} | crunch.Derivation,
  }},
}}"#
        );

        std::fs::write(dir.join("crunch.ncl"), ncl).unwrap();
    }

    #[test]
    fn shell_env_vars_visible_in_command() {
        if !can_build() {
            eprintln!("skipping: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();

        write_shell_project(dir.path(), &[("TEST_FOO", "hello_from_sidecar"), ("TEST_BAR", "42")], &[], None);

        let output = crunch_cmd()
            .current_dir(dir.path())
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .args(["shell", "--command", "env"])
            .output()
            .unwrap();

        assert!(output.status.success(), "shell --command env: {}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("TEST_FOO=hello_from_sidecar"), "env should have TEST_FOO: {stdout}");
        assert!(stdout.contains("TEST_BAR=42"), "env should have TEST_BAR: {stdout}");
        assert!(stdout.contains("CRUNCH_SHELL="), "env should have CRUNCH_SHELL: {stdout}");
    }

    #[test]
    fn shell_hook_output_appears() {
        if !can_build() {
            eprintln!("skipping: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();

        write_shell_project(dir.path(), &[], &[], Some("echo HOOK_MARKER >&2"));

        let output = crunch_cmd()
            .current_dir(dir.path())
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .args(["shell", "--command", "true"])
            .output()
            .unwrap();

        assert!(output.status.success(), "shell with hook: {}", String::from_utf8_lossy(&output.stderr));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("HOOK_MARKER"), "hook output should appear in stderr: {stderr}");
    }

    #[test]
    fn shell_no_hook_suppresses_hook() {
        if !can_build() {
            eprintln!("skipping: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();

        write_shell_project(dir.path(), &[], &[], Some("echo HOOK_MARKER >&2"));

        let output = crunch_cmd()
            .current_dir(dir.path())
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .args(["shell", "--no-hook", "--command", "true"])
            .output()
            .unwrap();

        assert!(output.status.success(), "shell --no-hook: {}", String::from_utf8_lossy(&output.stderr));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("HOOK_MARKER"), "--no-hook should suppress hook: {stderr}");
    }

    #[test]
    fn shell_strict_hooks_exits_on_failure() {
        if !can_build() {
            eprintln!("skipping: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();

        write_shell_project(dir.path(), &[], &[], Some("exit 7"));

        let output = crunch_cmd()
            .current_dir(dir.path())
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .args(["shell", "--strict-hooks", "--command", "true"])
            .output()
            .unwrap();

        assert!(!output.status.success(), "--strict-hooks with failing hook should fail");
        assert_eq!(output.status.code(), Some(7), "exit code should propagate from hook");
    }

    #[test]
    fn shell_with_adds_path_entry() {
        if !can_build() {
            eprintln!("skipping: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let tool_dir = tempfile::tempdir().unwrap();

        // Create a dummy executable in a bin/ subdirectory.
        // compute_activation appends /bin to --with paths.
        let bin_dir = tool_dir.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let tool_path = bin_dir.join("crunch-test-tool");
        std::fs::write(&tool_path, "#!/bin/sh\necho found-it\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tool_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        write_shell_project(dir.path(), &[], &[], None);

        let output = crunch_cmd()
            .current_dir(dir.path())
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .arg("shell")
            .arg("--with")
            .arg(tool_dir.path())
            .args(["--command", "crunch-test-tool"])
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "--with should add tool to PATH: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("found-it"), "tool output: {stdout}");
    }

    #[test]
    fn shell_missing_sidecar_clear_error() {
        if !can_build() {
            eprintln!("skipping: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();

        // Write a devShell that builds a plain file (no sidecar).
        std::fs::write(
            dir.path().join("crunch.ncl"),
            r#"let crunch = import "lib.ncl" in
{
  devShells = {
    default = {
      name = "no-sidecar",
      builder = "/bin/sh",
      args = ["-c", "echo just-a-file > $out"],
      addressing_mode = 'input-addressed,
    } | crunch.Derivation,
  },
}"#,
        )
        .unwrap();

        let output = crunch_cmd()
            .current_dir(dir.path())
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .args(["shell", "--command", "true"])
            .output()
            .unwrap();

        assert!(!output.status.success(), "missing sidecar should fail");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(".crunch-shell.json"), "error should name the sidecar file: {stderr}");
        assert!(stderr.contains("mkShell"), "error should suggest mkShell: {stderr}");
    }

    #[test]
    fn shell_exit_code_propagation() {
        if !can_build() {
            eprintln!("skipping: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();

        write_shell_project(dir.path(), &[], &[], None);

        let output = crunch_cmd()
            .current_dir(dir.path())
            .arg("--store")
            .arg(store.path())
            .arg("--state-dir")
            .arg(state.path())
            .args(["shell", "--run", "exit 42"])
            .output()
            .unwrap();

        assert!(!output.status.success(), "non-zero exit should propagate");
        assert_eq!(output.status.code(), Some(42), "exit code should be 42");
    }
}

// ── Shell CLI tests ──────────────────────────────────────────────

#[test]
fn shell_command_run_mutual_exclusion() {
    crunch_cmd()
        .args(["shell", "--command", "foo", "--run", "bar"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn shell_help_shows_new_flags() {
    crunch_cmd()
        .args(["shell", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--command"))
        .stdout(predicate::str::contains("--run"))
        .stdout(predicate::str::contains("--with"))
        .stdout(predicate::str::contains("--no-hook"))
        .stdout(predicate::str::contains("--strict-hooks"));
}

#[test]
fn develop_alias_exists() {
    crunch_cmd()
        .args(["develop", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Alias for `shell`"));
}

#[test]
fn shell_with_nonexistent_path_fails() {
    crunch_cmd()
        .args([
            "shell",
            "--with",
            "/nonexistent/store/path/that/definitely/does/not/exist",
        ])
        .assert()
        .failure();
}
