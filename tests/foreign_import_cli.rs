use std::fs;
use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const FIXTURE_DIR: &str = "tests/fixtures/foreign-import";
const GUIX_GRAPH: &str = "guix-hello.graph.json";
const GUIX_INDEX: &str = "guix-hello.index.json";
const NIX_GRAPH: &str = "nix-hello.graph.json";
const NIX_INDEX: &str = "nix-hello.index.json";
const POLICY: &str = "policy.json";
const GUIX_VALIDATE_SNAPSHOT: &str = "guix-hello.validate.snapshot.json";
const GUIX_PLAN_SNAPSHOT: &str = "guix-hello.plan.snapshot.json";
const NIX_VALIDATE_SNAPSHOT: &str = "nix-hello.validate.snapshot.json";
const NIX_PLAN_SNAPSHOT: &str = "nix-hello.plan.snapshot.json";
const HELLO_PACKAGE: &str = "hello";
const HELLO_SYSTEM: &str = "x86_64-linux";
const MALFORMED_JSON: &str = "malformed-json";
const UNSUPPORTED_METADATA: &str = "unsupported-package-index-metadata";
const STALE_RECEIPT: &str = "stale-raw-graph-digest";
const EMBEDDED_REWRITE: &str = "undeclared-embedded-source-rewrite";
const UNTRUSTED_CACHE: &str = "untrusted-cache-hint";
const SANDBOX_CAPABILITY: &str = "undeclared-sandbox-capability";
const FAKE_PATH_DIR: &str = "fake-path";

struct FixtureCase {
    graph: &'static str,
    index: &'static str,
    validate_snapshot: &'static str,
    plan_snapshot: &'static str,
    root_node: &'static str,
}

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

#[test]
fn foreign_import_cli_validates_and_plans_checked_fixtures() {
    let cases = [
        FixtureCase {
            graph: GUIX_GRAPH,
            index: GUIX_INDEX,
            validate_snapshot: GUIX_VALIDATE_SNAPSHOT,
            plan_snapshot: GUIX_PLAN_SNAPSHOT,
            root_node: "guix:hello",
        },
        FixtureCase {
            graph: NIX_GRAPH,
            index: NIX_INDEX,
            validate_snapshot: NIX_VALIDATE_SNAPSHOT,
            plan_snapshot: NIX_PLAN_SNAPSHOT,
            root_node: "nix:hello",
        },
    ];

    for case in cases {
        let validate = run_validate_json(fixture_path(case.graph), fixture_path(case.index), fixture_path(POLICY));
        let expected_validate = fixture_json(case.validate_snapshot);
        assert_eq!(validate, expected_validate);
        assert_eq!(validate["accepted"], true);
        assert!(validate["receipt"]["raw_graph_digest"].as_str().unwrap().len() > 0);

        let plan = run_plan_json(fixture_path(case.graph), fixture_path(case.index), fixture_path(POLICY));
        let expected_plan = fixture_json(case.plan_snapshot);
        assert_eq!(plan, expected_plan);
        assert_eq!(plan["accepted"], true);
        assert_eq!(plan["plan"]["roots"][0]["node_id"], case.root_node);
        assert!(plan["plan"]["forbidden_process_invocations"].as_array().unwrap().is_empty());
        assert!(
            plan["plan"]["non_claims"]
                .as_array()
                .unwrap()
                .contains(&Value::String("not-output-trust".to_string()))
        );
    }
}

#[test]
fn foreign_import_cli_rejects_malformed_json_and_policy_failures() {
    let temp = TempDir::new().expect("tempdir should be created");
    let malformed = temp.path().join("malformed.json");
    fs::write(&malformed, "{").expect("malformed JSON should be written");
    assert_validate_rejects(&malformed, &fixture_path(GUIX_INDEX), &fixture_path(POLICY), MALFORMED_JSON);

    let unsupported_index = write_json(temp.path(), "unsupported-index.json", unsupported_metadata_index());
    assert_validate_rejects(&fixture_path(GUIX_GRAPH), &unsupported_index, &fixture_path(POLICY), UNSUPPORTED_METADATA);

    let receipt =
        run_validate_json(fixture_path(GUIX_GRAPH), fixture_path(GUIX_INDEX), fixture_path(POLICY))["receipt"].clone();
    let receipt_path = write_json(temp.path(), "receipt.json", receipt);
    let stale_graph = write_json(temp.path(), "stale-graph.json", renamed_graph());
    assert_validate_rejects_with_receipt(
        &stale_graph,
        &fixture_path(GUIX_INDEX),
        &fixture_path(POLICY),
        &receipt_path,
        STALE_RECEIPT,
    );

    let embedded_graph = write_json(temp.path(), "embedded-graph.json", embedded_source_graph());
    assert_validate_rejects(&embedded_graph, &fixture_path(GUIX_INDEX), &fixture_path(POLICY), EMBEDDED_REWRITE);

    let cache_graph = write_json(temp.path(), "cache-graph.json", untrusted_cache_graph());
    assert_validate_rejects(&cache_graph, &fixture_path(GUIX_INDEX), &fixture_path(POLICY), UNTRUSTED_CACHE);

    let sandbox_graph = write_json(temp.path(), "sandbox-graph.json", sandbox_capability_graph());
    assert_validate_rejects(&sandbox_graph, &fixture_path(GUIX_INDEX), &fixture_path(POLICY), SANDBOX_CAPABILITY);
}

#[test]
fn foreign_import_cli_does_not_require_foreign_frontend_commands() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    fs::create_dir(&fake_path).expect("fake PATH should be created");

    let output = mantle_cmd()
        .env("PATH", &fake_path)
        .args([
            "--json",
            "foreign-import",
            "plan",
            "--graph",
            path_str(&fixture_path(NIX_GRAPH)),
            "--package-index",
            path_str(&fixture_path(NIX_INDEX)),
            "--policy",
            path_str(&fixture_path(POLICY)),
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
        ])
        .assert()
        .success()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");

    assert_eq!(report["accepted"], true);
    assert!(report["plan"]["forbidden_process_invocations"].as_array().unwrap().is_empty());
    assert!(output.stderr.is_empty());
}

fn run_validate_json(graph: PathBuf, index: PathBuf, policy: PathBuf) -> Value {
    let output = mantle_cmd()
        .args([
            "--json",
            "foreign-import",
            "validate",
            "--graph",
            path_str(&graph),
            "--package-index",
            path_str(&index),
            "--policy",
            path_str(&policy),
        ])
        .assert()
        .success()
        .get_output()
        .clone();
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

fn run_plan_json(graph: PathBuf, index: PathBuf, policy: PathBuf) -> Value {
    let output = mantle_cmd()
        .args([
            "--json",
            "foreign-import",
            "plan",
            "--graph",
            path_str(&graph),
            "--package-index",
            path_str(&index),
            "--policy",
            path_str(&policy),
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
        ])
        .assert()
        .success()
        .get_output()
        .clone();
    assert!(output.stderr.is_empty());
    serde_json::from_slice(&output.stdout).expect("stdout should be JSON")
}

fn assert_validate_rejects(graph: &Path, index: &Path, policy: &Path, expected_class: &str) {
    let output = mantle_cmd()
        .args([
            "--json",
            "foreign-import",
            "validate",
            "--graph",
            path_str(graph),
            "--package-index",
            path_str(index),
            "--policy",
            path_str(policy),
        ])
        .assert()
        .failure()
        .get_output()
        .clone();
    assert_rejected_class(output.stdout, expected_class);
    assert!(output.stderr.is_empty());
}

fn assert_validate_rejects_with_receipt(
    graph: &Path,
    index: &Path,
    policy: &Path,
    receipt: &Path,
    expected_class: &str,
) {
    let output = mantle_cmd()
        .args([
            "--json",
            "foreign-import",
            "validate",
            "--graph",
            path_str(graph),
            "--package-index",
            path_str(index),
            "--policy",
            path_str(policy),
            "--receipt",
            path_str(receipt),
        ])
        .assert()
        .failure()
        .get_output()
        .clone();
    assert_rejected_class(output.stdout, expected_class);
    assert!(output.stderr.is_empty());
}

fn assert_rejected_class(stdout: Vec<u8>, expected_class: &str) {
    let report: Value = serde_json::from_slice(&stdout).expect("stdout should be JSON");
    assert_eq!(report["accepted"], false);
    assert_eq!(report["diagnostics"][0]["class"], expected_class);
    assert!(report["receipt"].is_null());
    assert!(report["plan"].is_null());
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR).join(name)
}

fn fixture_json(name: &str) -> Value {
    let contents = fs::read_to_string(fixture_path(name)).expect("fixture JSON should be readable");
    serde_json::from_str(&contents).expect("fixture JSON should parse")
}

fn write_json(root: &Path, name: &str, value: Value) -> PathBuf {
    let path = root.join(name);
    let bytes = serde_json::to_vec_pretty(&value).expect("JSON should serialize");
    fs::write(&path, bytes).expect("JSON fixture should be written");
    path
}

fn renamed_graph() -> Value {
    let mut graph = fixture_json(GUIX_GRAPH);
    graph["nodes"][0]["name"] = Value::String("hello-renamed".to_string());
    graph
}

fn unsupported_metadata_index() -> Value {
    let mut index = fixture_json(GUIX_INDEX);
    index["entries"][0]["unsupported_metadata_classes"] = serde_json::json!(["nix-overlay-order"]);
    index
}

fn embedded_source_graph() -> Value {
    let mut graph = fixture_json(GUIX_GRAPH);
    graph["source_payloads"][0]["embedded_text"] = Value::String("embedded /gnu/store/extra-source".to_string());
    graph
}

fn untrusted_cache_graph() -> Value {
    let mut graph = fixture_json(GUIX_GRAPH);
    graph["nodes"][0]["cache_hints"] = serde_json::json!([
        {
            "cache_url": "https://cache.example.invalid",
            "trust_scope": "trusted-binary-cache"
        }
    ]);
    graph
}

fn sandbox_capability_graph() -> Value {
    let mut graph = fixture_json(GUIX_GRAPH);
    graph["nodes"][0]["sandbox_capabilities"] = serde_json::json!(["chmod-setuid"]);
    graph
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("test paths should be UTF-8")
}
