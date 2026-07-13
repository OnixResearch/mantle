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
use crate::ToolLimits;
use crate::ToolchainManifest;

const TEST_TIMEOUT_MS: u64 = 200;
const TEST_OUTPUT_BYTES: u64 = 128;

#[test]
fn cohort_identity_matches_sorted_compact_json_with_line_terminator() {
    let canonical =
        b"{\"rust_target\":\"wasm32-wasip2\",\"schema\":\"mantle-wasm-component-toolchain-v1\",\"tools\":[]}\n";
    let identity = Blake3Identity::from_slice(canonical);
    let manifest = ToolchainManifest {
        schema: "mantle-wasm-component-toolchain-v1".to_string(),
        rust_target: "wasm32-wasip2".to_string(),
        tools: Vec::new(),
        cohort_identity_blake3: identity.clone(),
    };
    let bytes = format!(
        "{{\"tools\":[],\"cohort_identity_blake3\":\"{}\",\"schema\":\"mantle-wasm-component-toolchain-v1\",\"rust_target\":\"wasm32-wasip2\"}}",
        identity.clone().into_hex()
    );

    verify_cohort_identity(bytes.as_bytes(), &manifest).unwrap();

    let mut tampered = manifest;
    tampered.cohort_identity_blake3 = Blake3Identity::from_slice(b"tampered-cohort");
    assert!(verify_cohort_identity(bytes.as_bytes(), &tampered).is_err());
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
