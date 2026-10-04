use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use crunch_store::StoreConfig;
use crunch_store::StoreFallbackMode;
use crunch_store::StoreHandle;
use nix_compat::store_path::StorePath;
use nix_compat::store_path::StorePathRef;
use predicates::prelude::*;
use serde_json::Value;
use sha2::Digest;
use snix_castore::Node;
use snix_store::nar::write_nar;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use tempfile::TempDir;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

const TEST_STORE_PREFIX: &str = "/mantle/store";
const TEST_KEYPAIR: &str =
    "archive-cli-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
const STORE_PATH_DIGEST_BYTES: usize = 20;
const TEST_NAR_PIPE_BYTES: usize = 65_536;
const NARIO_WIRE_WORD_BYTES: usize = 8;
const NARIO_TRUNCATION_BYTES: usize = NARIO_WIRE_WORD_BYTES + 1;

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn run_async<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Runtime::new().unwrap().block_on(future)
}

async fn open_store(output_dir: &Path, state_dir: &Path) -> StoreHandle {
    StoreHandle::open(StoreConfig {
        backend: crunch_store::StoreBackend::Snix,
        state_dir: state_dir.to_path_buf(),
        output_dir: output_dir.to_path_buf(),
        remote_cache_urls: Vec::new(),
        base_state_dirs: Vec::new(),
        fallback_mode: StoreFallbackMode::Practical,
        store_dir: TEST_STORE_PREFIX.to_string(),
    })
    .await
    .unwrap()
}

async fn seed_signed_file(output_dir: &Path, state_dir: &Path, name: &str, content: &[u8], signed: bool) -> PathInfo {
    let store = open_store(output_dir, state_dir).await;
    let path_info = signed_file_pathinfo(&store, name, content, signed).await;
    store.pathinfo_service().put(path_info.clone()).await.unwrap();
    path_info
}

async fn signed_file_pathinfo(store: &StoreHandle, name: &str, content: &[u8], signed: bool) -> PathInfo {
    let mut writer = store.blob_service().open_write().await;
    writer.write_all(content).await.unwrap();
    let blob_digest = writer.close().await.unwrap();
    let node = Node::File {
        digest: blob_digest,
        size: content.len() as u64,
        executable: false,
    };
    let nar_bytes = render_nar_bytes(store, &node).await;
    let nar_sha256: [u8; 32] = sha2::Sha256::digest(&nar_bytes).into();
    let mut path_info = PathInfo {
        store_path: StorePath::from_name_and_digest_fixed(name, digest_from_name(name)).unwrap(),
        node,
        references: vec![],
        nar_size: nar_bytes.len() as u64,
        nar_sha256,
        signatures: vec![],
        deriver: None,
        ca: None,
    };
    if signed {
        sign_pathinfo(&mut path_info);
    }
    path_info
}

async fn render_nar_bytes(store: &StoreHandle, node: &Node) -> Vec<u8> {
    let (mut reader, writer) = tokio::io::duplex(TEST_NAR_PIPE_BYTES);
    let blob_service = store.blob_service();
    let directory_service = store.directory_service();
    let node = node.clone();
    let write_task = tokio::spawn(async move { write_nar(writer, &node, blob_service, directory_service).await });
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await.unwrap();
    write_task.await.unwrap().unwrap();
    buf
}

fn digest_from_name(name: &str) -> [u8; STORE_PATH_DIGEST_BYTES] {
    let mut digest = [0u8; STORE_PATH_DIGEST_BYTES];
    for (index, byte) in name.as_bytes().iter().enumerate().take(STORE_PATH_DIGEST_BYTES) {
        digest[index] = *byte;
    }
    digest
}

fn trusted_public_key() -> String {
    let (_, verifying_key) = nix_compat::narinfo::parse_keypair(TEST_KEYPAIR).unwrap();
    verifying_key.to_string()
}

fn other_trusted_public_key(name: &str) -> String {
    let signer = ed25519_dalek::SigningKey::from_bytes(&[19_u8; 32]);
    nix_compat::narinfo::VerifyingKey::new(name.to_string(), signer.verifying_key()).to_string()
}

fn sign_pathinfo(path_info: &mut PathInfo) {
    let (signing_key, _) = nix_compat::narinfo::parse_keypair(TEST_KEYPAIR).unwrap();
    let store_path_ref: StorePathRef = path_info.store_path.as_ref();
    let refs: Vec<StorePathRef> = path_info.references.iter().map(StorePath::as_ref).collect();
    let fp = nix_compat::narinfo::fingerprint_with_store_dir(
        &store_path_ref,
        &path_info.nar_sha256,
        path_info.nar_size,
        refs.iter(),
        TEST_STORE_PREFIX,
    );
    path_info.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());
}

fn temp_path(root: &TempDir, name: &str) -> PathBuf {
    root.path().join(name)
}

fn state_files(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut files = std::collections::BTreeMap::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if entry.file_type().unwrap().is_dir() {
                directories.push(path);
            } else {
                files.insert(path.strip_prefix(root).unwrap().to_path_buf(), std::fs::read(path).unwrap());
            }
        }
    }
    files
}

fn mantle_state_files(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    state_files(root)
        .into_iter()
        .filter(|(path, _)| !path.starts_with("casita") && path != Path::new("store-mutation.lock"))
        .collect()
}

#[test]
fn casita_rejects_unsigned_archive_before_creating_state() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let store = root.path().join("store");
    let source = root.path().join("missing.mnar");
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&state)
        .arg("--store")
        .arg(&store)
        .args(["store", "archive", "import", "--from"])
        .arg(&source)
        .arg("--trust-unsigned")
        .assert()
        .failure()
        .stderr(predicate::str::contains("casita-trust-unsigned-unsupported"));
    assert!(!state.exists());
    assert!(!store.exists());
}

#[test]
fn casita_rejects_base_store_and_nario_unsigned_before_state_access() {
    let root = tempfile::tempdir().unwrap();
    let state = root.path().join("state");
    let store = root.path().join("store");
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&state)
        .arg("--store")
        .arg(&store)
        .arg("--base-store")
        .arg(root.path().join("missing-base"))
        .args(["store", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("casita-overlay-unsupported"));
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&state)
        .arg("--store")
        .arg(&store)
        .args(["foreign-import", "prepare-sources", "--plan"])
        .arg(root.path().join("missing-plan.json"))
        .arg("--out")
        .arg(root.path().join("sources.json"))
        .arg("--nario-trust-unsigned")
        .assert()
        .failure()
        .stderr(predicate::str::contains("casita-trust-unsigned-unsupported"));
    assert!(!state.exists());
    assert!(!store.exists());
}

#[test]
fn casita_archive_import_rejects_unprovisioned_keys_and_unnamed_signer_without_mutation() {
    let root = tempfile::tempdir().unwrap();
    let source_state = temp_path(&root, "source-state");
    let source_store = temp_path(&root, "source-store");
    let archive = temp_path(&root, "signed.mnar");
    let path_info = run_async(seed_signed_file(
        &source_store,
        &source_state,
        "policy-root",
        b"preprovisioned destination trust",
        true,
    ));
    mantle_cmd()
        .args(["--store-backend", "snix", "--state-dir"])
        .arg(&source_state)
        .arg("--store")
        .arg(&source_store)
        .args(["store", "archive", "export", "--to"])
        .arg(&archive)
        .arg(path_info.store_path.name())
        .assert()
        .success();
    let unsigned_archive = temp_path(&root, "unsigned.mnar");
    let unsigned_path_info = run_async(seed_signed_file(
        &source_store,
        &source_state,
        "unsigned-policy-root",
        b"unsigned archive record",
        false,
    ));
    mantle_cmd()
        .args(["--store-backend", "snix", "--state-dir"])
        .arg(&source_state)
        .arg("--store")
        .arg(&source_store)
        .args(["store", "archive", "export", "--to"])
        .arg(&unsigned_archive)
        .arg("--trust-unsigned")
        .arg(unsigned_path_info.store_path.name())
        .assert()
        .success();
    let source_files_before = state_files(&source_state);
    let original_key = trusted_public_key();
    let other_name_key = other_trusted_public_key("archive-cli-2");
    let same_name_other_key = other_trusted_public_key("archive-cli-1");
    for (case, archive_path, provisioned_key, requested_key, error) in [
        ("missing-policy", &archive, None, &original_key, "casita-trust-policy-missing"),
        (
            "key-not-provisioned",
            &archive,
            Some(&other_name_key),
            &original_key,
            "casita-import-key-unauthorized",
        ),
        (
            "same-name-different-bytes",
            &archive,
            Some(&same_name_other_key),
            &original_key,
            "casita-import-key-unauthorized",
        ),
        ("archive-signer-not-named", &archive, Some(&other_name_key), &other_name_key, "untrusted-signature"),
        ("unsigned-record", &unsigned_archive, Some(&original_key), &original_key, "casita-signer-untrusted"),
    ] {
        let state = temp_path(&root, &format!("target-state-{case}"));
        let store = temp_path(&root, &format!("target-store-{case}"));
        std::fs::create_dir_all(&store).unwrap();
        mantle_cmd()
            .args(["--store-backend", "casita", "--state-dir"])
            .arg(&state)
            .arg("--store")
            .arg(&store)
            .args(["store", "list"])
            .assert()
            .success();
        let policy_path = state.join("casita-trusted-public-keys");
        let expected_policy = provisioned_key.map(|key| format!("{key}\n").into_bytes());
        if let Some(bytes) = &expected_policy {
            std::fs::write(&policy_path, bytes).unwrap();
        }
        let state_before = mantle_state_files(&state);
        let store_before = state_files(&store);
        mantle_cmd()
            .args(["--store-backend", "casita", "--state-dir"])
            .arg(&state)
            .arg("--store")
            .arg(&store)
            .args(["store", "archive", "import", "--from"])
            .arg(archive_path)
            .args(["--trusted-public-keys", requested_key])
            .assert()
            .failure()
            .stderr(predicate::str::contains(error));
        let state_after = mantle_state_files(&state);
        assert_eq!(state_after.keys().collect::<Vec<_>>(), state_before.keys().collect::<Vec<_>>(), "{case}");
        for (path, bytes) in &state_before {
            assert_eq!(
                blake3::hash(&state_after[path]),
                blake3::hash(bytes),
                "{case} mutated Mantle-owned state file {}",
                path.display()
            );
        }
        assert_eq!(state_files(&store), store_before, "{case} materialized an output");
        assert_eq!(std::fs::read(&policy_path).ok(), expected_policy, "{case} changed destination policy");
        assert_eq!(state_files(&source_state), source_files_before, "{case} modified source state");
        let listed = mantle_cmd()
            .args(["--store-backend", "casita", "--state-dir"])
            .arg(&state)
            .arg("--store")
            .arg(&store)
            .args(["--json", "store", "list"])
            .output()
            .unwrap();
        assert!(listed.status.success(), "{case}: {}", String::from_utf8_lossy(&listed.stderr));
        assert!(serde_json::from_slice::<Value>(&listed.stdout).unwrap()["paths"].as_array().unwrap().is_empty());
    }
}

#[test]
fn snix_to_casita_cli_migration_pins_root_without_modifying_source() {
    let root = tempfile::tempdir().unwrap();
    let source_state = temp_path(&root, "source-state");
    let source_store = temp_path(&root, "source-store");
    let target_state = temp_path(&root, "target-state");
    let target_store = temp_path(&root, "target-store");
    let archive = temp_path(&root, "closure.mnar");
    let reexport = temp_path(&root, "casita-closure.mnar");
    let path_info = run_async(seed_signed_file(&source_store, &source_state, "migrated-root", b"durable casita", true));
    let selector = path_info.store_path.name();
    mantle_cmd()
        .args(["--store-backend", "snix", "--state-dir"])
        .arg(&source_state)
        .arg("--store")
        .arg(&source_store)
        .args(["store", "archive", "export", "--to"])
        .arg(&archive)
        .arg(selector)
        .assert()
        .success();
    let source_files = state_files(&source_state);
    std::fs::create_dir_all(&target_state).unwrap();
    let trusted_key = trusted_public_key();
    let destination_policy = target_state.join("casita-trusted-public-keys");
    let destination_policy_bytes = format!("{trusted_key}\n");
    std::fs::write(&destination_policy, &destination_policy_bytes).unwrap();
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "archive", "import", "--from"])
        .arg(&archive)
        .args(["--trusted-public-keys", &trusted_key])
        .assert()
        .success();
    let logical_path = format!("{TEST_STORE_PREFIX}/{}", path_info.store_path);
    let info_output = mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["--json", "store", "info", selector])
        .output()
        .unwrap();
    assert!(info_output.status.success(), "{}", String::from_utf8_lossy(&info_output.stderr));
    let info: Value = serde_json::from_slice(&info_output.stdout).unwrap();
    assert_eq!(info["backend"], "casita");
    assert_eq!(info["backend_capabilities"]["rust_unit_cache"], false);
    assert_eq!(info["backend_capabilities"]["unsigned_admission"], false);
    assert_eq!(info["backend_capabilities"]["max_root_changes"], 1024);
    assert_eq!(
        info["paths"][0]["signatures"],
        serde_json::json!(path_info.signatures.iter().map(ToString::to_string).collect::<Vec<_>>())
    );
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "info", selector])
        .assert()
        .success()
        .stdout(predicate::str::contains("rust-unit-cache: false"))
        .stdout(predicate::str::contains("unsigned-admission: false"))
        .stdout(predicate::str::contains("max-root-changes: 1024"));
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "pin", &logical_path])
        .assert()
        .success()
        .stdout(predicate::str::contains("PINNED"));
    let roots = mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "roots"])
        .output()
        .unwrap();
    assert!(roots.status.success(), "{}", String::from_utf8_lossy(&roots.stderr));
    assert!(String::from_utf8_lossy(&roots.stdout).contains(&logical_path));
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "verify", "--trusted-public-keys", &trusted_key, selector])
        .assert()
        .success()
        .stdout(predicate::str::contains("trusted_signatures=1/1"));
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args([
            "store",
            "verify",
            "--trusted-public-keys",
            &other_trusted_public_key("archive-cli-2"),
            selector,
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("UNTRUSTED "))
        .stdout(predicate::str::contains("trusted_signatures=0/1"));
    assert_eq!(std::fs::read(&destination_policy).unwrap(), destination_policy_bytes.as_bytes());
    assert!(!target_state.join("signing-key").exists(), "Casita verification must not mint a local signer");
    let run_gc = || {
        let usage_output = mantle_cmd()
            .args(["--store-backend", "casita", "--state-dir"])
            .arg(&target_state)
            .arg("--store")
            .arg(&target_store)
            .args(["--json", "store", "usage"])
            .output()
            .unwrap();
        assert!(usage_output.status.success(), "{}", String::from_utf8_lossy(&usage_output.stderr));
        let usage: Value = serde_json::from_slice(&usage_output.stdout).unwrap();
        let plan_output = mantle_cmd()
            .args(["--store-backend", "casita", "--state-dir"])
            .arg(&target_state)
            .arg("--store")
            .arg(&target_store)
            .args(["--json", "store", "gc"])
            .output()
            .unwrap();
        assert!(plan_output.status.success(), "{}", String::from_utf8_lossy(&plan_output.stderr));
        let plan: Value = serde_json::from_slice(&plan_output.stdout).unwrap();
        assert_eq!(usage["usage"], plan["usage"]);
        assert_eq!(usage["reclaim_observations"], plan["reclaim_observations"]);
        let plan_id = plan["plan_id"].as_str().unwrap();
        let execution = mantle_cmd()
            .args(["--store-backend", "casita", "--state-dir"])
            .arg(&target_state)
            .arg("--store")
            .arg(&target_store)
            .args(["--json", "store", "gc", "--execute", "--plan-id", plan_id])
            .output()
            .unwrap();
        assert!(execution.status.success(), "{}", String::from_utf8_lossy(&execution.stderr));
        let report: Value = serde_json::from_slice(&execution.stdout).unwrap();
        assert_eq!(report["execution_complete"], true);
        plan
    };
    let pinned_plan = run_gc();
    let fence_path = target_state.join("casita-gc-fence.json");
    std::fs::write(&fence_path, br#"{"plan_id":"interrupted-plan","entries":[]}"#).unwrap();
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["--json", "store", "usage"])
        .assert()
        .success();
    assert!(!fence_path.exists(), "guarded usage must recover a pending GC fence");
    assert_eq!(pinned_plan["candidate_path_count"], 0);
    let cache_dir = target_state.join(crunch_rust_cache::RUST_CACHE_STATE_DIRECTORY);
    std::fs::create_dir(&cache_dir).unwrap();
    let retained = crunch_rust_cache::RustCacheRetention {
        schema: crunch_rust_cache::RUST_CACHE_RETENTION_SCHEMA.to_string(),
        retained_results: std::collections::BTreeMap::from([(
            format!("mantle-rust-result://blake3/{}", blake3::hash(b"retained-result").to_hex()),
            crunch_rust_cache_core::CastoreNodeIdentity {
                kind: crunch_rust_cache_core::CastoreNodeKind::File,
                digest_blake3: blake3::hash(b"durable casita").to_hex().to_string(),
                size_bytes: b"durable casita".len() as u64,
            },
        )]),
    };
    std::fs::write(cache_dir.join("retention.json"), serde_json::to_vec(&retained).unwrap()).unwrap();
    let before_refusal = state_files(&target_state);
    for action in ["usage", "gc"] {
        mantle_cmd()
            .args(["--store-backend", "casita", "--state-dir"])
            .arg(&target_state)
            .arg("--store")
            .arg(&target_store)
            .args(["store", action])
            .assert()
            .failure()
            .stderr(predicate::str::contains("casita-rust-cache-unsupported"));
    }
    assert_eq!(state_files(&target_state), before_refusal);
    std::fs::remove_dir_all(cache_dir).unwrap();
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "archive", "export", "--to"])
        .arg(&reexport)
        .arg(selector)
        .assert()
        .success();
    assert_eq!(std::fs::read(&reexport).unwrap(), std::fs::read(&archive).unwrap());
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "unpin", &logical_path])
        .assert()
        .success();
    let stale_plan_output = mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["--json", "store", "gc"])
        .output()
        .unwrap();
    assert!(stale_plan_output.status.success(), "{}", String::from_utf8_lossy(&stale_plan_output.stderr));
    let stale_plan: Value = serde_json::from_slice(&stale_plan_output.stdout).unwrap();
    assert_eq!(stale_plan["candidate_path_count"], 1);
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "pin", &logical_path])
        .assert()
        .success();
    let root_registry_before_stale_execution = std::fs::read(target_state.join("gc-roots.json")).unwrap();
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args([
            "store",
            "gc",
            "--execute",
            "--plan-id",
            stale_plan["plan_id"].as_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("gc-plan-stale"));
    assert_eq!(std::fs::read(target_state.join("gc-roots.json")).unwrap(), root_registry_before_stale_execution);
    assert!(!fence_path.exists(), "stale GC plan must not start a deletion fence");
    assert!(
        !target_state.join("casita-gc-fence.progress").exists(),
        "stale GC plan must not record deletion progress"
    );
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "info", selector])
        .assert()
        .success();
    let preserved_archive = temp_path(&root, "preserved-after-stale.mnar");
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "archive", "export", "--to"])
        .arg(&preserved_archive)
        .arg(selector)
        .assert()
        .success();
    assert_eq!(std::fs::read(&preserved_archive).unwrap(), std::fs::read(&archive).unwrap());
    mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "unpin", &logical_path])
        .assert()
        .success();
    let unpinned_plan = run_gc();
    assert_eq!(unpinned_plan["candidate_path_count"], 1);
    let listed = mantle_cmd()
        .args(["--store-backend", "casita", "--state-dir"])
        .arg(&target_state)
        .arg("--store")
        .arg(&target_store)
        .args(["store", "list"])
        .output()
        .unwrap();
    assert!(listed.status.success(), "{}", String::from_utf8_lossy(&listed.stderr));
    assert!(!String::from_utf8_lossy(&listed.stdout).contains(&logical_path));
    assert_eq!(state_files(&source_state), source_files);
    assert_eq!(std::fs::read(&destination_policy).unwrap(), destination_policy_bytes.as_bytes());
}

#[test]
fn store_archive_cli_exports_lists_json_and_imports_from_file() {
    let temp = tempfile::tempdir().unwrap();
    let source_store = temp_path(&temp, "source-store");
    let source_state = temp_path(&temp, "source-state");
    let dest_store = temp_path(&temp, "dest-store");
    let dest_state = temp_path(&temp, "dest-state");
    let archive_path = temp_path(&temp, "bundle.mnar");
    let content = b"cli archive payload\n";
    let path_info = run_async(seed_signed_file(&source_store, &source_state, "cli-root", content, true));
    let trusted_key = trusted_public_key();

    mantle_cmd()
        .arg("--store")
        .arg(&source_store)
        .arg("--state-dir")
        .arg(&source_state)
        .arg("store")
        .arg("archive")
        .arg("export")
        .arg("--to")
        .arg(&archive_path)
        .arg(path_info.store_path.name())
        .assert()
        .success()
        .stdout(predicate::str::contains("ARCHIVE_EXPORT"))
        .stderr(predicate::str::contains("exported=1"));

    mantle_cmd()
        .arg("store")
        .arg("archive")
        .arg("list")
        .arg("--from")
        .arg(&archive_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("mantle-store-archive-v1"))
        .stdout(predicate::str::contains("nario-v2 byte compatibility unproven"))
        .stdout(predicate::str::contains("ARCHIVE_PATH"));

    let json_list = mantle_cmd()
        .arg("--json")
        .arg("store")
        .arg("archive")
        .arg("list")
        .arg("--from")
        .arg(&archive_path)
        .output()
        .unwrap();
    assert!(json_list.status.success(), "{}", String::from_utf8_lossy(&json_list.stderr));
    let list_value: Value = serde_json::from_slice(&json_list.stdout).unwrap();
    assert_eq!(list_value["compatibility"], "mantle-native; nario-v2 byte compatibility unproven");
    assert_eq!(list_value["paths"][0]["store_path"], path_info.store_path.to_string());

    mantle_cmd()
        .arg("--store")
        .arg(&dest_store)
        .arg("--state-dir")
        .arg(&dest_state)
        .arg("store")
        .arg("archive")
        .arg("import")
        .arg("--from")
        .arg(&archive_path)
        .arg("--trusted-public-keys")
        .arg(&trusted_key)
        .assert()
        .success()
        .stdout(predicate::str::contains("ARCHIVE_IMPORT"))
        .stderr(predicate::str::contains("imported=1"));
    let materialized = dest_store.join(path_info.store_path.to_string());
    assert_eq!(std::fs::read(materialized).unwrap(), content);

    let json_import = mantle_cmd()
        .arg("--json")
        .arg("--store")
        .arg(&dest_store)
        .arg("--state-dir")
        .arg(&dest_state)
        .arg("store")
        .arg("archive")
        .arg("import")
        .arg("--from")
        .arg(&archive_path)
        .arg("--trusted-public-keys")
        .arg(&trusted_key)
        .output()
        .unwrap();
    assert!(json_import.status.success(), "{}", String::from_utf8_lossy(&json_import.stderr));
    let import_value: Value = serde_json::from_slice(&json_import.stdout).unwrap();
    assert_eq!(import_value["imported_count"], 0);
    assert_eq!(import_value["skipped_already_present_count"], 1);
}

#[test]
fn store_archive_cli_streams_stdout_and_stdin() {
    let temp = tempfile::tempdir().unwrap();
    let source_store = temp_path(&temp, "source-store");
    let source_state = temp_path(&temp, "source-state");
    let dest_store = temp_path(&temp, "dest-store");
    let dest_state = temp_path(&temp, "dest-state");
    let path_info = run_async(seed_signed_file(&source_store, &source_state, "stream-root", b"stream payload", true));
    let trusted_key = trusted_public_key();

    let export_output = mantle_cmd()
        .arg("--store")
        .arg(&source_store)
        .arg("--state-dir")
        .arg(&source_state)
        .arg("store")
        .arg("archive")
        .arg("export")
        .arg("--to")
        .arg("-")
        .arg(path_info.store_path.name())
        .output()
        .unwrap();
    assert!(export_output.status.success(), "{}", String::from_utf8_lossy(&export_output.stderr));
    assert!(export_output.stdout.starts_with(b"mantle-store-archive-v1\n"));
    assert!(String::from_utf8_lossy(&export_output.stderr).contains("exported=1"));

    let list_output = mantle_cmd()
        .arg("--json")
        .arg("store")
        .arg("archive")
        .arg("list")
        .arg("--from")
        .arg("-")
        .write_stdin(export_output.stdout.clone())
        .output()
        .unwrap();
    assert!(list_output.status.success(), "{}", String::from_utf8_lossy(&list_output.stderr));
    let list_value: Value = serde_json::from_slice(&list_output.stdout).unwrap();
    assert_eq!(list_value["paths"][0]["store_path"], path_info.store_path.to_string());

    mantle_cmd()
        .arg("--store")
        .arg(&dest_store)
        .arg("--state-dir")
        .arg(&dest_state)
        .arg("store")
        .arg("archive")
        .arg("import")
        .arg("--from")
        .arg("-")
        .arg("--trusted-public-keys")
        .arg(&trusted_key)
        .arg("--no-materialize")
        .write_stdin(export_output.stdout)
        .assert()
        .success()
        .stdout(predicate::str::contains("ARCHIVE_IMPORT"))
        .stderr(predicate::str::contains("imported=1"));
}

#[test]
fn store_archive_cli_requires_unsigned_escape_hatch() {
    let temp = tempfile::tempdir().unwrap();
    let source_store = temp_path(&temp, "source-store");
    let source_state = temp_path(&temp, "source-state");
    let archive_path = temp_path(&temp, "unsigned.mnar");
    let path_info =
        run_async(seed_signed_file(&source_store, &source_state, "unsigned-cli-root", b"unsigned cli payload", false));

    mantle_cmd()
        .arg("--store")
        .arg(&source_store)
        .arg("--state-dir")
        .arg(&source_state)
        .arg("store")
        .arg("archive")
        .arg("export")
        .arg("--to")
        .arg(&archive_path)
        .arg(path_info.store_path.name())
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsigned"));

    mantle_cmd()
        .arg("--store")
        .arg(&source_store)
        .arg("--state-dir")
        .arg(&source_state)
        .arg("store")
        .arg("archive")
        .arg("export")
        .arg("--to")
        .arg(&archive_path)
        .arg("--trust-unsigned")
        .arg(path_info.store_path.name())
        .assert()
        .success()
        .stdout(predicate::str::contains("ARCHIVE_EXPORT"));
}

#[test]
fn nario_v2_cli_lists_imports_and_skips_pinned_producer_fixture() {
    // r[verify store_transports.nario_v2_read_compatibility]
    // r[verify store_transports.nario_v2_validation]
    let fixture = Path::new("fixtures/nario-v2/positive-single.nario");
    let expected_path = std::fs::read_to_string("fixtures/nario-v2/store-path.txt").unwrap();
    let expected_path = expected_path.trim();
    let temp = tempfile::tempdir().unwrap();
    let output_dir = temp_path(&temp, "store");
    let state_dir = temp_path(&temp, "state");

    mantle_cmd()
        .arg("--json")
        .arg("store")
        .arg("archive")
        .arg("list")
        .arg("--format")
        .arg("nario-v2")
        .arg("--from")
        .arg(fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"format\": \"nario-v2\""))
        .stdout(predicate::str::contains("\"format_version\": 2"))
        .stdout(predicate::str::contains("non-v16-worker-metadata"))
        .stdout(predicate::str::contains("9512828397f684d0f732ea76b7631f69a0db34f7"))
        .stdout(predicate::str::contains(expected_path));
    mantle_cmd()
        .arg("store")
        .arg("archive")
        .arg("list")
        .arg("--format")
        .arg("nario-v2")
        .arg("--from")
        .arg("-")
        .write_stdin(std::fs::read(fixture).unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains(expected_path));

    let import = || {
        let mut command = mantle_cmd();
        command
            .arg("--json")
            .arg("--store")
            .arg(&output_dir)
            .arg("--state-dir")
            .arg(&state_dir)
            .arg("--store-prefix")
            .arg("/nix/store")
            .arg("store")
            .arg("archive")
            .arg("import")
            .arg("--format")
            .arg("nario-v2")
            .arg("--from")
            .arg(fixture)
            .arg("--trust-unsigned");
        command
    };
    import()
        .assert()
        .success()
        .stdout(predicate::str::contains("\"imported_count\": 1"))
        .stdout(predicate::str::contains("non-v16-worker-metadata"));
    let admitted = mantle_cmd()
        .args(["--json", "--store-prefix", "/nix/store"])
        .arg("--store")
        .arg(&output_dir)
        .arg("--state-dir")
        .arg(&state_dir)
        .args(["store", "info", expected_path.strip_prefix("/nix/store/").unwrap()])
        .output()
        .unwrap();
    assert!(admitted.status.success(), "{}", String::from_utf8_lossy(&admitted.stderr));
    let admitted: Value = serde_json::from_slice(&admitted.stdout).unwrap();
    assert_eq!(admitted["paths"][0]["signatures"], serde_json::json!([]));
    import().assert().success().stdout(predicate::str::contains("\"skipped_already_present_count\": 1"));
    let materialized = output_dir.join(expected_path.strip_prefix("/nix/store/").unwrap());
    assert_eq!(std::fs::read_to_string(materialized).unwrap(), "Mantle pinned Nario v2 fixture\n");
}

#[test]
fn nario_v2_cli_rejects_bad_input_wrong_prefix_untrusted_and_export() {
    // r[verify store_transports.nario_v2_bounded_admission]
    let fixture_bytes = include_bytes!("../fixtures/nario-v2/positive-single.nario");
    let temp = tempfile::tempdir().unwrap();
    let corrupt = temp_path(&temp, "corrupt.nario");
    let truncated = temp_path(&temp, "truncated.nario");
    std::fs::write(&corrupt, [0u8; NARIO_WIRE_WORD_BYTES]).unwrap();
    std::fs::write(&truncated, &fixture_bytes[..fixture_bytes.len() - NARIO_TRUNCATION_BYTES]).unwrap();

    for (path, diagnostic) in [(&corrupt, "wrong-magic"), (&truncated, "truncated-nar")] {
        mantle_cmd()
            .arg("store")
            .arg("archive")
            .arg("list")
            .arg("--format")
            .arg("nario-v2")
            .arg("--from")
            .arg(path)
            .assert()
            .failure()
            .stderr(predicate::str::contains(diagnostic));
    }

    mantle_cmd()
        .arg("--state-dir")
        .arg(temp_path(&temp, "untrusted-state"))
        .arg("--store-prefix")
        .arg("/nix/store")
        .arg("store")
        .arg("archive")
        .arg("import")
        .arg("--format")
        .arg("nario-v2")
        .arg("--from")
        .arg("fixtures/nario-v2/positive-single.nario")
        .assert()
        .failure()
        .stderr(predicate::str::contains("untrusted-signature"));

    mantle_cmd()
        .arg("--state-dir")
        .arg(temp_path(&temp, "wrong-prefix-state"))
        .arg("store")
        .arg("archive")
        .arg("import")
        .arg("--format")
        .arg("nario-v2")
        .arg("--from")
        .arg("fixtures/nario-v2/positive-single.nario")
        .arg("--trust-unsigned")
        .assert()
        .failure()
        .stderr(predicate::str::contains("store-prefix-mismatch"));

    let output = temp_path(&temp, "unsupported.nario");
    mantle_cmd()
        .arg("store")
        .arg("archive")
        .arg("export")
        .arg("--format")
        .arg("nario-v2")
        .arg("--to")
        .arg(&output)
        .arg("--all")
        .assert()
        .failure()
        .stderr(predicate::str::contains("export is unsupported"));
    assert!(!output.exists());
}

#[test]
fn nario_v2_cli_rejects_durable_negative_fixture_corpus() {
    // r[verify store_transports.nario_v2_bounded_admission]
    let cases = [
        ("negative-wrong-magic.nario", "wrong-magic"),
        ("negative-truncated.nario", "truncated-nar"),
        ("negative-trailing-data.nario", "trailing-data"),
        ("negative-duplicate-path.nario", "duplicate-path"),
        ("negative-oversized-path.nario", "bytes length out of range"),
        ("negative-unsupported-ca.nario", "unsupported-content-address"),
        ("negative-hash-mismatch.nario", "nar-hash-mismatch"),
    ];
    for (fixture, diagnostic) in cases {
        mantle_cmd()
            .arg("store")
            .arg("archive")
            .arg("list")
            .arg("--format")
            .arg("nario-v2")
            .arg("--from")
            .arg(Path::new("fixtures/nario-v2").join(fixture))
            .assert()
            .failure()
            .stderr(predicate::str::contains(diagnostic));
    }
}

#[test]
fn independent_snix_stores_agree_on_signed_bytes_only_with_the_same_fixture_key() {
    let root = tempfile::tempdir().unwrap();
    let fixture = b"the same signed NAR in three independent Snix states\n";
    let first_state = root.path().join("first-state");
    let first_store = root.path().join("first-store");
    let second_state = root.path().join("second-state");
    let second_store = root.path().join("second-store");
    let third_state = root.path().join("third-state");
    for state in [&first_state, &second_state, &third_state] {
        std::fs::create_dir_all(state).unwrap();
    }
    std::fs::write(first_state.join("signing-key"), TEST_KEYPAIR).unwrap();
    std::fs::write(second_state.join("signing-key"), TEST_KEYPAIR).unwrap();
    let alternate_raw = ed25519_dalek::SigningKey::from_bytes(&[19_u8; 32]);
    let mut alternate_keypair = Vec::from(alternate_raw.to_bytes());
    alternate_keypair.extend_from_slice(alternate_raw.verifying_key().as_bytes());
    std::fs::write(
        third_state.join("signing-key"),
        format!("archive-cli-2:{}", data_encoding::BASE64.encode(&alternate_keypair)),
    )
    .unwrap();
    let third_store = root.path().join("third-store");
    let first = run_async(seed_signed_file(&first_store, &first_state, "key-parity", fixture, true));
    let second = run_async(seed_signed_file(&second_store, &second_state, "key-parity", fixture, true));
    let third = run_async(async {
        let store = open_store(&third_store, &third_state).await;
        let mut info = signed_file_pathinfo(&store, "key-parity", fixture, false).await;
        let raw_signer = ed25519_dalek::SigningKey::from_bytes(&[19_u8; 32]);
        let signer = nix_compat::narinfo::SigningKey::new("archive-cli-2".to_string(), raw_signer);
        let refs: Vec<StorePathRef> = Vec::new();
        let fingerprint = nix_compat::narinfo::fingerprint_with_store_dir(
            &info.store_path.as_ref(),
            &info.nar_sha256,
            info.nar_size,
            refs.iter(),
            TEST_STORE_PREFIX,
        );
        info.signatures.push(signer.sign(fingerprint.as_bytes()).to_owned());
        store.pathinfo_service().put(info.clone()).await.unwrap();
        info
    });
    for (output_dir, info) in [(&first_store, &first), (&second_store, &second), (&third_store, &third)] {
        std::fs::create_dir_all(output_dir).unwrap();
        std::fs::write(output_dir.join(info.store_path.to_string()), fixture).unwrap();
    }
    let signed_bytes = serde_json::to_vec(&first).unwrap();
    assert_eq!(signed_bytes, serde_json::to_vec(&second).unwrap(), "the same fixture key must sign identically");
    assert_ne!(
        signed_bytes,
        serde_json::to_vec(&third).unwrap(),
        "different keys cannot produce the same signed PathInfo"
    );
    let mut first_unsigned = first.clone();
    let mut third_unsigned = third.clone();
    first_unsigned.signatures.clear();
    third_unsigned.signatures.clear();
    assert_eq!(
        serde_json::to_vec(&first_unsigned).unwrap(),
        serde_json::to_vec(&third_unsigned).unwrap(),
        "key changes must not alter the store path, node, references, or NAR"
    );
    assert_eq!(first.nar_sha256, third.nar_sha256);
    assert_eq!(first.nar_size, third.nar_size);
    let fixture_key = trusted_public_key();
    let alternate_key = other_trusted_public_key("archive-cli-2");
    let selector = first.store_path.name();
    for (state, store, key, explicit_backend) in [
        (&first_state, &first_store, &fixture_key, false),
        (&second_state, &second_store, &fixture_key, true),
        (&third_state, &third_store, &alternate_key, true),
    ] {
        let mut command = mantle_cmd();
        command.arg("--state-dir").arg(state).arg("--store").arg(store);
        if explicit_backend {
            command.args(["--store-backend", "snix"]);
        }
        command
            .args(["store", "verify", "--trusted-public-keys", key, selector])
            .assert()
            .success()
            .stdout(predicate::str::contains("trusted_signatures=1/1"));
    }
}

#[test]
fn legacy_and_populated_identityless_snix_reopen_preserve_signed_output_and_unrelated_state() {
    let golden: Value = serde_json::from_str(include_str!(
        "../.cairn/archive/2026-10-04-add-store-backend-selection/evidence/prechange-snix-golden-2026-10-04.json"
    ))
    .unwrap();
    let legacy_identity = data_encoding::HEXLOWER
        .decode(golden["fresh_store_identity_json_hex"].as_str().unwrap().as_bytes())
        .unwrap();
    let root = tempfile::tempdir().unwrap();
    for (case, legacy) in [("legacy-v1", true), ("identityless-populated", false)] {
        let state = root.path().join(format!("{case}-state"));
        let output = root.path().join(format!("{case}-store"));
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(state.join("signing-key"), TEST_KEYPAIR).unwrap();
        let original = run_async(seed_signed_file(&output, &state, case, b"preserved signed output\n", true));
        let identity_path = state.join("store-identity.json");
        if legacy {
            std::fs::write(&identity_path, &legacy_identity).unwrap();
        } else {
            std::fs::remove_file(&identity_path).unwrap();
        }
        std::fs::write(state.join("operator-owned-data"), b"must survive selected Snix open\n").unwrap();
        let before = state_files(&state);
        let config = StoreConfig::new(
            crunch_store::StoreBackend::Snix,
            state.clone(),
            output.clone(),
            TEST_STORE_PREFIX.to_string(),
        );
        config.preflight_backend_identity().unwrap();
        assert_eq!(state_files(&state), before, "{case} preflight mutated state before opening services");
        let reopened = run_async(async {
            let store = StoreHandle::open(config).await.unwrap();
            store.pathinfo_service().get(*original.store_path.digest()).await.unwrap().unwrap()
        });
        assert_eq!(reopened, original, "{case} lost or rewrote the signed PathInfo");
        let after = state_files(&state);
        for (path, content) in &before {
            if path == Path::new("pathinfo.redb") || path == Path::new("directories.redb") {
                continue;
            }
            assert_eq!(after.get(path), Some(content), "{case} rewrote a non-database file: {}", path.display());
        }
        if legacy {
            assert_eq!(std::fs::read(identity_path).unwrap(), legacy_identity, "legacy v1 must remain byte-identical");
        } else {
            assert!(!before.contains_key(Path::new("store-identity.json")));
            let added_identity: Value = serde_json::from_slice(&std::fs::read(identity_path).unwrap()).unwrap();
            assert_eq!(added_identity["schema"], "mantle-store-state-v2");
            assert_eq!(added_identity["backend"], "snix");
            assert_eq!(added_identity["logical_prefix"], TEST_STORE_PREFIX);
        }
    }
}

#[test]
fn default_and_explicit_snix_preserve_prechange_signed_and_gc_golden_facts() {
    let golden: Value = serde_json::from_str(include_str!(
        "../.cairn/archive/2026-10-04-add-store-backend-selection/evidence/prechange-snix-golden-2026-10-04.json"
    ))
    .unwrap();
    assert_eq!(golden["signer_public_key"], trusted_public_key());
    let historical_identity = data_encoding::HEXLOWER
        .decode(golden["fresh_store_identity_json_hex"].as_str().unwrap().as_bytes())
        .unwrap();
    let historical_identity: Value = serde_json::from_slice(&historical_identity).unwrap();
    assert_eq!(historical_identity["schema"], "mantle-store-state-v1");

    for explicit_backend in [false, true] {
        let fixed_root = std::env::var_os("MANTLE_BASELINE_FIXTURE_ROOT").map(PathBuf::from);
        let temporary_root = fixed_root.is_none().then(|| tempfile::tempdir().unwrap());
        let root = fixed_root.clone().unwrap_or_else(|| temporary_root.as_ref().unwrap().path().to_path_buf());
        if fixed_root.is_some() {
            assert!(root.is_absolute() && !root.exists(), "fixed fixture root must be new and absolute");
            std::fs::create_dir(&root).unwrap();
        }
        let state = root.join("state");
        let store = root.join("store");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(state.join("signing-key"), TEST_KEYPAIR).unwrap();
        let kept = run_async(seed_signed_file(&store, &state, "baseline-keep", b"kept NAR\n", true));
        let candidate = run_async(seed_signed_file(&store, &state, "baseline-candidate", b"candidate NAR\n", true));
        assert_eq!(kept.store_path.to_string(), golden["kept_store_path"].as_str().unwrap());
        assert_eq!(candidate.store_path.to_string(), golden["candidate_store_path"].as_str().unwrap());
        assert_eq!(data_encoding::HEXLOWER.encode(&kept.nar_sha256), golden["kept_nar_sha256"].as_str().unwrap());
        assert_eq!(
            data_encoding::HEXLOWER.encode(&candidate.nar_sha256),
            golden["candidate_nar_sha256"].as_str().unwrap()
        );
        assert_eq!(
            data_encoding::HEXLOWER.encode(&serde_json::to_vec(&kept).unwrap()),
            golden["kept_signed_pathinfo_json_hex"].as_str().unwrap(),
            "selected Snix changed historical signed PathInfo"
        );
        assert_eq!(
            data_encoding::HEXLOWER.encode(&serde_json::to_vec(&candidate).unwrap()),
            golden["candidate_signed_pathinfo_json_hex"].as_str().unwrap(),
            "selected Snix changed historical candidate PathInfo"
        );
        let retained = kept.store_path.to_absolute_path_with_prefix(TEST_STORE_PREFIX);
        let legacy_root = serde_json::json!({
            retained.clone(): {
                "logical_path": retained,
                "source": "build",
                "created_unix_s": 100,
            }
        });
        std::fs::write(state.join("gc-roots.json"), serde_json::to_vec(&legacy_root).unwrap()).unwrap();

        let cli_json = |state: &Path, store: &Path, args: &[&str]| {
            let mut command = mantle_cmd();
            command
                .arg("--state-dir")
                .arg(state)
                .arg("--store")
                .arg(store)
                .arg("--store-prefix")
                .arg(TEST_STORE_PREFIX)
                .arg("--json");
            if explicit_backend {
                command.args(["--store-backend", "snix"]);
            }
            let output = command.args(args).output().unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            serde_json::from_slice::<Value>(&output.stdout).unwrap()
        };
        let fresh_state = root.join("fresh-state");
        let fresh_store = root.join("fresh-store");
        std::fs::create_dir_all(&fresh_state).unwrap();
        std::fs::write(fresh_state.join("signing-key"), TEST_KEYPAIR).unwrap();
        let listing = cli_json(&fresh_state, &fresh_store, &["store", "list"]);
        assert_eq!(
            listing,
            serde_json::from_str::<Value>(golden["fresh_store_list_stdout"].as_str().unwrap()).unwrap()
        );
        let selected_identity: Value =
            serde_json::from_slice(&std::fs::read(fresh_state.join("store-identity.json")).unwrap()).unwrap();
        assert_eq!(selected_identity["schema"], "mantle-store-state-v2");
        assert_eq!(selected_identity["backend"], "snix");
        assert_eq!(selected_identity["logical_prefix"], historical_identity["logical_prefix"]);
        assert_eq!(selected_identity["trust_policy_id"], historical_identity["trust_policy_id"]);

        let mut selected_info = cli_json(&state, &store, &["store", "info", kept.store_path.name()]);
        assert_eq!(selected_info["backend"], "snix");
        assert_eq!(selected_info["backend_capabilities"]["rust_unit_cache"], true);
        selected_info.as_object_mut().unwrap().remove("backend");
        selected_info.as_object_mut().unwrap().remove("backend_capabilities");
        let historical_info: Value = serde_json::from_str(golden["store_info_stdout"].as_str().unwrap()).unwrap();
        assert_eq!(selected_info, historical_info);
        let selected_roots = cli_json(&state, &store, &["store", "roots"]);
        let historical_roots: Value = serde_json::from_str(golden["store_roots_stdout"].as_str().unwrap()).unwrap();
        assert_eq!(selected_roots, historical_roots);

        let selected_gc = cli_json(&state, &store, &["store", "gc", "--dry-run"]);
        let historical_gc: Value = serde_json::from_str(golden["store_gc_dry_run_stdout"].as_str().unwrap()).unwrap();
        let relative_observations = |mut report: Value, fixture_root: &Path| {
            report.as_object_mut().unwrap().remove("plan_id");
            for observation in report["reclaim_observations"].as_array_mut().unwrap() {
                let path = observation["path"].as_str().unwrap();
                let relative = Path::new(path).strip_prefix(fixture_root).unwrap();
                observation["path"] = relative.to_string_lossy().to_string().into();
            }
            report
        };
        assert_eq!(
            relative_observations(selected_gc.clone(), &root),
            relative_observations(historical_gc.clone(), Path::new(golden["physical_fixture_root"].as_str().unwrap())),
            "GC candidates, roots, reclaim, and retention identity must match the prechange golden"
        );
        if root == Path::new(golden["physical_fixture_root"].as_str().unwrap()) {
            assert_eq!(
                selected_gc["plan_id"], historical_gc["plan_id"],
                "the GC execution identity must match when the physical fixture path is identical"
            );
        }
        if fixed_root.is_some() {
            std::fs::remove_dir_all(&root).unwrap();
        }
    }
}

fn rail_cmd(backend: crunch_store::StoreBackend, state: &Path, output: &Path) -> Command {
    let mut cmd = mantle_cmd();
    cmd.args(["--store-backend", backend.as_str(), "--store-prefix", TEST_STORE_PREFIX])
        .arg("--state-dir")
        .arg(state)
        .arg("--store")
        .arg(output);
    cmd
}

fn rail_seed(
    backend: crunch_store::StoreBackend,
    state: &Path,
    output: &Path,
    second_signer: &Path,
    fixture_key: &str,
) -> (PathInfo, PathInfo, PathInfo) {
    std::fs::create_dir_all(state).unwrap();
    std::fs::write(state.join("signing-key"), TEST_KEYPAIR).unwrap();
    let alternate_raw = ed25519_dalek::SigningKey::from_bytes(&[19_u8; 32]);
    let mut alternate_keypair = Vec::from(alternate_raw.to_bytes());
    alternate_keypair.extend_from_slice(alternate_raw.verifying_key().as_bytes());
    std::fs::write(second_signer, format!("archive-cli-2:{}", data_encoding::BASE64.encode(&alternate_keypair)))
        .unwrap();
    if backend == crunch_store::StoreBackend::Casita {
        std::fs::write(state.join("casita-trusted-public-keys"), format!("{fixture_key}\n")).unwrap();
    }
    let result = run_async(async {
        let mut store = StoreHandle::open(StoreConfig::new(
            backend,
            state.to_path_buf(),
            output.to_path_buf(),
            TEST_STORE_PREFIX.to_string(),
        ))
        .await
        .unwrap();
        let child = signed_file_pathinfo(&store, "rail-child", b"referenced NAR\n", true).await;
        let candidate = signed_file_pathinfo(&store, "rail-candidate", b"unretained NAR\n", true).await;
        let mut retained = signed_file_pathinfo(&store, "rail-retained", b"retained NAR\n", false).await;
        retained.references.push(child.store_path.clone());
        sign_pathinfo(&mut retained);
        for (info, content) in [
            (&child, b"referenced NAR\n".as_slice()),
            (&candidate, b"unretained NAR\n".as_slice()),
            (&retained, b"retained NAR\n".as_slice()),
        ] {
            store.pathinfo_service().put(info.clone()).await.unwrap();
            assert_eq!(store.export_cached_path_info(&info.store_path).await.unwrap(), Some(info.clone()));
            assert_eq!(std::fs::read(output.join(info.store_path.to_string())).unwrap(), content);
        }
        (retained, child, candidate)
    });
    let identity: Value = serde_json::from_slice(&std::fs::read(state.join("store-identity.json")).unwrap()).unwrap();
    assert_eq!(identity["schema"], "mantle-store-state-v2");
    assert_eq!(identity["backend"], backend.as_str());
    result
}

fn rail_compare_prechange_seed(golden: &Value, output: &Path, infos: [&PathInfo; 3]) {
    assert_eq!(golden["signer_public_key"], trusted_public_key());
    assert_eq!(golden["store_prefix"], TEST_STORE_PREFIX);
    let expected = golden["paths"].as_array().unwrap();
    assert_eq!(expected.len(), infos.len());
    for (info, historical) in infos.into_iter().zip(expected) {
        assert_eq!(info.store_path.to_string(), historical["store_path"]);
        assert_eq!(data_encoding::HEXLOWER.encode(&info.nar_sha256), historical["nar_sha256"]);
        assert_eq!(info.nar_size, historical["nar_size"]);
        assert_eq!(
            data_encoding::HEXLOWER.encode(&serde_json::to_vec(info).unwrap()),
            historical["signed_pathinfo_json_hex"],
            "same-key signed PathInfo must match pre-selection Snix bytes"
        );
        assert_eq!(
            data_encoding::HEXLOWER.encode(&std::fs::read(output.join(info.store_path.to_string())).unwrap()),
            historical["exported_content_hex"],
        );
    }
}

fn rail_archive_path_facts(info: &Value) -> Value {
    serde_json::json!({
        "store_path": info["store_path"],
        "nar_sha256": info["nar_sha256"],
        "nar_size": info["nar_size"],
        "signatures": info["signatures"],
        "references": info["references"],
        "ca": info["ca"],
        "deriver": info["deriver"],
    })
}

fn rail_compare_prechange_archive(golden: &Value, observed: &Value, resigned: &Value) {
    assert_eq!(observed["listed_paths"], golden["archive"]["listed_paths"]);
    for field in ["imported_retained", "imported_child"] {
        assert_eq!(
            rail_archive_path_facts(&observed[field]),
            rail_archive_path_facts(&golden["archive"][field]),
            "{field} changed same-key archive import facts"
        );
    }
    assert_eq!(
        rail_archive_path_facts(resigned),
        rail_archive_path_facts(&golden["archive"]["resigned_retained"]),
        "same-key second signer must produce the same verified signed PathInfo"
    );
    assert_eq!(golden["archive"]["verified_signatures_after_sign"], 2);
}

fn rail_normalize_gc_paths(mut report: Value, root: &Path) -> Value {
    report.as_object_mut().unwrap().remove("plan_id");
    for observation in report["reclaim_observations"].as_array_mut().unwrap() {
        let relative = Path::new(observation["path"].as_str().unwrap()).strip_prefix(root).unwrap();
        observation["path"] = relative.to_string_lossy().to_string().into();
    }
    report
}

fn rail_canonicalize_historical_gc(mut report: Value) -> Value {
    let observations = report["reclaim_observations"].as_array_mut().unwrap();
    // The selected Snix GC sorts dead paths within each blob category before
    // hashing; pre-selection read_dir order was an execution-ID input.
    for category in ["blob-index", "blob-chunk"] {
        if let Some(start) = observations.iter().position(|observation| observation["category"] == category) {
            let end = start
                + observations[start..].iter().take_while(|observation| observation["category"] == category).count();
            observations[start..end]
                .sort_unstable_by(|left, right| left["path"].as_str().unwrap().cmp(right["path"].as_str().unwrap()));
        }
    }
    report
}

fn rail_compare_prechange_gc(golden: &Value, root: &Path, observed: &Value) {
    let historical = &golden["gc"];
    let fixed_root = Path::new(golden["physical_fixture_root"].as_str().unwrap());
    for field in ["planned", "fresh"] {
        let selected = rail_normalize_gc_paths(observed[field].clone(), root);
        let historical_canonical =
            rail_canonicalize_historical_gc(rail_normalize_gc_paths(historical[field].clone(), fixed_root));
        assert_eq!(selected, historical_canonical, "same-key Snix {field} canonical GC facts changed");
        if root == fixed_root {
            println!(
                "PRECHANGE_SNIX_RAIL_GC_PLAN {field} old_first={} selected_canonical={}",
                historical[field]["plan_id"], observed[field]["plan_id"]
            );
        }
    }
    assert_eq!(observed["after_pin_candidate_paths"], historical["after_pin_candidate_paths"]);
    assert_eq!(observed["execution_candidate_paths"], historical["execution_candidate_paths"]);
    assert_eq!(observed["execution_complete"], historical["execution_complete"]);
}

fn rail_reopen_and_reuse(
    backend: crunch_store::StoreBackend,
    state: &Path,
    output: &Path,
    retained: &PathInfo,
    child: &PathInfo,
) -> Value {
    run_async(async {
        let reopened = StoreHandle::open(StoreConfig::new(
            backend,
            state.to_path_buf(),
            output.to_path_buf(),
            TEST_STORE_PREFIX.to_string(),
        ))
        .await
        .unwrap();
        let observed = reopened.pathinfo_service().get(*retained.store_path.digest()).await.unwrap().unwrap();
        assert_eq!(observed, *retained);
        let ports = reopened.into_builder_store_parts();
        let closure = ports.build_store.resolve_closure(&retained.store_path, StoreFallbackMode::Strict).await.unwrap();
        assert!(closure.audit_events.is_empty());
        assert_eq!(closure.paths, vec![retained.store_path.clone(), child.store_path.clone()]);
        let action = crunch_action_result_core::ActionResultRecord {
            schema: crunch_action_result_core::ACTION_RESULT_SCHEMA.to_string(),
            result_ref: String::new(),
            action_ref: String::new(),
            outputs: vec![crunch_action_result_core::ActionResultOutput {
                name: "out".to_string(),
                object_ref: String::new(),
                store_path: retained.store_path.to_absolute_path_with_prefix(TEST_STORE_PREFIX),
                path_info_ref: String::new(),
            }],
            action_receipt_ref: String::new(),
            reference_scan_refs: Vec::new(),
            sandbox_policy_ref: String::new(),
            network_policy_ref: String::new(),
            producer_identity: String::new(),
            producer_policy_ref: String::new(),
            signature_refs: Vec::new(),
            publication_policy_ref: String::new(),
            non_claims: Vec::new(),
        };
        let reused = ports.action_results.probe_outputs(&action).await.unwrap();
        assert_eq!(reused.outputs.get("out"), Some(retained));
        assert_eq!(reused.reused_nar_bytes, retained.nar_size);
        serde_json::json!({
            "reopened_signed_pathinfo_hex": data_encoding::HEXLOWER.encode(&serde_json::to_vec(&observed).unwrap()),
            "closure_paths": closure.paths.iter().map(ToString::to_string).collect::<Vec<_>>(),
            "reused_nar_bytes": reused.reused_nar_bytes,
            "reused_output_signed_pathinfo_hex": data_encoding::HEXLOWER.encode(&serde_json::to_vec(reused.outputs.get("out").unwrap()).unwrap()),
        })
    })
}

fn rail_archive_roundtrip(
    backend: crunch_store::StoreBackend,
    root: &Path,
    retained: &PathInfo,
    admitted: [&PathInfo; 3],
    fixture_key: &str,
    alternate_key: &str,
) -> (Value, Value) {
    let state = root.join("state");
    let output = root.join("output");
    let imported_state = root.join("imported-state");
    let imported_output = root.join("imported-output");
    let archive = root.join("closure.mnar");
    for info in admitted {
        rail_cmd(backend, &state, &output)
            .args([
                "store",
                "verify",
                "--trusted-public-keys",
                fixture_key,
                info.store_path.name(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("trusted_signatures=1/1"));
    }
    rail_cmd(backend, &state, &output)
        .args(["store", "archive", "export", "--to"])
        .arg(&archive)
        .arg(retained.store_path.name())
        .assert()
        .success();
    let listed = rail_cmd(backend, &state, &output)
        .args(["--json", "store", "archive", "list", "--from"])
        .arg(&archive)
        .output()
        .unwrap();
    assert!(listed.status.success(), "{}", String::from_utf8_lossy(&listed.stderr));
    let listed: Value = serde_json::from_slice(&listed.stdout).unwrap();
    std::fs::create_dir_all(&imported_state).unwrap();
    std::fs::write(imported_state.join("signing-key"), TEST_KEYPAIR).unwrap();
    if backend == crunch_store::StoreBackend::Casita {
        std::fs::write(imported_state.join("casita-trusted-public-keys"), format!("{fixture_key}\n{alternate_key}\n"))
            .unwrap();
    }
    rail_cmd(backend, &imported_state, &imported_output)
        .args(["store", "archive", "import", "--from"])
        .arg(&archive)
        .args(["--trusted-public-keys", fixture_key])
        .assert()
        .success();
    let imported = rail_cmd(backend, &imported_state, &imported_output)
        .args(["--json", "store", "info", retained.store_path.name()])
        .output()
        .unwrap();
    assert!(imported.status.success(), "{}", String::from_utf8_lossy(&imported.stderr));
    let imported: Value = serde_json::from_slice(&imported.stdout).unwrap();
    assert_eq!(imported["backend"], backend.as_str());
    assert_eq!(imported["paths"][0]["nar_sha256"], data_encoding::HEXLOWER.encode(&retained.nar_sha256));
    assert_eq!(imported["paths"][0]["signatures"][0], retained.signatures[0].to_string());
    let imported_child = rail_cmd(backend, &imported_state, &imported_output)
        .args(["--json", "store", "info", admitted[1].store_path.name()])
        .output()
        .unwrap();
    assert!(imported_child.status.success(), "{}", String::from_utf8_lossy(&imported_child.stderr));
    let imported_child: Value = serde_json::from_slice(&imported_child.stdout).unwrap();
    let archive_facts = serde_json::json!({
        "listed_paths": listed["paths"].as_array().unwrap().iter()
            .map(|path| path["store_path"].as_str().unwrap()).collect::<Vec<_>>(),
        "imported_retained": imported["paths"][0],
        "imported_child": imported_child["paths"][0],
    });
    (imported["backend_capabilities"].clone(), archive_facts)
}

fn rail_check_profile(backend: crunch_store::StoreBackend, root: &Path, capabilities: &Value) {
    assert_eq!(capabilities["rust_unit_cache"], backend == crunch_store::StoreBackend::Snix);
    assert_eq!(capabilities["overlay_composition"], backend == crunch_store::StoreBackend::Snix);
    assert_eq!(capabilities["atomic_batch_import"], true);
    assert_eq!(capabilities["unsigned_admission"], backend == crunch_store::StoreBackend::Snix);
    if backend == crunch_store::StoreBackend::Casita {
        assert_eq!(capabilities["max_root_changes"], 1024);
        let state = root.join("imported-state");
        let output = root.join("imported-output");
        let before = state_files(&state);
        rail_cmd(backend, &state, &output)
            .arg("--base-store")
            .arg(root.join("nonexistent-base"))
            .args(["store", "list"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("casita-overlay-unsupported"));
        assert_eq!(state_files(&state), before);
    } else {
        assert!(capabilities["max_root_changes"].is_null());
    }
}

fn rail_check_overlay(backend: crunch_store::StoreBackend, root: &Path, retained: &PathInfo) {
    let state = root.join("overlay-state");
    let output = root.join("overlay-output");
    if backend == crunch_store::StoreBackend::Casita {
        let before = state_files(&root.join("imported-state"));
        rail_cmd(backend, &state, &output)
            .arg("--base-store")
            .arg(root.join("imported-state"))
            .args(["store", "list"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("casita-overlay-unsupported"));
        assert!(!state.exists() && !output.exists());
        assert_eq!(state_files(&root.join("imported-state")), before);
        return;
    }

    let first = root.join("state");
    let second = root.join("imported-state");
    let second_info = run_async(seed_signed_file(
        &root.join("imported-output"),
        &second,
        "rail-second-base",
        b"second overlay base\n",
        true,
    ));
    let mut original_permissions = Vec::new();
    for base in [&first, &second] {
        let mut pending = vec![base.to_path_buf()];
        while let Some(path) = pending.pop() {
            if path.is_dir() {
                pending.extend(std::fs::read_dir(&path).unwrap().map(|entry| entry.unwrap().path()));
            }
            let original = std::fs::metadata(&path).unwrap().permissions();
            let mut readonly = original.clone();
            readonly.set_readonly(true);
            std::fs::set_permissions(&path, readonly).unwrap();
            original_permissions.push((path, original));
        }
    }
    let before_first = state_files(&first);
    let before_second = state_files(&second);
    for (info, index) in [(retained, 1), (&second_info, 2)] {
        let selected = rail_cmd(backend, &state, &output)
            .arg("--base-store")
            .arg(&first)
            .arg("--base-store")
            .arg(&second)
            .args(["--json", "store", "info", info.store_path.name()])
            .output()
            .unwrap();
        assert!(selected.status.success(), "{}", String::from_utf8_lossy(&selected.stderr));
        let report: Value = serde_json::from_slice(&selected.stdout).unwrap();
        assert_eq!(report["paths"][0]["store_path"], info.store_path.to_string());
        assert_eq!(report["paths"][0]["layer"]["kind"], "base");
        assert_eq!(report["paths"][0]["layer"]["index"], index);
        assert_eq!(report["overlay"]["bases"].as_array().unwrap().len(), 2);
        assert_eq!(report["backend_capabilities"]["overlay_composition"], true);
    }
    assert_eq!(state_files(&first), before_first, "first read-only base must remain byte-identical");
    assert_eq!(state_files(&second), before_second, "second read-only base must remain byte-identical");
    for (path, permissions) in original_permissions.into_iter().rev() {
        std::fs::set_permissions(path, permissions).unwrap();
    }
}

fn rail_check_atomic_batch(backend: crunch_store::StoreBackend, root: &Path, fixture_key: &str) {
    let state = root.join("batch-state");
    let output = root.join("batch-output");
    std::fs::create_dir(&state).unwrap();
    if backend == crunch_store::StoreBackend::Casita {
        std::fs::write(state.join("casita-trusted-public-keys"), format!("{fixture_key}\n")).unwrap();
    }
    run_async(async {
        let store = StoreHandle::open(StoreConfig::new(backend, state.clone(), output, TEST_STORE_PREFIX.to_owned()))
            .await
            .unwrap();
        let template = signed_file_pathinfo(&store, "rail-batch-template", b"shared batch NAR\n", true).await;
        let count = if backend == crunch_store::StoreBackend::Casita {
            1024
        } else {
            2
        };
        let infos = (0..=count)
            .map(|index| {
                let name = format!("rail-batch-{index:04}");
                let mut info = template.clone();
                info.store_path = StorePath::from_name_and_digest_fixed(&name, digest_from_name(&name)).unwrap();
                info.signatures.clear();
                sign_pathinfo(&mut info);
                info
            })
            .collect::<Vec<_>>();
        if backend == crunch_store::StoreBackend::Casita {
            assert_eq!(backend.profile().max_root_changes, Some(count));
            let before = state_files(&state);
            let error = store.pathinfo_service().put_batch_atomic(infos.clone()).await.unwrap_err().to_string();
            assert!(error.contains("casita-batch-limit: 1025 roots exceed 1024"), "{error}");
            assert_eq!(state_files(&state), before, "oversize batch must not publish any root bytes");
            assert!(store.pathinfo_service().get(*infos[0].store_path.digest()).await.unwrap().is_none());
            assert!(store.pathinfo_service().get(*infos[count].store_path.digest()).await.unwrap().is_none());
        }
        let admitted = store.pathinfo_service().put_batch_atomic(infos[..count].to_vec()).await.unwrap();
        assert_eq!(admitted, infos[..count]);
        drop(store);
        let reopened = StoreHandle::open(StoreConfig::new(
            backend,
            state.clone(),
            root.join("batch-output"),
            TEST_STORE_PREFIX.to_owned(),
        ))
        .await
        .unwrap();
        for index in [0, count / 2, count - 1] {
            assert_eq!(
                reopened.pathinfo_service().get(*infos[index].store_path.digest()).await.unwrap(),
                Some(infos[index].clone())
            );
        }
        assert!(reopened.pathinfo_service().get(*infos[count].store_path.digest()).await.unwrap().is_none());
    });
}

fn rail_check_unsigned(backend: crunch_store::StoreBackend, root: &Path) {
    let state = root.join("unsigned-state");
    let output = root.join("unsigned-output");
    let archive = Path::new("fixtures/nario-v2/positive-single.nario");
    if backend == crunch_store::StoreBackend::Casita {
        let before = state_files(&root.join("imported-state"));
        rail_cmd(backend, &root.join("imported-state"), &root.join("imported-output"))
            .args(["store", "archive", "import", "--from"])
            .arg(root.join("closure.mnar"))
            .arg("--trust-unsigned")
            .assert()
            .failure()
            .stderr(predicate::str::contains("casita-trust-unsigned-unsupported"));
        assert_eq!(state_files(&root.join("imported-state")), before);
        return;
    }
    let imported = mantle_cmd()
        .args(["--store-backend", "snix", "--store-prefix", "/nix/store"])
        .arg("--state-dir")
        .arg(&state)
        .arg("--store")
        .arg(&output)
        .args(["store", "archive", "import", "--format", "nario-v2", "--from"])
        .arg(archive)
        .arg("--trust-unsigned")
        .output()
        .unwrap();
    assert!(imported.status.success(), "{}", String::from_utf8_lossy(&imported.stderr));
    let expected_path = std::fs::read_to_string("fixtures/nario-v2/store-path.txt").unwrap();
    let name = expected_path.trim().strip_prefix("/nix/store/").unwrap();
    let observed = mantle_cmd()
        .args(["--store-backend", "snix", "--store-prefix", "/nix/store"])
        .arg("--state-dir")
        .arg(&state)
        .arg("--store")
        .arg(&output)
        .args(["--json", "store", "info", name])
        .output()
        .unwrap();
    assert!(observed.status.success(), "{}", String::from_utf8_lossy(&observed.stderr));
    let report: Value = serde_json::from_slice(&observed.stdout).unwrap();
    assert_eq!(report["backend"], "snix");
    assert_eq!(report["paths"][0]["store_path"], name);
    assert_eq!(report["paths"][0]["signatures"], serde_json::json!([]));
}

fn rail_check_rust_cache(backend: crunch_store::StoreBackend, root: &Path) {
    let state = root.join("rust-cache-state");
    let output = root.join("rust-cache-store");
    let config = || StoreConfig::new(backend, state.clone(), output.clone(), TEST_STORE_PREFIX.to_owned());
    if backend == crunch_store::StoreBackend::Casita {
        let error = run_async(crunch_rust_cache::RustCache::open_async(config())).unwrap_err().to_string();
        assert!(error.contains("casita-rust-cache-unsupported"), "{error}");
        assert!(!state.exists() && !output.exists());
        rail_cmd(backend, &state, &output)
            .args(["rust-cache", "serve", "--policy"])
            .arg(root.join("missing-rust-cache-policy.json"))
            .arg("--receipt-dir")
            .arg(root.join("missing-rust-cache-receipts"))
            .assert()
            .failure()
            .stderr(predicate::str::contains("casita-rust-cache-unsupported"));
        assert!(!state.exists() && !output.exists());
        assert!(!root.join("missing-rust-cache-receipts").exists());
        return;
    }
    use crunch_rust_cache::PublishRequest;
    use crunch_rust_cache_core::LocalCachePolicy;
    use crunch_rust_cache_core::RustUnitActionInput;
    use crunch_rust_cache_core::canonical_rust_action;
    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let action = canonical_rust_action(RustUnitActionInput {
        unit_id: "rail-unit".to_owned(),
        package_id: "rail-package".to_owned(),
        crate_name: "rail_crate".to_owned(),
        target_kind: "lib".to_owned(),
        execution_kind: "target".to_owned(),
        host_triple: "x86_64-unknown-linux-gnu".to_owned(),
        target_triple: "x86_64-unknown-linux-gnu".to_owned(),
        profile: "debug".to_owned(),
        mode: "build".to_owned(),
        features: Vec::new(),
        source_digest_blake3: DIGEST.to_owned(),
        compiler_digest_blake3: DIGEST.to_owned(),
        compiler_version_digest_blake3: DIGEST.to_owned(),
        toolchain_closure_digest_blake3: DIGEST.to_owned(),
        execution_platform_digest_blake3: DIGEST.to_owned(),
        semantic_arguments: Vec::new(),
        admitted_environment: Default::default(),
        dependency_artifacts: Vec::new(),
        host_artifacts: Vec::new(),
        build_script_facts: Vec::new(),
        native_link_facts: Vec::new(),
        compiler_policy_digest_blake3: DIGEST.to_owned(),
    })
    .unwrap();
    let policy = LocalCachePolicy {
        reads_enabled: true,
        writes_enabled: true,
        ..LocalCachePolicy::default()
    };
    let built = root.join("rust-cache-built");
    let restored = root.join("rust-cache-restored");
    std::fs::create_dir(&built).unwrap();
    std::fs::write(built.join("librail_crate.rlib"), b"real backend-selected rust cache artifact\n").unwrap();
    std::fs::write(built.join(crunch_rust_cache::RUST_UNIT_EXECUTION_RECEIPT_FILE), b"mutable receipt").unwrap();
    let first = run_async(crunch_rust_cache::RustCache::open_async(config())).unwrap();
    let published = run_async(first.publish(PublishRequest {
        action: &action,
        output_dir: &built,
        producer_receipt_ref: &format!("mantle-rust-receipt://blake3/{DIGEST}"),
        policy: &policy,
    }))
    .unwrap();
    drop(first);
    let reopened = run_async(crunch_rust_cache::RustCache::open_async(config())).unwrap();
    let hit = run_async(reopened.restore(&action, &restored, &policy)).unwrap();
    assert_eq!(hit.disposition, crunch_rust_cache::CACHE_DISPOSITION_HIT);
    assert_eq!(hit.selected_result_ref.as_deref(), Some(published.result_ref.as_str()));
    assert!(!hit.compiler_executed);
    assert_eq!(
        std::fs::read(restored.join("librail_crate.rlib")).unwrap(),
        b"real backend-selected rust cache artifact\n"
    );
    assert!(!restored.join(crunch_rust_cache::RUST_UNIT_EXECUTION_RECEIPT_FILE).exists());
}

fn rail_sign_and_check_mixed_backend(
    backend: crunch_store::StoreBackend,
    root: &Path,
    retained: &PathInfo,
    fixture_key: &str,
    alternate_key: &str,
) -> (usize, Value) {
    let state = root.join("imported-state");
    let output = root.join("imported-output");
    rail_cmd(backend, &state, &output)
        .args(["store", "sign", retained.store_path.name(), "--signing-key"])
        .arg(root.join("second-test-only-signing-key"))
        .assert()
        .success()
        .stdout(predicate::str::contains(retained.store_path.to_string()));
    let resigned = rail_cmd(backend, &state, &output)
        .args(["--json", "store", "info", retained.store_path.name()])
        .output()
        .unwrap();
    assert!(resigned.status.success(), "{}", String::from_utf8_lossy(&resigned.stderr));
    let resigned: Value = serde_json::from_slice(&resigned.stdout).unwrap();
    let signatures = resigned["paths"][0]["signatures"].as_array().unwrap();
    assert_eq!(signatures.len(), 2, "store sign must append the independently provisioned second signature");
    assert!(signatures.iter().any(|sig| sig.as_str().unwrap().starts_with("archive-cli-2:")));
    rail_cmd(backend, &state, &output)
        .args([
            "store",
            "verify",
            "--trusted-public-keys",
            fixture_key,
            "--trusted-public-keys",
            alternate_key,
            retained.store_path.name(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("trusted_signatures=2/2"));
    let identity_before = state_files(&state);
    let wrong_backend = if backend == crunch_store::StoreBackend::Snix {
        "casita"
    } else {
        "snix"
    };
    mantle_cmd()
        .args(["--store-backend", wrong_backend, "--store-prefix", TEST_STORE_PREFIX])
        .arg("--state-dir")
        .arg(&state)
        .arg("--store")
        .arg(&output)
        .args(["store", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("store-backend-mismatch"));
    assert_eq!(state_files(&state), identity_before);
    (signatures.len(), resigned["paths"][0].clone())
}

fn rail_gc_retains_closure(
    backend: crunch_store::StoreBackend,
    state: &Path,
    output: &Path,
    retained: &PathInfo,
    child: &PathInfo,
    fixture_key: &str,
) {
    for info in [retained, child] {
        let observed = rail_cmd(backend, state, output)
            .args(["--json", "store", "info", info.store_path.name()])
            .output()
            .unwrap();
        assert!(observed.status.success(), "{}", String::from_utf8_lossy(&observed.stderr));
        let observed: Value = serde_json::from_slice(&observed.stdout).unwrap();
        assert_eq!(observed["paths"][0]["nar_sha256"], data_encoding::HEXLOWER.encode(&info.nar_sha256));
        assert!(output.join(info.store_path.to_string()).exists(), "plan-bound GC must retain the pinned closure");
        rail_cmd(backend, state, output)
            .args([
                "store",
                "verify",
                "--trusted-public-keys",
                fixture_key,
                info.store_path.name(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("trusted_signatures=1/1"));
    }
}

fn rail_gc(
    backend: crunch_store::StoreBackend,
    state: &Path,
    output: &Path,
    retained: &PathInfo,
    child: &PathInfo,
    candidate: &PathInfo,
    fixture_key: &str,
) -> (&'static str, Value) {
    let retained_logical = retained.store_path.to_absolute_path_with_prefix(TEST_STORE_PREFIX);
    let candidate_logical = candidate.store_path.to_absolute_path_with_prefix(TEST_STORE_PREFIX);
    let legacy_root = serde_json::json!({
        retained_logical.clone(): {
            "logical_path": retained_logical,
            "source": "build",
            "created_unix_s": 100,
        }
    });
    std::fs::write(state.join("gc-roots.json"), serde_json::to_vec(&legacy_root).unwrap()).unwrap();
    let plan = rail_cmd(backend, state, output).args(["--json", "store", "gc", "--dry-run"]).output().unwrap();
    assert!(plan.status.success(), "{}", String::from_utf8_lossy(&plan.stderr));
    let plan: Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert_eq!(plan["candidate_paths"], serde_json::json!([candidate_logical.clone()]));
    let accepted_id = plan["plan_id"].as_str().unwrap();
    rail_cmd(backend, state, output).args(["store", "pin", &candidate_logical]).assert().success();
    let blocker = if backend == crunch_store::StoreBackend::Snix {
        "stale-gc-plan"
    } else {
        "gc-plan-stale"
    };
    rail_cmd(backend, state, output)
        .args(["store", "gc", "--execute", "--plan-id", accepted_id])
        .assert()
        .failure()
        .stderr(predicate::str::contains(blocker));
    assert!(output.join(candidate.store_path.to_string()).exists(), "stale plan must not collect live content");
    let after = rail_cmd(backend, state, output).args(["--json", "store", "gc", "--dry-run"]).output().unwrap();
    assert!(after.status.success(), "{}", String::from_utf8_lossy(&after.stderr));
    let after: Value = serde_json::from_slice(&after.stdout).unwrap();
    assert_eq!(after["candidate_paths"], serde_json::json!([]));
    rail_cmd(backend, state, output).args(["store", "unpin", &candidate_logical]).assert().success();
    let fresh = rail_cmd(backend, state, output).args(["--json", "store", "gc", "--dry-run"]).output().unwrap();
    assert!(fresh.status.success(), "{}", String::from_utf8_lossy(&fresh.stderr));
    let fresh: Value = serde_json::from_slice(&fresh.stdout).unwrap();
    assert_eq!(fresh["candidate_paths"], serde_json::json!([candidate_logical]));
    let execution = rail_cmd(backend, state, output)
        .args([
            "--json",
            "store",
            "gc",
            "--execute",
            "--plan-id",
            fresh["plan_id"].as_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(execution.status.success(), "{}", String::from_utf8_lossy(&execution.stderr));
    let execution: Value = serde_json::from_slice(&execution.stdout).unwrap();
    assert_eq!(execution["plan_id"], fresh["plan_id"]);
    assert_eq!(execution["execution_complete"], true);
    assert_eq!(execution["failed_operations"], serde_json::json!([]));
    assert_eq!(execution["candidate_paths"], fresh["candidate_paths"]);
    assert_eq!(
        std::fs::symlink_metadata(output.join(candidate.store_path.to_string())).unwrap_err().kind(),
        std::io::ErrorKind::NotFound,
        "fresh accepted plan must remove the candidate export"
    );
    rail_cmd(backend, state, output)
        .args(["store", "info", candidate.store_path.name()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no PathInfo matching"));
    rail_gc_retains_closure(backend, state, output, retained, child, fixture_key);
    let gc_facts = serde_json::json!({
        "planned": plan,
        "after_pin_candidate_paths": after["candidate_paths"],
        "fresh": fresh,
        "execution_complete": execution["execution_complete"],
        "execution_candidate_paths": execution["candidate_paths"],
    });
    (blocker, gc_facts)
}

fn rail_assert_snix_prechange(
    golden: &Value,
    root: &Path,
    reopen: &Value,
    archive: &Value,
    resigned: &Value,
    gc: &Value,
) {
    assert_eq!(reopen, &golden["reopen"], "same-key reopened PathInfo, closure, and reuse changed");
    rail_compare_prechange_archive(golden, archive, resigned);
    rail_compare_prechange_gc(golden, root, gc);
}

#[test]
fn admitted_backends_share_signed_core_gc_identity_and_profile_conformance_rail() {
    let golden: Value = serde_json::from_str(include_str!(
        "../.cairn/archive/2026-10-04-add-store-backend-selection/evidence/prechange-snix-rail-unsorted-observed-2026-10-04.json"
    ))
    .unwrap();
    let mut snix_signed_pathinfo = None;
    let fixture_key = trusted_public_key();
    let alternate_key = other_trusted_public_key("archive-cli-2");
    for backend in [crunch_store::StoreBackend::Snix, crunch_store::StoreBackend::Casita] {
        let fixed_root = (backend == crunch_store::StoreBackend::Snix)
            .then(|| std::env::var_os("MANTLE_RAIL_FIXTURE_ROOT"))
            .flatten()
            .map(std::path::PathBuf::from);
        let temp_root = fixed_root.is_none().then(|| tempfile::tempdir().unwrap());
        let root = fixed_root.as_deref().unwrap_or_else(|| temp_root.as_ref().unwrap().path());
        if fixed_root.is_some() {
            assert!(root.is_absolute() && !root.exists(), "fixed rail root must be fresh and absolute");
            std::fs::create_dir(root).unwrap();
        }
        let state = root.join("state");
        let output = root.join("output");
        let (retained, child, candidate) =
            rail_seed(backend, &state, &output, &root.join("second-test-only-signing-key"), &fixture_key);
        if backend == crunch_store::StoreBackend::Snix {
            rail_compare_prechange_seed(&golden, &output, [&retained, &child, &candidate]);
        }
        let signed_bytes = serde_json::to_vec(&retained).unwrap();
        if let Some(snix_bytes) = &snix_signed_pathinfo {
            assert_eq!(&signed_bytes, snix_bytes, "equal signing keys and NARs must agree across backends");
        } else {
            snix_signed_pathinfo = Some(signed_bytes);
        }
        let reopen = rail_reopen_and_reuse(backend, &state, &output, &retained, &child);
        let (capabilities, archive) = rail_archive_roundtrip(
            backend,
            root,
            &retained,
            [&retained, &child, &candidate],
            &fixture_key,
            &alternate_key,
        );
        rail_check_profile(backend, root, &capabilities);
        let (signatures, resigned) =
            rail_sign_and_check_mixed_backend(backend, root, &retained, &fixture_key, &alternate_key);
        let (blocker, gc) = rail_gc(backend, &state, &output, &retained, &child, &candidate, &fixture_key);
        if backend == crunch_store::StoreBackend::Snix {
            rail_assert_snix_prechange(&golden, root, &reopen, &archive, &resigned, &gc);
        }
        rail_check_overlay(backend, root, &retained);
        rail_check_atomic_batch(backend, root, &fixture_key);
        rail_check_unsigned(backend, root);
        rail_check_rust_cache(backend, root);
        println!(
            "BACKEND_CORE_RAIL {} {}",
            backend.as_str(),
            serde_json::json!({
                "retained_store_path": retained.store_path.to_string(),
                "nar_sha256": data_encoding::HEXLOWER.encode(&retained.nar_sha256),
                "stale_plan_blocker": blocker,
                "signed_count_after_store_sign": signatures,
                "profile_max_root_changes": capabilities["max_root_changes"],
            })
        );
        if fixed_root.is_some() {
            std::fs::remove_dir_all(root).unwrap();
        }
    }
}
