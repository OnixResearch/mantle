//! Consumer-visible boundary tests for `cargo-build-bundle`.
//!
//! Each action plans a Rust operation, runs it through the bundle port and classifies the actual
//! evidence before it prints a report. `inspect` is the one action that needs no signed archive, so
//! its success path is exercised with a canonical manifest written to disk; the other actions are
//! driven to their refusals. The tests pin stdout, exit class and the absence of a success report;
//! they do not pin diagnostic wording.
//! r[application_architecture.effect_observation_boundary]
//! r[application_architecture.typed_error_ownership]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use assert_cmd::Command;
use mantle_cargo_build_bundle_core::Artifact;
use mantle_cargo_build_bundle_core::Compatibility;
use mantle_cargo_build_bundle_core::Manifest;
use mantle_cargo_build_bundle_core::NON_CLAIM;
use mantle_cargo_build_bundle_core::Provenance;
use mantle_cargo_build_bundle_core::SCHEMA;
use mantle_cargo_build_bundle_core::Unit;
use mantle_cargo_build_bundle_core::canonicalize;
use mantle_cargo_build_bundle_core::encode;
use serde_json::Value;
use tempfile::TempDir;

const MANIFEST_FILE: &str = "cargo-build-bundle.json";
const EXIT_REFUSED: i32 = 3;
const EXIT_USAGE: i32 = 2;
/// A syntactically valid Nix narinfo verifying key (name and 32 key bytes); it signs nothing here.
const TRUSTED_KEY: &str = "cache.example-1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

fn store_root(name: &str) -> String {
    format!("/nix/store/00000000000000000000000000000000-{name}")
}

fn artifact(name: &str) -> Artifact {
    Artifact {
        file: format!("{name}.json"),
        bytes: 1,
        blake3: "a".repeat(64),
    }
}

fn manifest() -> Manifest {
    let tools: BTreeMap<String, String> = ["rustc", "cargo", "toolchain", "producer", "planner", "helper", "linker"]
        .into_iter()
        .map(|role| (role.to_string(), store_root(role)))
        .collect();
    let producer_outputs: BTreeMap<String, String> =
        ["out", "plan", "sources"].into_iter().map(|role| (role.to_string(), store_root(role))).collect();
    let required_store_paths = tools.values().chain(producer_outputs.values()).cloned().collect();
    let evidence: BTreeMap<String, Artifact> = [
        "cargo-metadata",
        "cargo-unit-graph",
        "rust-plan-receipt",
        "cargo-unit-plan-evidence",
        "cargo-unit-bindings",
    ]
    .into_iter()
    .map(|key| (key.to_string(), artifact(key)))
    .collect();
    let mut manifest = Manifest {
        schema: SCHEMA.into(),
        version: 1,
        compatibility: Compatibility {
            system: "x86_64-linux".into(),
            host_triple: "x86_64-unknown-linux-gnu".into(),
            target_triple: "x86_64-unknown-linux-gnu".into(),
            store_prefix: "/nix/store".into(),
            sandbox: "native".into(),
            network: "denied".into(),
            policy_blake3: "b".repeat(64),
            trust_policy_blake3: "c".repeat(64),
            plan_schema: "mantle-plan-v2".into(),
            source_schema: "mantle-source-bundle-v1".into(),
            receipt_schema: "mantle-build-receipt-bundle-v1".into(),
            max_plan_bytes: 4_194_304,
            max_units: 4096,
            max_sources: 256,
            max_direct_inputs: 256,
        },
        profile: "release".into(),
        selected_features: vec!["fmt".into()],
        declared_env: BTreeMap::new(),
        tools,
        required_store_paths,
        no_default_features: false,
        producer_outputs,
        source_manifest_blake3: "d".repeat(64),
        receipt_bundle_blake3: "e".repeat(64),
        lock_blake3: "f".repeat(64),
        canonical_plan_blake3: "0".repeat(64),
        package_sources: [
            ("engine".into(), "source-engine".into()),
            ("app".into(), "source-app".into()),
        ]
        .into(),
        registry_git_sources: BTreeMap::new(),
        vendor_store_path: None,
        units: [
            ("engine.lib".into(), Unit {
                package_id: Some("engine".into()),
                dependencies: vec![],
                features: vec![],
            }),
            ("app.bin".into(), Unit {
                package_id: Some("app".into()),
                dependencies: vec!["engine.lib".into()],
                features: vec![],
            }),
        ]
        .into(),
        roots: vec!["app.bin".into()],
        native_inputs: BTreeMap::new(),
        provenance: Provenance {
            cargo_unit_graph_version: 1,
            planner_receipt_schema_version: 1,
            producer_revision: "1".repeat(40),
            oracle_inputs: evidence.iter().map(|(key, item)| (key.clone(), item.blake3.clone())).collect(),
            evidence,
        },
        artifacts: ["source", "receipts", "plan", "goals", "store", "lock", "config"]
            .into_iter()
            .map(|key| (key.into(), artifact(key)))
            .collect(),
        bundle_blake3: String::new(),
        non_claim: NON_CLAIM.into(),
    };
    manifest.artifacts.get_mut("source").unwrap().file = "sources.bundle.json".into();
    canonicalize(manifest).unwrap()
}

struct Bundle {
    root: TempDir,
    manifest: Manifest,
}

impl Bundle {
    fn new() -> Self {
        let root = TempDir::new().unwrap();
        let manifest = manifest();
        fs::create_dir(root.path().join("bundle")).unwrap();
        fs::write(root.path().join("bundle").join(MANIFEST_FILE), encode(&manifest).unwrap()).unwrap();
        fs::write(root.path().join("compatibility.json"), serde_json::to_vec(&manifest.compatibility).unwrap())
            .unwrap();
        Self { root, manifest }
    }

    fn dir(&self) -> std::path::PathBuf {
        self.root.path().join("bundle")
    }

    fn compatibility(&self) -> std::path::PathBuf {
        self.root.path().join("compatibility.json")
    }

    fn mantle(&self, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("mantle")
            .unwrap()
            .arg("--state-dir")
            .arg(self.root.path().join("state"))
            .arg("cargo-build-bundle")
            .args(args)
            .output()
            .unwrap()
    }

    fn inspect(&self, from: &Path, compatibility: &Path) -> std::process::Output {
        self.mantle(&[
            "inspect",
            "--from",
            from.to_str().unwrap(),
            "--compatibility",
            compatibility.to_str().unwrap(),
        ])
    }
}

fn assert_refused(output: &std::process::Output, code: i32) {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stdout.is_empty(), "a refused action must not print a success report: {output:?}");
}

#[test]
fn inspect_prints_the_admitted_manifest_for_matching_compatibility() {
    let bundle = Bundle::new();
    let output = bundle.inspect(&bundle.dir(), &bundle.compatibility());
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let printed: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(printed["bundle_blake3"], bundle.manifest.bundle_blake3);
    assert_eq!(printed["profile"], "release");
    assert_eq!(printed["roots"], serde_json::json!(["app.bin"]));
    assert!(!bundle.root.path().join("state").exists(), "inspection must not create state");
}

#[test]
fn inspect_refuses_a_compatibility_expectation_the_manifest_does_not_satisfy() {
    let bundle = Bundle::new();
    let mut other = bundle.manifest.compatibility.clone();
    other.store_prefix = "/different/store".into();
    let path = bundle.root.path().join("other-compatibility.json");
    fs::write(&path, serde_json::to_vec(&other).unwrap()).unwrap();
    assert_refused(&bundle.inspect(&bundle.dir(), &path), EXIT_REFUSED);
}

#[test]
fn inspect_refuses_a_manifest_whose_content_no_longer_matches_its_identity() {
    let bundle = Bundle::new();
    let path = bundle.dir().join(MANIFEST_FILE);
    let tampered = fs::read_to_string(&path).unwrap().replace("\"release\"", "\"debug\"");
    assert_ne!(tampered, fs::read_to_string(&path).unwrap(), "the fixture must contain the profile it tampers with");
    fs::write(&path, tampered).unwrap();
    assert_refused(&bundle.inspect(&bundle.dir(), &bundle.compatibility()), EXIT_REFUSED);
}

#[test]
fn inspect_refuses_missing_and_malformed_inputs_without_a_report() {
    let bundle = Bundle::new();
    let absent = bundle.root.path().join("absent");
    assert_refused(&bundle.inspect(&absent, &bundle.compatibility()), EXIT_REFUSED);
    assert_refused(&bundle.inspect(&bundle.dir(), &absent.join("compatibility.json")), EXIT_REFUSED);
    let malformed = bundle.root.path().join("malformed.json");
    fs::write(&malformed, b"{not json").unwrap();
    assert_refused(&bundle.inspect(&bundle.dir(), &malformed), EXIT_REFUSED);
    fs::write(bundle.dir().join(MANIFEST_FILE), b"{}").unwrap();
    assert_refused(&bundle.inspect(&bundle.dir(), &bundle.compatibility()), EXIT_REFUSED);
}

#[test]
fn inspect_without_its_required_inputs_is_a_usage_error() {
    let bundle = Bundle::new();
    for args in [
        &["inspect", "--from", "bundle"][..],
        &["inspect", "--compatibility", "c.json"][..],
        &["inspect"][..],
    ] {
        assert_refused(&bundle.mantle(args), EXIT_USAGE);
    }
}

fn verification_args<'a>(bundle: &'a Bundle, policy_hash: &'a str, compatibility: &'a str) -> Vec<String> {
    [
        "verify",
        "--from",
        bundle.dir().to_str().unwrap(),
        "--compatibility",
        compatibility,
        "--policy-hash",
        policy_hash,
        "--valid-at-unix-s",
        "1000",
        "--trusted-public-key",
        TRUSTED_KEY,
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

#[test]
fn verify_refuses_a_trust_policy_that_is_not_the_one_the_manifest_declares() {
    let bundle = Bundle::new();
    let compatibility = bundle.compatibility();
    let wrong_policy = "9".repeat(64);
    let args = verification_args(&bundle, &wrong_policy, compatibility.to_str().unwrap());
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    assert_refused(&bundle.mantle(&args), EXIT_REFUSED);
}

#[test]
fn verify_refuses_a_bundle_whose_declared_payloads_are_absent() {
    let bundle = Bundle::new();
    let compatibility = bundle.compatibility();
    let declared_policy = bundle.manifest.compatibility.trust_policy_blake3.clone();
    let args = verification_args(&bundle, &declared_policy, compatibility.to_str().unwrap());
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    // The manifest is admitted but no transport files exist: verification must stop before reporting.
    assert_refused(&bundle.mantle(&args), EXIT_REFUSED);
}

#[test]
fn verify_without_a_trusted_key_is_a_usage_error() {
    let bundle = Bundle::new();
    let output = bundle.mantle(&[
        "verify",
        "--from",
        bundle.dir().to_str().unwrap(),
        "--compatibility",
        bundle.compatibility().to_str().unwrap(),
        "--policy-hash",
        &"c".repeat(64),
        "--valid-at-unix-s",
        "1000",
    ]);
    assert_refused(&output, EXIT_USAGE);
}
