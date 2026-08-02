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
const NIXPKGS_DERIVATION_JSON: &str = "nixpkgs-hello.derivation-json.json";
const NIXPKGS_ROOT_DRV_FILE: &str = "nixpkgs-hello-root.drv";
const NIXPKGS_SOURCE_DRV_FILE: &str = "nixpkgs-hello-source.drv";
const GUIXPKGS_ROOT_DRV_FILE: &str = "guixpkgs-hello-root.drv";
const GUIXPKGS_SOURCE_DRV_FILE: &str = "guixpkgs-hello-source.drv";
const NIXPKGS_POLICY: &str = "nixpkgs-policy.json";
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
const MALFORMED_NIX_DERIVATION: &str = "malformed-nix-derivation";
const FAKE_PATH_DIR: &str = "fake-path";
const ATERM_GRAPH_FILE: &str = "foreign-aterm.graph.json";
const ATERM_INDEX_FILE: &str = "foreign-aterm.index.json";
const NIX_SOURCE_PREFIX: &str = "/nix/store";
const GUIX_SOURCE_PREFIX: &str = "/gnu/store";
const NIXPKGS_HELLO_DRV: &str = "/nix/store/22222222222222222222222222222222-hello.drv";
const NIXPKGS_SOURCE_DRV: &str = "/nix/store/44444444444444444444444444444444-hello-source.drv";
const GUIXPKGS_HELLO_DRV: &str = "/gnu/store/22222222222222222222222222222222-hello.drv";
const GUIXPKGS_SOURCE_DRV: &str = "/gnu/store/44444444444444444444444444444444-hello-source.drv";
const NIXPKGS_UNRELATED_DRV: &str = "/nix/store/66666666666666666666666666666666-unrelated.drv";
const NIXPKGS_REACHABLE_DRV_COUNT: usize = 2;
const CACHE_NIXOS_ORG: &str = "https://cache.nixos.org";
const TRUSTED_CACHE_SCOPE: &str = "trusted-binary-cache";
const EXECUTABLE_PLAN_SCHEMA: &str = "mantle-foreign-executable-plan-v1";
const BLAKE3_ALGORITHM: &str = "blake3";
const KIBIBYTE_BYTES: usize = 1_024;
const MEBIBYTE_BYTES: usize = KIBIBYTE_BYTES * KIBIBYTE_BYTES;
const MAX_ATERM_DERIVATION_MEBIBYTES: usize = 16;
const OVERSIZED_ATERM_BYTES: usize = MAX_ATERM_DERIVATION_MEBIBYTES * MEBIBYTE_BYTES + 1;

struct FixtureCase {
    graph: &'static str,
    index: &'static str,
    validate_snapshot: &'static str,
    plan_snapshot: &'static str,
    root_node: &'static str,
}

struct AtermDrvProduceCase<'a> {
    source_prefix: &'a str,
    root_derivation: &'a str,
    source_derivation: &'a str,
    root_fixture: &'a str,
    source_fixture: &'a str,
    producer_kind: &'a str,
    producer_identity: &'a str,
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
        assert!(!validate["receipt"]["raw_graph_digest"].as_str().unwrap().is_empty());

        let plan = run_plan_json(fixture_path(case.graph), fixture_path(case.index), fixture_path(POLICY));
        let expected_plan = fixture_json(case.plan_snapshot);
        assert_eq!(plan, expected_plan);
        assert_eq!(plan["accepted"], true);
        assert_eq!(plan["plan"]["schema"], EXECUTABLE_PLAN_SCHEMA);
        assert_eq!(plan["plan"]["plan_identity"]["algorithm"], BLAKE3_ALGORITHM);
        assert_eq!(plan["plan"]["roots"][0]["node_id"], case.root_node);
        assert!(!plan["plan"]["native_units"].as_array().unwrap().is_empty());
        assert!(!plan["plan"]["exact_path_maps"]["derivations"].as_object().unwrap().is_empty());
        assert!(!plan["plan"]["source_requirements"].as_array().unwrap().is_empty());
        assert!(plan["plan"]["forbidden_process_invocations"].as_array().unwrap().is_empty());
        assert!(
            plan["plan"]["non_claims"]
                .as_array()
                .unwrap()
                .contains(&Value::String("not-output-trust".to_string()))
        );
        assert!(
            plan["plan"]["non_claims"]
                .as_array()
                .unwrap()
                .contains(&Value::String("not-realization".to_string()))
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

#[test]
fn foreign_import_cli_produces_nixpkgs_artifacts_then_validates_and_plans_without_nix() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    let out_dir = temp.path().join("artifacts");
    fs::create_dir(&fake_path).expect("fake PATH should be created");

    let produce = produce_nixpkgs_json_artifacts(&fake_path, &out_dir);
    let produce_report: Value = serde_json::from_slice(&produce.stdout).expect("stdout should be JSON");
    let graph = out_dir.join("nixpkgs.graph.json");
    let index = out_dir.join("nixpkgs.index.json");

    assert_eq!(produce_report["accepted"], true);
    assert!(graph.exists());
    assert!(index.exists());
    assert!(produce.stderr.is_empty());

    let validate = run_validate_json(graph.clone(), index.clone(), fixture_path(NIXPKGS_POLICY));
    let plan = run_plan_json(graph, index, fixture_path(NIXPKGS_POLICY));

    assert_eq!(validate["accepted"], true);
    assert!(validate["receipt"]["hash_domains"].as_array().unwrap().len() > 1);
    assert_eq!(plan["accepted"], true);
    assert_eq!(plan["plan"]["substitution_audit"][0]["cache_url"], CACHE_NIXOS_ORG);
    assert_eq!(plan["plan"]["substitution_audit"][0]["store_admission_required"], true);
    assert!(plan["plan"]["forbidden_process_invocations"].as_array().unwrap().is_empty());
}

#[test]
fn foreign_import_cli_produces_nixpkgs_artifacts_from_drv_files_without_nix() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    let out_dir = temp.path().join("drv-artifacts");
    fs::create_dir(&fake_path).expect("fake PATH should be created");

    let produce = produce_nixpkgs_drv_artifacts(&fake_path, &out_dir);
    let produce_report: Value = serde_json::from_slice(&produce.stdout).expect("stdout should be JSON");
    let graph = out_dir.join("nixpkgs.graph.json");
    let index = out_dir.join("nixpkgs.index.json");

    assert_eq!(produce_report["accepted"], true);
    assert!(graph.exists());
    assert!(index.exists());
    assert!(produce.stderr.is_empty());

    let validate = run_validate_json(graph.clone(), index.clone(), fixture_path(NIXPKGS_POLICY));
    let plan = run_plan_json(graph, index, fixture_path(NIXPKGS_POLICY));

    assert_eq!(validate["accepted"], true);
    assert!(validate["receipt"]["hash_domains"].as_array().unwrap().len() > 1);
    assert_eq!(plan["accepted"], true);
    assert_eq!(plan["plan"]["substitution_audit"][0]["cache_url"], CACHE_NIXOS_ORG);
    assert!(plan["plan"]["forbidden_process_invocations"].as_array().unwrap().is_empty());
}

#[test]
fn foreign_import_cli_produces_nixpkgs_artifacts_from_drv_dir_without_nix() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    let drv_dir = temp.path().join("drv-dir");
    let out_dir = temp.path().join("drv-dir-artifacts");
    fs::create_dir(&fake_path).expect("fake PATH should be created");
    write_drv_dir_fixture(&drv_dir, true);

    let produce = produce_nixpkgs_drv_dir_artifacts(&fake_path, &drv_dir, &out_dir);
    let produce_report: Value = serde_json::from_slice(&produce.stdout).expect("stdout should be JSON");
    let graph = out_dir.join("nixpkgs.graph.json");
    let index = out_dir.join("nixpkgs.index.json");

    assert_eq!(produce_report["accepted"], true);
    assert!(graph.exists());
    assert!(index.exists());
    assert!(produce.stderr.is_empty());

    let graph_json = json_file(&graph);
    assert_eq!(graph_json["nodes"].as_array().unwrap().len(), NIXPKGS_REACHABLE_DRV_COUNT);
    assert!(
        !graph_json["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| { node["original_derivation"] == Value::String(NIXPKGS_UNRELATED_DRV.to_string()) })
    );

    let validate = run_validate_json(graph.clone(), index.clone(), fixture_path(NIXPKGS_POLICY));
    let plan = run_plan_json(graph, index, fixture_path(NIXPKGS_POLICY));

    assert_eq!(validate["accepted"], true);
    assert_eq!(plan["accepted"], true);
    assert_eq!(plan["plan"]["substitution_audit"][0]["cache_url"], CACHE_NIXOS_ORG);
    assert!(plan["plan"]["forbidden_process_invocations"].as_array().unwrap().is_empty());
}

#[test]
fn foreign_import_cli_rejects_drv_dir_missing_reachable_input_without_partial_artifacts() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    let drv_dir = temp.path().join("missing-input-drv-dir");
    let out_dir = temp.path().join("missing-input-artifacts");
    fs::create_dir(&fake_path).expect("fake PATH should be created");
    write_drv_dir_fixture(&drv_dir, false);

    let output = mantle_cmd()
        .env("PATH", &fake_path)
        .args([
            "--json",
            "foreign-import",
            "produce-nix",
            "--drv-dir",
            path_str(&drv_dir),
            "--root-derivation",
            NIXPKGS_HELLO_DRV,
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
            "--producer-identity",
            "nixpkgs:missing-input-drv-dir-fixture",
            "--out-dir",
            path_str(&out_dir),
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    assert_rejected_class(output.stdout, "missing-nix-input-derivation");
    assert!(output.stderr.is_empty());
    assert!(!out_dir.join("nixpkgs.graph.json").exists());
    assert!(!out_dir.join("nixpkgs.index.json").exists());
}

#[test]
fn foreign_import_cli_rejects_malformed_drv_without_partial_artifacts() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    let out_dir = temp.path().join("malformed-artifacts");
    let malformed_drv = temp.path().join("malformed.drv");
    fs::create_dir(&fake_path).expect("fake PATH should be created");
    fs::write(&malformed_drv, "not a derivation").expect("malformed drv should be written");

    let drv_arg = format!("{NIXPKGS_HELLO_DRV}={}", path_str(&malformed_drv));
    let output = mantle_cmd()
        .env("PATH", &fake_path)
        .args([
            "--json",
            "foreign-import",
            "produce-nix",
            "--drv",
            &drv_arg,
            "--root-derivation",
            NIXPKGS_HELLO_DRV,
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
            "--producer-identity",
            "nixpkgs:malformed-drv-fixture",
            "--out-dir",
            path_str(&out_dir),
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    assert_rejected_class(output.stdout, MALFORMED_NIX_DERIVATION);
    assert!(output.stderr.is_empty());
    assert!(!out_dir.join("nixpkgs.graph.json").exists());
    assert!(!out_dir.join("nixpkgs.index.json").exists());
}

#[test]
fn foreign_import_cli_produce_aterm_matches_nix_and_emits_guix_directory_artifacts() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    let generic_nix_out = temp.path().join("generic-nix-artifacts");
    let compatible_nix_out = temp.path().join("compatible-nix-artifacts");
    let guix_drv_dir = temp.path().join("guix-drv-dir");
    let guix_out = temp.path().join("guix-artifacts");
    fs::create_dir(&fake_path).expect("fake PATH should be created");

    let generic_nix = produce_aterm_drv_artifacts(&fake_path, &generic_nix_out, AtermDrvProduceCase {
        source_prefix: NIX_SOURCE_PREFIX,
        root_derivation: NIXPKGS_HELLO_DRV,
        source_derivation: NIXPKGS_SOURCE_DRV,
        root_fixture: NIXPKGS_ROOT_DRV_FILE,
        source_fixture: NIXPKGS_SOURCE_DRV_FILE,
        producer_kind: "nixpkgs",
        producer_identity: "nixpkgs:hello-drv-fixture",
    });
    let compatible_nix = produce_nixpkgs_drv_artifacts(&fake_path, &compatible_nix_out);
    assert!(generic_nix.stderr.is_empty());
    assert!(compatible_nix.stderr.is_empty());
    assert_eq!(
        json_file(&generic_nix_out.join(ATERM_GRAPH_FILE)),
        json_file(&compatible_nix_out.join("nixpkgs.graph.json"))
    );
    assert_eq!(
        json_file(&generic_nix_out.join(ATERM_INDEX_FILE)),
        json_file(&compatible_nix_out.join("nixpkgs.index.json"))
    );

    write_guix_drv_dir_fixture(&guix_drv_dir, true);
    let guix = produce_aterm_drv_dir_artifacts(&fake_path, &guix_drv_dir, &guix_out);
    let guix_report: Value = serde_json::from_slice(&guix.stdout).expect("stdout should be JSON");
    let guix_graph = json_file(&guix_out.join(ATERM_GRAPH_FILE));

    assert_eq!(guix_report["accepted"], true);
    assert!(guix.stderr.is_empty());
    assert!(guix_out.join(ATERM_INDEX_FILE).exists());
    assert_eq!(guix_graph["producer"]["kind"], "guix");
    assert_eq!(guix_graph["source_store_prefixes"][0], GUIX_SOURCE_PREFIX);
    assert_eq!(guix_graph["nodes"].as_array().unwrap().len(), NIXPKGS_REACHABLE_DRV_COUNT);
    assert!(
        guix_graph["nodes"].as_array().unwrap().iter().all(|node| {
            node["original_derivation"].as_str().is_some_and(|path| path.starts_with(GUIX_SOURCE_PREFIX))
        })
    );
}

#[test]
fn foreign_import_cli_produce_aterm_rejects_bad_bundles_without_partial_artifacts() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    fs::create_dir(&fake_path).expect("fake PATH should be created");

    let malformed_file = temp.path().join("malformed.drv");
    fs::write(&malformed_file, "not an ATerm derivation").expect("malformed fixture should be written");
    assert_produce_aterm_rejects(
        &fake_path,
        &temp.path().join("malformed-out"),
        NIX_SOURCE_PREFIX,
        NIXPKGS_SOURCE_DRV,
        vec![format!("{NIXPKGS_SOURCE_DRV}={}", path_str(&malformed_file))],
        "malformed-foreign-aterm",
    );

    assert_produce_aterm_rejects(
        &fake_path,
        &temp.path().join("mixed-out"),
        GUIX_SOURCE_PREFIX,
        GUIXPKGS_SOURCE_DRV,
        vec![format!(
            "{GUIXPKGS_SOURCE_DRV}={}",
            path_str(&fixture_path(NIXPKGS_SOURCE_DRV_FILE))
        )],
        "mixed-foreign-store-prefix",
    );

    assert_produce_aterm_rejects(
        &fake_path,
        &temp.path().join("missing-out"),
        GUIX_SOURCE_PREFIX,
        GUIXPKGS_HELLO_DRV,
        vec![format!(
            "{GUIXPKGS_HELLO_DRV}={}",
            path_str(&fixture_path(GUIXPKGS_ROOT_DRV_FILE))
        )],
        "missing-foreign-input-derivation",
    );
}

#[test]
fn foreign_import_cli_produce_aterm_rejects_duplicate_and_bounded_byte_inputs() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    fs::create_dir(&fake_path).expect("fake PATH should be created");

    let duplicate_spec = format!("{NIXPKGS_SOURCE_DRV}={}", path_str(&fixture_path(NIXPKGS_SOURCE_DRV_FILE)));
    assert_produce_aterm_rejects(
        &fake_path,
        &temp.path().join("duplicate-out"),
        NIX_SOURCE_PREFIX,
        NIXPKGS_SOURCE_DRV,
        vec![duplicate_spec.clone(), duplicate_spec],
        "duplicate-foreign-derivation-path",
    );

    let non_utf8_file = temp.path().join("non-utf8.drv");
    fs::write(&non_utf8_file, [u8::MAX]).expect("non-UTF-8 fixture should be written");
    assert_produce_aterm_rejects(
        &fake_path,
        &temp.path().join("non-utf8-out"),
        NIX_SOURCE_PREFIX,
        NIXPKGS_SOURCE_DRV,
        vec![format!("{NIXPKGS_SOURCE_DRV}={}", path_str(&non_utf8_file))],
        "non-utf8-foreign-aterm",
    );

    let oversized_file = temp.path().join("oversized.drv");
    fs::write(&oversized_file, vec![b'x'; OVERSIZED_ATERM_BYTES]).expect("oversized fixture should be written");
    assert_produce_aterm_rejects(
        &fake_path,
        &temp.path().join("oversized-out"),
        NIX_SOURCE_PREFIX,
        NIXPKGS_SOURCE_DRV,
        vec![format!("{NIXPKGS_SOURCE_DRV}={}", path_str(&oversized_file))],
        "foreign-aterm-bytes-out-of-range",
    );
}

#[test]
fn foreign_import_cli_produce_aterm_rejects_input_mode_conflicts_without_artifacts() {
    let temp = TempDir::new().expect("tempdir should be created");
    let fake_path = temp.path().join(FAKE_PATH_DIR);
    let drv_dir = temp.path().join("drv-dir");
    let out_dir = temp.path().join("conflict-out");
    fs::create_dir(&fake_path).expect("fake PATH should be created");
    write_guix_drv_dir_fixture(&drv_dir, true);
    let source_spec = format!("{GUIXPKGS_SOURCE_DRV}={}", path_str(&fixture_path(GUIXPKGS_SOURCE_DRV_FILE)));

    let output = mantle_cmd()
        .env("PATH", &fake_path)
        .args([
            "--json",
            "foreign-import",
            "produce-aterm",
            "--source-prefix",
            GUIX_SOURCE_PREFIX,
            "--drv",
            &source_spec,
            "--drv-dir",
            path_str(&drv_dir),
            "--root-derivation",
            GUIXPKGS_HELLO_DRV,
            "--producer-kind",
            "guix",
            "--producer-identity",
            "guix:mode-conflict",
            "--out-dir",
            path_str(&out_dir),
        ])
        .assert()
        .failure()
        .get_output()
        .clone();

    assert_rejected_class(output.stdout, "aterm-producer-input-mode-conflict");
    assert!(output.stderr.is_empty());
    assert!(!out_dir.join(ATERM_GRAPH_FILE).exists());
    assert!(!out_dir.join(ATERM_INDEX_FILE).exists());
}

fn produce_nixpkgs_json_artifacts(fake_path: &Path, out_dir: &Path) -> std::process::Output {
    mantle_cmd()
        .env("PATH", fake_path)
        .args([
            "--json",
            "foreign-import",
            "produce-nix",
            "--derivation-json",
            path_str(&fixture_path(NIXPKGS_DERIVATION_JSON)),
            "--root-derivation",
            NIXPKGS_HELLO_DRV,
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
            "--producer-identity",
            "nixpkgs:hello-fixture",
            "--producer-revision",
            "fixture-revision",
            "--cache-url",
            CACHE_NIXOS_ORG,
            "--cache-trust-scope",
            TRUSTED_CACHE_SCOPE,
            "--out-dir",
            path_str(out_dir),
        ])
        .assert()
        .success()
        .get_output()
        .clone()
}

fn produce_nixpkgs_drv_artifacts(fake_path: &Path, out_dir: &Path) -> std::process::Output {
    let root_drv = format!("{NIXPKGS_HELLO_DRV}={}", path_str(&fixture_path(NIXPKGS_ROOT_DRV_FILE)));
    let source_drv = format!("{NIXPKGS_SOURCE_DRV}={}", path_str(&fixture_path(NIXPKGS_SOURCE_DRV_FILE)));
    mantle_cmd()
        .env("PATH", fake_path)
        .args([
            "--json",
            "foreign-import",
            "produce-nix",
            "--drv",
            &root_drv,
            "--drv",
            &source_drv,
            "--root-derivation",
            NIXPKGS_HELLO_DRV,
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
            "--producer-identity",
            "nixpkgs:hello-drv-fixture",
            "--producer-revision",
            "fixture-revision",
            "--cache-url",
            CACHE_NIXOS_ORG,
            "--cache-trust-scope",
            TRUSTED_CACHE_SCOPE,
            "--out-dir",
            path_str(out_dir),
        ])
        .assert()
        .success()
        .get_output()
        .clone()
}

fn produce_nixpkgs_drv_dir_artifacts(fake_path: &Path, drv_dir: &Path, out_dir: &Path) -> std::process::Output {
    mantle_cmd()
        .env("PATH", fake_path)
        .args([
            "--json",
            "foreign-import",
            "produce-nix",
            "--drv-dir",
            path_str(drv_dir),
            "--root-derivation",
            NIXPKGS_HELLO_DRV,
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
            "--producer-identity",
            "nixpkgs:hello-drv-dir-fixture",
            "--producer-revision",
            "fixture-revision",
            "--cache-url",
            CACHE_NIXOS_ORG,
            "--cache-trust-scope",
            TRUSTED_CACHE_SCOPE,
            "--out-dir",
            path_str(out_dir),
        ])
        .assert()
        .success()
        .get_output()
        .clone()
}

fn produce_aterm_drv_artifacts(
    fake_path: &Path,
    out_dir: &Path,
    case: AtermDrvProduceCase<'_>,
) -> std::process::Output {
    let root_drv = format!("{}={}", case.root_derivation, path_str(&fixture_path(case.root_fixture)));
    let source_drv = format!("{}={}", case.source_derivation, path_str(&fixture_path(case.source_fixture)));
    mantle_cmd()
        .env("PATH", fake_path)
        .args([
            "--json",
            "foreign-import",
            "produce-aterm",
            "--source-prefix",
            case.source_prefix,
            "--drv",
            &root_drv,
            "--drv",
            &source_drv,
            "--root-derivation",
            case.root_derivation,
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
            "--producer-kind",
            case.producer_kind,
            "--producer-identity",
            case.producer_identity,
            "--producer-revision",
            "fixture-revision",
            "--cache-url",
            CACHE_NIXOS_ORG,
            "--cache-trust-scope",
            TRUSTED_CACHE_SCOPE,
            "--out-dir",
            path_str(out_dir),
        ])
        .assert()
        .success()
        .get_output()
        .clone()
}

fn produce_aterm_drv_dir_artifacts(fake_path: &Path, drv_dir: &Path, out_dir: &Path) -> std::process::Output {
    mantle_cmd()
        .env("PATH", fake_path)
        .args([
            "--json",
            "foreign-import",
            "produce-aterm",
            "--source-prefix",
            GUIX_SOURCE_PREFIX,
            "--drv-dir",
            path_str(drv_dir),
            "--root-derivation",
            GUIXPKGS_HELLO_DRV,
            "--package",
            HELLO_PACKAGE,
            "--system",
            HELLO_SYSTEM,
            "--producer-kind",
            "guix",
            "--producer-identity",
            "guix:hello-drv-dir-fixture",
            "--producer-revision",
            "fixture-revision",
            "--out-dir",
            path_str(out_dir),
        ])
        .assert()
        .success()
        .get_output()
        .clone()
}

fn assert_produce_aterm_rejects(
    fake_path: &Path,
    out_dir: &Path,
    source_prefix: &str,
    root_derivation: &str,
    drv_specs: Vec<String>,
    expected_class: &str,
) {
    let mut command = mantle_cmd();
    command.env("PATH", fake_path).args([
        "--json",
        "foreign-import",
        "produce-aterm",
        "--source-prefix",
        source_prefix,
        "--root-derivation",
        root_derivation,
        "--package",
        HELLO_PACKAGE,
        "--system",
        HELLO_SYSTEM,
        "--producer-kind",
        "fixture",
        "--producer-identity",
        "fixture:negative",
        "--out-dir",
        path_str(out_dir),
    ]);
    for drv_spec in drv_specs {
        command.args(["--drv", &drv_spec]);
    }
    let output = command.assert().failure().get_output().clone();
    assert_rejected_class(output.stdout, expected_class);
    assert!(output.stderr.is_empty());
    assert!(!out_dir.join(ATERM_GRAPH_FILE).exists());
    assert!(!out_dir.join(ATERM_INDEX_FILE).exists());
}

fn write_guix_drv_dir_fixture(drv_dir: &Path, include_source: bool) {
    fs::create_dir(drv_dir).expect("drv dir should be created");
    copy_drv_fixture(drv_dir, GUIXPKGS_HELLO_DRV, GUIXPKGS_ROOT_DRV_FILE);
    if include_source {
        copy_drv_fixture(drv_dir, GUIXPKGS_SOURCE_DRV, GUIXPKGS_SOURCE_DRV_FILE);
    }
}

fn write_drv_dir_fixture(drv_dir: &Path, include_source: bool) {
    fs::create_dir(drv_dir).expect("drv dir should be created");
    copy_drv_fixture(drv_dir, NIXPKGS_HELLO_DRV, NIXPKGS_ROOT_DRV_FILE);
    copy_drv_fixture(drv_dir, NIXPKGS_UNRELATED_DRV, NIXPKGS_SOURCE_DRV_FILE);
    if include_source {
        copy_drv_fixture(drv_dir, NIXPKGS_SOURCE_DRV, NIXPKGS_SOURCE_DRV_FILE);
    }
}

fn copy_drv_fixture(drv_dir: &Path, logical_path: &str, fixture_name: &str) {
    let basename = logical_path.rsplit('/').next().expect("logical drv path should have basename");
    fs::copy(fixture_path(fixture_name), drv_dir.join(basename)).expect("drv fixture should copy");
}

fn json_file(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("JSON file should be readable");
    serde_json::from_str(&contents).expect("JSON file should parse")
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
