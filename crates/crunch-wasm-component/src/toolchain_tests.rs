use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use crunch_wasm_component_core::Blake3Identity;

use super::MAX_TOOLCHAIN_MANIFEST_BYTES;
use super::read_version_output_with_limits;
use super::verify_cohort_identity;
use super::verify_toolchain_manifest;
use crate::OCTET_PACKAGE_NAME;
use crate::OCTET_PACKAGE_VERSION;
use crate::OCTET_PROFILE_ID;
use crate::OCTET_SOURCE_REPOSITORY;
use crate::OCTET_SOURCE_REVISION;
use crate::OctetProfileIdentity;
use crate::OctetRailIdentity;
use crate::ToolLimits;
use crate::ToolchainManifest;

const TEST_TIMEOUT_MS: u64 = 200;
const TEST_OUTPUT_BYTES: u64 = 128;

#[test]
fn cohort_identity_matches_sorted_compact_json_with_line_terminator() {
    let octet = test_octet_identity();
    let octet_digest = octet.config_digest_blake3.clone().into_hex();
    let octet_revision = OCTET_SOURCE_REVISION;
    let canonical = format!(
        "{{\"octet\":{{\"config_digest_blake3\":\"{octet_digest}\",\"config_path\":\"share/octet/profiles.json\",\"package_name\":\"cargo-octet\",\"package_version\":\"0.1.0\",\"profile_id\":\"portable-component-baseline\",\"profile_identity_blake3\":\"{octet_digest}\",\"profiles\":[{{\"profile_id\":\"portable-component-baseline\",\"profile_identity_blake3\":\"{octet_digest}\"}}],\"source_repository\":\"https://github.com/OnixResearch/octet\",\"source_revision\":\"{octet_revision}\",\"wasm_tools_cohort_identity_blake3\":\"{octet_digest}\"}},\"rust_target\":\"wasm32-wasip2\",\"schema\":\"mantle-wasm-component-toolchain-v1\",\"tools\":[]}}\n"
    );
    let identity = Blake3Identity::from_slice(canonical.as_bytes());
    let manifest = ToolchainManifest {
        schema: "mantle-wasm-component-toolchain-v1".to_string(),
        rust_target: "wasm32-wasip2".to_string(),
        octet,
        tools: Vec::new(),
        cohort_identity_blake3: identity.clone(),
    };
    let bytes = serde_json::to_vec(&manifest).unwrap();

    verify_cohort_identity(&bytes, &manifest).unwrap();

    let mut tampered = manifest;
    tampered.cohort_identity_blake3 = Blake3Identity::from_slice(b"tampered-cohort");
    assert!(verify_cohort_identity(&bytes, &tampered).is_err());
}

// r[verify mantle.wasm_component.profile_selection]
#[test]
fn profile_selection_uses_declared_identities_only() {
    let mut identity = test_octet_identity();
    let default_identity = identity.profile_identity_blake3.clone();
    assert_eq!(identity.select_profile(&default_identity).expect("default profile").profile_id, OCTET_PROFILE_ID);
    let alternative = Blake3Identity::parse("b".repeat(64)).expect("identity");
    assert!(identity.select_profile(&alternative).is_none());
    identity.profiles.push(OctetProfileIdentity {
        profile_id: "molten-component-baseline".to_string(),
        profile_identity_blake3: alternative.clone(),
    });
    assert_eq!(
        identity.select_profile(&alternative).expect("declared profile").profile_id,
        "molten-component-baseline"
    );
    identity.profiles.clear();
    assert!(identity.select_profile(&default_identity).is_none());
    assert!(identity.select_profile(&alternative).is_none());
}

// r[verify mantle.wasm_component.profile_selection]
#[test]
fn selectable_cohort_rejects_missing_duplicate_and_misbound_profiles() {
    let mut octet = test_octet_identity();
    assert!(super::verify_octet_profiles(&octet).is_err());
    octet.profiles.push(OctetProfileIdentity {
        profile_id: crate::OCTET_MOLTEN_PROFILE_ID.to_string(),
        profile_identity_blake3: Blake3Identity::parse(crate::OCTET_MOLTEN_PROFILE_IDENTITY_BLAKE3.to_string())
            .unwrap(),
    });
    super::verify_octet_profiles(&octet).expect("both distinct declared profiles");
    octet.profiles.swap(0, 1);
    super::verify_octet_profiles(&octet).expect("profile order is not authority");
    let mut invalid = octet.clone();
    invalid.profiles[0].profile_identity_blake3 = invalid.profile_identity_blake3.clone();
    assert!(super::verify_octet_profiles(&invalid).is_err());
    let mut invalid = octet.clone();
    invalid.profiles[0].profile_id = invalid.profile_id.clone();
    assert!(super::verify_octet_profiles(&invalid).is_err());
    let mut invalid = octet.clone();
    invalid.profiles[1].profile_identity_blake3 = Blake3Identity::from_slice(b"wrong-default-profile");
    assert!(super::verify_octet_profiles(&invalid).is_err());
    octet.profiles.push(octet.profiles[0].clone());
    assert!(super::verify_octet_profiles(&octet).is_err());
}

fn test_octet_identity() -> OctetRailIdentity {
    let identity = Blake3Identity::from_slice(b"test-octet-identity");
    OctetRailIdentity {
        source_repository: OCTET_SOURCE_REPOSITORY.to_string(),
        source_revision: OCTET_SOURCE_REVISION.to_string(),
        package_name: OCTET_PACKAGE_NAME.to_string(),
        package_version: OCTET_PACKAGE_VERSION.to_string(),
        config_path: "share/octet/profiles.json".to_string(),
        config_digest_blake3: identity.clone(),
        profile_id: OCTET_PROFILE_ID.to_string(),
        profile_identity_blake3: identity.clone(),
        wasm_tools_cohort_identity_blake3: identity.clone(),
        profiles: vec![OctetProfileIdentity {
            profile_id: OCTET_PROFILE_ID.to_string(),
            profile_identity_blake3: identity,
        }],
    }
}

#[test]
fn toolchain_manifest_rejects_symlink_and_oversize_before_parsing() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("manifest-target.json");
    fs::write(&target, b"{}").unwrap();
    let link = dir.path().join("manifest-link.json");
    symlink(&target, &link).unwrap();
    let error = verify_toolchain_manifest(&link).expect_err("symlink manifest must fail no-follow read");
    assert!(error.to_string().contains("symlink file component"));

    let oversized = dir.path().join("oversized.json");
    let bytes = vec![b' '; usize::try_from(MAX_TOOLCHAIN_MANIFEST_BYTES + 1).unwrap()];
    fs::write(&oversized, bytes).unwrap();
    let error = verify_toolchain_manifest(&oversized).expect_err("oversized manifest must fail before parsing");
    assert!(error.to_string().contains("bounded no-follow regular file"));
}

#[test]
fn version_probe_terminates_hang_and_output_flood() {
    let dir = tempfile::tempdir().unwrap();
    let shell = find_program("sh");
    let sleep = find_program("sleep");
    let hang = write_script(&dir.path().join("hang"), &shell, &format!("{} 30", sleep.display()));
    let limits = ToolLimits {
        timeout_ms: TEST_TIMEOUT_MS,
        output_bytes: TEST_OUTPUT_BYTES,
    };
    let started = std::time::Instant::now();
    let error = read_version_output_with_limits(&hang, limits).expect_err("hanging version probe must time out");
    assert!(error.to_string().contains("timeout-exceeded"));
    assert!(started.elapsed() < Duration::from_secs(5));

    let yes = find_program("yes");
    let flood = write_script(&dir.path().join("flood"), &shell, &format!("exec {} version", yes.display()));
    let error =
        read_version_output_with_limits(&flood, limits).expect_err("flooding version probe must hit output bound");
    assert!(error.to_string().contains("output-limit-exceeded"));
}

fn write_script(path: &Path, shell: &Path, body: &str) -> PathBuf {
    fs::write(path, format!("#!{}\n{body}\n", shell.display())).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(path, permissions).unwrap();
    path.to_path_buf()
}

fn find_program(name: &str) -> PathBuf {
    std::env::split_paths(&std::env::var_os("PATH").expect("test PATH"))
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("test program `{name}` is unavailable"))
}
