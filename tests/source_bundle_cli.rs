use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::Value;

const EXPECTED_RECORD_COUNT: u64 = 1;
const SOURCE_BUNDLE_STATE_DIR: &str = "source-bundles";
const SOURCE_BUNDLE_RECORDS_DIR: &str = "records";
const READY_CLASS_NETWORK_REQUIRED: &str = "network-required";
const READY_CLASS_STALE: &str = "stale";
const READY_CLASS_UNPINNED: &str = "unpinned";
const NEXT_ACTIONS_FIELD: &str = "next_actions";
const NETWORK_REQUIRED_NEXT_ACTION: &str = "network-required-source";
const UNPINNED_NEXT_ACTION: &str = "unpinned-source-state";
const TEST_FIXED_OUTPUT_HASH: &str = "sha256-3jC9jts9aiU9fst7yGXpoRxcwMsB2CQ2EOnFjBhJ3CI=";
const TEST_FETCH_NAME: &str = "source-fixture";
const TEST_SOURCE_KIND: &str = "local-path";
const TEST_SOURCE_IDENTITY: &str = "fixture";
const TEST_SOURCE_FILE: &str = "src/main.txt";
const TEST_SOURCE_BYTES: &[u8] = b"hello";
const TAMPERED_SOURCE_CONTENT_HEX: &str = "6a656c6c6f";
const TEST_CACHED_HTTPS_URL: &str = "https://example.invalid/pinned-android-tool.zip";
const TEST_CACHED_HTTPS_BYTES: &[u8] = b"deterministic Android tool archive\n";

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
    parse_json_stdout(&output)
}

fn read_json_stdout_from_failure(output: std::process::Output) -> Value {
    assert!(!output.status.success(), "command should fail closed");
    parse_json_stdout(&output)
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

fn state_records_dir(state_dir: &Path) -> PathBuf {
    state_dir.join(SOURCE_BUNDLE_STATE_DIR).join(SOURCE_BUNDLE_RECORDS_DIR)
}

fn write_fetchurl_build_root(root: &Path, payload: &Path) {
    let payload_url = format!("file://{}", payload.display());
    let source = format!(
        r#"let mantle = import "lib.ncl" in

mantle.fetchurl {{
  url = "{payload_url}",
  hash = "{TEST_FIXED_OUTPUT_HASH}",
  name = "{TEST_FETCH_NAME}",
}}
"#
    );
    std::fs::write(root, source).expect("write fetchurl build root");
}

fn write_cached_https_build_root(root: &Path, url: &str) -> String {
    use base64::Engine as _;
    use sha2::Digest as _;

    let hash = format!(
        "sha256-{}",
        base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(TEST_CACHED_HTTPS_BYTES))
    );
    let source = format!(
        r#"let mantle = import "lib.ncl" in

mantle.fetchurl {{
  url = "{url}",
  hash = "{hash}",
  name = "pinned-android-tool",
}}
"#
    );
    std::fs::write(root, source).expect("write pinned HTTPS fetchurl build root");
    hash
}

fn cached_https_export_command(state_dir: &Path, root: &Path) -> Command {
    let mut command = crunch_cmd();
    command
        .arg("--json")
        .arg("--state-dir")
        .arg(state_dir)
        .args(["source", "bundle", "export", "--build-root"])
        .arg(root)
        .args(["--import-path", "lib"]);
    command
}

fn write_dynamic_plan_output_build_root(root: &Path, payload: &Path) {
    let payload_url = format!("file://{}", payload.display());
    let source = format!(
        r#"let mantle = import "lib.ncl" in
let source = mantle.fetchurl {{
  url = "{payload_url}",
  hash = "{TEST_FIXED_OUTPUT_HASH}",
  name = "{TEST_FETCH_NAME}",
}} in
{{
  name = "consumer",
  builder = "/bin/sh",
  inputs = [{{
    name = "app",
    producer = {{
      name = "producer",
      builder = "/bin/sh",
      outputs = ["out", "plan"],
      dynamic_plan_outputs = ["plan"],
      inputs = [source],
    }},
    plan_output = "plan",
    root = "unit.app",
    unit_output = "out",
  }}],
}}
"#
    );
    std::fs::write(root, source).expect("write dynamic plan-output build root");
}
fn first_state_record_path(state_dir: &Path) -> PathBuf {
    let records_dir = state_records_dir(state_dir);
    std::fs::read_dir(records_dir)
        .expect("read source state records dir")
        .map(|entry| entry.expect("read source state record entry").path())
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .expect("source state record exists")
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

    let replay = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "import", "--from"])
            .arg(&bundle_path)
            .arg("--pin")
            .output()
            .expect("replay source bundle import"),
    );
    assert_eq!(replay["imported_count"].as_u64(), Some(0));
    assert_eq!(replay["skipped_present_count"].as_u64(), Some(EXPECTED_RECORD_COUNT));
    assert_eq!(replay["manifest_blake3"], import["manifest_blake3"]);

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
fn source_bundle_cli_reports_stale_imported_state() {
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
    read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "import", "--from"])
            .arg(&bundle_path)
            .output()
            .expect("run source bundle import"),
    );
    let record_path = first_state_record_path(&state_dir);
    let mut record: Value = serde_json::from_slice(&std::fs::read(&record_path).expect("read state record"))
        .expect("state record should be JSON");
    record["identity"] = Value::String("fixture-other-identity".to_string());
    std::fs::write(&record_path, serde_json::to_vec_pretty(&record).expect("render state record"))
        .expect("write state record");

    let stale = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "verify", "--from"])
            .arg(&bundle_path)
            .arg("--imported")
            .output()
            .expect("run source bundle verify"),
    );

    assert_eq!(stale["ready_class"], READY_CLASS_STALE);
    assert_eq!(stale["stale_records"].as_array().expect("stale records").len(), EXPECTED_RECORD_COUNT as usize);
}

#[test]
fn source_bundle_cli_preflight_reports_unpinned_imported_state() {
    let temp = tempfile::tempdir().expect("tempdir");
    let payload = temp.path().join("payload.txt");
    let root = temp.path().join("root.ncl");
    let state_dir = temp.path().join("state");
    let bundle_path = temp.path().join("bundle.json");
    std::fs::write(&payload, TEST_SOURCE_BYTES).expect("write payload");
    write_fetchurl_build_root(&root, &payload);

    read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "export", "--build-root"])
            .arg(&root)
            .args(["--import-path", "lib", "--to"])
            .arg(&bundle_path)
            .output()
            .expect("run source bundle export"),
    );
    read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "import", "--from"])
            .arg(&bundle_path)
            .output()
            .expect("run source bundle import"),
    );

    let unpinned = read_json_stdout_from_failure(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "preflight", "--build-root"])
            .arg(&root)
            .args(["--import-path", "lib"])
            .output()
            .expect("run source bundle preflight"),
    );

    assert_eq!(unpinned["ready_class"], READY_CLASS_UNPINNED);
    assert_eq!(
        unpinned["unpinned_records"].as_array().expect("unpinned records").len(),
        EXPECTED_RECORD_COUNT as usize
    );
    assert_eq!(unpinned[NEXT_ACTIONS_FIELD][0]["blocker_class"], UNPINNED_NEXT_ACTION);
    assert!(
        unpinned[NEXT_ACTIONS_FIELD][0]["command_hint"]
            .as_str()
            .expect("next-action command hint")
            .contains("--pin")
    );
}

#[test]
fn source_bundle_cli_preflight_reports_network_required_before_build() {
    let temp = tempfile::tempdir().expect("tempdir");
    let state_dir = temp.path().join("state");

    let report = read_json_stdout_from_failure(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args([
                "source",
                "bundle",
                "preflight",
                "--build-root",
                "examples/fetch-crate-crc64.ncl",
            ])
            .args(["--import-path", "lib"])
            .output()
            .expect("run source bundle preflight"),
    );

    assert_eq!(report["ready_class"], READY_CLASS_NETWORK_REQUIRED);
    assert_eq!(
        report["network_required_records"].as_array().expect("network-required records").len(),
        EXPECTED_RECORD_COUNT as usize
    );
    assert_eq!(report[NEXT_ACTIONS_FIELD][0]["blocker_class"], NETWORK_REQUIRED_NEXT_ACTION);
    assert!(
        report[NEXT_ACTIONS_FIELD][0]["command_hint"]
            .as_str()
            .expect("next-action command hint")
            .contains("--offline-source-preflight")
    );
}

#[test]
fn source_bundle_cli_preflights_dynamic_producer_sources_before_and_after_import() {
    let temp = tempfile::tempdir().expect("tempdir");
    let payload = temp.path().join("payload.txt");
    let root = temp.path().join("root.ncl");
    let state_dir = temp.path().join("state");
    let bundle_path = temp.path().join("bundle.json");
    std::fs::write(&payload, TEST_SOURCE_BYTES).expect("write source payload");
    write_dynamic_plan_output_build_root(&root, &payload);

    let missing = read_json_stdout_from_failure(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "preflight", "--build-root"])
            .arg(&root)
            .args(["--import-path", "lib"])
            .output()
            .expect("preflight dynamic source before import"),
    );
    assert_eq!(missing["ready_class"], "missing");
    assert_eq!(missing["missing_records"].as_array().expect("missing source records").len(), 1);

    let exported = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "export", "--build-root"])
            .arg(&root)
            .args(["--import-path", "lib", "--to"])
            .arg(&bundle_path)
            .output()
            .expect("export dynamic producer source"),
    );
    assert_eq!(exported["record_count"], EXPECTED_RECORD_COUNT);
    let imported = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "import", "--from"])
            .arg(&bundle_path)
            .arg("--pin")
            .output()
            .expect("pin dynamic producer source"),
    );
    assert_eq!(imported["imported_count"], EXPECTED_RECORD_COUNT);

    let ready = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "preflight", "--build-root"])
            .arg(&root)
            .args(["--import-path", "lib"])
            .output()
            .expect("preflight dynamic source after import"),
    );
    assert_eq!(ready["ready_class"], "ready");
    assert_eq!(ready["record_count"], exported["record_count"]);
    assert!(ready["missing_records"].as_array().expect("ready source records").is_empty());
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

#[test]
fn source_bundle_cli_caches_pinned_https_archive_for_offline_import_and_preflight() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("android-tool.ncl");
    let archive = temp.path().join("android-tool.zip");
    let state_dir = temp.path().join("state");
    let bundle_path = temp.path().join("android-tool.bundle.json");
    let hash = write_cached_https_build_root(&root, TEST_CACHED_HTTPS_URL);
    std::fs::write(&archive, TEST_CACHED_HTTPS_BYTES).expect("write local pinned archive");
    let mapping = format!("{TEST_CACHED_HTTPS_URL}={}", archive.display());

    let export = read_json_stdout(
        cached_https_export_command(&state_dir, &root)
            .arg("--cached-fetch")
            .arg(&mapping)
            .arg("--to")
            .arg(&bundle_path)
            .output()
            .expect("export exact pinned HTTPS archive from local file"),
    );
    assert_eq!(export["ready_class"], "ready");
    assert_eq!(export["record_count"].as_u64(), Some(EXPECTED_RECORD_COUNT));
    assert_eq!(
        export["payload_bytes"].as_u64(),
        Some(u64::try_from(TEST_CACHED_HTTPS_BYTES.len()).expect("fixture byte count fits"))
    );
    let bundle: Value =
        serde_json::from_slice(&std::fs::read(&bundle_path).expect("read source bundle")).expect("parse source bundle");
    assert_eq!(bundle["records"][0]["kind"], "fixed-url");
    assert_eq!(bundle["records"][0]["metadata"]["url"], TEST_CACHED_HTTPS_URL);
    assert_eq!(bundle["records"][0]["metadata"]["hash"], hash);

    let imported = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "import", "--from"])
            .arg(&bundle_path)
            .arg("--pin")
            .output()
            .expect("import and pin cached HTTPS source"),
    );
    assert_eq!(imported["imported_count"].as_u64(), Some(EXPECTED_RECORD_COUNT));
    assert_eq!(imported["pinned"], true);

    let preflight = read_json_stdout(
        crunch_cmd()
            .arg("--json")
            .arg("--state-dir")
            .arg(&state_dir)
            .args(["source", "bundle", "preflight", "--build-root"])
            .arg(&root)
            .args(["--import-path", "lib"])
            .output()
            .expect("preflight pinned HTTPS source without network"),
    );
    assert_eq!(preflight["ready_class"], "ready");
    assert_eq!(preflight["record_count"].as_u64(), Some(EXPECTED_RECORD_COUNT));
    assert!(preflight["network_required_records"].as_array().expect("network blockers").is_empty());
    assert!(preflight["unpinned_records"].as_array().expect("unpinned blockers").is_empty());
}

#[test]
fn source_bundle_cli_preserves_pinned_query_url_and_archive_path_containing_equals() {
    const QUERY_URL: &str = "https://example.invalid/pinned-android-tool.zip?channel=stable";
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("android-tool-query.ncl");
    let archive = temp.path().join("android=tool.zip");
    let state_dir = temp.path().join("state");
    let bundle_path = temp.path().join("android-tool-query.bundle.json");
    let rejected_path = temp.path().join("wrong-url.bundle.json");
    let expected_hash = write_cached_https_build_root(&root, QUERY_URL);
    std::fs::write(&archive, TEST_CACHED_HTTPS_BYTES).expect("write pinned local archive");

    let wrong_url = format!("https://example.invalid/pinned-android-tool.zip?channel=other={}", archive.display());
    let wrong = cached_https_export_command(&state_dir, &root)
        .arg("--cached-fetch")
        .arg(&wrong_url)
        .arg("--to")
        .arg(&rejected_path)
        .output()
        .expect("reject an unpinned query URL");
    let diagnostic = String::from_utf8_lossy(&wrong.stderr);
    assert!(!wrong.status.success(), "wrong pinned URL must fail closed");
    assert!(diagnostic.contains("does not match a pinned fixed-output source"), "{diagnostic}");
    assert!(!rejected_path.exists(), "unmatched URL must not publish a bundle");

    let mapping = format!("{QUERY_URL}={}", archive.display());
    let export = read_json_stdout(
        cached_https_export_command(&state_dir, &root)
            .arg("--cached-fetch")
            .arg(&mapping)
            .arg("--to")
            .arg(&bundle_path)
            .output()
            .expect("export pinned query URL using equals-bearing local archive path"),
    );
    assert_eq!(export["ready_class"], "ready");
    let bundle: Value =
        serde_json::from_slice(&std::fs::read(&bundle_path).expect("read exported bundle")).expect("parse bundle");
    let record = &bundle["records"][0];
    assert_eq!(record["metadata"]["url"], QUERY_URL);
    assert_eq!(record["metadata"]["hash"], expected_hash);
    assert_eq!(record["payload_bytes"].as_u64(), Some(TEST_CACHED_HTTPS_BYTES.len() as u64));
    assert_eq!(record["files"][0]["content_hex"], data_encoding::HEXLOWER.encode(TEST_CACHED_HTTPS_BYTES));
}

#[test]
fn source_bundle_cli_rejects_wrong_duplicate_and_drifted_cached_https_archives() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path().join("android-tool.ncl");
    let archive = temp.path().join("android-tool.zip");
    let state_dir = temp.path().join("state");
    let bundle_path = temp.path().join("rejected.bundle.json");
    write_cached_https_build_root(&root, TEST_CACHED_HTTPS_URL);
    std::fs::write(&archive, TEST_CACHED_HTTPS_BYTES).expect("write local pinned archive");
    let mapping = format!("{TEST_CACHED_HTTPS_URL}={}", archive.display());
    let wrong_mapping = format!("https://example.invalid/other-android-tool.zip={}", archive.display());

    let wrong = cached_https_export_command(&state_dir, &root)
        .arg("--cached-fetch")
        .arg(&wrong_mapping)
        .arg("--to")
        .arg(&bundle_path)
        .output()
        .expect("reject unmatched cached HTTPS URL");
    let wrong_diagnostic = String::from_utf8_lossy(&wrong.stderr);
    assert!(!wrong.status.success(), "unmatched URL must fail closed");
    assert!(wrong_diagnostic.contains("does not match a pinned fixed-output source"), "{wrong_diagnostic}");
    assert!(!bundle_path.exists(), "unmatched URL must not publish a bundle");

    let duplicate = cached_https_export_command(&state_dir, &root)
        .arg("--cached-fetch")
        .arg(&mapping)
        .arg("--cached-fetch")
        .arg(&mapping)
        .arg("--to")
        .arg(&bundle_path)
        .output()
        .expect("reject duplicate cached HTTPS URL");
    let duplicate_diagnostic = String::from_utf8_lossy(&duplicate.stderr);
    assert!(!duplicate.status.success(), "duplicate URL must fail closed");
    assert!(duplicate_diagnostic.contains("duplicate cached fetch URL"), "{duplicate_diagnostic}");
    assert!(!bundle_path.exists(), "duplicate URL must not publish a bundle");

    std::fs::write(&archive, b"drifted Android tool archive\n").expect("drift cached archive bytes");
    let drifted = cached_https_export_command(&state_dir, &root)
        .arg("--cached-fetch")
        .arg(&mapping)
        .arg("--to")
        .arg(&bundle_path)
        .output()
        .expect("reject drifted cached archive");
    let drift_diagnostic = String::from_utf8_lossy(&drifted.stderr);
    assert!(!drifted.status.success(), "SHA drift must fail closed");
    assert!(drift_diagnostic.contains("hash mismatch"), "{drift_diagnostic}");
    assert!(!bundle_path.exists(), "drifted archive must not publish a bundle");
    assert!(!state_records_dir(&state_dir).exists(), "rejected cached sources must not be imported");
}
