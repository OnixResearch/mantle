use std::collections::BTreeMap;
use std::path::Path;

use assert_cmd::Command;
use crunch_build::generate_keypair;
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
use crunch_store::StoreConfig;
use crunch_store::StoreHandle;
use nix_compat::store_path::StorePath;
use snix_castore::Node;
use snix_castore::SymlinkTarget;
use snix_store::path_info::PathInfo;
use tempfile::TempDir;

const STORE_DIR: &str = "/crunch/store";

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should build")
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
            .persist_and_export_signed_output("out", &root_path, root_info, node.clone(), None, true)
            .await
            .unwrap();
        store
            .persist_and_export_signed_output(
                "out",
                &StorePath::from_name_and_digest_fixed("attest-dep", [1u8; 20]).unwrap(),
                dep_info,
                node,
                None,
                false,
            )
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
