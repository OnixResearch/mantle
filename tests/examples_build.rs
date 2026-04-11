use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use assert_cmd::Command;

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should be built")
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

fn can_build() -> bool {
    Path::new("/nix/store").exists() && find_bwrap().is_some()
}

struct BuildRun {
    store: tempfile::TempDir,
    #[allow(dead_code)]
    state: tempfile::TempDir,
    output: std::process::Output,
}

fn build_example(example: &str) -> BuildRun {
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut cmd = crunch_cmd();
    cmd.current_dir(repo_root());
    cmd.args([
        "build",
        example,
        "--store",
        store.path().to_str().unwrap(),
        "--state-dir",
        state.path().to_str().unwrap(),
        "--no-substitute",
    ]);
    if let Some(bwrap) = find_bwrap() {
        let bwrap_dir = bwrap.parent().unwrap();
        let current_path = std::env::var_os("PATH").unwrap_or_default();
        let joined =
            std::env::join_paths(std::iter::once(bwrap_dir.to_path_buf()).chain(std::env::split_paths(&current_path)))
                .unwrap();
        cmd.env("PATH", joined);
    }
    let output = cmd.output().expect("example build should run");
    BuildRun { store, state, output }
}

#[test]
fn fetch_crate_crc64_example_builds() {
    if !can_build() {
        eprintln!("SKIP: example build requires Linux + bwrap + /nix/store");
        return;
    }

    let run = build_example("examples/fetch-crate-crc64.ncl");
    let stdout = String::from_utf8(run.output.stdout.clone()).unwrap();
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(run.output.status.success(), "fetch example failed:\n{stderr}");

    let out_path = PathBuf::from(stdout.trim());
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
    let stdout = String::from_utf8(run.output.stdout.clone()).unwrap();
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(run.output.status.success(), "crate example failed:\n{stderr}");

    let out_path = PathBuf::from(stdout.trim());
    assert!(out_path.starts_with(run.store.path()), "output should land in temp store: {}", out_path.display());
    let bin = out_path.join("bin/crc64");
    assert!(bin.is_file(), "built binary missing: {}", bin.display());

    let input_path = out_path.join("sample.txt");
    std::fs::write(&input_path, b"crunch example payload\n").unwrap();
    let run = StdCommand::new(&bin).arg(&input_path).output().unwrap();
    let run_stdout = String::from_utf8(run.stdout).unwrap();
    assert!(run.status.success(), "crc64 binary failed: {}", String::from_utf8_lossy(&run.stderr));
    assert!(run_stdout.contains("sample.txt"), "checksum output should mention input file: {run_stdout}");
}
