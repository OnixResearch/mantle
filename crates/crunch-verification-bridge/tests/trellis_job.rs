//! Synthetic Trellis/Verus producer-job handoff: one positive observation and
//! the fail-closed negatives the bridge must reject.
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crunch_verification_bridge::Handoff;
use crunch_verification_bridge::consume_trellis_job;
use crunch_verification_bridge::crunch_verification_bridge_core::identity;
use crunch_verification_bridge::export;
use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;

const REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PROVIDER: &str = "b3:cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625";
const RAIL: &str = "trellis.bounds.verus#rail-0";
const LIB_PATH: &str = "source/src/lib.rs";
const BOUNDS_PATH: &str = "source/src/bounds.rs";
const STDOUT_PATH: &str = "artifacts/verus.stdout";
const STDERR_PATH: &str = "artifacts/verus.stderr";
const ROOT_DRV: &str = "00000000000000000000000000000000-root.drv";
const SOURCE_STORE: &str = "11111111111111111111111111111111-source";

fn digest(bytes: &[u8]) -> String {
    identity(bytes.to_vec()).identity
}

fn write(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// A producer output and acquisition export that satisfy the design's output
/// contract, built entirely in temporary directories.
struct SyntheticJob {
    owner: TempDir,
    export: TempDir,
    profile: Value,
    receipt: Value,
    report: Value,
    job: Value,
    graph: Value,
    metadata: Value,
    lib_rs: Vec<u8>,
    bounds_rs: Vec<u8>,
    stdout_bytes: Vec<u8>,
    stderr_bytes: Vec<u8>,
    cards: Vec<u8>,
    evidence: Vec<u8>,
}

impl SyntheticJob {
    fn new() -> Self {
        let lib_rs =
            b"pub mod bounds;\n\nverus! {\n    pub fn clamp(x: u64, lo: u64, hi: u64) -> u64 { x }\n}\n".to_vec();
        let bounds_rs =
            b"verus! {\n    pub open spec fn in_range(x: u64, lo: u64, hi: u64) -> bool { lo <= x && x <= hi }\n}\n"
                .to_vec();
        let stdout_bytes = b"verus: 4 verified, 0 errors\n".to_vec();
        let stderr_bytes = b"note: bounds claim scoped to src/bounds.rs\n".to_vec();
        let cards = serde_json::to_vec(
            &json!({"schema_version": "octet-proof-cards/v1", "cards": [{"card_id": "trellis.bounds.verus#rail-0"}]}),
        )
        .unwrap();
        let evidence =
            serde_json::to_vec(&json!({"schema_version": "octet-requirement-evidence/v1", "decks": []})).unwrap();
        let stdout_row = json!({
            "locator": STDOUT_PATH,
            "blake3": digest(&stdout_bytes),
            "bytes": stdout_bytes.len(),
            "truncated": false,
        });
        let stderr_row = json!({
            "locator": STDERR_PATH,
            "blake3": digest(&stderr_bytes),
            "bytes": stderr_bytes.len(),
            "truncated": false,
        });
        let artifacts = vec![
            json!({
                "role": "verus-stdout",
                "locator": STDOUT_PATH,
                "blake3": digest(&stdout_bytes),
                "bytes": stdout_bytes.len(),
                "truncated": false,
            }),
            json!({
                "role": "verus-stderr",
                "locator": STDERR_PATH,
                "blake3": digest(&stderr_bytes),
                "bytes": stderr_bytes.len(),
                "truncated": false,
            }),
        ];
        let receipt = json!({
            "schema_version": "octet-verus-proof-receipt/v1",
            "policy_id": "trellis.bounds.verus",
            "rail_index": 0,
            "checked_scope": RAIL,
            "backend": "verus",
            "verifier_mode": "production",
            "producer_id": "terrane",
            "checker_id": "terrane-verus-checker",
            "status": "passed",
            "pass_signal": "verus: 4 verified, 0 errors",
            "proof_artifact_identity": digest(&stdout_bytes),
            "stdout_artifact": stdout_row,
            "stderr_artifact": stderr_row,
            "source_bindings": [
                {"path": "src/lib.rs", "blake3": digest(&lib_rs)},
                {"path": "src/bounds.rs", "blake3": digest(&bounds_rs)},
            ],
            "proof_bindings": [],
            "trust_boundaries": ["verus binary", "z3 solver", "rust toolchain"],
            "non_claims": ["does not prove Verus or Z3 soundness", "does not establish release eligibility"],
        });
        let profile = json!({
            "schema_version": "trellis-verus-producer-profile/v1",
            "profile_id": "trellis.bounds.verus.mantle.v1",
            "consumer": {
                "consumer_id": "trellis.bounds",
                "provider": {
                    "backend": "verus",
                    "network_allowed": false,
                    "profile_id": "trellis-verus-0.2026.10.18",
                    "profile_identity": PROVIDER,
                },
                "source": {
                    "root": ".",
                    "paths": [
                        {"path": LIB_PATH, "identity": digest(&lib_rs)},
                        {"path": BOUNDS_PATH, "identity": digest(&bounds_rs)},
                    ],
                },
            },
            "provider_tools": {
                "identity_schema": "octet-verus-provider-closure/v1",
                "verus_revision": "0.2026.10.18.7d82994",
                "z3": "4.15.3",
                "rust_toolchain": "nightly-2026-10-14",
            },
            "extraction": {
                "command_argv": ["terrane-verus", "--triggers-mode", "silent", "--rlimit", "20", "src/lib.rs"],
                "rlimit": 20,
            },
            "artifacts": artifacts,
            "verus": {"receipt_path": "verus-receipt.json", "receipt_identity": digest(&receipt_bytes(&receipt))},
            "non_claims": ["does not prove Verus or Z3 soundness", "does not establish release eligibility"],
        });
        let profile_bytes = serde_json::to_vec(&profile).unwrap();
        let report = json!({
            "schema_version": "trellis-verus-producer-report/v1",
            "job_id": "trellis.bounds.verus.mantle.v1",
            "policy_id": "trellis.bounds.verus",
            "rail_index": 0,
            "rail_id": RAIL,
            "status": "passed",
            "exit_code": 0,
            "source_revision": REVISION,
            "profile_identity": digest(&profile_bytes),
            "provider_profile_identity": PROVIDER,
            "non_claims": ["does not establish release eligibility"],
        });
        let job = json!({
            "schema_version": "trellis-mantle-producer-job/v1",
            "owner": "Trellis",
            "job_id": "trellis.bounds.verus.mantle.v1",
            "system": "x86_64-linux",
            "package_attribute": "packages.x86_64-linux.trellis-bounds-verus-job",
            "profile": {
                "path": "proof-rails/verus-bounds/generated/profile.json",
                "identity": digest(&profile_bytes),
            },
            "provider": {
                "path": "proof-rails/verus-bounds/generated/provider-profile.json",
                "identity": PROVIDER,
            },
            "source": {"root": "src", "paths": ["src/lib.rs", "src/bounds.rs"], "network_allowed": false},
            "execution": {
                "command_argv": ["terrane-verus", "--triggers-mode", "silent", "src/lib.rs"],
                "rail_id": RAIL,
                "network_allowed": false,
            },
            "limits": {"cores": 2, "timeout_ms": 600_000, "stdout_max_bytes": 1_048_576, "stderr_max_bytes": 1_048_576},
            "outputs": ["producer-report.json", "verus-receipt.json", "proof-cards.json", "evidence.json"],
            "non_claims": ["does not establish release eligibility"],
        });
        let graph = json!({
            "version": 4,
            "derivations": {
                ROOT_DRV: {"version": 4, "inputs": {"drvs": {}, "srcs": [SOURCE_STORE]}},
            },
        });
        let metadata = json!({"revision": REVISION});
        Self {
            owner: TempDir::new().unwrap(),
            export: TempDir::new().unwrap(),
            profile,
            receipt,
            report,
            job,
            graph,
            metadata,
            lib_rs,
            bounds_rs,
            stdout_bytes,
            stderr_bytes,
            cards,
            evidence,
        }
    }

    fn root(&self) -> PathBuf {
        self.owner.path().to_path_buf()
    }

    fn export_root(&self) -> PathBuf {
        self.export.path().to_path_buf()
    }

    /// Recompute the cross-document identities, then write both directories.
    fn commit(&mut self) {
        let receipt_bytes = serde_json::to_vec(&self.receipt).unwrap();
        self.profile["verus"]["receipt_identity"] = Value::String(digest(&receipt_bytes));
        let profile_bytes = serde_json::to_vec(&self.profile).unwrap();
        self.job["profile"]["identity"] = Value::String(digest(&profile_bytes));
        self.report["profile_identity"] = Value::String(digest(&profile_bytes));
        let owner = self.owner.path().to_path_buf();
        write(&owner, "profile.json", &profile_bytes);
        write(&owner, "verus-receipt.json", &receipt_bytes);
        write(&owner, "producer-report.json", &serde_json::to_vec(&self.report).unwrap());
        write(&owner, "proof-cards.json", &self.cards);
        write(&owner, "evidence.json", &self.evidence);
        write(&owner, LIB_PATH, &self.lib_rs);
        write(&owner, BOUNDS_PATH, &self.bounds_rs);
        write(&owner, STDOUT_PATH, &self.stdout_bytes);
        write(&owner, STDERR_PATH, &self.stderr_bytes);
        let export = self.export.path().to_path_buf();
        write(&export, "job.json", &serde_json::to_vec(&self.job).unwrap());
        write(&export, "tool-closure.derivation-json.json", &serde_json::to_vec(&self.graph).unwrap());
        write(&export, "root-derivation", format!("/nix/store/{ROOT_DRV}\n").as_bytes());
        write(&export, "flake-metadata.json", &serde_json::to_vec(&self.metadata).unwrap());
        write(
            &export,
            "flake.lock",
            &serde_json::to_vec(&json!({"nodes": {}, "root": "root", "version": 7})).unwrap(),
        );
    }

    fn consume(&mut self) -> Result<Handoff, String> {
        self.commit();
        consume_trellis_job(&self.root(), &self.export_root(), REVISION)
    }
}

fn receipt_bytes(receipt: &Value) -> Vec<u8> {
    serde_json::to_vec(receipt).unwrap()
}

fn rejected(job: &mut SyntheticJob, expected: &str) {
    let error = job.consume().err().expect("producer output must be rejected");
    assert!(error.contains(expected), "expected {expected:?} in {error:?}");
}

#[test]
fn synthetic_trellis_job_observation_binds_exact_bytes() {
    let mut job = SyntheticJob::new();
    let receipt_bytes = serde_json::to_vec(&job.receipt).unwrap();
    let report_bytes = serde_json::to_vec(&job.report).unwrap();
    let handoff = job.consume().unwrap();
    let observation = &handoff.observation;
    assert_eq!(observation.producer, "Trellis");
    assert_eq!(observation.proof_acceptance, "not-evaluated-by-mantle");
    assert_eq!(observation.externally_reported_outcome, "supported");
    assert_eq!(observation.packaging.source_revision, REVISION);
    assert_eq!(observation.packaging.source_identity, digest(&job.lib_rs));
    assert_eq!(observation.packaging.provider_identity, PROVIDER);
    assert_eq!(observation.receipt_identity, digest(&receipt_bytes));
    assert_eq!(observation.external_report_identity, Some(digest(&report_bytes)));
    assert_eq!(
        handoff.producer_binding.as_ref().map(|binding| digest(&binding.job)),
        observation.producer_job_identity
    );
    assert_eq!(
        handoff.producer_binding.as_ref().map(|binding| digest(&binding.graph)),
        observation.tool_graph_identity
    );
    for path in [
        LIB_PATH,
        BOUNDS_PATH,
        "verus-receipt.json",
        STDOUT_PATH,
        STDERR_PATH,
        "producer-report.json",
        "proof-cards.json",
        "evidence.json",
    ] {
        assert!(observation.checked_materials.iter().any(|checked| checked == path), "unchecked {path}");
    }
}

#[test]
fn exported_trellis_job_handoff_replays_from_its_own_output() {
    let mut job = SyntheticJob::new();
    let handoff = job.consume().unwrap();
    let scratch = TempDir::new().unwrap();
    let output = scratch.path().join("handoff");
    let exported = export(handoff, &output).unwrap();
    let replay = consume_trellis_job(&output, &output, REVISION).unwrap();
    assert_eq!(replay.observation.association_identity, exported.observation.association_identity);
    assert_eq!(replay.observation.external_report_identity, exported.observation.external_report_identity);
    assert_eq!(replay.observation.proof_acceptance, "not-evaluated-by-mantle");
    assert_eq!(replay.receipt, fs::read(job.root().join("verus-receipt.json")).unwrap());
    assert_eq!(replay.profile, fs::read(job.root().join("profile.json")).unwrap());
}

#[test]
fn mutated_profile_source_identity_fails_closed() {
    let mut job = SyntheticJob::new();
    job.profile["consumer"]["source"]["paths"][0]["identity"] = Value::String(digest(b"changed source"));
    rejected(&mut job, "source/src/lib.rs mismatch");
}

#[test]
fn failed_receipt_status_fails_closed() {
    let mut job = SyntheticJob::new();
    job.receipt["status"] = Value::String("failed".to_string());
    rejected(&mut job, "externally reported status mismatch: expected passed, observed failed");
}

#[test]
fn mutated_receipt_source_binding_fails_closed() {
    let mut job = SyntheticJob::new();
    job.receipt["source_bindings"][0]["blake3"] = Value::String(digest(b"changed binding"));
    rejected(&mut job, "source absent from producer profile");
}

#[test]
fn mutated_receipt_artifact_row_fails_closed() {
    let mut job = SyntheticJob::new();
    job.receipt["stdout_artifact"]["blake3"] = Value::String(digest(b"changed artifact"));
    rejected(&mut job, "receipt artifacts differ from producer profile");
}

#[test]
fn unsupported_trellis_job_profile_fails_closed() {
    let mut job = SyntheticJob::new();
    job.profile["schema_version"] = Value::String("trellis-aeneas-cross-verifier-profile/v1".to_string());
    rejected(&mut job, "unsupported Trellis producer-job profile schema");
}

#[test]
fn unsupported_verus_receipt_fails_closed() {
    let mut job = SyntheticJob::new();
    job.receipt["schema_version"] = Value::String("trellis-aeneas-proof-receipt/v1".to_string());
    rejected(&mut job, "unsupported Verus proof receipt schema");
}

#[test]
fn unsupported_producer_report_fails_closed() {
    let mut job = SyntheticJob::new();
    job.report["schema_version"] = Value::String("trellis-aeneas-proof-receipt/v1".to_string());
    rejected(&mut job, "unsupported Trellis producer report schema");
}

#[test]
fn wrong_job_schema_fails_closed() {
    let mut job = SyntheticJob::new();
    job.job["schema_version"] = Value::String("octet-mantle-producer-job/v1".to_string());
    rejected(&mut job, "unsupported producer job schema");
}

#[test]
fn wrong_job_owner_fails_closed() {
    let mut job = SyntheticJob::new();
    job.job["owner"] = Value::String("Octet".to_string());
    rejected(&mut job, "unsupported producer job owner");
}

#[test]
fn wrong_rail_fails_closed() {
    let mut job = SyntheticJob::new();
    job.job["execution"]["rail_id"] = Value::String("trellis.bounds.verus#rail-1".to_string());
    rejected(&mut job, "job rail mismatch");
}

#[test]
fn network_allowed_execution_fails_closed() {
    let mut job = SyntheticJob::new();
    job.job["execution"]["network_allowed"] = Value::Bool(true);
    rejected(&mut job, "selected producer job does not declare denied-network execution");
}

#[test]
fn graph_missing_root_fails_closed() {
    let mut job = SyntheticJob::new();
    job.graph = json!({"version": 4, "derivations": {}});
    rejected(&mut job, "tool graph root absent or derivation bound exceeded");
}

#[test]
fn incomplete_tool_graph_fails_closed() {
    let mut job = SyntheticJob::new();
    let dependency = "22222222222222222222222222222222-dependency.drv";
    job.graph["derivations"][ROOT_DRV]["inputs"]["drvs"] =
        json!({dependency: {"outputs": ["out"], "dynamicOutputs": {}}});
    rejected(&mut job, "producer tool dependency closure is incomplete");
}

#[test]
fn revision_mismatch_fails_closed() {
    let mut job = SyntheticJob::new();
    job.metadata["revision"] = Value::String("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string());
    rejected(&mut job, "producer source revision mismatch");
}

#[test]
fn tampered_artifact_bytes_fail_closed() {
    let mut job = SyntheticJob::new();
    job.commit();
    write(&job.root(), STDOUT_PATH, b"verus: 0 verified, 1 error\n");
    let error = consume_trellis_job(&job.root(), &job.export_root(), REVISION)
        .err()
        .expect("producer output must be rejected");
    assert!(error.contains(STDOUT_PATH), "unexpected error {error:?}");
}

#[test]
fn missing_producer_report_fails_closed() {
    let mut job = SyntheticJob::new();
    job.commit();
    fs::remove_file(job.root().join("producer-report.json")).unwrap();
    let error = consume_trellis_job(&job.root(), &job.export_root(), REVISION)
        .err()
        .expect("producer output must be rejected");
    assert!(error.contains("producer-report.json"), "unexpected error {error:?}");
}

#[test]
fn missing_evidence_document_fails_closed() {
    let mut job = SyntheticJob::new();
    job.commit();
    fs::remove_file(job.root().join("evidence.json")).unwrap();
    let error = consume_trellis_job(&job.root(), &job.export_root(), REVISION)
        .err()
        .expect("producer output must be rejected");
    assert!(error.contains("evidence.json"), "unexpected error {error:?}");
}
