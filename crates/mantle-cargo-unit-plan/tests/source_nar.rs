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

#[test]
fn nested_members_and_siblings_are_not_parent_content_but_includes_are() {
    let root = std::env::temp_dir().join(format!("unit-source-boundaries-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let parent = root.join("workspace/parent");
    let nested = parent.join("members/nested");
    let sibling = root.join("workspace/sibling");
    for package in [&parent, &nested, &sibling] {
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(package.join("Cargo.toml"), "[package]\nname='fixture'\nversion='0.1.0'\n").unwrap();
    }
    fs::write(nested.join("src/lib.rs"), "pub const VALUE:u32=1;").unwrap();
    fs::write(sibling.join("secret"), "undeclared").unwrap();
    fs::write(parent.join("src/message.txt"), "included").unwrap();
    fs::write(parent.join("src/payload.bin"), [0, 42, 255]).unwrap();
    fs::write(
        parent.join("src/main.rs"),
        r#"fn main() {
        assert_eq!(include_str!("message.txt"), "included");
        assert_eq!(include_bytes!("payload.bin"), &[0,42,255]);
        println!("included-data=42");
    }"#,
    )
    .unwrap();
    let roots = vec![parent.clone(), nested.clone(), sibling.clone()];
    let first = root.join("slice-first");
    source_nar::copy_package_slice(&parent, &first, &roots).unwrap();
    assert!(!first.join("members/nested").exists());
    assert!(!first.join("../sibling/secret").exists());
    let executable = root.join("consumer");
    let compile = Command::new("rustc").arg(first.join("src/main.rs")).arg("-o").arg(&executable).output().unwrap();
    assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
    let run = Command::new(executable).output().unwrap();
    assert!(run.status.success());
    assert_eq!(run.stdout, b"included-data=42\n");
    fs::write(nested.join("src/lib.rs"), "pub const VALUE:u32=2;").unwrap();
    fs::write(sibling.join("secret"), "changed outside parent").unwrap();
    let second = root.join("slice-second");
    source_nar::copy_package_slice(&parent, &second, &roots).unwrap();
    assert_eq!(source_nar::nar_blake3(&first).unwrap(), source_nar::nar_blake3(&second).unwrap());
    fs::write(parent.join("src/payload.bin"), [0, 43, 255]).unwrap();
    let third = root.join("slice-third");
    source_nar::copy_package_slice(&parent, &third, &roots).unwrap();
    assert_ne!(source_nar::nar_blake3(&first).unwrap(), source_nar::nar_blake3(&third).unwrap());
    fs::write(parent.join("src/message.txt"), "changed included text").unwrap();
    let fourth = root.join("slice-fourth");
    source_nar::copy_package_slice(&parent, &fourth, &roots).unwrap();
    assert_ne!(source_nar::nar_blake3(&third).unwrap(), source_nar::nar_blake3(&fourth).unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn undeclared_package_external_compile_and_build_script_reads_fail_closed() {
    let root = std::env::temp_dir().join(format!("unit-source-external-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let source = root.join("workspace/package");
    fs::create_dir_all(source.join("src")).unwrap();
    fs::create_dir_all(root.join("workspace/sibling")).unwrap();
    fs::write(root.join("workspace/sibling/secret"), "present in checkout only").unwrap();
    fs::write(source.join("src/lib.rs"), "pub const SECRET:&str=include_str!(\"../../sibling/secret\");").unwrap();
    fs::write(source.join("build.rs"), "fn main(){std::fs::read_to_string(\"../sibling/secret\").expect(\"undeclared source input outside package\");}").unwrap();
    let slice = root.join("slice");
    source_nar::copy_package_slice(&source, &slice, &[]).unwrap();
    let compile = Command::new("rustc")
        .arg("--crate-type=lib")
        .arg(slice.join("src/lib.rs"))
        .arg("--out-dir")
        .arg(&root)
        .output()
        .unwrap();
    assert!(!compile.status.success());
    assert!(String::from_utf8_lossy(&compile.stderr).contains("sibling/secret"));
    let script = root.join("build-script");
    let compile_script = Command::new("rustc")
        .arg("--crate-name=build_script_build")
        .arg(slice.join("build.rs"))
        .arg("-o")
        .arg(&script)
        .output()
        .unwrap();
    assert!(compile_script.status.success(), "{}", String::from_utf8_lossy(&compile_script.stderr));
    let run = Command::new(script).current_dir(slice).output().unwrap();
    assert!(!run.status.success());
    assert!(String::from_utf8_lossy(&run.stderr).contains("undeclared source input outside package"));
    fs::remove_dir_all(root).unwrap();
}
