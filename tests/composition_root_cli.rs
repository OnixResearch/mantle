use std::path::Path;

use assert_cmd::Command;
use crunch_composition_core::CastoreRootRef;
use crunch_composition_core::CompositionBinding;
use crunch_composition_core::CompositionPlan;
use crunch_composition_core::CompositionRequest;
use crunch_composition_core::MERGE_POLICY_VERSION;
use crunch_composition_core::PLAN_SCHEMA;
use crunch_composition_core::POLICY_SCHEMA;
use crunch_composition_core::RealizationPolicy;
use crunch_composition_core::RealizationReceipt;
use crunch_store::StoreConfig;
use crunch_store::StoreFallbackMode;
use crunch_store::StoreHandle;
use data_encoding::HEXLOWER;
use predicates::prelude::*;
use snix_castore::Directory;
use snix_castore::Node;
use snix_castore::PathComponent;
use tokio::io::AsyncWriteExt;

const TEST_STORE_PREFIX: &str = "/mantle/store";
const TEST_LIMIT: u32 = 64;
const TEST_BYTES_LIMIT: u64 = 1_024;
const FIXTURE_A: &str = "fixtures/composition-roots/equivalent-a.json";
const FIXTURE_B: &str = "fixtures/composition-roots/equivalent-b.json";
const NEGATIVE_FRONTEND_FIXTURE: &str = "fixtures/composition-roots/negative-frontend-intent.json";

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn run_async<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Runtime::new().unwrap().block_on(future)
}

async fn open_store(output_dir: &Path, state_dir: &Path) -> StoreHandle {
    StoreHandle::open(StoreConfig {
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        base_state_dirs: Vec::new(),
        fallback_mode: StoreFallbackMode::Strict,
        store_dir: TEST_STORE_PREFIX.to_string(),
    })
    .await
    .unwrap()
}

async fn seed_root(store: &StoreHandle, name: &str, content: &[u8]) -> CastoreRootRef {
    let mut writer = store.blob_service().open_write().await;
    writer.write_all(content).await.unwrap();
    let blob_digest = writer.close().await.unwrap();
    let directory = Directory::try_from_iter([(PathComponent::try_from(name).unwrap(), Node::File {
        digest: blob_digest,
        size: u64::try_from(content.len()).unwrap(),
        executable: false,
    })])
    .unwrap();
    let size = directory.size();
    let digest = store.directory_service().put(directory).await.unwrap();
    CastoreRootRef {
        digest_blake3: HEXLOWER.encode(digest.as_slice()),
        size,
    }
}

fn request(first: CastoreRootRef, second: CastoreRootRef) -> CompositionRequest {
    CompositionRequest {
        plan: CompositionPlan {
            schema: PLAN_SCHEMA.to_string(),
            merge_policy_version: MERGE_POLICY_VERSION,
            bindings: vec![
                CompositionBinding {
                    root: first,
                    mount: String::new(),
                    label: Some("/nix/store/frontend-label".to_string()),
                },
                CompositionBinding {
                    root: second,
                    mount: "usr".to_string(),
                    label: Some("/mantle/store/frontend-label".to_string()),
                },
            ],
            collision_decisions: Vec::new(),
        },
        realization_policy: RealizationPolicy {
            schema: POLICY_SCHEMA.to_string(),
            max_bindings: TEST_LIMIT,
            max_collision_decisions: TEST_LIMIT,
            max_entries: TEST_LIMIT,
            max_depth: TEST_LIMIT,
            max_path_bytes: TEST_LIMIT,
            max_file_bytes: TEST_BYTES_LIMIT,
            max_total_file_bytes: TEST_BYTES_LIMIT,
            allow_symlinks: true,
        },
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn command_base(command: &mut Command, output_dir: &Path, state_dir: &Path) {
    command.args([
        "--json",
        "--state-dir",
        state_dir.to_str().unwrap(),
        "--store",
        output_dir.to_str().unwrap(),
        "--store-prefix",
        TEST_STORE_PREFIX,
        "store",
        "composition",
    ]);
}

#[test]
fn composition_plan_fixtures_are_order_independent_and_frontend_neutral() {
    let temp = tempfile::tempdir().unwrap();
    let output_dir = temp.path().join("store");
    let state_dir = temp.path().join("state");
    let mut first = mantle_cmd();
    command_base(&mut first, &output_dir, &state_dir);
    let first_output = first.args(["plan", "--from", FIXTURE_A]).assert().success().get_output().stdout.clone();
    let first_plan: crunch_composition_core::PreparedComposition = serde_json::from_slice(&first_output).unwrap();

    let mut second = mantle_cmd();
    command_base(&mut second, &output_dir, &state_dir);
    let second_output = second.args(["plan", "--from", FIXTURE_B]).assert().success().get_output().stdout.clone();
    let second_plan: crunch_composition_core::PreparedComposition = serde_json::from_slice(&second_output).unwrap();
    assert_eq!(first_plan, second_plan);
    assert!(first_output.windows(b"nix/store".len()).all(|window| window != b"nix/store"));
    assert!(second_output.windows(b"mantle/store".len()).all(|window| window != b"mantle/store"));

    let mut invalid = mantle_cmd();
    command_base(&mut invalid, &output_dir, &state_dir);
    invalid
        .args(["plan", "--from", NEGATIVE_FRONTEND_FIXTURE])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown field `packages`"));
}

#[test]
fn composition_cli_has_stable_identity_and_realizes_complete_roots() {
    let temp = tempfile::tempdir().unwrap();
    let output_dir = temp.path().join("store");
    let state_dir = temp.path().join("state");
    let store = run_async(open_store(&output_dir, &state_dir));
    let first = run_async(seed_root(&store, "etc", b"first"));
    let second = run_async(seed_root(&store, "bin", b"second"));
    drop(store);

    let request_path = temp.path().join("request.json");
    let mut first_request = request(first, second);
    write_json(&request_path, &first_request);
    let mut plan_command = mantle_cmd();
    command_base(&mut plan_command, &output_dir, &state_dir);
    let first_plan = plan_command
        .args(["plan", "--from", request_path.to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let first_plan: crunch_composition_core::PreparedComposition = serde_json::from_slice(&first_plan).unwrap();

    first_request.plan.bindings.reverse();
    first_request.plan.bindings[0].label = Some("changed caller label".to_string());
    write_json(&request_path, &first_request);
    let mut reordered_command = mantle_cmd();
    command_base(&mut reordered_command, &output_dir, &state_dir);
    let reordered_plan = reordered_command
        .args(["plan", "--from", request_path.to_str().unwrap()])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let reordered_plan: crunch_composition_core::PreparedComposition = serde_json::from_slice(&reordered_plan).unwrap();
    assert_eq!(first_plan.plan_ref, reordered_plan.plan_ref);
    assert_eq!(first_plan.bindings.len(), reordered_plan.bindings.len());

    let receipt_path = temp.path().join("receipt.json");
    let mut realize_command = mantle_cmd();
    command_base(&mut realize_command, &output_dir, &state_dir);
    let output = realize_command
        .args([
            "realize",
            "--from",
            request_path.to_str().unwrap(),
            "--receipt-out",
            receipt_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout_receipt: RealizationReceipt = serde_json::from_slice(&output).unwrap();
    let file_receipt: RealizationReceipt = serde_json::from_slice(&std::fs::read(&receipt_path).unwrap()).unwrap();
    assert_eq!(stdout_receipt, file_receipt);
    assert_eq!(file_receipt.plan_ref, first_plan.plan_ref);
    assert_eq!(file_receipt.resulting_root.digest_blake3.len(), crunch_composition_core::BLAKE3_HEX_CHARS);
    assert!(file_receipt.non_claim.contains("does-not-prove"));
}

#[test]
fn composition_cli_rejects_frontend_intent_and_missing_roots_without_receipts() {
    let temp = tempfile::tempdir().unwrap();
    let output_dir = temp.path().join("store");
    let state_dir = temp.path().join("state");
    let request_path = temp.path().join("request.json");
    let receipt_path = temp.path().join("receipt.json");
    std::fs::write(
        &request_path,
        br#"{
          "plan": {
            "schema": "mantle-composition-plan-v1",
            "merge_policy_version": 1,
            "bindings": [],
            "collision_decisions": [],
            "packages": ["hello"]
          },
          "realization_policy": {
            "schema": "mantle-composition-policy-v1",
            "max_bindings": 64,
            "max_collision_decisions": 64,
            "max_entries": 64,
            "max_depth": 64,
            "max_path_bytes": 64,
            "max_file_bytes": 1024,
            "max_total_file_bytes": 1024,
            "allow_symlinks": true
          }
        }"#,
    )
    .unwrap();
    let mut raw_intent = mantle_cmd();
    command_base(&mut raw_intent, &output_dir, &state_dir);
    raw_intent
        .args(["plan", "--from", request_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown field `packages`"));
    assert!(!receipt_path.exists());

    let missing = CastoreRootRef {
        digest_blake3: "f".repeat(crunch_composition_core::BLAKE3_HEX_CHARS),
        size: 1,
    };
    let mut missing_request = request(missing.clone(), CastoreRootRef {
        digest_blake3: "e".repeat(crunch_composition_core::BLAKE3_HEX_CHARS),
        size: 1,
    });
    missing_request.plan.bindings.truncate(1);
    write_json(&request_path, &missing_request);
    let mut missing_root = mantle_cmd();
    command_base(&mut missing_root, &output_dir, &state_dir);
    missing_root
        .args([
            "realize",
            "--from",
            request_path.to_str().unwrap(),
            "--receipt-out",
            receipt_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("composition-missing-directory"));
    assert!(!receipt_path.exists());
}
