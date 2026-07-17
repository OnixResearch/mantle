use std::fs;
use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::Value;

const FOREIGN_ROOT: &str = "examples/projects/foreign-import-handoff";
const CARGO_ROOT: &str = "examples/projects/cargo-import-offline";
const RECEIPT_ROOT: &str = "examples/projects/portable-receipt-handoff";
const REMOTE_ROOT: &str = "examples/projects/remote-build-loopback";
const WASM_ROOT: &str = "examples/projects/wasm-component-hello";
const TRANSCRIPT_PATH: &str = "examples/transcripts/hello-eval.md";
const POLICY_HASH: &str = "gallery-policy-v1";
const TEST_NOW_UNIX_S: u64 = 1_000;
const TEST_TICKET_TTL_SECS: u64 = 100;
const BLAKE3_A: &str = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const BLAKE3_B: &str = "blake3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn example_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn parse_json(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout should be JSON: {error}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("destination directory should be created");
    for entry in fs::read_dir(source).expect("source directory should be readable") {
        let entry = entry.expect("source entry should be readable");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_tree(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("source file should copy");
        }
    }
}

fn blocker_classes(report: &Value) -> Vec<&str> {
    report["blockers"]
        .as_array()
        .expect("blockers should be an array")
        .iter()
        .map(|blocker| blocker["class"].as_str().expect("blocker class should be a string"))
        .collect()
}

#[test]
fn workflow_entrypoints_evaluate_to_their_declared_shapes() {
    let cases = [
        (format!("{FOREIGN_ROOT}/workflow.ncl"), "mantle-example-workflow-v1"),
        (format!("{CARGO_ROOT}/workflow.ncl"), "mantle-example-workflow-v1"),
        (format!("{RECEIPT_ROOT}/mantle-project.ncl"), "portable-receipt-payload"),
        (format!("{REMOTE_ROOT}/mantle-project.ncl"), "remote-loopback-payload"),
        (format!("{WASM_ROOT}/workflow.ncl"), "mantle-wasm-component-export-v1"),
    ];

    for (path, expected) in cases {
        let output = mantle_cmd().args(["eval", &path]).output().expect("workflow evaluation should run");
        assert!(output.status.success(), "{path}: {}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains(expected), "{path} missing {expected}");
    }
}

#[test]
fn foreign_import_example_accepts_checked_inputs_and_rejects_untrusted_cache() {
    let root = example_path(FOREIGN_ROOT);
    let validate = mantle_cmd()
        .current_dir(&root)
        .args([
            "--json",
            "foreign-import",
            "validate",
            "--graph",
            "fixtures/guix-hello.graph.json",
            "--package-index",
            "fixtures/guix-hello.index.json",
            "--policy",
            "fixtures/policy.json",
        ])
        .output()
        .expect("foreign validate should run");
    let validate_report = parse_json(&validate);
    assert!(validate.status.success(), "stderr={}", String::from_utf8_lossy(&validate.stderr));
    assert_eq!(validate_report["accepted"], true);
    assert!(validate_report["receipt"]["raw_graph_digest"].as_str().is_some_and(|digest| !digest.is_empty()));

    let reject = mantle_cmd()
        .current_dir(&root)
        .args([
            "--json",
            "foreign-import",
            "validate",
            "--graph",
            "fixtures/untrusted-cache.graph.json",
            "--package-index",
            "fixtures/guix-hello.index.json",
            "--policy",
            "fixtures/policy.json",
        ])
        .output()
        .expect("negative foreign validate should run");
    let reject_report = parse_json(&reject);
    assert!(!reject.status.success());
    assert_eq!(reject_report["accepted"], false);
    assert_eq!(reject_report["diagnostics"][0]["class"], "untrusted-cache-hint");
}

#[test]
fn cargo_import_example_plans_applies_and_fails_before_invalid_writes() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    copy_tree(&example_path(&format!("{CARGO_ROOT}/workspace")), temp.path());

    let plan = mantle_cmd()
        .current_dir(temp.path())
        .args(["--json", "import", "cargo", "--plan"])
        .output()
        .expect("cargo import plan should run");
    let plan_report = parse_json(&plan);
    assert!(plan.status.success(), "stderr={}", String::from_utf8_lossy(&plan.stderr));
    assert!(blocker_classes(&plan_report).is_empty());
    assert!(!temp.path().join("mantle-project.ncl").exists());
    assert!(!temp.path().join(".mantle/inputs.ncl").exists());

    let apply = mantle_cmd()
        .current_dir(temp.path())
        .args(["--json", "import", "cargo", "--apply"])
        .output()
        .expect("cargo import apply should run");
    let apply_report = parse_json(&apply);
    assert!(apply.status.success(), "stderr={}", String::from_utf8_lossy(&apply.stderr));
    assert!(blocker_classes(&apply_report).is_empty());
    assert!(temp.path().join("mantle-project.ncl").is_file());
    assert!(temp.path().join(".mantle/inputs.ncl").is_file());

    let broken = tempfile::tempdir().expect("negative tempdir should be created");
    copy_tree(&example_path(&format!("{CARGO_ROOT}/workspace")), broken.path());
    fs::remove_file(broken.path().join("Cargo.lock")).expect("lockfile should be removed");
    let rejected = mantle_cmd()
        .current_dir(broken.path())
        .args(["--json", "import", "cargo", "--apply"])
        .output()
        .expect("negative cargo import should run");
    let rejected_report = parse_json(&rejected);
    assert!(!rejected.status.success());
    assert!(blocker_classes(&rejected_report).contains(&"missing-lockfile"));
    assert!(!broken.path().join("mantle-project.ncl").exists());
    assert!(!broken.path().join(".mantle/inputs.ncl").exists());
}

#[test]
fn cargo_import_example_rejects_ambiguous_default_package() {
    let output = mantle_cmd()
        .current_dir(example_path(&format!("{CARGO_ROOT}/ambiguous-workspace")))
        .args(["--json", "import", "cargo", "--plan"])
        .output()
        .expect("ambiguous cargo import should run");
    let report = parse_json(&output);

    assert!(!output.status.success());
    assert!(blocker_classes(&report).contains(&"ambiguous-default-package"));
    assert!(report["file_operations"].as_array().is_some_and(Vec::is_empty));
}

#[test]
fn receipt_example_exports_lists_and_rejects_tamper() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let bundle = temp.path().join("receipt.json");
    let record = format!("action-ref:gallery-recipe:{BLAKE3_A}");
    let export = mantle_cmd()
        .args([
            "--json",
            "receipt",
            "bundle",
            "export",
            "--record",
            &record,
            "--policy-hash",
            POLICY_HASH,
            "--to",
            bundle.to_str().expect("bundle path should be UTF-8"),
        ])
        .output()
        .expect("receipt export should run");
    assert!(export.status.success(), "stderr={}", String::from_utf8_lossy(&export.stderr));
    assert!(bundle.is_file());

    let list = mantle_cmd()
        .args([
            "--json",
            "receipt",
            "bundle",
            "list",
            "--from",
            bundle.to_str().expect("bundle path should be UTF-8"),
        ])
        .output()
        .expect("receipt list should run");
    let list_report = parse_json(&list);
    assert!(list.status.success());
    assert_eq!(list_report["format"], "mantle-build-receipt-bundle-v1");

    let mut tampered: Value = serde_json::from_slice(&fs::read(&bundle).expect("bundle should be readable")).unwrap();
    tampered["records"][0]["digest"] = Value::String(BLAKE3_B.to_string());
    fs::write(&bundle, serde_json::to_vec_pretty(&tampered).unwrap()).expect("tampered bundle should write");
    let rejected = mantle_cmd()
        .args([
            "--json",
            "receipt",
            "bundle",
            "list",
            "--from",
            bundle.to_str().expect("bundle path should be UTF-8"),
        ])
        .output()
        .expect("tampered receipt list should run");
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("receipt bundle digest mismatch"));
}

#[test]
fn semantic_graph_example_explains_output_and_dependents() {
    let graph_root = example_path(RECEIPT_ROOT);
    let why = mantle_cmd()
        .current_dir(&graph_root)
        .args([
            "--json",
            "why",
            "gallery-output",
            "--graph-file",
            "fixtures/semantic-graph.json",
        ])
        .output()
        .expect("semantic why should run");
    let why_report = parse_json(&why);
    assert!(why.status.success(), "stderr={}", String::from_utf8_lossy(&why.stderr));
    assert_eq!(why_report["producing_recipe"]["id"], "recipe:portable-receipt-gallery");

    let dependents = mantle_cmd()
        .current_dir(&graph_root)
        .args([
            "--json",
            "dependents",
            "source:portable-receipt-gallery",
            "--graph-file",
            "fixtures/semantic-graph.json",
        ])
        .output()
        .expect("semantic dependents should run");
    let dependents_report = parse_json(&dependents);
    assert!(dependents.status.success());
    assert_eq!(dependents_report["dependents"][0]["id"], "recipe:portable-receipt-gallery");
}

#[test]
fn semantic_graph_example_reports_incomplete_evidence_without_invention() {
    let output = mantle_cmd()
        .current_dir(example_path(RECEIPT_ROOT))
        .args([
            "--json",
            "why",
            "gallery-output",
            "--graph-file",
            "fixtures/incomplete-semantic-graph.json",
        ])
        .output()
        .expect("incomplete graph query should run");
    let report = parse_json(&output);

    assert!(!output.status.success());
    assert_eq!(report["schema"], "mantle-incomplete-semantic-graph-v1");
    assert!(report["missing"].as_array().unwrap().contains(&Value::String("producing recipe edge".to_string())));
}

#[test]
fn remote_ticket_example_redacts_reveals_and_revokes_explicitly() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let state = temp.path().to_str().expect("state path should be UTF-8");
    let now_unix_s = TEST_NOW_UNIX_S.to_string();
    let ticket_ttl_secs = TEST_TICKET_TTL_SECS.to_string();
    let create = mantle_cmd()
        .args([
            "--json",
            "--state-dir",
            state,
            "remote",
            "ticket",
            "create",
            "--display-name",
            "gallery-loopback",
            "--now-unix-s",
            &now_unix_s,
            "--ttl-secs",
            &ticket_ttl_secs,
            "--uses",
            "1",
        ])
        .output()
        .expect("remote ticket create should run");
    let created = parse_json(&create);
    let id = created["id"].as_str().expect("ticket id should be present");
    assert!(create.status.success());
    assert_eq!(created["secret"], "<redacted>");

    let reveal = mantle_cmd()
        .args(["--json", "--state-dir", state, "remote", "ticket", "reveal", id])
        .output()
        .expect("remote ticket reveal should run");
    let revealed = parse_json(&reveal);
    let secret = revealed["secret"].as_str().expect("revealed secret should be present");
    assert!(reveal.status.success());
    assert_ne!(secret, "<redacted>");

    let list = mantle_cmd()
        .args(["--json", "--state-dir", state, "remote", "ticket", "list"])
        .output()
        .expect("remote ticket list should run");
    assert!(list.status.success());
    assert!(!String::from_utf8_lossy(&list.stdout).contains(secret));

    let revoke = mantle_cmd()
        .args(["--json", "--state-dir", state, "remote", "ticket", "revoke", id])
        .output()
        .expect("remote ticket revoke should run");
    let revoked = parse_json(&revoke);
    assert!(revoke.status.success());
    assert_eq!(revoked["revoked"], true);
}

#[test]
fn remote_ticket_example_rejects_unknown_ticket() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let output = mantle_cmd()
        .args([
            "--json",
            "--state-dir",
            temp.path().to_str().expect("state path should be UTF-8"),
            "remote",
            "ticket",
            "inspect",
            "missing-ticket",
        ])
        .output()
        .expect("unknown ticket inspect should run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown remote ticket"));
}

#[test]
fn wasm_workflow_rejects_empty_world_at_the_typed_boundary() {
    let source = fs::read_to_string(example_path(&format!("{WASM_ROOT}/workflow.ncl"))).expect("workflow should read");
    let portable_source = source.replace("../../../lib/lib.ncl", "lib.ncl");
    let invalid = portable_source.replace("world = \"app\", source", "world = \"\", source");
    assert_ne!(portable_source, invalid, "fixture mutation must change the WIT world");
    let import_paths = vec![crunch_eval::stdlib::stdlib_import_path().unwrap().into_os_string()];

    let result = crunch_eval::evaluate_str(&invalid, &import_paths);

    assert!(result.is_err(), "empty WIT world must fail the typed contract");
}

#[test]
fn checked_gallery_transcript_executes_with_the_real_mantle_binary() {
    let temp = tempfile::tempdir().expect("tempdir should be created");
    let evidence = temp.path().join("transcript-output.json");
    let output = mantle_cmd()
        .args(["transcript", "run"])
        .arg(example_path(TRANSCRIPT_PATH))
        .arg("--output")
        .arg(&evidence)
        .arg("--mantle-bin")
        .arg(env!("CARGO_BIN_EXE_mantle"))
        .output()
        .expect("gallery transcript should run");
    let evidence_json: Value =
        serde_json::from_slice(&fs::read(&evidence).expect("transcript evidence should be written")).unwrap();

    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(evidence_json["schema"], "mantle-transcript-output-v1");
    assert_eq!(evidence_json["visible_steps"], 1);
    assert_eq!(evidence_json["runs"][0]["exit_code"], 0);
    assert!(evidence_json["cleanup_failures"].as_array().is_some_and(Vec::is_empty));
}
