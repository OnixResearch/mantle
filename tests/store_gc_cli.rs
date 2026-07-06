use std::path::Path;

use assert_cmd::Command;
use crunch_store::GcRootSource;
use crunch_store::PersistOutputRequest;
use crunch_store::StoreConfig;
use crunch_store::StoreFallbackMode;
use crunch_store::StoreHandle;
use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_store::path_info::PathInfo;

const STORE_DIR: &str = "/nix/store";

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
        .stdout(predicates::str::contains(format!("DELETE {logical_drop}")));

    assert!(export_path.exists(), "dry-run must not remove exported output");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let store = open_store(state_dir.path(), output_dir.path()).await;
        let listed = crunch_store::store_list(store.pathinfo_service().as_ref()).await.unwrap();
        assert_eq!(listed.len(), 2);
    });
}

#[test]
fn store_gc_refuses_when_mutation_lock_is_held() {
    let (state_dir, output_dir) = seed_store(|_rt, _state, _output| {});
    let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir.path()).unwrap();

    crunch_cmd(state_dir.path(), output_dir.path())
        .args(["store", "gc", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("another local build, substitution, or store mutation is active"));
}
