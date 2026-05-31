use std::fs;
use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

const MAX_PUBLIC_SCAN_FILES: usize = 512;
const RAW_INVENTORY_REJECTION_INPUT: &str = r#"
{
  machines = {
    web = {
      roles = [ "web" ],
      settings = {},
    },
  },
  modules = [],
}
"#;
const REMOVED_SYSTEM_SURFACE_PATHS: &[&str] = &[
    "src/system_cmd.rs",
    "crates/crunch-system",
    "docs/system-config.md",
    "examples/system-config",
    "lib/inventory.ncl",
    "lib/system_module.ncl",
    "tests/system_cli.rs",
];
const PUBLIC_REFERENCE_ROOTS: &[&str] = &[
    "README.md",
    "docs",
    "examples",
    "lib/lib.ncl",
    "Cargo.toml",
    "flake.nix",
    "crates/crunch-eval/src/stdlib.rs",
];
const FORBIDDEN_PUBLIC_PATTERNS: &[&str] = &[
    "mantle system",
    "crunch system eval",
    "examples/system-config",
    "docs/system-config",
    "SystemModule = import",
    "Inventory = import",
    "crunch-system",
    "system_cmd",
];

#[test]
fn help_succeeds_without_system_subcommand() {
    let output = mantle_command().arg("--help").assert().success().get_output().clone();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("Usage:"));
    assert!(!stdout.contains("\n  system"), "system subcommand should be absent from help:\n{stdout}");
}

#[test]
fn system_eval_is_not_a_supported_subcommand() {
    let mut cmd = mantle_command();
    cmd.args(["system", "eval", "inventory.ncl"]);
    cmd.assert().failure().stderr(
        predicate::str::contains("unrecognized subcommand 'system'")
            .or(predicate::str::contains("unrecognized subcommand \"system\"")),
    );
}

#[test]
fn raw_module_inventory_is_rejected_as_build_plan_input() {
    let dir = TempDir::new().unwrap();
    let inventory_path = dir.path().join("raw-onix-inventory.ncl");
    let store_path = dir.path().join("store");
    let state_path = dir.path().join("state");
    fs::write(&inventory_path, RAW_INVENTORY_REJECTION_INPUT).unwrap();

    let mut cmd = mantle_command();
    cmd.arg("--store")
        .arg(&store_path)
        .arg("--state-dir")
        .arg(&state_path)
        .arg("build")
        .arg("--plan")
        .arg("--no-substitute")
        .arg(&inventory_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("deserialization error").and(predicate::str::contains("machines")));
}

#[test]
fn public_docs_and_stdlib_do_not_reference_system_eval_surface() {
    let repo_root = repo_root();

    for removed_path in REMOVED_SYSTEM_SURFACE_PATHS {
        let path = repo_root.join(removed_path);
        assert!(!path.exists(), "removed system surface path still exists: {}", path.display());
    }

    let files = public_reference_files(repo_root);
    assert!(!files.is_empty());
    assert!(files.len() <= MAX_PUBLIC_SCAN_FILES);

    for file in files {
        let content = fs::read_to_string(&file).unwrap();
        for forbidden in FORBIDDEN_PUBLIC_PATTERNS {
            assert!(!content.contains(forbidden), "forbidden `{forbidden}` found in {}", file.display());
        }
    }
}

fn mantle_command() -> Command {
    Command::cargo_bin("mantle").unwrap()
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn public_reference_files(repo_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in PUBLIC_REFERENCE_ROOTS {
        collect_files(&repo_root.join(root), &mut files);
    }
    files.sort();
    files.dedup();
    files
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) {
    assert!(files.len() <= MAX_PUBLIC_SCAN_FILES);
    if !path.exists() {
        return;
    }
    if path.is_file() {
        files.push(path.to_path_buf());
        assert!(files.len() <= MAX_PUBLIC_SCAN_FILES);
        return;
    }
    let entries = fs::read_dir(path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        collect_files(&entry.path(), files);
    }
}
