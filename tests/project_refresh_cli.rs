use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use assert_cmd::Command;
use crunch_project::{HashAlgo, LockEntry, Lockfile, LockedHash, LockedKind, generate_inputs_ncl};
use digest::Digest;
use flate2::Compression;
use flate2::write::GzEncoder;
use nix_compat::nixhash::NixHash;
use snix_castore::Node;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::{RedbDirectoryService, RedbDirectoryServiceConfig};
use snix_castore::import::fs::ingest_path;
use snix_store::nar::write_nar;
use snix_store::utils::AsyncIoBridge;
use tempfile::{TempDir, tempdir};

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

fn init_project(dir: &Path) {
    crunch().arg("init").current_dir(dir).assert().success();
}

fn write_project_files(dir: &Path, manifest: &str, lock: &Lockfile) {
    std::fs::write(dir.join("crunch-project.ncl"), manifest).unwrap();
    std::fs::write(dir.join("crunch.lock"), lock.to_json().unwrap()).unwrap();
    std::fs::create_dir_all(dir.join(".crunch")).unwrap();
    std::fs::write(dir.join(".crunch/inputs.ncl"), generate_inputs_ncl(lock)).unwrap();
}

fn read_lock(dir: &Path) -> Lockfile {
    let text = std::fs::read_to_string(dir.join("crunch.lock")).unwrap();
    Lockfile::from_json(&text).unwrap()
}

fn run_git(dir: &Path, args: &[&str]) -> String {
    let output = ProcessCommand::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn create_git_repo(parent: &Path) -> (PathBuf, String) {
    let repo = parent.join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    run_git(&repo, &["init"]);
    run_git(&repo, &["branch", "-M", "main"]);
    run_git(&repo, &["config", "user.email", "test@example.com"]);
    run_git(&repo, &["config", "user.name", "Test User"]);
    std::fs::write(repo.join("hello.txt"), "hello from git\n").unwrap();
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "initial"]);
    let rev = run_git(&repo, &["rev-parse", "HEAD"]);
    (repo, rev)
}

fn checkout_git_tree(repo: &Path, rev: &str) -> TempDir {
    let tmp = tempdir().unwrap();
    let bare = tmp.path().join("source.git");
    let checkout = tmp.path().join("checkout");

    let clone = ProcessCommand::new("git")
        .args(["clone", "--bare"])
        .arg(repo)
        .arg(&bare)
        .output()
        .unwrap();
    assert!(clone.status.success(), "git clone failed: {}", String::from_utf8_lossy(&clone.stderr));

    std::fs::create_dir_all(&checkout).unwrap();
    let checkout_out = ProcessCommand::new("git")
        .arg("--git-dir")
        .arg(&bare)
        .arg("--work-tree")
        .arg(&checkout)
        .args(["checkout", rev, "--", "."])
        .output()
        .unwrap();
    assert!(checkout_out.status.success(), "git checkout failed: {}", String::from_utf8_lossy(&checkout_out.stderr));

    tmp
}

fn compute_recursive_hash(path: &Path, algo: HashAlgo) -> String {
    let root = path.to_path_buf();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async move {
        let blob_service = MemoryBlobService::default();
        let directory_service = RedbDirectoryService::new_temporary(
            "test-refresh".to_string(),
            RedbDirectoryServiceConfig::default(),
        )
        .unwrap();
        let node = ingest_path::<_, _, _, &[u8]>(
            blob_service.clone(),
            directory_service.clone(),
            &root,
            None,
        )
        .await
        .unwrap();
        nar_hash_to_sri(&node, algo, blob_service, directory_service).await
    })
}

async fn nar_hash_to_sri(
    node: &Node,
    algo: HashAlgo,
    blob_service: MemoryBlobService,
    directory_service: RedbDirectoryService,
) -> String {
    let hash = match algo {
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .unwrap();
            let digest: [u8; 32] = hasher.finalize().into();
            NixHash::Sha256(digest)
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .unwrap();
            let digest: [u8; 64] = hasher.finalize().into();
            NixHash::Sha512(Box::new(digest))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .unwrap();
            NixHash::Blake3(*blake3::Hasher::finalize(&hasher).as_bytes())
        }
    };
    hash.to_sri_string()
}

fn flat_sha256_sri(bytes: &[u8]) -> String {
    let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
    NixHash::Sha256(digest).to_sri_string()
}

fn create_tarball(parent: &Path) -> (PathBuf, PathBuf) {
    let source_dir = parent.join("tar-src");
    std::fs::create_dir_all(source_dir.join("nested")).unwrap();
    std::fs::write(source_dir.join("nested/hello.txt"), "hello tarball\n").unwrap();
    std::fs::write(source_dir.join("root.txt"), "root\n").unwrap();

    let tarball = parent.join("hello.tar.gz");
    let file = std::fs::File::create(&tarball).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(encoder);
    builder.append_dir_all("hello-1.0", &source_dir).unwrap();
    builder.finish().unwrap();

    (tarball, source_dir)
}

#[test]
fn refresh_git_input_locks_resolved_rev_and_tree_hash() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let (repo, rev) = create_git_repo(dir.path());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "repo",
      kind = {{
        type = "git",
        repository = {},
        reference = {{ ref_type = "branch", ref_value = "main" }},
      }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&repo.display().to_string())
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    crunch().arg("refresh").current_dir(dir.path()).assert().success();

    let expected_tree = {
        let checkout = checkout_git_tree(&repo, &rev);
        compute_recursive_hash(&checkout.path().join("checkout"), HashAlgo::Sha256)
    };
    let lock = read_lock(dir.path());
    match &lock.inputs["repo"].kind {
        LockedKind::Git { rev: locked_rev, .. } => assert_eq!(locked_rev, &rev),
        other => panic!("expected git lock entry, got {other:?}"),
    }
    assert_eq!(lock.inputs["repo"].hash.value, expected_tree);
}

#[test]
fn refresh_tarball_uses_unpacked_tree_hash_not_archive_bytes() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let (tarball, source_dir) = create_tarball(dir.path());
    let tarball_url = format!("file://{}", tarball.display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "hello-src",
      kind = {{ type = "tarball", url = {} }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&tarball_url)
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    crunch().arg("refresh").current_dir(dir.path()).assert().success();

    let lock = read_lock(dir.path());
    let expected_tree = compute_recursive_hash(&source_dir, HashAlgo::Sha256);
    let flat_archive = flat_sha256_sri(&std::fs::read(&tarball).unwrap());
    assert_eq!(lock.inputs["hello-src"].hash.value, expected_tree);
    assert_ne!(lock.inputs["hello-src"].hash.value, flat_archive);
}

#[test]
fn refresh_partial_failure_writes_successes_and_exits_nonzero() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let source = dir.path().join("good.txt");
    std::fs::write(&source, "good\n").unwrap();
    let good_url = format!("file://{}", source.display());
    let bad_url = format!("file://{}/missing.txt", dir.path().display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{ name = "good", kind = {{ type = "file", url = {} }} }},
    {{ name = "bad", kind = {{ type = "file", url = {} }} }},
  ],
  patches = [],
}}
"#,
        quoted(&good_url),
        quoted(&bad_url),
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    let assert = crunch().arg("refresh").current_dir(dir.path()).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("updated: good"), "stderr was: {stderr}");
    assert!(stderr.contains("failed: bad:"), "stderr was: {stderr}");
    assert!(stderr.contains("refresh failed for 1 item(s)"), "stderr was: {stderr}");

    let lock = read_lock(dir.path());
    assert!(lock.inputs.contains_key("good"));
    assert!(!lock.inputs.contains_key("bad"));
}

#[test]
fn list_stale_reports_stale_and_failed_without_mutating_files() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());

    let source = dir.path().join("stale.txt");
    std::fs::write(&source, "current\n").unwrap();
    let stale_url = format!("file://{}", source.display());
    let broken_url = format!("file://{}/missing.txt", dir.path().display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{ name = "stale", kind = {{ type = "file", url = {} }} }},
    {{ name = "broken", kind = {{ type = "file", url = {} }} }},
  ],
  patches = [],
}}
"#,
        quoted(&stale_url),
        quoted(&broken_url),
    );

    let mut lock = Lockfile::new();
    lock.inputs.insert(
        "stale".into(),
        LockEntry {
            kind: LockedKind::File { url: stale_url.clone() },
            hash: LockedHash {
                algo: HashAlgo::Sha256,
                value: "sha256-old=".into(),
            },
            patches: vec![],
            mirrors: vec![],
        },
    );
    write_project_files(dir.path(), &manifest, &lock);

    let lock_before = std::fs::read_to_string(dir.path().join("crunch.lock")).unwrap();
    let inputs_before = std::fs::read_to_string(dir.path().join(".crunch/inputs.ncl")).unwrap();

    let assert = crunch().arg("list-stale").current_dir(dir.path()).assert().failure();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stdout.contains("stale"), "stdout was: {stdout}");
    assert!(!stdout.contains("all inputs up to date"), "stdout was: {stdout}");
    assert!(stderr.contains("failed: broken:"), "stderr was: {stderr}");
    assert!(stderr.contains("stale check failed for 1 item(s)"), "stderr was: {stderr}");

    let lock_after = std::fs::read_to_string(dir.path().join("crunch.lock")).unwrap();
    let inputs_after = std::fs::read_to_string(dir.path().join(".crunch/inputs.ncl")).unwrap();
    assert_eq!(lock_after, lock_before);
    assert_eq!(inputs_after, inputs_before);
}
