use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;
use serde_json::json;

fn tool(name: &str) -> PathBuf {
    for path in std::env::split_paths(&std::env::var_os("PATH").unwrap()) {
        let executable = path.join(name);
        if executable.is_file() {
            return fs::canonicalize(executable).unwrap();
        }
    }
    panic!("missing declared test tool {name}")
}

fn helper(out: &Path, source: &Path, direct: &[&Path], externs: Value, arguments: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"));
    command
        .env("out", out)
        .env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_LINKER", tool("clang"))
        .env("MANTLE_UNIT_SOURCE", source)
        .env(
            "MANTLE_UNIT_DEPENDENCIES",
            serde_json::to_string(&direct.iter().map(|path| path.display().to_string()).collect::<Vec<_>>()).unwrap(),
        )
        .env("MANTLE_UNIT_EXTERNS", externs.to_string())
        .env("MANTLE_UNIT_COMPILE_ENV", "{\"CARGO_PKG_VERSION\":\"0.1.0\"}")
        .env("CARGO_PKG_VERSION", "spoofed")
        .env("CARGO_HOME", "/nonexistent/ambient-cargo-state")
        .args(arguments);
    command.output().unwrap()
}

fn fixture_root(scenario: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mantle-cargo-unit-helper-{}-{scenario}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn three_unit_build_links_transitive_library_through_manifests() {
    let fixture = fixture_root("transitive");
    let store = fixture.join("store");
    let source = store.join("source");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("leaf.rs"), "pub fn answer() -> u32 { env!(\"CARGO_PKG_VERSION\").len() as u32 + 37 }")
        .unwrap();
    fs::write(source.join("middle.rs"), "pub fn answer() -> u32 { leaf::answer() }").unwrap();
    fs::write(source.join("app.rs"), "fn main() { assert_eq!(middle::answer(), 42); }").unwrap();
    let leaf = store.join("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-leaf");
    let middle = store.join("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-middle");
    let app = store.join("cccccccccccccccccccccccccccccccc-app");
    let output = helper(&leaf, &source, &[], json!({}), &[
        "--crate-name=leaf",
        "--crate-type=rlib",
        "--edition=2024",
        source.join("leaf.rs").to_str().unwrap(),
    ]);
    assert!(output.status.success(), "leaf failed: {}", String::from_utf8_lossy(&output.stderr));
    let output = helper(&middle, &source, &[&leaf], json!({"leaf": leaf.display().to_string()}), &[
        "--crate-name=middle",
        "--crate-type=rlib",
        "--edition=2024",
        source.join("middle.rs").to_str().unwrap(),
    ]);
    assert!(output.status.success(), "middle failed: {}", String::from_utf8_lossy(&output.stderr));
    let middle_manifest: Value =
        serde_json::from_slice(&fs::read(middle.join("share/mantle/unit-dependencies-v1.json")).unwrap()).unwrap();
    assert_eq!(middle_manifest["direct_dependencies"], json!([leaf.display().to_string()]));
    let output = helper(&app, &source, &[&middle], json!({"middle": middle.display().to_string()}), &[
        "--crate-name=app",
        "--crate-type=bin",
        "--edition=2024",
        source.join("app.rs").to_str().unwrap(),
    ]);
    assert!(output.status.success(), "app failed: {}", String::from_utf8_lossy(&output.stderr));
    let built = Command::new(app.join("lib/app")).output().unwrap();
    assert!(built.status.success(), "compiled app exited {built:?}");
    let app_manifest: Value =
        serde_json::from_slice(&fs::read(app.join("share/mantle/unit-dependencies-v1.json")).unwrap()).unwrap();
    assert_eq!(app_manifest["direct_dependencies"], json!([middle.display().to_string()]));
    assert_eq!(app_manifest["artifacts"], json!(["lib/app"]));
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn absent_direct_manifest_blocks_before_compilation() {
    let fixture = fixture_root("missing-manifest");
    let source = fixture.join("source");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("app.rs"), "fn main() {}").unwrap();
    let dependency = fixture.join("store/01234567890123456789012345678901-absent");
    fs::create_dir_all(&dependency).unwrap();
    let output = helper(&fixture.join("out"), &source, &[&dependency], json!({}), &[
        "--crate-name=app",
        "--crate-type=bin",
        source.join("app.rs").to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unit-helper-missing-manifest"));
    assert!(!fixture.join("out/lib").exists());
    fs::remove_dir_all(fixture).unwrap();
}
