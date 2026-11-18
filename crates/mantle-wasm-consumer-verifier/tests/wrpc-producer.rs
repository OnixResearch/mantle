//! Opt-in acceptance against actual producer outputs, never synthetic Wasm.
//! Parent runs these after materializing the immutable package.

mod support;

use std::fs;

use crunch_wasm_component_core::MaterializationBundle;
use mantle_wasm_consumer_verifier::ConsumerVerificationStatus;
use support::PublishedPackage;

struct PublishedFixture {
    package: PublishedPackage,
    client: MaterializationBundle,
    server: MaterializationBundle,
}

impl PublishedFixture {
    fn load() -> Self {
        let package = PublishedPackage::load("MANTLE_WRPC_REFERENCE_BUNDLE");
        let client = package.bundle("client/materialization-bundle.json");
        let server = package.bundle("server/materialization-bundle.json");
        assert_ne!(client.final_portable.digest_blake3, server.final_portable.digest_blake3);
        Self {
            package,
            client,
            server,
        }
    }
}

#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn actual_dual_component_bundles_pass_published_consumer_verifier() {
    let fixture = PublishedFixture::load();
    assert_eq!(fixture.package.status(&fixture.client), ConsumerVerificationStatus::Verified);
    assert_eq!(fixture.package.status(&fixture.server), ConsumerVerificationStatus::Verified);
    let manifest = fixture.package.manifest();
    assert_eq!(manifest["remote_interface"], "wrpc-examples:hello/handler");
    assert_eq!(manifest["source"]["revision"], "aa01409ed38bf7a978570ff567c4475376925df6");
    assert_eq!(manifest["comparison_only"][0], "client/artifacts/runtime-composed.wasm");
}

#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn missing_component_blocks_without_local_guest_rebuild() {
    let fixture = PublishedFixture::load();
    let artifact = fixture.package.path(&fixture.client.final_portable.logical_path);
    fs::remove_file(&artifact).unwrap();
    assert_eq!(fixture.package.status(&fixture.client), ConsumerVerificationStatus::Blocked);
    assert!(!artifact.exists(), "verification must never build missing guests");
}

#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn same_length_component_drift_is_rejected() {
    let fixture = PublishedFixture::load();
    let artifact = fixture.package.path(&fixture.client.final_portable.logical_path);
    let mut bytes = fs::read(&artifact).unwrap();
    assert!(bytes.len() > 8);
    bytes[8] ^= 1;
    fs::write(artifact, bytes).unwrap();
    assert_eq!(fixture.package.status(&fixture.client), ConsumerVerificationStatus::Rejected);
    assert_eq!(fixture.package.status(&fixture.server), ConsumerVerificationStatus::Verified);
}

#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn substituting_the_other_real_component_is_rejected() {
    let fixture = PublishedFixture::load();
    fs::copy(
        fixture.package.path(&fixture.server.final_portable.logical_path),
        fixture.package.path(&fixture.client.final_portable.logical_path),
    )
    .unwrap();
    assert_eq!(fixture.package.status(&fixture.client), ConsumerVerificationStatus::Rejected);
    assert_eq!(fixture.package.status(&fixture.server), ConsumerVerificationStatus::Verified);
}

#[cfg(unix)]
#[test]
#[ignore = "requires the real wrpc-reference-bundle producer output"]
fn local_build_symlink_cannot_replace_a_published_component() {
    let fixture = PublishedFixture::load();
    let artifact = fixture.package.path(&fixture.client.final_portable.logical_path);
    let local = fixture.package.root.join("locally-built.wasm");
    fs::copy(&artifact, &local).unwrap();
    fs::remove_file(&artifact).unwrap();
    std::os::unix::fs::symlink(&local, &artifact).unwrap();
    assert_eq!(fixture.package.status(&fixture.client), ConsumerVerificationStatus::Blocked);
    assert_eq!(fixture.package.status(&fixture.server), ConsumerVerificationStatus::Verified);
}
