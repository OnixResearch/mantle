use std::collections::BTreeMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::Shutdown;
use std::net::TcpListener;
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use assert_cmd::Command;
use crunch_attestation::Canonicalize;
use crunch_attestation::ClosureAttestation;

const PROJECTS_ROOT: &str = "examples/projects";
const GENERATED_SITE_PROJECT: &str = "generated-site";
const CODEGEN_PROJECT: &str = "codegen-pipeline";
const C_PROJECT: &str = "c-library-cli";
const RUST_PROJECT: &str = "rust-workspace";
const FETCHED_PATCHED_PROJECT: &str = "fetched-and-patched";
const MULTI_OUTPUT_SDK_PROJECT: &str = "multi-output-sdk";
const SCHEMA_CODEGEN_PROJECT: &str = "schema-codegen";
const REPRODUCIBLE_RELEASE_PROJECT: &str = "reproducible-release";
const SIGNED_CACHE_PROJECT: &str = "signed-cache-roundtrip";
const LOCKED_DEPENDENCY_PROJECT: &str = "locked-dependency-lifecycle";
const CROSS_COMPILED_PROJECT: &str = "cross-compiled-host-tool";
const STORE_GC_PROJECT: &str = "store-gc-lifecycle";
const DELTA_SUBSTITUTION_PROJECT: &str = "delta-substitution";
const RELEASE_WITNESS_PROJECT: &str = "release-witness-handoff";
const OFFLINE_SOURCE_BUNDLE_PROJECT: &str = "offline-source-bundle";
const REVIEWED_FILEGEN_PROJECT: &str = "reviewed-file-generation";
const DEVELOPER_SHELL_RUN_PROJECT: &str = "developer-shell-run";
const ARTIFACT_PROVENANCE_PROJECT: &str = "artifact-provenance-walkthrough";
const HERMETIC_PLAN_REBUILD_PROJECT: &str = "hermetic-plan-rebuild";
const SHARED_ACTION_RESULT_PROJECT: &str = "shared-action-result-roundtrip";
const SOURCE_BUNDLE_RECORDS_RELATIVE: &str = "source-bundles/records";
const SOURCE_BUNDLE_RECORD_COUNT: u64 = 1;
const SOURCE_BUNDLE_RECORD_COUNT_USIZE: usize = 1;
const GENERATED_FILE_COUNT: usize = 2;
const HEX_BYTE_TEXT_WIDTH: usize = 2;
const LOCK_RETENTION_ROOT_RECORD: &str =
    ".mantle/retention-roots/9dcaf0da80828fc8c2c8d41160c0d12f98bb1abc03c1d1f611bc534b579c00ad.json";
const SITE_TITLE: &str = "Mantle Project Gallery";
const SITE_CHECK_RESULT: &str = "generated-site-check: ok";
const CODEGEN_MESSAGE: &str = "Hello from generated project code";
const CODEGEN_CHECK_RESULT: &str = "codegen-demo-check: ok";
const C_GREETING: &str = "Hello, Mantle!";
const C_CHECK_RESULT: &str = "c-library-cli-check: ok";
const C_POSITIVE_TEST: &str = "positive_status == GREET_STATUS_OK";
const C_NEGATIVE_TEST: &str = "zero_capacity_status == GREET_STATUS_INVALID_ARGUMENT";
const RUST_POSITIVE_TEST: &str = "renders_a_normalized_name";
const RUST_NEGATIVE_TEST: &str = "rejects_an_empty_name";
const RUST_GREETING: &str = "Hello, Mantle!";
const RUST_CHECK_RESULT: &str = "rust-workspace-smoke-check: ok";
const FETCHED_PATCHED_CHECK_RESULT: &str = "fetched-and-patched-check: ok";
const MULTI_OUTPUT_RUNTIME_CHECK_RESULT: &str = "multi-output-sdk-runtime-check: ok";
const MULTI_OUTPUT_DEVELOPMENT_CHECK_RESULT: &str = "multi-output-sdk-development-check: ok";
const SCHEMA_CODEGEN_CHECK_RESULT: &str = "schema-codegen-integration-check: ok";
const RELEASE_CHECK_RESULT: &str = "reproducible-release-check: ok";
const RELEASE_TAMPER_CHECK_RESULT: &str = "reproducible-release-tamper-check: ok";
const RELEASE_ARCHIVE_BLAKE3: &str = "blake3-Roawb3qp5wK7exFYdIBJiEwtIjGz2YivY1sVtEI+YDI=";
const MISSING_SELECTOR: &str = ".#missing-package";
const DEFAULT_OUTPUT_LABEL: &str = "out";
const CACHED_OUTPUT_ANNOTATION: &str = "cached";
const MAX_PROJECT_OUTPUTS: usize = 16;
const EXECUTABLE_MODE: u32 = 0o755;
const MAX_HTTP_HEADER_LINES: usize = 128;
const MAX_HTTP_REQUESTS: u32 = 256;
const HTTP_POLL_INTERVAL_MS: u64 = 10;
const _: () = {
    assert!(MAX_HTTP_HEADER_LINES > 1);
    assert!(MAX_HTTP_REQUESTS > 1);
};
const ACTION_RESULT_TRUSTED_KEY: &str = "action.example.com-1:yKUSiqP9yaMSduDmGtw8U9iVVd/Coyv9csB1rjHtiRM=";
const ACTION_RESULT_UNKNOWN_SIGNER_KEY: &str = "unknown.example.com-1:yKUSiqP9yaMSduDmGtw8U9iVVd/Coyv9csB1rjHtiRM=";

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn project_root(project: &str) -> PathBuf {
    repo_root().join(PROJECTS_ROOT).join(project)
}

fn find_bwrap() -> Option<PathBuf> {
    let path_dirs = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path_dirs) {
        let candidate = directory.join("bwrap");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn find_static_busybox() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("SNIX_BUILD_SANDBOX_SHELL") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() && candidate.to_string_lossy().contains("busybox-static") {
            return Some(candidate);
        }
    }

    let entries = std::fs::read_dir("/nix/store").ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join("bin/busybox");
        if candidate.is_file() && candidate.to_string_lossy().contains("busybox-static") {
            return Some(candidate);
        }
    }
    None
}

fn can_build_fast_projects() -> bool {
    cfg!(target_os = "linux")
        && Path::new("/nix/store").is_dir()
        && find_bwrap().is_some()
        && find_static_busybox().is_some()
}

struct StaticCacheServer {
    base_url: String,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl StaticCacheServer {
    fn start(root: &Path) -> Self {
        assert!(root.is_dir(), "static cache root must exist");
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let root = root.to_path_buf();
        let handle = thread::spawn(move || {
            let mut served_requests = 0u32;
            while !stop_thread.load(Ordering::SeqCst) && served_requests < MAX_HTTP_REQUESTS {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        serve_static_cache_request(&root, &mut stream);
                        served_requests = served_requests.saturating_add(1);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(HTTP_POLL_INTERVAL_MS));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            base_url: format!("http://{address}"),
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for StaticCacheServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.base_url.trim_start_matches("http://"));
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn serve_static_cache_request(root: &Path, stream: &mut TcpStream) {
    assert!(root.is_dir(), "static cache root must remain available");
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).unwrap_or(0) == 0 {
        return;
    }
    for _ in 0..MAX_HTTP_HEADER_LINES {
        let mut header = String::new();
        if reader.read_line(&mut header).unwrap_or(0) == 0 || header == "\r\n" {
            break;
        }
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let request_target = parts.next().unwrap_or_default();
    let body = static_cache_relative_path(request_target).and_then(|path| std::fs::read(root.join(path)).ok());
    let (status, body) = match body {
        Some(body) if method == "GET" || method == "HEAD" => ("HTTP/1.1 200 OK", body),
        _ => ("HTTP/1.1 404 Not Found", b"not found\n".to_vec()),
    };
    write!(stream, "{status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
    if method != "HEAD" {
        stream.write_all(&body).unwrap();
    }
    stream.flush().unwrap();
    let _ = stream.shutdown(Shutdown::Both);
}

fn static_cache_relative_path(request_target: &str) -> Option<PathBuf> {
    let path = request_target.split('?').next()?.strip_prefix('/')?;
    let mut relative = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(segment) => relative.push(segment),
            _ => return None,
        }
    }
    (!relative.as_os_str().is_empty()).then_some(relative)
}

struct ProjectRun {
    _store: tempfile::TempDir,
    _state: tempfile::TempDir,
    output: std::process::Output,
}

fn run_project_build(project: &str, selector: &str) -> ProjectRun {
    run_project_build_with_verbosity(project, selector, false)
}

fn run_project_build_verbose(project: &str, selector: &str) -> ProjectRun {
    run_project_build_with_verbosity(project, selector, true)
}

fn project_command_with_build_environment(project: &str) -> Command {
    assert!(!project.is_empty(), "project name must not be empty");
    let root = project_root(project);
    assert!(root.is_dir(), "project root must exist: {}", root.display());
    let mut command = mantle_cmd();
    command.current_dir(root);
    if let Some(bwrap) = find_bwrap() {
        let wrappers = PathBuf::from("/run/wrappers/bin");
        let path = std::env::join_paths(
            std::iter::once(bwrap.parent().unwrap().to_path_buf())
                .chain(wrappers.is_dir().then_some(wrappers))
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())),
        )
        .unwrap();
        command.env("PATH", path);
    }
    if let Some(busybox) = find_static_busybox() {
        command.env("SNIX_BUILD_SANDBOX_SHELL", busybox);
    }
    command.env("CRUNCH_NO_FUSE", "1");
    command
}

fn run_project_build_with_verbosity(project: &str, selector: &str, verbose: bool) -> ProjectRun {
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut command = project_command_with_build_environment(project);
    if verbose {
        command.arg("--verbose");
    }
    command.args([
        "build",
        selector,
        "--store",
        store.path().to_str().unwrap(),
        "--state-dir",
        state.path().to_str().unwrap(),
        "--no-substitute",
    ]);
    let output = command.output().expect("project build should run");
    ProjectRun {
        _store: store,
        _state: state,
        output,
    }
}

fn parse_output_paths(stdout: &[u8]) -> Result<BTreeMap<String, PathBuf>, String> {
    let text = std::str::from_utf8(stdout).map_err(|error| format!("build output is not UTF-8: {error}"))?;
    let mut paths = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if paths.len() >= MAX_PROJECT_OUTPUTS {
            return Err(format!("build reported more than {MAX_PROJECT_OUTPUTS} outputs"));
        }
        let (path, label) = match line.rsplit_once(" (") {
            Some((path, suffix)) => {
                let annotations = suffix.strip_suffix(')').ok_or_else(|| format!("malformed output label: {line}"))?;
                let first_annotation = annotations.split(',').next().unwrap_or_default().trim();
                let label = if first_annotation == CACHED_OUTPUT_ANNOTATION {
                    DEFAULT_OUTPUT_LABEL
                } else {
                    first_annotation
                };
                (path, label)
            }
            None => (line, DEFAULT_OUTPUT_LABEL),
        };
        if path.is_empty() || label.is_empty() {
            return Err(format!("empty output path or label: {line}"));
        }
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(format!("output path is not absolute: {}", path.display()));
        }
        if paths.insert(label.to_string(), path).is_some() {
            return Err(format!("duplicate output label: {label}"));
        }
    }
    if paths.is_empty() {
        return Err("build did not report an output path".to_string());
    }
    Ok(paths)
}

fn output_path_for_label(run: &ProjectRun, label: &str) -> PathBuf {
    parse_output_paths(&run.output.stdout)
        .unwrap()
        .remove(label)
        .unwrap_or_else(|| panic!("missing output label {label}"))
}

fn output_path(run: &ProjectRun) -> PathBuf {
    output_path_for_label(run, DEFAULT_OUTPUT_LABEL)
}

fn assert_success(run: &ProjectRun, label: &str) {
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(run.output.status.success(), "{label} failed:\n{stderr}");
    assert!(!run.output.stdout.is_empty(), "{label} should print an output path");
}

fn assert_project_evaluates(project: &str, label: &str) {
    let mut command = mantle_cmd();
    let output = command.current_dir(project_root(project)).args(["eval", "mantle-project.ncl"]).output().unwrap();
    assert!(output.status.success(), "{label} should evaluate: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.stdout.is_empty(), "{label} evaluation should export JSON");
}

fn copy_locked_dependency_project(destination: &Path) {
    const PROJECT_FILES: &[&str] = &[
        "mantle-project.ncl",
        "mantle.lock",
        ".mantle/inputs.ncl",
        ".mantle/retention.json",
        LOCK_RETENTION_ROOT_RECORD,
        "fixtures/unresolved-revision.ncl",
        "patches/message.patch",
        "sources/message.txt",
    ];
    assert!(destination.is_dir(), "fixture destination must exist");
    assert!(!PROJECT_FILES.is_empty(), "locked fixture must copy project files");
    let source = project_root(LOCKED_DEPENDENCY_PROJECT);
    for relative_path in PROJECT_FILES {
        let target = destination.join(relative_path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(source.join(relative_path), target).unwrap();
    }
}

fn copy_reviewed_filegen_project(destination: &Path) {
    const PROJECT_FILES: &[&str] = &[
        "mantle-project.ncl",
        "filegen-schema.ncl",
        "fixtures/missing-required-field.ncl",
        "fixtures/target-escape.ncl",
    ];
    assert!(destination.is_dir(), "fixture destination must exist");
    assert!(!PROJECT_FILES.is_empty(), "filegen fixture must copy project files");
    let source = project_root(REVIEWED_FILEGEN_PROJECT);
    for relative_path in PROJECT_FILES {
        let target = destination.join(relative_path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(source.join(relative_path), target).unwrap();
    }
}

fn write_source_bundle_build_root(destination: &Path) {
    assert!(destination.is_absolute(), "source bundle build root must be absolute");
    assert!(destination.parent().is_some(), "source bundle build root must have a parent");
    let project = project_root(OFFLINE_SOURCE_BUNDLE_PROJECT);
    let constructor = project.join("source-root.ncl").canonicalize().unwrap();
    let payload = project.join("sources/payload.txt").canonicalize().unwrap();
    let source = format!(
        "let make_source = import \"{}\" in\nmake_source \"file://{}\"\n",
        constructor.display(),
        payload.display()
    );
    std::fs::write(destination, source).unwrap();
}

fn run_project_command(current_dir: &Path, args: &[&str]) -> std::process::Output {
    assert!(current_dir.is_dir(), "project command directory must exist");
    assert!(!args.is_empty(), "project command arguments must not be empty");
    let mut command = mantle_cmd();
    command.current_dir(current_dir).args(args).output().unwrap()
}

fn workflow_build_command(project: &str, store: &Path, state: &Path) -> Command {
    assert!(store.is_dir(), "workflow store must exist");
    assert!(state.is_dir(), "workflow state must exist");
    let mut command = project_command_with_build_environment(project);
    command.arg("--json").arg("--store").arg(store).arg("--state-dir").arg(state).arg("build");
    command
}

fn workflow_attest_command(project: &str, store: &Path, state: &Path) -> Command {
    assert!(store.is_dir(), "attestation store must exist");
    assert!(state.is_dir(), "attestation state must exist");
    let mut command = project_command_with_build_environment(project);
    command.arg("--store").arg(store).arg("--state-dir").arg(state).arg("attest");
    command
}

fn parse_successful_json(output: std::process::Output, label: &str) -> serde_json::Value {
    assert!(!label.is_empty(), "JSON command label must not be empty");
    assert!(output.status.success(), "{label} failed: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| panic!("{label} stdout was not JSON: {error}"))
}

fn parse_failed_json(output: std::process::Output, label: &str) -> serde_json::Value {
    assert!(!label.is_empty(), "JSON command label must not be empty");
    assert!(!output.status.success(), "{label} unexpectedly succeeded");
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| panic!("{label} stdout was not JSON: {error}"))
}

fn check_result(run: &ProjectRun) -> String {
    std::fs::read_to_string(output_path(run).join("result.txt")).unwrap().trim().to_string()
}

#[test]
fn project_output_parser_accepts_named_outputs_and_rejects_duplicates() {
    let parsed =
        parse_output_paths(b"/tmp/example-dev (dev, cached)\n/tmp/example-doc (doc)\n/tmp/example-out (cached)\n")
            .unwrap();
    assert_eq!(parsed.get("dev"), Some(&PathBuf::from("/tmp/example-dev")));
    assert_eq!(parsed.get("doc"), Some(&PathBuf::from("/tmp/example-doc")));
    assert_eq!(parsed.get(DEFAULT_OUTPUT_LABEL), Some(&PathBuf::from("/tmp/example-out")));

    let duplicate = parse_output_paths(b"/tmp/first\n/tmp/second\n").unwrap_err();
    assert!(duplicate.contains("duplicate output label"), "unexpected duplicate error: {duplicate}");
    let relative = parse_output_paths(b"relative-output\n").unwrap_err();
    assert!(relative.contains("not absolute"), "unexpected relative path error: {relative}");
}

#[test]
fn production_workflow_projects_evaluate_and_keep_negative_paths() {
    for (project, label) in [
        (SIGNED_CACHE_PROJECT, "signed cache project"),
        (LOCKED_DEPENDENCY_PROJECT, "locked dependency project"),
        (CROSS_COMPILED_PROJECT, "cross-compiled project"),
        (STORE_GC_PROJECT, "store GC project"),
        (DELTA_SUBSTITUTION_PROJECT, "delta substitution project"),
        (RELEASE_WITNESS_PROJECT, "release witness project"),
        (OFFLINE_SOURCE_BUNDLE_PROJECT, "offline source bundle project"),
        (REVIEWED_FILEGEN_PROJECT, "reviewed file generation project"),
        (DEVELOPER_SHELL_RUN_PROJECT, "developer shell and run project"),
        (ARTIFACT_PROVENANCE_PROJECT, "artifact provenance project"),
        (HERMETIC_PLAN_REBUILD_PROJECT, "hermetic plan and rebuild project"),
        (SHARED_ACTION_RESULT_PROJECT, "shared action-result project"),
    ] {
        assert_project_evaluates(project, label);
    }

    let cache = std::fs::read_to_string(project_root(SIGNED_CACHE_PROJECT).join("README.md")).unwrap();
    assert!(cache.contains("Mantle skips an unknown signer"));
    assert!(cache.contains("wrong key material fails signature verification"));
    assert!(cache.contains("a corrupt NAR fails its declared hash check"));
    let cross = std::fs::read_to_string(project_root(CROSS_COMPILED_PROJECT).join("mantle-project.ncl")).unwrap();
    assert!(cross.contains("host/target role mismatch"));
    assert!(cross.contains("-x c -std=c11"));
    let gc = std::fs::read_to_string(project_root(STORE_GC_PROJECT).join("mantle-project.ncl")).unwrap();
    assert!(gc.contains("lock-holder"));
    let delta = std::fs::read_to_string(project_root(DELTA_SUBSTITUTION_PROJECT).join("demo.rs")).unwrap();
    assert!(delta.contains("FullArtifactFallback"));
    assert!(delta.contains("MissingSenderChunk"));
    let release = std::fs::read_to_string(project_root(RELEASE_WITNESS_PROJECT).join("demo.rs")).unwrap();
    assert!(release.contains("InsufficientQuorum"));
    assert!(release.contains("ReleaseRevocations"));
    let source_bundle = std::fs::read_to_string(project_root(OFFLINE_SOURCE_BUNDLE_PROJECT).join("README.md")).unwrap();
    assert!(source_bundle.contains("digest mismatch"));
    assert!(source_bundle.contains("route execution is future work"));
    let filegen = std::fs::read_to_string(project_root(REVIEWED_FILEGEN_PROJECT).join("README.md")).unwrap();
    assert!(filegen.contains("stale reviewed plan"));
    assert!(filegen.contains("target-escape.ncl"));
    let developer_loop =
        std::fs::read_to_string(project_root(DEVELOPER_SHELL_RUN_PROJECT).join("mantle-project.ncl")).unwrap();
    assert!(developer_loop.contains("MANTLE_EXAMPLE_PROFILE"));
    assert!(developer_loop.contains("usage: operator-demo NAME"));
    let provenance = std::fs::read_to_string(project_root(ARTIFACT_PROVENANCE_PROJECT).join("README.md")).unwrap();
    assert!(provenance.contains("they are not release or witness proofs"));
    assert!(provenance.contains("bind canonical recorded claims"));
    let hermetic = std::fs::read_to_string(project_root(HERMETIC_PLAN_REBUILD_PROJECT).join("README.md")).unwrap();
    assert!(hermetic.contains("matching BLAKE3 child-environment digests"));
    assert!(hermetic.contains("does not prove compiler correctness"));
    let action_result = std::fs::read_to_string(project_root(SHARED_ACTION_RESULT_PROJECT).join("README.md")).unwrap();
    assert!(action_result.contains("Index presence is discovery only"));
    assert!(action_result.contains("unknown signer is rejected"));
}

struct SourceBundleWorkflow {
    _root: tempfile::TempDir,
    project: PathBuf,
    build_root: PathBuf,
    bundle: PathBuf,
    producer_state: PathBuf,
    consumer_state: PathBuf,
    tampered_state: PathBuf,
}

impl SourceBundleWorkflow {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let build_root = root.path().join("source-root.ncl");
        write_source_bundle_build_root(&build_root);
        let workflow = Self {
            project: project_root(OFFLINE_SOURCE_BUNDLE_PROJECT),
            build_root,
            bundle: root.path().join("source-bundle.json"),
            producer_state: root.path().join("producer-state"),
            consumer_state: root.path().join("consumer-state"),
            tampered_state: root.path().join("tampered-state"),
            _root: root,
        };
        assert!(workflow.project.is_dir());
        assert!(workflow.build_root.is_file());
        workflow
    }

    fn command(&self, state: &Path) -> Command {
        assert!(self.project.is_dir());
        assert!(state.is_absolute());
        let mut command = mantle_cmd();
        command.current_dir(&self.project).arg("--json").arg("--state-dir").arg(state);
        command
    }

    fn plan(&self) -> serde_json::Value {
        let output = self
            .command(&self.producer_state)
            .args(["source", "bundle", "plan", "--build-root"])
            .arg(&self.build_root)
            .output()
            .unwrap();
        parse_successful_json(output, "source bundle plan")
    }

    fn export(&self) -> serde_json::Value {
        let output = self
            .command(&self.producer_state)
            .args(["source", "bundle", "export", "--build-root"])
            .arg(&self.build_root)
            .arg("--to")
            .arg(&self.bundle)
            .output()
            .unwrap();
        parse_successful_json(output, "source bundle export")
    }

    fn list(&self) -> serde_json::Value {
        let output = self
            .command(&self.consumer_state)
            .args(["source", "bundle", "list", "--from"])
            .arg(&self.bundle)
            .output()
            .unwrap();
        parse_successful_json(output, "source bundle list")
    }

    fn verify(&self) -> serde_json::Value {
        let output = self
            .command(&self.consumer_state)
            .args(["source", "bundle", "verify", "--from"])
            .arg(&self.bundle)
            .arg("--imported")
            .output()
            .unwrap();
        parse_successful_json(output, "source bundle verify")
    }

    fn import_and_pin(&self) -> serde_json::Value {
        let output = self
            .command(&self.consumer_state)
            .args(["source", "bundle", "import", "--from"])
            .arg(&self.bundle)
            .arg("--pin")
            .output()
            .unwrap();
        parse_successful_json(output, "source bundle import")
    }

    fn preflight(&self) -> serde_json::Value {
        let output = self
            .command(&self.consumer_state)
            .args(["source", "bundle", "preflight", "--build-root"])
            .arg(&self.build_root)
            .output()
            .unwrap();
        parse_successful_json(output, "source bundle preflight")
    }

    fn import_tampered_bundle(&self) -> std::process::Output {
        let mut tampered: serde_json::Value = serde_json::from_slice(&std::fs::read(&self.bundle).unwrap()).unwrap();
        let content_hex = tampered["records"][0]["files"][0]["content_hex"].as_str().unwrap().to_string();
        assert!(content_hex.len() >= HEX_BYTE_TEXT_WIDTH);
        let replacement = if content_hex.starts_with("00") { "ff" } else { "00" };
        let mut changed_hex = content_hex;
        changed_hex.replace_range(..HEX_BYTE_TEXT_WIDTH, replacement);
        tampered["records"][0]["files"][0]["content_hex"] = serde_json::Value::String(changed_hex);
        let tampered_bundle = self._root.path().join("tampered-source-bundle.json");
        std::fs::write(&tampered_bundle, serde_json::to_vec_pretty(&tampered).unwrap()).unwrap();
        self.command(&self.tampered_state)
            .args(["source", "bundle", "import", "--from"])
            .arg(tampered_bundle)
            .output()
            .unwrap()
    }
}

#[test]
fn offline_source_bundle_round_trips_and_rejects_tampering() {
    let workflow = SourceBundleWorkflow::new();
    let plan = workflow.plan();
    assert_eq!(plan["ready_class"], "ready");
    assert_eq!(plan["record_count"].as_u64(), Some(SOURCE_BUNDLE_RECORD_COUNT));
    assert!(!workflow.producer_state.join(SOURCE_BUNDLE_RECORDS_RELATIVE).exists());

    let exported = workflow.export();
    assert_eq!(exported["ready_class"], "ready");
    assert_eq!(exported["record_count"].as_u64(), Some(SOURCE_BUNDLE_RECORD_COUNT));
    assert!(workflow.bundle.is_file());
    let listed = workflow.list();
    assert_eq!(listed["record_count"].as_u64(), Some(SOURCE_BUNDLE_RECORD_COUNT));
    assert!(!workflow.consumer_state.join(SOURCE_BUNDLE_RECORDS_RELATIVE).exists());

    let missing = workflow.verify();
    assert_eq!(missing["ready_class"], "missing");
    assert_eq!(missing["missing_records"].as_array().unwrap().len(), SOURCE_BUNDLE_RECORD_COUNT_USIZE);
    let imported = workflow.import_and_pin();
    assert_eq!(imported["imported_count"].as_u64(), Some(SOURCE_BUNDLE_RECORD_COUNT));
    assert_eq!(imported["pinned"], true);
    let verified = workflow.verify();
    assert_eq!(verified["ready_class"], "ready");
    assert!(verified["missing_records"].as_array().unwrap().is_empty());

    let preflight = workflow.preflight();
    assert_eq!(preflight["ready_class"], "ready");
    assert_eq!(preflight["record_count"].as_u64(), Some(SOURCE_BUNDLE_RECORD_COUNT));
    assert!(preflight["source_state_blake3"].as_str().is_some_and(|digest| !digest.is_empty()));
    let tampered = workflow.import_tampered_bundle();
    assert!(!tampered.status.success(), "tampered source bundle unexpectedly imported");
    assert!(String::from_utf8_lossy(&tampered.stderr).contains("digest mismatch"));
    assert!(!workflow.tampered_state.join(SOURCE_BUNDLE_RECORDS_RELATIVE).exists());
}

#[test]
fn reviewed_file_generation_applies_current_plan_and_tracks_state() {
    let fixture = tempfile::tempdir().unwrap();
    copy_reviewed_filegen_project(fixture.path());
    let target_dir = fixture.path().join("target");
    std::fs::create_dir_all(&target_dir).unwrap();
    let plan_path = target_dir.join("filegen-plan.json");
    let plan = parse_successful_json(
        run_project_command(fixture.path(), &["--json", "filegen", "plan", "--plan-out", plan_path.to_str().unwrap()]),
        "filegen plan",
    );
    assert_eq!(plan["operations"].as_array().unwrap().len(), GENERATED_FILE_COUNT);
    assert!(plan["operations"].as_array().unwrap().iter().all(|operation| operation["action"] == "create"));
    assert!(!fixture.path().join("generated").exists(), "filegen plan must not write generated files");

    let applied = parse_successful_json(
        run_project_command(fixture.path(), &["--json", "filegen", "apply", "--plan", plan_path.to_str().unwrap()]),
        "filegen apply",
    );
    assert_eq!(applied["operations"].as_array().unwrap().len(), GENERATED_FILE_COUNT);
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.path().join("generated/app-config.json")).unwrap()).unwrap();
    assert_eq!(config["name"], "mantle-filegen-demo");
    assert_eq!(config["mode"], "reviewed");
    assert_eq!(
        std::fs::read_to_string(fixture.path().join("generated/README.txt")).unwrap(),
        "generated only after a reviewed Mantle plan\n"
    );
    let state: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.path().join(".mantle/filegen-state.json")).unwrap()).unwrap();
    assert_eq!(state["files"].as_object().unwrap().len(), GENERATED_FILE_COUNT);

    let second_plan = parse_successful_json(
        run_project_command(fixture.path(), &["--json", "filegen", "plan"]),
        "second filegen plan",
    );
    assert!(
        second_plan["operations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|operation| operation["action"] == "unchanged")
    );
}

#[test]
fn reviewed_file_generation_rejects_unsupported_schema_before_applying() {
    let fixture = tempfile::tempdir().unwrap();
    copy_reviewed_filegen_project(fixture.path());
    let reviewed = fixture.path().join("reviewed-plan.json");
    let mut plan = parse_successful_json(
        run_project_command(fixture.path(), &["--json", "filegen", "plan", "--plan-out", reviewed.to_str().unwrap()]),
        "filegen plan",
    );
    assert_eq!(plan["schema"], "mantle-project-filegen-plan-v1");
    assert!(!fixture.path().join("generated").exists());
    assert!(!fixture.path().join(".mantle/filegen-state.json").exists());

    plan["schema"] = serde_json::Value::String("mantle-project-filegen-plan-v999".to_string());
    let unsupported = fixture.path().join("unsupported-plan.json");
    std::fs::write(&unsupported, serde_json::to_vec_pretty(&plan).unwrap()).unwrap();
    let failed =
        run_project_command(fixture.path(), &["--json", "filegen", "apply", "--plan", unsupported.to_str().unwrap()]);
    assert_eq!(failed.status.code(), Some(3));
    assert!(failed.stderr.is_empty(), "rejected reviewed plan must return a blocker, not panic");
    let blockers: serde_json::Value = serde_json::from_slice(&failed.stdout).unwrap();
    let blockers = blockers.as_array().unwrap();
    assert_eq!(blockers.len(), 1);
    assert_eq!(blockers[0]["code"], "unsupported-reviewed-plan-schema");
    assert_eq!(blockers[0]["target"], "<plan>");
    let message = blockers[0]["message"].as_str().unwrap();
    assert!(message.contains("mantle-project-filegen-plan-v999"), "{message}");
    assert!(message.contains("mantle-project-filegen-plan-v1"), "{message}");
    assert!(!fixture.path().join("generated").exists());
    assert!(!fixture.path().join(".mantle/filegen-state.json").exists());
}

#[test]
fn reviewed_file_generation_rejects_drift_conflict_and_escape() {
    let drift_fixture = tempfile::tempdir().unwrap();
    copy_reviewed_filegen_project(drift_fixture.path());
    let plan_path = drift_fixture.path().join("filegen-plan.json");
    parse_successful_json(
        run_project_command(drift_fixture.path(), &[
            "--json",
            "filegen",
            "plan",
            "--plan-out",
            plan_path.to_str().unwrap(),
        ]),
        "filegen drift plan",
    );
    let manifest_path = drift_fixture.path().join("mantle-project.ncl");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let changed = manifest.replace("\"mode\":\"reviewed\"", "\"mode\":\"changed\"");
    assert_ne!(changed, manifest, "drift fixture replacement must change the manifest");
    std::fs::write(&manifest_path, changed).unwrap();
    let drift = parse_failed_json(
        run_project_command(drift_fixture.path(), &[
            "--json",
            "filegen",
            "apply",
            "--plan",
            plan_path.to_str().unwrap(),
        ]),
        "stale filegen apply",
    );
    assert!(drift.as_array().unwrap().iter().any(|blocker| blocker["code"] == "plan-drift"));
    assert!(!drift_fixture.path().join("generated").exists());

    let conflict_fixture = tempfile::tempdir().unwrap();
    copy_reviewed_filegen_project(conflict_fixture.path());
    std::fs::create_dir_all(conflict_fixture.path().join("generated")).unwrap();
    std::fs::write(conflict_fixture.path().join("generated/app-config.json"), b"unmanaged\n").unwrap();
    let conflict = parse_failed_json(
        run_project_command(conflict_fixture.path(), &["--json", "filegen", "plan"]),
        "filegen conflict plan",
    );
    assert!(
        conflict["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker["code"] == "existing-file-conflict")
    );

    let escape_workspace = tempfile::tempdir().unwrap();
    let escape_project = escape_workspace.path().join("project");
    std::fs::create_dir_all(&escape_project).unwrap();
    copy_reviewed_filegen_project(&escape_project);
    let escape = parse_failed_json(
        run_project_command(&escape_project, &[
            "--json",
            "filegen",
            "plan",
            "--manifest",
            "fixtures/target-escape.ncl",
        ]),
        "filegen target escape plan",
    );
    assert!(escape["blockers"].as_array().unwrap().iter().any(|blocker| blocker["code"] == "target-escape"));
    assert!(!escape_workspace.path().join("escaped-config.json").exists());
}

#[test]
fn reviewed_file_generation_rejects_missing_contract_field() {
    let fixture = tempfile::tempdir().unwrap();
    copy_reviewed_filegen_project(fixture.path());
    let invalid = parse_failed_json(
        run_project_command(fixture.path(), &[
            "--json",
            "filegen",
            "plan",
            "--manifest",
            "fixtures/missing-required-field.ncl",
        ]),
        "filegen missing field plan",
    );
    assert!(
        invalid["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|blocker| blocker["code"] == "contract-invalid-content")
    );
    assert!(!fixture.path().join("generated/incomplete.json").exists());
}

fn developer_workflow_command(store: &Path, state: &Path) -> Command {
    assert!(store.is_dir(), "developer workflow store must exist");
    assert!(state.is_dir(), "developer workflow state must exist");
    let mut command = project_command_with_build_environment(DEVELOPER_SHELL_RUN_PROJECT);
    command.arg("--store").arg(store).arg("--state-dir").arg(state);
    command
}

#[test]
fn developer_shell_and_run_loop_executes_named_profiles() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: developer shell/run project requires Linux, bwrap, and static BusyBox");
        return;
    }
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let run = developer_workflow_command(store.path(), state.path())
        .args(["run", ".#tool", "--no-substitute", "--", "Mantle"])
        .output()
        .unwrap();
    assert!(run.status.success(), "mantle run failed: {}", String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "operator-demo: hello, Mantle");

    let dev = developer_workflow_command(store.path(), state.path())
        .env("MANTLE_EXAMPLE_PROFILE", "ambient")
        .args(["shell", "--no-substitute", "--command", "env"])
        .output()
        .unwrap();
    assert!(dev.status.success(), "default shell failed: {}", String::from_utf8_lossy(&dev.stderr));
    let dev_stdout = String::from_utf8_lossy(&dev.stdout);
    assert!(dev_stdout.contains("MANTLE_EXAMPLE_PROFILE=dev"));
    assert!(dev_stdout.contains("MANTLE_EXAMPLE_MESSAGE=development tools are active"));
    assert!(!dev_stdout.contains("MANTLE_EXAMPLE_PROFILE=ambient"));
    assert!(String::from_utf8_lossy(&dev.stderr).contains("developer-shell-hook: active"));

    let minimal = developer_workflow_command(store.path(), state.path())
        .args(["shell", ".#minimal", "--no-substitute", "--command", "env"])
        .output()
        .unwrap();
    assert!(minimal.status.success(), "minimal shell failed: {}", String::from_utf8_lossy(&minimal.stderr));
    assert!(String::from_utf8_lossy(&minimal.stdout).contains("MANTLE_EXAMPLE_PROFILE=minimal"));
    assert!(!String::from_utf8_lossy(&minimal.stderr).contains("developer-shell-hook: active"));
}

#[test]
fn developer_shell_and_run_loop_rejects_invalid_selection_and_arguments() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: developer shell/run project requires Linux, bwrap, and static BusyBox");
        return;
    }
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let missing = developer_workflow_command(store.path(), state.path())
        .args(["shell", ".#missing", "--no-substitute", "--command", "env"])
        .output()
        .unwrap();
    assert!(!missing.status.success(), "missing shell profile unexpectedly activated");
    assert!(String::from_utf8_lossy(&missing.stderr).contains("missing"));

    let invalid_run = developer_workflow_command(store.path(), state.path())
        .args(["run", ".#tool", "--no-substitute"])
        .output()
        .unwrap();
    assert!(!invalid_run.status.success(), "missing tool argument unexpectedly succeeded");
    assert!(String::from_utf8_lossy(&invalid_run.stderr).contains("usage: operator-demo NAME"));
}

struct ArtifactProvenanceWorkflow {
    _root: tempfile::TempDir,
    store: PathBuf,
    state: PathBuf,
}

impl ArtifactProvenanceWorkflow {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let store = root.path().join("store");
        let state = root.path().join("state");
        std::fs::create_dir_all(&store).unwrap();
        std::fs::create_dir_all(&state).unwrap();
        assert!(store.is_dir());
        assert!(state.is_dir());
        Self {
            _root: root,
            store,
            state,
        }
    }

    fn build(&self) -> serde_json::Value {
        parse_successful_json(
            workflow_build_command(ARTIFACT_PROVENANCE_PROJECT, &self.store, &self.state)
                .args(["--no-substitute", ".#artifact"])
                .output()
                .unwrap(),
            "artifact provenance build",
        )
    }

    fn attest(&self, args: &[&str]) -> std::process::Output {
        assert!(!args.is_empty(), "attestation arguments must not be empty");
        workflow_attest_command(ARTIFACT_PROVENANCE_PROJECT, &self.store, &self.state)
            .args(args)
            .output()
            .unwrap()
    }
}

#[test]
fn artifact_provenance_walkthrough_inspects_verifies_and_diffs_evidence() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: artifact provenance workflow requires Linux, bwrap, and static BusyBox");
        return;
    }
    let workflow = ArtifactProvenanceWorkflow::new();
    let build = workflow.build();
    assert_eq!(build["counts"]["succeeded_total"], 1);
    let artifact_path = build["outcomes"][0]["outputs"][0]["path"].as_str().unwrap();
    assert!(
        std::fs::read_to_string(Path::new(artifact_path).join("artifact.txt"))
            .unwrap()
            .contains("assembled")
    );

    let show = parse_successful_json(workflow.attest(&["show", artifact_path]), "attest show");
    assert_eq!(show["kind"], "artifact");
    assert!(show["attestation"]["edges"].as_array().unwrap().iter().any(|edge| edge["kind"] == "build-input"));
    let source_logical = show["attestation"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|node| {
            node["attributes"]["logical_path"]
                .as_str()
                .filter(|path| path.contains("-provenance-walkthrough-source"))
        })
        .unwrap();
    let closure = parse_successful_json(workflow.attest(&["closure", artifact_path]), "attest closure");
    assert_eq!(closure["attestation"]["facts"]["members"].as_array().unwrap().len(), 2);

    for (kind, selector) in [("artifact", artifact_path), ("closure", artifact_path)] {
        let verify = workflow.attest(&["verify", kind, selector]);
        assert!(verify.status.success(), "{kind} verify failed: {}", String::from_utf8_lossy(&verify.stderr));
        assert!(String::from_utf8_lossy(&verify.stdout).contains(&format!("OK {kind} digest=")));
    }
    let diff = workflow.attest(&["diff", source_logical, artifact_path]);
    assert!(diff.status.success(), "attestation diff failed: {}", String::from_utf8_lossy(&diff.stderr));
    let diff_stdout = String::from_utf8_lossy(&diff.stdout);
    assert!(diff_stdout.contains("---") && diff_stdout.contains("+++"));
}

#[test]
fn artifact_provenance_walkthrough_rejects_tampered_and_missing_evidence() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: artifact provenance workflow requires Linux, bwrap, and static BusyBox");
        return;
    }
    let workflow = ArtifactProvenanceWorkflow::new();
    let build = workflow.build();
    let artifact_path = build["outcomes"][0]["outputs"][0]["path"].as_str().unwrap();
    let show = parse_successful_json(workflow.attest(&["show", artifact_path]), "attest show before tamper");
    let sidecar = PathBuf::from(show["stored_path"].as_str().unwrap());
    let original = std::fs::read(&sidecar).unwrap();
    std::fs::OpenOptions::new().append(true).open(&sidecar).unwrap().write_all(b"tampered").unwrap();
    let artifact_verify = workflow.attest(&["verify", "artifact", artifact_path]);
    assert!(!artifact_verify.status.success(), "tampered artifact sidecar unexpectedly verified");
    assert!(!String::from_utf8_lossy(&artifact_verify.stderr).trim().is_empty());
    std::fs::write(&sidecar, original).unwrap();

    let closure = parse_successful_json(workflow.attest(&["closure", artifact_path]), "attest closure before tamper");
    let closure_sidecar = PathBuf::from(closure["stored_path"].as_str().unwrap());
    let original_closure = std::fs::read(&closure_sidecar).unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(&closure_sidecar)
        .unwrap()
        .write_all(b"tampered")
        .unwrap();
    let closure_verify = workflow.attest(&["verify", "closure", artifact_path]);
    assert!(!closure_verify.status.success(), "tampered closure sidecar unexpectedly verified");
    assert!(!String::from_utf8_lossy(&closure_verify.stderr).trim().is_empty());

    let mut stale_membership: serde_json::Value = serde_json::from_slice(&original_closure).unwrap();
    let root_node_id = stale_membership["facts"]["root_node_ids"][0].as_str().unwrap().to_string();
    let members = stale_membership["facts"]["members"].as_array_mut().unwrap();
    assert_eq!(members.len(), 2);
    let root_member_index = members.iter().position(|member| member["node_id"] == root_node_id).unwrap();
    members.remove(root_member_index);
    assert_eq!(members.len(), 1);
    let stale_attestation: ClosureAttestation = serde_json::from_value(stale_membership.clone()).unwrap();
    let stale_error = stale_attestation.canonical_bytes().unwrap_err();
    assert!(stale_error.to_string().contains("member"));
    std::fs::write(&closure_sidecar, serde_json::to_vec(&stale_membership).unwrap()).unwrap();
    let stale_verify = workflow.attest(&["verify", "closure", artifact_path]);
    assert!(!stale_verify.status.success(), "stale closure root membership unexpectedly verified");
    assert!(String::from_utf8_lossy(&stale_verify.stderr).contains("closure root missing from members"));

    let missing = workflow.attest(&["show", "/mantle/store/00000000000000000000000000000000-missing"]);
    assert!(!missing.status.success(), "missing artifact selector unexpectedly resolved");
    assert!(String::from_utf8_lossy(&missing.stderr).contains("no artifact attestation"));
}

struct HermeticPlanWorkflow {
    _root: tempfile::TempDir,
    plan_store: PathBuf,
    plan_state: PathBuf,
    plan_tools: PathBuf,
    first_store: PathBuf,
    first_state: PathBuf,
    second_store: PathBuf,
    second_state: PathBuf,
}

impl HermeticPlanWorkflow {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let make_dir = |name: &str| {
            let path = root.path().join(name);
            std::fs::create_dir_all(&path).unwrap();
            path
        };
        let plan_tools = make_dir("plan-tools");
        let fusermount = plan_tools.join("fusermount3");
        std::fs::write(&fusermount, b"#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        std::fs::set_permissions(&fusermount, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).unwrap();
        let workflow = Self {
            plan_store: make_dir("plan-store"),
            plan_state: make_dir("plan-state"),
            plan_tools,
            first_store: make_dir("first-store"),
            first_state: make_dir("first-state"),
            second_store: make_dir("second-store"),
            second_state: make_dir("second-state"),
            _root: root,
        };
        assert!(workflow.plan_store.is_dir());
        assert!(workflow.second_state.is_dir());
        workflow
    }

    fn plan(&self, target: &str, nix_compat: bool) -> serde_json::Value {
        let mut command = workflow_build_command(HERMETIC_PLAN_REBUILD_PROJECT, &self.plan_store, &self.plan_state);
        let path = std::env::join_paths(
            std::iter::once(self.plan_tools.clone())
                .chain(find_bwrap().and_then(|path| path.parent().map(Path::to_path_buf)))
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())),
        )
        .unwrap();
        command.env("PATH", path);
        if nix_compat {
            command.arg("--nix-compat");
        }
        parse_successful_json(
            command.args(["--plan", "--strict-hermetic", "--no-substitute", target]).output().unwrap(),
            "strict build plan",
        )
    }

    fn build(&self, store: &Path, state: &Path, host_token: &str) -> serde_json::Value {
        parse_successful_json(
            workflow_build_command(HERMETIC_PLAN_REBUILD_PROJECT, store, state)
                .env("MANTLE_EXAMPLE_HOST_TOKEN", host_token)
                .args(["--strict-hermetic", "--no-substitute", ".#payload"])
                .output()
                .unwrap(),
            "strict clean build",
        )
    }
}

#[test]
fn hermetic_plan_and_rebuild_match_across_ambient_host_changes() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: hermetic plan workflow requires Linux, bwrap, and static BusyBox");
        return;
    }
    let workflow = HermeticPlanWorkflow::new();
    let plan = workflow.plan(".#payload", false);
    assert_eq!(plan["entries"][0]["action"], "build");
    assert_eq!(std::fs::read_dir(&workflow.plan_store).unwrap().count(), 0);
    let first = workflow.build(&workflow.first_store, &workflow.first_state, "first-host-value");
    let second = workflow.build(&workflow.second_store, &workflow.second_state, "second-host-value");
    assert_eq!(first["hermeticity_mode"], "strict");
    assert_eq!(second["hermeticity_audit_events"], serde_json::json!([]));
    assert_eq!(plan["entries"][0]["drv_key"], first["outcomes"][0]["drv_key"]);
    assert_eq!(first["outcomes"][0]["drv_key"], second["outcomes"][0]["drv_key"]);
    assert_eq!(
        first["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"],
        second["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"]
    );
    assert_eq!(
        first["build_environment_reports"][0]["digest_blake3"],
        second["build_environment_reports"][0]["digest_blake3"]
    );
    assert_eq!(first["build_environment_reports"][0]["determinism"]["strong_claim_blocked"], false);
    let first_output = PathBuf::from(first["outcomes"][0]["outputs"][0]["path"].as_str().unwrap());
    let second_output = PathBuf::from(second["outcomes"][0]["outputs"][0]["path"].as_str().unwrap());
    assert_eq!(
        std::fs::read(first_output.join("payload.txt")).unwrap(),
        std::fs::read(second_output.join("payload.txt")).unwrap()
    );
    assert_eq!(
        std::fs::read(first_output.join("environment.txt")).unwrap(),
        std::fs::read(second_output.join("environment.txt")).unwrap()
    );
}

#[test]
fn hermetic_plan_rejects_impure_mode_and_scopes_identity_to_inputs_and_store_prefix() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: hermetic plan workflow requires Linux, bwrap, and static BusyBox");
        return;
    }
    let workflow = HermeticPlanWorkflow::new();
    let default_plan = workflow.plan(".#payload", false);
    let changed_plan = workflow.plan("fixtures/changed-payload.ncl", false);
    let nix_plan = workflow.plan(".#payload", true);
    assert_ne!(default_plan["entries"][0]["drv_key"], changed_plan["entries"][0]["drv_key"]);
    assert_ne!(default_plan["entries"][0]["drv_key"], nix_plan["entries"][0]["drv_key"]);
    assert_eq!(default_plan["store_dir"], "/mantle/store");
    assert_eq!(nix_plan["store_dir"], "/nix/store");

    let conflict = workflow_build_command(HERMETIC_PLAN_REBUILD_PROJECT, &workflow.plan_store, &workflow.plan_state)
        .args(["--strict-hermetic", "--impure", "--no-substitute", ".#payload"])
        .output()
        .unwrap();
    assert!(!conflict.status.success(), "strict plus impure unexpectedly executed");
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("cannot be used with"));
    assert_eq!(std::fs::read_dir(&workflow.plan_store).unwrap().count(), 0);
}

fn project_action_result_http_layout(producer_state: &Path, cache: &Path) {
    assert!(producer_state.is_dir());
    assert!(cache.is_dir());
    let local_root = producer_state.join("action-results/v1");
    let mut records = std::fs::read_dir(local_root.join("records"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    let mut action_dirs = std::fs::read_dir(local_root.join("indexes"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    records.sort();
    action_dirs.sort();
    assert_eq!(records.len(), 1, "fixture must publish exactly one record");
    assert_eq!(action_dirs.len(), 1, "fixture must publish exactly one action index");
    let signed: crunch_action_result_core::SignedActionResultRecord =
        serde_json::from_slice(&std::fs::read(&records[0]).unwrap()).unwrap();
    let result_digest =
        signed.record.result_ref.strip_prefix(crunch_action_result_core::ACTION_RESULT_REF_PREFIX).unwrap();
    let marker = action_dirs[0].join(format!("{result_digest}.ref"));
    assert_eq!(std::fs::read_to_string(marker).unwrap().trim(), signed.record.result_ref);
    let index = crunch_action_result_core::canonical_action_result_index(signed.record.action_ref.clone(), vec![
        signed.record.result_ref.clone(),
    ])
    .unwrap();
    let http_root = cache.join("action-results/v1");
    std::fs::create_dir_all(http_root.join("records")).unwrap();
    std::fs::create_dir_all(http_root.join("indexes")).unwrap();
    let action_digest = signed.record.action_ref.strip_prefix(crunch_action_result_core::ACTION_REF_PREFIX).unwrap();
    std::fs::write(
        http_root.join("indexes").join(format!("{action_digest}.json")),
        crunch_action_result_core::canonical_index_bytes(&index).unwrap(),
    )
    .unwrap();
    std::fs::write(
        http_root.join("records").join(format!("{result_digest}.json")),
        crunch_action_result_core::canonical_signed_record_bytes(&signed).unwrap(),
    )
    .unwrap();
}

struct ActionResultWorkflow {
    _root: tempfile::TempDir,
    producer_store: PathBuf,
    producer_state: PathBuf,
    cache: PathBuf,
}

struct ActionResultConsumer {
    store: tempfile::TempDir,
    state: tempfile::TempDir,
    report: serde_json::Value,
}

impl ActionResultWorkflow {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let producer_store = root.path().join("producer-store");
        let producer_state = root.path().join("producer-state");
        let cache = root.path().join("cache");
        for path in [&producer_store, &producer_state, &cache] {
            std::fs::create_dir_all(path).unwrap();
        }
        assert!(producer_store.is_dir());
        assert!(cache.is_dir());
        Self {
            _root: root,
            producer_store,
            producer_state,
            cache,
        }
    }

    fn publish(&self) -> serde_json::Value {
        let key = project_root(SHARED_ACTION_RESULT_PROJECT).join("fixtures/action.key");
        let report = parse_successful_json(
            workflow_build_command(SHARED_ACTION_RESULT_PROJECT, &self.producer_store, &self.producer_state)
                .args(["--nix-compat", "--no-substitute", "--signing-key"])
                .arg(&key)
                .arg(".#payload")
                .output()
                .unwrap(),
            "shared action-result producer build",
        );
        let mut push = project_command_with_build_environment(SHARED_ACTION_RESULT_PROJECT);
        let pushed = push
            .arg("--store")
            .arg(&self.producer_store)
            .arg("--state-dir")
            .arg(&self.producer_state)
            .arg("--nix-compat")
            .args(["store", "push", "--to"])
            .arg(&self.cache)
            .arg("--all")
            .output()
            .unwrap();
        assert!(pushed.status.success(), "cache push failed: {}", String::from_utf8_lossy(&pushed.stderr));
        project_action_result_http_layout(&self.producer_state, &self.cache);
        report
    }

    fn consume(&self, server: &StaticCacheServer, selector: &str, trusted_key: &str) -> ActionResultConsumer {
        assert!(server.base_url.starts_with("http://"));
        assert!(!trusted_key.is_empty());
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let report = parse_successful_json(
            workflow_build_command(SHARED_ACTION_RESULT_PROJECT, store.path(), state.path())
                .args([
                    "--nix-compat",
                    "--substituters",
                    &server.base_url,
                    "--trusted-public-keys",
                    trusted_key,
                    selector,
                ])
                .output()
                .unwrap(),
            "shared action-result consumer build",
        );
        ActionResultConsumer { store, state, report }
    }

    fn only_nar(&self) -> PathBuf {
        let nar_dir = self.cache.join("nar");
        let mut nars = std::fs::read_dir(&nar_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "nar"))
            .collect::<Vec<_>>();
        nars.sort();
        assert!(nar_dir.is_dir());
        assert_eq!(nars.len(), 1, "fixture must publish exactly one NAR");
        nars.pop().unwrap()
    }

    fn corrupt_only_nar(&self) {
        std::fs::OpenOptions::new()
            .append(true)
            .open(self.only_nar())
            .unwrap()
            .write_all(b"corrupted")
            .unwrap();
    }

    fn remove_only_nar(&self) {
        std::fs::remove_file(self.only_nar()).unwrap();
        assert!(std::fs::read_dir(self.cache.join("nar")).unwrap().next().is_none());
    }
}

fn load_workflow_path_info(state: &Path, logical_path: &str, store_prefix: &str) -> snix_store::path_info::PathInfo {
    let (store_path, suffix) =
        nix_compat::store_path::StorePath::<String>::from_absolute_path_full_with_prefix(logical_path, store_prefix)
            .unwrap();
    assert!(suffix.as_os_str().is_empty());
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let service = snix_store::pathinfoservice::RedbPathInfoService::new(
            "example-pathinfo-verification".to_string(),
            snix_store::pathinfoservice::RedbPathInfoServiceConfig {
                path: Some(state.join("pathinfo.redb")),
                read_only: true,
                cache_size: None,
            },
        )
        .await
        .unwrap();
        snix_store::pathinfoservice::PathInfoService::get(&service, *store_path.digest())
            .await
            .unwrap()
            .unwrap()
    })
}

fn action_result_report_with_disposition<'a>(
    report: &'a serde_json::Value,
    disposition: &str,
) -> &'a serde_json::Value {
    assert!(!disposition.is_empty());
    assert!(report["action_result_reports"].is_array());
    report["action_result_reports"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["disposition"] == disposition)
        .unwrap_or_else(|| {
            panic!("missing action-result disposition {disposition}: {}", report["action_result_reports"])
        })
}

#[test]
fn shared_action_result_roundtrip_reuses_trusted_http_result_and_separates_action_identity() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: shared action-result workflow requires Linux, bwrap, static BusyBox, and loopback HTTP");
        return;
    }
    let workflow = ActionResultWorkflow::new();
    let producer = workflow.publish();
    assert_eq!(producer["counts"]["built_total"], 1);
    assert!(action_result_report_with_disposition(&producer, "published")["publication_result_refs"].is_array());
    let server = StaticCacheServer::start(&workflow.cache);

    let trusted = workflow.consume(&server, ".#payload", ACTION_RESULT_TRUSTED_KEY);
    assert_eq!(trusted.report["counts"]["built_total"], 0);
    assert_eq!(trusted.report["counts"]["cached_total"], 1);
    let producer_logical =
        producer["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"].as_str().unwrap();
    let consumer_logical = trusted.report["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"]
        .as_str()
        .unwrap();
    let producer_path_info = load_workflow_path_info(&workflow.producer_state, producer_logical, "/nix/store");
    let consumer_path_info = load_workflow_path_info(trusted.state.path(), consumer_logical, "/nix/store");
    assert_eq!(producer_path_info.store_path, consumer_path_info.store_path);
    assert_eq!(producer_path_info.node, consumer_path_info.node);
    assert_eq!(producer_path_info.references, consumer_path_info.references);
    assert_eq!(producer_path_info.nar_size, consumer_path_info.nar_size);
    assert_eq!(producer_path_info.nar_sha256, consumer_path_info.nar_sha256);
    assert_eq!(producer_path_info.signatures, consumer_path_info.signatures);
    assert_eq!(producer_path_info.deriver, consumer_path_info.deriver);
    let reused = action_result_report_with_disposition(&trusted.report, "reused");
    assert_eq!(reused["selected_source_class"], "http");
    assert!(reused["trust_basis"].as_array().unwrap().iter().any(|basis| basis == "action.example.com-1"));
    let output_path = trusted.report["outcomes"][0]["outputs"][0]["path"].as_str().unwrap();
    assert_eq!(
        std::fs::read_to_string(Path::new(output_path).join("result.txt")).unwrap(),
        "shared-action-result: original action\n"
    );
    let verify = workflow_attest_command(SHARED_ACTION_RESULT_PROJECT, trusted.store.path(), trusted.state.path())
        .args(["--nix-compat", "verify", "artifact", output_path])
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "reused artifact verification failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );

    for target in [".#changed_command", ".#changed_environment"] {
        let changed = workflow.consume(&server, target, ACTION_RESULT_TRUSTED_KEY);
        assert_eq!(changed.report["counts"]["built_total"], 1);
        let miss = action_result_report_with_disposition(&changed.report, "miss");
        assert_ne!(miss["action_ref"], reused["action_ref"]);
        assert!(
            !changed.report["action_result_reports"]
                .as_array()
                .unwrap()
                .iter()
                .any(|row| row["disposition"] == "reused")
        );
    }
}

#[test]
fn casita_local_action_result_reuses_verified_output_after_export_removal() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: Casita action-result reuse requires Linux, bwrap, and static BusyBox");
        return;
    }

    let root = tempfile::tempdir().unwrap();
    let store = root.path().join("store");
    let state = root.path().join("state");
    std::fs::create_dir(&store).unwrap();
    std::fs::create_dir(&state).unwrap();
    let policy = state.join("casita-trusted-public-keys");
    std::fs::write(&policy, format!("{ACTION_RESULT_TRUSTED_KEY}\n")).unwrap();
    let key = project_root(SHARED_ACTION_RESULT_PROJECT).join("fixtures/action.key");
    let build = || {
        parse_successful_json(
            project_command_with_build_environment(SHARED_ACTION_RESULT_PROJECT)
                .args(["--json", "--store-backend", "casita", "--store"])
                .arg(&store)
                .arg("--state-dir")
                .arg(&state)
                .args(["--nix-compat", "build", "--no-substitute", "--signing-key"])
                .arg(&key)
                .arg(".#payload")
                .output()
                .unwrap(),
            "Casita local action-result build",
        )
    };

    let first = build();
    assert_eq!(first["counts"]["built_total"], 1);
    let published = action_result_report_with_disposition(&first, "published");
    let output = PathBuf::from(first["outcomes"][0]["outputs"][0]["path"].as_str().unwrap());
    let logical = first["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"]
        .as_str()
        .unwrap();
    assert_eq!(std::fs::read_to_string(output.join("result.txt")).unwrap(), "shared-action-result: original action\n");
    let record_dir = state.join("action-results/v1/records");
    let records = std::fs::read_dir(record_dir).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(records.len(), 1, "builder must publish a real local action-result record");
    let signed: crunch_action_result_core::SignedActionResultRecord =
        serde_json::from_slice(&std::fs::read(records[0].path()).unwrap()).unwrap();
    assert_eq!(published["action_ref"].as_str(), Some(signed.record.action_ref.as_str()));
    assert_eq!(signed.record.outputs.len(), 1);
    assert_eq!(signed.record.outputs[0].store_path, logical);
    assert!(!signed.record.outputs[0].path_info_ref.is_empty());
    assert!(!signed.record_signatures.is_empty());

    #[cfg(unix)]
    std::fs::set_permissions(&output, std::fs::Permissions::from_mode(EXECUTABLE_MODE)).unwrap();
    std::fs::remove_dir_all(&output).unwrap();
    assert!(!output.exists());
    assert!(!state.join("pathinfo.redb").exists());
    assert!(!state.join("directories.redb").exists());
    assert!(!state.join("blobs").exists());
    assert!(state.join("casita/casita.sqlite").is_file());
    let selector = output.file_name().unwrap().to_str().unwrap();
    let info = parse_successful_json(
        project_command_with_build_environment(SHARED_ACTION_RESULT_PROJECT)
            .args(["--json", "--store-backend", "casita", "--state-dir"])
            .arg(&state)
            .arg("--store")
            .arg(&store)
            .args(["--nix-compat", "store", "info", selector])
            .output()
            .unwrap(),
        "Casita output-root lookup without physical export",
    );
    assert_eq!(info["backend"], "casita");
    assert_eq!(info["backend_capabilities"]["rust_unit_cache"], false);
    assert_eq!(info["paths"].as_array().unwrap().len(), 1);
    assert_eq!(info["paths"][0]["store_path"], selector);
    assert_eq!(info["paths"][0]["signatures"].as_array().unwrap().len(), 1);
    assert!(!output.exists(), "verified root lookup must not substitute for Builder rehydration");

    let second = build();
    assert_eq!(second["counts"]["built_total"], 0, "fresh Builder must not execute the action again");
    assert_eq!(second["counts"]["cached_total"], 1);
    let reused = action_result_report_with_disposition(&second, "reused");
    assert_eq!(reused["action_ref"].as_str(), Some(signed.record.action_ref.as_str()));
    assert_eq!(reused["selected_result_ref"].as_str(), Some(signed.record.result_ref.as_str()));
    assert_eq!(reused["selected_source_class"], "local");
    assert_eq!(second["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"], logical);
    assert_eq!(std::fs::read_to_string(output.join("result.txt")).unwrap(), "shared-action-result: original action\n");
    assert_eq!(reused["transfer"]["transferred_nar_bytes"], 0);
    assert_eq!(reused["transfer"]["reused_nar_bytes"], info["paths"][0]["nar_size"]);
    project_command_with_build_environment(SHARED_ACTION_RESULT_PROJECT)
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&state)
        .arg("--store")
        .arg(&store)
        .args(["--nix-compat", "store", "verify", "--trusted-public-keys", ACTION_RESULT_TRUSTED_KEY, selector])
        .assert()
        .success()
        .stdout(predicates::str::contains("trusted_signatures=1/1"));
    assert_eq!(std::fs::read(&policy).unwrap(), format!("{ACTION_RESULT_TRUSTED_KEY}\n").into_bytes());
}

#[test]
fn shared_action_result_roundtrip_rejects_unknown_signer_missing_output_and_corrupt_artifact() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: shared action-result workflow requires Linux, bwrap, static BusyBox, and loopback HTTP");
        return;
    }
    let workflow = ActionResultWorkflow::new();
    workflow.publish();
    let server = StaticCacheServer::start(&workflow.cache);
    let untrusted = workflow.consume(&server, ".#payload", ACTION_RESULT_UNKNOWN_SIGNER_KEY);
    assert_eq!(untrusted.report["counts"]["built_total"], 1);
    assert!(
        untrusted.report["action_result_reports"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|row| row["candidate_decisions"].as_array().into_iter().flatten())
            .flat_map(|candidate| candidate["diagnostics"].as_array().into_iter().flatten())
            .any(|diagnostic| diagnostic == "action-result-record-signature-untrusted")
    );

    workflow.corrupt_only_nar();
    let corrupt = workflow.consume(&server, ".#payload", ACTION_RESULT_TRUSTED_KEY);
    assert_eq!(corrupt.report["counts"]["built_total"], 1);
    assert!(
        !corrupt.report["action_result_reports"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["disposition"] == "reused")
    );
    assert!(
        corrupt.report["action_result_reports"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|row| row["candidate_decisions"].as_array().into_iter().flatten())
            .flat_map(|candidate| candidate["diagnostics"].as_array().into_iter().flatten())
            .any(|diagnostic| diagnostic == "action-result-object-incomplete")
    );

    workflow.remove_only_nar();
    let missing = workflow.consume(&server, ".#payload", ACTION_RESULT_TRUSTED_KEY);
    assert_eq!(missing.report["counts"]["built_total"], 1);
    assert!(
        missing.report["action_result_reports"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|row| row["candidate_decisions"].as_array().into_iter().flatten())
            .flat_map(|candidate| candidate["diagnostics"].as_array().into_iter().flatten())
            .any(|diagnostic| diagnostic == "action-result-object-incomplete")
    );
}

#[test]
fn static_cache_request_paths_accept_normal_files_and_reject_escape() {
    assert_eq!(static_cache_relative_path("/nar/example.nar"), Some(PathBuf::from("nar/example.nar")));
    assert_eq!(static_cache_relative_path("/index.json?ignored=true"), Some(PathBuf::from("index.json")));
    assert_eq!(static_cache_relative_path("/../secret"), None);
    assert_eq!(static_cache_relative_path("//absolute"), None);
}

#[test]
fn locked_dependency_lifecycle_detects_staleness_refreshes_and_upgrades() {
    let fixture = tempfile::tempdir().unwrap();
    copy_locked_dependency_project(fixture.path());
    let check = run_project_command(fixture.path(), &["check"]);
    assert!(check.status.success(), "initial check failed: {}", String::from_utf8_lossy(&check.stderr));

    let lock_before = std::fs::read(fixture.path().join("mantle.lock")).unwrap();
    let retention_path = fixture.path().join(".mantle/retention.json");
    assert!(fixture.path().join(LOCK_RETENTION_ROOT_RECORD).is_file());
    let retention_before: serde_json::Value = serde_json::from_slice(&std::fs::read(&retention_path).unwrap()).unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(fixture.path().join("sources/message.txt"))
        .unwrap()
        .write_all(b"changed locally\n")
        .unwrap();
    let stale = run_project_command(fixture.path(), &["list-stale", "--no-network"]);
    assert!(stale.status.success(), "stale listing failed: {}", String::from_utf8_lossy(&stale.stderr));
    assert!(String::from_utf8_lossy(&stale.stdout).contains("message-source"));
    assert_eq!(std::fs::read(fixture.path().join("mantle.lock")).unwrap(), lock_before);

    let refresh = run_project_command(fixture.path(), &["refresh", "--no-network", "message-source"]);
    assert!(refresh.status.success(), "refresh failed: {}", String::from_utf8_lossy(&refresh.stderr));
    assert_ne!(std::fs::read(fixture.path().join("mantle.lock")).unwrap(), lock_before);
    let retention_after: serde_json::Value = serde_json::from_slice(&std::fs::read(&retention_path).unwrap()).unwrap();
    assert_ne!(retention_after["records"][0]["content_digest"], retention_before["records"][0]["content_digest"]);
    assert_eq!(
        retention_after["records"][0]["generation"].as_u64(),
        retention_before["records"][0]["generation"]
            .as_u64()
            .and_then(|generation| generation.checked_add(1))
    );
    let no_stale = run_project_command(fixture.path(), &["list-stale", "--no-network"]);
    assert!(no_stale.status.success());
    assert_eq!(String::from_utf8_lossy(&no_stale.stdout).trim(), "all inputs up to date");

    let lock_path = fixture.path().join("mantle.lock");
    let mut old_lock: serde_json::Value = serde_json::from_slice(&std::fs::read(&lock_path).unwrap()).unwrap();
    old_lock["version"] = serde_json::Value::String("0.9.0".to_string());
    std::fs::write(&lock_path, serde_json::to_vec_pretty(&old_lock).unwrap()).unwrap();
    let upgrade = run_project_command(fixture.path(), &["upgrade"]);
    assert!(upgrade.status.success(), "upgrade failed: {}", String::from_utf8_lossy(&upgrade.stderr));
    let upgraded: serde_json::Value = serde_json::from_slice(&std::fs::read(&lock_path).unwrap()).unwrap();
    assert_eq!(upgraded["version"], "1.0.0");
}

#[test]
fn locked_dependency_refresh_rejects_a_missing_local_patch() {
    let fixture = tempfile::tempdir().unwrap();
    copy_locked_dependency_project(fixture.path());
    let lock_path = fixture.path().join("mantle.lock");
    let mut lock: serde_json::Value = serde_json::from_slice(&std::fs::read(&lock_path).unwrap()).unwrap();
    lock["patches"] = serde_json::json!({});
    std::fs::write(&lock_path, serde_json::to_vec_pretty(&lock).unwrap()).unwrap();
    std::fs::remove_file(fixture.path().join("patches/message.patch")).unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(fixture.path().join("sources/message.txt"))
        .unwrap()
        .write_all(b"force refresh\n")
        .unwrap();

    let refresh = run_project_command(fixture.path(), &["refresh", "--no-network", "message-source"]);

    assert!(!refresh.status.success(), "missing patch unexpectedly refreshed");
    let stderr = String::from_utf8_lossy(&refresh.stderr);
    assert!(stderr.contains("message.patch"), "missing patch diagnostic omitted its path: {stderr}");
}

#[test]
fn locked_dependency_refresh_rejects_an_unresolved_revision() {
    let fixture = tempfile::tempdir().unwrap();
    copy_locked_dependency_project(fixture.path());
    std::fs::copy(fixture.path().join("fixtures/unresolved-revision.ncl"), fixture.path().join("mantle-project.ncl"))
        .unwrap();

    let refresh = run_project_command(fixture.path(), &["refresh", "--no-network", "unresolved-revision"]);

    assert!(!refresh.status.success(), "empty Git revision unexpectedly refreshed");
    let stderr = String::from_utf8_lossy(&refresh.stderr);
    assert!(stderr.contains("git rev must be 40 hex characters"), "unexpected revision diagnostic: {stderr}");
}

#[test]
fn generated_site_project_builds_package_and_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: generated site project requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(GENERATED_SITE_PROJECT, ".#site");
    assert_success(&package, "generated site package");
    let html = std::fs::read_to_string(output_path(&package).join("index.html")).unwrap();
    assert!(html.contains(SITE_TITLE), "generated site title missing: {html}");

    let check = run_project_build(GENERATED_SITE_PROJECT, ".#checks.site-content");
    assert_success(&check, "generated site check");
    let result = std::fs::read_to_string(output_path(&check).join("result.txt")).unwrap();
    assert_eq!(result.trim(), SITE_CHECK_RESULT);
}

#[test]
fn codegen_project_builds_runnable_package_and_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: codegen project requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(CODEGEN_PROJECT, ".#app");
    assert_success(&package, "codegen package");
    let binary = output_path(&package).join("bin/codegen-demo");
    let execution = StdCommand::new(&binary).output().unwrap();
    assert!(execution.status.success(), "generated application should run");
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), CODEGEN_MESSAGE);

    let check = run_project_build(CODEGEN_PROJECT, ".#checks.app");
    assert_success(&check, "codegen check");
    let result = std::fs::read_to_string(output_path(&check).join("result.txt")).unwrap();
    assert_eq!(result.trim(), CODEGEN_CHECK_RESULT);
}

#[test]
fn fixed_source_projects_assemble_authoritative_files() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: fixed source assembly requires Linux, bwrap, and static BusyBox");
        return;
    }

    let c_source = run_project_build(C_PROJECT, ".#source");
    assert_success(&c_source, "C source assembly");
    let c_output = output_path(&c_source);
    assert!(c_output.join("include/greet.h").is_file(), "assembled C header missing");
    assert!(c_output.join("tests/test_greet.c").is_file(), "assembled C test missing");

    let rust_source = run_project_build(RUST_PROJECT, ".#source");
    assert_success(&rust_source, "Rust source assembly");
    let rust_output = output_path(&rust_source);
    assert!(rust_output.join("Cargo.lock").is_file(), "assembled Rust lockfile missing");
    assert!(rust_output.join("greeting/src/lib.rs").is_file(), "assembled Rust library missing");

    let delta_source = run_project_build(DELTA_SUBSTITUTION_PROJECT, ".#source");
    assert_success(&delta_source, "delta adaptor source assembly");
    let delta_text = std::fs::read_to_string(output_path(&delta_source).join("demo.rs")).unwrap();
    assert!(delta_text.contains("DeltaAcceptanceMode::Delta"));

    let release_source = run_project_build(RELEASE_WITNESS_PROJECT, ".#source");
    assert_success(&release_source, "release witness source assembly");
    let release_text = std::fs::read_to_string(output_path(&release_source).join("demo.rs")).unwrap();
    assert!(release_text.contains("VerificationDirectory"));
}

#[test]
fn c_library_project_exports_and_keeps_positive_and_negative_tests() {
    let project = project_root(C_PROJECT);
    let header = std::fs::read_to_string(project.join("include/greet.h")).unwrap();
    let test_source = std::fs::read_to_string(project.join("tests/test_greet.c")).unwrap();
    assert!(header.contains("GreetStatus greet_format"), "public C API missing");
    assert!(test_source.contains(C_POSITIVE_TEST), "positive C test missing");
    assert!(test_source.contains(C_NEGATIVE_TEST), "negative C test missing");

    let mut command = mantle_cmd();
    let output = command.current_dir(project).args(["eval", "mantle-project.ncl"]).output().unwrap();
    assert!(output.status.success(), "C project should evaluate: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.stdout.is_empty(), "C project evaluation should export JSON");
}

#[test]
#[ignore = "realizes the pinned bootstrap C toolchain on first build"]
fn c_library_project_builds_library_cli_and_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: C project build requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(C_PROJECT, ".#greet");
    assert_success(&package, "C library package");
    let output = output_path(&package);
    assert!(output.join("lib/libgreet.a").is_file(), "static library missing");
    assert!(output.join("include/greet.h").is_file(), "public header missing");
    let execution = StdCommand::new(output.join("bin/greet")).arg("Mantle").output().unwrap();
    assert!(execution.status.success(), "C CLI should run");
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), C_GREETING);

    let check = run_project_build(C_PROJECT, ".#checks.test-greet");
    assert_success(&check, "C project check");
    let result = std::fs::read_to_string(output_path(&check).join("result.txt")).unwrap();
    assert_eq!(result.trim(), C_CHECK_RESULT);
}

#[test]
fn rust_workspace_project_exports_and_keeps_positive_and_negative_tests() {
    let project = project_root(RUST_PROJECT);
    let library_source = std::fs::read_to_string(project.join("greeting/src/lib.rs")).unwrap();
    let app_source = std::fs::read_to_string(project.join("workspace-app/src/main.rs")).unwrap();
    assert!(library_source.contains(RUST_POSITIVE_TEST), "positive Rust test missing");
    assert!(library_source.contains(RUST_NEGATIVE_TEST), "negative Rust test missing");
    assert!(app_source.contains("rejects_extra_arguments"), "negative CLI test missing");

    let mut command = mantle_cmd();
    let output = command.current_dir(project).args(["eval", "mantle-project.ncl"]).output().unwrap();
    assert!(output.status.success(), "Rust project should evaluate: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.stdout.is_empty(), "Rust project evaluation should export JSON");
}

#[test]
#[ignore = "realizes the source-built Rust, seed-toolchain, and musl closure on first build"]
fn rust_workspace_project_builds_package_and_smoke_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: Rust workspace build requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(RUST_PROJECT, ".#workspace-app");
    assert_success(&package, "Rust workspace package");
    let binary = output_path(&package).join("bin/workspace-app");
    let execution = StdCommand::new(&binary).arg("Mantle").output().unwrap();
    assert!(execution.status.success(), "Rust workspace application should run");
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), RUST_GREETING);

    let check = run_project_build(RUST_PROJECT, ".#checks.smoke");
    assert_success(&check, "Rust workspace check");
    let result = std::fs::read_to_string(output_path(&check).join("result.txt")).unwrap();
    assert_eq!(result.trim(), RUST_CHECK_RESULT);
}

#[test]
fn fetched_and_patched_project_declares_fixed_positive_and_negative_patches() {
    let project = project_root(FETCHED_PATCHED_PROJECT);
    let project_source = std::fs::read_to_string(project.join("mantle-project.ncl")).unwrap();
    let positive_patch = std::fs::read_to_string(project.join("patches/readme.patch")).unwrap();
    let negative_patch = std::fs::read_to_string(project.join("patches/invalid-context.patch")).unwrap();
    assert!(project_source.contains("crc64-2.0.0.crate"), "pinned upstream crate missing");
    assert!(project_source.contains("sha256-G2t3VkrTsVRXQrfE4NYeYPnQUgnzxpGz2eI3QlVE/rA="));
    assert!(project_source.contains("sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="));
    assert!(positive_patch.contains("patched reproducibly by the Mantle example project"));
    assert!(negative_patch.contains("DOES-NOT-EXIST"));
    assert_project_evaluates(FETCHED_PATCHED_PROJECT, "fetched and patched project");
}

#[test]
#[ignore = "uses the live crates.io fixed-output source"]
fn fetched_and_patched_project_builds_check_and_rejects_invalid_patch() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: fetched project requires Linux, bwrap, and static BusyBox");
        return;
    }

    let check = run_project_build(FETCHED_PATCHED_PROJECT, ".#checks.patch");
    assert_success(&check, "fetched and patched check");
    assert_eq!(check_result(&check), FETCHED_PATCHED_CHECK_RESULT);

    let invalid = run_project_build_verbose(FETCHED_PATCHED_PROJECT, ".#invalid-patch");
    let stderr = String::from_utf8_lossy(&invalid.output.stderr);
    assert!(!invalid.output.status.success(), "invalid patch unexpectedly succeeded");
    assert!(stderr.contains("DOES-NOT-EXIST"), "invalid patch error missing failed target: {stderr}");

    let invalid_hash = run_project_build_verbose(FETCHED_PATCHED_PROJECT, ".#invalid-hash");
    let stderr = String::from_utf8_lossy(&invalid_hash.output.stderr);
    assert!(!invalid_hash.output.status.success(), "invalid fixed-output hash unexpectedly succeeded");
    assert!(stderr.contains("hash mismatch"), "fixed-output mismatch diagnostic missing: {stderr}");
}

#[test]
fn multi_output_sdk_source_assembles_and_project_evaluates() {
    assert_project_evaluates(MULTI_OUTPUT_SDK_PROJECT, "multi-output SDK project");
    if !can_build_fast_projects() {
        eprintln!("SKIP: SDK source assembly requires Linux, bwrap, and static BusyBox");
        return;
    }

    let source = run_project_build(MULTI_OUTPUT_SDK_PROJECT, ".#source");
    assert_success(&source, "multi-output SDK source assembly");
    let output = output_path(&source);
    assert!(output.join("include/mantle_sdk.h").is_file(), "SDK header missing");
    assert!(output.join("tests/test_sdk.c").is_file(), "SDK negative tests missing");
    assert!(output.join("docs/mantle-sdk.1").is_file(), "SDK manual page missing");
}

#[test]
#[ignore = "realizes the pinned bootstrap C toolchain and all selected SDK outputs"]
fn multi_output_sdk_builds_outputs_and_selected_consumers() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: SDK project build requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(MULTI_OUTPUT_SDK_PROJECT, ".#sdk");
    assert_success(&package, "multi-output SDK package");
    let runtime = output_path_for_label(&package, "out");
    let development = output_path_for_label(&package, "dev");
    let documentation = output_path_for_label(&package, "doc");
    let debug = output_path_for_label(&package, "debug");
    assert!(runtime.join("bin/mantle-sdk-greet").is_file());
    assert!(development.join("include/mantle_sdk.h").is_file());
    assert!(development.join("lib/libmantle_sdk.a").is_file());
    assert!(documentation.join("share/man/man1/mantle-sdk-greet.1").is_file());
    assert!(debug.join("lib/debug/mantle-sdk-greet").is_file());
    let execution = StdCommand::new(runtime.join("bin/mantle-sdk-greet")).arg("Mantle").output().unwrap();
    assert!(execution.status.success(), "SDK runtime should execute");
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), "Hello, Mantle!");

    let runtime_check = run_project_build(MULTI_OUTPUT_SDK_PROJECT, ".#checks.runtime");
    assert_success(&runtime_check, "SDK runtime consumer");
    assert_eq!(check_result(&runtime_check), MULTI_OUTPUT_RUNTIME_CHECK_RESULT);
    let development_check = run_project_build(MULTI_OUTPUT_SDK_PROJECT, ".#checks.development");
    assert_success(&development_check, "SDK development consumer");
    assert_eq!(check_result(&development_check), MULTI_OUTPUT_DEVELOPMENT_CHECK_RESULT);
}

#[test]
fn schema_codegen_builds_bindings_and_rejects_invalid_schema() {
    assert_project_evaluates(SCHEMA_CODEGEN_PROJECT, "schema codegen project");
    if !can_build_fast_projects() {
        eprintln!("SKIP: schema generation requires Linux, bwrap, and static BusyBox");
        return;
    }

    let bindings = run_project_build(SCHEMA_CODEGEN_PROJECT, ".#bindings");
    assert_success(&bindings, "schema generated bindings");
    let output = output_path(&bindings);
    assert!(output.join("c/greeting.h").is_file(), "generated C header missing");
    assert!(output.join("rust/greeting.rs").is_file(), "generated Rust module missing");
    let manifest = std::fs::read_to_string(output.join("bindings.manifest")).unwrap();
    assert!(manifest.contains("schema_version=1"));
    assert!(manifest.contains("default_name=World"));

    let invalid = run_project_build_verbose(SCHEMA_CODEGEN_PROJECT, ".#invalid-schema");
    let stderr = String::from_utf8_lossy(&invalid.output.stderr);
    assert!(!invalid.output.status.success(), "invalid schema unexpectedly generated bindings");
    assert!(
        stderr.contains("requires non-empty prefix and default_name"),
        "invalid schema error missing: {stderr}"
    );

    let unsafe_schema = run_project_build_verbose(SCHEMA_CODEGEN_PROJECT, ".#unsafe-schema");
    let stderr = String::from_utf8_lossy(&unsafe_schema.output.stderr);
    assert!(!unsafe_schema.output.status.success(), "unsafe schema unexpectedly generated source");
    assert!(stderr.contains("unsupported characters"), "unsafe schema error missing: {stderr}");
}

#[test]
#[ignore = "realizes both bootstrap C and source-built Rust toolchains"]
fn schema_codegen_builds_both_languages_and_integration_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: schema integration requires Linux, bwrap, and static BusyBox");
        return;
    }

    let check = run_project_build(SCHEMA_CODEGEN_PROJECT, ".#checks.integration");
    assert_success(&check, "schema C/Rust integration check");
    assert_eq!(check_result(&check), SCHEMA_CODEGEN_CHECK_RESULT);
}

#[test]
fn reproducible_release_matches_blake3_and_detects_tampering() {
    assert_project_evaluates(REPRODUCIBLE_RELEASE_PROJECT, "reproducible release project");
    if !can_build_fast_projects() {
        eprintln!("SKIP: release project requires Linux, bwrap, and static BusyBox");
        return;
    }

    let release_a = run_project_build(REPRODUCIBLE_RELEASE_PROJECT, ".#release-a");
    let release_b = run_project_build(REPRODUCIBLE_RELEASE_PROJECT, ".#release-b");
    assert_success(&release_a, "release A");
    assert_success(&release_b, "release B");
    let archive_a = std::fs::read(output_path(&release_a).join("release-demo.tar")).unwrap();
    let archive_b = std::fs::read(output_path(&release_b).join("release-demo.tar")).unwrap();
    assert_eq!(archive_a, archive_b, "independent release archives differ");
    let digest = blake3::hash(&archive_a);
    let digest_sri = format!("blake3-{}", data_encoding::BASE64.encode(digest.as_bytes()));
    assert_eq!(digest_sri, RELEASE_ARCHIVE_BLAKE3);
    let sidecar = std::fs::read_to_string(output_path(&release_a).join("release-demo.tar.blake3")).unwrap();
    assert_eq!(sidecar.trim(), RELEASE_ARCHIVE_BLAKE3);

    let reproducibility = run_project_build(REPRODUCIBLE_RELEASE_PROJECT, ".#checks.reproducible");
    assert_success(&reproducibility, "reproducibility check");
    assert_eq!(check_result(&reproducibility), RELEASE_CHECK_RESULT);
    let tamper = run_project_build(REPRODUCIBLE_RELEASE_PROJECT, ".#checks.tamper-detection");
    assert_success(&tamper, "release tamper detection");
    assert_eq!(check_result(&tamper), RELEASE_TAMPER_CHECK_RESULT);
}

#[test]
fn missing_project_selector_fails_closed() {
    let run = run_project_build(GENERATED_SITE_PROJECT, MISSING_SELECTOR);
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(!run.output.status.success(), "missing selector should fail");
    assert!(stderr.contains("missing-package"), "selector error should name the missing package: {stderr}");
}
