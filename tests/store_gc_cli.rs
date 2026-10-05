use std::path::Path;
use std::path::PathBuf;

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

fn snapshot_state_tree(root: &Path) -> std::collections::BTreeMap<PathBuf, Option<Vec<u8>>> {
    let mut entries = std::collections::BTreeMap::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for child in std::fs::read_dir(directory).unwrap() {
            let child = child.unwrap();
            let path = child.path();
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            let kind = child.file_type().unwrap();
            let contents = if kind.is_dir() {
                directories.push(path);
                None
            } else {
                assert!(kind.is_file(), "unexpected state member {}", relative.display());
                Some(std::fs::read(path).unwrap())
            };
            assert!(entries.insert(relative, contents).is_none());
        }
    }
    entries
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
        backend: crunch_store::StoreBackend::Snix,
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

async fn casita_signed_symlink_output(
    store: &StoreHandle,
    name: &str,
    target: &str,
    signer: &nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey>,
) -> PathInfo {
    use nix_compat::nixhash::CAHash;
    use nix_compat::nixhash::NixHash;
    use nix_compat::store_path::build_ca_path_with_store_dir;
    use snix_store::nar::NarCalculationService;
    use snix_store::nar::SimpleRenderer;

    let node = Node::Symlink {
        target: SymlinkTarget::try_from(target).unwrap(),
    };
    let (nar_size, nar_sha256) = SimpleRenderer::new(store.blob_service(), store.directory_service())
        .calculate_nar(&node)
        .await
        .unwrap();
    let ca = CAHash::Nar(NixHash::Sha256(nar_sha256));
    let mut path_info = PathInfo {
        store_path: build_ca_path_with_store_dir(name, &ca, Vec::<String>::new(), false, STORE_DIR).unwrap(),
        node,
        references: Vec::new(),
        nar_size,
        nar_sha256,
        signatures: Vec::new(),
        deriver: None,
        ca: Some(ca),
    };
    crunch_build::sign_pathinfo(&mut path_info, signer);
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
    let (state_dir, output_dir) = seed_store(|rt, state, output| {
        rt.block_on(open_store(state, output));
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
    let usage_output = crunch_cmd(state_dir.path(), output_dir.path())
        .arg("--json")
        .args(["store", "usage"])
        .output()
        .unwrap();
    assert!(usage_output.status.success(), "{}", String::from_utf8_lossy(&usage_output.stderr));
    assert!(!state_dir.path().join(crunch_rust_cache::RUST_CACHE_STATE_DIRECTORY).exists());
    let usage: serde_json::Value = serde_json::from_slice(&usage_output.stdout).unwrap();
    let gc_plan_output = crunch_cmd(state_dir.path(), output_dir.path())
        .arg("--json")
        .args(["store", "gc"])
        .output()
        .unwrap();
    assert!(gc_plan_output.status.success(), "{}", String::from_utf8_lossy(&gc_plan_output.stderr));
    let gc_plan: serde_json::Value = serde_json::from_slice(&gc_plan_output.stdout).unwrap();
    assert_eq!(usage["usage"], gc_plan["usage"]);
    assert_eq!(usage["reclaim_observations"], gc_plan["reclaim_observations"]);
    assert_eq!(gc_plan["candidate_path_count"], 1);

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

#[cfg(unix)]
#[test]
fn casita_store_gc_cli_collects_unpinned_signed_output_and_preserves_retained_nar() {
    let state_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let signing_name = "casita-gc-cli";
    let raw_signer = ed25519_dalek::SigningKey::from_bytes(&[23_u8; 32]);
    let trusted_key =
        nix_compat::narinfo::VerifyingKey::new(signing_name.to_string(), raw_signer.verifying_key()).to_string();
    let signer = nix_compat::narinfo::SigningKey::new(signing_name.to_string(), raw_signer);
    std::fs::write(state_dir.path().join("casita-trusted-public-keys"), format!("{trusted_key}\n")).unwrap();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (kept, removed) = runtime.block_on(async {
        let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir.path()).unwrap();
        let mut store = StoreHandle::open(StoreConfig::new(
            crunch_store::StoreBackend::Casita,
            state_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            STORE_DIR.to_string(),
        ))
        .await
        .unwrap();
        let kept = casita_signed_symlink_output(&store, "cli-kept", "kept-nar-content", &signer).await;
        let removed = casita_signed_symlink_output(&store, "cli-collected", "collected-nar-content", &signer).await;
        store.pathinfo_service().put(kept.clone()).await.unwrap();
        store.pathinfo_service().put(removed.clone()).await.unwrap();
        assert_eq!(store.export_cached_path_info(&kept.store_path).await.unwrap(), Some(kept.clone()));
        assert_eq!(store.export_cached_path_info(&removed.store_path).await.unwrap(), Some(removed.clone()));
        (kept, removed)
    });
    let kept_logical = kept.store_path.to_absolute_path();
    let removed_logical = removed.store_path.to_absolute_path();
    let kept_selector = kept.store_path.to_string();
    let removed_selector = removed.store_path.to_string();
    let kept_export = output_dir.path().join(&kept_selector);
    let removed_export = output_dir.path().join(&removed_selector);
    assert!(std::fs::symlink_metadata(&kept_export).unwrap().file_type().is_symlink());
    assert!(std::fs::symlink_metadata(&removed_export).unwrap().file_type().is_symlink());
    let casita_cmd = || {
        let mut cmd = crunch_cmd(state_dir.path(), output_dir.path());
        cmd.args(["--store-backend", "casita"]);
        cmd
    };
    casita_cmd()
        .args(["store", "pin", &kept_logical])
        .assert()
        .success()
        .stdout(predicates::str::contains("PINNED"));

    let plan_output = casita_cmd().args(["--json", "store", "gc"]).output().unwrap();
    assert!(plan_output.status.success(), "{}", String::from_utf8_lossy(&plan_output.stderr));
    let plan: serde_json::Value = serde_json::from_slice(&plan_output.stdout).unwrap();
    assert_eq!(plan["is_dry_run"], true);
    assert_eq!(plan["retained_root_count"], 1);
    assert_eq!(plan["candidate_path_count"], 1);
    assert_eq!(plan["candidate_exported_output_count"], 1);
    assert_eq!(plan["candidate_paths"], serde_json::json!([removed_logical]));
    assert!(std::fs::symlink_metadata(&removed_export).is_ok(), "GC plan must not remove an export");

    let plan_id = plan["plan_id"].as_str().unwrap();
    let execution_output =
        casita_cmd().args(["--json", "store", "gc", "--execute", "--plan-id", plan_id]).output().unwrap();
    assert!(execution_output.status.success(), "{}", String::from_utf8_lossy(&execution_output.stderr));
    let execution: serde_json::Value = serde_json::from_slice(&execution_output.stdout).unwrap();
    assert_eq!(execution["plan_id"], plan["plan_id"]);
    assert_eq!(execution["is_dry_run"], false);
    assert_eq!(execution["execution_complete"], true);
    assert_eq!(execution["failed_operations"], serde_json::json!([]));
    assert_eq!(execution["candidate_paths"], plan["candidate_paths"]);
    assert!(execution["operations"].as_array().unwrap().contains(&serde_json::json!("casita-collection")));

    let info_output = casita_cmd().args(["--json", "store", "info", &kept_selector]).output().unwrap();
    assert!(info_output.status.success(), "{}", String::from_utf8_lossy(&info_output.stderr));
    let info: serde_json::Value = serde_json::from_slice(&info_output.stdout).unwrap();
    assert_eq!(info["backend"], "casita");
    assert_eq!(info["paths"].as_array().unwrap().len(), 1);
    assert_eq!(info["paths"][0]["store_path"], kept_selector);
    assert_eq!(info["paths"][0]["nar_size"], kept.nar_size);
    assert_eq!(info["paths"][0]["nar_sha256"], data_encoding::HEXLOWER.encode(kept.nar_sha256.as_ref()));
    assert_eq!(
        info["paths"][0]["signatures"],
        serde_json::json!(kept.signatures.iter().map(ToString::to_string).collect::<Vec<_>>())
    );
    casita_cmd()
        .args(["store", "verify", "--trusted-public-keys", &trusted_key, &kept_selector])
        .assert()
        .success()
        .stdout(predicates::str::contains(format!("OK {kept_selector}  trusted_signatures=1/1")));
    assert_eq!(std::fs::read_link(&kept_export).unwrap(), Path::new("kept-nar-content"));

    let listed_output = casita_cmd().args(["--json", "store", "list"]).output().unwrap();
    assert!(listed_output.status.success(), "{}", String::from_utf8_lossy(&listed_output.stderr));
    let listed: serde_json::Value = serde_json::from_slice(&listed_output.stdout).unwrap();
    assert_eq!(listed["paths"].as_array().unwrap().len(), 1);
    assert_eq!(listed["paths"][0]["store_path"], kept_selector);
    casita_cmd()
        .args(["store", "info", &removed_selector])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no PathInfo matching"));
    assert_eq!(std::fs::symlink_metadata(&removed_export).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    println!(
        "CASITA_GC_CLI_EVIDENCE {}",
        serde_json::json!({
            "plan_id": plan["plan_id"],
            "retained_root_count": plan["retained_root_count"],
            "candidate_paths": execution["candidate_paths"],
            "execution_complete": execution["execution_complete"],
            "operations": execution["operations"],
            "retained_nar_size": info["paths"][0]["nar_size"],
            "retained_nar_sha256": info["paths"][0]["nar_sha256"],
        })
    );
}

#[cfg(unix)]
#[test]
fn casita_gc_cli_recovers_removed_root_before_planning_without_removing_other_roots() {
    use casita::experimental::ChunkedBlobStore;
    use casita::experimental::MetadataStore;
    use casita::experimental::ObjectKey;
    use casita::experimental::Repository;
    use casita::experimental::RootName;
    use casita::experimental::TursoMetadataStore;

    let state = tempfile::tempdir().unwrap();
    let exports = tempfile::tempdir().unwrap();
    let raw_signer = ed25519_dalek::SigningKey::from_bytes(&[23_u8; 32]);
    let trusted = nix_compat::narinfo::VerifyingKey::new("casita-gc-cli".to_string(), raw_signer.verifying_key());
    let signer = nix_compat::narinfo::SigningKey::new("casita-gc-cli".to_string(), raw_signer);
    std::fs::write(state.path().join("casita-trusted-public-keys"), format!("{trusted}\n")).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (removed, fenced_other, retained) = rt.block_on(async {
        let _guard = crunch_store::StoreMutationGuard::acquire_wait(state.path()).unwrap();
        let mut store = StoreHandle::open(StoreConfig::new(
            crunch_store::StoreBackend::Casita,
            state.path().to_path_buf(),
            exports.path().to_path_buf(),
            STORE_DIR.to_string(),
        ))
        .await
        .unwrap();
        let removed = casita_signed_symlink_output(&store, "fenced-removed", "removed-content", &signer).await;
        let fenced_other = casita_signed_symlink_output(&store, "fenced-other", "other-content", &signer).await;
        let retained = casita_signed_symlink_output(&store, "fenced-retained", "retained-content", &signer).await;
        for info in [&removed, &fenced_other, &retained] {
            store.pathinfo_service().put(info.clone()).await.unwrap();
            assert_eq!(store.export_cached_path_info(&info.store_path).await.unwrap(), Some(info.clone()));
        }
        (removed, fenced_other, retained)
    });
    // Model a stop immediately after the first root in the actual plan order.
    let (removed, fenced_other) = if removed.store_path.to_absolute_path() < fenced_other.store_path.to_absolute_path() {
        (removed, fenced_other)
    } else {
        (fenced_other, removed)
    };
    let casita_cmd = || {
        let mut cmd = crunch_cmd(state.path(), exports.path());
        cmd.args(["--store-backend", "casita"]);
        cmd
    };
    casita_cmd().args(["store", "pin", &retained.store_path.to_absolute_path()]).assert().success();
    let initial = casita_cmd().args(["--json", "store", "gc"]).output().unwrap();
    assert!(initial.status.success(), "{}", String::from_utf8_lossy(&initial.stderr));
    let initial: serde_json::Value = serde_json::from_slice(&initial.stdout).unwrap();
    assert_eq!(initial["candidate_path_count"], 2);
    let removed_logical = removed.store_path.to_absolute_path();
    let other_logical = fenced_other.store_path.to_absolute_path();
    let initial_paths = initial["candidate_paths"].as_array().unwrap();
    assert_eq!(initial_paths, &vec![serde_json::json!(removed_logical), serde_json::json!(other_logical)]);

    rt.block_on(async {
        let repository = Repository::<ChunkedBlobStore, TursoMetadataStore>::local(state.path().join("casita"))
            .await
            .unwrap();
        let name = |info: &PathInfo| {
            RootName::try_from(format!("mantle/outputs/{}", data_encoding::HEXLOWER.encode(info.store_path.digest())))
                .unwrap()
        };
        let removed_name = name(&removed);
        let other_name = name(&fenced_other);
        let retained_name = name(&retained);
        let snapshot = repository.metadata().snapshot().await.unwrap();
        let removed_target = snapshot.root(&removed_name).await.unwrap().unwrap();
        let other_target = snapshot.root(&other_name).await.unwrap().unwrap();
        let retained_target = snapshot.root(&retained_name).await.unwrap().unwrap();
        drop(snapshot);
        let fence_entry = |info: &PathInfo, name: &RootName, target: &ObjectKey| {
            serde_json::json!({
                "kind": "output",
                "path": info.store_path.to_absolute_path(),
                "root_name": name.to_string(),
                "expected_target": target.to_string(),
                "removed": false,
                "cleaned": false,
            })
        };
        std::fs::write(
            state.path().join("casita-gc-fence.json"),
            serde_json::to_vec(&serde_json::json!({
                "plan_id": initial["plan_id"],
                "entries": [
                    fence_entry(&removed, &removed_name, &removed_target),
                    fence_entry(&fenced_other, &other_name, &other_target),
                ],
            }))
            .unwrap(),
        )
        .unwrap();
        assert!(repository.remove_root_if_matches(&removed_name, &removed_target).await.unwrap().is_some());
        assert_eq!(repository.metadata().snapshot().await.unwrap().root(&other_name).await.unwrap(), Some(other_target.clone()));
        assert_eq!(repository.metadata().snapshot().await.unwrap().root(&retained_name).await.unwrap(), Some(retained_target.clone()));

        let recovered = casita_cmd().args(["--json", "store", "gc"]).output().unwrap();
        assert!(recovered.status.success(), "{}", String::from_utf8_lossy(&recovered.stderr));
        let plan: serde_json::Value = serde_json::from_slice(&recovered.stdout).unwrap();
        assert_eq!(plan["is_dry_run"], true);
        assert_eq!(plan["candidate_paths"], serde_json::json!([other_logical]));
        assert!(!state.path().join("casita-gc-fence.json").exists());
        assert_eq!(repository.metadata().snapshot().await.unwrap().root(&removed_name).await.unwrap(), None);
        assert_eq!(repository.metadata().snapshot().await.unwrap().root(&other_name).await.unwrap(), Some(other_target));
        assert_eq!(repository.metadata().snapshot().await.unwrap().root(&retained_name).await.unwrap(), Some(retained_target));
    });
    let removed_export = exports.path().join(removed.store_path.to_string());
    assert_eq!(std::fs::symlink_metadata(removed_export).unwrap_err().kind(), std::io::ErrorKind::NotFound);
    for info in [&fenced_other, &retained] {
        let selector = info.store_path.to_string();
        let result = casita_cmd().args(["--json", "store", "info", &selector]).output().unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let observed: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(observed["paths"][0]["nar_sha256"], data_encoding::HEXLOWER.encode(&info.nar_sha256));
        casita_cmd()
            .args(["store", "verify", "--trusted-public-keys", &trusted.to_string(), &selector])
            .assert()
            .success();
    }
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
    assert_eq!(report["backend"], "snix");
    assert_eq!(report["backend_capabilities"]["overlay_composition"], true);
    assert_eq!(report["backend_capabilities"]["atomic_batch_import"], true);
    assert_eq!(report["backend_capabilities"]["rust_unit_cache"], true);
    assert_eq!(report["backend_capabilities"]["unsigned_admission"], true);
    assert!(report["backend_capabilities"]["max_root_changes"].is_null());
    assert_eq!(report["paths"][0]["layer"]["kind"], "base");
    assert_eq!(report["paths"][0]["layer"]["index"], 1);
    assert_eq!(
        report["overlay"]["bases"][0]["accepted_signer_names"][0],
        report["paths"][0]["signatures"][0].as_str().unwrap().split(':').next().unwrap()
    );
    assert!(report["overlay"]["bases"][0]["generation_blake3"].as_str().unwrap().starts_with("b3:"));
    crunch_cmd(overlay_state.path(), overlay_output.path())
        .arg("--base-store")
        .arg(base_state.path())
        .args(["store", "info", "cli-base"])
        .assert()
        .success()
        .stdout(predicates::str::contains("backend:    snix"))
        .stdout(predicates::str::contains("overlay-composition: true"))
        .stdout(predicates::str::contains("atomic-batch-import: true"))
        .stdout(predicates::str::contains("unsigned-admission: true"))
        .stdout(predicates::str::contains("max-root-changes: unbounded-by-backend"));
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

#[test]
fn selected_backend_is_recorded_and_mismatch_rejects_without_mutation() {
    let state = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let selected = crunch_cmd(state.path(), output_dir.path())
        .args(["--store-backend", "casita", "--json", "store", "list"])
        .output()
        .unwrap();
    assert!(selected.status.success(), "{}", String::from_utf8_lossy(&selected.stderr));
    let identity_path = state.path().join("store-identity.json");
    let identity = std::fs::read(&identity_path).unwrap();
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&identity).unwrap()["backend"], "casita");
    let before = snapshot_state_tree(state.path());
    let output_before = snapshot_state_tree(output_dir.path());
    let mismatch = crunch_cmd(state.path(), output_dir.path())
        .args(["--store-backend", "snix", "store", "list"])
        .output()
        .unwrap();
    assert!(!mismatch.status.success());
    assert!(
        String::from_utf8_lossy(&mismatch.stderr)
            .contains("store-backend-mismatch: requested snix, state declares casita")
    );
    assert_eq!(snapshot_state_tree(state.path()), before);
    assert_eq!(snapshot_state_tree(output_dir.path()), output_before);
}

fn seed_identityless_snix_state(state: &Path, output_dir: &Path) {
    crunch_cmd(state, output_dir)
        .env_remove("CRUNCH_CONFIG_DIR")
        .args(["--store-backend", "snix", "store", "list"])
        .assert()
        .success();
    assert!(state.join("pathinfo.redb").is_file(), "fixture must contain a real Snix database");
    std::fs::remove_file(state.join("store-identity.json")).unwrap();
}

#[test]
fn identityless_casita_repository_rejects_both_backends_with_or_without_snix_state() {
    for backend in ["snix", "casita"] {
        for with_snix in [false, true] {
            let state = tempfile::tempdir().unwrap();
            let output_dir = tempfile::tempdir().unwrap();
            if with_snix {
                seed_identityless_snix_state(state.path(), output_dir.path());
            }
            let repository = state.path().join("casita");
            std::fs::create_dir(&repository).unwrap();
            std::fs::write(repository.join("casita.sqlite"), b"existing repository bytes").unwrap();
            assert!(!state.path().join("store-identity.json").exists());
            let state_before = snapshot_state_tree(state.path());
            let output_before = snapshot_state_tree(output_dir.path());

            let rejected = crunch_cmd(state.path(), output_dir.path())
                .env_remove("CRUNCH_CONFIG_DIR")
                .args(["--store-backend", backend, "store", "list"])
                .output()
                .unwrap();
            assert!(!rejected.status.success(), "{backend}, with_snix={with_snix}");
            let stderr = String::from_utf8_lossy(&rejected.stderr);
            assert!(
                stderr.contains("store-backend-mismatch: state directory")
                    && stderr.contains("has content but no backend identity"),
                "{backend}, with_snix={with_snix}: {stderr}"
            );
            assert_eq!(
                snapshot_state_tree(state.path()),
                state_before,
                "{backend}, with_snix={with_snix}: rejected open changed state"
            );
            assert_eq!(
                snapshot_state_tree(output_dir.path()),
                output_before,
                "{backend}, with_snix={with_snix}: rejected open changed outputs"
            );
        }
    }
}

#[test]
fn identityless_foreign_content_rejects_casita_before_state_or_output_mutation() {
    for with_snix in [false, true] {
        let state = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        if with_snix {
            seed_identityless_snix_state(state.path(), output_dir.path());
        } else {
            std::fs::write(state.path().join("foreign-state.dat"), b"unclaimed persistent state").unwrap();
        }
        assert!(!state.path().join("store-identity.json").exists());
        let state_before = snapshot_state_tree(state.path());
        let output_before = snapshot_state_tree(output_dir.path());

        let rejected = crunch_cmd(state.path(), output_dir.path())
            .env_remove("CRUNCH_CONFIG_DIR")
            .args(["--store-backend", "casita", "store", "list"])
            .output()
            .unwrap();
        assert!(!rejected.status.success(), "with_snix={with_snix}");
        let stderr = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            stderr.contains("store-backend-mismatch: state directory")
                && stderr.contains("has content but no backend identity"),
            "with_snix={with_snix}: {stderr}"
        );
        assert_eq!(snapshot_state_tree(state.path()), state_before, "rejected open changed state");
        assert_eq!(snapshot_state_tree(output_dir.path()), output_before, "rejected open changed outputs");
    }
}

#[test]
fn wrong_backend_sign_rejects_before_creating_a_signer_or_changing_state() {
    let state = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    crunch_cmd(state.path(), output_dir.path())
        .env_remove("CRUNCH_CONFIG_DIR")
        .args(["--store-backend", "snix", "store", "list"])
        .assert()
        .success();
    let before = snapshot_state_tree(state.path());
    let output_before = snapshot_state_tree(output_dir.path());
    assert!(!before.contains_key(Path::new("signing-key")));

    let rejected = crunch_cmd(state.path(), output_dir.path())
        .env_remove("CRUNCH_CONFIG_DIR")
        .args(["--store-backend", "casita", "store", "sign", "--all"])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    let stderr = String::from_utf8_lossy(&rejected.stderr);
    assert!(stderr.contains("store-backend-mismatch: requested casita, state declares snix"), "{stderr}");
    assert!(!stderr.contains("Generated signing key"), "{stderr}");
    assert_eq!(snapshot_state_tree(state.path()), before, "rejected sign changed existing state");
    assert_eq!(snapshot_state_tree(output_dir.path()), output_before, "rejected sign changed outputs");
    assert!(!state.path().join("signing-key").exists());
}

#[test]
fn wrong_backend_build_rejects_before_creating_a_signer_or_changing_state() {
    let state = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("crunch.ncl"), "let crunch = import \"lib.ncl\" in { packages = { hello = { name = \"hello\", builder = \"/bin/sh\", args = [\"-c\", \"echo hello > $out\"], addressing_mode = 'input-addressed } | crunch.Derivation }, default = { package = \"hello\" } } | crunch.Project").unwrap();
    let selected = crunch_cmd(state.path(), output_dir.path())
        .env_remove("CRUNCH_CONFIG_DIR")
        .args(["--store-backend", "snix", "store", "list"])
        .output()
        .unwrap();
    assert!(selected.status.success(), "{}", String::from_utf8_lossy(&selected.stderr));
    let before = snapshot_state_tree(state.path());
    let output_before = snapshot_state_tree(output_dir.path());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(before[Path::new("store-identity.json")].as_ref().unwrap())
            .unwrap()["backend"],
        "snix"
    );
    assert!(!before.contains_key(Path::new("signing-key")));

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ncl");
    for (target, current_dir) in [(example.as_path(), None), (Path::new(".#hello"), Some(project.path()))] {
        let mut cmd = crunch_cmd(state.path(), output_dir.path());
        cmd.env_remove("CRUNCH_CONFIG_DIR")
            .args(["--store-backend", "casita", "build"])
            .arg(target)
            .arg("--no-substitute");
        if let Some(current_dir) = current_dir {
            cmd.current_dir(current_dir);
        }
        let mismatch = cmd.output().unwrap();
        assert!(!mismatch.status.success(), "{target:?} unexpectedly succeeded");
        let stderr = String::from_utf8_lossy(&mismatch.stderr);
        assert!(
            stderr.contains("store-backend-mismatch: requested casita, state declares snix"),
            "{target:?}: {stderr}"
        );
        assert!(!stderr.contains("Generated signing key"), "{target:?}: {stderr}");
        assert_eq!(snapshot_state_tree(state.path()), before, "{target:?} changed state");
        assert_eq!(snapshot_state_tree(output_dir.path()), output_before, "{target:?} changed outputs");
        assert!(!state.path().join("signing-key").exists());
    }

    crunch_cmd(state.path(), output_dir.path())
        .env_remove("CRUNCH_CONFIG_DIR")
        .args(["--store-backend", "snix", "store", "list"])
        .assert()
        .success();
}

#[test]
fn wrong_backend_overlay_base_build_rejects_before_creating_a_signer() {
    let base = tempfile::tempdir().unwrap();
    let base_output = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let base_selected = crunch_cmd(base.path(), base_output.path())
        .env_remove("CRUNCH_CONFIG_DIR")
        .args(["--store-backend", "casita", "store", "list"])
        .output()
        .unwrap();
    assert!(base_selected.status.success(), "{}", String::from_utf8_lossy(&base_selected.stderr));
    set_tree_read_only(base.path(), true);
    let base_before = snapshot_state_tree(base.path());
    let base_output_before = snapshot_state_tree(base_output.path());
    let selected = crunch_cmd(state.path(), output_dir.path())
        .env_remove("CRUNCH_CONFIG_DIR")
        .args(["--store-backend", "snix", "store", "list"])
        .output()
        .unwrap();
    assert!(selected.status.success(), "{}", String::from_utf8_lossy(&selected.stderr));
    let before = snapshot_state_tree(state.path());
    let output_before = snapshot_state_tree(output_dir.path());
    assert!(!before.contains_key(Path::new("signing-key")));

    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/hello.ncl");
    let mismatch = crunch_cmd(state.path(), output_dir.path())
        .env_remove("CRUNCH_CONFIG_DIR")
        .arg("--base-store")
        .arg(base.path())
        .args(["--store-backend", "snix", "build"])
        .arg(example)
        .arg("--no-substitute")
        .output()
        .unwrap();
    assert!(!mismatch.status.success());
    let stderr = String::from_utf8_lossy(&mismatch.stderr);
    assert!(stderr.contains("store-backend-mismatch: requested snix, state declares casita"), "{stderr}");
    assert!(!stderr.contains("Generated signing key"), "{stderr}");
    assert_eq!(snapshot_state_tree(state.path()), before, "rejected overlay base changed writable state");
    assert_eq!(snapshot_state_tree(base.path()), base_before, "rejected overlay base changed read-only base");
    assert_eq!(snapshot_state_tree(output_dir.path()), output_before, "rejected overlay base changed outputs");
    assert_eq!(
        snapshot_state_tree(base_output.path()),
        base_output_before,
        "rejected overlay base changed base outputs"
    );
    assert!(!state.path().join("signing-key").exists());
    set_tree_read_only(base.path(), false);
}

#[test]
fn unknown_backend_is_rejected_before_creating_state() {
    let root = tempfile::tempdir().unwrap();
    let state_dir = root.path().join("absent-state");
    let output_dir = root.path().join("absent-store");
    let before = snapshot_state_tree(root.path());
    let output = crunch_cmd(&state_dir, &output_dir)
        .args(["--store-backend", "tape", "store", "list"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("store-backend-unknown"));
    assert!(!state_dir.exists());
    assert!(!output_dir.exists());
    assert_eq!(snapshot_state_tree(root.path()), before, "unknown backend changed filesystem");
}


#[test]
fn ambient_backend_name_cannot_override_default_snix_selection() {
    let state = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let selected = crunch_cmd(state.path(), output_dir.path())
        .env("MANTLE_STORE_BACKEND", "casita")
        .args(["store", "list"])
        .output()
        .unwrap();
    assert!(selected.status.success(), "{}", String::from_utf8_lossy(&selected.stderr));
    let identity: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state.path().join("store-identity.json")).unwrap()).unwrap();
    assert_eq!(identity["backend"], "snix");

    let casita_state = tempfile::tempdir().unwrap();
    let casita_output = tempfile::tempdir().unwrap();
    crunch_cmd(casita_state.path(), casita_output.path())
        .args(["--store-backend", "casita", "store", "list"])
        .assert()
        .success();
    let before = snapshot_state_tree(casita_state.path());
    let output_before = snapshot_state_tree(casita_output.path());
    let rejected = crunch_cmd(casita_state.path(), casita_output.path())
        .env("MANTLE_STORE_BACKEND", "casita")
        .args(["store", "list"])
        .output()
        .unwrap();
    assert!(!rejected.status.success());
    assert!(
        String::from_utf8_lossy(&rejected.stderr)
            .contains("store-backend-mismatch: requested snix, state declares casita")
    );
    assert_eq!(snapshot_state_tree(casita_state.path()), before);
    assert_eq!(snapshot_state_tree(casita_output.path()), output_before);
}
