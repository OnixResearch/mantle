//! Private writable copy of one immutable producer package's capability tree.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component_core::MaterializationBundle;
use mantle_wasm_consumer_verifier::ConsumerRoot;
use mantle_wasm_consumer_verifier::ConsumerVerificationStatus;

pub struct PublishedPackage {
    /// Immutable package named by the parent's environment variable.
    pub package: PathBuf,
    /// Temporary consumer root that negative cases may mutate.
    pub root: PathBuf,
}

impl PublishedPackage {
    pub fn load(variable: &str) -> Self {
        let package = PathBuf::from(
            std::env::var_os(variable).unwrap_or_else(|| panic!("set {variable} to the built immutable package")),
        );
        assert!(package.starts_with("/nix/store"));
        let root = std::env::temp_dir().join(format!(
            "wrpc-bundle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        copy_tree(&package.join("capability"), &root);
        Self { package, root }
    }

    pub fn bundle(&self, member: &str) -> MaterializationBundle {
        serde_json::from_slice(&fs::read(self.package.join(member)).unwrap()).unwrap()
    }

    pub fn manifest(&self) -> serde_json::Value {
        serde_json::from_slice(&fs::read(self.package.join("manifest.json")).unwrap()).unwrap()
    }

    pub fn path(&self, logical: &str) -> PathBuf {
        self.root.join(logical.strip_prefix('/').unwrap())
    }

    pub fn status(&self, bundle: &MaterializationBundle) -> ConsumerVerificationStatus {
        ConsumerRoot::open(&self.root).unwrap().verify_bundle(bundle.clone()).unwrap().status
    }
}

impl Drop for PublishedPackage {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
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
