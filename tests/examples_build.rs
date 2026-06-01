use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use assert_cmd::Command;

const HELLO_OUTPUT: &str = "Hello, mantle!";
const MULTI_STEP_MARKER: &str = "name: multi-step";
const FAIL_MARKER: &str = "this will fail";
const PROJECT_CHECK_RESULT: &str = "ok";
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
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut cmd = mantle_cmd();
    cmd.current_dir(repo_root());
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
