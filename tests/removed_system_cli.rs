use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use base64::Engine;
use predicates::prelude::*;
use sha2::Digest;
use tempfile::TempDir;

// The required src + crates surface alone has 699 files at the repair baseline.
// Cover it completely while bounding traversal and actual content reads.
const MAX_PUBLIC_SCAN_FILES: usize = 1_024;
const MAX_SCAN_ENTRIES: usize = 2_048;
const MAX_SCAN_DEPTH: usize = 16;
const MAX_SCAN_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_SCAN_TOTAL_BYTES: u64 = 32 * 1024 * 1024;
const FRONTEND_PAYLOAD: &[u8] = b"frontend-produced build input\n";
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
const IMPLEMENTATION_REFERENCE_ROOTS: &[&str] = &["Cargo.toml", "flake.nix", "src", "crates", "lib"];
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
const FORBIDDEN_IMPLEMENTATION_COUPLING_PATTERNS: &[&str] = &[
    "crunch-system",
    "system_cmd",
    "SystemModule",
    "Inventory = import",
    "onix-modules",
    "nixosSystem",
    "nixosModules",
    "evalModules",
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
fn external_frontend_can_hand_mantle_build_shaped_input() {
    let dir = TempDir::new().unwrap();
    let source_path = dir.path().join("frontend-payload.txt");
    let ncl_path = dir.path().join("frontend-output.ncl");
    let store_path = dir.path().join("store");
    let state_path = dir.path().join("state");
    fs::write(&source_path, FRONTEND_PAYLOAD).unwrap();
    fs::create_dir(&store_path).unwrap();
    write_fetchurl_ncl(&ncl_path, &source_path, &sha256_sri(FRONTEND_PAYLOAD));

    let output = mantle_command()
        .arg("--store")
        .arg(&store_path)
        .arg("--state-dir")
        .arg(&state_path)
        .arg("build")
        .arg("--plan")
        .arg("--no-substitute")
        .arg(&ncl_path)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let plan_output = format!("{stdout}\n{stderr}");
    let planned_or_preflighted = plan_output.contains(": build") || plan_output.contains("preflight-error");

    assert!(plan_output.contains("frontend-output"), "build plan should name frontend output:\n{plan_output}");
    assert!(planned_or_preflighted, "build-shaped input should reach build planning:\n{plan_output}");
    assert!(
        !plan_output.contains("deserialization error"),
        "build-shaped input must deserialize:\n{plan_output}"
    );
    assert!(!plan_output.contains("roles"), "build plan should not interpret module roles:\n{plan_output}");
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

    let output = cmd.assert().failure().get_output().clone();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("eager derivation evaluation:"), "{stderr}");
    assert!(stderr.contains("completed 0 of 2 roots and discovered 0 actions"), "{stderr}");
    assert!(!store_path.exists(), "raw inventory must not create a store");
    assert!(!state_path.exists(), "raw inventory must not create state");
}

#[test]
fn public_docs_and_stdlib_do_not_reference_system_eval_surface() {
    let repo_root = repo_root();

    for removed_path in REMOVED_SYSTEM_SURFACE_PATHS {
        let path = repo_root.join(removed_path);
        assert!(!path.exists(), "removed system surface path still exists: {}", path.display());
    }

    let files = public_reference_files(repo_root);
    assert_no_forbidden_patterns(&files, FORBIDDEN_PUBLIC_PATTERNS);
}

#[test]
fn implementation_surface_does_not_gain_module_layer_coupling() {
    let files = implementation_reference_files(repo_root());
    assert_no_forbidden_patterns(&files, FORBIDDEN_IMPLEMENTATION_COUPLING_PATTERNS);
}

fn mantle_command() -> Command {
    Command::cargo_bin("mantle").unwrap()
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn write_fetchurl_ncl(path: &Path, source_path: &Path, hash_sri: &str) {
    let lib_path = repo_root().join("lib/lib.ncl");
    fs::write(
        path,
        format!(
            r#"let mantle = import "{}" in
mantle.fetchurl {{
  url = "file://{}",
  hash = "{}",
  name = "frontend-output",
}}
"#,
            lib_path.display(),
            source_path.display(),
            hash_sri,
        ),
    )
    .unwrap();
}

fn sha256_sri(bytes: &[u8]) -> String {
    let digest = sha2::Sha256::digest(bytes);
    let encoded = base64::engine::general_purpose::STANDARD.encode(digest);
    format!("sha256-{encoded}")
}

fn public_reference_files(repo_root: &Path) -> Vec<PathBuf> {
    reference_files(repo_root, PUBLIC_REFERENCE_ROOTS)
}

fn implementation_reference_files(repo_root: &Path) -> Vec<PathBuf> {
    reference_files(repo_root, IMPLEMENTATION_REFERENCE_ROOTS)
}

fn reference_files(repo_root: &Path, roots: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut entries = 0;
    for root in roots {
        collect_files(&repo_root.join(root), &mut files, &mut entries, 0);
    }
    files.sort();
    files.dedup();
    assert!(!files.is_empty());
    assert!(files.len() <= MAX_PUBLIC_SCAN_FILES);
    files
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>, entries: &mut usize, depth: usize) {
    assert!(depth <= MAX_SCAN_DEPTH, "scan depth limit");
    assert!(*entries < MAX_SCAN_ENTRIES, "scan entry limit");
    *entries = entries.saturating_add(1);
    let metadata = fs::symlink_metadata(path).unwrap();
    assert!(!metadata.is_symlink(), "scan must not follow symlinks");
    if metadata.is_file() {
        assert!(files.len() < MAX_PUBLIC_SCAN_FILES, "scan file limit");
        files.push(path.to_path_buf());
        return;
    }
    assert!(metadata.is_dir(), "scan rejects special files");
    for entry in fs::read_dir(path).unwrap() {
        collect_files(&entry.unwrap().path(), files, entries, depth.saturating_add(1));
    }
}

fn read_reference_file(path: &Path, total_bytes: &mut u64, file_limit_bytes: u64, total_limit_bytes: u64) -> String {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .unwrap()
        .take(file_limit_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .unwrap();
    let count = u64::try_from(bytes.len()).unwrap();
    assert!(count <= file_limit_bytes, "scan file byte limit");
    *total_bytes = total_bytes.checked_add(count).unwrap();
    assert!(*total_bytes <= total_limit_bytes, "scan total byte limit");
    String::from_utf8(bytes).unwrap()
}

#[test]
fn scan_byte_limits_accept_exact_and_reject_excess_without_truncation() {
    // r[verify native_package_parity.inventories]
    let root = TempDir::new().unwrap();
    let file = root.path().join("source.rs");
    fs::write(&file, "ab").unwrap();
    assert_eq!(read_reference_file(&file, &mut 0, 2, 2), "ab");
    assert!(std::panic::catch_unwind(|| read_reference_file(&file, &mut 0, 1, 2)).is_err());
    assert!(std::panic::catch_unwind(|| read_reference_file(&file, &mut 1, 2, 2)).is_err());
}

#[test]
fn scan_entry_limit_rejects_before_incomplete_coverage_can_pass() {
    let root = TempDir::new().unwrap();
    let file = root.path().join("source.rs");
    fs::write(&file, "allowed").unwrap();
    let mut files = Vec::new();
    collect_files(&file, &mut files, &mut 0, MAX_SCAN_DEPTH);
    assert_eq!(files, vec![file.clone()]);
    assert!(
        std::panic::catch_unwind(|| {
            let mut entries = MAX_SCAN_ENTRIES;
            collect_files(&file, &mut Vec::new(), &mut entries, 0);
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            collect_files(&file, &mut Vec::new(), &mut 0, MAX_SCAN_DEPTH.saturating_add(1));
        })
        .is_err()
    );
}

#[test]
#[should_panic(expected = "forbidden")]
fn boundary_scan_rejects_an_injected_module_dependency() {
    let root = TempDir::new().unwrap();
    let file = root.path().join("source.rs");
    fs::write(&file, FORBIDDEN_IMPLEMENTATION_COUPLING_PATTERNS[0]).unwrap();
    assert_no_forbidden_patterns(&[file], FORBIDDEN_IMPLEMENTATION_COUPLING_PATTERNS);
}

fn assert_no_forbidden_patterns(files: &[PathBuf], forbidden_patterns: &[&str]) {
    assert!(!files.is_empty());
    assert!(!forbidden_patterns.is_empty());
    let mut total_bytes = 0;
    for file in files {
        let content = read_reference_file(file, &mut total_bytes, MAX_SCAN_FILE_BYTES, MAX_SCAN_TOTAL_BYTES);
        assert_reference_content(file, &content, forbidden_patterns);
    }
}

fn assert_reference_content(file: &Path, content: &str, forbidden_patterns: &[&str]) {
    // This exact generator field records reviewed external-source provenance.
    // It does not import or evaluate modules. No other occurrence is exempt.
    let reference_owner =
        repo_root().join("crates/crunch-resource-policy-core/examples/generate_resource_policy_fixtures.rs");
    let reference_line = "        repository: \"github.com/onixcomputer/onix-modules\".to_string(),";
    let mut reference_seen = false;
    for line in content.lines() {
        if file == reference_owner && line == reference_line && !reference_seen {
            reference_seen = true;
            continue;
        }
        for forbidden in forbidden_patterns {
            assert!(!line.contains(forbidden), "forbidden `{forbidden}` found in {}", file.display());
        }
    }
}

#[test]
fn reviewed_source_metadata_does_not_exempt_another_reference_or_runtime_owner() {
    // r[verify native_package_parity.inventories]
    let owner = repo_root().join("crates/crunch-resource-policy-core/examples/generate_resource_policy_fixtures.rs");
    let field = "        repository: \"github.com/onixcomputer/onix-modules\".to_string(),";
    assert_reference_content(&owner, field, FORBIDDEN_IMPLEMENTATION_COUPLING_PATTERNS);
    assert!(
        std::panic::catch_unwind(|| assert_reference_content(
            &repo_root().join("src/main.rs"),
            field,
            FORBIDDEN_IMPLEMENTATION_COUPLING_PATTERNS
        ))
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| assert_reference_content(
            &owner,
            &format!("{field}\n{field}"),
            FORBIDDEN_IMPLEMENTATION_COUPLING_PATTERNS
        ))
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| assert_reference_content(
            &owner,
            &format!("{field}\nuse SystemModule;"),
            FORBIDDEN_IMPLEMENTATION_COUPLING_PATTERNS
        ))
        .is_err()
    );
}
