//! End-to-end smoke tests: build derivations and verify outputs on disk.
//!
//! Each test uses `--store <tempdir>` so outputs land in a writable
//! directory without needing a writable /nix/store. Requires bwrap.

mod audit_support;

use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use audit_support::AuditArtifact;
use audit_support::write_command_audit;
use serde::Deserialize;

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should be built")
}

fn can_build() -> bool {
    Path::new("/nix/store").exists()
        && std::process::Command::new("bwrap").arg("--version").output().is_ok_and(|o| o.status.success())
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonReport {
    schema: String,
    counts: BuildJsonCounts,
    outcomes: Vec<BuildJsonOutcome>,
    failed: Vec<BuildJsonFailure>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonCounts {
    succeeded_total: u32,
    built_total: u32,
    cached_total: u32,
    failed_total: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonOutcome {
    label: String,
    cached: bool,
    outputs: Vec<BuildJsonOutput>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonOutput {
    name: String,
    path: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonFailure {
    root: String,
    phase: String,
    error_class: String,
    message: String,
    saved_log_path: Option<PathBuf>,
}

struct BuildRun {
    report: BuildJsonReport,
}

/// Run `crunch --json build` with `--store <dir>` and return the parsed
/// report. Uses a per-call state dir to isolate pathinfo.redb across tests.
/// Pass `state_dir` to share state between calls (e.g. for cache tests).
fn build_ncl(ncl_content: &str, store: &Path) -> BuildRun {
    build_ncl_with_state(ncl_content, store, None)
}

fn build_ncl_with_state(ncl_content: &str, store: &Path, state_dir: Option<&Path>) -> BuildRun {
    let work = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("test.ncl");
    std::fs::write(&ncl_file, ncl_content).unwrap();
    let resolved_state_dir = state_dir.map(PathBuf::from).unwrap_or_else(|| work.path().join("state"));

    let mut cmd = crunch_cmd();
    cmd.arg("--json");
    cmd.arg("--store").arg(store);
    cmd.arg("--state-dir").arg(&resolved_state_dir);
    cmd.arg("build").arg("--no-substitute").arg("-I").arg(work.path()).arg(&ncl_file);

    let command = vec![
        "crunch".to_string(),
        "--json".to_string(),
        "--store".to_string(),
        store.display().to_string(),
        "--state-dir".to_string(),
        resolved_state_dir.display().to_string(),
        "build".to_string(),
        "--no-substitute".to_string(),
        "-I".to_string(),
        work.path().display().to_string(),
        ncl_file.display().to_string(),
    ];

    let output = cmd.output().expect("should execute");
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let report: BuildJsonReport = serde_json::from_str(&stdout).unwrap_or_else(|err| {
        panic!("build stdout should be valid JSON report: {err}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    assert_eq!(report.schema, "crunch-build-report-v1");
    assert_eq!(report.counts.succeeded_total as usize, report.outcomes.len());
    assert_eq!(report.counts.failed_total as usize, report.failed.len());

    let artifacts: Vec<AuditArtifact<'_>> = report
        .outcomes
        .iter()
        .flat_map(|outcome| outcome.outputs.iter())
        .map(|output| AuditArtifact {
            label: output.name.as_str(),
            path: output.path.as_path(),
        })
        .collect();
    let _audit_dir = write_command_audit("smoke", "build", work.path(), &command, &output, &artifacts, &[
        ("CRUNCH_STATE_DIR", resolved_state_dir.display().to_string()),
        ("CRUNCH_STORE_DIR", store.display().to_string()),
    ])
    .unwrap();

    assert!(output.status.success(), "build failed (exit {}):\n{stderr}", output.status.code().unwrap_or(-1),);
    assert!(
        report.failed.is_empty(),
        "build report had failures: {:?}",
        report
            .failed
            .iter()
            .map(|failure| format!("{}: {}", failure.root, failure.message))
            .collect::<Vec<_>>()
    );
    BuildRun { report }
}

fn first_output_path(run: &BuildRun) -> PathBuf {
    run.report
        .outcomes
        .first()
        .and_then(|outcome| outcome.outputs.first())
        .map(|output| output.path.clone())
        .expect("build should produce at least one output path")
}

fn all_output_paths(run: &BuildRun) -> Vec<PathBuf> {
    run.report
        .outcomes
        .iter()
        .flat_map(|outcome| outcome.outputs.iter().map(|output| output.path.clone()))
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────

#[test]
fn smoke_build_flat_file_and_read_content() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "flat-file",
  builder = "/bin/sh",
  args = ["-c", "echo 'crunch works' > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&run);
    assert!(out.exists(), "output should exist on disk: {}", out.display());
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content.trim(), "crunch works");
}

#[test]
fn smoke_build_single_derivation_direct_path_regression() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "single-direct-path",
  builder = "/bin/sh",
  args = ["-c", "echo single-direct-path > $out"],
  system = 'x86_64-linux,
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    assert_eq!(run.report.counts.succeeded_total, 1);
    assert_eq!(run.report.outcomes.len(), 1);
    assert_eq!(run.report.outcomes[0].label, "single-direct-path");
    let out = first_output_path(&run);
    assert_eq!(std::fs::read_to_string(&out).unwrap().trim(), "single-direct-path");
}

#[test]
fn smoke_build_directory_output_with_structure() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "dir-output",
  builder = "/bin/sh",
  args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin $out/lib && echo '#!/bin/sh' > $out/bin/run && echo 'libfoo' > $out/lib/foo.txt && $BB chmod +x $out/bin/run"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&run);
    assert!(out.join("bin/run").exists(), "bin/run should exist");
    assert!(out.join("lib/foo.txt").exists(), "lib/foo.txt should exist");

    let script = std::fs::read_to_string(out.join("bin/run")).unwrap();
    assert_eq!(script.trim(), "#!/bin/sh");

    let lib = std::fs::read_to_string(out.join("lib/foo.txt")).unwrap();
    assert_eq!(lib.trim(), "libfoo");

    // Check executable permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(out.join("bin/run")).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "bin/run should be executable, mode={mode:o}");
    }
}

#[test]
fn smoke_build_and_run_shell_script() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "runnable",
  builder = "/bin/sh",
  args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin && $BB printf '#!/bin/sh\necho hello-from-crunch\nexit 0\n' > $out/bin/greet && $BB chmod +x $out/bin/greet"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&run);
    let greet = out.join("bin/greet");
    assert!(greet.exists(), "greet script should exist");

    // Actually run the built script
    let run = std::process::Command::new(&greet).output().expect("should be able to execute the built script");
    assert!(run.status.success(), "greet should exit 0");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(run_stdout.trim(), "hello-from-crunch");
}

#[test]
fn smoke_build_failure_reports_error() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("fail.ncl");
    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
{
  name = "will-fail",
  builder = "/bin/sh",
  args = ["-c", "echo 'something went wrong' >&2; exit 42"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
    )
    .unwrap();

    let state_dir = work.path().join("state");
    let output = crunch_cmd()
        .arg("--json")
        .arg("--store")
        .arg(store.path())
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("build")
        .arg("--no-substitute")
        .arg("-I")
        .arg(work.path())
        .arg(&ncl_file)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let report: BuildJsonReport = serde_json::from_str(&stdout).expect("failure path should still emit a JSON report");
    let _audit_dir = write_command_audit(
        "smoke",
        "build-failure",
        work.path(),
        &[
            "crunch".to_string(),
            "--json".to_string(),
            "--store".to_string(),
            store.path().display().to_string(),
            "--state-dir".to_string(),
            state_dir.display().to_string(),
            "build".to_string(),
            "--no-substitute".to_string(),
            "-I".to_string(),
            work.path().display().to_string(),
            ncl_file.display().to_string(),
        ],
        &output,
        &[],
        &[
            ("CRUNCH_STATE_DIR", state_dir.display().to_string()),
            ("CRUNCH_STORE_DIR", store.path().display().to_string()),
        ],
    )
    .unwrap();

    assert!(!output.status.success(), "build should fail");
    assert_eq!(report.counts.failed_total, 1, "report should record one failed root");
    assert_eq!(report.failed[0].root, "will-fail");
    assert_eq!(report.failed[0].phase, "build");
    assert_eq!(report.failed[0].error_class, "builder");
    assert!(report.failed[0].message.contains("42") || report.failed[0].message.contains("will-fail"));
    assert!(report.failed[0].saved_log_path.is_some(), "build failure should record a saved log path");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.trim().is_empty(), "reported build failures should not print a second JSON error");
}

#[test]
fn smoke_build_cached_on_second_run() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let ncl = r#"let crunch = import "lib.ncl" in
{
  name = "cached-test",
  builder = "/bin/sh",
  args = ["-c", "echo cached > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#;

    // First build — shared state dir so pathinfo.redb persists.
    let run1 = build_ncl_with_state(ncl, store.path(), Some(state.path()));
    let path1 = first_output_path(&run1);
    assert!(path1.exists());
    assert_eq!(run1.report.counts.cached_total, 0, "first build should not be cached");
    assert_eq!(run1.report.counts.built_total, 1, "first build should report one local build");
    assert!(!run1.report.outcomes[0].cached, "first build should not be cached");

    // Second build — same store + same state dir → cache hit
    let run2 = build_ncl_with_state(ncl, store.path(), Some(state.path()));
    let path2 = first_output_path(&run2);
    assert_eq!(path1, path2, "same derivation should produce same path");
    assert_eq!(run2.report.counts.cached_total, 1, "second build should report one cache hit");
    assert!(run2.report.outcomes[0].cached, "second build should be cached");
}

#[test]
fn smoke_build_ca_derivation() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "ca-smoke",
  builder = "/bin/sh",
  args = ["-c", "echo content-addressed > $out"],
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&run);
    assert!(out.exists(), "CA output should exist: {}", out.display());
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content.trim(), "content-addressed");
}

#[test]
fn smoke_fetchurl_downloads_and_stores() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    // Spin up a local HTTP server with known content
    let body = b"fetched-by-crunch\n";
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        use std::io::Write;
        // Serve exactly one request
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            let resp = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.write_all(body);
        }
    });

    // Compute the expected sha256 of the content for the FOD hash.
    // fetchurl does a flat hash of the downloaded bytes.
    use base64::Engine;
    use sha2::Digest;
    use sha2::Sha256;
    let hash = Sha256::digest(body);
    let sri = format!("sha256-{}", base64::engine::general_purpose::STANDARD.encode(hash));

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        &format!(
            r#"let crunch = import "lib.ncl" in
crunch.fetchurl {{
  url = "http://{addr}/data.txt",
  hash = "{sri}",
}}"#
        ),
        store.path(),
    );

    server.join().unwrap();

    let out = first_output_path(&run);
    assert!(out.exists(), "fetched output should exist: {}", out.display());
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content, "fetched-by-crunch\n");
}

#[test]
fn smoke_build_multi_derivation_file() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        r#"let crunch = import "lib.ncl" in
[
  {
    name = "alpha",
    builder = "/bin/sh",
    args = ["-c", "echo alpha-output > $out"],
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,
  {
    name = "beta",
    builder = "/bin/sh",
    args = ["-c", "echo beta-output > $out"],
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,
]"#,
        store.path(),
    );

    let paths = all_output_paths(&run);
    assert_eq!(paths.len(), 2, "should have 2 outputs: {:?}", paths);

    // Both outputs should exist and have distinct content
    let mut contents: Vec<String> = paths
        .iter()
        .map(|p| {
            assert!(p.exists(), "output should exist: {}", p.display());
            std::fs::read_to_string(p).unwrap().trim().to_string()
        })
        .collect();
    contents.sort();
    assert_eq!(contents, vec!["alpha-output", "beta-output"]);
}

#[test]
fn smoke_build_package_set_record_direct_path_regression() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  alpha = {
    name = "alpha-record",
    builder = "/bin/sh",
    args = ["-c", "echo alpha-record > $out"],
    system = 'x86_64-linux,
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,
  beta = {
    name = "beta-record",
    builder = "/bin/sh",
    args = ["-c", "echo beta-record > $out"],
    system = 'x86_64-linux,
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,
}"#,
        store.path(),
    );

    assert_eq!(run.report.counts.succeeded_total, 2);

    let mut content_by_label = std::collections::HashMap::new();
    for outcome in &run.report.outcomes {
        assert_eq!(outcome.outputs.len(), 1, "each root should expose one output");
        let output = &outcome.outputs[0];
        let content = std::fs::read_to_string(&output.path).unwrap();
        let previous = content_by_label.insert(outcome.label.clone(), content.trim().to_string());
        assert!(previous.is_none(), "duplicate outcome label: {}", outcome.label);
    }

    assert_eq!(content_by_label.len(), 2);
    assert_eq!(content_by_label.get("alpha").map(String::as_str), Some("alpha-record"));
    assert_eq!(content_by_label.get("beta").map(String::as_str), Some("beta-record"));
}

#[test]
fn smoke_build_multi_derivation_partial_failure_keeps_successful_root() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let ncl_file = work.path().join("partial-failure.ncl");
    std::fs::write(
        &ncl_file,
        r#"let crunch = import "lib.ncl" in
[
  {
    name = "alpha",
    builder = "/bin/sh",
    args = ["-c", "echo alpha-output > $out"],
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,
  {
    name = "beta",
    builder = "/bin/sh",
    args = ["-c", "echo beta-fail >&2; exit 17"],
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,
]"#,
    )
    .unwrap();

    let state_dir = work.path().join("state");
    let output = crunch_cmd()
        .arg("--json")
        .arg("--store")
        .arg(store.path())
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("build")
        .arg("--no-substitute")
        .arg("-I")
        .arg(work.path())
        .arg(&ncl_file)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let report: BuildJsonReport =
        serde_json::from_str(&stdout).expect("partial failure should still emit a JSON report");

    assert!(!output.status.success(), "build should fail with one bad root");
    assert_eq!(report.counts.succeeded_total, 1);
    assert_eq!(report.counts.failed_total, 1);
    assert!(report.outcomes.iter().any(|outcome| outcome.label == "alpha"));
    assert!(report.failed.iter().any(|failure| failure.root == "beta"));

    let alpha_output = report
        .outcomes
        .iter()
        .find(|outcome| outcome.label == "alpha")
        .and_then(|outcome| outcome.outputs.first())
        .map(|output| output.path.clone())
        .expect("successful root should keep its output path");
    assert!(alpha_output.exists(), "successful root output should exist: {}", alpha_output.display());
}

#[test]
fn smoke_build_symlink_in_output() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let store = tempfile::tempdir().unwrap();
    let run = build_ncl(
        r#"let crunch = import "lib.ncl" in
{
  name = "with-symlink",
  builder = "/bin/sh",
  args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin && echo '#!/bin/sh' > $out/bin/real && $BB chmod +x $out/bin/real && $BB ln -s real $out/bin/alias"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        store.path(),
    );

    let out = first_output_path(&run);
    let alias = out.join("bin/alias");
    assert!(alias.exists(), "symlink should exist");

    #[cfg(unix)]
    {
        let meta = std::fs::symlink_metadata(&alias).unwrap();
        assert!(meta.is_symlink(), "alias should be a symlink");
        let target = std::fs::read_link(&alias).unwrap();
        assert_eq!(target.to_str().unwrap(), "real");
    }
}

#[test]
fn smoke_deterministic_output_path() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let ncl = r#"let crunch = import "lib.ncl" in
{
  name = "deterministic",
  builder = "/bin/sh",
  args = ["-c", "echo stable > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#;

    // Build in two independent store dirs
    let store1 = tempfile::tempdir().unwrap();
    let store2 = tempfile::tempdir().unwrap();

    let run1 = build_ncl(ncl, store1.path());
    let run2 = build_ncl(ncl, store2.path());

    let path1 = first_output_path(&run1);
    let path2 = first_output_path(&run2);

    // The filename (hash-name) should be identical even though store dirs differ
    let name1 = path1.file_name().unwrap();
    let name2 = path2.file_name().unwrap();
    assert_eq!(name1, name2, "same derivation should produce same hash-name");
}

#[test]
fn smoke_build_then_push_narinfo_and_nar_match() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    // Step 1: build a hello derivation with a shared state dir.
    let work = tempfile::tempdir().unwrap();
    let store = work.path().join("store");
    let state = work.path().join("state");
    let cache = work.path().join("cache");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(&state).unwrap();

    let run = build_ncl_with_state(
        r#"let crunch = import "lib.ncl" in
{
  name = "push-hello",
  builder = "/bin/sh",
  args = ["-c", "echo 'push test content' > $out"],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        &store,
        Some(&state),
    );
    assert_eq!(run.report.counts.succeeded_total, 1);
    let out = first_output_path(&run);
    assert!(out.exists(), "build output should exist: {}", out.display());

    // Step 2: push all paths to a binary cache directory.
    let mut push_cmd = crunch_cmd();
    push_cmd
        .arg("--store").arg(&store)
        .arg("--state-dir").arg(&state)
        .arg("store").arg("push")
        .arg("--to").arg(&cache)
        .arg("--all");
    let push_output = push_cmd.output().expect("push should execute");
    let push_stderr = String::from_utf8_lossy(&push_output.stderr);
    assert!(
        push_output.status.success(),
        "push failed (exit {}):\n{push_stderr}",
        push_output.status.code().unwrap_or(-1),
    );
    let push_stdout = String::from_utf8_lossy(&push_output.stdout);
    assert!(push_stdout.contains("PUSH "), "push should report at least one pushed path, got: {push_stdout}");

    // Step 3: verify nix-cache-info.
    let cache_info = std::fs::read_to_string(cache.join("nix-cache-info")).unwrap();
    assert!(cache_info.contains("StoreDir:"), "nix-cache-info must contain StoreDir");
    assert!(cache_info.contains("WantMassQuery: 1"), "nix-cache-info must set WantMassQuery");

    // Step 4: find and parse the narinfo file.
    let narinfo_entries: Vec<_> = std::fs::read_dir(&cache)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map_or(false, |ext| ext == "narinfo")
        })
        .collect();
    assert!(
        !narinfo_entries.is_empty(),
        "cache dir should contain at least one .narinfo file",
    );

    // Find the narinfo for the output we built (match by store path name).
    let output_name = out.file_name().unwrap().to_str().unwrap();
    let mut found_match = false;

    for entry in &narinfo_entries {
        let narinfo_text = std::fs::read_to_string(entry.path()).unwrap();
        let narinfo = nix_compat::narinfo::NarInfo::parse(&narinfo_text)
            .unwrap_or_else(|e| panic!("narinfo should parse: {e}\ncontent:\n{narinfo_text}"));

        let narinfo_store_name = narinfo.store_path.to_string();
        if !narinfo_store_name.contains(output_name) {
            continue;
        }
        found_match = true;

        // Step 5: read the NAR file referenced by the narinfo.
        let nar_path = cache.join(narinfo.url);
        assert!(nar_path.exists(), "NAR file should exist: {}", nar_path.display());
        let nar_bytes = std::fs::read(&nar_path).unwrap();

        // Verify NAR sha256 matches the narinfo NarHash.
        let actual_sha256: [u8; 32] = {
            use sha2::Digest;
            sha2::Sha256::digest(&nar_bytes).into()
        };
        assert_eq!(
            actual_sha256, narinfo.nar_hash,
            "NAR file sha256 must match narinfo NarHash",
        );

        // Verify NarSize matches actual file size.
        assert_eq!(
            nar_bytes.len() as u64, narinfo.nar_size,
            "NAR file size must match narinfo NarSize",
        );

        // Verify FileHash matches if present.
        if let Some(file_hash) = narinfo.file_hash {
            assert_eq!(
                actual_sha256, file_hash,
                "FileHash must match actual NAR sha256 (uncompressed)",
            );
        }

        // Verify FileSize matches if present.
        if let Some(file_size) = narinfo.file_size {
            assert_eq!(
                nar_bytes.len() as u64, file_size,
                "FileSize must match actual NAR file size",
            );
        }

        // Verify at least one signature is present.
        assert!(
            !narinfo.signatures.is_empty(),
            "pushed narinfo should have at least one signature",
        );

        break;
    }

    assert!(found_match, "should find a narinfo matching output '{output_name}'");
}
