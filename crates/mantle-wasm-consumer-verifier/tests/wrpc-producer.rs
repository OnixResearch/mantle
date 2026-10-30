//! Opt-in acceptance against actual producer outputs, never synthetic Wasm.
//! Parent runs these after materializing the immutable package.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component_core::MaterializationBundle;
use mantle_wasm_consumer_verifier::ConsumerRoot;
use mantle_wasm_consumer_verifier::ConsumerVerificationStatus;

struct PublishedFixture {
    root: PathBuf,
    client: MaterializationBundle,
    server: MaterializationBundle,
}

impl PublishedFixture {
    fn load() -> Self {
        let package = PathBuf::from(
            std::env::var_os("MANTLE_WRPC_REFERENCE_BUNDLE")
                .expect("set MANTLE_WRPC_REFERENCE_BUNDLE to the built immutable package"),
        );
        assert!(package.starts_with("/nix/store"));
        let root = std::env::temp_dir().join(format!(
            "wrpc-bundle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        copy_tree(&package.join("capability"), &root);
        let client = read_bundle(&package.join("client/materialization-bundle.json"));
        let server = read_bundle(&package.join("server/materialization-bundle.json"));
        assert_ne!(client.final_portable.digest_blake3, server.final_portable.digest_blake3);
        Self { root, client, server }
    }

    fn path(&self, logical: &str) -> PathBuf {
        self.root.join(logical.strip_prefix('/').unwrap())
    }

    fn status(&self, bundle: &MaterializationBundle) -> ConsumerVerificationStatus {
        ConsumerRoot::open(&self.root).unwrap().verify_bundle(bundle.clone()).unwrap().status
    }
}

impl Drop for PublishedFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn read_bundle(path: &Path) -> MaterializationBundle {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn copy_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        assert!(!ty.is_symlink(), "published producer member must not be a symlink");
        if ty.is_dir() {
            copy_tree(&entry.path(), &target.join(entry.file_name()));
        } else {
            let copied = target.join(entry.file_name());
            fs::copy(entry.path(), &copied).unwrap();
            let mode = fs::metadata(&copied).unwrap().permissions().mode();
            fs::set_permissions(copied, fs::Permissions::from_mode(mode | 0o200)).unwrap();
        }
    }
}

#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn actual_dual_component_bundles_pass_published_consumer_verifier() {
    let fixture = PublishedFixture::load();
    assert_eq!(fixture.status(&fixture.client), ConsumerVerificationStatus::Verified);
    assert_eq!(fixture.status(&fixture.server), ConsumerVerificationStatus::Verified);
    let package = PathBuf::from(std::env::var_os("MANTLE_WRPC_REFERENCE_BUNDLE").unwrap());
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(package.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["remote_interface"], "wrpc-examples:hello/handler");
    assert_eq!(manifest["source"]["revision"], "aa01409ed38bf7a978570ff567c4475376925df6");
    assert_eq!(manifest["comparison_only"][0], "client/artifacts/runtime-composed.wasm");
}

#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn missing_component_blocks_without_local_guest_rebuild() {
    let fixture = PublishedFixture::load();
    let artifact = fixture.path(&fixture.client.final_portable.logical_path);
    fs::remove_file(&artifact).unwrap();
    assert_eq!(fixture.status(&fixture.client), ConsumerVerificationStatus::Blocked);
    assert!(!artifact.exists(), "verification must never build missing guests");
}

#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn same_length_component_drift_is_rejected() {
    let fixture = PublishedFixture::load();
    let artifact = fixture.path(&fixture.client.final_portable.logical_path);
    let mut bytes = fs::read(&artifact).unwrap();
    assert!(bytes.len() > 8);
    bytes[8] ^= 1;
    fs::write(artifact, bytes).unwrap();
    assert_eq!(fixture.status(&fixture.client), ConsumerVerificationStatus::Rejected);
    assert_eq!(fixture.status(&fixture.server), ConsumerVerificationStatus::Verified);
}

#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn substituting_the_other_real_component_is_rejected() {
    let fixture = PublishedFixture::load();
    fs::copy(
        fixture.path(&fixture.server.final_portable.logical_path),
        fixture.path(&fixture.client.final_portable.logical_path),
    )
    .unwrap();
    assert_eq!(fixture.status(&fixture.client), ConsumerVerificationStatus::Rejected);
    assert_eq!(fixture.status(&fixture.server), ConsumerVerificationStatus::Verified);
}

#[cfg(unix)]
#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn local_build_symlink_cannot_replace_a_published_component() {
    let fixture = PublishedFixture::load();
    let artifact = fixture.path(&fixture.client.final_portable.logical_path);
    let local = fixture.root.join("locally-built.wasm");
    fs::copy(&artifact, &local).unwrap();
    fs::remove_file(&artifact).unwrap();
    std::os::unix::fs::symlink(&local, &artifact).unwrap();
    assert_eq!(fixture.status(&fixture.client), ConsumerVerificationStatus::Blocked);
    assert_eq!(fixture.status(&fixture.server), ConsumerVerificationStatus::Verified);
}
