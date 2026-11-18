//! Opt-in acceptance against the actual optional telemetry producer output,
//! never synthetic Wasm. Parent runs these after materializing the package.

mod support;

use std::fs;
use std::path::PathBuf;

use crunch_wasm_component_core::MaterializationBundle;
use mantle_wasm_consumer_verifier::ConsumerVerificationStatus;
use serde_json::json;
use support::PublishedPackage;

struct TelemetryFixture {
    package: PublishedPackage,
    emitter: MaterializationBundle,
}

impl TelemetryFixture {
    fn load() -> Self {
        let package = PublishedPackage::load("MANTLE_WRPC_TELEMETRY_REFERENCE_BUNDLE");
        let emitter = package.bundle("telemetry/materialization-bundle.json");
        Self { package, emitter }
    }

    fn emitter_path(&self) -> PathBuf {
        self.package.path(&self.emitter.final_portable.logical_path)
    }
}

// r[verify materialize_wrpc_telemetry.producer]
#[test]
#[ignore = "requires the real wrpc-telemetry-reference-bundle producer output"]
fn actual_emitter_bundle_passes_published_consumer_verifier() {
    let fixture = TelemetryFixture::load();
    assert_eq!(fixture.package.status(&fixture.emitter), ConsumerVerificationStatus::Verified);
    let manifest = fixture.package.manifest();
    assert_eq!(manifest["source"]["revision"], "e5262a84044aff064c9599eebc05013bb3432795");
    assert_eq!(manifest["telemetry"]["bundle_identity_blake3"], fixture.emitter.bundle_identity_blake3.to_string());
    assert_eq!(manifest["effect_interface"], "wasi:otel/tracing@0.2.0-rc.2");
    // The compiled emitter imports the 0.2.3 clock interface, not the vendored 0.2.0.
    assert_eq!(
        manifest["type_only_dependencies"],
        json!(["wasi:otel/types@0.2.0-rc.2", "wasi:clocks/wall-clock@0.2.3"])
    );
    assert_eq!(
        manifest["comparison_only"],
        json!(["telemetry/artifacts/runtime-composed.wasm", "comparison/sink.wasm"])
    );
}

// r[verify materialize_wrpc_telemetry.producer]
#[test]
#[ignore = "requires the real wrpc-telemetry-reference-bundle producer output"]
fn missing_emitter_blocks_without_local_guest_rebuild() {
    let fixture = TelemetryFixture::load();
    let artifact = fixture.emitter_path();
    fs::remove_file(&artifact).unwrap();
    assert_eq!(fixture.package.status(&fixture.emitter), ConsumerVerificationStatus::Blocked);
    assert!(!artifact.exists(), "verification must never build missing guests");
}

// r[verify materialize_wrpc_telemetry.producer]
#[test]
#[ignore = "requires the real wrpc-telemetry-reference-bundle producer output"]
fn same_length_emitter_drift_is_rejected() {
    let fixture = TelemetryFixture::load();
    let artifact = fixture.emitter_path();
    let mut bytes = fs::read(&artifact).unwrap();
    assert!(bytes.len() > 8);
    bytes[8] ^= 1;
    fs::write(artifact, bytes).unwrap();
    assert_eq!(fixture.package.status(&fixture.emitter), ConsumerVerificationStatus::Rejected);
}

// r[verify materialize_wrpc_telemetry.producer]
#[test]
#[ignore = "requires the real wrpc-telemetry-reference-bundle producer output"]
fn comparison_only_artifacts_cannot_stand_in_for_the_emitter() {
    let fixture = TelemetryFixture::load();
    let manifest = fixture.package.manifest();
    let members = manifest["comparison_only"].as_array().unwrap();
    assert_eq!(members.len(), 2);
    let published = fs::read(fixture.emitter_path()).unwrap();
    for member in members {
        let member = member.as_str().unwrap();
        let comparison = fs::read(fixture.package.package.join(member)).unwrap();
        assert_ne!(comparison, published, "{member} must not be the published emitter");
        fs::write(fixture.emitter_path(), &comparison).unwrap();
        assert_eq!(fixture.package.status(&fixture.emitter), ConsumerVerificationStatus::Rejected, "{member}");
    }
}

// r[verify materialize_wrpc_telemetry.producer]
#[cfg(unix)]
#[test]
#[ignore = "requires the real wrpc-telemetry-reference-bundle producer output"]
fn local_build_symlink_cannot_replace_the_published_emitter() {
    let fixture = TelemetryFixture::load();
    let artifact = fixture.emitter_path();
    let local = fixture.package.root.join("locally-built.wasm");
    fs::copy(&artifact, &local).unwrap();
    fs::remove_file(&artifact).unwrap();
    std::os::unix::fs::symlink(&local, &artifact).unwrap();
    assert_eq!(fixture.package.status(&fixture.emitter), ConsumerVerificationStatus::Blocked);
}
