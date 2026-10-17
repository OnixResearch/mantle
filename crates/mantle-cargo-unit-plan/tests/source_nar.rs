#[path = "../src/source_nar.rs"]
mod source_nar;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[test]
fn package_slice_digest_matches_nix_nar_bytes() {
    let root = std::env::temp_dir().join(format!("unit-nar-test-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(root.join("src/sub")).unwrap();
    fs::write(root.join("src/sub/lib.rs"), "pub fn answer() -> u32 { 42 }").unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname='nar-test'\nversion='0.1.0'\n").unwrap();
    fs::write(root.join("src/build"), "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(root.join("src/build"), fs::Permissions::from_mode(0o755)).unwrap();
    let expected = Command::new("nix-store").arg("--dump").arg(&root).output().unwrap();
    assert!(expected.status.success(), "nix-store --dump failed: {}", String::from_utf8_lossy(&expected.stderr));
    assert_eq!(source_nar::nar_blake3(&root).unwrap(), blake3::hash(&expected.stdout).to_hex().to_string());
    let copied = std::env::temp_dir().join(format!("unit-nar-copied-{}", std::process::id()));
    if copied.exists() {
        fs::remove_dir_all(&copied).unwrap();
    }
    source_nar::copy_package(&root, &copied, 0).unwrap();
    assert_eq!(source_nar::nar_blake3(&copied).unwrap(), source_nar::nar_blake3(&root).unwrap());
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(copied).unwrap();
}

#[test]
fn package_slice_compiles_nested_target_module_without_copying_cargo_build_artifacts() {
    let root = std::env::temp_dir().join(format!("unit-nar-nested-target-{}", std::process::id()));
    let copied = root.with_extension("copy");
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    if copied.exists() {
        fs::remove_dir_all(&copied).unwrap();
    }
    fs::create_dir_all(root.join("src/target")).unwrap();
    fs::create_dir(root.join("target")).unwrap();
    fs::write(root.join("src/lib.rs"), "mod target; pub fn platform() -> &'static str { target::platform() }\n")
        .unwrap();
    fs::write(root.join("src/target.rs"), "mod apple; pub fn platform() -> &'static str { apple::platform() }\n")
        .unwrap();
    fs::write(root.join("src/target/apple.rs"), "pub fn platform() -> &'static str { \"apple\" }\n").unwrap();
    fs::write(root.join("target/generated.out"), "not source").unwrap();

    source_nar::copy_package(&root, &copied, 0).unwrap();
    assert!(!copied.join("target/generated.out").exists());
    let compile = Command::new("rustc")
        .args(["--edition=2024", "--crate-type=lib", "--emit=metadata"])
        .arg(copied.join("src/lib.rs"))
        .arg("--out-dir")
        .arg(&root)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "nested module missing from package slice: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(copied).unwrap();
}

#[test]
fn source_slice_rejects_symlinks_and_special_files() {
    let root = std::env::temp_dir().join(format!("unit-nar-bad-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink("/etc/passwd", root.join("secret")).unwrap();
    assert!(source_nar::nar_blake3(&root).unwrap_err().contains("unit-plan-source-unsupported"));
    let copied = root.with_extension("copy");
    assert!(source_nar::copy_package(&root, &copied, 0).unwrap_err().contains("unit-plan-source-unsupported"));
    fs::remove_dir_all(root).unwrap();
    if copied.exists() {
        fs::remove_dir_all(copied).unwrap();
    }
}
