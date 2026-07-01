use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::Value;

const EXPECTED_RECORD_COUNT: u64 = 1;
const SOURCE_BUNDLE_STATE_DIR: &str = "source-bundles";
const SOURCE_BUNDLE_RECORDS_DIR: &str = "records";
const TEST_SOURCE_KIND: &str = "local-path";
const TEST_SOURCE_IDENTITY: &str = "fixture";
const TEST_SOURCE_FILE: &str = "src/main.txt";
const TEST_SOURCE_BYTES: &[u8] = b"hello";
const TAMPERED_SOURCE_CONTENT_HEX: &str = "6a656c6c6f";

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should be built")
}

fn write_source_tree(root: &Path) {
    std::fs::create_dir_all(root.join("src")).expect("create source src dir");
    std::fs::write(root.join(TEST_SOURCE_FILE), TEST_SOURCE_BYTES).expect("write source fixture");
}

fn source_spec(path: &Path) -> String {
    format!("{TEST_SOURCE_KIND}:{TEST_SOURCE_IDENTITY}:{}", path.display())
}

fn read_json_stdout(output: std::process::Output) -> Value {
    assert!(output.status.success(), "command failed: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

fn state_records_dir(state_dir: &Path) -> PathBuf {
    state_dir.join(SOURCE_BUNDLE_STATE_DIR).join(SOURCE_BUNDLE_RECORDS_DIR)
}

#[test]
fn source_bundle_cli_round_trips_imported_source_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source");
    let state_dir = temp.path().join("state");
    let bundle_path = temp.path().join("bundle.json");
    write_source_tree(&source_root);

    let plan = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "plan", "--source"])
            .arg(source_spec(&source_root))
            .output()
            .expect("run source bundle plan"),
    );
    assert_eq!(plan["ready_class"], "ready");
    assert_eq!(plan["record_count"].as_u64(), Some(EXPECTED_RECORD_COUNT));
    assert!(!state_records_dir(&state_dir).exists(), "plan must not mutate source state");

    let export = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "export", "--source"])
            .arg(source_spec(&source_root))
            .arg("--to")
            .arg(&bundle_path)
            .output()
            .expect("run source bundle export"),
    );
    assert_eq!(export["ready_class"], "ready");
    assert!(bundle_path.is_file(), "export should write the bundle");

    let list = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "list", "--from"])
            .arg(&bundle_path)
            .output()
            .expect("run source bundle list"),
    );
    assert_eq!(list["record_count"].as_u64(), Some(EXPECTED_RECORD_COUNT));
    assert!(!state_records_dir(&state_dir).exists(), "list must not mutate source state");

    let missing = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "verify", "--from"])
            .arg(&bundle_path)
            .arg("--imported")
            .output()
            .expect("run source bundle verify before import"),
    );
    assert_eq!(missing["ready_class"], "missing");
    assert_eq!(
        missing["missing_records"].as_array().expect("missing records").len(),
        EXPECTED_RECORD_COUNT as usize
    );

    let import = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "import", "--from"])
            .arg(&bundle_path)
            .arg("--pin")
            .output()
            .expect("run source bundle import"),
    );
    assert_eq!(import["imported_count"].as_u64(), Some(EXPECTED_RECORD_COUNT));
    assert_eq!(import["pinned"], true);

    let ready = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "verify", "--from"])
            .arg(&bundle_path)
            .arg("--imported")
            .output()
            .expect("run source bundle verify after import"),
    );
    assert_eq!(ready["ready_class"], "ready");
    assert!(ready["missing_records"].as_array().expect("missing records").is_empty());
}

#[test]
fn source_bundle_cli_rejects_tampered_bundle_without_persisting_records() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source");
    let state_dir = temp.path().join("state");
    let bundle_path = temp.path().join("bundle.json");
    write_source_tree(&source_root);

    read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "export", "--source"])
            .arg(source_spec(&source_root))
            .arg("--to")
            .arg(&bundle_path)
            .output()
            .expect("run source bundle export"),
    );

    let mut bundle: Value =
        serde_json::from_slice(&std::fs::read(&bundle_path).expect("read bundle")).expect("bundle should be JSON");
    bundle["records"][0]["files"][0]["content_hex"] = Value::String(TAMPERED_SOURCE_CONTENT_HEX.to_string());
    std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).expect("render tampered bundle"))
        .expect("write tampered bundle");

    let output = crunch_cmd()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .args(["source", "bundle", "import", "--from"])
        .arg(&bundle_path)
        .output()
        .expect("run source bundle import");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "tampered bundle import must fail");
    assert!(stderr.contains("digest mismatch"), "stderr should name the digest mismatch: {stderr}");
    assert!(!state_records_dir(&state_dir).exists(), "failed import must not persist source records");
}
