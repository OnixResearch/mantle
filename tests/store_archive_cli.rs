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
