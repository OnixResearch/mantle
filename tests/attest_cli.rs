use std::collections::BTreeMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::Shutdown;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use assert_cmd::Command;
use crunch_build::generate_keypair;
use crunch_build::load_keypair;
use crunch_build::sign_pathinfo;
use crunch_project::HashAlgo;
use crunch_project::InputKind;
use crunch_project::LockEntry;
use crunch_project::LockedHash;
use crunch_project::LockedKind;
use crunch_project::LockedPatch;
use crunch_project::LockedPatchSource;
use crunch_project::Lockfile;
use crunch_project::ManifestInput;
use crunch_project::PatchDef;
use crunch_project::PatchSource;
use crunch_project::ProjectManifest;
use crunch_project::SchemaVersion;
use crunch_store::PersistOutputRequest;
use crunch_store::StoreConfig;
use crunch_store::StoreHandle;
use nix_compat::nixbase32;
use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_castore::SymlinkTarget;
use snix_castore::blobservice::ObjectStoreBlobService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_store::nar::write_nar;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use snix_store::pathinfoservice::RedbPathInfoService;
use snix_store::pathinfoservice::RedbPathInfoServiceConfig;
use snix_store::utils::AsyncIoBridge;
use tempfile::TempDir;

const STORE_DIR: &str = "/crunch/store";

fn can_build() -> bool {
    Path::new("/nix/store").exists()
        && std::process::Command::new("bwrap").arg("--version").output().is_ok_and(|o| o.status.success())
}

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should build")
}

fn build_simple_ncl_with_nix_compat(
    work_dir: &Path,
    store_dir: &Path,
    state_dir: &Path,
    name: &str,
    nix_compat: bool,
) -> serde_json::Value {
    let ncl_file = work_dir.join(format!("{name}.ncl"));
    let ncl = format!(
        r#"let crunch = import "lib.ncl" in
{{
  name = "{name}",
  builder = "/bin/sh",
  args = ["-c", "echo workflow > $out"],
  addressing_mode = 'input-addressed,
}} | crunch.Derivation"#
    );
    std::fs::write(&ncl_file, ncl).unwrap();

    let mut command = crunch_cmd();
    command.arg("--json").arg("--store").arg(store_dir).arg("--state-dir").arg(state_dir);
    if nix_compat {
        command.arg("--nix-compat");
    }
    let output = command.arg("build").arg("--no-substitute").arg("-I").arg(work_dir).arg(&ncl_file).output().unwrap();

    assert!(output.status.success(), "build failed: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}

fn build_simple_ncl(work_dir: &Path, store_dir: &Path, state_dir: &Path, name: &str) -> serde_json::Value {
    build_simple_ncl_with_nix_compat(work_dir, store_dir, state_dir, name, false)
}

fn load_signed_output_pathinfo(state_dir: &Path, logical_path: &str, store_dir: &str) -> PathInfo {
    let (store_path, suffix) = StorePath::from_absolute_path_full_with_prefix(logical_path, store_dir).unwrap();
    assert!(suffix.as_os_str().is_empty(), "logical path should not have suffix: {logical_path}");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let svc = RedbPathInfoService::new("attest-cli-remote-cache".to_string(), RedbPathInfoServiceConfig {
            path: Some(state_dir.join("pathinfo.redb")),
            read_only: true,
            cache_size: None,
        })
        .await
        .unwrap();

        let path_info = svc.get(*store_path.digest()).await.unwrap().unwrap();
        assert_eq!(path_info.store_path, store_path);
        assert!(!path_info.signatures.is_empty(), "seeded pathinfo should be signed");
        path_info
    })
}

fn render_nar_bytes(state_dir: &Path, path_info: &PathInfo) -> Vec<u8> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let blob_service = ObjectStoreBlobService::new_local(state_dir.join("blobs")).unwrap();
        let directory_service =
            RedbDirectoryService::new_temporary("attest-cli-render".to_string(), RedbDirectoryServiceConfig::default())
                .unwrap();
        let mut nar_bytes = Vec::new();
        write_nar(AsyncIoBridge(&mut nar_bytes), &path_info.node, blob_service, directory_service)
            .await
            .unwrap();
        assert!(!nar_bytes.is_empty(), "rendered NAR should not be empty");
        nar_bytes
    })
}

fn trusted_public_key_arg(state_dir: &Path) -> String {
    let key_contents = std::fs::read_to_string(state_dir.join("signing-key")).unwrap();
    let keypair = load_keypair(&key_contents).unwrap();
    let trusted_key = keypair.verifying_key.to_string();
    assert!(trusted_key.contains(':'), "trusted key should include name separator");
    trusted_key
}

struct FakeBinaryCache {
    url: String,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl FakeBinaryCache {
    fn serve(path_info: &PathInfo, nar_bytes: Vec<u8>) -> Self {
        let digest_base32 = nixbase32::encode(path_info.store_path.digest());
        let narinfo_path = format!("/{digest_base32}.narinfo");
        let nar_relative_url = format!("nar/{}.nar", path_info.store_path);
        let nar_path = format!("/{nar_relative_url}");
        let narinfo_text = {
            let mut narinfo = path_info.to_narinfo();
            narinfo.url = &nar_relative_url;
            narinfo.to_string()
        };
        assert!(narinfo_text.contains("StorePath: /nix/store/"));
        assert!(narinfo_text.contains("NarHash: sha256:"));

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !stop_thread.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _peer)) => {
                        respond_once(&mut stream, &narinfo_path, narinfo_text.as_bytes(), &nar_path, &nar_bytes);
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            url: format!("http://{addr}"),
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for FakeBinaryCache {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let wake_addr = self.url.trim_start_matches("http://");
        let _ = TcpStream::connect(wake_addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn respond_once(stream: &mut TcpStream, narinfo_path: &str, narinfo_body: &[u8], nar_path: &str, nar_body: &[u8]) {
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut request_line = String::new();
    reader.read_line(&mut request_line).unwrap();
    if request_line.is_empty() {
        return;
    }

    let mut headers_done = false;
    while !headers_done {
        let mut header_line = String::new();
        reader.read_line(&mut header_line).unwrap();
        if header_line == "\r\n" || header_line.is_empty() {
            headers_done = true;
        }
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    let is_head = method == "HEAD";
    let (status_line, content_type, body) = if path == narinfo_path {
        ("HTTP/1.1 200 OK", "text/x-nix-narinfo", narinfo_body)
    } else if path == nar_path {
        ("HTTP/1.1 200 OK", "application/x-nix-nar", nar_body)
    } else {
        ("HTTP/1.1 404 Not Found", "text/plain", b"not found".as_slice())
    };

    write!(
        stream,
        "{status_line}\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .unwrap();
    if !is_head {
        stream.write_all(body).unwrap();
    }
    stream.flush().unwrap();
    let _ = stream.shutdown(Shutdown::Both);
}

struct SeededStore {
    state_dir: TempDir,
    output_dir: TempDir,
    root_logical_path: String,
    root_exported_path: String,
}

fn seed_store() -> SeededStore {
    let state_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let root_path = StorePath::from_name_and_digest_fixed("attest-root", [2u8; 20]).unwrap();
    let dep_path = StorePath::from_name_and_digest_fixed("attest-dep", [1u8; 20]).unwrap();
    std::fs::write(output_dir.path().join("target"), b"artifact target").unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut store = StoreHandle::open(StoreConfig {
            state_dir: state_dir.path().to_path_buf(),
            output_dir: output_dir.path().to_path_buf(),
            remote_cache_url: None,
            fallback_mode: crunch_store::StoreFallbackMode::Practical,
            store_dir: STORE_DIR.to_string(),
        })
        .await
        .unwrap();

        let (keypair, _) = generate_keypair();
        let dep_info = signed_pathinfo(dep_path.clone(), Vec::new(), &keypair);
        let root_info = signed_pathinfo(root_path.clone(), vec![dep_path], &keypair);
        let node = Node::Symlink {
            target: SymlinkTarget::try_from("target").unwrap(),
        };

        store
            .persist_and_export_signed_output(PersistOutputRequest {
                output_name: "out",
                output_path: &root_path,
                path_info: root_info,
                final_node: node.clone(),
                provenance: None,
                is_root: true,
                root_source: None,
            })
            .await
            .unwrap();
        store
            .persist_and_export_signed_output(PersistOutputRequest {
                output_name: "out",
                output_path: &StorePath::from_name_and_digest_fixed("attest-dep", [1u8; 20]).unwrap(),
                path_info: dep_info,
                final_node: node,
                provenance: None,
                is_root: false,
                root_source: None,
            })
            .await
            .unwrap();
    });

    let root_exported_path = root_path.to_absolute_path_with_prefix(output_dir.path().to_str().unwrap());
    SeededStore {
        state_dir,
        output_dir,
        root_logical_path: root_path.to_absolute_path_with_prefix(STORE_DIR),
        root_exported_path,
    }
}

fn signed_pathinfo(
    store_path: StorePath<String>,
    references: Vec<StorePath<String>>,
    keypair: &crunch_build::KeyPair,
) -> PathInfo {
    let mut path_info = PathInfo {
        store_path,
        node: Node::Symlink {
            target: SymlinkTarget::try_from("target").unwrap(),
        },
        references,
        nar_size: 1,
        nar_sha256: [0xAB; 32],
        signatures: Vec::new(),
        deriver: None,
        ca: None,
    };
    let signature_name = sign_pathinfo(&mut path_info, &keypair.signing_key);
    assert!(!signature_name.is_empty());
    assert_eq!(path_info.signatures.len(), 1);
    path_info
}

#[test]
fn attest_build_workflow_and_cached_second_build() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work_dir = tempfile::tempdir().unwrap();
    let store_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();

    let first = build_simple_ncl(work_dir.path(), store_dir.path(), state_dir.path(), "attest-build-workflow");
    assert_eq!(first["counts"]["built_total"], 1);
    assert_eq!(first["counts"]["cached_total"], 0);
    let output_path = first["outcomes"][0]["outputs"][0]["path"].as_str().unwrap();
    let attestation_path = first["outcomes"][0]["outputs"][0]["artifact_attestation"]["path"].as_str().unwrap();
    assert!(Path::new(output_path).exists(), "missing output path: {output_path}");
    assert!(Path::new(attestation_path).exists(), "missing attestation path: {attestation_path}");

    let verify_artifact = crunch_cmd()
        .arg("--store")
        .arg(store_dir.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("attest")
        .arg("verify")
        .arg("artifact")
        .arg(output_path)
        .output()
        .unwrap();
    assert!(
        verify_artifact.status.success(),
        "artifact verify failed: {}",
        String::from_utf8_lossy(&verify_artifact.stderr)
    );

    let verify_closure = crunch_cmd()
        .arg("--store")
        .arg(store_dir.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("attest")
        .arg("verify")
        .arg("closure")
        .arg(output_path)
        .output()
        .unwrap();
    assert!(
        verify_closure.status.success(),
        "closure verify failed: {}",
        String::from_utf8_lossy(&verify_closure.stderr)
    );

    let second = build_simple_ncl(work_dir.path(), store_dir.path(), state_dir.path(), "attest-build-workflow");
    assert_eq!(second["counts"]["built_total"], 0);
    assert_eq!(second["counts"]["cached_total"], 1);
    assert_eq!(second["outcomes"][0]["cached"], true);
    let cached_attestation_path = second["outcomes"][0]["outputs"][0]["artifact_attestation"]["path"].as_str().unwrap();
    assert!(
        Path::new(cached_attestation_path).exists(),
        "missing cached attestation path: {cached_attestation_path}"
    );
}

#[test]
fn attest_remote_substitution_workflow_persists_sidecars_and_verifies() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work_dir = tempfile::tempdir().unwrap();
    let local_store_dir = tempfile::tempdir().unwrap();
    let local_state_dir = tempfile::tempdir().unwrap();
    let substitute_store_dir = tempfile::tempdir().unwrap();
    let substitute_state_dir = tempfile::tempdir().unwrap();

    let first = build_simple_ncl_with_nix_compat(
        work_dir.path(),
        local_store_dir.path(),
        local_state_dir.path(),
        "attest-remote-subst",
        true,
    );
    let logical_path = first["outcomes"][0]["outputs"][0]["artifact_attestation"]["logical_path"]
        .as_str()
        .unwrap()
        .to_string();
    let path_info = load_signed_output_pathinfo(local_state_dir.path(), &logical_path, "/nix/store");
    let nar_bytes = render_nar_bytes(local_state_dir.path(), &path_info);
    let remote_cache = FakeBinaryCache::serve(&path_info, nar_bytes);
    let trusted_key = trusted_public_key_arg(local_state_dir.path());
    let ncl_file = work_dir.path().join("attest-remote-subst.ncl");

    let second_output = crunch_cmd()
        .arg("--json")
        .arg("--store")
        .arg(substitute_store_dir.path())
        .arg("--state-dir")
        .arg(substitute_state_dir.path())
        .arg("--nix-compat")
        .arg("build")
        .arg("--substituters")
        .arg(&remote_cache.url)
        .arg("--trusted-public-keys")
        .arg(&trusted_key)
        .arg("-I")
        .arg(work_dir.path())
        .arg(&ncl_file)
        .output()
        .unwrap();
    assert!(
        second_output.status.success(),
        "substitution build failed: {}",
        String::from_utf8_lossy(&second_output.stderr)
    );

    let second: serde_json::Value = serde_json::from_slice(&second_output.stdout).unwrap();
    assert_eq!(second["counts"]["built_total"], 0);
    assert_eq!(second["counts"]["cached_total"], 1);
    assert_eq!(second["outcomes"][0]["cached"], true);
    let output_path = second["outcomes"][0]["outputs"][0]["path"].as_str().unwrap();
    let attestation_path = second["outcomes"][0]["outputs"][0]["artifact_attestation"]["path"].as_str().unwrap();
    assert!(Path::new(output_path).exists(), "missing substituted output path: {output_path}");
    assert!(Path::new(attestation_path).exists(), "missing substituted attestation path: {attestation_path}");
    assert!(std::fs::read_to_string(output_path).unwrap().contains("workflow"));

    let show = crunch_cmd()
        .arg("--store")
        .arg(substitute_store_dir.path())
        .arg("--state-dir")
        .arg(substitute_state_dir.path())
        .arg("--nix-compat")
        .arg("attest")
        .arg("show")
        .arg(output_path)
        .output()
        .unwrap();
    assert!(show.status.success(), "show failed: {}", String::from_utf8_lossy(&show.stderr));
    let show_json: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(show_json["kind"], "artifact");
    assert_eq!(show_json["attestation"]["facts"]["logical_path"], logical_path);

    let verify_artifact = crunch_cmd()
        .arg("--store")
        .arg(substitute_store_dir.path())
        .arg("--state-dir")
        .arg(substitute_state_dir.path())
        .arg("--nix-compat")
        .arg("attest")
        .arg("verify")
        .arg("artifact")
        .arg(output_path)
        .output()
        .unwrap();
    assert!(
        verify_artifact.status.success(),
        "artifact verify failed: {}",
        String::from_utf8_lossy(&verify_artifact.stderr)
    );
    assert!(String::from_utf8(verify_artifact.stdout).unwrap().contains("OK artifact digest="));

    let verify_closure = crunch_cmd()
        .arg("--store")
        .arg(substitute_store_dir.path())
        .arg("--state-dir")
        .arg(substitute_state_dir.path())
        .arg("--nix-compat")
        .arg("attest")
        .arg("verify")
        .arg("closure")
        .arg(output_path)
        .output()
        .unwrap();
    assert!(
        verify_closure.status.success(),
        "closure verify failed: {}",
        String::from_utf8_lossy(&verify_closure.stderr)
    );
    assert!(String::from_utf8(verify_closure.stdout).unwrap().contains("OK closure digest="));
}

#[test]
fn attest_project_workflow_on_built_root() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let work_dir = tempfile::tempdir().unwrap();
    let store_dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let project_dir = tempfile::tempdir().unwrap();

    let report = build_simple_ncl(work_dir.path(), store_dir.path(), state_dir.path(), "attest-project-root");
    let output_path = report["outcomes"][0]["outputs"][0]["path"].as_str().unwrap().to_string();
    write_project_files(project_dir.path(), "sha256-lock-built=");

    let project_output = crunch_cmd()
        .current_dir(project_dir.path())
        .arg("--store")
        .arg(store_dir.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("attest")
        .arg("project")
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(
        project_output.status.success(),
        "project output failed: {}",
        String::from_utf8_lossy(&project_output.stderr)
    );
    let project_file = project_dir.path().join("project.json");
    std::fs::write(&project_file, &project_output.stdout).unwrap();

    let verify = crunch_cmd()
        .current_dir(project_dir.path())
        .arg("--store")
        .arg(store_dir.path())
        .arg("--state-dir")
        .arg(state_dir.path())
        .arg("attest")
        .arg("verify")
        .arg("project")
        .arg("--file")
        .arg(&project_file)
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(verify.status.success(), "project verify failed: {}", String::from_utf8_lossy(&verify.stderr));
}

#[test]
fn attest_show_and_verify_artifact() {
    let seed = seed_store();

    let show = crunch_cmd()
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("show")
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();

    assert!(show.status.success(), "show failed: {}", String::from_utf8_lossy(&show.stderr));
    let show_json: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(show_json["kind"], "artifact");
    assert_eq!(show_json["attestation"]["facts"]["logical_path"], seed.root_logical_path);
    let stored_path = show_json["stored_path"].as_str().unwrap();
    assert!(Path::new(stored_path).exists(), "missing artifact sidecar: {stored_path}");

    let verify = crunch_cmd()
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("verify")
        .arg("artifact")
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();

    assert!(verify.status.success(), "verify failed: {}", String::from_utf8_lossy(&verify.stderr));
    let verify_stdout = String::from_utf8(verify.stdout).unwrap();
    assert!(verify_stdout.contains("OK artifact digest="));
}

#[test]
fn attest_closure_and_verify_runtime_roots() {
    let seed = seed_store();

    let closure = crunch_cmd()
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("closure")
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();

    assert!(closure.status.success(), "closure failed: {}", String::from_utf8_lossy(&closure.stderr));
    let closure_json: serde_json::Value = serde_json::from_slice(&closure.stdout).unwrap();
    assert_eq!(closure_json["kind"], "closure");
    assert_eq!(closure_json["attestation"]["facts"]["members"].as_array().unwrap().len(), 2);

    let verify = crunch_cmd()
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("verify")
        .arg("closure")
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();

    assert!(verify.status.success(), "verify failed: {}", String::from_utf8_lossy(&verify.stderr));
    let verify_stdout = String::from_utf8(verify.stdout).unwrap();
    assert!(verify_stdout.contains("OK closure digest="));
}

#[test]
fn attest_diff_accepts_existing_exported_artifact_selectors() {
    let seed = seed_store();
    assert!(Path::new(&seed.root_exported_path).exists(), "missing exported root: {}", seed.root_exported_path);

    let diff = crunch_cmd()
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("diff")
        .arg(&seed.root_exported_path)
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();

    assert!(diff.status.success(), "diff failed: {}", String::from_utf8_lossy(&diff.stderr));
    assert_eq!(String::from_utf8(diff.stdout).unwrap().trim(), "no differences");
}

#[test]
fn attest_project_verify_and_diff() {
    let seed = seed_store();
    let project_a = tempfile::tempdir().unwrap();
    let project_b = tempfile::tempdir().unwrap();
    write_project_files(project_a.path(), "sha256-lock-a=");
    write_project_files(project_b.path(), "sha256-lock-b=");

    let project_a_output = crunch_cmd()
        .current_dir(project_a.path())
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("project")
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();
    assert!(
        project_a_output.status.success(),
        "project output failed: {}",
        String::from_utf8_lossy(&project_a_output.stderr)
    );
    let project_json: serde_json::Value = serde_json::from_slice(&project_a_output.stdout).unwrap();
    assert_eq!(project_json["kind"], "project");
    assert_eq!(project_json["attestation"]["facts"]["selected_roots"].as_array().unwrap().len(), 1);

    let left_file = project_a.path().join("left.json");
    let right_file = project_b.path().join("right.json");
    std::fs::write(&left_file, &project_a_output.stdout).unwrap();

    let verify = crunch_cmd()
        .current_dir(project_a.path())
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("verify")
        .arg("project")
        .arg("--file")
        .arg(&left_file)
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();
    assert!(verify.status.success(), "project verify failed: {}", String::from_utf8_lossy(&verify.stderr));
    assert!(String::from_utf8(verify.stdout).unwrap().contains("OK project digest="));

    let mut tampered: serde_json::Value = serde_json::from_slice(&project_a_output.stdout).unwrap();
    tampered["digest"] = serde_json::Value::String("deadbeef".repeat(8));
    let tampered_file = project_a.path().join("tampered.json");
    std::fs::write(&tampered_file, serde_json::to_vec_pretty(&tampered).unwrap()).unwrap();
    let verify_bad = crunch_cmd()
        .current_dir(project_a.path())
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("verify")
        .arg("project")
        .arg("--file")
        .arg(&tampered_file)
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();
    assert_eq!(
        verify_bad.status.code(),
        Some(1),
        "tampered verify stderr: {}",
        String::from_utf8_lossy(&verify_bad.stderr)
    );
    assert!(String::from_utf8_lossy(&verify_bad.stderr).contains("digest mismatch"));

    let project_b_output = crunch_cmd()
        .current_dir(project_b.path())
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("project")
        .arg(&seed.root_logical_path)
        .output()
        .unwrap();
    assert!(
        project_b_output.status.success(),
        "project output failed: {}",
        String::from_utf8_lossy(&project_b_output.stderr)
    );
    std::fs::write(&right_file, &project_b_output.stdout).unwrap();

    let diff = crunch_cmd()
        .arg("--store")
        .arg(seed.output_dir.path())
        .arg("--state-dir")
        .arg(seed.state_dir.path())
        .arg("attest")
        .arg("diff")
        .arg(&left_file)
        .arg(&right_file)
        .output()
        .unwrap();

    assert!(diff.status.success(), "diff failed: {}", String::from_utf8_lossy(&diff.stderr));
    let diff_stdout = String::from_utf8(diff.stdout).unwrap();
    assert!(diff_stdout.contains("---"));
    assert!(diff_stdout.contains("+++"));
    assert!(diff_stdout.contains("lockfile_digest"));
}

fn write_project_files(dir: &Path, locked_hash: &str) {
    let manifest = sample_manifest();
    let lock = sample_lock(locked_hash);
    let manifest_text = r#"{
  version = "1.0.0",
  inputs = [
    {
      name = "hello-src",
      kind = { type = "tarball", url = "https://example.invalid/hello.tar.gz" },
      frozen = false,
      mirrors = ["https://mirror.invalid/hello.tar.gz"],
      patches = ["hello-fix"],
    },
  ],
  patches = [
    {
      name = "hello-fix",
      source = { type = "local", path = "patches/hello-fix.patch" },
    },
  ],
}
"#;
    std::fs::write(dir.join("crunch-project.ncl"), manifest_text).unwrap();
    std::fs::write(dir.join("crunch.lock"), lock.to_json().unwrap()).unwrap();
    std::fs::write(dir.join("hello-fix.patch"), "diff --git a/a b/a\n").unwrap();
    assert_eq!(manifest.version, "1.0.0");
}

fn sample_manifest() -> ProjectManifest {
    ProjectManifest {
        version: "1.0.0".to_string(),
        inputs: vec![ManifestInput {
            name: "hello-src".to_string(),
            kind: InputKind::Tarball {
                url: "https://example.invalid/hello.tar.gz".to_string(),
            },
            hash: Default::default(),
            frozen: false,
            mirrors: vec!["https://mirror.invalid/hello.tar.gz".to_string()],
            patches: vec!["hello-fix".to_string()],
        }],
        patches: vec![PatchDef {
            name: "hello-fix".to_string(),
            source: PatchSource::Local {
                path: "patches/hello-fix.patch".to_string(),
            },
        }],
    }
}

fn sample_lock(locked_hash: &str) -> Lockfile {
    let mut inputs = BTreeMap::new();
    inputs.insert("hello-src".to_string(), LockEntry {
        kind: LockedKind::Tarball {
            url: "https://example.invalid/hello.tar.gz".to_string(),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: locked_hash.to_string(),
        },
        patches: vec!["hello-fix".to_string()],
        mirrors: vec!["https://mirror.invalid/hello.tar.gz".to_string()],
    });

    let mut patches = BTreeMap::new();
    patches.insert("hello-fix".to_string(), LockedPatch {
        source: LockedPatchSource::Local {
            path: "patches/hello-fix.patch".to_string(),
        },
        hash: LockedHash {
            algo: HashAlgo::Sha256,
            value: "sha256-patch=".to_string(),
        },
    });

    Lockfile {
        version: SchemaVersion::CURRENT,
        inputs,
        patches,
    }
}
