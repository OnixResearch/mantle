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
const RUST_CACHE_OUTPUT_BYTES: &[u8] = b"retained Rust unit artifact";

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

async fn signed_ca_symlink_output(
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

async fn seed_casita_rust_cache(
    state_dir: &Path,
    output_dir: &Path,
    rust_output_dir: &Path,
) -> crunch_rust_cache_core::RustUnitAction {
    use crunch_rust_cache_core::LocalCachePolicy;
    use crunch_rust_cache_core::RustBuildFact;
    use crunch_rust_cache_core::RustSemanticArgument;
    use crunch_rust_cache_core::RustUnitActionInput;
    use crunch_rust_cache_core::canonical_rust_action;
    use crunch_rust_cache::PublishRequest;

    let digest = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let action = canonical_rust_action(RustUnitActionInput {
        unit_id: "gc-retained-unit".to_string(),
        package_id: "gc-retained-package".to_string(),
        crate_name: "gc_retained".to_string(),
        target_kind: "lib".to_string(),
        execution_kind: "target".to_string(),
        host_triple: "x86_64-unknown-linux-gnu".to_string(),
        target_triple: "x86_64-unknown-linux-gnu".to_string(),
        profile: "debug".to_string(),
        mode: "build".to_string(),
        features: Vec::new(),
        source_digest_blake3: digest.to_string(),
        compiler_digest_blake3: digest.to_string(),
        compiler_version_digest_blake3: digest.to_string(),
        toolchain_closure_digest_blake3: digest.to_string(),
        execution_platform_digest_blake3: digest.to_string(),
        semantic_arguments: vec![RustSemanticArgument {
            value: "--crate-type=lib".to_string(),
            contains_absolute_path: false,
            absolute_paths_classified: false,
        }],
        admitted_environment: std::collections::BTreeMap::new(),
        dependency_artifacts: Vec::new(),
        host_artifacts: Vec::new(),
        build_script_facts: vec![RustBuildFact {
            name: "none".to_string(),
            value_digest_blake3: digest.to_string(),
        }],
        native_link_facts: Vec::new(),
        compiler_policy_digest_blake3: digest.to_string(),
    })
    .unwrap();
    std::fs::create_dir_all(rust_output_dir).unwrap();
    std::fs::write(rust_output_dir.join("libcrate.rlib"), RUST_CACHE_OUTPUT_BYTES).unwrap();
    std::fs::write(
        rust_output_dir.join(crunch_rust_cache::RUST_UNIT_EXECUTION_RECEIPT_FILE),
        b"mutable execution receipt",
    )
    .unwrap();
    let cache = crunch_rust_cache::RustCache::open_async(StoreConfig::new(
        crunch_store::StoreBackend::Casita,
        state_dir.to_path_buf(),
        output_dir.to_path_buf(),
        STORE_DIR.to_string(),
    ))
    .await
    .unwrap();
    let policy = LocalCachePolicy {
        reads_enabled: true,
        writes_enabled: true,
        ..LocalCachePolicy::default()
    };
    let receipt = format!("mantle-rust-receipt://blake3/{digest}");
    let published = cache
        .publish(PublishRequest {
            action: &action,
            output_dir: rust_output_dir,
            producer_receipt_ref: &receipt,
            policy: &policy,
        })
        .await
        .unwrap();
    assert_eq!(
        cache.retention().unwrap().retained_results.get(&published.result_ref),
        Some(&published.input.root_node)
    );
    assert_eq!(cache.plan_retention_gc_verified().await.unwrap().live_nodes().len(), 1);
    action
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
    const EXPECTED_DECLARATION_SCHEMA_VERSION: u64 = 3;
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

    assert!(!state_dir.path().join("gc-roots.json").exists());
    let archived: serde_json::Value =
        serde_json::from_slice(&std::fs::read(state_dir.path().join("gc-roots.migrated.json")).unwrap()).unwrap();
    assert_eq!(archived[&legacy_path]["source"], "build");
    let interest_files = std::fs::read_dir(state_dir.path().join("retention-interests"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(interest_files.len(), 1);
    let persisted: serde_json::Value =
        serde_json::from_slice(&std::fs::read(interest_files[0].path()).unwrap()).unwrap();
    assert_eq!(persisted["schema_version"], 1);
    assert_eq!(persisted["owner"], "legacy-unmanaged");
    assert_eq!(persisted["logical_path"], legacy_path);
    assert_eq!(persisted["reason"], "migrated-from-path-only-v1");
    assert_eq!(persisted["declaration"]["schema_version"], EXPECTED_DECLARATION_SCHEMA_VERSION);
    assert_eq!(persisted["declaration"]["root_class"], "legacy-unmanaged");
    assert_eq!(persisted["declaration"]["owner_scope"], "legacy-unmanaged");
    assert!(persisted["declaration"]["project_identity"].is_null());
    assert_eq!(persisted["declaration"]["last_transition_reason"], "migrated-from-path-only-v1");
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
        let kept = signed_ca_symlink_output(&store, "cli-kept", "kept-nar-content", &signer).await;
        let removed = signed_ca_symlink_output(&store, "cli-collected", "collected-nar-content", &signer).await;
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

    let rust_outputs = tempfile::tempdir().unwrap();
    let cache_action = runtime.block_on(seed_casita_rust_cache(
        state_dir.path(),
        output_dir.path(),
        &rust_outputs.path().join("unit"),
    ));
    let rust_cache_state = state_dir.path().join(crunch_rust_cache::RUST_CACHE_STATE_DIRECTORY);
    let retention_path = rust_cache_state.join("retained-nodes.json");
    let retained_before = std::fs::read(&retention_path).unwrap();
    let usage_output = casita_cmd().args(["--json", "store", "usage"]).output().unwrap();
    assert!(usage_output.status.success(), "{}", String::from_utf8_lossy(&usage_output.stderr));
    let usage: serde_json::Value = serde_json::from_slice(&usage_output.stdout).unwrap();
    assert_eq!(usage["schema"], "mantle-store-usage-v1");
    assert_eq!(std::fs::read(&retention_path).unwrap(), retained_before);
    let plan_output = casita_cmd().args(["--json", "store", "gc"]).output().unwrap();
    assert!(plan_output.status.success(), "{}", String::from_utf8_lossy(&plan_output.stderr));
    let plan: serde_json::Value = serde_json::from_slice(&plan_output.stdout).unwrap();
    assert_eq!(plan["is_dry_run"], true);
    assert_eq!(plan["retained_root_count"], 1);
    assert_eq!(plan["retained_castore_root_count"], 1);
    assert_eq!(plan["candidate_path_count"], 1);
    assert_eq!(plan["candidate_exported_output_count"], 1);
    assert_eq!(plan["candidate_paths"], serde_json::json!([removed_logical]));
    assert!(std::fs::symlink_metadata(&removed_export).is_ok(), "GC plan must not remove an export");

    let plan_id = plan["plan_id"].as_str().unwrap();
    let interest_path = std::fs::read_dir(state_dir.path().join("retention-interests"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let roots_before = std::fs::read(&interest_path).unwrap();
    let listed_before_output = casita_cmd().args(["--json", "store", "list"]).output().unwrap();
    assert!(listed_before_output.status.success(), "{}", String::from_utf8_lossy(&listed_before_output.stderr));
    let listed_before: serde_json::Value = serde_json::from_slice(&listed_before_output.stdout).unwrap();
    assert_eq!(listed_before["paths"].as_array().unwrap().len(), 2);
    assert_eq!(std::fs::read(&interest_path).unwrap(), roots_before);
    assert!(!state_dir.path().join("casita-gc-fence.json").exists());
    assert!(!state_dir.path().join("casita-gc-fence.progress").exists());
    assert_eq!(std::fs::read_link(&kept_export).unwrap(), Path::new("kept-nar-content"));
    assert_eq!(std::fs::read_link(&removed_export).unwrap(), Path::new("collected-nar-content"));
    let listed_after_output = casita_cmd().args(["--json", "store", "list"]).output().unwrap();
    assert!(listed_after_output.status.success(), "{}", String::from_utf8_lossy(&listed_after_output.stderr));
    let listed_after: serde_json::Value = serde_json::from_slice(&listed_after_output.stdout).unwrap();
    assert_eq!(listed_after["paths"], listed_before["paths"]);

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
    let post_gc_output = casita_cmd().args(["--json", "store", "gc"]).output().unwrap();
    assert!(post_gc_output.status.success(), "{}", String::from_utf8_lossy(&post_gc_output.stderr));
    let post_gc: serde_json::Value = serde_json::from_slice(&post_gc_output.stdout).unwrap();
    assert_eq!(post_gc["candidate_path_count"], 0);
    assert_eq!(post_gc["retained_root_count"], 1);
    assert_eq!(post_gc["retained_castore_root_count"], 1);
    assert_eq!(std::fs::read(&retention_path).unwrap(), retained_before);
    assert!(!state_dir.path().join("casita-gc-fence.json").exists());
    runtime.block_on(async {
        let reopened = crunch_rust_cache::RustCache::open_async(StoreConfig::new(
            crunch_store::StoreBackend::Casita,
            state_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            STORE_DIR.to_string(),
        ))
        .await
        .unwrap();
        let policy = crunch_rust_cache_core::LocalCachePolicy {
            reads_enabled: true,
            writes_enabled: true,
            ..crunch_rust_cache_core::LocalCachePolicy::default()
        };
        let restored_dir = rust_outputs.path().join("restored");
        let restored = reopened.restore(&cache_action, &restored_dir, &policy).await.unwrap();
        assert_eq!(restored.disposition, crunch_rust_cache::CACHE_DISPOSITION_HIT);
        assert!(!restored.compiler_executed);
        assert_eq!(std::fs::read(restored_dir.join("libcrate.rlib")).unwrap(), RUST_CACHE_OUTPUT_BYTES);
    });
    println!(
        "CASITA_GC_CLI_EVIDENCE {}",
        serde_json::json!({
            "plan_id": plan["plan_id"],
            "retained_root_count": plan["retained_root_count"],
            "rust_cache_gc_preflight": "verified-with-existing-cache-state",
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
fn casita_store_gc_missing_retained_cache_root_fails_before_removing_signed_output() {
    let state_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let rust_outputs = tempfile::tempdir().unwrap();
    let signing_name = "casita-gc-missing-cache";
    let raw_signer = ed25519_dalek::SigningKey::from_bytes(&[25_u8; 32]);
    let trusted_key =
        nix_compat::narinfo::VerifyingKey::new(signing_name.to_string(), raw_signer.verifying_key()).to_string();
    let signer = nix_compat::narinfo::SigningKey::new(signing_name.to_string(), raw_signer);
    let policy_bytes = format!("{trusted_key}\n");
    let policy_path = state_dir.path().join("casita-trusted-public-keys");
    std::fs::write(&policy_path, &policy_bytes).unwrap();
    let config = || {
        StoreConfig::new(
            crunch_store::StoreBackend::Casita,
            state_dir.path().to_path_buf(),
            output_dir.path().to_path_buf(),
            STORE_DIR.to_string(),
        )
    };
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let kept = runtime.block_on(async {
        let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir.path()).unwrap();
        let mut store = StoreHandle::open(config()).await.unwrap();
        let kept = signed_ca_symlink_output(&store, "missing-cache-kept", "kept-after-failure", &signer).await;
        store.pathinfo_service().put(kept.clone()).await.unwrap();
        assert_eq!(store.export_cached_path_info(&kept.store_path).await.unwrap(), Some(kept.clone()));
        kept
    });
    let casita_cmd = || {
        let mut cmd = crunch_cmd(state_dir.path(), output_dir.path());
        cmd.args(["--store-backend", "casita"]);
        cmd
    };
    casita_cmd().args(["store", "pin", &kept.store_path.to_absolute_path()]).assert().success();
    runtime.block_on(seed_casita_rust_cache(
        state_dir.path(),
        output_dir.path(),
        &rust_outputs.path().join("unit"),
    ));
    let retention_path = state_dir
        .path()
        .join(crunch_rust_cache::RUST_CACHE_STATE_DIRECTORY)
        .join("retained-nodes.json");
    let retained_before = std::fs::read(&retention_path).unwrap();

    // Deliberate test-only misuse: the raw guarded store GC omits the live Rust cache root.
    runtime.block_on(async {
        let guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir.path()).unwrap();
        let mut store = StoreHandle::open(config()).await.unwrap();
        store.recover_casita_gc_under_guard(&guard).await.unwrap();
        let plan = store.garbage_collect_with_castore_roots_under_guard(&guard, None, &[]).await.unwrap();
        let removed = store
            .garbage_collect_with_castore_roots_under_guard(&guard, Some(&plan.plan_id), &[])
            .await
            .unwrap();
        assert!(removed.execution_complete);
        assert!(removed.failed_operations.is_empty());
    });
    assert_eq!(std::fs::read(&retention_path).unwrap(), retained_before);
    assert!(!state_dir.path().join("casita-gc-fence.json").exists());

    // Publish a separate unpinned output AFTER removing the payload, so raw GC
    // cannot have removed the output whose survival the literal CLI plan proves.
    let candidate = runtime.block_on(async {
        let _guard = crunch_store::StoreMutationGuard::acquire_wait(state_dir.path()).unwrap();
        let mut store = StoreHandle::open(config()).await.unwrap();
        let candidate =
            signed_ca_symlink_output(&store, "missing-cache-candidate", "must-survive-failed-plan", &signer).await;
        store.pathinfo_service().put(candidate.clone()).await.unwrap();
        assert_eq!(
            store.export_cached_path_info(&candidate.store_path).await.unwrap(),
            Some(candidate.clone())
        );
        candidate
    });
    let kept_selector = kept.store_path.to_string();
    let candidate_selector = candidate.store_path.to_string();
    let kept_export = output_dir.path().join(&kept_selector);
    let candidate_export = output_dir.path().join(&candidate_selector);
    let kept_target = std::fs::read_link(&kept_export).unwrap();
    let candidate_target = std::fs::read_link(&candidate_export).unwrap();
    let read_info = |selector: &str| {
        let response = casita_cmd().args(["--json", "store", "info", selector]).output().unwrap();
        assert!(response.status.success(), "{}", String::from_utf8_lossy(&response.stderr));
        serde_json::from_slice::<serde_json::Value>(&response.stdout).unwrap()
    };
    let kept_info = read_info(&kept_selector);
    let candidate_info = read_info(&candidate_selector);
    assert_eq!(kept_info["paths"][0]["signatures"].as_array().unwrap().len(), 1);
    assert_eq!(candidate_info["paths"][0]["signatures"].as_array().unwrap().len(), 1);
    let repository_before = snapshot_state_tree(&state_dir.path().join("casita"));

    let failed = casita_cmd().args(["--json", "store", "gc"]).output().unwrap();
    assert!(!failed.status.success(), "missing retained cache payload unexpectedly permitted a GC plan");
    assert!(
        String::from_utf8_lossy(&failed.stderr).contains("casita-root-missing"),
        "{}",
        String::from_utf8_lossy(&failed.stderr)
    );
    assert_eq!(std::fs::read(&retention_path).unwrap(), retained_before);
    assert_eq!(std::fs::read(&policy_path).unwrap(), policy_bytes.as_bytes());
    assert!(!state_dir.path().join("casita-gc-fence.json").exists());
    assert!(!state_dir.path().join("casita-gc-fence.progress").exists());
    assert_eq!(snapshot_state_tree(&state_dir.path().join("casita")), repository_before);
    assert_eq!(std::fs::read_link(&kept_export).unwrap(), kept_target);
    assert_eq!(std::fs::read_link(&candidate_export).unwrap(), candidate_target);
    assert_eq!(read_info(&kept_selector), kept_info);
    assert_eq!(read_info(&candidate_selector), candidate_info);
}

#[cfg(unix)]
#[test]
fn casita_store_gc_without_cache_state_collects_only_unpinned_signed_output() {
    let state_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let signing_name = "casita-gc-no-cache";
    let raw_signer = ed25519_dalek::SigningKey::from_bytes(&[24_u8; 32]);
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
        let kept = signed_ca_symlink_output(&store, "no-cache-kept", "kept-no-cache-content", &signer).await;
        let removed = signed_ca_symlink_output(&store, "no-cache-collected", "removed-no-cache-content", &signer).await;
        store.pathinfo_service().put(kept.clone()).await.unwrap();
        store.pathinfo_service().put(removed.clone()).await.unwrap();
        assert_eq!(store.export_cached_path_info(&kept.store_path).await.unwrap(), Some(kept.clone()));
        assert_eq!(store.export_cached_path_info(&removed.store_path).await.unwrap(), Some(removed.clone()));
        (kept, removed)
    });
    let cache_state = state_dir.path().join(crunch_rust_cache::RUST_CACHE_STATE_DIRECTORY);
    let casita_cmd = || {
        let mut command = crunch_cmd(state_dir.path(), output_dir.path());
        command.args(["--store-backend", "casita"]);
        command
    };
    casita_cmd().args(["store", "pin", &kept.store_path.to_absolute_path()]).assert().success();
    assert!(!cache_state.exists(), "admitting and pinning outputs must not create Rust cache state");

    let planned = casita_cmd().args(["--json", "store", "gc"]).output().unwrap();
    assert!(planned.status.success(), "{}", String::from_utf8_lossy(&planned.stderr));
    let plan: serde_json::Value = serde_json::from_slice(&planned.stdout).unwrap();
    assert_eq!(plan["candidate_paths"], serde_json::json!([removed.store_path.to_absolute_path()]));
    assert_eq!(plan["retained_root_count"], 1);
    assert!(!cache_state.exists(), "planning output GC must not open Rust cache state");

    let executed = casita_cmd()
        .args([
            "--json",
            "store",
            "gc",
            "--execute",
            "--plan-id",
            plan["plan_id"].as_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(executed.status.success(), "{}", String::from_utf8_lossy(&executed.stderr));
    let execution: serde_json::Value = serde_json::from_slice(&executed.stdout).unwrap();
    assert_eq!(execution["execution_complete"], true);
    assert_eq!(execution["failed_operations"], serde_json::json!([]));
    assert_eq!(execution["candidate_paths"], plan["candidate_paths"]);
    assert!(execution["operations"].as_array().unwrap().contains(&serde_json::json!("casita-collection")));
    assert!(!cache_state.exists(), "executing output GC must not create Rust cache state");
    assert!(!state_dir.path().join("casita-gc-fence.json").exists());
    assert_eq!(
        std::fs::read_link(output_dir.path().join(kept.store_path.to_string())).unwrap(),
        Path::new("kept-no-cache-content")
    );
    assert_eq!(
        std::fs::symlink_metadata(output_dir.path().join(removed.store_path.to_string()))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::NotFound
    );
    let retained = casita_cmd().args(["--json", "store", "info", &kept.store_path.to_string()]).output().unwrap();
    assert!(retained.status.success(), "{}", String::from_utf8_lossy(&retained.stderr));
    let retained: serde_json::Value = serde_json::from_slice(&retained.stdout).unwrap();
    assert_eq!(retained["paths"][0]["nar_size"], kept.nar_size);
    assert_eq!(retained["paths"][0]["nar_sha256"], data_encoding::HEXLOWER.encode(kept.nar_sha256.as_ref()));
    casita_cmd()
        .args([
            "store",
            "verify",
            "--trusted-public-keys",
            &trusted_key,
            &kept.store_path.to_string(),
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("trusted_signatures=1/1"));
    casita_cmd()
        .args(["store", "info", &removed.store_path.to_string()])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no PathInfo matching"));
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
    assert_eq!(report["schema"], "mantle-store-info-v3");
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
        .stdout(predicates::str::contains("rust-unit-cache: true"))
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
    let before = std::fs::read_dir(state.path()).unwrap().count();
    let snapshot = snapshot_state_tree(state.path());
    let mismatch = crunch_cmd(state.path(), output_dir.path())
        .args(["--store-backend", "snix", "store", "list"])
        .output()
        .unwrap();
    assert!(!mismatch.status.success());
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("store-backend-mismatch"));
    assert_eq!(std::fs::read(&identity_path).unwrap(), identity);
    assert_eq!(std::fs::read_dir(state.path()).unwrap().count(), before);
    assert_eq!(snapshot_state_tree(state.path()), snapshot, "selected mismatch changed state");
    let dropped = crunch_cmd(state.path(), output_dir.path()).args(["store", "list"]).output().unwrap();
    assert!(!dropped.status.success(), "omitting the launcher backend opened a Casita state as Snix");
    assert!(String::from_utf8_lossy(&dropped.stderr).contains("store-backend-mismatch"));
    assert_eq!(snapshot_state_tree(state.path()), snapshot, "missing backend argument changed state");
    let ambient = crunch_cmd(state.path(), output_dir.path())
        .env("MANTLE_STORE_BACKEND", "casita")
        .args(["store", "list"])
        .output()
        .unwrap();
    assert!(!ambient.status.success(), "ambient selection overrode the CLI default");
    assert!(String::from_utf8_lossy(&ambient.stderr).contains("store-backend-mismatch"));
    assert_eq!(snapshot_state_tree(state.path()), snapshot, "ambient backend changed recorded state");
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
    let selected = crunch_cmd(state.path(), output_dir.path())
        .env_remove("CRUNCH_CONFIG_DIR")
        .args(["--store-backend", "snix", "store", "list"])
        .output()
        .unwrap();
    assert!(selected.status.success(), "{}", String::from_utf8_lossy(&selected.stderr));
    let before = snapshot_state_tree(state.path());
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
    assert!(!state.path().join("signing-key").exists());
    set_tree_read_only(base.path(), false);
}

#[test]
fn unknown_backend_is_rejected_before_creating_state() {
    let root = tempfile::tempdir().unwrap();
    let state_dir = root.path().join("absent-state");
    let output_dir = root.path().join("absent-store");
    let output = crunch_cmd(&state_dir, &output_dir)
        .args(["--store-backend", "tape", "store", "list"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("store-backend-unknown"));
    assert!(!state_dir.exists());
    assert!(!output_dir.exists());
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
}

#[test]
fn selected_backends_reopen_the_same_signed_ca_closure_and_plan_the_same_gc_candidate() {
    // The same explicitly provisioned test-only signer as the Run 25 Snix baseline.
    let signing_name = "mantle-baseline-fixture-1";
    let raw_signer = ed25519_dalek::SigningKey::from_bytes(&[0x2a_u8; 32]);
    let trusted_key =
        nix_compat::narinfo::VerifyingKey::new(signing_name.to_string(), raw_signer.verifying_key()).to_string();
    assert_eq!(trusted_key, "mantle-baseline-fixture-1:GX9rI+FshTLGq8g4+s1ep4m+DHaykgM0A5v6iz02jWE=");
    let keypair_bytes = raw_signer.to_keypair_bytes();
    let signer = nix_compat::narinfo::SigningKey::new(signing_name.to_string(), raw_signer);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let mut first_info: Option<serde_json::Value> = None;
    let mut first_archive: Option<Vec<u8>> = None;

    for backend in [crunch_store::StoreBackend::Snix, crunch_store::StoreBackend::Casita] {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("state");
        let output = root.path().join("output");
        std::fs::create_dir(&state).unwrap();
        if backend == crunch_store::StoreBackend::Casita {
            std::fs::write(state.join("casita-trusted-public-keys"), format!("{trusted_key}\n")).unwrap();
        }
        let path = runtime.block_on(async {
            let _guard = crunch_store::StoreMutationGuard::acquire_wait(&state).unwrap();
            let mut store =
                StoreHandle::open(StoreConfig::new(backend, state.clone(), output.clone(), STORE_DIR.to_string()))
                    .await
                    .unwrap();
            let info = signed_ca_symlink_output(&store, "backend-parity", "offline-fixture", &signer).await;
            store.pathinfo_service().put(info.clone()).await.unwrap();
            assert_eq!(store.export_cached_path_info(&info.store_path).await.unwrap(), Some(info.clone()));
            info.store_path
        });
        let selector = path.to_string();
        let command = || {
            let mut cmd = crunch_cmd(&state, &output);
            cmd.args(["--store-backend", backend.as_str()]);
            cmd
        };
        let identity_before = std::fs::read(state.join("store-identity.json")).unwrap();
        let info_output = command().args(["--json", "store", "info", &selector]).output().unwrap();
        assert!(info_output.status.success(), "{}", String::from_utf8_lossy(&info_output.stderr));
        let info: serde_json::Value = serde_json::from_slice(&info_output.stdout).unwrap();
        assert_eq!(info["backend"], backend.as_str());
        assert_eq!(info["schema"], "mantle-store-info-v3");
        let capabilities = info["backend_capabilities"]["core"].as_array().unwrap();
        for capability in [
            "output-admission",
            "lookup",
            "fresh-process-reopen",
            "closure-resolution",
            "export",
            "store-archive-export",
            "store-archive-import",
            "gc-plan",
            "plan-bound-gc",
            "action-result-pathinfo-output-reuse",
            "store-sign",
        ] {
            assert!(capabilities.iter().any(|entry| entry == capability), "{backend:?} omits {capability}");
        }
        assert_eq!(info["backend_capabilities"]["atomic_batch_import"], true);
        assert_eq!(info["backend_capabilities"]["rust_unit_cache"], true);
        if backend == crunch_store::StoreBackend::Casita {
            assert_eq!(info["backend_capabilities"]["max_root_changes"], 1024);
            assert_eq!(info["backend_capabilities"]["overlay_composition"], false);
            assert!(!capabilities.iter().any(|entry| entry == "store-repair-final-nar"));
            assert_eq!(info["backend_capabilities"]["unsigned_admission"], false);
        } else {
            assert!(info["backend_capabilities"]["max_root_changes"].is_null());
            assert_eq!(info["backend_capabilities"]["overlay_composition"], true);
            assert_eq!(info["backend_capabilities"]["unsigned_admission"], true);
            assert!(capabilities.iter().any(|entry| entry == "store-repair-final-nar"));
        }
        assert_eq!(info["paths"][0]["store_path"], selector);
        assert_eq!(info["paths"][0]["signatures"].as_array().unwrap().len(), 1);
        if let Some(previous) = &first_info {
            assert_eq!(&info["paths"], &previous["paths"], "selected backends must reopen equal signed PathInfo");
        } else {
            first_info = Some(info.clone());
        }
        let signing_key = root.path().join("signing-key");
        std::fs::write(
            &signing_key,
            format!("{signing_name}:{}\n", data_encoding::BASE64.encode(&keypair_bytes)),
        )
        .unwrap();
        // An in-place signature update must preserve the signed PathInfo facts.
        let signed = command()
            .args(["store", "sign", "--signing-key"])
            .arg(&signing_key)
            .arg(&selector)
            .output()
            .unwrap();
        assert!(signed.status.success(), "{}", String::from_utf8_lossy(&signed.stderr));
        assert!(String::from_utf8_lossy(&signed.stdout).contains(&format!("REPLACE {selector}")));
        let signed_info = command().args(["--json", "store", "info", &selector]).output().unwrap();
        assert!(signed_info.status.success(), "{}", String::from_utf8_lossy(&signed_info.stderr));
        let signed_info: serde_json::Value = serde_json::from_slice(&signed_info.stdout).unwrap();
        assert_eq!(signed_info["paths"], info["paths"]);

        let archive_path = root.path().join("closure.archive");
        let export_output = command()
            .args(["--json", "store", "archive", "export", "--all", "--to"])
            .arg(&archive_path)
            .output()
            .unwrap();
        assert!(export_output.status.success(), "{}", String::from_utf8_lossy(&export_output.stderr));
        let exported: serde_json::Value = serde_json::from_slice(&export_output.stdout).unwrap();
        assert_eq!(exported["exported_count"], 1);
        let archive = std::fs::read(archive_path).unwrap();
        if let Some(previous) = &first_archive {
            assert_eq!(&archive, previous, "selected backends must export byte-identical signed archives");
        }
        let import_state = root.path().join("import-state");
        let import_output = root.path().join("import-output");
        if backend == crunch_store::StoreBackend::Casita {
            std::fs::create_dir(&import_state).unwrap();
            std::fs::write(import_state.join("casita-trusted-public-keys"), format!("{trusted_key}\n")).unwrap();
        }
        let import_command = || {
            let mut cmd = crunch_cmd(&import_state, &import_output);
            cmd.args(["--store-backend", backend.as_str(), "--json"]);
            cmd
        };
        let imported = import_command()
            .args(["store", "archive", "import", "--from"])
            .arg(root.path().join("closure.archive"))
            .args(["--trusted-public-keys", &trusted_key])
            .output()
            .unwrap();
        assert!(imported.status.success(), "{}", String::from_utf8_lossy(&imported.stderr));
        let imported: serde_json::Value = serde_json::from_slice(&imported.stdout).unwrap();
        assert_eq!(imported["imported_count"], 1);
        assert_eq!(imported["paths"], exported["paths"]);
        assert_eq!(std::fs::read_link(import_output.join(&selector)).unwrap(), Path::new("offline-fixture"));
        let imported_info = import_command().args(["store", "info", &selector]).output().unwrap();
        assert!(imported_info.status.success(), "{}", String::from_utf8_lossy(&imported_info.stderr));
        let imported_info: serde_json::Value = serde_json::from_slice(&imported_info.stdout).unwrap();
        assert_eq!(imported_info["backend"], backend.as_str());
        let imported_identity: serde_json::Value =
            serde_json::from_slice(&std::fs::read(import_state.join("store-identity.json")).unwrap()).unwrap();
        assert_eq!(imported_identity["backend"], backend.as_str());
        for field in ["store_path", "nar_size", "nar_sha256", "node", "references", "signatures", "ca", "deriver"] {
            assert_eq!(imported_info["paths"][0][field], info["paths"][0][field], "imported {backend:?} {field}");
        }

        let plan_output = command().args(["--json", "store", "gc", "--dry-run"]).output().unwrap();
        assert!(plan_output.status.success(), "{}", String::from_utf8_lossy(&plan_output.stderr));
        let plan: serde_json::Value = serde_json::from_slice(&plan_output.stdout).unwrap();
        assert_eq!(plan["candidate_paths"], serde_json::json!([path.to_absolute_path()]));
        assert_eq!(plan["candidate_path_count"], 1);
        let stale_plan_id = plan["plan_id"].as_str().unwrap();
        command().args(["store", "pin", &path.to_absolute_path()]).assert().success();
        let rejected = command().args(["store", "gc", "--execute", "--plan-id", stale_plan_id]).output().unwrap();
        assert!(!rejected.status.success());
        let stale_blocker = if backend == crunch_store::StoreBackend::Casita {
            "gc-plan-stale"
        } else {
            "stale-gc-plan"
        };
        assert!(String::from_utf8_lossy(&rejected.stderr).contains(stale_blocker));
        assert!(std::fs::symlink_metadata(output.join(&selector)).is_ok(), "stale GC must preserve the output");
        assert_eq!(std::fs::read(state.join("store-identity.json")).unwrap(), identity_before);
        assert!(
            std::fs::symlink_metadata(output.join(&selector)).is_ok(),
            "dry run must preserve the physical export"
        );
        std::fs::remove_file(output.join(&selector)).unwrap();
        assert!(std::fs::symlink_metadata(output.join(&selector)).is_err());
        let restored_archive_path = root.path().join("closure-after-export-removal.archive");
        let reopened = command()
            .args(["--json", "store", "archive", "export", "--all", "--to"])
            .arg(&restored_archive_path)
            .output()
            .unwrap();
        assert!(reopened.status.success(), "{}", String::from_utf8_lossy(&reopened.stderr));
        assert_eq!(std::fs::read(restored_archive_path).unwrap(), archive);
        if first_archive.is_none() {
            first_archive = Some(archive);
        }
        assert_eq!(std::fs::read(state.join("store-identity.json")).unwrap(), identity_before);
        let other_backend = if backend == crunch_store::StoreBackend::Snix {
            crunch_store::StoreBackend::Casita
        } else {
            crunch_store::StoreBackend::Snix
        };
        let mismatched = crunch_cmd(&state, &output)
            .args(["--store-backend", other_backend.as_str(), "store", "list"])
            .output()
            .unwrap();
        assert!(!mismatched.status.success());
        assert!(String::from_utf8_lossy(&mismatched.stderr).contains(&format!(
            "store-backend-mismatch: requested {}, state declares {}",
            other_backend.as_str(),
            backend.as_str()
        )));
        assert_eq!(std::fs::read(state.join("store-identity.json")).unwrap(), identity_before);
    }
}

#[test]
fn selected_backend_states_preserve_frozen_hello_and_signed_facts_under_provisioned_fixture_keys() {
    if !Path::new("/nix/store").is_dir()
        || !std::process::Command::new("bwrap")
            .args(["--ro-bind", "/", "/", "--", "/bin/true"])
            .output()
            .is_ok_and(|probe| probe.status.success())
    {
        eprintln!("skipping: frozen backend conformance requires /nix/store and working bwrap");
        return;
    }

    // Runs 25-28: deliberately nonsecret test vectors, never production signing authorities.
    // Frozen Runs 25-28 use a logical /mantle/store prefix and an isolated,
    // explicitly provisioned test key. Physical output/state paths are not identities.
    const GOLDEN_FIRST_BUILD: &str = include_str!(
        "../.cairn/changes/add-store-backend-selection/evidence/baseline-snix-frozen-replay-first-build.json"
    );
    const GOLDEN_SECOND_BUILD: &str = include_str!(
        "../.cairn/changes/add-store-backend-selection/evidence/baseline-snix-frozen-replay-second-build.json"
    );
    const GOLDEN_INFO: &str =
        include_str!("../.cairn/changes/add-store-backend-selection/evidence/baseline-snix-frozen-info.json");
    const GOLDEN_LIST: &str =
        include_str!("../.cairn/changes/add-store-backend-selection/evidence/baseline-snix-frozen-list.json");
    const GOLDEN_ROOTS: &str =
        include_str!("../.cairn/changes/add-store-backend-selection/evidence/baseline-snix-frozen-roots.json");
    const GOLDEN_GC: &str = include_str!(
        "../.cairn/changes/add-store-backend-selection/evidence/baseline-snix-frozen-replay-first-gc.json"
    );
    const GOLDEN_SECOND_GC: &str = include_str!(
        "../.cairn/changes/add-store-backend-selection/evidence/baseline-snix-frozen-replay-second-gc.json"
    );
    const GOLDEN_ARCHIVE: &str =
        include_str!("../.cairn/changes/add-store-backend-selection/evidence/baseline-snix-frozen-archive-export.json");
    let golden_builds = [
        serde_json::from_str::<serde_json::Value>(GOLDEN_FIRST_BUILD).unwrap(),
        serde_json::from_str::<serde_json::Value>(GOLDEN_SECOND_BUILD).unwrap(),
    ];
    let golden_info: serde_json::Value = serde_json::from_str(GOLDEN_INFO).unwrap();
    let golden_list: serde_json::Value = serde_json::from_str(GOLDEN_LIST).unwrap();
    let golden_roots: serde_json::Value = serde_json::from_str(GOLDEN_ROOTS).unwrap();
    let golden_gc: serde_json::Value = serde_json::from_str(GOLDEN_GC).unwrap();
    let golden_second_gc: serde_json::Value = serde_json::from_str(GOLDEN_SECOND_GC).unwrap();
    let golden_archive: serde_json::Value = serde_json::from_str(GOLDEN_ARCHIVE).unwrap();
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("config");
    std::fs::create_dir(&config).unwrap();
    let mut observed = Vec::new();

    for (index, backend, seed, name, public_token) in [
        (0, crunch_store::StoreBackend::Snix, 0x2a, "mantle-baseline-fixture-1", "GX9rI+FshTLGq8g4+s1ep4m+DHaykgM0A5v6iz02jWE="),
        (1, crunch_store::StoreBackend::Snix, 0x2a, "mantle-baseline-fixture-1", "GX9rI+FshTLGq8g4+s1ep4m+DHaykgM0A5v6iz02jWE="),
        (2, crunch_store::StoreBackend::Snix, 0x2b, "mantle-secondary-fixture-1", "RQigeqlBcH8+stuUyIl6gLLBGXR2tt4hOsJz332GxP8="),
        (3, crunch_store::StoreBackend::Casita, 0x2a, "mantle-baseline-fixture-1", "GX9rI+FshTLGq8g4+s1ep4m+DHaykgM0A5v6iz02jWE="),
    ] {
        let state = root.path().join(format!("state-{index}"));
        let output = root.path().join(format!("store-{index}"));
        let key_file = root.path().join(format!("signing-key-{index}"));
        std::fs::create_dir(&output).unwrap();
        let raw_signer = ed25519_dalek::SigningKey::from_bytes(&[seed; 32]);
        let trusted_key = format!("{name}:{public_token}");
        assert_eq!(
            nix_compat::narinfo::VerifyingKey::new(name.to_string(), raw_signer.verifying_key()).to_string(),
            trusted_key
        );
        if backend == crunch_store::StoreBackend::Casita {
            std::fs::create_dir(&state).unwrap();
            std::fs::write(state.join("casita-trusted-public-keys"), format!("{trusted_key}\n")).unwrap();
        }
        {
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut key = options.open(&key_file).unwrap();
            std::io::Write::write_all(
                &mut key,
                format!("{name}:{}\n", data_encoding::BASE64.encode(&raw_signer.to_keypair_bytes())).as_bytes(),
            )
            .unwrap();
        }
        let command = || {
            let mut cmd = Command::cargo_bin("crunch").unwrap();
            cmd.current_dir(env!("CARGO_MANIFEST_DIR"))
                .arg("--state-dir")
                .arg(&state)
                .arg("--store")
                .arg(&output)
                .args(["--store-backend", backend.as_str(), "--json"])
                .env("CRUNCH_CONFIG_DIR", &config)
                .env("CRUNCH_NO_FUSE", "1")
                .env("SOURCE_DATE_EPOCH", "1790810000")
                .env("TZ", "UTC")
                .env("LANG", "C")
                .env("LC_ALL", "C");
            cmd
        };
        let built = command()
            .args(["build", "examples/hello.ncl", "--no-substitute", "--signing-key"])
            .arg(&key_file)
            .output()
            .unwrap();
        assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
        let build: serde_json::Value = serde_json::from_slice(&built.stdout).unwrap();
        let selector = golden_info["paths"][0]["store_path"].as_str().unwrap();
        let expected_build = &golden_builds[index.min(1)];
        assert_eq!(build["output_dir"], output.to_str().unwrap());
        assert_eq!(build["state_dir"], state.to_str().unwrap());
        if index < 2 || backend == crunch_store::StoreBackend::Casita {
            for field in [
                "schema",
                "file",
                "store_dir",
                "scheduler_policy",
                "hermeticity_mode",
                "hermeticity_audit_events",
                "build_environment_reports",
                "network_policy_reports",
                "workspace_reports",
                "action_result_reports",
                "native_dynamic_plans",
                "scheduler_priority_decisions",
                "overlay_base_generations",
                "store_layer_selections",
                "remote_telemetry_events",
                "frontend_artifact_attestations",
                "ast_grep_structural_evidence",
                "cargo_build_evidence",
                "cargo_build_evidence_diagnostics",
                "diagnostic_persistence_failures",
                "counts",
                "failed",
                "fod_mismatches",
            ] {
                assert_eq!(build[field], expected_build[field], "frozen build {index} field {field}");
            }
            for field in ["drv_key", "label", "cached"] {
                assert_eq!(build["outcomes"][0][field], expected_build["outcomes"][0][field], "{field}");
            }
            assert_eq!(build["outcomes"][0]["outputs"].as_array().unwrap().len(), 1);
            assert_eq!(
                build["outcomes"][0]["outputs"][0]["name"],
                expected_build["outcomes"][0]["outputs"][0]["name"]
            );
            assert_eq!(
                build["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"],
                expected_build["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"]
            );
            let physical = build["outcomes"][0]["outputs"][0]["path"].as_str().unwrap();
            assert_eq!(Path::new(physical), output.join(selector));
            let log = build["outcomes"][0]["log_file"].as_str().unwrap();
            assert_eq!(
                Path::new(log).strip_prefix(&state).unwrap(),
                Path::new(expected_build["outcomes"][0]["log_file"].as_str().unwrap())
                    .strip_prefix(expected_build["state_dir"].as_str().unwrap())
                    .unwrap()
            );
            let attestation = build["outcomes"][0]["outputs"][0]["artifact_attestation"]["path"]
                .as_str()
                .unwrap();
            if index < 2 {
                assert_eq!(
                    Path::new(attestation).strip_prefix(&state).unwrap(),
                    Path::new(expected_build["outcomes"][0]["outputs"][0]["artifact_attestation"]["path"].as_str().unwrap())
                        .strip_prefix(expected_build["state_dir"].as_str().unwrap())
                        .unwrap()
                );
            } else {
                assert!(
                    Path::new(attestation).starts_with(state.join("attestations/artifacts")),
                    "Casita attestation must stay in the selected state"
                );
                assert_eq!(Path::new(attestation).extension().unwrap(), "json");
            }
            assert!(Path::new(attestation).is_file());
            let diagnostics = &build["ast_grep_structural_evidence_diagnostics"];
            let frozen_diagnostics = &expected_build["ast_grep_structural_evidence_diagnostics"];
            assert_eq!(diagnostics.as_array().unwrap().len(), frozen_diagnostics.as_array().unwrap().len());
            for field in ["label", "output_name", "blocker_class", "next_action"] {
                assert_eq!(diagnostics[0][field], frozen_diagnostics[0][field], "structural diagnostic {field}");
            }
            assert_eq!(
                Path::new(diagnostics[0]["evidence_path"].as_str().unwrap()).strip_prefix(&output).unwrap(),
                Path::new(frozen_diagnostics[0]["evidence_path"].as_str().unwrap())
                    .strip_prefix(expected_build["output_dir"].as_str().unwrap())
                    .unwrap()
            );
            assert_eq!(
                diagnostics[0]["message"],
                frozen_diagnostics[0]["message"]
                    .as_str()
                    .unwrap()
                    .replace(expected_build["output_dir"].as_str().unwrap(), output.to_str().unwrap())
            );
        }
        assert_eq!(build["counts"]["built_total"], 1, "{build}");
        assert_eq!(build["counts"]["failed_total"], 0, "{build}");
        assert_eq!(
            build["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"],
            format!("/mantle/store/{selector}")
        );
        let inspected = command().args(["store", "info", selector]).output().unwrap();
        assert!(inspected.status.success(), "{}", String::from_utf8_lossy(&inspected.stderr));
        let info: serde_json::Value = serde_json::from_slice(&inspected.stdout).unwrap();
        assert_eq!(info["backend"], backend.as_str());
        assert_eq!(info["schema"], "mantle-store-info-v3");
        if index < 2 {
            for (field, expected) in golden_info["backend_capabilities"].as_object().unwrap() {
                assert_eq!(info["backend_capabilities"].get(field.as_str()).unwrap(), expected, "frozen profile field {field}");
            }
        } else if backend == crunch_store::StoreBackend::Casita {
            assert_eq!(info["backend_capabilities"]["max_root_changes"], 1024);
            assert_eq!(info["backend_capabilities"]["unsigned_admission"], false);
            assert_eq!(info["backend_capabilities"]["overlay_composition"], false);
            assert_eq!(info["backend_capabilities"]["rust_unit_cache"], true);
            assert_eq!(info["backend_capabilities"]["atomic_batch_import"], true);
            assert!(info["backend_capabilities"]["core"].as_array().unwrap().iter().any(|capability| capability == "action-result-pathinfo-output-reuse"));
        }
        assert_eq!(info["overlay"], golden_info["overlay"]);
        assert_eq!(info["paths"].as_array().unwrap().len(), 1);
        assert!(info["paths"][0]["retention_interests"].is_object());
        if index < 2 || backend == crunch_store::StoreBackend::Casita {
            // v3 adds retention_interests; compare only the v2-declared signed facts.
            for (field, expected) in golden_info["paths"][0].as_object().unwrap() {
                assert_eq!(info["paths"][0].get(field.as_str()).unwrap(), expected, "frozen signed PathInfo field {field}");
            }
        }

        let archive_path = root.path().join(format!("archive-{index}"));
        // The build itself published this canonical action result. Open a fresh
        // store port, rediscover the persisted record, and reuse its real PathInfo
        // and castore payload rather than constructing a record in the test.
        let action_ref = build["action_result_reports"][1]["action_ref"].as_str().unwrap();
        let result_ref = build["action_result_reports"][1]["publication_result_refs"][0].as_str().unwrap();
        let action_runtime = tokio::runtime::Runtime::new().unwrap();
        action_runtime.block_on(async {
            let store = StoreHandle::open(StoreConfig::new(
                backend,
                state.clone(),
                output.clone(),
                "/mantle/store".to_string(),
            ))
            .await
            .unwrap();
            let port = store.into_builder_store_parts().action_results;
            let discovered = port.discover(action_ref).await;
            assert!(discovered.diagnostics.is_empty(), "{:?}", discovered.diagnostics);
            let published = discovered
                .lookups
                .iter()
                .flat_map(|lookup| &lookup.records)
                .find(|candidate| candidate.record.result_ref == result_ref)
                .expect("build-published signed action result must survive fresh store reopen");
            assert_eq!(published.record.outputs[0].store_path, format!("/mantle/store/{selector}"));
            let reused = port.probe_outputs(&published.record).await.unwrap();
            let output_info = reused.outputs.get("out").expect("published output");
            assert_eq!(output_info.store_path.to_string(), selector);
            assert_eq!(reused.reused_nar_bytes, golden_info["paths"][0]["nar_size"].as_u64().unwrap());
            assert_eq!(reused.transferred_nar_bytes, 0);
            assert_eq!(
                output_info.signatures.iter().map(ToString::to_string).collect::<Vec<_>>(),
                info["paths"][0]["signatures"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|value| value.as_str().unwrap().to_string())
                    .collect::<Vec<_>>()
            );
            let empty_state = root.path().join(format!("unseeded-action-state-{index}"));
            if backend == crunch_store::StoreBackend::Casita {
                std::fs::create_dir(&empty_state).unwrap();
                std::fs::write(empty_state.join("casita-trusted-public-keys"), format!("{trusted_key}\n")).unwrap();
            }
            let empty = StoreHandle::open(StoreConfig::new(
                backend,
                empty_state,
                root.path().join(format!("unseeded-action-output-{index}")),
                "/mantle/store".to_string(),
            ))
            .await
            .unwrap();
            let empty_port = empty.into_builder_store_parts().action_results;
            let missing = empty_port.probe_outputs(&published.record).await.unwrap_err();
            assert!(missing.contains("action-result-output-pathinfo-missing"), "{missing}");
        });
        let exported =
            command().args(["store", "archive", "export", "--all", "--to"]).arg(&archive_path).output().unwrap();
        assert!(exported.status.success(), "{}", String::from_utf8_lossy(&exported.stderr));
        let archive_report: serde_json::Value = serde_json::from_slice(&exported.stdout).unwrap();
        assert_eq!(archive_report, golden_archive, "frozen NAR and archive-export semantics");
        // The frozen archive's PathInfo serializes /nix/store while this build signs
        // /mantle/store. Signed bytes cannot be equated across those prefixes.
        // Import into a separate real backend instead of asserting false byte equality.
        let imported_state = root.path().join(format!("import-state-{index}"));
        let imported_output = root.path().join(format!("import-store-{index}"));
        if backend == crunch_store::StoreBackend::Casita {
            std::fs::create_dir(&imported_state).unwrap();
            std::fs::write(imported_state.join("casita-trusted-public-keys"), format!("{trusted_key}\n")).unwrap();
        }
        let import_command = || {
            let mut cmd = Command::cargo_bin("crunch").unwrap();
            cmd.arg("--state-dir")
                .arg(&imported_state)
                .arg("--store")
                .arg(&imported_output)
                .args(["--store-backend", backend.as_str(), "--json"]);
            cmd
        };
        let imported = import_command()
            .args(["store", "archive", "import", "--from"])
            .arg(&archive_path)
            .args(["--trusted-public-keys", &trusted_key])
            .output()
            .unwrap();
        assert!(imported.status.success(), "{}", String::from_utf8_lossy(&imported.stderr));
        let import_report: serde_json::Value = serde_json::from_slice(&imported.stdout).unwrap();
        assert_eq!(import_report["imported_count"], 1);
        assert_eq!(import_report["paths"], archive_report["paths"]);
        assert_eq!(std::fs::read(imported_output.join(selector)).unwrap(), b"Hello, mantle!\n");
        let imported_info = import_command().args(["store", "info", selector]).output().unwrap();
        assert!(imported_info.status.success(), "{}", String::from_utf8_lossy(&imported_info.stderr));
        let imported_info: serde_json::Value = serde_json::from_slice(&imported_info.stdout).unwrap();
        assert_eq!(imported_info["backend"], backend.as_str());
        for field in ["store_path", "nar_size", "nar_sha256", "node", "references", "signatures", "ca", "deriver"] {
            assert_eq!(imported_info["paths"][0][field], info["paths"][0][field], "archive-import PathInfo {field}");
        }

        let listed = command().args(["store", "list"]).output().unwrap();
        assert!(listed.status.success(), "{}", String::from_utf8_lossy(&listed.stderr));
        let list: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
        assert_eq!(list, golden_list);
        let rooted = command().args(["store", "roots"]).output().unwrap();
        assert!(rooted.status.success(), "{}", String::from_utf8_lossy(&rooted.stderr));
        let roots: serde_json::Value = serde_json::from_slice(&rooted.stdout).unwrap();
        assert_eq!(roots["schema"], "mantle-store-roots-v1");
        assert_eq!(roots["roots"].as_array().unwrap().len(), 1);
        assert_eq!(roots["retention_interests"].as_array().unwrap().len(), 1);
        if index < 2 || backend == crunch_store::StoreBackend::Casita {
            // Registration reads the live clock, not SOURCE_DATE_EPOCH. Its
            // transition identity includes that time; only declared stable
            // root facts can be compared across state directories.
            for (field, expected) in golden_roots[0].as_object().unwrap() {
                if ["created_unix_s", "last_transition_id"].contains(&field.as_str()) {
                    continue;
                }
                assert_eq!(roots["roots"][0].get(field.as_str()).unwrap(), expected, "frozen root field {field}");
            }
            assert!(roots["roots"][0]["created_unix_s"].as_i64().unwrap() > 0);
            let transition_id = roots["roots"][0]["last_transition_id"].as_str().unwrap();
            let digest = transition_id.strip_prefix("b3:").expect("BLAKE3 root transition identity");
            assert!(
                digest.len() == 64 && digest.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
                "invalid root transition identity: {transition_id}"
            );
            let reopened_roots = command().args(["store", "roots"]).output().unwrap();
            assert!(reopened_roots.status.success(), "{}", String::from_utf8_lossy(&reopened_roots.stderr));
            let reopened_roots: serde_json::Value = serde_json::from_slice(&reopened_roots.stdout).unwrap();
            assert_eq!(roots, reopened_roots, "root timestamp and transition identity must survive same-state reopen");
        } else {
            assert_eq!(roots["roots"][0]["logical_path"], golden_roots[0]["logical_path"]);
            assert_eq!(roots["roots"][0]["source"], golden_roots[0]["source"]);
        }

        let verified = command()
            .args(["store", "verify", "--signing-key"])
            .arg(&key_file)
            .args(["--trusted-public-keys", &trusted_key, selector])
            .output()
            .unwrap();
        assert!(verified.status.success(), "{}", String::from_utf8_lossy(&verified.stderr));
        assert!(
            String::from_utf8_lossy(&verified.stdout).contains("trusted_signatures=1/1"),
            "own provisioned key must authenticate the stored NAR and PathInfo"
        );
        let dry_run = command().args(["store", "gc", "--dry-run"]).output().unwrap();
        assert!(dry_run.status.success(), "{}", String::from_utf8_lossy(&dry_run.stderr));
        let plan: serde_json::Value = serde_json::from_slice(&dry_run.stdout).unwrap();
        let expected_gc = if index == 1 { &golden_second_gc } else { &golden_gc };
        if index < 2 {
            // State placement and registration time affect the plan identities;
            // compare them only across repeated planning in this same state.
            for (field, expected) in expected_gc.as_object().unwrap() {
                if ["plan_id", "retention_plan_id", "reclaim_observations", "retention_explanations"]
                    .contains(&field.as_str())
                {
                    continue;
                }
                assert_eq!(plan.get(field.as_str()).unwrap(), expected, "frozen GC {index} field {field}");
            }
            assert_eq!(plan["reclaim_observations"].as_array().unwrap().len(), 1);
            for (field, expected) in expected_gc["reclaim_observations"][0].as_object().unwrap() {
                if field == "path" {
                    let observed_path = Path::new(plan["reclaim_observations"][0]["path"].as_str().unwrap());
                    let golden_path = Path::new(expected.as_str().unwrap());
                    assert_eq!(
                        observed_path.strip_prefix(&state).unwrap(),
                        golden_path.strip_prefix(expected_build["state_dir"].as_str().unwrap()).unwrap()
                    );
                } else {
                    assert_eq!(plan["reclaim_observations"][0].get(field.as_str()).unwrap(), expected, "GC reclaim {field}");
                }
            }
        }
        if index < 2 || backend == crunch_store::StoreBackend::Casita {
            let explanations = plan["retention_explanations"].as_array().unwrap();
            let frozen_explanations = expected_gc["retention_explanations"].as_array().unwrap();
            assert_eq!(explanations.len(), frozen_explanations.len());
            assert_eq!(explanations.len(), 1, "the retained built root must be explained");
            for (field, expected) in frozen_explanations[0].as_object().unwrap() {
                if field == "transition_id" {
                    continue;
                }
                assert_eq!(
                    explanations[0].get(field.as_str()).unwrap(),
                    expected,
                    "frozen retention explanation field {field}"
                );
            }
            assert_eq!(
                explanations[0]["transition_id"],
                roots["roots"][0]["last_transition_id"],
                "GC explanation must identify the retained root observed in this state"
            );
        }
        assert_eq!(plan["candidate_paths"], expected_gc["candidate_paths"]);
        assert_eq!(plan["retained_root_count"], expected_gc["retained_root_count"]);
        assert_eq!(plan["usage"], expected_gc["usage"], "root-level NAR usage must match the golden");
        let repeated = command().args(["store", "gc", "--dry-run"]).output().unwrap();
        assert!(repeated.status.success(), "{}", String::from_utf8_lossy(&repeated.stderr));
        let repeated_plan: serde_json::Value = serde_json::from_slice(&repeated.stdout).unwrap();
        assert_eq!(plan["plan_id"], repeated_plan["plan_id"]);
        assert_eq!(plan["retention_plan_id"], repeated_plan["retention_plan_id"]);
        let signed = command()
            .args(["store", "sign", "--signing-key"])
            .arg(&key_file)
            .arg(selector)
            .output()
            .unwrap();
        assert!(signed.status.success(), "{}", String::from_utf8_lossy(&signed.stderr));
        assert!(String::from_utf8_lossy(&signed.stdout).contains(&format!("REPLACE {selector}")));
        let after_sign = command().args(["store", "info", selector]).output().unwrap();
        assert!(after_sign.status.success(), "{}", String::from_utf8_lossy(&after_sign.stderr));
        let after_sign: serde_json::Value = serde_json::from_slice(&after_sign.stdout).unwrap();
        assert_eq!(after_sign["paths"][0]["signatures"], info["paths"][0]["signatures"]);
        let interests = &info["paths"][0]["retention_interests"];
        assert_eq!(interests, &roots["retention_interests"][0], "fresh roots and PathInfo must agree on interest identity");
        let facts = interests["interests"].as_array().unwrap();
        assert_eq!(facts.len(), 1);
        let record_id = facts[0]["record_id"].as_str().unwrap();
        assert!(
            record_id.len() == 64 && record_id.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
            "invalid persisted retention interest identity: {record_id}"
        );
        assert_eq!(
            after_sign["paths"][0]["retention_interests"],
            *interests,
            "signing must not alter the persisted interest identity"
        );
        observed.push(info["paths"][0].clone());
    }

    // Each independent state registers its own live-clock root declaration,
    // which gives its content-addressed interest record a different identity.
    // Preserve every other interest and PathInfo fact in cross-state checks.
    let mut comparable = observed.clone();
    for path in &mut comparable {
        for interest in path["retention_interests"]["interests"].as_array_mut().unwrap() {
            interest.as_object_mut().unwrap().remove("record_id").expect("persisted interest identity");
        }
    }
    assert_eq!(comparable[0], comparable[1], "one provisioned signer reproduces every stable signed fact");
    assert_ne!(comparable[0]["signatures"], comparable[2]["signatures"]);
    assert_eq!(comparable[0], comparable[3], "both backends preserve signed hello under one provisioned key");
    comparable[0].as_object_mut().unwrap().remove("signatures");
    comparable[2].as_object_mut().unwrap().remove("signatures");
    assert_eq!(comparable[0], comparable[2], "different signers preserve every stable unsigned PathInfo field");
}
