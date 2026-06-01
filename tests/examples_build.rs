use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use assert_cmd::Command;

const HELLO_OUTPUT: &str = "Hello, mantle!";
const MULTI_STEP_MARKER: &str = "name: multi-step";
const FAIL_MARKER: &str = "this will fail";
const PROJECT_CHECK_RESULT: &str = "ok";
const LOCAL_LAYOUT_HEADER: &str = "#define LOCAL_OUTPUT_LAYOUT 1";
const LOCAL_LAYOUT_DOC: &str = "local output layout docs";
const BUILD_REPORT_SCHEMA: &str = "crunch-build-report-v1";
const SHA256_DIGEST_BYTES: usize = 32;
const WRONG_SHA256_SRI: &str = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
const OFFLINE_FETCH_FILE_CONTENT: &[u8] = b"offline fetchurl fixture\n";
const OFFLINE_TARBALL_README: &str = "# Offline tarball fixture\n";
const OFFLINE_TARBALL_LIB: &str = "pub const OFFLINE_TARBALL: &str = \"fixture\";\n";
const OFFLINE_TARBALL_ROOT: &str = "offline-tarball-fixture-1.0";
const OFFLINE_GIT_CONTENT: &str = "offline fetchgit fixture\n";
const PROJECT_FIXTURE: &str = r#"
let mantle = import "lib.ncl" in
{
  packages = {
    hello = {
      name = "hello-project-fixture",
      builder = "/bin/sh",
      args = ["-c", m%"
        BB=/bin/busybox
        $BB mkdir -p $out/bin
        $BB cat > $out/bin/hello << 'EOF'
#!/bin/sh
echo "Hello from project fixture"
EOF
        $BB chmod +x $out/bin/hello
      "%],
    } | mantle.Derivation,
  },
  checks = {
    test-hello = {
      name = "test-hello-project-fixture",
      builder = "/bin/sh",
      args = ["-c", m%"
        BB=/bin/busybox
        $BB mkdir -p $out
        $BB echo "ok" > $out/result
      "%],
    } | mantle.Derivation,
  },
  default = { package = "hello" },
} | mantle.Project
"#;
const MULTI_OUTPUT_FIXTURE: &str = r#"
let mantle = import "lib.ncl" in
{
  name = "local-multi-output-fixture",
  builder = "/bin/sh",
  outputs = ["out", "dev", "man"],
  args = ["-c", m%"
    BB=/bin/busybox
    $BB mkdir -p $out/bin $dev/include $man/share/man/man1
    $BB cat > $out/bin/hello << 'EOF'
#!/bin/sh
echo "Hello from multi output fixture"
EOF
    $BB chmod +x $out/bin/hello
    $BB echo '#define HELLO_VERSION "1.0"' > $dev/include/hello.h
    $BB echo '.TH HELLO 1' > $man/share/man/man1/hello.1
  "%],
} | mantle.Derivation
"#;

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn find_bwrap() -> Option<PathBuf> {
    let path_dirs = std::env::var_os("PATH");
    if let Some(path_dirs) = path_dirs {
        for dir in std::env::split_paths(&path_dirs) {
            let candidate = dir.join("bwrap");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    for candidate in ["/run/wrappers/bin/bwrap", "/run/current-system/sw/bin/bwrap"] {
        let path = PathBuf::from(candidate);
        if path.is_file() {
            return Some(path);
        }
    }

    let store_dir = Path::new("/nix/store");
    let entries = std::fs::read_dir(store_dir).ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join("bin/bwrap");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn find_static_busybox() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("SNIX_BUILD_SANDBOX_SHELL") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let store_dir = Path::new("/nix/store");
    let entries = std::fs::read_dir(store_dir).ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join("bin/busybox");
        let candidate_text = candidate.to_string_lossy();
        if candidate.is_file() && candidate_text.contains("busybox-static") {
            return Some(candidate);
        }
    }
    None
}

fn can_build() -> bool {
    Path::new("/nix/store").exists() && find_bwrap().is_some() && find_static_busybox().is_some()
}

struct BuildRun {
    store: tempfile::TempDir,
    #[allow(dead_code)]
    state: tempfile::TempDir,
    output: std::process::Output,
}

fn build_example(example: &str) -> BuildRun {
    build_path(&repo_root().join(example))
}

fn build_path(path: &Path) -> BuildRun {
    build_path_with_json_mode(path, false)
}

fn build_path_json(path: &Path) -> BuildRun {
    build_path_with_json_mode(path, true)
}

fn build_path_with_json_mode(path: &Path, json_mode: bool) -> BuildRun {
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut cmd = mantle_cmd();
    cmd.current_dir(repo_root());
    if json_mode {
        cmd.arg("--json");
    }
    cmd.args([
        "build",
        path.to_str().unwrap(),
        "--store",
        store.path().to_str().unwrap(),
        "--state-dir",
        state.path().to_str().unwrap(),
        "--no-substitute",
    ]);
    inject_build_environment(&mut cmd);
    let output = cmd.output().expect("example build should run");
    BuildRun { store, state, output }
}

fn inject_build_environment(cmd: &mut Command) {
    if let Some(bwrap) = find_bwrap() {
        let bwrap_dir = bwrap.parent().unwrap();
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let joined =
            std::env::join_paths(std::iter::once(bwrap_dir.to_path_buf()).chain(std::env::split_paths(&current_path)))
                .unwrap();
        cmd.env("PATH", joined);
    }
    if let Some(busybox) = find_static_busybox() {
        cmd.env("SNIX_BUILD_SANDBOX_SHELL", busybox);
    }
}

fn build_stdout_path(run: &BuildRun) -> PathBuf {
    let stdout = String::from_utf8(run.output.stdout.clone()).unwrap();
    PathBuf::from(strip_output_label(stdout.trim()))
}

fn strip_output_label(line: &str) -> &str {
    line.split_once(" (").map(|(path, _label)| path).unwrap_or(line)
}

fn build_stdout_paths(run: &BuildRun) -> Vec<PathBuf> {
    let stdout = String::from_utf8(run.output.stdout.clone()).unwrap();
    stdout.lines().map(strip_output_label).map(PathBuf::from).collect()
}

fn assert_success(run: &BuildRun, label: &str) {
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(run.output.status.success(), "{label} failed:\n{stderr}");
}

fn write_fixture(dir: &tempfile::TempDir, name: &str, source: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, source).unwrap();
    path
}

fn build_path_with_fix(path: &Path) -> BuildRun {
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut cmd = mantle_cmd();
    cmd.current_dir(repo_root());
    cmd.arg("build");
    cmd.arg("--fix");
    cmd.arg(path);
    cmd.arg("--store");
    cmd.arg(store.path());
    cmd.arg("--state-dir");
    cmd.arg(state.path());
    cmd.arg("--no-substitute");
    inject_build_environment(&mut cmd);
    let output = cmd.output().expect("example build should run");
    BuildRun { store, state, output }
}

fn sha256_sri(bytes: &[u8]) -> String {
    use sha2::Digest;

    let digest: [u8; SHA256_DIGEST_BYTES] = sha2::Sha256::digest(bytes).into();
    format!("sha256-{}", data_encoding::BASE64.encode(&digest))
}

fn compute_recursive_sha256_sri(path: &Path) -> String {
    use sha2::Digest;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_castore::import::fs::ingest_path;
    use snix_store::nar::write_nar;
    use snix_store::utils::AsyncIoBridge;

    let root = path.to_path_buf();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let blob_service = MemoryBlobService::default();
        let directory_service = RedbDirectoryService::new_temporary(
            "examples-fetcher-hash".to_string(),
            RedbDirectoryServiceConfig::default(),
        )
        .unwrap();
        let node = ingest_path::<_, _, _, &[u8]>(blob_service.clone(), directory_service.clone(), &root, None)
            .await
            .unwrap();
        let mut hasher = sha2::Sha256::new();
        write_nar(AsyncIoBridge(&mut hasher), &node, blob_service, directory_service).await.unwrap();
        let digest: [u8; SHA256_DIGEST_BYTES] = hasher.finalize().into();
        format!("sha256-{}", data_encoding::BASE64.encode(&digest))
    })
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

fn fetchurl_fixture(url: &str, hash: &str) -> String {
    format!(
        r#"let mantle = import "lib.ncl" in
mantle.fetchurl {{
  url = "{url}",
  hash = "{hash}",
  name = "offline-fetch-file-fixture",
}}
"#
    )
}

fn fetch_tarball_fixture(url: &str, hash: &str) -> String {
    format!(
        r#"let mantle = import "lib.ncl" in
mantle.fetchTarball {{
  url = "{url}",
  hash = "{hash}",
  name = "offline-fetch-tarball-fixture",
}}
"#
    )
}

fn fetch_git_fixture(url: &str, rev: &str, hash: &str) -> String {
    format!(
        r#"let mantle = import "lib.ncl" in
mantle.fetchGit {{
  url = "{url}",
  rev = "{rev}",
  hash = "{hash}",
  name = "offline-fetch-git-fixture",
}}
"#
    )
}

fn create_offline_file_fixture(dir: &tempfile::TempDir) -> PathBuf {
    let source = dir.path().join("offline-fetch-file.txt");
    std::fs::write(&source, OFFLINE_FETCH_FILE_CONTENT).unwrap();
    source
}

fn create_offline_tarball_fixture(dir: &tempfile::TempDir) -> (PathBuf, PathBuf) {
    use std::io::Write;

    let source_root = dir.path().join(OFFLINE_TARBALL_ROOT);
    std::fs::create_dir(&source_root).unwrap();
    std::fs::write(source_root.join("README.md"), OFFLINE_TARBALL_README).unwrap();
    std::fs::create_dir(source_root.join("src")).unwrap();
    std::fs::write(source_root.join("src/lib.rs"), OFFLINE_TARBALL_LIB).unwrap();

    let mut tar_builder = tar::Builder::new(Vec::new());
    tar_builder.append_dir_all(OFFLINE_TARBALL_ROOT, &source_root).unwrap();
    let tar_data = tar_builder.into_inner().unwrap();

    let tarball = dir.path().join("offline-fetch-tarball.tar.gz");
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&tar_data).unwrap();
    let gz_data = encoder.finish().unwrap();
    std::fs::write(&tarball, gz_data).unwrap();

    (tarball, source_root)
}

fn run_git(repo: &Path, args: &[&str]) -> String {
    let output = StdCommand::new("git").current_dir(repo).args(args).output().expect("git should run");
    assert!(output.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn create_offline_git_fixture(dir: &tempfile::TempDir) -> (PathBuf, String, tempfile::TempDir) {
    let repo = dir.path().join("offline-fetch-git-repo");
    std::fs::create_dir(&repo).unwrap();
    run_git(&repo, &["init", "--quiet"]);
    run_git(&repo, &["config", "user.email", "mantle@example.invalid"]);
    run_git(&repo, &["config", "user.name", "Mantle Example"]);
    std::fs::write(repo.join("hello.txt"), OFFLINE_GIT_CONTENT).unwrap();
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "--quiet", "-m", "offline fixture"]);
    let rev = run_git(&repo, &["rev-parse", "HEAD"]);

    let expected_tree = tempfile::tempdir().unwrap();
    copy_tree_without_dot_git(&repo, expected_tree.path());
    (repo, rev, expected_tree)
}

fn copy_tree_without_dot_git(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        if file_name == ".git" {
            continue;
        }
        let source_path = entry.path();
        let dest_path = dst.join(&file_name);
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            copy_tree_without_dot_git(&source_path, &dest_path);
            continue;
        }
        std::fs::copy(&source_path, &dest_path).unwrap();
    }
}

fn assert_fod_failure(run: &BuildRun, label: &str) {
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    let stdout = String::from_utf8_lossy(&run.output.stdout);
    assert!(!run.output.status.success(), "{label} should fail on wrong hash");
    assert!(
        stderr.contains("hash mismatch") || stderr.contains("fixed-output"),
        "unexpected {label} stderr:\n{stderr}"
    );
    assert!(stdout.trim().is_empty(), "{label} should not report successful outputs: {stdout}");
    assert_store_has_no_entries(&run.store, label);
}

fn assert_store_has_no_entries(run_store: &tempfile::TempDir, label: &str) {
    let entries: Vec<PathBuf> =
        std::fs::read_dir(run_store.path()).unwrap().map(|entry| entry.unwrap().path()).collect();
    assert!(entries.is_empty(), "{label} should not persist failed output entries: {entries:?}");
}

fn project_command(project_dir: &Path, selector: &str) -> (tempfile::TempDir, tempfile::TempDir, std::process::Output) {
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut cmd = mantle_cmd();
    cmd.current_dir(project_dir);
    cmd.args([
        "build",
        selector,
        "--store",
        store.path().to_str().unwrap(),
        "--state-dir",
        state.path().to_str().unwrap(),
        "--no-substitute",
    ]);
    inject_build_environment(&mut cmd);
    let output = cmd.output().expect("project build should run");
    (store, state, output)
}

fn make_project_fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("crunch.ncl"), PROJECT_FIXTURE).unwrap();
    dir
}

#[test]
fn hello_example_builds_flat_output() {
    if !can_build() {
        eprintln!("SKIP: example build requires Linux + bwrap + /nix/store");
        return;
    }

    let run = build_example("examples/hello.ncl");
    assert_success(&run, "hello example");

    let out_path = build_stdout_path(&run);
    assert!(out_path.starts_with(run.store.path()), "output should land in temp store: {}", out_path.display());
    let output = std::fs::read_to_string(&out_path).unwrap();
    assert!(output.contains(HELLO_OUTPUT), "unexpected hello output: {output}");
}

#[test]
fn multi_step_example_builds_structured_output() {
    if !can_build() {
        eprintln!("SKIP: example build requires Linux + bwrap + /nix/store");
        return;
    }

    let run = build_example("examples/multi-step.ncl");
    assert_success(&run, "multi-step example");

    let out_path = build_stdout_path(&run);
    assert!(out_path.starts_with(run.store.path()), "output should land in temp store: {}", out_path.display());
    let output = std::fs::read_to_string(&out_path).unwrap();
    assert!(output.contains(MULTI_STEP_MARKER), "unexpected multi-step output: {output}");
}

#[test]
fn local_output_layout_example_builds_named_outputs() {
    if !can_build() {
        eprintln!("SKIP: example build requires Linux + bwrap + /nix/store");
        return;
    }

    let run = build_example("examples/local-output-layout.ncl");
    assert_success(&run, "local output layout example");

    let outputs = build_stdout_paths(&run);
    assert_eq!(outputs.len(), 3, "local output layout should print three named outputs: {outputs:?}");
    assert!(
        outputs.iter().all(|path| path.starts_with(run.store.path())),
        "outputs should use temp store: {outputs:?}"
    );
    assert!(outputs.iter().any(|path| path.join("bin/show-layout").is_file()), "missing out output: {outputs:?}");
    assert!(
        outputs.iter().any(|path| std::fs::read_to_string(path.join("include/local_output_layout.h"))
            .is_ok_and(|text| text.contains(LOCAL_LAYOUT_HEADER))),
        "missing dev header output: {outputs:?}"
    );
    assert!(
        outputs.iter().any(|path| {
            std::fs::read_to_string(path.join("share/doc/local-output-layout/README"))
                .is_ok_and(|text| text.trim() == LOCAL_LAYOUT_DOC)
        }),
        "missing doc output: {outputs:?}"
    );
}

#[test]
fn hello_json_build_report_exposes_artifact_attestation_shape() {
    if !can_build() {
        eprintln!("SKIP: JSON build report requires Linux + bwrap + /nix/store");
        return;
    }

    let run = build_path_json(&repo_root().join("examples/hello.ncl"));
    assert_success(&run, "hello JSON build report");

    let stdout = String::from_utf8(run.output.stdout.clone()).unwrap();
    let report: serde_json::Value = serde_json::from_str(&stdout).expect("hello build report should be JSON");
    assert_eq!(report["schema"], BUILD_REPORT_SCHEMA);
    assert_eq!(report["counts"]["succeeded_total"], 1);
    assert_eq!(report["counts"]["failed_total"], 0);
    assert_eq!(report["outcomes"][0]["label"], "hello");
    let attestation = &report["outcomes"][0]["outputs"][0]["artifact_attestation"];
    let logical_path = attestation["logical_path"].as_str().unwrap();
    let sidecar_path = attestation["path"].as_str().unwrap();
    assert!(logical_path.contains("hello"), "logical path should name hello: {logical_path}");
    assert!(
        sidecar_path.contains("attestations/artifacts"),
        "sidecar path should be artifact attestation: {sidecar_path}"
    );
    assert!(Path::new(sidecar_path).is_file(), "artifact attestation sidecar should exist: {sidecar_path}");
}

#[test]
fn fail_example_reports_expected_failure() {
    if !can_build() {
        eprintln!("SKIP: example build requires Linux + bwrap + /nix/store");
        return;
    }

    let run = build_example("examples/fail.ncl");
    let stderr = String::from_utf8_lossy(&run.output.stderr);

    assert!(!run.output.status.success(), "fail example should not build successfully");
    assert!(stderr.contains(FAIL_MARKER), "failure stderr should include marker:\n{stderr}");
}

#[test]
fn local_multi_output_fixture_builds_named_layout() {
    if !can_build() {
        eprintln!("SKIP: example build requires Linux + bwrap + /nix/store");
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let fixture = write_fixture(&dir, "multi-output-fixture.ncl", MULTI_OUTPUT_FIXTURE);
    let run = build_path(&fixture);
    assert_success(&run, "local multi-output fixture");

    let outputs = build_stdout_paths(&run);
    assert!(!outputs.is_empty(), "multi-output build should print output paths");
    assert!(
        outputs.iter().any(|path| path.join("bin/hello").is_file()),
        "missing runnable out output: {outputs:?}"
    );
    assert!(outputs.iter().any(|path| path.join("include/hello.h").is_file()), "missing dev output: {outputs:?}");
    assert!(
        outputs.iter().any(|path| path.join("share/man/man1/hello.1").is_file()),
        "missing man output: {outputs:?}"
    );
}

#[test]
fn project_check_fixture_builds_result_output() {
    if !can_build() {
        eprintln!("SKIP: project example build requires Linux + bwrap + /nix/store");
        return;
    }

    let project = make_project_fixture();
    let (store, _state, output) = project_command(project.path(), ".#checks.test-hello");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "project check failed:\n{stderr}");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let out_path = PathBuf::from(stdout.trim());
    assert!(out_path.starts_with(store.path()), "output should land in temp store: {}", out_path.display());
    let result = std::fs::read_to_string(out_path.join("result")).unwrap();
    assert_eq!(result.trim(), PROJECT_CHECK_RESULT);
}

#[test]
fn project_missing_selector_fails_before_build_success() {
    let project = make_project_fixture();
    let (_store, _state, output) = project_command(project.path(), ".#missing");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "missing selector should fail");
    assert!(stderr.contains("missing") || stderr.contains("not found"), "unexpected selector error:\n{stderr}");
}

#[test]
fn offline_fetchurl_fixture_builds_without_network() {
    let dir = tempfile::tempdir().unwrap();
    let source = create_offline_file_fixture(&dir);
    let hash = sha256_sri(OFFLINE_FETCH_FILE_CONTENT);
    let fixture = write_fixture(&dir, "offline-fetchurl.ncl", &fetchurl_fixture(&file_url(&source), &hash));

    let run = build_path(&fixture);
    assert_success(&run, "offline fetchurl fixture");

    let out_path = build_stdout_path(&run);
    assert!(out_path.starts_with(run.store.path()), "output should land in temp store: {}", out_path.display());
    assert_eq!(std::fs::read(&out_path).unwrap(), OFFLINE_FETCH_FILE_CONTENT);
}

#[test]
fn offline_fetch_tarball_fixture_builds_without_network() {
    let dir = tempfile::tempdir().unwrap();
    let (tarball, expected_tree) = create_offline_tarball_fixture(&dir);
    let hash = compute_recursive_sha256_sri(&expected_tree);
    let fixture = write_fixture(&dir, "offline-fetch-tarball.ncl", &fetch_tarball_fixture(&file_url(&tarball), &hash));

    let run = build_path(&fixture);
    assert_success(&run, "offline fetchTarball fixture");

    let out_path = build_stdout_path(&run);
    assert!(out_path.starts_with(run.store.path()), "output should land in temp store: {}", out_path.display());
    assert_eq!(std::fs::read_to_string(out_path.join("README.md")).unwrap(), OFFLINE_TARBALL_README);
    assert_eq!(std::fs::read_to_string(out_path.join("src/lib.rs")).unwrap(), OFFLINE_TARBALL_LIB);
}

#[test]
fn offline_fetchgit_fixture_builds_without_network() {
    let dir = tempfile::tempdir().unwrap();
    let (repo, rev, expected_tree) = create_offline_git_fixture(&dir);
    let hash = compute_recursive_sha256_sri(expected_tree.path());
    let fixture = write_fixture(&dir, "offline-fetchgit.ncl", &fetch_git_fixture(&file_url(&repo), &rev, &hash));

    let run = build_path(&fixture);
    assert_success(&run, "offline fetchGit fixture");

    let out_path = build_stdout_path(&run);
    assert!(out_path.starts_with(run.store.path()), "output should land in temp store: {}", out_path.display());
    assert_eq!(std::fs::read_to_string(out_path.join("hello.txt")).unwrap(), OFFLINE_GIT_CONTENT);
    assert!(!out_path.join(".git").exists(), "fetchGit output must not contain .git metadata");
}

#[test]
fn offline_fetcher_wrong_hashes_fail_closed() {
    let dir = tempfile::tempdir().unwrap();

    let file_source = create_offline_file_fixture(&dir);
    let file_fixture = write_fixture(
        &dir,
        "offline-fetchurl-wrong-hash.ncl",
        &fetchurl_fixture(&file_url(&file_source), WRONG_SHA256_SRI),
    );
    assert_fod_failure(&build_path(&file_fixture), "offline fetchurl wrong hash");

    let (tarball, _expected_tree) = create_offline_tarball_fixture(&dir);
    let tarball_fixture = write_fixture(
        &dir,
        "offline-fetch-tarball-wrong-hash.ncl",
        &fetch_tarball_fixture(&file_url(&tarball), WRONG_SHA256_SRI),
    );
    assert_fod_failure(&build_path(&tarball_fixture), "offline fetchTarball wrong hash");

    let (repo, rev, _expected_git_tree) = create_offline_git_fixture(&dir);
    let git_fixture = write_fixture(
        &dir,
        "offline-fetchgit-wrong-hash.ncl",
        &fetch_git_fixture(&file_url(&repo), &rev, WRONG_SHA256_SRI),
    );
    assert_fod_failure(&build_path(&git_fixture), "offline fetchGit wrong hash");
}

#[test]
fn fix_flag_updates_temp_fetchurl_fixture_hash() {
    let dir = tempfile::tempdir().unwrap();
    let source = create_offline_file_fixture(&dir);
    let fixture =
        write_fixture(&dir, "offline-fetchurl-fix.ncl", &fetchurl_fixture(&file_url(&source), WRONG_SHA256_SRI));
    assert!(!fixture.starts_with(repo_root()), "repair fixture must be outside checked-in examples");

    let run = build_path_with_fix(&fixture);
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(!run.output.status.success(), "--fix should ask for a rebuild after editing hash");
    assert!(stderr.contains("fixed:"), "--fix should report repaired temp fixture:\n{stderr}");

    let corrected = std::fs::read_to_string(&fixture).unwrap();
    assert!(!corrected.contains(WRONG_SHA256_SRI), "wrong hash should be removed from temp fixture");
    assert!(
        corrected.contains(&sha256_sri(OFFLINE_FETCH_FILE_CONTENT)),
        "correct hash missing from temp fixture: {corrected}"
    );

    let rerun = build_path(&fixture);
    assert_success(&rerun, "repaired offline fetchurl fixture");
    let out_path = build_stdout_path(&rerun);
    assert_eq!(std::fs::read(&out_path).unwrap(), OFFLINE_FETCH_FILE_CONTENT);
}

#[test]
fn fetch_crate_crc64_example_builds() {
    if !can_build() {
        eprintln!("SKIP: example build requires Linux + bwrap + /nix/store");
        return;
    }

    let run = build_example("examples/fetch-crate-crc64.ncl");
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(run.output.status.success(), "fetch example failed:\n{stderr}");

    let out_path = build_stdout_path(&run);
    assert!(out_path.starts_with(run.store.path()), "output should land in temp store: {}", out_path.display());
    assert!(out_path.exists(), "output path missing: {}", out_path.display());
    assert!(out_path.join("Cargo.toml").is_file(), "crate source should contain Cargo.toml");
}

#[test]
#[ignore = "heavy bootstrap build; run explicitly when validating the real crate example"]
fn build_crate_crc64_example_builds_binary() {
    if !can_build() {
        eprintln!("SKIP: example build requires Linux + bwrap + /nix/store");
        return;
    }

    let run = build_example("examples/build-crate-crc64.ncl");
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(run.output.status.success(), "crate example failed:\n{stderr}");

    let out_path = build_stdout_path(&run);
    assert!(out_path.starts_with(run.store.path()), "output should land in temp store: {}", out_path.display());
    let bin = out_path.join("bin/crc64");
    assert!(bin.is_file(), "built binary missing: {}", bin.display());

    let input_path = out_path.join("sample.txt");
    std::fs::write(&input_path, b"mantle example payload\n").unwrap();
    let run = StdCommand::new(&bin).arg(&input_path).output().unwrap();
    let run_stdout = String::from_utf8(run.stdout).unwrap();
    assert!(run.status.success(), "crc64 binary failed: {}", String::from_utf8_lossy(&run.stderr));
    assert!(run_stdout.contains("sample.txt"), "checksum output should mention input file: {run_stdout}");
}
