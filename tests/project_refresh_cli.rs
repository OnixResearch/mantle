use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use assert_cmd::Command;
use crunch_project::HashAlgo;
use crunch_project::LockEntry;
use crunch_project::LockedHash;
use crunch_project::LockedKind;
use crunch_project::Lockfile;
use crunch_project::generate_inputs_ncl;
use digest::Digest;
use flate2::Compression;
use flate2::write::GzEncoder;
use nix_compat::nixhash::NixHash;
use snix_castore::Node;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_castore::import::fs::ingest_path;
use snix_store::nar::write_nar;
use snix_store::utils::AsyncIoBridge;
use tempfile::TempDir;
use tempfile::tempdir;

const HTTP_POLL_SLEEP_MS: u64 = 10;
const HTTP_READ_BUFFER_BYTES: usize = 4096;
const COMMAND_TIMEOUT_MS: u32 = 100;
const COMMAND_OUTPUT_LIMIT_BYTES: u32 = 8;
const COMMAND_SUCCESS_STATUS: i32 = 0;
const TRUST_TEST_KEYPAIR: &str =
    "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn freshness_digest(value: &str) -> String {
    format!("blake3:{}", blake3::hash(value.as_bytes()).to_hex())
}

fn locked_freshness(name: &str, value: &str) -> crunch_project::LockedFreshnessValue {
    crunch_project::LockedFreshnessValue {
        input_name: name.to_string(),
        value_digest: freshness_digest(value),
    }
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

fn init_project(dir: &Path) {
    crunch().arg("init").current_dir(dir).assert().success();
}

fn write_project_files(dir: &Path, manifest: &str, lock: &Lockfile) {
    std::fs::write(dir.join("mantle-project.ncl"), manifest).unwrap();
    std::fs::write(dir.join("mantle.lock"), lock.clone().to_json().unwrap()).unwrap();
    std::fs::create_dir_all(dir.join(".mantle")).unwrap();
    std::fs::write(dir.join(".mantle/inputs.ncl"), generate_inputs_ncl(lock.clone())).unwrap();
}

fn read_lock(dir: &Path) -> Lockfile {
    let text = std::fs::read_to_string(dir.join("mantle.lock")).unwrap();
    Lockfile::from_json(text).unwrap()
}

fn run_git(dir: &Path, args: &[&str]) -> String {
    let output = ProcessCommand::new("git").args(args).current_dir(dir).output().unwrap();
    assert!(output.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

struct HttpProbeServer {
    url: String,
    requests: Arc<AtomicU32>,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Drop for HttpProbeServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.url.trim_start_matches("http://"));
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn spawn_http_probe_server(body: &'static str) -> HttpProbeServer {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(AtomicU32::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let requests_ref = requests.clone();
    let stop_ref = stop.clone();
    let handle = thread::spawn(move || {
        while !stop_ref.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _peer)) => handle_http_probe_connection(&mut stream, body, &requests_ref),
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(HTTP_POLL_SLEEP_MS));
                }
                Err(_err) => break,
            }
        }
    });
    HttpProbeServer {
        url: format!("http://{addr}"),
        requests,
        stop,
        handle: Some(handle),
    }
}

fn handle_http_probe_connection(stream: &mut TcpStream, body: &str, requests: &Arc<AtomicU32>) {
    let mut buffer = [0u8; HTTP_READ_BUFFER_BYTES];
    let _ = stream.read(&mut buffer);
    requests.fetch_add(1, Ordering::SeqCst);
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
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

    let clone = ProcessCommand::new("git").args(["clone", "--bare"]).arg(repo).arg(&bare).output().unwrap();
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
    assert!(
        checkout_out.status.success(),
        "git checkout failed: {}",
        String::from_utf8_lossy(&checkout_out.stderr)
    );

    tmp
}

fn compute_recursive_hash(path: &Path, algo: HashAlgo) -> String {
    let root = path.to_path_buf();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async move {
        let blob_service = MemoryBlobService::default();
        let directory_service =
            RedbDirectoryService::new_temporary("test-refresh".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap();
        let node = ingest_path::<_, _, _, &[u8]>(blob_service.clone(), directory_service.clone(), &root, None)
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
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service).await.unwrap();
            let digest: [u8; 32] = hasher.finalize().into();
            NixHash::Sha256(digest)
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service).await.unwrap();
            let digest: [u8; 64] = hasher.finalize().into();
            NixHash::Sha512(Box::new(digest))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service).await.unwrap();
            NixHash::Blake3(*blake3::Hasher::finalize(&hasher).as_bytes())
        }
    };
    hash.to_sri_string()
}

fn flat_sha256_sri(bytes: &[u8]) -> String {
    let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
    NixHash::Sha256(digest).to_sri_string()
}

fn write_input_trust_signature(dir: &Path, input_name: &str, signature_path: &str, hash_value: &str) -> String {
    let (signing_key, verifying_key) = nix_compat::narinfo::parse_keypair(TRUST_TEST_KEYPAIR).unwrap();
    let subject = crunch_project::TrustSubject::input(input_name.to_string());
    let payload = crunch_project::trust_signature_payload(
        &subject,
        crunch_project::TrustDigestBinding::ContentHash,
        &HashAlgo::Sha256,
        hash_value,
    );
    let signature = signing_key.sign(payload.as_bytes()).to_owned().to_string();
    std::fs::write(dir.join(signature_path), signature).unwrap();
    verifying_key.to_string()
}

fn input_trust_policy_ncl(signature_path: &str, trusted_public_key: &str) -> String {
    let required_signer = trusted_public_key.split(':').next().unwrap();
    format!(
        r#"{{
        verifier = "ed25519-detached",
        signatures = [{{ type = "local-file", path = {} }}],
        trusted_public_keys = [{}],
        required_signers = [{}],
        quorum = 1,
        digest_binding = "content-hash",
      }}"#,
        quoted(signature_path),
        quoted(trusted_public_key),
        quoted(required_signer),
    )
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
fn build_fetch_policy_refresh_uses_expected_hash_without_network_resolution() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let remote_url = "https://example.invalid/unreachable.tar.gz";
    let expected_hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "remote-src",
      kind = {{ type = "tarball", url = {} }},
      hash = {{ algo = 'sha256, expected = {} }},
      fetch_policy = "build-fetch-action",
    }},
  ],
  patches = [],
}}
"#,
        quoted(remote_url),
        quoted(expected_hash),
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    crunch().arg("refresh").current_dir(dir.path()).assert().success();
    let lock = read_lock(dir.path());
    let entry = lock.inputs.get("remote-src").expect("remote-src should be locked");

    assert_eq!(entry.hash.value, expected_hash);
    assert_eq!(entry.fetch_policy, crunch_project::InputFetchPolicy::BuildFetchAction);
    let generated = std::fs::read_to_string(dir.path().join(".mantle/inputs.ncl")).unwrap();
    assert!(generated.contains("fetch_policy = \"build-fetch-action\""));

    crunch().arg("list-stale").current_dir(dir.path()).assert().success();
}

#[test]
fn refresh_trusted_file_input_writes_lock_evidence_and_show_reports_claim() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let source = dir.path().join("trusted.txt");
    std::fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let expected_hash = flat_sha256_sri(b"payload\n");
    let trusted_public_key = write_input_trust_signature(dir.path(), "pkg", "pkg.sig", &expected_hash);
    let trust_policy = input_trust_policy_ncl("pkg.sig", &trusted_public_key);
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      trust = {},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        trust_policy,
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    crunch().arg("refresh").current_dir(dir.path()).assert().success();

    let lock = read_lock(dir.path());
    let entry = lock.inputs.get("pkg").expect("trusted input should be locked");
    let trust = entry.trust.as_ref().expect("lock should carry trust evidence");
    assert_eq!(entry.hash.value, expected_hash);
    assert_eq!(trust.signers, vec!["cache.example.com-1".to_string()]);
    assert!(trust.claim.contains(crunch_project::PROJECT_INPUT_TRUST_NON_CLAIM));

    let show = crunch().arg("show").current_dir(dir.path()).assert().success();
    let stdout = String::from_utf8_lossy(&show.get_output().stdout);
    assert!(stdout.contains("trust: 1 signer(s) via ed25519-detached"), "stdout was: {stdout}");
    assert!(stdout.contains("does not prove build reproducibility"), "stdout was: {stdout}");
}

#[test]
fn refresh_trusted_file_input_with_wrong_signature_rejects_lock_write() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let source = dir.path().join("trusted.txt");
    std::fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let wrong_hash = flat_sha256_sri(b"other\n");
    let trusted_public_key = write_input_trust_signature(dir.path(), "pkg", "pkg.sig", &wrong_hash);
    let trust_policy = input_trust_policy_ncl("pkg.sig", &trusted_public_key);
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      trust = {},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        trust_policy,
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    let assert = crunch().arg("refresh").current_dir(dir.path()).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("failed: pkg:"), "stderr was: {stderr}");
    assert!(stderr.contains("missing required trust signer"), "stderr was: {stderr}");
    assert!(stderr.contains("refresh failed for 1 item(s)"), "stderr was: {stderr}");

    let lock = read_lock(dir.path());
    assert!(lock.inputs.is_empty(), "failed trust refresh must not write lock evidence: {lock:?}");
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
    lock.inputs.insert("stale".into(), LockEntry {
        kind: LockedKind::File { url: stale_url.clone() },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-old=".into(),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: None,
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);

    let lock_before = std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap();
    let inputs_before = std::fs::read_to_string(dir.path().join(".mantle/inputs.ncl")).unwrap();

    let assert = crunch().arg("list-stale").current_dir(dir.path()).assert().failure();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stdout.contains("stale"), "stdout was: {stdout}");
    assert!(!stdout.contains("all inputs up to date"), "stdout was: {stdout}");
    assert!(stderr.contains("failed: broken:"), "stderr was: {stderr}");
    assert!(stderr.contains("stale check failed for 1 item(s)"), "stderr was: {stderr}");

    let lock_after = std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap();
    let inputs_after = std::fs::read_to_string(dir.path().join(".mantle/inputs.ncl")).unwrap();
    assert_eq!(lock_after, lock_before);
    assert_eq!(inputs_after, inputs_before);
}

#[test]
fn freshness_http_json_template_refreshes_selected_stale_input() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let server = spawn_http_probe_server(r#"{"version":"v2"}"#);

    let old_source = dir.path().join("pkg-v1.txt");
    let new_source = dir.path().join("pkg-v2.txt");
    std::fs::write(&old_source, "old\n").unwrap();
    std::fs::write(&new_source, "new\n").unwrap();
    let source_template = format!("file://{}/pkg-{{{{ freshness.value }}}}.txt", dir.path().display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      freshness = {{ type = "http-json", url = {}, pointer = "/version" }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_template),
        quoted(&server.url),
    );
    let mut lock = Lockfile::new();
    lock.inputs.insert("pkg".into(), LockEntry {
        kind: LockedKind::File {
            url: format!("file://{}", old_source.display()),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: flat_sha256_sri(b"old\n"),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: Some(locked_freshness("pkg", "v1")),
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);

    crunch().arg("refresh").arg("pkg").current_dir(dir.path()).assert().success();

    let refreshed = read_lock(dir.path());
    assert_eq!(refreshed.inputs["pkg"].hash.value, flat_sha256_sri(b"new\n"));
    assert_eq!(refreshed.inputs["pkg"].freshness.as_ref().unwrap().value_digest, freshness_digest("v2"));
    assert!(server.requests.load(Ordering::SeqCst) > 0);
    let generated = std::fs::read_to_string(dir.path().join(".mantle/inputs.ncl")).unwrap();
    assert!(generated.contains("freshness_value_digest"));
}

#[test]
fn freshness_git_ref_probe_reports_stale_without_mutating_lock() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let (repo, old_rev) = create_git_repo(dir.path());
    std::fs::write(repo.join("hello.txt"), "hello from git v2\n").unwrap();
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "second"]);
    let new_rev = run_git(&repo, &["rev-parse", "HEAD"]);
    assert_ne!(old_rev, new_rev);

    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "repo",
      kind = {{ type = "git", repository = {}, reference = {{ ref_type = "branch", ref_value = "main" }} }},
      freshness = {{ type = "git-ref", repository = {}, reference = "refs/heads/main" }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&repo.display().to_string()),
        quoted(&repo.display().to_string()),
    );
    let mut lock = Lockfile::new();
    lock.inputs.insert("repo".into(), LockEntry {
        kind: LockedKind::Git {
            repository: repo.display().to_string(),
            rev: old_rev.clone(),
            ref_name: Some("main".to_string()),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-old=".to_string(),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: Some(locked_freshness("repo", &old_rev)),
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);
    let lock_before = std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap();

    let assert = crunch().arg("list-stale").current_dir(dir.path()).assert().success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("repo"), "stdout was: {stdout}");
    let lock_after = std::fs::read_to_string(dir.path().join("mantle.lock")).unwrap();
    assert_eq!(lock_before, lock_after);
}

#[test]
fn freshness_local_directory_probe_updates_lock_digest() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let source = dir.path().join("pkg.txt");
    let watched = dir.path().join("watched");
    std::fs::create_dir_all(&watched).unwrap();
    std::fs::write(&source, "payload\n").unwrap();
    std::fs::write(watched.join("state.txt"), "state-v2\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      freshness = {{ type = "local-directory", path = "watched" }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
    );
    let mut lock = Lockfile::new();
    lock.inputs.insert("pkg".into(), LockEntry {
        kind: LockedKind::File { url: source_url },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-old=".into(),
        },
        patches: vec![],
        mirrors: vec![],
        fetch_policy: crunch_project::InputFetchPolicy::GenerationMaterial,
        freshness: Some(locked_freshness("pkg", "state-v1")),
        trust: None,
    });
    write_project_files(dir.path(), &manifest, &lock);

    crunch().arg("refresh").current_dir(dir.path()).assert().success();

    let refreshed = read_lock(dir.path());
    assert_ne!(refreshed.inputs["pkg"].freshness.as_ref().unwrap().value_digest, freshness_digest("state-v1"));
    assert_eq!(refreshed.inputs["pkg"].hash.value, flat_sha256_sri(b"payload\n"));
}

#[test]
fn freshness_command_missing_timeout_and_output_limit_fail_deterministically() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let source = dir.path().join("pkg.txt");
    std::fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let sleep_script = dir.path().join("sleep-probe.sh");
    let loud_script = dir.path().join("loud-probe.sh");
    std::fs::write(&sleep_script, "#!/bin/sh\nwhile :; do :; done\n").unwrap();
    std::fs::write(&loud_script, "#!/bin/sh\nprintf 123456789\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&sleep_script, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::set_permissions(&loud_script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "missing",
      kind = {{ type = "file", url = {} }},
      freshness = {{
        type = "command",
        requires_network = false,
        command = {{ argv = ["definitely-missing-mantle-freshness-probe"], cwd = ".", env = [], timeout_ms = {}, output_limit_bytes = {}, success_statuses = [{}], utf8_required = true }},
      }},
    }},
    {{
      name = "timeout",
      kind = {{ type = "file", url = {} }},
      freshness = {{
        type = "command",
        requires_network = false,
        command = {{ argv = [{}], cwd = ".", env = [], timeout_ms = {}, output_limit_bytes = {}, success_statuses = [{}], utf8_required = true }},
      }},
    }},
    {{
      name = "too-loud",
      kind = {{ type = "file", url = {} }},
      freshness = {{
        type = "command",
        requires_network = false,
        command = {{ argv = [{}], cwd = ".", env = [], timeout_ms = {}, output_limit_bytes = {}, success_statuses = [{}], utf8_required = true }},
      }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        COMMAND_TIMEOUT_MS,
        COMMAND_OUTPUT_LIMIT_BYTES,
        COMMAND_SUCCESS_STATUS,
        quoted(&source_url),
        quoted(&sleep_script.display().to_string()),
        COMMAND_TIMEOUT_MS,
        COMMAND_OUTPUT_LIMIT_BYTES,
        COMMAND_SUCCESS_STATUS,
        quoted(&source_url),
        quoted(&loud_script.display().to_string()),
        COMMAND_TIMEOUT_MS,
        COMMAND_OUTPUT_LIMIT_BYTES,
        COMMAND_SUCCESS_STATUS,
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    let assert = crunch().arg("list-stale").current_dir(dir.path()).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("failed: missing:"), "stderr was: {stderr}");
    assert!(stderr.contains("failed: timeout:"), "stderr was: {stderr}");
    assert!(stderr.contains("failed: too-loud:"), "stderr was: {stderr}");
    assert!(stderr.contains("not found"), "stderr was: {stderr}");
    assert!(stderr.contains("timed out"), "stderr was: {stderr}");
    assert!(stderr.contains("exceeds limit"), "stderr was: {stderr}");
}

#[test]
fn freshness_no_network_mode_does_not_contact_http_probe() {
    let dir = TempDir::new().unwrap();
    init_project(dir.path());
    let server = spawn_http_probe_server("v2\n");
    let source = dir.path().join("pkg.txt");
    std::fs::write(&source, "payload\n").unwrap();
    let source_url = format!("file://{}", source.display());
    let manifest = format!(
        r#"{{
  version = "1.0.0",
  inputs = [
    {{
      name = "pkg",
      kind = {{ type = "file", url = {} }},
      freshness = {{ type = "http-text", url = {} }},
    }},
  ],
  patches = [],
}}
"#,
        quoted(&source_url),
        quoted(&server.url),
    );
    write_project_files(dir.path(), &manifest, &Lockfile::new());

    let assert = crunch().arg("list-stale").arg("--no-network").current_dir(dir.path()).assert().failure();
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("network-required: pkg"), "stderr was: {stderr}");
    assert_eq!(server.requests.load(Ordering::SeqCst), 0);
}
