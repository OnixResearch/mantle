use std::path::Path;

use assert_cmd::Command;
use crunch_store::GcRootSource;
use crunch_store::PersistOutputRequest;
use crunch_store::StoreConfig;
use crunch_store::StoreFallbackMode;
use crunch_store::StoreHandle;
use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_castore::SymlinkTarget;
use snix_store::path_info::PathInfo;

const STORE_DIR: &str = "/nix/store";
const NAR_SHA256_BYTES: usize = 32;

fn crunch_cmd(state_dir: &Path, output_dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("crunch").unwrap();
    cmd.arg("--state-dir").arg(state_dir).arg("--store").arg(output_dir).arg("--nix-compat");
    cmd
}

fn seed_store<F>(seed: F) -> (tempfile::TempDir, tempfile::TempDir)
where F: FnOnce(&tokio::runtime::Runtime, &Path, &Path) {
    let state_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    seed(&rt, state_dir.path(), output_dir.path());
    (state_dir, output_dir)
}

async fn open_store(state_dir: &Path, output_dir: &Path) -> StoreHandle {
    StoreHandle::open(StoreConfig {
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        base_state_dirs: Vec::new(),
        fallback_mode: StoreFallbackMode::Practical,
        store_dir: STORE_DIR.to_string(),
    })
    .await
    .unwrap()
}

fn test_output(name: &str, seed: u8) -> StorePath<String> {
    StorePath::from_name_and_digest_fixed(name, [seed; 20]).unwrap()
}

fn signed_pathinfo(store_path: StorePath<String>, node: Node, refs: Vec<StorePath<String>>) -> PathInfo {
    let (keypair, _) = crunch_build::generate_keypair();
    let mut path_info = PathInfo {
        store_path,
        node,
        references: refs,
        nar_size: 1,
        nar_sha256: [3u8; 32],
        signatures: vec![],
        deriver: None,
        ca: None,
    };
    crunch_build::sign_pathinfo(&mut path_info, &keypair.signing_key);
    path_info
}

fn set_tree_read_only(root: &Path, is_read_only: bool) {
    let mut directories = vec![root.to_path_buf()];
    let mut visited = Vec::new();
    while let Some(directory) = directories.pop() {
        visited.push(directory.clone());
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            let metadata = std::fs::symlink_metadata(&path).unwrap();
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                directories.push(path);
            } else {
                let mut permissions = metadata.permissions();
                permissions.set_readonly(is_read_only);
                std::fs::set_permissions(path, permissions).unwrap();
            }
        }
    }
    for directory in visited.into_iter().rev() {
        let mut permissions = std::fs::metadata(&directory).unwrap().permissions();
        permissions.set_readonly(is_read_only);
        std::fs::set_permissions(directory, permissions).unwrap();
    }
}

async fn seed_overlay_base(state_dir: &Path, output_dir: &Path, store_path: &StorePath<String>) {
    let store = open_store(state_dir, output_dir).await;
    let (keypair, _) = crunch_build::generate_keypair();
    std::fs::write(state_dir.join("overlay-trusted-public-keys"), format!("{}\n", keypair.verifying_key)).unwrap();
    let mut path_info = PathInfo {
        store_path: store_path.clone(),
        node: Node::Symlink {
            target: SymlinkTarget::try_from("base-target").unwrap(),
        },
        references: Vec::new(),
        nar_size: 1,
        nar_sha256: [5_u8; NAR_SHA256_BYTES],
        signatures: Vec::new(),
        deriver: None,
        ca: None,
    };
    crunch_build::sign_pathinfo(&mut path_info, &keypair.signing_key);
    store.pathinfo_service().put(path_info).await.unwrap();
    drop(store);
}

async fn write_blob(store: &StoreHandle, contents: &[u8]) -> Node {
    use tokio::io::AsyncWriteExt;

    let mut writer = store.blob_service().open_write().await;
    writer.write_all(contents).await.unwrap();
    let digest = writer.close().await.unwrap();
    Node::File {
        digest,
        size: contents.len() as u64,
        executable: false,
    }
}

async fn persist_output(
    store: &mut StoreHandle,
    output_name: &str,
    store_path: StorePath<String>,
    node: Node,
    refs: Vec<StorePath<String>>,
    is_root: bool,
    source: Option<GcRootSource>,
) {
    let path_info = signed_pathinfo(store_path.clone(), node.clone(), refs);
    store
        .persist_and_export_signed_output(PersistOutputRequest {
            output_name,
            output_path: &store_path,
            path_info,
            final_node: node,
            provenance: None,
            is_root,
            root_source: source,
        })
        .await
        .unwrap();
}

#[test]
fn store_pin_and_unpin_round_trip() {
    let logical_path = test_output("pin-me", 1).to_absolute_path();
    let (state_dir, output_dir) = seed_store(|rt, state, output| {
        rt.block_on(async {
            let mut store = open_store(state, output).await;
            let node = write_blob(&store, b"pin").await;
            persist_output(&mut store, "out", test_output("pin-me", 1), node, vec![], false, None).await;
        });
    });

    crunch_cmd(state_dir.path(), output_dir.path()).args(["store", "roots"]).assert().success();

    crunch_cmd(state_dir.path(), output_dir.path())
        .args(["store", "pin", &logical_path])
        .assert()
        .success()
        .stdout(predicates::str::contains("PINNED"));

    crunch_cmd(state_dir.path(), output_dir.path())
        .args(["store", "roots"])
        .assert()
        .success()
        .stdout(predicates::str::contains(&logical_path));

    crunch_cmd(state_dir.path(), output_dir.path())
        .args(["store", "unpin", &logical_path])
        .assert()
        .success()
        .stdout(predicates::str::contains("UNPINNED"));
}

#[test]
fn store_roots_migrates_legacy_records_without_inventing_ownership() {
    const LEGACY_CREATED_UNIX_S: i64 = 100;
    const LEGACY_PATH_DIGEST_BYTE: u8 = 4;
    const EXPECTED_SCHEMA_VERSION: u64 = 3;
    let legacy_path = test_output("legacy", LEGACY_PATH_DIGEST_BYTE).to_absolute_path();
    let (state_dir, output_dir) = seed_store(|_rt, state, _output| {
        let legacy = serde_json::json!({
            legacy_path.clone(): {
                "logical_path": legacy_path.clone(),
                "source": "build",
                "created_unix_s": LEGACY_CREATED_UNIX_S,
            }
        });
        std::fs::write(state.join("gc-roots.json"), serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();
    });

    crunch_cmd(state_dir.path(), output_dir.path())
        .args(["store", "roots", "--migrate"])
        .assert()
        .success()
        .stdout(predicates::str::contains("legacy-unmanaged"));

    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state_dir.path().join("gc-roots.json")).unwrap()).unwrap();
    assert_eq!(persisted[&legacy_path]["schema_version"], EXPECTED_SCHEMA_VERSION);
    assert_eq!(persisted[&legacy_path]["root_class"], "legacy-unmanaged");
    assert_eq!(persisted[&legacy_path]["owner_scope"], "legacy-unmanaged");
    assert!(persisted[&legacy_path]["project_identity"].is_null());
    assert_eq!(persisted[&legacy_path]["last_transition_reason"], "migrated-from-path-only-v1");
}

#[test]
fn store_gc_dry_run_reports_candidate_without_mutating() {
    let keep_path = test_output("keep", 2);
    let drop_path = test_output("drop", 3);
    let logical_drop = drop_path.to_absolute_path();
    let (state_dir, output_dir) = seed_store(|rt, state, output| {
        rt.block_on(async {
            let mut store = open_store(state, output).await;
            let keep_node = write_blob(&store, b"keep").await;
            let drop_node = write_blob(&store, b"drop").await;
            persist_output(&mut store, "out", keep_path.clone(), keep_node, vec![], true, Some(GcRootSource::Build))
                .await;
            persist_output(&mut store, "out", drop_path.clone(), drop_node, vec![], true, None).await;
        });
    });
    let export_path = output_dir.path().join(drop_path.to_string());
    assert!(export_path.exists());

    crunch_cmd(state_dir.path(), output_dir.path())
        .args(["store", "gc", "--dry-run"])
        .assert()
        .success()
        .stdout(predicates::str::contains("candidate_paths=1"))
        .stdout(predicates::str::contains(format!("CANDIDATE {logical_drop}")));

    assert!(export_path.exists(), "dry-run must not remove exported output");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let store = open_store(state_dir.path(), output_dir.path()).await;
        let listed = crunch_store::store_list(store.pathinfo_service().as_ref()).await.unwrap();
        assert_eq!(listed.len(), 2);
    });
}

#[test]
fn store_gc_execution_refuses_when_mutation_lock_is_held() {
    let (state_dir, output_dir) = seed_store(|_rt, _state, _output| {});
    let plan_output = crunch_cmd(state_dir.path(), output_dir.path())
        .arg("--json")
        .args(["store", "gc"])
        .output()
        .expect("GC plan command");
    assert!(plan_output.status.success());
    let plan: serde_json::Value = serde_json::from_slice(&plan_output.stdout).expect("GC plan JSON");
    let plan_id = plan["plan_id"].as_str().expect("GC plan identity");
    let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir.path()).unwrap();

    crunch_cmd(state_dir.path(), output_dir.path())
        .args(["store", "gc", "--execute", "--plan-id", plan_id])
        .assert()
        .failure()
        .stderr(predicates::str::contains("another local build, substitution, or store mutation is active"));
}

#[test]
fn store_info_reports_base_layer_without_mutating_base() {
    const BASE_PATH_DIGEST_BYTE: u8 = 61;
    let base_state = tempfile::tempdir().unwrap();
    let base_output = tempfile::tempdir().unwrap();
    let overlay_state = tempfile::tempdir().unwrap();
    let overlay_output = tempfile::tempdir().unwrap();
    let base_path = test_output("cli-base", BASE_PATH_DIGEST_BYTE);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(seed_overlay_base(base_state.path(), base_output.path(), &base_path));
    // Debug: list base state dir contents
    for entry in std::fs::read_dir(base_state.path()).unwrap() {
        let entry = entry.unwrap();
        let meta = std::fs::symlink_metadata(entry.path());
    }
    set_tree_read_only(base_state.path(), true);
    let base_database = base_state.path().join("pathinfo.redb");
    let base_before = blake3::hash(&std::fs::read(&base_database).unwrap());

    let output = crunch_cmd(overlay_state.path(), overlay_output.path())
        .arg("--base-store")
        .arg(base_state.path())
        .arg("--json")
        .args(["store", "info", "cli-base"])
        .output()
        .expect("composed store info command");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema"], "mantle-store-info-v2");
    assert_eq!(report["paths"][0]["layer"]["kind"], "base");
    assert_eq!(report["paths"][0]["layer"]["index"], 1);
    assert_eq!(
        report["overlay"]["bases"][0]["accepted_signer_names"][0],
        report["paths"][0]["signatures"][0].as_str().unwrap().split(':').next().unwrap()
    );
    assert!(report["overlay"]["bases"][0]["generation_blake3"].as_str().unwrap().starts_with("b3:"));
    let base_after = blake3::hash(&std::fs::read(&base_database).unwrap());
    assert_eq!(base_before, base_after);
    set_tree_read_only(base_state.path(), false);
}

#[test]
fn store_info_rejects_writable_base_before_overlay_creation() {
    const WRITABLE_BASE_DIGEST_BYTE: u8 = 62;
    let base_state = tempfile::tempdir().unwrap();
    let base_output = tempfile::tempdir().unwrap();
    let overlay_state = tempfile::tempdir().unwrap();
    let overlay_output = tempfile::tempdir().unwrap();
    let base_path = test_output("writable-cli-base", WRITABLE_BASE_DIGEST_BYTE);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(seed_overlay_base(base_state.path(), base_output.path(), &base_path));

    crunch_cmd(overlay_state.path(), overlay_output.path())
        .arg("--base-store")
        .arg(base_state.path())
        .args(["store", "info", "writable-cli-base"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("overlay-base-writable"));
    assert!(!overlay_state.path().join("pathinfo.redb").exists());
}
