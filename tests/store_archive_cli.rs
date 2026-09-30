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
