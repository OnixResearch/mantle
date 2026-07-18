use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::Value;
use sha2::Digest as _;
use tempfile::TempDir;

const EXPECTED_PROVIDER_KIND: &str = "musl.cc-native-reduced-v1";
const EXPECTED_REPORT_FORMAT: &str = "mantle-self-build-source-hydration-v1";
const EXPECTED_SOURCE_BUNDLE_FORMAT: &str = "mantle-source-bundle-v1";
const EXPECTED_VENDOR_FILE: &str = "dep-a/src/lib.rs";
const PACKAGE_CHECKSUM: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WRONG_MANIFEST_BLAKE3: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

struct HydrationFixture {
    _temp: TempDir,
    bundle: PathBuf,
    checkout: PathBuf,
    state: PathBuf,
    manifest_blake3: String,
}

fn mantle() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("mantle"))
}

fn cargo_sha256_hex(bytes: &[u8]) -> String {
    data_encoding::HEXLOWER.encode(&sha2::Sha256::digest(bytes))
}

fn write_payload(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.txt"), b"payload\n").unwrap();
}

fn write_provider_manifest(path: &Path) {
    let manifest = serde_json::json!({
        "schema_version": 1,
        "provider_kind": EXPECTED_PROVIDER_KIND,
        "reduction": {
            "retained_tools": ["cc", "ar"],
            "dropped_components": ["locale-catalogs"]
        },
        "normalized_seed_contract": {
            "target": "x86_64-linux-musl",
            "dynamic_linker": "ld-musl-x86_64.so.1"
        }
    });
    std::fs::write(path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
}

fn write_vendor(vendor: &Path) {
    let package = vendor.join("dep-a");
    let manifest = b"[package]\nname = \"dep-a\"\nversion = \"0.1.0\"\n";
    let lib = b"pub fn dep_a() {}\n";
    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::write(package.join("Cargo.toml"), manifest).unwrap();
    std::fs::write(package.join("src/lib.rs"), lib).unwrap();
    let checksum = serde_json::json!({
        "files": {
            "Cargo.toml": cargo_sha256_hex(manifest),
            "src/lib.rs": cargo_sha256_hex(lib),
        },
        "package": PACKAGE_CHECKSUM,
    });
    std::fs::write(package.join(".cargo-checksum.json"), serde_json::to_vec(&checksum).unwrap()).unwrap();
}

fn write_checkout(checkout: &Path) {
    std::fs::create_dir_all(checkout.join(".cargo")).unwrap();
    std::fs::write(
        checkout.join(".cargo/vendor-config.toml"),
        "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor-deps\"\n",
    )
    .unwrap();
    std::fs::write(
        checkout.join("Cargo.lock"),
        format!(
            "version = 4\n\n[[package]]\nname = \"dep-a\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{PACKAGE_CHECKSUM}\"\n"
        ),
    )
    .unwrap();
}

fn prepare_fixture() -> HydrationFixture {
    let temp = tempfile::tempdir().unwrap();
    let profile = temp.path().join("profile");
    let provider_archive = profile.join("provider-archive");
    let provider_manifest = profile.join("provider.json");
    let vendor = profile.join("vendor-deps");
    let bundle = temp.path().join("self-build-source-bundle.json");
    let checkout = temp.path().join("fresh-clone");
    let state = temp.path().join("fresh-state");
    write_payload(&provider_archive);
    std::fs::create_dir_all(&profile).unwrap();
    write_provider_manifest(&provider_manifest);
    write_vendor(&vendor);
    write_checkout(&checkout);

    let output = mantle()
        .args([
            "--json",
            "source",
            "bundle",
            "bootstrap-profile",
            "--mode",
            "fresh-clone-inputs",
            "--provider-archive",
        ])
        .arg(&provider_archive)
        .arg("--provider-manifest")
        .arg(&provider_manifest)
        .arg("--vendor-deps")
        .arg(&vendor)
        .arg("--to")
        .arg(&bundle)
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let manifest_blake3 = report["manifest_blake3"].as_str().unwrap().to_string();
    assert_eq!(report["format"], "mantle-bootstrap-source-profile-v1");
    assert!(bundle.is_file());

    HydrationFixture {
        _temp: temp,
        bundle,
        checkout,
        state,
        manifest_blake3,
    }
}

#[test]
fn hydrate_self_build_cli_materializes_fresh_clone_and_emits_contracted_report() {
    let fixture = prepare_fixture();
    let output = mantle()
        .arg("--json")
        .arg("--state-dir")
        .arg(&fixture.state)
        .args(["source", "bundle", "hydrate-self-build", "--from"])
        .arg(&fixture.bundle)
        .arg("--expected-manifest-blake3")
        .arg(&fixture.manifest_blake3)
        .arg("--checkout")
        .arg(&fixture.checkout)
        .output()
        .unwrap();

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["format"], EXPECTED_REPORT_FORMAT);
    assert_eq!(report["manifest_blake3"], fixture.manifest_blake3);
    assert_eq!(report["pinned"], true);
    assert!(fixture.checkout.join("vendor-deps").join(EXPECTED_VENDOR_FILE).is_file());

    let verify = mantle()
        .arg("--json")
        .arg("--state-dir")
        .arg(&fixture.state)
        .args(["source", "bundle", "verify", "--from"])
        .arg(&fixture.bundle)
        .arg("--imported")
        .output()
        .unwrap();
    assert!(verify.status.success(), "stderr: {}", String::from_utf8_lossy(&verify.stderr));
    let verify_report: Value = serde_json::from_slice(&verify.stdout).unwrap();
    assert_eq!(verify_report["ready_class"], "ready");
}

#[test]
fn hydrate_self_build_cli_rejects_wrong_external_identity_without_outputs() {
    let fixture = prepare_fixture();
    let output = mantle()
        .arg("--json")
        .arg("--state-dir")
        .arg(&fixture.state)
        .args(["source", "bundle", "hydrate-self-build", "--from"])
        .arg(&fixture.bundle)
        .arg("--expected-manifest-blake3")
        .arg(WRONG_MANIFEST_BLAKE3)
        .arg("--checkout")
        .arg(&fixture.checkout)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("manifest BLAKE3 mismatch"));
    assert!(!fixture.checkout.join("vendor-deps").exists());
    assert!(!fixture.state.exists());
    let bundle: Value = serde_json::from_slice(&std::fs::read(&fixture.bundle).unwrap()).unwrap();
    assert_eq!(bundle["format"], EXPECTED_SOURCE_BUNDLE_FORMAT);
}
