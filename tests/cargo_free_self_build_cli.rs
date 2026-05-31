use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

#[cfg(unix)]
const EXECUTABLE_PERMISSIONS: u32 = 0o755;
const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const DIGEST_E: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const DIGEST_F: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const REQUIRED_TOOLCHAIN_MEMBER_COUNT: u64 = 4;
const BLAKE3_HEX_CHAR_COUNT: usize = DIGEST_A.len();
const FIXED_POINT_SUCCESS_SOURCE: &str = r##"
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "rust-plan") {
        emit_receipt(true, &args);
        return;
    }
    println!("tiny mantle");
}

fn emit_receipt(copy_self: bool, args: &[String]) {
    let output_root = output_root(args);
    let unit_dir = output_root.join("fake-unit");
    fs::create_dir_all(&unit_dir).unwrap();
    let binary = unit_dir.join("mantle");
    if copy_self {
        fs::copy(env::current_exe().unwrap(), &binary).unwrap();
    } else {
        fs::write(&binary, "#!/bin/sh\nexit 0\n").unwrap();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    }
    println!(r#"{{"topology_execution":{{"execution_status":"success","unit_executions":[{{"unit_id":"fake-unit","target_name":"mantle","target_kind":"bin","execution_status":"success","source_digest":{{"algorithm":"blake3","value":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}}}]}}}}"#);
}

fn output_root(args: &[String]) -> PathBuf {
    args.windows(2)
        .find(|pair| pair[0] == "--execution-output-root")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap()
}
"##;
const FIXED_POINT_MISMATCH_SOURCE: &str = r##"
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "rust-plan") {
        emit_receipt(&args);
        return;
    }
    println!("tiny mantle");
}

fn emit_receipt(args: &[String]) {
    let output_root = output_root(args);
    let unit_dir = output_root.join("fake-unit");
    fs::create_dir_all(&unit_dir).unwrap();
    let binary = unit_dir.join("mantle");
    fs::write(&binary, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    }
    println!(r#"{{"topology_execution":{{"execution_status":"success","unit_executions":[{{"unit_id":"fake-unit","target_name":"mantle","target_kind":"bin","execution_status":"success","source_digest":{{"algorithm":"blake3","value":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}}}]}}}}"#);
}

fn output_root(args: &[String]) -> PathBuf {
    args.windows(2)
        .find(|pair| pair[0] == "--execution-output-root")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap()
}
"##;

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn write_tiny_mantle_fixture(dir: &TempDir) -> std::path::PathBuf {
    write_mantle_fixture(dir, "fn main() { println!(\"tiny mantle\"); }\n")
}

fn write_fixed_point_fixture(dir: &TempDir, source: &str) -> std::path::PathBuf {
    write_mantle_fixture(dir, source)
}

fn write_toolchain_closure_manifest(dir: &TempDir, include_sysroot: bool) -> std::path::PathBuf {
    let path = dir.path().join("toolchain-closure.json");
    let mut members = vec![
        toolchain_member("rustc", "rustc", DIGEST_A, DIGEST_B),
        toolchain_member("linker", "ld", DIGEST_B, DIGEST_C),
        toolchain_member("c-compiler", "cc", DIGEST_C, DIGEST_D),
    ];
    if include_sysroot {
        members.push(toolchain_member("sysroot", "sysroot", DIGEST_D, DIGEST_E));
    }
    let manifest = serde_json::json!({
        "schema": "mantle-source-built-toolchain-closure-v1",
        "members": members,
        "seed_exceptions": []
    });
    let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    std::fs::write(&path, bytes).unwrap();
    path
}

fn toolchain_member(role: &str, name: &str, content_digest: &str, receipt_digest: &str) -> Value {
    serde_json::json!({
        "role": role,
        "name": name,
        "execution_path": format!("/toolchain/{name}"),
        "content_digest_blake3": content_digest,
        "trust": "source-built",
        "source": {
            "kind": "tarball",
            "name": format!("{name}-source"),
            "digest_blake3": DIGEST_F
        },
        "build_receipt": {
            "kind": "mantle-rust-topology",
            "name": format!("{name}-receipt"),
            "digest_blake3": receipt_digest
        }
    })
}

fn assert_validated_toolchain_closure(closure: &Value, manifest: &std::path::Path) {
    assert_eq!(closure["status"], "validated-not-enforced");
    assert_eq!(closure["claim"], false);
    assert_eq!(closure["non_claim"], "not-source-built-toolchain-closure");
    assert_eq!(closure["manifest_path"], manifest.to_string_lossy().as_ref());
    assert_eq!(closure["member_count"], REQUIRED_TOOLCHAIN_MEMBER_COUNT);
    assert_eq!(closure["seed_exception_count"], 0);
    assert_eq!(closure["policy_digest_blake3"].as_str().unwrap().len(), BLAKE3_HEX_CHAR_COUNT);
}

fn write_mantle_fixture(dir: &TempDir, source: &str) -> std::path::PathBuf {
    let root = dir.path().join("tiny-mantle");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), source).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"mantle\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"mantle\"\npath = \"src/main.rs\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\nversion = 4\n\n[[package]]\nname = \"mantle\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    root
}

#[cfg(unix)]
fn write_executable(path: &std::path::Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, contents).unwrap();
    let permissions = std::fs::Permissions::from_mode(EXECUTABLE_PERMISSIONS);
    std::fs::set_permissions(path, permissions).unwrap();
}

#[test]
fn cargo_free_self_build_builds_tiny_mantle_fixture() {
    let dir = TempDir::new().unwrap();
    let root = write_tiny_mantle_fixture(&dir);
    let out_dir = dir.path().join("cargo-free-out");

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--out")
        .arg(&out_dir)
        .output()
        .expect("cargo-free self-build CLI should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: Value = serde_json::from_slice(&output.stdout).expect("summary JSON should parse");
    assert_eq!(summary["schema"], "mantle-cargo-free-self-build-v1");
    assert_eq!(summary["status"], "success");
    assert_eq!(summary["cargo_marker_absent"], true);
    assert_eq!(summary["execution_status"], "success");
    assert!(summary["unit_count"].as_u64().unwrap() >= 1);
    assert_eq!(summary["failed_unit_count"], 0);
    assert!(summary["source_digest"].is_object(), "{summary:#?}");
    assert!(out_dir.join("mantle").is_file());
    assert!(out_dir.join("receipt.json").is_file());
    assert!(out_dir.join("meta.json").is_file());
    assert!(!out_dir.join("cargo-was-invoked").exists());
}

#[test]
fn cargo_free_self_build_validates_toolchain_closure_manifest_without_claiming_enforcement() {
    let dir = TempDir::new().unwrap();
    let root = write_tiny_mantle_fixture(&dir);
    let out_dir = dir.path().join("cargo-free-out");
    let manifest = write_toolchain_closure_manifest(&dir, true);

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--out")
        .arg(&out_dir)
        .arg("--toolchain-closure")
        .arg(&manifest)
        .output()
        .expect("cargo-free self-build CLI should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: Value = serde_json::from_slice(&output.stdout).expect("summary JSON should parse");
    assert_eq!(summary["schema"], "mantle-cargo-free-self-build-v1");
    assert_eq!(summary["status"], "success");
    assert_validated_toolchain_closure(&summary["source_built_toolchain_closure"], &manifest);
    assert!(
        summary["non_claims"]
            .as_array()
            .unwrap()
            .contains(&Value::from("not-source-built-toolchain-closure"))
    );
    assert!(out_dir.join("mantle").is_file());
    assert!(!out_dir.join("cargo-was-invoked").exists());
}

#[test]
fn cargo_free_self_build_rejects_invalid_toolchain_closure_manifest() {
    let dir = TempDir::new().unwrap();
    let root = write_tiny_mantle_fixture(&dir);
    let out_dir = dir.path().join("cargo-free-out");
    let manifest = write_toolchain_closure_manifest(&dir, false);

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--out")
        .arg(&out_dir)
        .arg("--toolchain-closure")
        .arg(&manifest)
        .output()
        .expect("cargo-free self-build CLI should run");

    assert!(!output.status.success(), "stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid --toolchain-closure"), "{stderr}");
    assert!(stderr.contains("missing required toolchain role Sysroot"), "{stderr}");
    assert!(!out_dir.join("mantle").exists());
}

#[cfg(unix)]
#[test]
fn cargo_free_self_build_fails_if_ambient_cargo_is_invoked() {
    let dir = TempDir::new().unwrap();
    let root = write_tiny_mantle_fixture(&dir);
    let out_dir = dir.path().join("cargo-free-out");
    let fake_rustc = dir.path().join("rustc-invokes-cargo");
    write_executable(&fake_rustc, "#!/bin/sh\ncargo --version >/dev/null 2>&1\nexit 1\n");

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--out")
        .arg(&out_dir)
        .arg("--rustc")
        .arg(&fake_rustc)
        .output()
        .expect("cargo-free self-build CLI should run");

    assert!(!output.status.success(), "stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    assert!(out_dir.join("cargo-was-invoked").is_file());
    let summary: Value = serde_json::from_slice(&output.stdout).expect("blocked summary JSON should parse");
    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["cargo_marker_absent"], false);
    assert_eq!(summary["blocker"], "cargo guard was invoked");
    assert!(!out_dir.join("mantle").exists());
    assert!(out_dir.join("smoke-stdout.txt").is_file());
    assert!(out_dir.join("smoke-stderr.txt").is_file());
    let smoke_stderr = std::fs::read_to_string(out_dir.join("smoke-stderr.txt")).unwrap();
    assert!(smoke_stderr.contains("not run: blocked before binary"));
}

#[test]
fn cargo_free_fixed_point_builds_tiny_mantle_fixture() {
    let dir = TempDir::new().unwrap();
    let root = write_fixed_point_fixture(&dir, FIXED_POINT_SUCCESS_SOURCE);
    let out_dir = dir.path().join("cargo-free-fixed-point-out");

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--fixed-point")
        .arg("--out")
        .arg(&out_dir)
        .output()
        .expect("cargo-free fixed-point CLI should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: Value = serde_json::from_slice(&output.stdout).expect("summary JSON should parse");
    assert_eq!(summary["schema"], "mantle-cargo-free-fixed-point-proof-v1");
    assert_eq!(summary["status"], "success");
    assert_eq!(summary["fixed_point"], true);
    assert_eq!(summary["stage1"]["success"], true);
    assert_eq!(summary["stage2"]["success"], true);
    assert_eq!(summary["stage1"]["cargo_marker_absent"], true);
    assert_eq!(summary["stage2"]["cargo_marker_absent"], true);
    assert_eq!(summary["stage1"]["binary_blake3"], summary["stage2"]["binary_blake3"]);
    assert!(out_dir.join("stage1/mantle").is_file());
    assert!(out_dir.join("stage2/mantle").is_file());
    assert!(out_dir.join("preflight.json").is_file());
    assert!(out_dir.join("toolchain/compatibility.json").is_file());
    assert!(!out_dir.join("stage1/cargo-was-invoked").exists());
    assert!(!out_dir.join("stage2/cargo-was-invoked").exists());
}

#[test]
fn cargo_free_fixed_point_validates_toolchain_closure_manifest_without_claiming_enforcement() {
    let dir = TempDir::new().unwrap();
    let root = write_fixed_point_fixture(&dir, FIXED_POINT_SUCCESS_SOURCE);
    let out_dir = dir.path().join("cargo-free-fixed-point-out");
    let manifest = write_toolchain_closure_manifest(&dir, true);

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--fixed-point")
        .arg("--out")
        .arg(&out_dir)
        .arg("--toolchain-closure")
        .arg(&manifest)
        .output()
        .expect("cargo-free fixed-point CLI should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let summary: Value = serde_json::from_slice(&output.stdout).expect("summary JSON should parse");
    let closure = &summary["source_built_toolchain_closure"];
    assert_validated_toolchain_closure(closure, &manifest);

    let preflight: Value = serde_json::from_slice(&std::fs::read(out_dir.join("preflight.json")).unwrap()).unwrap();
    assert_eq!(preflight["source_built_toolchain_closure"], *closure);
}

#[test]
fn cargo_free_fixed_point_rejects_invalid_toolchain_closure_manifest() {
    let dir = TempDir::new().unwrap();
    let root = write_fixed_point_fixture(&dir, FIXED_POINT_SUCCESS_SOURCE);
    let out_dir = dir.path().join("cargo-free-fixed-point-out");
    let manifest = write_toolchain_closure_manifest(&dir, false);

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--fixed-point")
        .arg("--out")
        .arg(&out_dir)
        .arg("--toolchain-closure")
        .arg(&manifest)
        .output()
        .expect("cargo-free fixed-point CLI should run");

    assert!(!output.status.success(), "stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid --toolchain-closure"), "{stderr}");
    assert!(stderr.contains("missing required toolchain role Sysroot"), "{stderr}");
    assert!(!out_dir.join("stage1/mantle").exists());
}

#[cfg(unix)]
#[test]
fn cargo_free_fixed_point_fails_if_stage_invokes_cargo() {
    let dir = TempDir::new().unwrap();
    let root = write_fixed_point_fixture(&dir, FIXED_POINT_SUCCESS_SOURCE);
    let out_dir = dir.path().join("cargo-free-fixed-point-out");
    let fake_rustc = dir.path().join("rustc-invokes-cargo");
    write_executable(
        &fake_rustc,
        "#!/bin/sh\ncase \" $* \" in *\" --version \"*) exit 0;; esac\ncargo --version >/dev/null 2>&1\nexit 1\n",
    );

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--fixed-point")
        .arg("--out")
        .arg(&out_dir)
        .arg("--rustc")
        .arg(&fake_rustc)
        .output()
        .expect("cargo-free fixed-point CLI should run");

    assert!(!output.status.success(), "stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    assert!(out_dir.join("stage1/cargo-was-invoked").is_file());
    let summary: Value = serde_json::from_slice(&output.stdout).expect("blocked summary JSON should parse");
    assert_eq!(summary["status"], "blocked");
    assert_eq!(summary["stage1"]["cargo_marker_absent"], false);
    assert_eq!(summary["stage1"]["blocker"], "cargo guard was invoked");
    assert!(out_dir.join("stage1/smoke-stdout.txt").is_file());
    assert!(out_dir.join("stage1/smoke-stderr.txt").is_file());
}

#[test]
fn cargo_free_fixed_point_rejects_inside_source_output_dir() {
    let dir = TempDir::new().unwrap();
    let root = write_fixed_point_fixture(&dir, FIXED_POINT_SUCCESS_SOURCE);
    let out_dir = root.join("target/proof");

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--fixed-point")
        .arg("--out")
        .arg(&out_dir)
        .output()
        .expect("cargo-free fixed-point CLI should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("inside source root"), "{stderr}");
    assert!(stderr.contains("choose /tmp"), "{stderr}");
}

#[test]
fn cargo_free_fixed_point_reports_missing_rustc_toolchain() {
    let dir = TempDir::new().unwrap();
    let root = write_fixed_point_fixture(&dir, FIXED_POINT_SUCCESS_SOURCE);
    let out_dir = dir.path().join("cargo-free-fixed-point-out");
    let missing_rustc = dir.path().join("missing-rustc");

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--fixed-point")
        .arg("--out")
        .arg(&out_dir)
        .arg("--rustc")
        .arg(&missing_rustc)
        .output()
        .expect("cargo-free fixed-point CLI should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("canonicalize rustc"), "{stderr}");
    assert!(stderr.contains("missing-rustc"), "{stderr}");
}

#[test]
fn cargo_free_fixed_point_reports_stage_digest_mismatch() {
    let dir = TempDir::new().unwrap();
    let root = write_fixed_point_fixture(&dir, FIXED_POINT_MISMATCH_SOURCE);
    let out_dir = dir.path().join("cargo-free-fixed-point-out");

    let output = mantle_cmd()
        .current_dir(&root)
        .arg("--json")
        .arg("self-build")
        .arg("--cargo-free")
        .arg("--fixed-point")
        .arg("--out")
        .arg(&out_dir)
        .output()
        .expect("cargo-free fixed-point CLI should run");

    assert!(!output.status.success());
    let summary: Value = serde_json::from_slice(&output.stdout).expect("mismatch summary JSON should parse");
    assert_eq!(summary["status"], "mismatch");
    assert_eq!(summary["fixed_point"], false);
    assert_ne!(summary["stage1"]["binary_blake3"], summary["stage2"]["binary_blake3"]);
    assert_eq!(summary["blocker"], "stage1/stage2 Mantle binary digests differ");
}
